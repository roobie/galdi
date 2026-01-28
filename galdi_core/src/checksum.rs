use blake3::Hasher as Blake3Impl;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    hash::Hasher,
    io::{self, Read},
    path::Path,
};

use crate::snapshot::ChecksumAlgorithm;

pub trait GaldiHasher {
    fn hash_file(&self, path: &Path) -> io::Result<String>;
}

pub struct XXH3_64Hasher;
pub struct Sha256Hasher;
pub struct Blake3Hasher;

impl GaldiHasher for XXH3_64Hasher {
    fn hash_file(&self, path: &Path) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = xxhash_rust::xxh3::Xxh3Default::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("xxh3_64:{:016x}", hasher.finish()))
    }
}

impl GaldiHasher for Sha256Hasher {
    fn hash_file(&self, path: &Path) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        io::copy(&mut file, &mut hasher)?;
        Ok(format!("sha256:{:064x}", hasher.finalize()))
    }
}

impl GaldiHasher for Blake3Hasher {
    fn hash_file(&self, path: &Path) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Blake3Impl::new();
        io::copy(&mut file, &mut hasher)?;
        Ok(format!("blake3:{}", hasher.finalize().to_hex()))
    }
}

pub fn get_hasher(algorithm: ChecksumAlgorithm) -> Box<dyn GaldiHasher> {
    match algorithm {
        ChecksumAlgorithm::XXH3_64 => Box::new(XXH3_64Hasher),
        ChecksumAlgorithm::Sha256 => Box::new(Sha256Hasher),
        ChecksumAlgorithm::Blake3 => Box::new(Blake3Hasher),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xxh3_format_is_16_hex_chars() {
        // Create a test file with some content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        temp_file.flush().unwrap();

        let hasher = XXH3_64Hasher;
        let result = hasher.hash_file(temp_file.path()).unwrap();

        // Should have prefix
        assert!(result.starts_with("xxh3_64:"));

        // Hex part should be exactly 16 characters
        let hex_part = &result[8..];
        assert_eq!(hex_part.len(), 16, "XXH3_64 should produce 16 hex chars");

        // All characters should be valid hex digits
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha256_format_is_64_hex_chars() {
        // Create a test file with some content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        temp_file.flush().unwrap();

        let hasher = Sha256Hasher;
        let result = hasher.hash_file(temp_file.path()).unwrap();

        // Should have prefix
        assert!(result.starts_with("sha256:"));

        // Hex part should be exactly 64 characters (this test catches the bug!)
        let hex_part = &result[7..];
        assert_eq!(hex_part.len(), 64, "SHA256 should produce 64 hex chars");

        // All characters should be valid hex digits
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));

        // Should be lowercase
        assert!(hex_part.chars().all(|c| !c.is_uppercase()));
    }

    #[test]
    fn test_xxh3_has_correct_prefix() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"hello").unwrap();
        temp_file.flush().unwrap();

        let hasher = XXH3_64Hasher;
        let result = hasher.hash_file(temp_file.path()).unwrap();

        assert!(
            result.starts_with("xxh3_64:"),
            "XXH3 checksum should start with 'xxh3_64:'"
        );
    }

    #[test]
    fn test_sha256_has_correct_prefix() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"hello").unwrap();
        temp_file.flush().unwrap();

        let hasher = Sha256Hasher;
        let result = hasher.hash_file(temp_file.path()).unwrap();

        assert!(
            result.starts_with("sha256:"),
            "SHA256 checksum should start with 'sha256:'"
        );
    }

    #[test]
    fn test_empty_file_known_hash_xxh3() {
        // Empty file should have known hash value
        let temp_file = NamedTempFile::new().unwrap();
        // Don't write anything - file is empty

        let hasher = XXH3_64Hasher;
        let result = hasher.hash_file(temp_file.path()).unwrap();

        // XXH3 of empty input is 0x2d06800538d394c2
        assert_eq!(result, "xxh3_64:2d06800538d394c2");
    }

    #[test]
    fn test_empty_file_known_hash_sha256() {
        // Empty file should have known hash value
        let temp_file = NamedTempFile::new().unwrap();
        // Don't write anything - file is empty

        let hasher = Sha256Hasher;
        let result = hasher.hash_file(temp_file.path()).unwrap();

        // SHA256 of empty input (e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855)
        assert_eq!(
            result,
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_known_content_sha256() {
        // Test SHA256 with known input/output
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"hello world").unwrap();
        temp_file.flush().unwrap();

        let hasher = Sha256Hasher;
        let result = hasher.hash_file(temp_file.path()).unwrap();

        // SHA256("hello world") = b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
        assert_eq!(
            result,
            "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_get_hasher_xxh3() {
        let hasher = get_hasher(ChecksumAlgorithm::XXH3_64);

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test").unwrap();
        temp_file.flush().unwrap();

        let result = hasher.hash_file(temp_file.path()).unwrap();
        assert!(result.starts_with("xxh3_64:"));
    }

    #[test]
    fn test_get_hasher_sha256() {
        let hasher = get_hasher(ChecksumAlgorithm::Sha256);

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test").unwrap();
        temp_file.flush().unwrap();

        let result = hasher.hash_file(temp_file.path()).unwrap();
        assert!(result.starts_with("sha256:"));
    }

    #[test]
    fn test_blake3_format_is_64_hex_chars() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        temp_file.flush().unwrap();

        let hasher = Blake3Hasher;
        let result = hasher.hash_file(temp_file.path()).unwrap();

        assert!(result.starts_with("blake3:"));

        let hex_part = &result[7..];
        assert_eq!(hex_part.len(), 64, "Blake3 should produce 64 hex chars");
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_blake3_has_correct_prefix() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"hello").unwrap();
        temp_file.flush().unwrap();

        let hasher = Blake3Hasher;
        let result = hasher.hash_file(temp_file.path()).unwrap();

        assert!(
            result.starts_with("blake3:"),
            "Blake3 checksum should start with 'blake3:'"
        );
    }

    #[test]
    fn test_blake3_lowercase_hex() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test").unwrap();
        temp_file.flush().unwrap();

        let hasher = Blake3Hasher;
        let result = hasher.hash_file(temp_file.path()).unwrap();

        let hex_part = &result[7..];
        assert!(hex_part.chars().all(|c| !c.is_uppercase()));
    }

    #[test]
    fn test_empty_file_known_hash_blake3() {
        let temp_file = NamedTempFile::new().unwrap();

        let hasher = Blake3Hasher;
        let result = hasher.hash_file(temp_file.path()).unwrap();

        // Blake3 hash of empty input
        assert_eq!(
            result,
            "blake3:af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
    }

    #[test]
    fn test_known_content_blake3() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"hello world").unwrap();
        temp_file.flush().unwrap();

        let hasher = Blake3Hasher;
        let result = hasher.hash_file(temp_file.path()).unwrap();

        // Blake3("hello world")
        assert_eq!(
            result,
            "blake3:d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
        );
    }

    #[test]
    fn test_get_hasher_blake3() {
        let hasher = get_hasher(ChecksumAlgorithm::Blake3);

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test").unwrap();
        temp_file.flush().unwrap();

        let result = hasher.hash_file(temp_file.path()).unwrap();
        assert!(result.starts_with("blake3:"));
    }
}
