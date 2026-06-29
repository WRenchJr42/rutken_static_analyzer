use std::path::Path;
use std::fs;
use std::fs::File;
use crate::errors::ApkError;
use zip::ZipArchive;
use sha2::{Digest, Sha256};
use std::io::{BufReader, Read};

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
    fn get_file_size(path: impl AsRef<Path>) -> Result<u64, ApkError>{
        let metadata = fs::metadata(path)?;
        Ok(metadata.len())
    }
    
    fn compute_sha(path: impl AsRef<Path>) -> Result<String, ApkError>{
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop{
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0{
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }


    pub fn read(path: impl AsRef<Path>) -> Result<ApkMetadata, ApkError> {
        let file = File::open(&path)?;
        let archive = ZipArchive::new(file)?;
        
        let metadata = ApkMetadata {
            sha256: Self::compute_sha(&path)?,
            file_size: Self::get_file_size(&path)?,
            entries: archive.len(),
            dex_count: 0,
            architectures: Vec::new(),
        };

        Ok(metadata)
    }
}
