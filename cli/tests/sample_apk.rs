use std::path::PathBuf;

use cli::commands::{info, search};
use engine::Analyzer;

fn sample_apk() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../samples/test.apk")
}

fn sample_ir() -> ir::ApkIR {
    Analyzer::open(sample_apk())
        .expect("sample apk should open")
        .analyze()
        .expect("sample apk should analyze")
}

#[test]
fn info_counts_all_dex_files() {
    let ir = sample_ir();

    let report = info::collect(&ir);
    assert_eq!(report.classes, 2);
    assert_eq!(report.dex_files.len(), 5);
    assert!(report.architectures.is_empty());
}

#[test]
fn search_finds_methods_and_strings() {
    let ir = sample_ir();

    let results = search::collect(&ir, "admin");
    assert!(!results.is_empty());
    assert!(results.iter().any(|item| item.kind == "string"));
    assert!(results.iter().any(|item| item.kind == "instruction"));
}
