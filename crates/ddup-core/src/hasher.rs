//! SHA-256 hashing helpers.

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use sha2::{Digest, Sha256};

use crate::{CoreError, DirHash, FileHash, Result};

const BUFFER_SIZE: usize = 64 * 1024;

/// Computes the SHA-256 hash of a file using buffered streaming.
pub fn hash_file(path: &Path) -> Result<FileHash> {
    let file = File::open(path).map_err(|source| CoreError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; BUFFER_SIZE];

    loop {
        let read = reader.read(&mut buffer).map_err(|source| CoreError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(FileHash::new(hex_lower(hasher.finalize())))
}

/// Combines direct file hashes and child directory hashes into a deterministic directory hash.
///
/// # Notes
///
/// Inputs are sorted lexicographically before hashing, so filesystem iteration order and
/// parallelism cannot change the output.
#[must_use]
pub fn hash_dir(file_hashes: &[FileHash], child_dir_hashes: &[DirHash]) -> DirHash {
    let mut parts = file_hashes
        .iter()
        .map(|hash| format!("file:{}", hash.as_str()))
        .chain(
            child_dir_hashes
                .iter()
                .map(|hash| format!("dir:{}", hash.as_str())),
        )
        .collect::<Vec<_>>();
    parts.sort_unstable();
    let payload = parts.join("\n");
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    DirHash::new(hex_lower(hasher.finalize()))
}

fn hex_lower(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    out
}
