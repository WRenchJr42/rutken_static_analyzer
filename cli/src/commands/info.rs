use serde::Serialize;

use ir::ApkIR;

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

pub fn collect(ir: &ApkIR) -> InfoReport {
    let classes = ir.dex_files.iter().map(|dex| dex.classes.len()).sum();
    let methods = ir
        .dex_files
        .iter()
        .flat_map(|dex| &dex.classes)
        .map(|class| class.methods.len())
        .sum();
    let strings = ir.dex_files.iter().map(|dex| dex.strings.len()).sum();

    InfoReport {
        sha256: ir.metadata.sha256.clone().unwrap_or_default(),
        size: ir.metadata.size.unwrap_or_default(),
        dex_files: ir.metadata.dex_files.clone(),
        architectures: ir.metadata.architectures.clone(),
        package: ir
            .manifest
            .as_ref()
            .and_then(|manifest| manifest.package.clone()),
        min_sdk: ir.manifest.as_ref().and_then(|manifest| manifest.min_sdk),
        target_sdk: ir
            .manifest
            .as_ref()
            .and_then(|manifest| manifest.target_sdk),
        classes,
        methods,
        strings,
        native: !ir.metadata.architectures.is_empty(),
    }
}
