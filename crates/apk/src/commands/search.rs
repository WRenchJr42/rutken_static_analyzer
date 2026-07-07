use crate::dex::model::build_dex_model;
use crate::errors::ApkError;
use crate::reader::ApkContainer;

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub kind: String,
    pub dex: String,
    pub class_name: Option<String>,
    pub method_name: Option<String>,
    pub value: String,
}

pub fn collect(container: &ApkContainer, query: &str) -> Result<Vec<SearchMatch>, ApkError> {
    let query = query.to_lowercase();
    let mut matches = Vec::new();

    for dex_file in &container.dex_files {
        let model = build_dex_model(dex_file.name.clone(), &dex_file.bytes)?;

        for string in &model.strings {
            if string.to_lowercase().contains(&query) {
                matches.push(SearchMatch {
                    kind: "string".to_string(),
                    dex: dex_file.name.clone(),
                    class_name: None,
                    method_name: None,
                    value: string.clone(),
                });
            }
        }

        for class in model.classes {
            if class.name.to_lowercase().contains(&query) {
                matches.push(SearchMatch {
                    kind: "class".to_string(),
                    dex: dex_file.name.clone(),
                    class_name: Some(class.name.clone()),
                    method_name: None,
                    value: class.name.clone(),
                });
            }

            for method in class.methods {
                let method_name = method.name.split("->").last().unwrap_or(&method.name).to_string();
                if method_name.to_lowercase().contains(&query) || method.name.to_lowercase().contains(&query) {
                    matches.push(SearchMatch {
                        kind: "method".to_string(),
                        dex: dex_file.name.clone(),
                        class_name: Some(class.name.clone()),
                        method_name: Some(method_name.clone()),
                        value: method.name,
                    });
                }

                for instruction in method.instructions {
                    let text = format!("{:?}", instruction);
                    if text.to_lowercase().contains(&query) {
                        matches.push(SearchMatch {
                            kind: "instruction".to_string(),
                            dex: dex_file.name.clone(),
                            class_name: Some(class.name.clone()),
                            method_name: Some(method_name.clone()),
                            value: text,
                        });
                    }
                }
            }
        }
    }

    Ok(matches)
}

pub fn format(matches: &[SearchMatch]) -> String {
    let mut output = String::new();

    for item in matches {
        output.push_str(&format!("[{}] {}", item.kind, item.dex));
        if let Some(class_name) = &item.class_name {
            output.push_str(&format!(" {}", class_name));
        }
        if let Some(method_name) = &item.method_name {
            output.push_str(&format!(" -> {}", method_name));
        }
        output.push_str(&format!("\n  {}\n", item.value));
    }

    output
}
