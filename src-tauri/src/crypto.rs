use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::{Aead, AeadCore, KeyInit};
use aes_gcm::{Aes256Gcm, KeySizeUser};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::rngs::OsRng;
use rand::RngCore;

pub const MAGIC: &[u8] = b"NQ01";

pub fn encrypt(
    plaintext: &str,
    password: &str,
    time_cost: u32,
    memory_cost: u32,
    parallelism: u32,
    salt_size: usize,
    nonce_size: usize,
) -> Vec<u8> {
    let mut salt = vec![0u8; salt_size];
    OsRng.fill_bytes(&mut salt);

    let mut nonce_bytes = vec![0u8; nonce_size];
    OsRng.fill_bytes(&mut nonce_bytes);

    let key = derive_key(password, &salt, time_cost, memory_cost, parallelism);

    type KS = <Aes256Gcm as KeySizeUser>::KeySize;
    let key = GenericArray::<u8, KS>::from_slice(&key);
    let cipher = Aes256Gcm::new(key);

    type NS = <Aes256Gcm as AeadCore>::NonceSize;
    let nonce = GenericArray::<u8, NS>::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("encryption failed");

    let mut result = Vec::with_capacity(4 + salt_size + nonce_size + ciphertext.len());
    result.extend_from_slice(MAGIC);
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    result
}

pub fn decrypt(
    blob: &[u8],
    password: &str,
    time_cost: u32,
    memory_cost: u32,
    parallelism: u32,
    salt_size: usize,
    nonce_size: usize,
) -> Result<String, String> {
    if blob.len() < 4 + salt_size + nonce_size || &blob[..4] != MAGIC {
        return Err("Invalid file format".to_string());
    }

    let salt = &blob[4..4 + salt_size];
    let nonce_bytes = &blob[4 + salt_size..4 + salt_size + nonce_size];
    let ciphertext = &blob[4 + salt_size + nonce_size..];

    let key = derive_key(password, salt, time_cost, memory_cost, parallelism);

    type KS = <Aes256Gcm as KeySizeUser>::KeySize;
    let key = GenericArray::<u8, KS>::from_slice(&key);
    let cipher = Aes256Gcm::new(key);

    type NS = <Aes256Gcm as AeadCore>::NonceSize;
    let nonce = GenericArray::<u8, NS>::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map(|plaintext| String::from_utf8(plaintext).expect("invalid UTF-8"))
        .map_err(|_| "Invalid password or corrupted file".to_string())
}

fn derive_key(
    password: &str,
    salt: &[u8],
    time_cost: u32,
    memory_cost: u32,
    parallelism: u32,
) -> Vec<u8> {
    let params =
        Params::new(memory_cost, time_cost, parallelism, Some(32)).expect("invalid argon2 params");
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = vec![0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .expect("key derivation failed");
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    const T_ARGON_TIME: u32 = 8;
    const T_ARGON_MEMORY: u32 = 512 * 1024;
    const T_ARGON_PARALLEL: u32 = 4;
    const T_SALT_SIZE: usize = 32;
    const T_NONCE_SIZE: usize = 12;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = "Hello, NQTXT! Test 123";
        let password = "test_password";
        let encrypted = encrypt(plaintext, password, T_ARGON_TIME, T_ARGON_MEMORY, T_ARGON_PARALLEL, T_SALT_SIZE, T_NONCE_SIZE);
        assert_eq!(&encrypted[..4], MAGIC);
        let decrypted = decrypt(&encrypted, password, T_ARGON_TIME, T_ARGON_MEMORY, T_ARGON_PARALLEL, T_SALT_SIZE, T_NONCE_SIZE).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_password_fails() {
        let encrypted = encrypt("secret text", "correct_password", T_ARGON_TIME, T_ARGON_MEMORY, T_ARGON_PARALLEL, T_SALT_SIZE, T_NONCE_SIZE);
        let result = decrypt(&encrypted, "wrong_password", T_ARGON_TIME, T_ARGON_MEMORY, T_ARGON_PARALLEL, T_SALT_SIZE, T_NONCE_SIZE);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_format() {
        let result = decrypt(b"BADMAGICxxx", "password", T_ARGON_TIME, T_ARGON_MEMORY, T_ARGON_PARALLEL, T_SALT_SIZE, T_NONCE_SIZE);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_params() {
        let plaintext = "Custom params test";
        let password = "mypassword";
        let encrypted = encrypt(plaintext, password, 4, 256 * 1024, 2, 16, 12);
        let decrypted = decrypt(&encrypted, password, 4, 256 * 1024, 2, 16, 12).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
