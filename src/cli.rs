use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, about)]
pub struct Cli {
    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
    pub frame_rate: f64,
}
