use crate::errors::ApkError;
use std::io::{Read, Seek};
use zip::ZipArchive;

pub struct ManifestParser;

impl ManifestParser {
    pub fn extract<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Result<Vec<u8>, ApkError> {
        let mut file = archive.by_name("AndroidManifest.xml")?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}
