use std::path::Path;

use clap::{command, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about = "Convert CSV to JSON",long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(name = "csv", about = "Convert CSV to other formats")]
    Csv(CsvOpts),
}

#[derive(Debug, Parser)]
pub struct CsvOpts {
    #[arg(short, long, value_parser = file_check )]
    pub file: String,

    #[arg(short, long, default_value = "output.json")]
    pub output: String,

    #[arg(short, long, default_value_t = ',')]
    pub delimiter: char,

    #[arg(long, default_value_t = true)]
    pub header: bool,
}

fn file_check(fliename: &str) -> Result<String, &'static str> {
    if Path::new(fliename).exists() {
        Ok(fliename.into())
    } else {
        Err("File does not exist")
    }
}
