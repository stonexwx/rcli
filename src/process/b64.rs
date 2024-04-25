use std::io::Read;

use base64::{engine::general_purpose::URL_SAFE, prelude::BASE64_STANDARD, Engine as _};

use crate::cli::bas64_opts::Base64FormatType;

fn get_reader(input: &str) -> Result<Box<dyn Read>, anyhow::Error> {
    let reader: Box<dyn Read> = if input == "-" {
        Box::new(std::io::stdin())
    } else {
        Box::new(std::fs::File::open(input)?)
    };
    Ok(reader)
}

pub fn process_encode(input: &str, format: Base64FormatType) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;

    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let encoded = match format {
        Base64FormatType::STANDARD => BASE64_STANDARD.encode(buf),
        Base64FormatType::UrlSafe => URL_SAFE.encode(buf),
    };
    println!("{}", encoded);
    Ok(encoded)
}

pub fn process_decode(input: &str, format: Base64FormatType) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;

    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    buf = buf.trim().to_string();
    let decoded = match format {
        Base64FormatType::UrlSafe => URL_SAFE.decode(buf)?,
        Base64FormatType::STANDARD => BASE64_STANDARD.decode(buf)?,
    };
    let decoded = String::from_utf8(decoded)?;
    println!("{}", decoded);

    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_process_encode() {
        let input = "Cargo.toml";
        let format = Base64FormatType::UrlSafe;
        let encoded = process_encode(input, format).unwrap();
        let save_path = "fixtures/encode_urlsafe/Cargo_toml_b64.txt";
        if !std::path::Path::new("fixtures/encode_urlsafe").exists() {
            std::fs::create_dir("fixtures/encode_urlsafe").unwrap();
        }
        let mut file = std::fs::File::create(save_path).unwrap();
        file.write_all(encoded.as_bytes()).unwrap();
    }

    #[test]
    fn test_process_decode() {
        let input = "fixtures/encode_urlsafe/Cargo_toml_b64.txt";
        let format = Base64FormatType::UrlSafe;
        let decoded = process_decode(input, format).unwrap();
        let save_path = "fixtures/decode_urlsafe/Cargo_toml.txt";
        if !std::path::Path::new("fixtures/decode_urlsafe").exists() {
            std::fs::create_dir("fixtures/decode_urlsafe").unwrap();
        }
        let mut file = std::fs::File::create(save_path).unwrap();
        file.write_all(decoded.as_bytes()).unwrap();
    }
}
