use core::fmt;
use std::{path::PathBuf, str::FromStr};

use clap::Parser;

use super::{file_check, path_check};

#[derive(Debug, Parser)]
pub enum TextSubCmd {
    #[command(about = "Sign text with a private key / shared key")]
    Sign(TextSignOpts),
    #[command(about = " Verify text with a public key / shared key")]
    Verify(TextVerifyOpts),
    #[command(about = "Generate a new key")]
    Generate(TextKeyGenerateOpts),
}

#[derive(Debug, Parser)]
pub struct TextSignOpts {
    #[arg(short, long, value_parser = file_check,default_value = "-")]
    pub input: String,
    #[arg( long,value_parser = file_check)]
    pub key: String,
    #[arg(long,default_value = "blake3",value_parser =  parse_formate)]
    pub format: TextSignFormat,
}

#[derive(Debug, Parser)]
pub struct TextVerifyOpts {
    #[arg(short, long, value_parser = file_check,default_value = "-")]
    pub input: String,
    #[arg( long,value_parser = file_check)]
    pub key: String,
    #[arg(short, long)]
    pub signature: String,
    #[arg(long,default_value = "blake3",value_parser =  parse_formate)]
    pub format: TextSignFormat,
}

#[derive(Debug, Parser)]
pub struct TextKeyGenerateOpts {
    #[arg(short, long, default_value = "ed25519")]
    pub format: TextSignFormat,
    #[arg(long, default_value = "keys" , value_parser = path_check)]
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum TextSignFormat {
    Blake3,
    Ed25519,
}

impl FromStr for TextSignFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blake3" => Ok(TextSignFormat::Blake3),
            "ed25519" => Ok(TextSignFormat::Ed25519),
            v => anyhow::bail!("Unsupported base64 format: {}", v),
        }
    }
}

impl fmt::Display for TextSignFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TextSignFormat::Blake3 => write!(f, "urlsafe"),
            TextSignFormat::Ed25519 => write!(f, "standard"),
        }
    }
}

fn parse_formate(s: &str) -> Result<TextSignFormat, anyhow::Error> {
    s.parse()
}
