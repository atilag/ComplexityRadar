use anyhow::Result;

use futures::{stream, StreamExt};
use itertools::Itertools;
use octocrab::models::repos::RepoCommit;
pub use octocrab::Octocrab;
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait TopChangedFilesExt {
    async fn get_top_changed_files(
        &self,
        num_of_files: usize,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<(CodeFile, u32)>>;
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct CodeFile {
    pub filename: String,
    //TODO: Maybe use Url ?
    pub file_url: String,
}

#[async_trait::async_trait]
impl TopChangedFilesExt for Octocrab {
    async fn get_top_changed_files(
        &self,
        number_of_files: usize,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<(CodeFile, u32)>> {
        let commits = self.repos(owner, repo).list_commits().send().await?;
        let commits_stream = stream::iter(commits);
        let changed_files: Vec<(CodeFile, u32)> = commits_stream
            .filter_map(|repo_commit| async move {
                self.get(repo_commit.url, None::<&()>).await.ok() as Option<RepoCommit>
            })
            .flat_map(|commit| stream::iter(commit.files))
            .flat_map(|diff_entries| stream::iter(diff_entries))
            .fold(HashMap::new(), |mut changed_files, diff_entry| async move {
                // We want to measure how frequency a filename is changed, instead of how many changes the file has
                // for a specific commit.
                *changed_files
                    .entry(CodeFile {
                        filename: diff_entry.filename,
                        file_url: diff_entry.raw_url.to_string(),
                    })
                    .or_insert(0) += 1;
                changed_files
            })
            .await
            .into_iter()
            .sorted_by(|a, b| b.1.cmp(&a.1))
            .take(number_of_files)
            .collect();

        Ok(changed_files)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn setup(url: Option<&str>) -> Result<Octocrab> {
        let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
        match url {
            Some(url) => Ok(Octocrab::builder()
                .base_uri(url)?
                .personal_token(token)
                .build()?),
            None => Ok(Octocrab::builder().personal_token(token).build()?),
        }
    }

    #[tokio::test]
    async fn get_the_top_5_changed_files() {
        let octocrab = setup(Some("https://github.com/XAMPPRocky/")).unwrap();

        let top_5_changed_files = octocrab
            .get_top_changed_files(5, "atilag", "octocrab")
            .await;

        let expected: Vec<(CodeFile, u32)> = [
            (
                CodeFile {
                    filename: "README.md".into(),
                    file_url: "".to_string(),
                },
                15,
            ),
            (
                CodeFile {
                    filename: "generate-quantum-programs.py".into(),
                    file_url: "".to_string(),
                },
                7,
            ),
            (
                CodeFile {
                    filename: "large_quantum_program_input.json".into(),
                    file_url: "".to_string(),
                },
                4,
            ),
            (
                CodeFile {
                    filename: "quantum_program_input.json".into(),
                    file_url: "".to_string(),
                },
                3,
            ),
            (
                CodeFile {
                    filename: "LICENSE".into(),
                    file_url: "".to_string(),
                },
                1,
            ),
        ]
        .iter()
        .cloned()
        .collect();

        assert_eq!(expected, top_5_changed_files.unwrap());
    }
}
