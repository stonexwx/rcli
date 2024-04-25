use anyhow::Ok;
use clap::Parser;
use core::fmt;
use std::str::FromStr;

use super::file_check;

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

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Json,
    Yaml,
    Toml,
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
