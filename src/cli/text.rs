use core::fmt;
use std::{path::Path, str::FromStr};

use clap::Parser;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use super::{file_check, path_check};

/// `TextSubCmd` 是一个用于保存文本文件子命令的枚举。
/// 它使用 `enum_dispatch` 宏来实现 `CmdEexector` trait。
/// 它包含以下子命令：
/// * `Sign` - 用于签名文本文件。
/// * `Verify` - 用于验证文本文件。
/// * `Generate` - 用于生成新的密钥。
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

/// `TextSignOpts` 是一个用于保存签名文本文件选项的结构体。
///
/// # 字段
///
/// * `input` - 要签名的输入文件。它接受一个短或长参数，并使用 `file_check` 进行验证。默认值为 `-`。
/// * `key` - 用于签名的密钥文件。它接受一个长参数，并使用 `file_check` 进行验证。
/// * `format` - 文本签名的格式。它接受一个长参数，并使用 `parse_format` 进行验证。默认值为 `blake3`。
#[derive(Debug, Parser)]
pub struct TextSignOpts {
    #[arg(short, long, value_parser = file_check,default_value = "-")]
    pub input: String,
    #[arg( long,value_parser = file_check)]
    pub key: String,
    #[arg(long,default_value = "blake3",value_parser =  parse_format)]
    pub format: TextSignFormat,
}

impl crate::CmdEexector for TextSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let ret = crate::process_sign(&self.input, &self.key, self.format)?;
        println!("{}", ret);
        Ok(())
    }
}

/// `TextVerifyOpts` 是一个用于保存验证文本文件选项的结构体。
/// # 字段
/// * `input` - 要验证的输入文件。它接受一个短或长参数，并使用 `file_check` 进行验证。默认值为 `-`。
/// * `key` - 用于验证的密钥文件。它接受一个长参数，并使用 `file_check` 进行验证。
/// * `signature` - 要验证的签名文件。它接受一个长参数。
/// * `format` - 文本签名的格式。它接受一个长参数，并使用 `parse_format` 进行验证。默认值为 `blake3`。

#[derive(Debug, Parser)]
pub struct TextVerifyOpts {
    #[arg(short, long, value_parser = file_check,default_value = "-")]
    pub input: String,
    #[arg( long,value_parser = file_check)]
    pub key: String,
    #[arg(short, long)]
    pub signature: String,
    #[arg(long,default_value = "blake3",value_parser =  parse_format)]
    pub format: TextSignFormat,
}

impl crate::CmdEexector for TextVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let ret = crate::process_verify(&self.input, &self.key, &self.signature, self.format)?;
        println!("{}", ret);
        Ok(())
    }
}

/// `TextKeyGenerateOpts` 是一个用于保存生成新密钥选项的结构体。
/// # 字段
/// * `format` - 生成的密钥的格式。它接受一个长参数，并使用 `parse_format` 进行验证。默认值为 `ed25519`。
/// * `path` - 生成的密钥的路径。它接受一个长参数，并使用 `path_check` 进行验证。默认值为 `keys`。
/// # 示例
/// ```shell
/// # 生成一个新的 ed25519 密钥
/// $ cli text generate
/// # 生成一个新的 blake3 密钥
/// $ cli text generate --format blake3
/// # 生成一个新的 base64 密钥
/// $ cli text generate --format base64
/// # 生成一个新的密钥并将其保存到指定的路径
/// $ cli text generate --path /path/to/keys
/// ```
/// # 注意
/// * 如果指定的路径不存在，将会自动创建。
/// * 生成的密钥将会保存到指定的路径下。
/// * 生成的密钥文件名为 `blake3.key`、`ed25519.pub`、`ed25519.priv` 或 `chacha20.key`。

#[derive(Debug, Parser)]
pub struct TextKeyGenerateOpts {
    #[arg(short, long, default_value = "ed25519")]
    pub format: TextSignFormat,
    #[arg(long, default_value = "keys" , value_parser = path_check)]
    pub path: String,
}

impl crate::CmdEexector for TextKeyGenerateOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let res = crate::create_key(self.format)?;
        match self.format {
            crate::cli::text::TextSignFormat::Blake3 => {
                if !Path::new(&self.path).exists() {
                    fs::create_dir(&self.path).await?;
                }
                let mut file = File::create(format!("{}/blake3.key", self.path)).await?;
                file.write_all(&res[0]).await?;
                file.flush().await?;
            }
            crate::cli::text::TextSignFormat::Ed25519 => {
                if !Path::new(&self.path).exists() {
                    fs::create_dir(&self.path).await?;
                }
                let mut public_file = File::create(format!("{}/ed25519.pub", self.path)).await?;
                let mut private_file = File::create(format!("{}/ed25519.priv", self.path)).await?;
                public_file.write_all(&res[1]).await?;
                private_file.write_all(&res[0]).await?;
                public_file.flush().await?;
                private_file.flush().await?;
            }
            crate::cli::text::TextSignFormat::ChaCha20 => {
                if !Path::new(&self.path).exists() {
                    fs::create_dir(&self.path).await?;
                }
                let mut file = File::create(format!("{}/base64.key", self.path)).await?;
                for key in res.iter() {
                    file.write_all(key).await?;
                }

                file.flush().await?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextSignFormat {
    Blake3,
    Ed25519,
    ChaCha20,
}

impl FromStr for TextSignFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blake3" => Ok(TextSignFormat::Blake3),
            "ed25519" => Ok(TextSignFormat::Ed25519),
            "base64" => Ok(TextSignFormat::ChaCha20),
            v => anyhow::bail!("Unsupported base64 format: {}", v),
        }
    }
}

impl fmt::Display for TextSignFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TextSignFormat::Blake3 => write!(f, "urlsafe"),
            TextSignFormat::Ed25519 => write!(f, "standard"),
            TextSignFormat::ChaCha20 => write!(f, "base64"),
        }
    }
}

fn parse_format(s: &str) -> Result<TextSignFormat, anyhow::Error> {
    s.parse()
}
