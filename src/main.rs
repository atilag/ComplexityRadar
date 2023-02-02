use clap::Parser;
use anyhow::Result;
use complexity_radar::TopChangedFilesExt;
use octocrab::Octocrab;

#[derive(Parser, Debug)]
#[clap(name = "complexity-radar")]
#[clap(author = env!("CARGO_PKG_AUTHORS"), version = env!("CARGO_PKG_VERSION"), about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct CommandLineArguments {
    /// Owner of the Github repository
    #[clap(short, long)]
    pub owner: String,

    /// Github repository
    #[clap(short, long)]
    pub repo: u16,

    /// Github token
    #[clap(short='t', long="token")]
    pub token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = CommandLineArguments::parse();

    let token = match args.token{
        Some(token) => token,
        None => std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required")
    };

    let octocrab = Octocrab::builder().personal_token(token).build()?;
    octocrab
    .get_top_changed_files(5, "atilag", "IBM-Quantum-Systems-Exercise")
    .await?;


    Ok(())
}
    
