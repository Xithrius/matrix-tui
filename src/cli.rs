use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, about)]
pub struct Cli {
    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, default_value_t = 60)]
    pub frame_rate: u64,
}
