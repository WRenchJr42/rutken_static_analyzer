use crate::dex::model::{build_dex_model, DexModel};
use crate::errors::ApkError;
use crate::reader::ApkContainer;

pub fn collect(container: &ApkContainer) -> Result<Vec<DexModel>, ApkError> {
    let mut dex_models = Vec::new();

    for dex_file in &container.dex_files {
        dex_models.push(build_dex_model(dex_file.name.clone(), &dex_file.bytes)?);
    }

    Ok(dex_models)
}

pub fn format(container: &ApkContainer) -> Result<String, ApkError> {
    let mut output = String::new();

    for dex_file in &container.dex_files {
        let model = build_dex_model(dex_file.name.clone(), &dex_file.bytes)?;

        for class in model.classes {
            output.push_str(&format!("{}\n", class.name));
            output.push_str("  methods:\n");
            for method in class.methods {
                output.push_str(&format!("    {}\n", method.name.split("->").last().unwrap_or(&method.name)));
            }
            output.push('\n');
        }
    }

    Ok(output)
}
