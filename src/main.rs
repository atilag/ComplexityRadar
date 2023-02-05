mod report;

use clap::Parser;
use anyhow::Result;
use complexity_radar::TopChangedFilesExt;
use report::print_report;
use octocrab::Octocrab;

#[derive(Parser, Debug)]
#[clap(name = "complexity-radar")]
#[clap(author = env!("CARGO_PKG_AUTHORS"), version = env!("CARGO_PKG_VERSION"), about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct CommandLineArguments {
    #[clap(short='u', long="github-user")]
    pub github_user: String,

    #[clap(short='r', long="github-repo")]
    pub github_repo: String,

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
    let top_files = octocrab
    .get_top_changed_files(5, &args.github_user, &args.github_repo)
    .await?;

    print_report(&top_files);

    Ok(())
}
    
