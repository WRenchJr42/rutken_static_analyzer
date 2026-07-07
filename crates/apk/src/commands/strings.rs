use crate::commands::parse_dex;
use crate::errors::ApkError;
use crate::reader::ApkContainer;

pub fn collect(container: &ApkContainer, grep: Option<&str>) -> Result<Vec<String>, ApkError> {
    let mut strings = Vec::new();

    for dex_file in &container.dex_files {
        let dex = parse_dex(dex_file)?;

        for value in dex.strings.strings {
            if grep.map(|needle| value.contains(needle)).unwrap_or(true) {
                strings.push(value);
            }
        }
    }

    Ok(strings)
}
