use ir::{ApkIR, Instruction, InvokeKind};

pub fn render(ir: &ApkIR, query: &str) -> String {
    let mut output = String::new();

    for dex_file in &ir.dex_files {
        for class in &dex_file.classes {
            if !class.name.contains(query) {
                continue;
            }

            output.push_str(&format!("{}\n", class.name));

            for method in &class.methods {
                output.push_str(&format!(
                    "\n{}\n",
                    method.name.split("->").last().unwrap_or(&method.name)
                ));
                for instruction_at in &method.instructions {
                    output.push_str(&format!(
                        "  {:04x}: {}\n",
                        instruction_at.offset,
                        format_instruction(&instruction_at.instruction, &dex_file.strings)
                    ));
                }
            }
        }
    }

    output
}

/// Render an instruction to a human-readable line, resolving string/class/
/// method/field references against the owning DEX file's string pool.
pub(crate) fn format_instruction(instruction: &Instruction, strings: &[String]) -> String {
    match instruction {
        Instruction::Const { register, value } => {
            format!("const v{}, {}", register, value)
        }
        Instruction::ConstString { register, value } => {
            format!("const-string v{}, \"{}\"", register, value.resolve(strings))
        }
        Instruction::Invoke {
            kind,
            method,
            registers,
        } => {
            let kind = match kind {
                InvokeKind::Static => "invoke-static",
                InvokeKind::Virtual => "invoke-virtual",
                InvokeKind::Direct => "invoke-direct",
                InvokeKind::Super => "invoke-super",
                InvokeKind::Interface => "invoke-interface",
            };
            let registers = registers
                .iter()
                .map(|register| format!("v{}", register))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} {{ {} }} {}", kind, registers, method.display(strings))
        }
        Instruction::FieldAccess { field } => {
            format!("field-access {}", field.display(strings))
        }
        Instruction::NewInstance { class } => {
            format!("new-instance {}", class.display(strings))
        }
        Instruction::CheckCast { class } => {
            format!("check-cast {}", class.display(strings))
        }
        Instruction::MoveResult { register } => {
            format!("move-result v{}", register)
        }
        Instruction::Return => "return".to_string(),
        Instruction::Throw => "throw".to_string(),
        Instruction::Nop => "nop".to_string(),
        Instruction::Payload => "payload".to_string(),
        Instruction::Branch { kind, target } => {
            format!("branch {:?} -> 0x{:04x}", kind, target)
        }
        Instruction::Switch { packed, cases } => {
            let kind = if *packed {
                "packed-switch"
            } else {
                "sparse-switch"
            };
            let cases = cases
                .iter()
                .map(|c| format!("{}:0x{:04x}", c.key, c.target))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} {{ {} }}", kind, cases)
        }
        Instruction::Unknown { opcode, raw } => {
            format!("unknown 0x{:02x} 0x{:04x}", opcode, raw)
        }
    }
}
