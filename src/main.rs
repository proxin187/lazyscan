mod crawler;
mod config;
mod scan;

use crawler::Crawler;
use config::Config;

use clap::Parser;
use indicatif_log_bridge::LogWrapper;
use indicatif::MultiProgress;
use env_logger::Env;


#[derive(Debug, Parser)]
#[command(name = "lazyscan", version, about, arg_required_else_help = true)]
pub struct Args {
    config: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = env_logger::Builder::from_env(Env::default().default_filter_or("info")).build();
    let multi = MultiProgress::new();

    LogWrapper::new(multi.clone(), logger).try_init()?;

    let args = Args::parse();

    let config = Config::new(&args.config)?;

    let crawler = Crawler::new(config)?;

    crawler.run(multi)
}

