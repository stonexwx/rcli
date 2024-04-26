pub mod bas64_opts;
pub mod csv_opts;
pub mod gen_pass_opts;
pub mod text;

use std::path::Path;

use clap::{command, Parser, Subcommand};

use self::{
    bas64_opts::Base64Cmd, csv_opts::CsvOpts, gen_pass_opts::GenPassOpts, text::TextSubCmd,
};

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about = "use csv2json,generate password,encode or decode base64 tools by this cli ",long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(name = "csv", about = "Convert CSV to other formats")]
    Csv(CsvOpts),

    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpts),

    #[command(subcommand)]
    Base64(Base64Cmd),

    #[command(subcommand)]
    Text(TextSubCmd),
}

fn file_check(fliename: &str) -> Result<String, anyhow::Error> {
    if fliename == "-" || Path::new(fliename).exists() {
        Ok(fliename.to_string())
    } else {
        anyhow::bail!("File not found: {}", fliename)
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
