use anyhow::Result;

use chrono::{Duration, Utc};
use futures::stream;
use futures_util::StreamExt;
use itertools::Itertools;
use octocrab::models::repos::RepoCommit;
pub use octocrab::Octocrab;
use std::ops::Sub;

//pub type ChangedFileCounts = std::collections::BTreeMap<std::string::String, u32>;
pub type ChangedFileCounts = Vec<(std::string::String, u32)>;

#[async_trait::async_trait]
pub trait TopChangedFilesExt {
    async fn get_top_changed_files(
        &self,
        num_of_files: usize,
        owner: &str,
        repo: &str,
    ) -> Result<ChangedFileCounts>;
}

#[async_trait::async_trait]
impl TopChangedFilesExt for Octocrab {
    async fn get_top_changed_files(
        &self,
        number_of_files: usize,
        owner: &str,
        repo: &str,
    ) -> Result<ChangedFileCounts> {
        let commits_stream = self
            .repos(owner, repo)
            .list_commits()
            .since(Utc::now().sub(Duration::days(365)))
            .send()
            .await?
            .into_stream(&self);

        let changed_files: ChangedFileCounts = commits_stream
            .filter_map(
                |repo_commit| async move { repo_commit.ok().map(|repo_commit| repo_commit) },
            )
            .filter_map(|repo_commit| async move {
                self.get(repo_commit.url, None::<&()>).await.ok() as Option<RepoCommit>
            })
            .flat_map(|commit| stream::iter(commit.files))
            .flat_map(|diff_entries| stream::iter(diff_entries))
            .fold(
                Vec::new(),
                |mut interim_changed_files, diff_entry| async move {
                    // We want to measure how frequency a filename is changed, instead of how many changes the file has
                    // for a specific commit. That's why we count how many commits have changes for a specific file.
                    interim_changed_files
                        .iter_mut()
                        .find(|(filename, _)| *filename == diff_entry.filename)
                        .map(|existing_entry| existing_entry.1 += 1)
                        .or_else(|| Some(interim_changed_files.push((diff_entry.filename, 1))));
                    interim_changed_files
                },
            )
            .await
            .into_iter()
            .sorted_by(|(_, b1), (_, b2)| b2.cmp(b1))
            .take(number_of_files)
            .collect();

        Ok(changed_files)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::fs;
    use std::io;

    use octocrab::models::repos::{RepoCommit, RepoCommitPage};
    use octocrab::models::Author;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use wiremock::{
        matchers::{method, path, path_regex},
        Mock, MockServer, ResponseTemplate,
    };

    #[derive(Serialize, Deserialize)]
    struct FakePage<T> {
        items: Vec<T>,
    }

    pub async fn setup_error_handler(mock_server: &MockServer, message: &str) {
        Mock::given(method("GET"))
            .and(path_regex(".*"))
            .respond_with(ResponseTemplate::new(500).set_body_json(json!( {
                "documentation_url": "",
                "errors": None::<Vec<serde_json::Value>>,
                "message": message,
            })))
            .mount(mock_server)
            .await;
    }

    async fn setup_api(template: ResponseTemplate) -> MockServer {
        let owner = "owner";
        let repo = "repo";
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(format!("/repos/{owner}/{repo}/commits")))
            .respond_with(template)
            .mount(&mock_server)
            .await;
        setup_error_handler(
            &mock_server,
            &format!("GET on /repo/{owner}/{repo}/commits was not received"),
        )
        .await;
        mock_server
    }

    async fn setup(response_template: ResponseTemplate) -> Result<Octocrab> {
        let server = setup_api(response_template).await;
        let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
        Ok(Octocrab::builder().personal_token(token).build()?)
    }

    fn load_responses() -> String {
        fs::read_to_string("data/responses.dat").expect("Could not read test responses file")
    }

    #[tokio::test]
    async fn get_the_top_5_changed_files() {
        let github_response = load_responses();

        let response_template = ResponseTemplate::new(200).set_body_json(github_response);

        let octocrab = setup(response_template).await;

        let top_5_changed_files = octocrab.get_top_changed_files(5, "owner", "repo").await;

        let expected = vec![
            ("README.md".into(), 15),
            ("generate-quantum-programs.py".into(), 7),
            ("large_quantum_program_input.json".into(), 4),
            ("quantum_program_input.json".into(), 3),
            ("LICENSE".into(), 1),
        ];

        assert_eq!(expected, top_5_changed_files.unwrap());
    }
}
