use serde::Serialize;

use crate::binary::BinaryReader;
use crate::commands::{parse_dex, parse_manifest};
use crate::dex::class_data::ClassData;
use crate::errors::ApkError;
use crate::reader::ApkContainer;

#[derive(Debug, Clone, Serialize)]
pub struct InfoReport {
    pub sha256: String,
    pub size: u64,
    pub dex_files: Vec<String>,
    pub architectures: Vec<String>,
    pub package: Option<String>,
    pub min_sdk: Option<u32>,
    pub target_sdk: Option<u32>,
    pub classes: usize,
    pub methods: usize,
    pub strings: usize,
    pub native: bool,
}

pub fn collect(container: &ApkContainer) -> Result<InfoReport, ApkError> {
    let manifest = parse_manifest(container)?;
    let mut classes = 0usize;
    let mut methods = 0usize;
    let mut strings = 0usize;

    for dex_file in &container.dex_files {
        let dex = parse_dex(dex_file)?;
        strings += dex.strings.strings.len();
        classes += dex.class_defs.classes.len();

        for class in &dex.class_defs.classes {
            if class.class_data_off == 0 {
                continue;
            }

            let data = ClassData::parse(&mut BinaryReader::new(&dex_file.bytes), class.class_data_off)?;
            methods += data.direct_methods.len() + data.virtual_methods.len();
        }
    }

    Ok(InfoReport {
        sha256: container.sha256.clone(),
        size: container.file_size,
        dex_files: container.dex_files.iter().map(|dex| dex.name.clone()).collect(),
        architectures: container.architectures.clone(),
        package: manifest.package,
        min_sdk: manifest.min_sdk,
        target_sdk: manifest.target_sdk,
        classes,
        methods,
        strings,
        native: !container.architectures.is_empty(),
    })
}
