use clap::Parser;

#[derive(Parser)]
#[clap(author, version)]
#[clap(about = "MPS m3u8 generator")]
pub struct CliArgs {
    /// Input script file
    pub input: String,

    /// Output m3u8 playlist
    pub playlist: String,

    /// Parse input as script instead of as filename
    #[clap(long)]
    pub raw: bool,
}

pub fn parse() -> CliArgs {
    CliArgs::parse()
}
