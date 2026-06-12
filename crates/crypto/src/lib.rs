use base64::{Engine, engine::general_purpose::STANDARD};
use chacha20poly1305::{
    Key, KeyInit, XChaCha20Poly1305,
    aead::{Aead, OsRng, rand_core::RngCore},
};
use keyring::Entry;

// Explicitly bring the Nonce type into scope if needed,
// XChaCha20Poly1305 uses XNonce (24 bytes) automatically.
type Nonce = chacha20poly1305::XNonce;

/// Encrypts a secret using XChaCha20Poly1305 and returns the nonce and ciphertext.
pub fn encrypt(secret: String, key: &[u8; 32]) -> ([u8; 24], Vec<u8>) {
    let secret: &[u8] = secret.as_bytes();
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));

    let mut nonce_bytes: [u8; 24] = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);

    // Dereference the from_slice result to get an owned Nonce struct
    let nonce = *Nonce::from_slice(&nonce_bytes);

    // Pass a clean single reference &nonce
    let ciphertext: Vec<u8> = cipher.encrypt(&nonce, secret).unwrap();

    (nonce_bytes, ciphertext)
}

pub fn decrypt(nonce: &[u8; 24], ciphertext: &[u8], key: &[u8; 32]) -> String {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));

    // Dereference to get an owned Nonce struct from the input slice
    let nonce = *Nonce::from_slice(nonce);

    // Pass a clean single reference &nonce
    let plaintext: Vec<u8> = cipher.decrypt(&nonce, ciphertext).unwrap();

    String::from_utf8(plaintext).unwrap()
}

pub fn generate_nonce() -> [u8; 24] {
    let mut nonce: [u8; 24] = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn generate_key() -> [u8; 32] {
    let mut key: [u8; 32] = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn store_key(key: [u8; 32]) -> Result<(), Box<dyn std::error::Error>> {
    let key = &STANDARD.encode(key);

    let entry = Entry::new("onyx", "encryption_key")?;
    entry.set_password(key)?;
    Ok(())
}

pub fn retrieve_key() -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let entry = Entry::new("onyx", "encryption_key")?;
    let key_str = entry.get_password()?;
    let key_bytes = STANDARD.decode(key_str)?;

    if key_bytes.len() != 32 {
        return Err("Invalid key length".into());
    }

    let mut key: [u8; 32] = [0u8; 32];
    key.copy_from_slice(&key_bytes);
    Ok(key)
}

pub fn gen_or_retrieve_key() -> Result<[u8; 32], Box<dyn std::error::Error>> {
    match retrieve_key() {
        Ok(key) => Ok(key),
        Err(_) => {
            let key = generate_key();
            store_key(key)?;
            Ok(key)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key();
        let secret = "This is a secret message.".to_string();
        let (nonce, ciphertext) = encrypt(secret.clone(), &key);
        let decrypted = decrypt(&nonce, &ciphertext, &key);
        assert_eq!(secret, decrypted);
    }

    #[test]
    fn test_key_storage() {
        let key = generate_key();
        store_key(key).unwrap();
        let retrieved_key = retrieve_key().unwrap();
        assert_eq!(key, retrieved_key);
    }

    #[test]
    fn test_gen_or_retrieve_key() {
        let key1 = gen_or_retrieve_key().unwrap();
        let key2 = gen_or_retrieve_key().unwrap();
        assert_eq!(key1, key2);
    }
}
