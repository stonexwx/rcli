use std::fs;
use std::io::Read;

use crate::{cli::text::TextSignFormat, get_reader};
use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{ed25519::signature::SignerMut, Signature, SigningKey, VerifyingKey};
pub trait TextSign {
    fn sign<R: Read>(&self, reader: R) -> Result<Vec<u8>>;
}

pub trait TextVerify {
    fn verify<R: Read>(&self, reader: R, signature: &[u8]) -> Result<bool>;
}

pub trait KeyLoader {
    fn load_key(path: &str) -> Result<Self>
    where
        Self: Sized;
}

struct Blake3 {
    key: [u8; 32],
}

struct Ed25519 {
    key: [u8; 32],
}

pub fn process_sign(input: &str, key: &str, format: TextSignFormat) -> Result<String> {
    let mut reader = get_reader(input)?;
    let signature = match format {
        TextSignFormat::Blake3 => {
            let blake3 = Blake3::load_key(key)?;
            blake3.sign(&mut reader)?
        }
        TextSignFormat::Ed25519 => {
            let ed25519 = Ed25519::load_key(key)?;
            ed25519.sign(&mut reader)?
        }
    };
    let sign = URL_SAFE_NO_PAD.encode(signature);
    println!("{}", sign);
    Ok(sign)
}

pub fn process_verify(
    input: &str,
    key: &str,
    signature: &str,
    format: TextSignFormat,
) -> Result<bool> {
    let mut reader = get_reader(input)?;
    let signature = URL_SAFE_NO_PAD.decode(signature)?;
    let signature = signature.as_slice();
    let result = match format {
        TextSignFormat::Blake3 => {
            let blake3 = Blake3::load_key(key)?;
            blake3.verify(&mut reader, signature)?
        }
        TextSignFormat::Ed25519 => {
            let ed25519 = Ed25519::load_key(key)?;
            ed25519.verify(&mut reader, signature)?
        }
    };
    println!("{}", result);
    Ok(result)
}

impl TextSign for Blake3 {
    fn sign<R: Read>(&self, mut reader: R) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let hash = blake3::keyed_hash(&self.key, &buf);

        Ok(hash.as_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    fn verify<R: Read>(&self, mut reader: R, signature: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let hash = blake3::keyed_hash(&self.key, &buf);

        Ok(hash.as_bytes().to_vec() == signature.to_vec())
    }
}

impl TextSign for Ed25519 {
    fn sign<R: Read>(&self, mut reader: R) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let mut key = SigningKey::from_bytes(&self.key);
        let signature: Signature = key.sign(&buf);
        Ok(signature.to_bytes().to_vec())
    }
}

impl TextVerify for Ed25519 {
    fn verify<R: Read>(&self, mut reader: R, signature: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let verifying_key: VerifyingKey = VerifyingKey::from_bytes(&self.key)?;

        let signature = signature.try_into()?;
        let signature = Signature::from_bytes(signature);

        Ok(verifying_key.verify_strict(&buf, &signature).is_ok())
    }
}

impl KeyLoader for Blake3 {
    fn load_key(path: &str) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyLoader for Ed25519 {
    fn load_key(path: &str) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl Blake3 {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = &key[..32];
        let key: [u8; 32] = key.try_into()?;
        Ok(Self::new(key))
    }
}

impl Ed25519 {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = &key[..32];
        let key: [u8; 32] = key.try_into()?;
        Ok(Self::new(key))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_process_sign() {
        let input = "Cargo.toml";
        let key = "/root/rustProject/rcli/fixtures/blake3.txt";
        let format = TextSignFormat::Blake3;

        let sign = process_sign(input, key, format).unwrap();
        let save_path = "/root/rustProject/rcli/fixtures/sign/blake3.sig";
        if !std::path::Path::new("fixtures/sign").exists() {
            std::fs::create_dir("fixtures/sign").unwrap();
        }
        let mut file = std::fs::File::create(save_path).unwrap();
        file.write_all(sign.as_bytes()).unwrap();
        file.flush().unwrap();
    }

    #[test]
    fn test_process_verify() {
        let input = "Cargo.toml";
        let key = "/root/rustProject/rcli/fixtures/blake3.txt";
        let signature_file_path = "/root/rustProject/rcli/fixtures/sign/blake3.sig";
        let format = TextSignFormat::Blake3;
        let mut file = std::fs::File::open(signature_file_path).unwrap();
        let mut signature = String::new();
        file.read_to_string(&mut signature).unwrap();

        assert!(process_verify(input, key, &signature, format).unwrap());
    }

    #[test]
    fn test_process_sign_ed25519() {
        let input = "Cargo.toml";
        let key = "/root/rustProject/rcli/fixtures/blake3.txt";
        let format = TextSignFormat::Ed25519;

        let sign = process_sign(input, key, format).unwrap();
        let save_path = "fixtures/sign/ed25519.sig";
        if !std::path::Path::new("/root/rustProject/rcli/fixtures/sign").exists() {
            std::fs::create_dir("fixtures/sign").unwrap();
        }
        let mut file = std::fs::File::create(save_path).unwrap();
        file.write_all(sign.as_bytes()).unwrap();
        file.flush().unwrap();
    }

    #[test]
    fn test_process_verify_ed25519() {
        let input = "Cargo.toml";
        let key = "/root/rustProject/rcli/fixtures/blake3.txt";
        let signature_file_path = "/root/rustProject/rcli/fixtures/sign/ed25519.sig";
        let format = TextSignFormat::Ed25519;
        let mut file = std::fs::File::open(signature_file_path).unwrap();
        let mut signature = String::new();
        file.read_to_string(&mut signature).unwrap();

        assert!(process_verify(input, key, &signature, format).unwrap());
    }
}
