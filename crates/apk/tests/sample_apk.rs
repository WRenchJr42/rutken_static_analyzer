use std::path::PathBuf;

use apk::dex::model::build_dex_model;
use apk::reader::ApkReader;

fn sample_apk() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../samples/test.apk")
}

#[test]
fn reader_loads_all_dex_files_and_sorts_names() {
    let container = ApkReader::read(sample_apk()).expect("sample apk should parse");

    let dex_files: Vec<_> = container
        .dex_files
        .iter()
        .map(|dex| dex.name.as_str())
        .collect();
    assert_eq!(
        dex_files,
        vec![
            "classes.dex",
            "classes2.dex",
            "classes3.dex",
            "classes4.dex",
            "classes5.dex",
        ]
    );
}

#[test]
fn build_dex_model_resolves_method_names() {
    let container = ApkReader::read(sample_apk()).expect("sample apk should parse");

    let mut found = false;

    for dex_file in &container.dex_files {
        let model = build_dex_model(dex_file.name.clone(), &dex_file.bytes)
            .expect("dex model should build");

        if let Some(main_activity) = model
            .classes
            .iter()
            .find(|class| class.name.contains("MainActivity"))
        {
            assert!(
                main_activity
                    .methods
                    .iter()
                    .any(|method| method.name.ends_with("onClick"))
            );
            assert!(
                !main_activity
                    .methods
                    .iter()
                    .any(|method| method.name.contains("<bad_method:"))
            );
            found = true;
            break;
        }
    }

    assert!(
        found,
        "main activity class should exist in one of the dex files"
    );
}
