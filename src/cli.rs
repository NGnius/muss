use clap::{Parser};

#[derive(Parser)]
#[clap(author, version)]
#[clap(about = "Music playlist scripting language runtime")]
pub struct CliArgs {
    /// Script to run
    pub file: Option<String>,
    /// Generate m3u8 playlist
    #[clap(short, long)]
    pub playlist: Option<String>,
}

pub fn parse() -> CliArgs {
    CliArgs::parse()
}
