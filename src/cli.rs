use clap::Parser;

#[derive(Parser)]
#[clap(author, version)]
#[clap(about = "Music playlist scripting language runtime")]
pub struct CliArgs {
    /// Script to run
    pub file: Option<String>,

    /// Generate m3u8 playlist
    #[clap(short, long)]
    pub playlist: Option<String>,

    /// In REPL mode, wait for all music in the queue to complete before accepting new input
    #[clap(long)]
    pub wait: bool,

    /// In REPL mode, the prompt to display
    #[clap(long, default_value_t = String::from("|"))]
    pub prompt: String,

    /// The volume at which to playback audio, out of 1.0
    #[clap(long)]
    pub volume: Option<f32>,
}

pub fn parse() -> CliArgs {
    CliArgs::parse()
}
