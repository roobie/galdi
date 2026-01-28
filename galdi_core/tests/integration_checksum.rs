// Integration tests for checksum module
mod common;

use common::*;
use galdi_core::{ChecksumAlgorithm, GaldiHasher, Sha256Hasher, XXH3_64Hasher};
use proptest::prelude::*;

#[test]
fn test_hash_small_file() {
    // Test file smaller than buffer size (< 8KB)
    let temp_dir = create_test_dir();
    let content = b"This is a small test file with some content.";
    let file_path = create_file_with_content(temp_dir.path(), "small.txt", content);

    let xxh3_hasher = XXH3_64Hasher;
    let xxh3_result = xxh3_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&xxh3_result, ChecksumAlgorithm::XXH3_64);

    let sha256_hasher = Sha256Hasher;
    let sha256_result = sha256_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&sha256_result, ChecksumAlgorithm::Sha256);
}

#[test]
fn test_hash_medium_file() {
    // Test file around 1MB
    let temp_dir = create_test_dir();
    let size = 1024 * 1024; // 1MB
    let file_path = create_file_with_size(temp_dir.path(), "medium.bin", size);

    let xxh3_hasher = XXH3_64Hasher;
    let xxh3_result = xxh3_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&xxh3_result, ChecksumAlgorithm::XXH3_64);

    let sha256_hasher = Sha256Hasher;
    let sha256_result = sha256_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&sha256_result, ChecksumAlgorithm::Sha256);
}

#[test]
#[ignore] // Slow test - run separately
fn test_hash_large_file() {
    // Test file around 100MB
    let temp_dir = create_test_dir();
    let size = 100 * 1024 * 1024; // 100MB
    let file_path = create_file_with_size(temp_dir.path(), "large.bin", size);

    let xxh3_hasher = XXH3_64Hasher;
    let xxh3_result = xxh3_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&xxh3_result, ChecksumAlgorithm::XXH3_64);

    let sha256_hasher = Sha256Hasher;
    let sha256_result = sha256_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&sha256_result, ChecksumAlgorithm::Sha256);
}

#[test]
fn test_hash_identical_files_same_hash() {
    // Two files with same content should produce same hash
    let temp_dir = create_test_dir();
    let content = b"Identical content for both files";

    let file1 = create_file_with_content(temp_dir.path(), "file1.txt", content);
    let file2 = create_file_with_content(temp_dir.path(), "file2.txt", content);

    let xxh3_hasher = XXH3_64Hasher;
    let hash1 = xxh3_hasher.hash_file(&file1).unwrap();
    let hash2 = xxh3_hasher.hash_file(&file2).unwrap();
    assert_eq!(hash1, hash2, "Identical files should have same XXH3 hash");

    let sha256_hasher = Sha256Hasher;
    let hash1 = sha256_hasher.hash_file(&file1).unwrap();
    let hash2 = sha256_hasher.hash_file(&file2).unwrap();
    assert_eq!(hash1, hash2, "Identical files should have same SHA256 hash");
}

#[test]
fn test_hash_different_files_different_hash() {
    // Two files with different content should produce different hashes
    let temp_dir = create_test_dir();

    let file1 = create_file_with_content(temp_dir.path(), "file1.txt", b"content A");
    let file2 = create_file_with_content(temp_dir.path(), "file2.txt", b"content B");

    let xxh3_hasher = XXH3_64Hasher;
    let hash1 = xxh3_hasher.hash_file(&file1).unwrap();
    let hash2 = xxh3_hasher.hash_file(&file2).unwrap();
    assert_ne!(
        hash1, hash2,
        "Different files should have different XXH3 hashes"
    );

    let sha256_hasher = Sha256Hasher;
    let hash1 = sha256_hasher.hash_file(&file1).unwrap();
    let hash2 = sha256_hasher.hash_file(&file2).unwrap();
    assert_ne!(
        hash1, hash2,
        "Different files should have different SHA256 hashes"
    );
}

#[test]
fn test_hash_determinism_10_runs() {
    // Hashing the same file multiple times should produce the same result
    let temp_dir = create_test_dir();
    let content = b"Determinism test content";
    let file_path = create_file_with_content(temp_dir.path(), "determinism.txt", content);

    // XXH3 determinism
    let xxh3_hasher = XXH3_64Hasher;
    let first_hash = xxh3_hasher.hash_file(&file_path).unwrap();

    for _ in 0..10 {
        let hash = xxh3_hasher.hash_file(&file_path).unwrap();
        assert_eq!(
            hash, first_hash,
            "XXH3 hash should be deterministic across multiple runs"
        );
    }

    // SHA256 determinism
    let sha256_hasher = Sha256Hasher;
    let first_hash = sha256_hasher.hash_file(&file_path).unwrap();

    for _ in 0..10 {
        let hash = sha256_hasher.hash_file(&file_path).unwrap();
        assert_eq!(
            hash, first_hash,
            "SHA256 hash should be deterministic across multiple runs"
        );
    }
}

