mod parser;
mod quest;

use crate::quest::generate;
use anyhow::Context;
use env_logger::Env;
use handlebars::Handlebars;
use log::{error, info};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
struct CliArgs {
    /// Quest file
    #[structopt(long)]
    quest: String,

    /// Template file
    #[structopt(long)]
    template_file: Option<String>,
}

fn render_with_template(template_file: String, content: String) -> anyhow::Result<String> {
    let mut file_content = String::new();
    let mut file = File::open(template_file).context("cannot open template file")?;
    file.read_to_string(&mut file_content)?;

    let mut data = BTreeMap::new();
    data.insert("content".to_string(), content);
    let handlebars = Handlebars::new();
    Ok(handlebars.render_template(&file_content, &data)?)
}

fn run(cli_args: CliArgs) -> anyhow::Result<()> {
    let mut file = File::open(cli_args.quest).context("cannot open quest file")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let html_content = generate(content)?;

    let output = match cli_args.template_file {
        Some(template_file) => render_with_template(template_file, html_content)?,
        None => html_content,
    };
    println!("{}", output);
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
