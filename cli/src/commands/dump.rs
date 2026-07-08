use serde::Serialize;

use ir::{ApkIR, DexFile, Instruction, Manifest};

use crate::commands::info;

#[derive(Debug, Clone, Serialize)]
pub struct ApkReport {
    pub info: info::InfoReport,
    pub manifest: Option<Manifest>,
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
    pub manifest: Option<Manifest>,
    pub dex: Vec<DexFile>,
}

pub fn build(ir: &ApkIR, include_strings: bool) -> ApkReport {
    let info = info::collect(ir);
    let manifest = ir.manifest.clone();
    let mut files = Vec::new();
    let mut strings = if include_strings {
        Some(Vec::new())
    } else {
        None
    };

    for dex_file in &ir.dex_files {
        if let Some(collected_strings) = strings.as_mut() {
            collected_strings.extend(dex_file.strings.iter().cloned());
        }

        files.push(DexFileInfo {
            name: dex_file.name.clone(),
            classes: dex_file
                .classes
                .iter()
                .map(|class| ClassInfo {
                    name: class.name.clone(),
                    methods: class
                        .methods
                        .iter()
                        .map(|method| MethodInfo {
                            name: method.name.clone(),
                            instructions: method.instructions.clone(),
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

    ApkReport {
        info,
        manifest,
        dex: DexSummary { files },
        strings,
    }
}

pub fn build_raw(ir: &ApkIR) -> RawApkReport {
    RawApkReport {
        info: info::collect(ir),
        manifest: ir.manifest.clone(),
        dex: ir.dex_files.clone(),
    }
}
