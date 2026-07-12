use serde::{Deserialize, Serialize};

/// APK file metadata: hash, size, and basic characteristics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    pub sha256: Option<String>,
    pub size: Option<u64>,
    pub dex_files: Vec<String>,
    pub architectures: Vec<String>,
}
