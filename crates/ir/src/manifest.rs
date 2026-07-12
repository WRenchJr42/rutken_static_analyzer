use serde::{Deserialize, Serialize};

/// Android manifest metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub package: Option<String>,
    pub min_sdk: Option<u32>,
    pub target_sdk: Option<u32>,
    pub permissions: Vec<String>,
    pub application: Option<Application>,
}

/// Application metadata from manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Application {
    pub label: Option<String>,
    pub debuggable: bool,
    pub allow_backup: bool,
    pub activities: Vec<Activity>,
}

/// Activity definition from manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    pub name: String,
    pub exported: Option<bool>,
}
