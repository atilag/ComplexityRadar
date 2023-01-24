use anyhow::Result;
use async_trait;

use octocrab::{models::repos::RepoCommit, Octocrab};
use std::collections::HashMap;
use std::vec::Vec;
use futures::{stream, StreamExt};

fn sort_by_change_frequency(commits: Vec<RepoCommit>) -> HashMap<String, usize> {
    let filename_and_counts: Vec<String> = commits
        .into_iter()
        .map(|repo_commit| repo_commit.files.unwrap_or_default())
        //.map(|files| files)
        .flat_map(|files| files)
        .map(|file| file.filename)
        .collect();

    println!("Print");
    filename_and_counts.iter().for_each(|e| println!("e: {e}"));
    println!("End Print");
    HashMap::new()
}

#[async_trait::async_trait]
trait TopChangedFilesExt {
    async fn get_top_5_changed_files(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<HashMap<String, usize>>;
}

#[async_trait::async_trait]
impl TopChangedFilesExt for Octocrab {
    async fn get_top_5_changed_files(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<HashMap<String, usize>> {
        let commits = self.repos(owner, repo).list_commits().send().await?;
        let commits_stream = stream::iter(commits)
        .collect::<Vec<RepoCommit>>().await;

        let shas = stream::iter(commits_stream)
        // .map(|repo_commit| async {
        //         let c: octocrab::models::repos::Commit =
        //             self.get(repo_commit.url, None::<&()>).await.unwrap();
        //         //println!("c: {}", c.sha.unwrap());
        //        if let sha = Some(c.sha) {
        //         sha
        //        }
        // });
        .map(|repo_commit| async {
            println!("Calling {}", repo_commit.url);
            let commit: octocrab::models::repos::Commit = self.get(repo_commit.url, None::<&()>).await?;
            println!("Got some results!");
            Ok(commit) as Result<octocrab::models::repos::Commit, anyhow::Error>
        })
        .for_each(|commit| async {
            match commit.await {
                 Ok(commit) => match commit.sha {
                    Some(sha) => println!("sha: {}", sha),
                    None => println!("no sha!"),
                 },
                 Err(error) => println!("Error: {}", error),
            }
        })
        .await;

        Ok(HashMap::new())
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
            .get_top_5_changed_files("atilag", "IBM-Quantum-Systems-Exercise")
            .await;

        let expected: HashMap<String, usize> = [
            ("LICENSE".into(), 1),
            ("README.md".into(), 1),
            ("generate-quantum-programs.py".into(), 1),
            ("large_quantum_program_input.json".into(), 1),
            ("quantum_program_input.json".into(), 1),
        ]
        .iter()
        .cloned()
        .collect();

        //assert_eq!(expected, top_5_changed_files.unwrap());
        assert!(false);
    }
}
