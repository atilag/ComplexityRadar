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

    #[clap(short='n', long="num-rows", default_value_t=5)]
    pub num_rows: usize,

    #[clap(short='t', long="token")]
    pub token: Option<String>,

    
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = CommandLineArguments::parse();

    let token = args.token
        .map_or(
            std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required"),
            |token| token
        );
        

    let octocrab = Octocrab::builder().personal_token(token).build()?;
    let top_files = octocrab
    .get_top_changed_files(args.num_rows, &args.github_user, &args.github_repo)
    .await?;

    print_report(&top_files);

    Ok(())
}
    
