use clap::Parser;

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
}

fn main() {

}
    
