use serde::{Deserialize, Serialize};

/// A security or quality finding in the APK.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub title: String,
    pub severity: FindingSeverity,
    pub description: Option<String>,
    pub location: Option<String>,
    pub evidence: Vec<String>,
}

/// Severity level of a finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}
