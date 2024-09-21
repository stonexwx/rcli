use core::fmt;
use std::str::FromStr;

use clap::Parser;

use super::file_check;

#[derive(Debug, Parser)]
#[enum_dispatch::enum_dispatch(CmdEexector)]
pub enum Base64Cmd {
    #[command(name = "encode", about = "Encode base64")]
    Encode(Base64EncodeOpts),
    #[command(name = "decode", about = "Decode base64")]
    Decode(Base64DecodeOpts),
}

#[derive(Debug, Parser)]
pub struct Base64EncodeOpts {
    #[arg(short, long, value_parser = file_check,default_value = "-")]
    pub input: String,
    #[arg( long,default_value = "standard",value_parser = parse_base64_format)]
    pub format: Base64FormatType,
}

impl crate::CmdEexector for Base64EncodeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let ret = crate::process_encode(&self.input, self.format).await?;
        println!("{}", ret);
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct Base64DecodeOpts {
    #[arg(short, long, value_parser = file_check,default_value = "-")]
    pub input: String,
    #[arg( long,default_value = "standard",value_parser = parse_base64_format)]
    pub format: Base64FormatType,
}

impl crate::CmdEexector for Base64DecodeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let ret = crate::process_decode(&self.input, self.format).await?;
        println!("{}", ret);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Base64FormatType {
    UrlSafe,
    STANDARD,
}

impl FromStr for Base64FormatType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "urlsafe" => Ok(Base64FormatType::UrlSafe),
            "standard" => Ok(Base64FormatType::STANDARD),
            v => anyhow::bail!("Unsupported base64 format: {}", v),
        }
    }
}

impl fmt::Display for Base64FormatType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Base64FormatType::UrlSafe => write!(f, "urlsafe"),
            Base64FormatType::STANDARD => write!(f, "standard"),
        }
    }
}

fn parse_base64_format(s: &str) -> Result<Base64FormatType, anyhow::Error> {
    s.parse()
}
