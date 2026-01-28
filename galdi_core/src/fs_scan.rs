use std::{
    io, panic,
    path::{MAIN_SEPARATOR, PathBuf},
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
    thread,
    time::Instant,
};

use chrono::Utc;
use ignore::{DirEntry, WalkBuilder, overrides::OverrideBuilder};

use crate::{
    Meta, PlumbahObject, Status,
    error::ScanError,
    snapshot::{ChecksumAlgorithm, EntryType, Snapshot, SnapshotEntry},
};

pub struct ScanOptions {
    pub root: PathBuf,
    pub checksum_algorithm: ChecksumAlgorithm,
    pub follow_symlinks: bool,
    pub max_depth: Option<usize>,
    pub exclude_patterns: Vec<String>,
    pub timeout_ms: Option<u64>,
    pub threads: Option<usize>, // None = auto-detect, Some(n) = explicit
    /// Normalize paths to use '/' as separator (useful on Windows).
    pub normalize_paths: bool,
}

pub struct Scanner {
    pub options: ScanOptions,
}

/// Iterator that yields snapshot entries one at a time (streaming)
pub struct ScanIterator {
    receiver: Receiver<Result<SnapshotEntry, ScanError>>,
}

/// Shared scanner configuration for the iterator
struct ScannerRef {
    root: PathBuf,
    checksum_algorithm: ChecksumAlgorithm,
    normalize_paths: bool,
}

impl ScannerRef {
    fn create_entry(&self, entry: DirEntry) -> Result<SnapshotEntry, ScanError> {
        let metadata = entry.metadata()?;
        let relative_path = entry.path().strip_prefix(&self.root)?;

        let entry_type = if metadata.is_dir() {
            EntryType::Directory
        } else if metadata.is_symlink() {
            EntryType::Symlink
        } else if metadata.is_file() {
            EntryType::File
        } else {
            EntryType::Undefined
        };

        let checksum = if entry_type == EntryType::File {
            Some(self.compute_checksum(entry.path())?)
        } else {
            None
        };

        Ok(SnapshotEntry {
            path: if self.normalize_paths {
                to_unix_like_string(relative_path).into()
            } else {
                relative_path.to_path_buf()
            },
            entry_type,
            size: Some(metadata.len()),
            mode: Some(format_mode(&metadata)),
            mtime: metadata.modified()?.into(),
            checksum,
            target: if entry_type == EntryType::Symlink {
                Some(std::fs::read_link(entry.path())?)
            } else {
                None
            },
        })
    }

    fn compute_checksum(&self, path: &std::path::Path) -> Result<String, io::Error> {
        let hasher: Box<dyn crate::checksum::GaldiHasher> =
            crate::checksum::get_hasher(self.checksum_algorithm);
        hasher.hash_file(path)
    }
}

impl Iterator for ScanIterator {
    type Item = Result<SnapshotEntry, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.recv().ok()
    }
}

impl Scanner {
    pub fn new(options: ScanOptions) -> Self {
        Scanner { options }
    }

    /// Create a streaming iterator that yields entries one at a time.
    ///
    /// This is useful for JSONL streaming output where you don't want to
    /// collect all entries in memory. Uses parallel filesystem walking for
    /// maximum performance. Output order is non-deterministic - consumers
    /// should sort the JSONL output if deterministic order is needed (e.g., with jq).
    pub fn scan_iter(&self) -> ScanIterator {
        let mut binding = WalkBuilder::new(&self.options.root);
        let walk_builder = binding
            .max_depth(self.options.max_depth)
            .follow_links(self.options.follow_symlinks);

        walk_builder.add_custom_ignore_filename(".galdi_ignore");

        if !self.options.exclude_patterns.is_empty() {
            let mut overrides = OverrideBuilder::new(&self.options.root);
            for pattern in &self.options.exclude_patterns {
                let mut pattern = pattern.clone();
                // An override glob is a whitelist glob unless it starts with a !, in which case it is an ignore glob.
                pattern.insert(0, '!');
                overrides.add(&pattern).unwrap();
            }

            walk_builder.overrides(overrides.build().unwrap());
        }

        let scanner_ref = Arc::new(ScannerRef {
            root: self.options.root.clone(),
            checksum_algorithm: self.options.checksum_algorithm,
            normalize_paths: self.options.normalize_paths,
        });

        // Always use parallel walker with channel for streaming
        let num_threads = self.options.threads.unwrap_or_else(num_cpus::get);
        walk_builder.threads(num_threads);

        let (tx, rx) = mpsc::channel();
        let walker = walk_builder.build_parallel();

        // Spawn walker in background thread with panic handling
        let tx_panic = tx.clone();
        thread::Builder::new()
            .name("galdi-walker".to_string())
            .spawn(move || {
                // Catch any panics in the walker thread
                let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    walker.run(|| {
                        let tx = tx.clone();
                        let scanner_ref = scanner_ref.clone();
                        Box::new(move |result| {
                            let entry_result = match result {
                                Ok(entry) => scanner_ref.create_entry(entry),
                                Err(err) => Err(ScanError::from(err)),
                            };
                            // Send each entry through channel
                            if tx.send(entry_result).is_err() {
                                // Receiver dropped, stop walking
                                return ignore::WalkState::Quit;
                            }
                            ignore::WalkState::Continue
                        })
                    });
                }));

                // If walker panicked, log and send error through channel
                if let Err(panic_err) = result {
                    let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                        (*s).to_string()
                    } else if let Some(s) = panic_err.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };

                    eprintln!("galdi: filesystem walker thread panicked: {}", panic_msg);

                    // Try to send panic as an error (ignore if receiver already dropped)
                    let _ = tx_panic.send(Err(ScanError::Io(io::Error::other(format!(
                        "filesystem walker panicked: {}",
                        panic_msg
                    )))));
                }
            })
            .expect("failed to spawn filesystem walker thread");

        ScanIterator { receiver: rx }
    }

    pub fn scan(&self) -> Result<Snapshot, ScanError> {
        let start = Instant::now();
        let mut entries: Vec<SnapshotEntry> = Vec::new();
        let mut errors: Vec<ScanError> = Vec::new();

        // Reuse the streaming walker implementation to avoid duplication
        let iter = self.scan_iter();
        for item in iter {
            match item {
                Ok(entry) => entries.push(entry),
                Err(err) => errors.push(err),
            }
        }

        // Always sort for deterministic output in batch mode
        entries.sort_by(|a, b| a.path.cmp(&b.path));

        let status = if errors.is_empty() {
            Status::Ok
        } else {
            Status::Partial
        };

        Ok(Snapshot {
            version: "1.0".to_string(),
            root: self.options.root.clone(),
            checksum_algorithm: self.options.checksum_algorithm,
            plumbah: PlumbahObject::new(
                status,
                Meta::new(
                    "galdi_snapshot",
                    env!("CARGO_PKG_VERSION"),
                    true,
                    false,
                    true,
                    false,
                    start.elapsed().as_millis() as u64,
                    Utc::now(),
                ),
            ),
            count: entries.len(),
            entries,
        })
    }
}
fn to_unix_like_string(path: &std::path::Path) -> String {
    path.to_string_lossy().replace(MAIN_SEPARATOR, "/")
}

fn format_mode(metadata: &std::fs::Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        format!("{:o}", metadata.permissions().mode() & 0o7777)
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        let permissions = metadata.permissions();
        let readonly = permissions.readonly();
        // https://learn.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants
        let attrs = metadata.file_attributes();
        format!("{:08x}{}", attrs, if readonly { ",readonly" } else { "" })
    }
}
