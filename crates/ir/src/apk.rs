use serde::{Deserialize, Serialize};

use crate::dex::DexFile;
use crate::findings::Finding;
use crate::manifest::Manifest;
use crate::metadata::Metadata;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApkIR {
    pub metadata: Metadata,
    pub manifest: Option<Manifest>,
    pub dex_files: Vec<DexFile>,
    pub findings: Vec<Finding>,
}
