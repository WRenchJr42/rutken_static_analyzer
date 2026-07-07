pub mod classes;
pub mod disasm;
pub mod dump;
pub mod info;
pub mod manifest;
pub mod search;
pub mod strings;

use crate::axml::parser::AxmlParser;
use crate::dex::parser::{DexDocument, DexParser};
use crate::errors::ApkError;
use crate::manifest::decoder::decode_manifest;
use crate::manifest::model::AndroidManifest;
use crate::reader::{ApkContainer, ApkDexFile};

pub(crate) fn parse_manifest(container: &ApkContainer) -> Result<AndroidManifest, ApkError> {
    let document = AxmlParser::parse(&container.manifest)?;

    Ok(match document.root {
        Some(root) => decode_manifest(&root),
        None => AndroidManifest {
            package: None,
            min_sdk: None,
            target_sdk: None,
            permissions: Vec::new(),
            application: None,
        },
    })
}

pub(crate) fn parse_dex(dex_file: &ApkDexFile) -> Result<DexDocument, ApkError> {
    DexParser::parse(&dex_file.bytes)
}
