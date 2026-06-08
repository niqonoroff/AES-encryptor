use std::io::Write;
use std::path::Path;

use aes_gcm::Aes256Gcm;
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::{Aead, KeyInit, KeySizeUser, Payload};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use rand::rngs::OsRng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type ProgressFn = dyn Fn(u32) + Send + Sync;

pub const MAGIC: &[u8] = b"NQ02";
pub const SALT_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;
pub const CHUNK_SIZE: usize = 4 * 1024 * 1024;

pub const KIND_TEXT: &str = "text";
pub const KIND_BINARY: &str = "binary";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
    pub ext: String,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct KdfParams {
    pub time: u32,
    pub memory: u32,
    pub parallelism: u32,
}

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("invalid file format")]
    InvalidFormat,
    #[error("invalid password or corrupted file")]
    InvalidPassword,
    #[error("invalid argon2 parameters: {0}")]
    InvalidParams(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy)]
struct ChunkRange {
    start: usize,
    end: usize,
}

fn chunk_range(index: usize, total_len: usize) -> ChunkRange {
    let start = index * CHUNK_SIZE;
    let end = (start + CHUNK_SIZE).min(total_len);
    ChunkRange { start, end }
}

fn chunk_count(total_len: usize) -> usize {
    if total_len == 0 {
        0
    } else {
        total_len.div_ceil(CHUNK_SIZE)
    }
}

fn make_cipher(key: &[u8; 32]) -> Aes256Gcm {
    type KS = <Aes256Gcm as KeySizeUser>::KeySize;
    Aes256Gcm::new(GenericArray::<u8, KS>::from_slice(key))
}

fn chunk_nonce(base: &[u8; NONCE_SIZE], index: u64) -> [u8; NONCE_SIZE] {
    let mut nonce = *base;
    let idx_bytes = index.to_le_bytes();
    for (n, b) in nonce.iter_mut().zip(idx_bytes.iter()) {
        *n ^= b;
    }
    nonce
}

fn derive_key(password: &str, salt: &[u8], params: &KdfParams) -> Result<[u8; 32], CryptoError> {
    let argon_params = Params::new(params.memory, params.time, params.parallelism, Some(32))
        .map_err(|e| CryptoError::InvalidParams(e.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| CryptoError::InvalidParams(e.to_string()))?;
    Ok(key)
}

struct ParsedHeader<'a> {
    salt: [u8; SALT_SIZE],
    base_nonce: [u8; NONCE_SIZE],
    meta: Meta,
    body: &'a [u8],
}

fn parse_header(blob: &[u8]) -> Result<ParsedHeader<'_>, CryptoError> {
    let min_len = MAGIC.len() + SALT_SIZE + NONCE_SIZE + 4;
    if blob.len() < min_len || &blob[..MAGIC.len()] != MAGIC {
        return Err(CryptoError::InvalidFormat);
    }
    let mut offset = MAGIC.len();
    let mut salt = [0u8; SALT_SIZE];
    salt.copy_from_slice(&blob[offset..offset + SALT_SIZE]);
    offset += SALT_SIZE;
    let mut base_nonce = [0u8; NONCE_SIZE];
    base_nonce.copy_from_slice(&blob[offset..offset + NONCE_SIZE]);
    offset += NONCE_SIZE;
    let meta_len = u32::from_le_bytes(
        blob[offset..offset + 4]
            .try_into()
            .map_err(|_| CryptoError::InvalidFormat)?,
    ) as usize;
    offset += 4;
    if blob.len() < offset + meta_len {
        return Err(CryptoError::InvalidFormat);
    }
    let meta: Meta = serde_json::from_slice(&blob[offset..offset + meta_len])?;
    offset += meta_len;
    let body = &blob[offset..];
    Ok(ParsedHeader {
        salt,
        base_nonce,
        meta,
        body,
    })
}

#[derive(Debug, Clone, Copy)]
struct ChunkSpan {
    start: usize,
    len: usize,
}