#[test]
fn test_hash_empty_file() {
    // Empty file should be handled correctly
    let temp_dir = create_test_dir();
    let file_path = create_file_with_content(temp_dir.path(), "empty.txt", b"");

    let xxh3_hasher = XXH3_64Hasher;
    let hash = xxh3_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&hash, ChecksumAlgorithm::XXH3_64);
    assert_eq!(hash, "xxh3_64:2d06800538d394c2"); // Known empty file hash

    let sha256_hasher = Sha256Hasher;
    let hash = sha256_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&hash, ChecksumAlgorithm::Sha256);
    assert_eq!(
        hash,
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn test_hash_file_with_null_bytes() {
    // Binary file with null bytes
    let temp_dir = create_test_dir();
    let content = b"\x00\x00\x00\x00\x00";
    let file_path = create_file_with_content(temp_dir.path(), "nulls.bin", content);

    let xxh3_hasher = XXH3_64Hasher;
    let hash = xxh3_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&hash, ChecksumAlgorithm::XXH3_64);

    let sha256_hasher = Sha256Hasher;
    let hash = sha256_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&hash, ChecksumAlgorithm::Sha256);
}

#[test]
fn test_hash_exactly_8192_bytes() {
    // Test buffer boundary - exactly one buffer full
    let temp_dir = create_test_dir();
    let size = 8192;
    let file_path = create_file_with_size(temp_dir.path(), "exact_buffer.bin", size);

    let xxh3_hasher = XXH3_64Hasher;
    let hash = xxh3_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&hash, ChecksumAlgorithm::XXH3_64);

    let sha256_hasher = Sha256Hasher;
    let hash = sha256_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&hash, ChecksumAlgorithm::Sha256);
}

#[test]
fn test_hash_8191_bytes() {
    // Test buffer boundary - one byte less than buffer
    let temp_dir = create_test_dir();
    let size = 8191;
    let file_path = create_file_with_size(temp_dir.path(), "buffer_minus_one.bin", size);

    let xxh3_hasher = XXH3_64Hasher;
    let hash = xxh3_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&hash, ChecksumAlgorithm::XXH3_64);

    let sha256_hasher = Sha256Hasher;
    let hash = sha256_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&hash, ChecksumAlgorithm::Sha256);
}

#[test]
fn test_hash_8193_bytes() {
    // Test buffer boundary - one byte more than buffer
    let temp_dir = create_test_dir();
    let size = 8193;
    let file_path = create_file_with_size(temp_dir.path(), "buffer_plus_one.bin", size);

    let xxh3_hasher = XXH3_64Hasher;
    let hash = xxh3_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&hash, ChecksumAlgorithm::XXH3_64);

    let sha256_hasher = Sha256Hasher;
    let hash = sha256_hasher.hash_file(&file_path).unwrap();
    assert_checksum_format(&hash, ChecksumAlgorithm::Sha256);
}

#[test]
fn test_hash_nonexistent_file() {
    // Hashing a file that doesn't exist should return an error
    let temp_dir = create_test_dir();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let xxh3_hasher = XXH3_64Hasher;
    let result = xxh3_hasher.hash_file(&nonexistent);
    assert!(result.is_err(), "Should error on nonexistent file");

    let sha256_hasher = Sha256Hasher;
    let result = sha256_hasher.hash_file(&nonexistent);
    assert!(result.is_err(), "Should error on nonexistent file");
}

// Property-based tests using proptest
proptest! {
    #[test]
    fn proptest_xxh3_format_always_valid(content in prop::collection::vec(any::<u8>(), 0..10000)) {
        let temp_dir = create_test_dir();
        let file_path = create_file_with_content(temp_dir.path(), "proptest.bin", &content);

        let hasher = XXH3_64Hasher;
        let result = hasher.hash_file(&file_path).unwrap();

        // Should always have valid format
        assert_checksum_format(&result, ChecksumAlgorithm::XXH3_64);
    }

    #[test]
    fn proptest_sha256_format_always_valid(content in prop::collection::vec(any::<u8>(), 0..10000)) {
        let temp_dir = create_test_dir();
        let file_path = create_file_with_content(temp_dir.path(), "proptest.bin", &content);

        let hasher = Sha256Hasher;
        let result = hasher.hash_file(&file_path).unwrap();

        // Should always have valid format
        assert_checksum_format(&result, ChecksumAlgorithm::Sha256);
    }

    #[test]
    fn proptest_determinism(content in prop::collection::vec(any::<u8>(), 0..1000)) {
        // Same content should always produce same hash
        let temp_dir = create_test_dir();
        let file_path = create_file_with_content(temp_dir.path(), "prop_determ.bin", &content);

        let xxh3_hasher = XXH3_64Hasher;
        let hash1 = xxh3_hasher.hash_file(&file_path).unwrap();
        let hash2 = xxh3_hasher.hash_file(&file_path).unwrap();
        prop_assert_eq!(hash1, hash2);

        let sha256_hasher = Sha256Hasher;
        let hash1 = sha256_hasher.hash_file(&file_path).unwrap();
        let hash2 = sha256_hasher.hash_file(&file_path).unwrap();
        prop_assert_eq!(hash1, hash2);
    }
}
