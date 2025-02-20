mod config;
mod scan;
mod shodan;

use config::{Config, Source};
use shodan::Shodan;

use clap::Parser;
use env_logger::{Env, Target};
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use log::LevelFilter;

use std::io::{self, Stdout};
use std::fs::File;


pub struct Pipe {
    file: File,
    stdout: Stdout,
}

impl Pipe {
    pub fn new(path: &str) -> Result<Pipe, Box<dyn std::error::Error>> {
        Ok(Pipe {
            file: File::options().write(true).create(true).open(path)?,
            stdout: io::stdout(),
        })
    }
}

impl std::io::Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.stdout.write(buf)?;

        self.file.write(buf)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.stdout.flush()?;

        self.file.flush()
    }
}

#[derive(Debug, Parser)]
#[command(name = "lazyscan", version, about, arg_required_else_help = true)]
pub struct Args {
    config: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config = Config::new(&args.config)?;

    let pipe = Pipe::new(&config.general.log)?;

    let logger = env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .target(Target::Pipe(Box::new(pipe)))
        .filter(Some("html5ever::tree_builder"), LevelFilter::Off)
        .build();

    let multi = MultiProgress::new();

    LogWrapper::new(multi.clone(), logger).try_init()?;

    match config.source {
        Source::Shodan { query, modules } => {
            let mut shodan = Shodan::new(modules)?;

            shodan.run(multi, &query)?;
        },
    }

    Ok(())
}


