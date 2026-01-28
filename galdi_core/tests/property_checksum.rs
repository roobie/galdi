// Property-based tests for Checksum operations
mod common;

use common::*;
use galdi_core::{ChecksumAlgorithm, get_hasher};
use proptest::prelude::*;
use std::io::Write;

proptest! {
    /// Property A: Hash Length Property
    /// ∀ content: Vec<u8>.
    ///   hash_xxh3(content).len() == "xxh3_64:".len() + 16  // 16 hex digits
    ///   hash_sha256(content).len() == "sha256:".len() + 64  // 64 hex digits
    #[test]
    fn proptest_checksum_length(
        content in proptest::collection::vec(any::<u8>(), 0..1000),
        use_sha256 in proptest::bool::ANY
    ) {
        let temp_dir = create_test_dir();
        let file_path = temp_dir.path().join("test_file.bin");

        // Write content to file
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(&content).unwrap();
        drop(file);

        let algo = if use_sha256 {
            ChecksumAlgorithm::Sha256
        } else {
            ChecksumAlgorithm::XXH3_64
        };

        let hasher = get_hasher(algo);
        let checksum = hasher.hash_file(&file_path).expect("Hashing should succeed");

        if use_sha256 {
            prop_assert_eq!(checksum.len(), "sha256:".len() + 64,
                "SHA256 checksum should be 7 (prefix) + 64 (hex) = 71 chars");
        } else {
            prop_assert_eq!(checksum.len(), "xxh3_64:".len() + 16,
                "XXH3_64 checksum should be 8 (prefix) + 16 (hex) = 24 chars");
        }
    }

    /// Property B: Prefix Property
    /// ∀ hash_result.
    ///   hash_result.starts_with("xxh3_64:") || hash_result.starts_with("sha256:")
    #[test]
    fn proptest_checksum_prefix(
        content in proptest::collection::vec(any::<u8>(), 0..500),
        use_sha256 in proptest::bool::ANY
    ) {
        let temp_dir = create_test_dir();
        let file_path = temp_dir.path().join("test_file.bin");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(&content).unwrap();
        drop(file);

        let algo = if use_sha256 {
            ChecksumAlgorithm::Sha256
        } else {
            ChecksumAlgorithm::XXH3_64
        };

        let hasher = get_hasher(algo);
        let checksum = hasher.hash_file(&file_path).expect("Hashing should succeed");

        if use_sha256 {
            prop_assert!(checksum.starts_with("sha256:"),
                "SHA256 checksum should start with 'sha256:'");
        } else {
            prop_assert!(checksum.starts_with("xxh3_64:"),
                "XXH3_64 checksum should start with 'xxh3_64:'");
        }
    }

    /// Property C: Hex Character Property
    /// ∀ hash_result.
    ///   ∀ char in hash_result.after_prefix().
    ///     char.is_ascii_hexdigit()
    #[test]
    fn proptest_checksum_hex_digits(
        content in proptest::collection::vec(any::<u8>(), 0..500),
        use_sha256 in proptest::bool::ANY
    ) {
        let temp_dir = create_test_dir();
        let file_path = temp_dir.path().join("test_file.bin");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(&content).unwrap();
        drop(file);

        let algo = if use_sha256 {
            ChecksumAlgorithm::Sha256
        } else {
            ChecksumAlgorithm::XXH3_64
        };

        let hasher = get_hasher(algo);
        let checksum = hasher.hash_file(&file_path).expect("Hashing should succeed");

        let prefix_len = if use_sha256 { 7 } else { 8 };
        let hex_part = &checksum[prefix_len..];

        for (i, ch) in hex_part.chars().enumerate() {
            prop_assert!(ch.is_ascii_hexdigit(),
                "Character {} at position {} in '{}' should be a hex digit",
                ch, i, hex_part);
        }
    }

    /// Property D: Lowercase Hex
    /// SHA256 checksums should use lowercase hex
    #[test]
    fn proptest_sha256_lowercase(
        content in proptest::collection::vec(any::<u8>(), 0..500)
    ) {
        let temp_dir = create_test_dir();
        let file_path = temp_dir.path().join("test_file.bin");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(&content).unwrap();
        drop(file);

        let hasher = get_hasher(ChecksumAlgorithm::Sha256);
        let checksum = hasher.hash_file(&file_path).expect("Hashing should succeed");

        let hex_part = &checksum[7..]; // Skip "sha256:" prefix

        for ch in hex_part.chars() {
            prop_assert!(!ch.is_uppercase(),
                "SHA256 hex should be lowercase, found uppercase: {}", ch);
        }
    }

    /// Property E: Determinism
    /// Hashing the same content twice should produce the same result
    #[test]
    fn proptest_checksum_determinism(
        content in proptest::collection::vec(any::<u8>(), 0..1000),
        use_sha256 in proptest::bool::ANY
    ) {
        let temp_dir = create_test_dir();
        let file_path = temp_dir.path().join("test_file.bin");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(&content).unwrap();
        drop(file);

        let algo = if use_sha256 {
            ChecksumAlgorithm::Sha256
        } else {
            ChecksumAlgorithm::XXH3_64
        };

        let hasher1 = get_hasher(algo);
        let checksum1 = hasher1.hash_file(&file_path).expect("First hash should succeed");

        let hasher2 = get_hasher(algo);
        let checksum2 = hasher2.hash_file(&file_path).expect("Second hash should succeed");

        prop_assert_eq!(checksum1, checksum2,
            "Hashing the same file twice should produce identical results");
    }

    /// Property F: Different Content → Different Hash (Collision Resistance)
    /// This is probabilistic but should hold with very high probability
    #[test]
    fn proptest_checksum_collision_resistance(
        content1 in proptest::collection::vec(any::<u8>(), 1..500),
        content2 in proptest::collection::vec(any::<u8>(), 1..500),
        use_sha256 in proptest::bool::ANY
    ) {
        // Skip if contents are identical
        prop_assume!(content1 != content2);

        let temp_dir = create_test_dir();
        let file1_path = temp_dir.path().join("file1.bin");
        let file2_path = temp_dir.path().join("file2.bin");

        let mut file1 = std::fs::File::create(&file1_path).unwrap();
        file1.write_all(&content1).unwrap();
        drop(file1);

        let mut file2 = std::fs::File::create(&file2_path).unwrap();
        file2.write_all(&content2).unwrap();
        drop(file2);

        let algo = if use_sha256 {
            ChecksumAlgorithm::Sha256
        } else {
            ChecksumAlgorithm::XXH3_64
        };

        let hasher = get_hasher(algo);
        let checksum1 = hasher.hash_file(&file1_path).expect("Hash 1 should succeed");
        let checksum2 = hasher.hash_file(&file2_path).expect("Hash 2 should succeed");

        prop_assert_ne!(checksum1, checksum2,
            "Different content should produce different checksums (collision resistance)");
    }
}
