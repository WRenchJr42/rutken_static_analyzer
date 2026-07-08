use std::path::PathBuf;

use engine::Analyzer;

fn sample_apk() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../samples/test.apk")
}

#[test]
fn analyzer_builds_apk_ir() {
    let ir = Analyzer::open(sample_apk())
        .expect("sample apk should open")
        .analyze()
        .expect("sample apk should analyze");

    assert_eq!(ir.metadata.dex_files.len(), 5);
    assert!(ir.metadata.sha256.is_some());
    assert!(ir.manifest.is_some());
    assert_eq!(ir.dex_files.len(), 5);
    assert!(
        ir.dex_files
            .iter()
            .flat_map(|dex| &dex.classes)
            .any(|class| class.name.contains("MainActivity"))
    );
}
