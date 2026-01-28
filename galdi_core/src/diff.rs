use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{PlumbahObject, snapshot::SnapshotEntry};
#[derive(Debug, Serialize, Deserialize)]
pub struct DiffResult {
    #[serde(rename = "$plumbah")]
    pub plumbah: PlumbahObject,
    pub identical: bool,
    pub summary: DiffSummary,
    pub differences: Vec<Difference>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffSummary {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub unchanged: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difference {
    pub path: PathBuf,
    pub change_type: ChangeType,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<AttributeChange>,
    pub source: Option<SnapshotEntry>,
    pub target: Option<SnapshotEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    PermissionDenied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttributeChange {
    Content,
    Mode,
    Mtime,
    Type,
    Size,
    Target,
}
