use std::path::Path;
use std::fs;
use std::fs::File;
use crate::errors::ApkError;
use zip::ZipArchive;
use sha2::{Digest, Sha256};
use std::io::{BufReader, Read};
use crate::manifest::ManifestParser;
use crate::axml::parser::AxmlParser;
use crate::dex::parser::DexParser;
use crate::dex::class_data::ClassData;
use crate::binary::BinaryReader;
use crate::dex::code_item::CodeItem;

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

        let manifest = ManifestParser::extract(&mut archive)?;
        let document = AxmlParser::parse(&manifest)?;
        println!("Strings: {}, Resources: {}", document.string_pool.strings.len(), document.resource_map.resources.len());
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            if name.starts_with("classes") && name.ends_with(".dex") {
                let mut dex_bytes = Vec::new();
                file.read_to_end(&mut dex_bytes)?;
                let dex = DexParser::parse(&dex_bytes)?;
                println!("Strings: {}", dex.strings.strings.len());
                println!("Types: {}", dex.type_ids.types.len());
                println!("Protos: {}", dex.proto_ids.protos.len());

                println!("Methods: {}", dex.method_ids.methods.len());

                for method in dex.method_ids.methods.iter().take(20) {
                    let class = &dex.type_ids.types[method.class_idx as usize];
                    let class_name = &dex.strings.strings[class.descriptor_idx as usize];
                    let method_name = &dex.strings.strings[method.name_idx as usize];

                    println!(
                        "{} -> {}",
                        class_name,
                        method_name,
                    );
                }

                println!("Fields: {}", dex.field_ids.fields.len());
                for field in dex.field_ids.fields.iter().take(20) {
                    let class = &dex.type_ids.types[field.class_idx as usize];
                    let class_name = &dex.strings.strings[class.descriptor_idx as usize];
                    let field_name = &dex.strings.strings[field.name_idx as usize];
                    let field_type = &dex.type_ids.types[field.type_idx as usize];
                    let type_name = &dex.strings.strings[field_type.descriptor_idx as usize];
                    println!(
                        "{} -> {} : {}",
                        class_name,
                        field_name,
                        type_name
                    );
                }

                println!("Classes: {}", dex.class_defs.classes.len());
                for class in dex.class_defs.classes.iter().take(20) {
                    let class_type = &dex.type_ids.types[class.class_idx as usize];
                    let class_name = &dex.strings.strings[class_type.descriptor_idx as usize];
                    println!("CLASS: {}", class_name);
                    if class.class_data_off != 0 {
                        let data = ClassData::parse(&mut BinaryReader::new(&dex_bytes), class.class_data_off)?;
                        println!(
                            "Direct: {}, Virtual: {}",
                            data.direct_methods.len(),
                            data.virtual_methods.len()
                        );
                        for m in data.direct_methods.iter() {
                            println!("code_off: {}", m.code_off);
                            if m.code_off != 0 {
                                let code = CodeItem::parse(&mut BinaryReader::new(&dex_bytes), m.code_off)?;
                                println!("{:#?}", code);
                            }
                        }
                        for m in data.virtual_methods.iter() {
                            println!("code_off: {}", m.code_off);
                            if m.code_off != 0 {
                                let code = CodeItem::parse(&mut BinaryReader::new(&dex_bytes), m.code_off)?;
                                println!("{:#?}", code);
                            }
                        }
                    }
                }
            }
        }
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
