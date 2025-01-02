mod crawler;
mod config;
mod scan;

use crawler::Crawler;
use config::{Config, Source};

use clap::Parser;
use indicatif_log_bridge::LogWrapper;
use indicatif::MultiProgress;
use env_logger::Env;
use log::LevelFilter;


#[derive(Debug, Parser)]
#[command(name = "lazyscan", version, about, arg_required_else_help = true)]
pub struct Args {
    config: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .filter(Some("html5ever::tree_builder"), LevelFilter::Off)
        .build();

    let multi = MultiProgress::new();

    LogWrapper::new(multi.clone(), logger).try_init()?;

    let args = Args::parse();

    let config = Config::new(&args.config)?;

    match &config.source {
        Source::File { path } => {
        },
        Source::Shodan { query } => {
        },
        Source::Crawler { queue, seeds } => {
            let crawler = Crawler::new(&config, queue.clone(), seeds.clone())?;

            crawler.run(multi)?;
        },
    }

    Ok(())
}


