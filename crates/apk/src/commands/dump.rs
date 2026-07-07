use serde::Serialize;

use crate::commands::{info, parse_manifest};
use crate::dex::model::{build_dex_model, DexModel};
use crate::dex::instruction::Instruction;
use crate::errors::ApkError;
use crate::reader::ApkContainer;
use crate::manifest::model::AndroidManifest;

#[derive(Debug, Clone, Serialize)]
pub struct ApkReport {
    pub info: info::InfoReport,
    pub manifest: AndroidManifest,
    pub dex: DexSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strings: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DexSummary {
    pub files: Vec<DexFileInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DexFileInfo {
    pub name: String,
    pub classes: Vec<ClassInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<MethodInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MethodInfo {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RawApkReport {
    pub info: info::InfoReport,
    pub manifest: AndroidManifest,
    pub dex: Vec<DexModel>,
}

pub fn build(container: &ApkContainer, include_strings: bool) -> Result<ApkReport, ApkError> {
    let info = info::collect(container)?;
    let manifest = parse_manifest(container)?;
    let mut files = Vec::new();
    let mut strings = if include_strings { Some(Vec::new()) } else { None };

    for dex_file in &container.dex_files {
        let model = build_dex_model(dex_file.name.clone(), &dex_file.bytes)?;

        if let Some(collected_strings) = strings.as_mut() {
            collected_strings.extend(model.strings.iter().cloned());
        }

        files.push(DexFileInfo {
            name: model.name,
            classes: model
                .classes
                .into_iter()
                .map(|class| ClassInfo {
                    name: class.name,
                    methods: class
                        .methods
                        .into_iter()
                        .map(|method| MethodInfo {
                            name: method.name,
                            instructions: method.instructions,
                        })
                        .collect(),
                })
                .collect(),
        });
    }

    if let Some(collected_strings) = strings.as_mut() {
        collected_strings.sort();
        collected_strings.dedup();
    }

    Ok(ApkReport {
        info,
        manifest,
        dex: DexSummary { files },
        strings,
    })
}

pub fn build_raw(container: &ApkContainer) -> Result<RawApkReport, ApkError> {
    let info = info::collect(container)?;
    let manifest = parse_manifest(container)?;
    let mut dex = Vec::new();

    for dex_file in &container.dex_files {
        dex.push(build_dex_model(dex_file.name.clone(), &dex_file.bytes)?);
    }

    Ok(RawApkReport { info, manifest, dex })
}
