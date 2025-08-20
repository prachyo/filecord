use anyhow::*;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::XNonce;

pub fn encrypt_xchacha20(key: &[u8;32], nonce: &[u8;24], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let ct = cipher.encrypt(XNonce::from_slice(nonce), plaintext)?;
    Ok(ct)
}

pub fn decrypt_xchacha20(key: &[u8;32], nonce: &[u8;24], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let pt = cipher.decrypt(XNonce::from_slice(nonce), ciphertext)?;
    Ok(pt)
}
