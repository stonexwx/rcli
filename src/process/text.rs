use std::fs;
use std::io::Read;

use crate::{cli::text::TextSignFormat, get_reader, process_gen_pass};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{
    ed25519::signature::SignerMut, SecretKey, Signature, SigningKey, VerifyingKey,
};

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

pub trait KeyGenerator {
    fn generate() -> Result<Vec<Vec<u8>>>;
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

pub fn create_key(format: TextSignFormat) -> Result<Vec<Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519::generate(),
    }
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

impl KeyGenerator for Ed25519 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let secret_key =
            process_gen_pass(32, true, true, true, true).context("Failed to generate key")?;
        let secret_key: SecretKey = secret_key.as_bytes().try_into().context("Invalid key")?;
        let signing_key = SigningKey::from_bytes(&secret_key);
        let verifying_key = signing_key.verifying_key();
        let verifying_key = verifying_key.to_bytes().to_vec();
        let secret_key = secret_key.to_vec();
        Ok(vec![secret_key, verifying_key])
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = process_gen_pass(32, true, true, true, true)?;
        let key = key.into_bytes();
        Ok(vec![key])
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
    use std::{env, io::Write, path::PathBuf};

    use super::*;

    fn get_fixture_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        path.push("fixtures");
        path.push(filename);
        path
    }

    fn test_create_key() {
        let format = TextSignFormat::Blake3;
        let key = create_key(format).unwrap();
        let save_path = get_fixture_path("blake3.txt");
        let mut file = std::fs::File::create(save_path).unwrap();
        file.write_all(&key[0]).unwrap();
        file.flush().unwrap();
    }

    fn test_create_key_ed25519() {
        let format = TextSignFormat::Ed25519;
        let key = create_key(format).unwrap();
        let public_path = get_fixture_path("ed25519.pub");
        let privete_path = get_fixture_path("ed25519.priv");
        let mut public_file = std::fs::File::create(public_path).unwrap();
        public_file.write_all(&key[1]).unwrap();
        public_file.flush().unwrap();
        let mut private_file = std::fs::File::create(privete_path).unwrap();
        private_file.write_all(&key[0]).unwrap();
        private_file.flush().unwrap();
    }

    fn test_process_sign() {
        let input = "cliff.toml";
        let binding = get_fixture_path("blake3.txt");
        let key = binding.to_str().unwrap();
        let format = TextSignFormat::Blake3;

        let sign = process_sign(input, key, format).unwrap();
        let save_file_path = get_fixture_path("sign/blake3.sig");
        let save_path = get_fixture_path("sign");
        if !std::path::Path::new(&save_path).exists() {
            std::fs::create_dir(save_path).unwrap();
        }
        let mut file = std::fs::File::create(save_file_path).unwrap();
        file.write_all(sign.as_bytes()).unwrap();
        file.flush().unwrap();
    }

    fn test_process_verify() {
        let input = "cliff.toml";
        let binding = get_fixture_path("blake3.txt");
        let key = binding.to_str().unwrap();
        let signature_file_path = get_fixture_path("sign/blake3.sig");
        let format = TextSignFormat::Blake3;
        let mut file = std::fs::File::open(signature_file_path).unwrap();
        let mut signature = String::new();
        file.read_to_string(&mut signature).unwrap();

        assert!(process_verify(input, key, &signature, format).unwrap());
    }

    fn test_process_sign_ed25519() {
        let input = "Cargo.toml";
        let binding = get_fixture_path("ed25519.priv");
        let key = binding.to_str().unwrap();
        let format = TextSignFormat::Ed25519;
        let save_file_path = get_fixture_path("sign/ed25519.sig");
        let sign = process_sign(input, key, format).unwrap();
        let save_path = get_fixture_path("sign");
        if !std::path::Path::new(&save_path).exists() {
            std::fs::create_dir(save_path).unwrap();
        }
        let mut file = std::fs::File::create(save_file_path).unwrap();
        file.write_all(sign.as_bytes()).unwrap();
        file.flush().unwrap();
    }

    fn test_process_verify_ed25519() {
        let input = "Cargo.toml";
        let binding = get_fixture_path("ed25519.pub");
        let key = binding.to_str().unwrap();
        let signature_file_path = get_fixture_path("sign/ed25519.sig");
        let format = TextSignFormat::Ed25519;
        let mut file = std::fs::File::open(signature_file_path).unwrap();
        let mut signature = String::new();
        file.read_to_string(&mut signature).unwrap();

        assert!(process_verify(input, key, &signature, format).unwrap());
    }

    #[test]
    fn test_text_sign() {
        test_create_key();
        test_process_sign();
        test_process_verify();
        test_create_key_ed25519();
        test_process_sign_ed25519();
        test_process_verify_ed25519();
    }
}
