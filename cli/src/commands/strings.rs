use ir::ApkIR;

pub fn collect(ir: &ApkIR, grep: Option<&str>) -> Vec<String> {
    let mut strings = Vec::new();

    for dex_file in &ir.dex_files {
        for value in &dex_file.strings {
            if grep.map(|needle| value.contains(needle)).unwrap_or(true) {
                strings.push(value.clone());
            }
        }
    }

    strings
}
