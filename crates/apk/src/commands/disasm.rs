use crate::dex::model::build_dex_model;
use crate::errors::ApkError;
use crate::reader::ApkContainer;

pub fn render(container: &ApkContainer, query: &str) -> Result<String, ApkError> {
    let mut output = String::new();

    for dex_file in &container.dex_files {
        let model = build_dex_model(dex_file.name.clone(), &dex_file.bytes)?;

        for class in &model.classes {
            if !class.name.contains(query) {
                continue;
            }

            output.push_str(&format!("{}\n", class.name));

            for method in &class.methods {
                output.push_str(&format!("\n{}\n", method.name.split("->").last().unwrap_or(&method.name)));
                for instruction in &method.instructions {
                    output.push_str(&format!("  {}\n", format_instruction(instruction)));
                }
            }
        }
    }

    Ok(output)
}

fn format_instruction(instruction: &crate::dex::instruction::Instruction) -> String {
    match instruction {
        crate::dex::instruction::Instruction::Const { register, value } => {
            format!("const v{}, {}", register, value)
        }
        crate::dex::instruction::Instruction::ConstString { register, value } => {
            format!("const-string v{}, \"{}\"", register, value)
        }
        crate::dex::instruction::Instruction::Invoke { kind, method, registers } => {
            let kind = match kind {
                crate::dex::instruction::InvokeKind::Static => "invoke-static",
                crate::dex::instruction::InvokeKind::Virtual => "invoke-virtual",
                crate::dex::instruction::InvokeKind::Direct => "invoke-direct",
                crate::dex::instruction::InvokeKind::Super => "invoke-super",
                crate::dex::instruction::InvokeKind::Interface => "invoke-interface",
            };
            let registers = registers.iter().map(|register| format!("v{}", register)).collect::<Vec<_>>().join(", ");
            format!("{} {{ {} }} {}", kind, registers, method)
        }
        crate::dex::instruction::Instruction::FieldAccess { field } => {
            format!("field-access {}", field)
        }
        crate::dex::instruction::Instruction::NewInstance { class } => {
            format!("new-instance {}", class)
        }
        crate::dex::instruction::Instruction::CheckCast { class } => {
            format!("check-cast {}", class)
        }
        crate::dex::instruction::Instruction::MoveResult { register } => {
            format!("move-result v{}", register)
        }
        crate::dex::instruction::Instruction::Return => "return".to_string(),
        crate::dex::instruction::Instruction::Throw => "throw".to_string(),
        crate::dex::instruction::Instruction::Nop => "nop".to_string(),
        crate::dex::instruction::Instruction::Payload => "payload".to_string(),
        crate::dex::instruction::Instruction::Branch { kind } => format!("branch {:?}", kind),
        crate::dex::instruction::Instruction::Unknown { opcode, raw } => {
            format!("unknown 0x{:02x} 0x{:04x}", opcode, raw)
        }
    }
}
