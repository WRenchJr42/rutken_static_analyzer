use std::path::Path;
use crate::errors::ApkError;

#[derive(Debug)]
pub struct ApkMetadata {
    ///SHA256 hash of APK 
    pub sha256: String, 
    ///Size of APK 
    pub file_size: u64,    
    ///Number of DEX files 
    pub dex_count: usize,
    ///Native architectures under lib/
    pub architectures: Vec<String>,
    ///Number of files inside APK 
    pub entries: usize,
}

//Read APK Files
pub struct ApkReader; 
impl ApkReader {
    pub fn read(path: impl AsRef<Path>) -> Result<ApkMetadata, ApkError> {
        todo!()
    }
}
