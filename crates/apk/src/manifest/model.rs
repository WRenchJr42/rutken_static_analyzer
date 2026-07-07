use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AndroidManifest {
    pub package: Option<String>,
    pub min_sdk: Option<u32>,
    pub target_sdk: Option<u32>,
    pub permissions: Vec<String>,
    pub application: Option<Application>,
}


#[derive(Debug, Clone, Serialize)]
pub struct Application {
    pub label: Option<String>,
    pub debuggable: bool,
    pub allow_backup: bool,
    pub activities: Vec<Activity>,
}


#[derive(Debug, Clone, Serialize)]
pub struct Activity {
    pub name: String,
    pub exported: Option<bool>,
}
