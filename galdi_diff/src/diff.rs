use std::{
    collections::{BTreeSet, HashMap},
    time::Instant,
};

use galdi_core::{
    AttributeChange, ChangeType, DiffResult, DiffSummary, Difference, PlumbahObject, Snapshot,
    SnapshotEntry,
};

pub struct DiffEngine {
    ignore_time: bool,
    ignore_mode: bool,
    structure_only: bool,
}

pub struct DiffOptions {
    pub ignore_time: bool,
    pub ignore_mode: bool,
    pub structure_only: bool,
}

impl DiffEngine {
    pub fn new(options: DiffOptions) -> Self {
        Self {
            ignore_time: options.ignore_time,
            ignore_mode: options.ignore_mode,
            structure_only: options.structure_only,
        }
    }
    pub fn diff(&self, source: &Snapshot, target: &Snapshot) -> DiffResult {
        let start = Instant::now();
        let mut differences = Vec::new();
        let mut summary = DiffSummary::default();

        // Create lookup maps
        let source_map: HashMap<_, _> = source
            .entries
            .iter()
            .map(|e| (e.path.clone(), e.clone()))
            .collect();
        let target_map: HashMap<_, _> = target
            .entries
            .iter()
            .map(|e| (e.path.clone(), e.clone()))
            .collect();

        // Find all unique paths
        let all_paths: BTreeSet<_> = source_map.keys().chain(target_map.keys()).collect();

        for path in all_paths {
            match (source_map.get(path), target_map.get(path)) {
                (Some(src), Some(tgt)) => {
                    // Entry exists in both
                    if let Some(diff) = self.compare_entries(src, tgt) {
                        summary.modified += 1;
                        differences.push(diff);
                    } else {
                        summary.unchanged += 1;
                    }
                }
                (Some(src), None) => {
                    // Removed
                    summary.removed += 1;
                    differences.push(Difference {
                        path: path.clone(),
                        change_type: ChangeType::Removed,
                        changes: vec![],
                        source: Some(src.clone()),
                        target: None,
                        error: None,
                    });
                }
                (None, Some(tgt)) => {
                    // Added
                    summary.added += 1;
                    differences.push(Difference {
                        path: path.clone(),
                        change_type: ChangeType::Added,
                        changes: vec![],
                        source: None,
                        target: Some(tgt.clone()),
                        error: None,
                    });
                }
                (None, None) => unreachable!(),
            }
        }

        // Sort for determinism
        differences.sort_by(|a, b| a.path.cmp(&b.path));

        DiffResult {
            plumbah: PlumbahObject::new(
                galdi_core::Status::Ok,
                galdi_core::Meta::new(
                    "galdi_diff",
                    env!("CARGO_PKG_VERSION"),
                    true,
                    false,
                    true,
                    true,
                    start.elapsed().as_millis() as u64,
                    chrono::Utc::now(),
                ),
            ),
            identical: differences.is_empty(),
            summary,
            differences,
        }
    }

    fn compare_entries(&self, src: &SnapshotEntry, tgt: &SnapshotEntry) -> Option<Difference> {
        let mut changes = Vec::new();

        if src.entry_type != tgt.entry_type {
            changes.push(AttributeChange::Type);
        }

        if !self.structure_only {
            if src.checksum != tgt.checksum {
                changes.push(AttributeChange::Content);
            }

            if !self.ignore_mode && src.mode != tgt.mode {
                changes.push(AttributeChange::Mode);
            }

            if !self.ignore_time && src.mtime != tgt.mtime {
                changes.push(AttributeChange::Mtime);
            }

            if src.size != tgt.size {
                changes.push(AttributeChange::Size);
            }

            if src.target != tgt.target {
                changes.push(AttributeChange::Target);
            }
        }

        if changes.is_empty() {
            None
        } else {
            Some(Difference {
                path: src.path.clone(),
                change_type: ChangeType::Modified,
                changes,
                source: Some(src.clone()),
                target: Some(tgt.clone()),
                error: None,
            })
        }
    }
}