fn parse_chunks(body: &[u8]) -> Result<Vec<ChunkSpan>, CryptoError> {
    let mut spans = Vec::new();
    let mut offset = 0usize;
    while offset < body.len() {
        if body.len() < offset + 4 {
            return Err(CryptoError::InvalidFormat);
        }
        let len = u32::from_le_bytes(
            body[offset..offset + 4]
                .try_into()
                .map_err(|_| CryptoError::InvalidFormat)?,
        ) as usize;
        offset += 4;
        if body.len() < offset + len {
            return Err(CryptoError::InvalidFormat);
        }
        spans.push(ChunkSpan { start: offset, len });
        offset += len;
    }
    if offset != body.len() {
        return Err(CryptoError::InvalidFormat);
    }
    Ok(spans)
}

fn plaintext_size(spans: &[ChunkSpan]) -> usize {
    spans.iter().map(|s| s.len.saturating_sub(16)).sum()
}

pub fn encrypt_bytes(
    plaintext: &[u8],
    password: &str,
    params: &KdfParams,
    meta: &Meta,
    progress: Option<&ProgressFn>,
) -> Result<Vec<u8>, CryptoError> {
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    let mut base_nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut base_nonce);

    if let Some(p) = progress {
        p(0);
    }
    let key = derive_key(password, &salt, params)?;
    if let Some(p) = progress {
        p(30);
    }
    let meta_json = serde_json::to_vec(meta)?;

    let num_chunks = chunk_count(plaintext.len());
    let key_arr: [u8; 32] = key;

    let chunks: Vec<Vec<u8>> = (0..num_chunks)
        .into_par_iter()
        .map(|i| {
            let r = chunk_range(i, plaintext.len());
            let pt = &plaintext[r.start..r.end];
            let nonce = chunk_nonce(&base_nonce, i as u64);
            let aad = (i as u64).to_le_bytes();
            let cipher = make_cipher(&key_arr);
            let payload = Payload { aad: &aad, msg: pt };
            cipher
                .encrypt(GenericArray::from_slice(&nonce), payload)
                .expect("AEAD encrypt failure")
        })
        .collect();

    if let Some(p) = progress {
        p(80);
    }

    let header_size = MAGIC.len() + SALT_SIZE + NONCE_SIZE + 4 + meta_json.len();
    let body_size = num_chunks * 4 + plaintext.len() + num_chunks * 16;
    let mut out = Vec::with_capacity(header_size + body_size);

    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&base_nonce);
    out.extend_from_slice(&(meta_json.len() as u32).to_le_bytes());
    out.extend_from_slice(&meta_json);
    for chunk in chunks {
        out.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
        out.extend_from_slice(&chunk);
    }
    if let Some(p) = progress {
        p(100);
    }
    Ok(out)
}

