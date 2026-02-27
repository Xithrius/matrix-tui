#![forbid(unsafe_code)]
#![warn(clippy::nursery, clippy::pedantic)]

use clap::Parser;
use color_eyre::Result;

use crate::{app::App, cli::Cli, config::CoreConfig};

mod app;
mod cli;
mod config;
mod events;
mod logging;
mod matrix;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    crate::logging::init()?;

    let args = Cli::parse();
    let config = CoreConfig::new(args.frame_rate)?;

    let terminal = tui::init();
    let app = App::new(&config);
    app.run(terminal).await?;
    tui::restore();

    Ok(())
}
