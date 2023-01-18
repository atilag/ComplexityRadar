use std::path::PathBuf;
use std::vec::Vec;
use octocrab::{Octocrab, params::pulls::comments};
use async_trait;
use anyhow::Result;

#[async_trait::async_trait]
trait TopChangedFilesExt {
    async fn get_top_5_changed_files(&self) -> Result<Vec<PathBuf>>;
}


#[async_trait::async_trait]
impl TopChangedFilesExt for Octocrab {
    async fn get_top_5_changed_files(&self) -> Result<Vec<PathBuf>> {
        //let result = self.get("/repos/atilag/IBM-Quantum-Systems-Exercise/commits", None::<&()>).await?;
        let result = self.repos("atilag", "IBM-Quantum-Systems-Exercise")
        .get_content()
        .send()
        .await?;

        Ok(result.items
            .into_iter()
            .map(|content|{
                println!("Content: {:?}", content.path);
                content.path.into()

            })
            .collect::<Vec<PathBuf>>())
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
            .get_top_5_changed_files().await;

        let expected: Vec<PathBuf> = vec![
            "LICENSE".into(),
            "README.md".into(),
            "generate-quantum-programs.py".into(),
            "large_quantum_program_input.json".into(),
            "quantum_program_input.json".into(),
        ];
        assert_eq!(expected, top_5_changed_files.unwrap());
    }
}


