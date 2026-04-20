use anyhow::{Context, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{AeadInPlace, KeyInit, XChaCha20Poly1305, aead::Aead};
use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};

const NONCE_LEN: usize = 24;
const KEY_LEN: usize = 32;
const SALT_LEN: usize = 16;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedBlob {
  pub salt_base64: String,
  pub nonce_base64: String,
  pub ciphertext_base64: String,
}

pub fn encrypt(plaintext: &[u8], passphrase: &str) -> Result<EncryptedBlob> {
  let mut salt = [0u8; SALT_LEN];
  OsRng.fill_bytes(&mut salt);
  let mut key = [0u8; KEY_LEN];
  derive_key(passphrase.as_bytes(), &salt, &mut key)?;

  let mut nonce_bytes = [0u8; NONCE_LEN];
  OsRng.fill_bytes(&mut nonce_bytes);

  let cipher = XChaCha20Poly1305::new_from_slice(&key).map_err(|error| anyhow::anyhow!("chacha key init: {error}"))?;
  let mut buffer: Vec<u8> = plaintext.to_vec();
  cipher.encrypt_in_place((&nonce_bytes).into(), b"", &mut buffer).map_err(|error| anyhow::anyhow!("chacha encrypt: {error}"))?;

  use base64::Engine as _;
  Ok(EncryptedBlob {
    salt_base64: base64::engine::general_purpose::STANDARD.encode(salt),
    nonce_base64: base64::engine::general_purpose::STANDARD.encode(nonce_bytes),
    ciphertext_base64: base64::engine::general_purpose::STANDARD.encode(&buffer),
  })
}

pub fn decrypt(blob: &EncryptedBlob, passphrase: &str) -> Result<Vec<u8>> {
  use base64::Engine as _;
  let salt = base64::engine::general_purpose::STANDARD.decode(&blob.salt_base64).context("decode salt")?;
  let nonce = base64::engine::general_purpose::STANDARD.decode(&blob.nonce_base64).context("decode nonce")?;
  let ciphertext = base64::engine::general_purpose::STANDARD.decode(&blob.ciphertext_base64).context("decode ciphertext")?;

  if salt.len() != SALT_LEN || nonce.len() != NONCE_LEN {
    return Err(anyhow::anyhow!("invalid salt or nonce length"));
  }
  let mut key = [0u8; KEY_LEN];
  derive_key(passphrase.as_bytes(), &salt, &mut key)?;
  let cipher = XChaCha20Poly1305::new_from_slice(&key).map_err(|error| anyhow::anyhow!("chacha key init: {error}"))?;
  let nonce_arr: [u8; NONCE_LEN] = nonce.as_slice().try_into().context("nonce length")?;
  let plaintext = cipher.decrypt((&nonce_arr).into(), ciphertext.as_slice()).map_err(|error| anyhow::anyhow!("chacha decrypt: {error}"))?;
  Ok(plaintext)
}

fn derive_key(passphrase: &[u8], salt: &[u8], key: &mut [u8]) -> Result<()> {
  let params = Params::new(19456, 2, 1, Some(KEY_LEN)).map_err(|error| anyhow::anyhow!("argon2 params: {error}"))?;
  let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
  argon2.hash_password_into(passphrase, salt, key).map_err(|error| anyhow::anyhow!("argon2 derive: {error}"))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip_encrypt_decrypt() {
    let plaintext = b"secret payload 12345";
    let blob = encrypt(plaintext, "s3cret").unwrap();
    let decrypted = decrypt(&blob, "s3cret").unwrap();
    assert_eq!(decrypted, plaintext);
  }

  #[test]
  fn wrong_passphrase_fails() {
    let plaintext = b"hello";
    let blob = encrypt(plaintext, "right").unwrap();
    assert!(decrypt(&blob, "wrong").is_err());
  }
}
