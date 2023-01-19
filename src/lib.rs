use std::collections::HashMap;
use itertools::Itertools;
use std::vec::Vec;
use octocrab::{Octocrab, models::repos::RepoCommit};
use async_trait;
use anyhow::Result;

fn sort_by_change_frequency(commits: Vec<RepoCommit>) ->  HashMap<String, usize> {
    let commit_total_change: Vec<String> = commits.into_iter()
    .filter(|commit| commit.files.is_some())
    .flat_map(|commit| commit.files.unwrap_or_default())    
    .group_by(|file| file.filename.clone())
    .into_iter()
    .map(|file| file.0)
    .collect();

    println!("Hash map: {:#?}", commit_total_change);
    HashMap::new()
}


#[async_trait::async_trait]
trait TopChangedFilesExt {
    async fn get_top_5_changed_files(&self, owner: &str, repo: &str) -> Result<HashMap<String, usize>>;
}


#[async_trait::async_trait]
impl TopChangedFilesExt for Octocrab {
    async fn get_top_5_changed_files(&self, owner: &str, repo: &str) -> Result<HashMap<String, usize>> {
        let commits = self.repos(owner, repo)
        .list_commits()
        .send()
        .await?;

        let commits: Vec<RepoCommit> = commits.items
        .into_iter()
        .map(|repo_commit|{
            println!("{:#?}", repo_commit);
            repo_commit
        })
        .collect();
        Ok(sort_by_change_frequency(commits))
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

        let top_5_changed_files = octocrab::instance()
            .get_top_5_changed_files("atilag", "IBM-Quantum-Systems-Exercise").await;

        let expected: HashMap<String, usize> = [
            ("LICENSE".into(), 1),
            ("README.md".into(),1),
            ("generate-quantum-programs.py".into(),1),
            ("large_quantum_program_input.json".into(),1),
            ("quantum_program_input.json".into(),1),
        ]
        .iter()
        .cloned()
        .collect();


        //assert_eq!(expected, top_5_changed_files.unwrap());
        assert!(false);
    }
}


