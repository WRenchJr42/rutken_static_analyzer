use std::path::Path;
use std::fs;
use std::fs::File;
use crate::errors::ApkError;
use zip::ZipArchive;
use sha2::{Digest, Sha256};
use std::io::{BufReader, Read};
use crate::manifest::ManifestParser;
use crate::axml::parser::AxmlParser;

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

struct ArchiveInfo {
    ///Number of files in the APK 
    entries: usize,
    ///Number of DEX files 
    dex_count: usize,
    ///Native architectures
    architectures: Vec<String>,
}

//Read APK Files
pub struct ApkReader; 
impl ApkReader {
    fn get_file_size(path: impl AsRef<Path>) -> Result<u64, ApkError> {
        let metadata = fs::metadata(path)?;
        Ok(metadata.len())
    }
    
    fn compute_sha(path: impl AsRef<Path>) -> Result<String, ApkError> {
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

    fn analyze_archive<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Result<ArchiveInfo, ApkError> {
        let mut info = ArchiveInfo {
            entries: archive.len(),
            dex_count: 0,
            architectures: Vec::new(),
        };

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name();
            if name.starts_with("classes") && name.ends_with(".dex") {
                info.dex_count += 1;
            }
            if name.ends_with(".so") {
                if let Some(rest) = name.strip_prefix("lib/") {
                    if let Some(arch) = rest.split('/').next() {
                        if !info.architectures.iter().any(|a| a == arch) {
                            info.architectures.push(arch.to_string());
                        }
                    }
                }
            }
        }
        Ok(info)
    }

    pub fn read(path: impl AsRef<Path>) -> Result<ApkMetadata, ApkError> {
        let file = File::open(&path)?;
        let mut archive = ZipArchive::new(file)?;

        let manifest_bytes = ManifestParser::extract(&mut archive)?;
        println!("Manifest size: {} bytes", manifest_bytes.len());
        
        let manifest = ManifestParser::extract(&mut archive)?;
        let header = AxmlParser::parse(&manifest)?;
        println!("{:#?}", header);

        let archive_info = Self::analyze_archive(&mut archive)?;
        Ok(ApkMetadata {
            sha256: Self::compute_sha(&path)?,
            file_size: Self::get_file_size(&path)?,
            entries: archive_info.entries,
            dex_count: archive_info.dex_count,
            architectures: archive_info.architectures,
        })
    }
}
