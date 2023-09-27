mod parser;
mod quest;

use crate::quest::generate;
use anyhow::Context;
use env_logger::Env;
use log::{error, info};
use std::fs::File;
use std::io::Read;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
struct CliArgs {
    /// Quest file
    #[structopt(long)]
    quest: String,
}

fn run(cli_args: CliArgs) -> anyhow::Result<()> {
    let mut file = File::open(cli_args.quest).context("cannot open file")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    generate(content)?;
    Ok(())
}

/// Entry point
fn main() -> Result<(), anyhow::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .write_style(if atty::is(atty::Stream::Stdout) {
            env_logger::WriteStyle::Auto
        } else {
            env_logger::WriteStyle::Never
        })
        .format_timestamp(None)
        .init();

    let cli_args = CliArgs::from_args();
    let result = run(cli_args);
    match result {
        Ok(_) => {
            info!("Done");
            Ok(())
        }
        Err(err) => {
            error!("Error: {}", err);
            Err(err)
        }
    }
}
