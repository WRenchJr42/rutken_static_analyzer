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

#[test]
fn build_dex_model_produces_no_bad_sentinels_for_well_formed_input() {
    let container = ApkReader::read(sample_apk()).expect("sample apk should parse");

    for dex_file in &container.dex_files {
        let model = build_dex_model(dex_file.name.clone(), &dex_file.bytes)
            .expect("dex model should build");

        // Check that well-formed input does not produce sentinel values for types, methods, fields, or strings
        for class in &model.classes {
            for method in &class.methods {
                assert!(
                    !method.name.contains("<bad_method:"),
                    "well-formed input should not contain bad_method sentinels, found in method: {}",
                    method.name
                );
                assert!(
                    !method.name.contains("<bad_type:"),
                    "well-formed input should not contain bad_type sentinels, found in method: {}",
                    method.name
                );

                for instruction in &method.instructions {
                    // Well-formed input should resolve every operand to a
                    // valid index into the DEX file's string pool.
                    let in_range = |idx: u32| (idx as usize) < model.strings.len();

                    match instruction {
                        apk::dex::instruction::Instruction::Invoke {
                            class_idx,
                            name_idx,
                            descriptor_idx,
                            ..
                        } => {
                            assert!(
                                in_range(*class_idx) && in_range(*name_idx) && in_range(*descriptor_idx),
                                "invoke instruction should not have out-of-range string indices: {:?}",
                                instruction
                            );
                        }
                        apk::dex::instruction::Instruction::ConstString { string_idx, .. } => {
                            assert!(
                                in_range(*string_idx),
                                "const-string should not have an out-of-range string index: {}",
                                string_idx
                            );
                        }
                        apk::dex::instruction::Instruction::CheckCast { class_idx } => {
                            assert!(
                                in_range(*class_idx),
                                "check-cast should not have an out-of-range string index: {}",
                                class_idx
                            );
                        }
                        apk::dex::instruction::Instruction::NewInstance { class_idx } => {
                            assert!(
                                in_range(*class_idx),
                                "new-instance should not have an out-of-range string index: {}",
                                class_idx
                            );
                        }
                        apk::dex::instruction::Instruction::FieldAccess {
                            class_idx,
                            name_idx,
                            type_idx,
                        } => {
                            assert!(
                                in_range(*class_idx) && in_range(*name_idx) && in_range(*type_idx),
                                "field access should not have out-of-range string indices: {:?}",
                                instruction
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