pub fn decrypt_bytes(
    blob: &[u8],
    password: &str,
    params: &KdfParams,
    progress: Option<&ProgressFn>,
) -> Result<(Meta, Vec<u8>), CryptoError> {
    let header = parse_header(blob)?;
    let spans = parse_chunks(header.body)?;
    let pt_len = plaintext_size(&spans);

    if let Some(p) = progress {
        p(0);
    }
    let key = derive_key(password, &header.salt, params)?;
    if let Some(p) = progress {
        p(40);
    }

    let key_arr: [u8; 32] = key;

    let body = header.body;
    let base_nonce = header.base_nonce;

    let decrypted: Vec<Vec<u8>> = spans
        .par_iter()
        .enumerate()
        .map(|(i, span)| -> Result<Vec<u8>, CryptoError> {
            let ct = &body[span.start..span.start + span.len];
            let nonce = chunk_nonce(&base_nonce, i as u64);
            let aad = (i as u64).to_le_bytes();
            let cipher = make_cipher(&key_arr);
            let payload = Payload { aad: &aad, msg: ct };
            cipher
                .decrypt(GenericArray::from_slice(&nonce), payload)
                .map_err(|_| CryptoError::InvalidPassword)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if let Some(p) = progress {
        p(80);
    }

    let mut plaintext = Vec::with_capacity(pt_len);
    for chunk in &decrypted {
        plaintext.extend_from_slice(chunk);
    }
    if let Some(p) = progress {
        p(100);
    }
    Ok((header.meta, plaintext))
}

pub fn encrypt_file(
    input: &Path,
    output: &Path,
    password: &str,
    params: &KdfParams,
    meta: &Meta,
    progress: Option<&ProgressFn>,
) -> Result<(), CryptoError> {
    let plaintext = std::fs::read(input)?;
    let blob = encrypt_bytes(&plaintext, password, params, meta, progress)?;
    write_atomic(output, &blob)
}

fn write_atomic(path: &Path, data: &[u8]) -> Result<(), CryptoError> {
    let tmp = path.with_extension("nqtmp");
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(data)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const T_TIME: u32 = 2;
    const T_MEMORY: u32 = 64 * 1024;
    const T_PAR: u32 = 2;

    fn params() -> KdfParams {
        KdfParams {
            time: T_TIME,
            memory: T_MEMORY,
            parallelism: T_PAR,
        }
    }

    fn meta_text() -> Meta {
        Meta {
            name: "note.nqtxt".into(),
            ext: "nqtxt".into(),
            kind: KIND_TEXT.into(),
        }
    }

    fn meta_bin() -> Meta {
        Meta {
            name: "image.png".into(),
            ext: "png".into(),
            kind: KIND_BINARY.into(),
        }
    }

    #[test]
    fn roundtrip_text_small() {
        let p = params();
        let m = meta_text();
        let blob = encrypt_bytes(b"Hello, NQ02!", "pw", &p, &m, None).unwrap();
        assert_eq!(&blob[..4], MAGIC);
        let (out_m, pt) = decrypt_bytes(&blob, "pw", &p, None).unwrap();
        assert_eq!(out_m.kind, KIND_TEXT);
        assert_eq!(pt, b"Hello, NQ02!");
    }

    #[test]
    fn roundtrip_binary_random() {
        let p = params();
        let m = meta_bin();
        let data: Vec<u8> = (0..(CHUNK_SIZE * 2 + 12345))
            .map(|i| (i % 251) as u8)
            .collect();
        let blob = encrypt_bytes(&data, "pw", &p, &m, None).unwrap();
        let (out_m, pt) = decrypt_bytes(&blob, "pw", &p, None).unwrap();
        assert_eq!(out_m.name, "image.png");
        assert_eq!(pt, data);
    }

    #[test]
    fn roundtrip_empty() {
        let p = params();
        let m = meta_text();
        let blob = encrypt_bytes(b"", "pw", &p, &m, None).unwrap();
        let (out_m, pt) = decrypt_bytes(&blob, "pw", &p, None).unwrap();
        assert_eq!(out_m.name, "note.nqtxt");
        assert_eq!(pt, b"");
    }

    #[test]
    fn wrong_password_fails() {
        let p = params();
        let blob = encrypt_bytes(b"secret", "right", &p, &meta_text(), None).unwrap();
        let res = decrypt_bytes(&blob, "wrong", &p, None);
        assert!(matches!(res, Err(CryptoError::InvalidPassword)));
    }

    #[test]
    fn invalid_magic_fails() {
        let p = params();
        let mut bad = vec![0u8; 64];
        bad[..4].copy_from_slice(b"BADM");
        let res = decrypt_bytes(&bad, "pw", &p, None);
        assert!(matches!(res, Err(CryptoError::InvalidFormat)));
    }

    #[test]
    fn file_roundtrip() {
        let dir = std::env::temp_dir();
        let in_path = dir.join("nq_test_in.bin");
        let enc_path = dir.join("nq_test_enc.nqtxt");
        let dec_path = dir.join("nq_test_dec.bin");

        let original: Vec<u8> = (0..(CHUNK_SIZE + 17)).map(|i| (i % 251) as u8).collect();
        std::fs::write(&in_path, &original).unwrap();

        let p = params();
        let m = Meta {
            name: "nq_test_in.bin".into(),
            ext: "bin".into(),
            kind: KIND_BINARY.into(),
        };
        encrypt_file(&in_path, &enc_path, "pw", &p, &m, None).unwrap();

        let blob = std::fs::read(&enc_path).unwrap();
        let (m2, pt) = decrypt_bytes(&blob, "pw", &p, None).unwrap();
        assert_eq!(m2.name, "nq_test_in.bin");
        assert_eq!(pt, original);

        std::fs::write(&dec_path, &pt).unwrap();
        let recovered = std::fs::read(&dec_path).unwrap();
        assert_eq!(recovered, original);

        let _ = std::fs::remove_file(&in_path);
        let _ = std::fs::remove_file(&enc_path);
        let _ = std::fs::remove_file(&dec_path);
    }
}
