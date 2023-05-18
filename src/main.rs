mod complexity;
mod report;

use anyhow::Result;
use clap::Parser;
use complexity::{compute_cognitive_index, FunctionComplexity, ProgrammingLang};
use complexity_radar::TopChangedFilesExt;
use octocrab::Octocrab;
use report::{print_heat_map_report, print_top_complexities_report};

#[derive(Parser, Debug)]
#[clap(name = "complexity-radar")]
#[clap(author = env!("CARGO_PKG_AUTHORS"), version = env!("CARGO_PKG_VERSION"), about = env!("CARGO_PKG_DESCRIPTION"))]
pub struct CommandLineArguments {
    /// set base url, like: https://your-company.github.com/
    #[clap(short = 'b', long = "base-url")]
    pub base_url: Option<String>,

    #[clap(short = 'u', long = "github-user")]
    pub github_user: String,

    #[clap(short = 'r', long = "github-repo")]
    pub github_repo: String,

    /// Number or files to show in the report
    #[clap(short = 'n', long = "num-rows", default_value_t = 5)]
    pub num_rows: usize,

    /// GitHub token to authenticate against GitHub API
    #[clap(short = 't', long = "token")]
    pub token: Option<String>,

    /// Do not compute complexity, only shows the top modified files of the repo
    #[clap(long)]
    pub heat_map_only: bool,
}

pub struct TopComplexities {
    code_filename: String, /* TODO: Use PathBuf? */
    num_changes: u32,
    function_complexities: Vec<FunctionComplexity>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = CommandLineArguments::parse();

    let token = args.token.map_or(
        std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required"),
        |token| token,
    );

    let octocrab = match args.base_url {
        Some(base_url) => Octocrab::builder()
            .base_uri(base_url)?
            .personal_token(token)
            .build()?,
        _ => Octocrab::builder().personal_token(token).build()?,
    };

    let top_changed_files = octocrab
        .get_top_changed_files(args.num_rows, &args.github_user, &args.github_repo)
        .await?;

    if args.heat_map_only {
        print_heat_map_report(&top_changed_files);
        return Ok(());
    }

    let top_complexities = top_changed_files
        .iter()
        .map(|(code_filename, num_changes)| {
            compute_cognitive_index(ProgrammingLang::Rust, code_filename.into())
                .and_then(|cognitive_complex_indexes| {
                    Ok(TopComplexities {
                        code_filename: code_filename.clone(),
                        num_changes: *num_changes,
                        function_complexities: cognitive_complex_indexes,
                    })
                })
                .map_err(|msg| msg.into())
        })
        .collect::<Vec<Result<TopComplexities, _>>>();

    print_top_complexities_report(&top_complexities);
    Ok(())
}
