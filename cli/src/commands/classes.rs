use ir::{ApkIR, DexFile};

pub fn collect(ir: &ApkIR) -> Vec<DexFile> {
    ir.dex_files.clone()
}

pub fn format(ir: &ApkIR) -> String {
    let mut output = String::new();

    for dex_file in &ir.dex_files {
        for class in &dex_file.classes {
            output.push_str(&format!("{}\n", class.name));
            output.push_str("  methods:\n");
            for method in &class.methods {
                output.push_str(&format!(
                    "    {}\n",
                    method.name.split("->").last().unwrap_or(&method.name)
                ));
            }
            output.push('\n');
        }
    }

    output
}
