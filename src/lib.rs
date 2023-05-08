use anyhow::Result;

use chrono::{Duration, Utc};
use core::hash::{Hash, Hasher};
use futures::stream;
use futures_util::StreamExt;
use itertools::Itertools;
use octocrab::models::repos::RepoCommit;
pub use octocrab::Octocrab;
use std::collections::HashMap;
use std::ops::Sub;

#[async_trait::async_trait]
pub trait TopChangedFilesExt {
    async fn get_top_changed_files(
        &self,
        num_of_files: usize,
        owner: &str,
        repo: &str,
    ) -> Result<HashMap<String, u32>>;
}

#[async_trait::async_trait]
impl TopChangedFilesExt for Octocrab {
    async fn get_top_changed_files(
        &self,
        number_of_files: usize,
        owner: &str,
        repo: &str,
    ) -> Result<HashMap<String, u32>> {
        let commits_stream = self
            .repos(owner, repo)
            .list_commits()
            .since(Utc::now().sub(Duration::days(365)))
            .send()
            .await?
            .into_stream(&self);

        //let commits_stream = stream::iter(commits);
        let changed_files: HashMap<String, u32> = commits_stream
            .filter_map(
                |repo_commit| async move { repo_commit.ok().map(|repo_commit| repo_commit) },
            )
            .filter_map(|repo_commit| async move {
                self.get(repo_commit.url, None::<&()>).await.ok() as Option<RepoCommit>
            })
            .flat_map(|commit| stream::iter(commit.files))
            .flat_map(|diff_entries| stream::iter(diff_entries))
            .fold(
                HashMap::new(),
                |mut iterim_changed_files, diff_entry| async move {
                    // We want to measure how frequency a filename is changed, instead of how many changes the file has
                    // for a specific commit. That's why we count how many commits have changes for a specific file.
                    *iterim_changed_files.entry(diff_entry.filename).or_insert(0) += 1;
                    iterim_changed_files
                },
            )
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

    fn setup() -> Result<Octocrab> {
        let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
        Ok(Octocrab::builder().personal_token(token).build()?)
    }

    #[tokio::test]
    async fn get_the_top_5_changed_files() {
        let octocrab = setup().unwrap();

        let top_5_changed_files = octocrab
            .get_top_changed_files(5, "qiskit", "qiskit-terra")
            .await;

        let expected = HashMap::from([
            ("README.md".into(), 15),
            ("generate-quantum-programs.py".into(), 7),
            ("large_quantum_program_input.json".into(), 4),
            ("quantum_program_input.json".into(), 3),
            ("LICENSE".into(), 1),
        ]);

        assert_eq!(expected, top_5_changed_files.unwrap());
    }
}
