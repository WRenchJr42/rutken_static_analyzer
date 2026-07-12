use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

use sha2::{Digest, Sha256};
use zip::ZipArchive;

use crate::errors::ApkError;
use crate::manifest::ManifestParser;

#[derive(Debug, Clone)]
/// A DEX file extracted from an APK.
pub struct ApkDexFile {
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
/// Parsed APK contents: manifest, DEX files, and metadata.
pub struct ApkContainer {
    pub sha256: String,
    pub file_size: u64,
    pub entries: usize,
    pub dex_count: usize,
    pub architectures: Vec<String>,
    pub manifest: Vec<u8>,
    pub dex_files: Vec<ApkDexFile>,
}

struct ArchiveInfo {
    entries: usize,
    dex_count: usize,
    architectures: Vec<String>,
}

pub(crate) fn should_skip_class(name: &str) -> bool {
    name.starts_with("Ldalvik/")
        || name.starts_with("Landroid/")
        || name.starts_with("Landroidx/")
        || name.starts_with("Lkotlin/")
        || name.starts_with("Lkotlinx/")
        || name.starts_with("Lcom/google/")
        || name.starts_with("Lcom/android/tools/r8/")
        || name.starts_with("Lorg/intellij/")
        || name.starts_with("Lorg/jetbrains/")
        || name.starts_with("Lorg/jspecify/")
        || name.starts_with("L_COROUTINE/")
        || name.contains("/R$")
        || name.ends_with("/R;")
}

/// APK file parser.
pub struct ApkReader;

impl ApkReader {
    fn get_file_size(path: impl AsRef<Path>) -> Result<u64, ApkError> {
        Ok(fs::metadata(path)?.len())
    }

    fn compute_sha(path: impl AsRef<Path>) -> Result<String, ApkError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    fn analyze_archive<R>(archive: &mut ZipArchive<R>) -> Result<ArchiveInfo, ApkError>
    where
        R: Read + Seek,
    {
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

            if name.ends_with(".so")
                && let Some(rest) = name.strip_prefix("lib/")
                    && let Some(arch) = rest.split('/').next()
                        && !info.architectures.iter().any(|existing| existing == arch) {
                            info.architectures.push(arch.to_string());
                        }
        }

        Ok(info)
    }

    /// Parse an APK file and return its contents.
    pub fn read(path: impl AsRef<Path>) -> Result<ApkContainer, ApkError> {
        let file = File::open(&path)?;
        let mut archive = ZipArchive::new(file)?;

        let manifest = ManifestParser::extract(&mut archive)?;
        let archive_info = Self::analyze_archive(&mut archive)?;

        let mut dex_files = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if !name.starts_with("classes") || !name.ends_with(".dex") {
                continue;
            }

            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;

            dex_files.push(ApkDexFile { name, bytes });
        }

        dex_files.sort_by_key(|dex_file| dex_sort_key(&dex_file.name));

        Ok(ApkContainer {
            sha256: Self::compute_sha(&path)?,
            file_size: Self::get_file_size(&path)?,
            entries: archive_info.entries,
            dex_count: archive_info.dex_count,
            architectures: archive_info.architectures,
            manifest,
            dex_files,
        })
    }
}

fn dex_sort_key(name: &str) -> (u8, usize, String) {
    if !name.starts_with("classes") || !name.ends_with(".dex") {
        return (1, usize::MAX, name.to_string());
    }

    let middle = name
        .trim_start_matches("classes")
        .trim_end_matches(".dex");

    let index = if middle.is_empty() {
        1
    } else {
        middle.parse::<usize>().unwrap_or(usize::MAX)
    };

    (0, index, name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    #[test]
    fn analyze_archive_detects_architectures() {
        let mut bytes = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut bytes);
            let options = SimpleFileOptions::default();
            writer.start_file("classes.dex", options).expect("start classes.dex");
            writer.write_all(&[]).expect("write classes.dex");
            writer.start_file("lib/arm64-v8a/libfoo.so", options).expect("start libfoo");
            writer.write_all(&[]).expect("write libfoo");
            writer.start_file("lib/armeabi-v7a/libbar.so", options).expect("start libbar");
            writer.write_all(&[]).expect("write libbar");
            writer.finish().expect("finish zip");
        }

        bytes.set_position(0);
        let mut archive = zip::ZipArchive::new(bytes).expect("zip should open");
        let info = ApkReader::analyze_archive(&mut archive).expect("archive analysis should work");

        assert_eq!(info.dex_count, 1);
        assert_eq!(info.architectures, vec!["arm64-v8a".to_string(), "armeabi-v7a".to_string()]);
    }
}
