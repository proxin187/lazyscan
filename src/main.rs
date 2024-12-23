mod crawler;
mod config;
mod scan;

use config::Config;

use clap::Parser;


#[derive(Debug, Parser)]
#[command(name = "lazyscan", version, about, arg_required_else_help = true)]
pub struct Args {
    config: String,
    seed: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config = Config::new(&args.config)?;

    println!("config: {:?}", config);

    Ok(())
}
