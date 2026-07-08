use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    pub sha256: Option<String>,
    pub size: Option<u64>,
    pub dex_files: Vec<String>,
    pub architectures: Vec<String>,
}
