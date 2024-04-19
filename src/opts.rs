use core::fmt;
use std::{path::Path, str::FromStr};

use anyhow::Ok;
use clap::{command, Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct CsvOpts {
    #[arg(short, long, value_parser = file_check )]
    pub file: String,

    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(long, value_parser = parse_output_format ,default_value = "json")]
    pub format: OutputFormat,

    #[arg(short, long, default_value_t = ',')]
    pub delimiter: char,

    #[arg(long, default_value_t = true)]
    pub header: bool,
}

#[derive(Debug, Parser)]
pub struct GenPassOpts {
    #[arg(short, long, default_value_t = 16)]
    pub length: u8,

    #[arg(short, long, default_value_t = true)]
    pub uppercase: bool,

    #[arg(long, default_value_t = true)]
    pub lowercase: bool,

    #[arg(short, long, default_value_t = true)]
    pub numbers: bool,

    #[arg(short, long, default_value_t = true)]
    pub symbols: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(name = "csv", about = "Convert CSV to other formats")]
    Csv(CsvOpts),

    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpts),
}
#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about = "Convert CSV to JSON",long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Json,
    Yaml,
    Toml,
}

fn file_check(fliename: &str) -> Result<String, anyhow::Error> {
    if Path::new(fliename).exists() {
        Ok(fliename.to_string())
    } else {
        anyhow::bail!("File not found: {}", fliename)
    }
}

fn parse_output_format(s: &str) -> Result<OutputFormat, anyhow::Error> {
    s.parse()
}

impl From<OutputFormat> for &'static str {
    fn from(f: OutputFormat) -> Self {
        match f {
            OutputFormat::Json => "json",
            OutputFormat::Yaml => "yaml",
            OutputFormat::Toml => "toml",
        }
    }
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "yaml" => Ok(OutputFormat::Yaml),
            "toml" => Ok(OutputFormat::Toml),
            v => anyhow::bail!("Unsupported output format: {}", v),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(*self))
    }
}
