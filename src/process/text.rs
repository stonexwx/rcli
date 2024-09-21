use std::fs;
use std::io::Read;

use crate::{cli::text::TextSignFormat, get_reader, process_gen_pass};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    AeadCore, ChaCha20Poly1305,
};
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

impl KeyLoader for Blake3 {
    fn load_key(path: &str) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = process_gen_pass(32, true, true, true, true)?;
        let key = key.into_bytes();
        Ok(vec![key])
    }
}

struct Ed25519 {
    key: [u8; 32],
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

struct ChaCha20 {
    key: [u8; 32],
    nonce: [u8; 12],
}

impl ChaCha20 {
    pub fn new(key: [u8; 32], nonce: [u8; 12]) -> Self {
        Self { key, nonce }
    }

    pub fn try_new(key: &[u8], nonce: &[u8]) -> Result<Self> {
        let key = &key[..32];
        let key: [u8; 32] = key.try_into()?;
        let nonce = &nonce[..12];
        let nonce: [u8; 12] = nonce.try_into()?;
        Ok(Self::new(key, nonce))
    }
}

impl TextSign for ChaCha20 {
    fn sign<R: Read>(&self, mut reader: R) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let key = chacha20poly1305::Key::from_slice(&self.key[..32]);
        let nonce = chacha20poly1305::Nonce::from_slice(&self.nonce);
        let cipher = ChaCha20Poly1305::new(key);
        let ciphertext = cipher
            .encrypt(nonce, buf.as_ref())
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(ciphertext)
    }
}

impl TextVerify for ChaCha20 {
    fn verify<R: Read>(&self, mut reader: R, signature: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let key = chacha20poly1305::Key::from_slice(&self.key[..32]);
        let nonce = chacha20poly1305::Nonce::from_slice(&self.nonce);
        let cipher = ChaCha20Poly1305::new(key);
        let plaintext = cipher
            .decrypt(nonce, signature.as_ref())
            .map_err(|e| anyhow::anyhow!(e))?;
        println!("{}", String::from_utf8_lossy(&plaintext));
        Ok(true)
    }
}

impl KeyLoader for ChaCha20 {
    fn load_key(path: &str) -> Result<Self> {
        let read_data = fs::read(path)?;
        let key = &read_data[..32];
        let nonce = &read_data[32..];
        Self::try_new(key, nonce)
    }
}

impl KeyGenerator for ChaCha20 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = ChaCha20Poly1305::generate_key(&mut chacha20poly1305::aead::OsRng);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut chacha20poly1305::aead::OsRng);
        let key = key.as_slice().into();
        let nonce = nonce.as_slice().into();
        Ok(vec![key, nonce])
    }
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
        TextSignFormat::ChaCha20 => {
            let chacha20 = ChaCha20::load_key(key)?;
            chacha20.sign(&mut reader)?
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
        TextSignFormat::ChaCha20 => {
            let chacha20 = ChaCha20::load_key(key)?;
            chacha20.verify(&mut reader, signature)?
        }
    };
    println!("{}", result);
    Ok(result)
}

pub fn create_key(format: TextSignFormat) -> Result<Vec<Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519::generate(),
        TextSignFormat::ChaCha20 => ChaCha20::generate(),
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

    fn test_create_key_base64() {
        let format = TextSignFormat::ChaCha20;
        let key = create_key(format).unwrap();
        let save_path = get_fixture_path("base64.key");
        let mut file = std::fs::File::create(save_path).unwrap();
        file.write_all(&key[0]).unwrap();
        file.write_all(&key[1]).unwrap();
        file.flush().unwrap();
    }

    fn test_process_sign_base64() {
        let input = "Cargo.toml";
        let binding = get_fixture_path("base64.key");
        let key = binding.to_str().unwrap();
        let format = TextSignFormat::ChaCha20;
        let save_file_path = get_fixture_path("sign/chacha20.sig");
        let sign = process_sign(input, key, format).unwrap();
        let save_path = get_fixture_path("sign");
        if !std::path::Path::new(&save_path).exists() {
            std::fs::create_dir(save_path).unwrap();
        }
        let mut file = std::fs::File::create(save_file_path).unwrap();
        file.write_all(sign.as_bytes()).unwrap();
        file.flush().unwrap();
    }

    fn test_process_verify_base64() {
        let input = "Cargo.toml";
        let binding = get_fixture_path("base64.key");
        let key = binding.to_str().unwrap();
        let signature_file_path = get_fixture_path("sign/chacha20.sig");
        let format = TextSignFormat::ChaCha20;
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
        test_create_key_base64();
        test_process_sign_base64();
        test_process_verify_base64();
    }
}
