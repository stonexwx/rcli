use core::fmt;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use super::{file_check, path_check};

#[derive(Debug, Parser)]
#[enum_dispatch::enum_dispatch(CmdEexector)]
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

impl crate::CmdEexector for TextSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let ret = crate::process_sign(&self.input, &self.key, self.format)?;
        println!("{}", ret);
        Ok(())
    }
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

impl crate::CmdEexector for TextVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let ret = crate::process_verify(&self.input, &self.key, &self.signature, self.format)?;
        println!("{}", ret);
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct TextKeyGenerateOpts {
    #[arg(short, long, default_value = "ed25519")]
    pub format: TextSignFormat,
    #[arg(long, default_value = "keys" , value_parser = path_check)]
    pub path: PathBuf,
}

impl crate::CmdEexector for TextKeyGenerateOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let res = crate::create_key(self.format)?;
        match self.format {
            crate::cli::text::TextSignFormat::Blake3 => {
                if !Path::new(&self.path).exists() {
                    fs::create_dir(&self.path).await?;
                }
                let mut file = File::create(format!("{}/blake3.key", self.path.display())).await?;
                file.write_all(&res[0]).await?;
                file.flush().await?;
            }
            crate::cli::text::TextSignFormat::Ed25519 => {
                if !Path::new(&self.path).exists() {
                    fs::create_dir(&self.path).await?;
                }
                let mut public_file =
                    File::create(format!("{}/ed25519.pub", self.path.display())).await?;
                let mut private_file =
                    File::create(format!("{}/ed25519.priv", self.path.display())).await?;
                public_file.write_all(&res[1]).await?;
                private_file.write_all(&res[0]).await?;
                public_file.flush().await?;
                private_file.flush().await?;
            }
        }
        Ok(())
    }
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
