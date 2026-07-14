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

#[test]
fn analyzer_resolves_mainactivity_and_onclick_without_bad_sentinels() {
    let ir = Analyzer::open(sample_apk())
        .expect("sample apk should open")
        .analyze()
        .expect("sample apk should analyze");

    // Find MainActivity in the IR
    let mut found_activity = false;
    let mut found_onclick = false;

    for dex_file in &ir.dex_files {
        for class in &dex_file.classes {
            if class.name.contains("MainActivity") {
                found_activity = true;

                // Check that no bad sentinels appear in the resolved methods and instructions
                for method in &class.methods {
                    assert!(
                        !method.name.contains("<bad_"),
                        "MainActivity method should not contain bad sentinels: {}",
                        method.name
                    );

                    if method.name.contains("onClick") {
                        found_onclick = true;
                    }

                    // Check all instructions for bad sentinels
                    for instruction_at in &method.instructions {
                        match &instruction_at.instruction {
                            ir::Instruction::Invoke { method, .. } => {
                                let display = method.display(&dex_file.strings);
                                assert!(
                                    !display.contains("<bad_"),
                                    "invoke in MainActivity should not have bad sentinels: {}",
                                    display
                                );
                            }
                            ir::Instruction::ConstString { value, .. } => {
                                let resolved = value.resolve(&dex_file.strings);
                                assert!(
                                    !resolved.contains("<bad_"),
                                    "const-string in MainActivity should not have bad sentinels: {}",
                                    resolved
                                );
                            }
                            ir::Instruction::CheckCast { class } => {
                                let display = class.display(&dex_file.strings);
                                assert!(
                                    !display.contains("<bad_"),
                                    "check-cast in MainActivity should not have bad sentinels: {}",
                                    display
                                );
                            }
                            ir::Instruction::NewInstance { class } => {
                                let display = class.display(&dex_file.strings);
                                assert!(
                                    !display.contains("<bad_"),
                                    "new-instance in MainActivity should not have bad sentinels: {}",
                                    display
                                );
                            }
                            ir::Instruction::FieldAccess { field } => {
                                let display = field.display(&dex_file.strings);
                                assert!(
                                    !display.contains("<bad_"),
                                    "field access in MainActivity should not have bad sentinels: {}",
                                    display
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    assert!(
        found_activity,
        "MainActivity should be found in the IR"
    );
    assert!(
        found_onclick,
        "onClick method should be found in MainActivity"
    );
}
