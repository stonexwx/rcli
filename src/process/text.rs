use std::fs;
use std::io::Read;

use crate::{cli::text::TextSignFormat, get_reader};
use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::SigningKey;

trait TextSign {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

trait TextVerify {
    fn verify<R: Read>(&self, reader: R, signature: &[u8]) -> Result<bool>;
}

struct Blake3 {
    key: [u8; 32],
}

// struct Ed25519Sign {
//     key: SigningKey,
// }
// struct Ed25519Verify {
//     key: SigningKey,
// }

pub fn process_sign(input: &str, key: &str, format: TextSignFormat) -> Result<()> {
    let mut reader = get_reader(input)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    let signature = match format {
        TextSignFormat::Blake3 => {
            let key = fs::read(key)?;
            let key = &key[..32];
            let key: [u8; 32] = key.try_into()?;
            let blake3 = Blake3 { key };
            blake3.sign(&mut reader)?
        }
        TextSignFormat::Ed25519 => todo!(),
    };
    let sign = URL_SAFE_NO_PAD.encode(signature);
    println!("{}", sign);
    Ok(())
}

pub fn process_verify(
    input: &str,
    key: &str,
    signature: &str,
    format: TextSignFormat,
) -> Result<()> {
    let mut reader = get_reader(input)?;
    let signature = URL_SAFE_NO_PAD.decode(signature)?;
    let signature = signature.as_slice();
    let result = match format {
        TextSignFormat::Blake3 => {
            let key = fs::read(key)?;
            let key = &key[..32];
            let key: [u8; 32] = key.try_into()?;
            let blake3 = Blake3 { key };
            blake3.verify(&mut reader, signature)?
        }
        TextSignFormat::Ed25519 => todo!(),
    };
    println!("{}", result);
    Ok(())
}

impl TextSign for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        println!("{:?}", &buf);
        let mut hash = blake3::Hasher::new_keyed(&self.key);
        hash.update(&buf);
        let hasn = hash.finalize();
        println!("{:?}", hasn);
        Ok(hasn.as_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    fn verify<R: Read>(&self, mut reader: R, signature: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        println!("{:?}", &buf);
        let mut hash = blake3::Hasher::new_keyed(&self.key);
        hash.update(&buf);
        let hasn = hash.finalize();
        println!("{:?}", hasn);
        println!("{:?}", signature);
        Ok(hasn.as_bytes().to_vec() == signature.to_vec())
    }
}

// impl TextSign for Ed25519Sign {
//     fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
//         let mut buf = Vec::new();
//         reader.read_to_end(&mut buf)?;
//         let hash = blake3::hash(&buf);
//         let hasn = hash.as_bytes();
//         let mut csprng = OsRng;
//         let signing_key = SigningKey::from_bytes(&self.key);

//         Ok(signature.to_bytes().to_vec())
//     }
// }

// impl TextVerify for Ed25519Verify {
//     fn verify<R: Read>(&self, reader: R, signature: &[u8]) -> Result<bool> {
//         let mut buf = Vec::new();
//         reader.read_to_end(&mut buf)?;
//         let
//     }
// }
