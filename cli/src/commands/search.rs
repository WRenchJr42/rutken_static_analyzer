use ir::ApkIR;

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub kind: String,
    pub dex: String,
    pub class_name: Option<String>,
    pub method_name: Option<String>,
    pub value: String,
}

pub fn collect(ir: &ApkIR, query: &str) -> Vec<SearchMatch> {
    let query = query.to_lowercase();
    let mut matches = Vec::new();

    for dex_file in &ir.dex_files {
        for string in &dex_file.strings {
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

        for class in &dex_file.classes {
            if class.name.to_lowercase().contains(&query) {
                matches.push(SearchMatch {
                    kind: "class".to_string(),
                    dex: dex_file.name.clone(),
                    class_name: Some(class.name.clone()),
                    method_name: None,
                    value: class.name.clone(),
                });
            }

            for method in &class.methods {
                let method_name = method
                    .name
                    .split("->")
                    .last()
                    .unwrap_or(&method.name)
                    .to_string();
                if method_name.to_lowercase().contains(&query)
                    || method.name.to_lowercase().contains(&query)
                {
                    matches.push(SearchMatch {
                        kind: "method".to_string(),
                        dex: dex_file.name.clone(),
                        class_name: Some(class.name.clone()),
                        method_name: Some(method_name.clone()),
                        value: method.name.clone(),
                    });
                }

                for instruction in &method.instructions {
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

    matches
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
