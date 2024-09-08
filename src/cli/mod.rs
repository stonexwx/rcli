pub mod bas64_opts;
pub mod csv_opts;
pub mod gen_pass_opts;
pub mod http;
pub mod text;

use std::path::{Path, PathBuf};

use clap::{command, Parser, Subcommand};
use enum_dispatch::enum_dispatch;

pub use self::{bas64_opts::*, csv_opts::*, gen_pass_opts::*, http::*, text::*};

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about = "use csv2json,generate password,encode or decode base64 tools by this cli ",long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Subcommand)]
#[enum_dispatch(CmdEexector)]
pub enum Command {
    #[command(name = "csv", about = "Convert CSV to other formats")]
    Csv(CsvOpts),

    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpts),

    #[command(subcommand)]
    Base64(Base64Cmd),

    #[command(subcommand)]
    Text(TextSubCmd),

    #[command(subcommand)]
    Http(HttpSubCmd),
}

fn file_check(fliename: &str) -> Result<String, anyhow::Error> {
    if fliename == "-" || Path::new(fliename).exists() {
        Ok(fliename.to_string())
    } else {
        anyhow::bail!("File not found: {}", fliename)
    }
}

fn path_check(path: &str) -> Result<String, anyhow::Error> {
    if Path::new(path).exists() {
        Ok(path.to_string())
    } else {
        anyhow::bail!("Path not found: {}", path)
    }
}
fn path_buf_check(path: &str) -> Result<PathBuf, anyhow::Error> {
    if Path::new(path).exists() {
        Ok(path.into())
    } else {
        anyhow::bail!("Path not found: {}", path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_check() {
        assert!(file_check("-").is_ok());
        assert!(file_check("Cargo.toml").is_ok());
        assert!(file_check("not_found").is_err());
    }
}
