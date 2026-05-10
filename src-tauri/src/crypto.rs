//! AES-256-GCM encryption utilities for storing sensitive configuration.
//!
//! Uses HKDF-SHA256 to derive an AES key from a hard-coded application secret,
//! then encrypts/decrypts with AES-256-GCM.  Ciphertext is stored as
//! `base64(12-byte-nonce || ciphertext)`.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

/// Hard-coded input key material embedded in the binary.
/// Changing this value invalidates all previously stored encrypted values.
const APP_IKM: &[u8] = b"permpoon-v1-secret-ikm-2024!!!!!"; // exactly 32 bytes
const APP_SALT: &[u8] = b"permpoon-db-salt-v1-2024";
const APP_INFO: &[u8] = b"permpoon-db-config-aes256gcm";

/// Derives a 32-byte AES key via HKDF-SHA256.
fn derive_key() -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(APP_SALT), APP_IKM);
    let mut okm = [0u8; 32];
    hk.expand(APP_INFO, &mut okm)
        .expect("HKDF expand should never fail with a 32-byte output");
    okm
}

/// Encrypts `plaintext` with AES-256-GCM.
///
/// Returns a base64-encoded string containing `12-byte-nonce || ciphertext`.
///
/// # Errors
/// Returns an error string if key initialisation or encryption fails.
pub fn encrypt(plaintext: &str) -> Result<String, String> {
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(combined))
}

/// Decrypts a base64-encoded `nonce || ciphertext` produced by [`encrypt`].
///
/// # Errors
/// Returns an error string if decoding, key initialisation, or decryption fails.
pub fn decrypt(encoded: &str) -> Result<String, String> {
    let combined = BASE64.decode(encoded).map_err(|e| e.to_string())?;
    if combined.len() < 12 {
        return Err("Invalid encrypted data: too short".to_string());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed: wrong key or corrupted data".to_string())?;

    String::from_utf8(plaintext_bytes).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty() {
        let enc = encrypt("").unwrap();
        let dec = decrypt(&enc).unwrap();
        assert_eq!(dec, "");
    }

    #[test]
    fn roundtrip_ascii() {
        let plaintext = "my_username_123";
        let enc = encrypt(plaintext).unwrap();
        let dec = decrypt(&enc).unwrap();
        assert_eq!(dec, plaintext);
    }

    #[test]
    fn roundtrip_unicode() {
        let plaintext = "รหัสผ่าน@2024!";
        let enc = encrypt(plaintext).unwrap();
        let dec = decrypt(&enc).unwrap();
        assert_eq!(dec, plaintext);
    }

    #[test]
    fn different_ciphertext_each_time() {
        let enc1 = encrypt("hello").unwrap();
        let enc2 = encrypt("hello").unwrap();
        // Random nonces must produce different ciphertexts
        assert_ne!(enc1, enc2);
    }

    #[test]
    fn decrypt_invalid_base64() {
        assert!(decrypt("not-valid-base64!!!").is_err());
    }

    #[test]
    fn decrypt_too_short() {
        // Valid base64 but too short for nonce
        assert!(decrypt("aGVsbG8=").is_err());
    }
}
