use crate::dex::parser::DexDocument;
use crate::dex::opcode::opcode_width;
use crate::dex::instruction::{
    Instruction,
    InvokeKind,
    BranchKind,
};

pub fn decode_instruction(insns: &[u16], pc: usize, dex: &DexDocument) -> (Instruction, usize) {
    let ins = insns[pc];
    match ins {
        0x0100 | 0x0200 | 0x0300 => {
            return (
                Instruction::Payload,
                insns.len() - pc
            );
        }

        _ => {}
    }

    let opcode = (ins & 0xff) as u8;
    match opcode {

        0x00 => (
            Instruction::Nop,
            1
        ),

        0x0c => {
            let reg = (ins >> 8) as u8;
            (
                Instruction::MoveResult {
                    register: reg
                },
                1
            )
        }

        0x0e | 0x0f | 0x10 => (
            Instruction::Return,
            1
        ),

        0x12 => {
            let reg = ((ins >> 8) & 0xf) as u8;
            let value = ((ins >> 12)&0xf) as i32;
            (
                Instruction::Const {
                    register: reg,
                    value
                },
                1
            )
        }

        0x13 => {
            let reg = (ins >> 8) as u8;
            let value = get(insns, pc + 1) as i16 as i32;
            (
                Instruction::Const {
                    register: reg,
                    value,
                },
                2
            )
        }

        0x14 => {
            let reg = (ins >> 8) as u8;
            let value = ((get(insns, pc + 1) as i16) as i32) << 16;
            (
                Instruction::Const {
                    register: reg,
                    value,
                },
                2
            )
        }

        0x1a => {
            let reg = (ins >> 8) as u8;
            let idx = get(insns, pc + 1) as usize;
            let value =
                dex.strings
                    .strings
                    .get(idx)
                    .cloned()
                    .unwrap_or(
                        format!(
                            "<bad_string:{}>",
                            idx
                        )
                    );
            (
                Instruction::ConstString {
                    register: reg,
                    value,
                },
                2
            )

        }

        0x1f => {
            let idx = get(insns,pc+1) as usize;
            (
                Instruction::CheckCast {
                    class:
                        resolve_type(
                            idx,
                            dex
                        )
                },
                2
            )
        }

        0x22 => {
            let idx = get(insns,pc+1) as usize;
            (
                Instruction::NewInstance {
                    class:
                        resolve_type(
                            idx,
                            dex
                        )
                },
                2
            )
        }

        0x23 => (
            Instruction::Unknown {
                opcode,
                raw:ins
            },
            2
        ),

        0x24 => (
            Instruction::Unknown {
                opcode,
                raw:ins
            },
            3
        ),

        0x26 => (
            Instruction::Unknown {
                opcode,
                raw:ins
            },
            3
        ),

        0x27 => (
            Instruction::Throw,
            1
        ),

        0x28 => (
            Instruction::Branch {
                kind:
                    BranchKind::Goto
            },
            1
        ),

        0x29 => (
            Instruction::Branch {
                kind:
                    BranchKind::Goto
            },
            2
        ),

        0x2a => (
            Instruction::Branch {
                kind:
                    BranchKind::Goto
            },
            3
        ),

        // switch
        0x2b | 0x2c => (
            Instruction::Unknown {
                opcode,
                raw:ins
            },
            3
        ),

        // if family
        0x32..=0x3d => (
            Instruction::Branch {
                kind:
                    BranchKind::IfEqz
            },
            2
        ),

        0x52..=0x5f => {
            let idx = get(insns,pc+1) as usize;
            (
                Instruction::FieldAccess {
                    field:
                        resolve_field(
                            idx,
                            dex
                        )
                },
                2
            )
        }

        0x60..=0x6d => {
            let idx = get(insns,pc+1) as usize;
            (
                Instruction::FieldAccess {
                    field:
                        resolve_field(
                            idx,
                            dex
                        )
                },
                2
            )
        }

        0x6e => invoke(
            ins,pc,insns,dex,
            InvokeKind::Virtual
        ),

        0x6f => invoke(
            ins,pc,insns,dex,
            InvokeKind::Super
        ),

        0x70 => invoke(
            ins,pc,insns,dex,
            InvokeKind::Direct
        ),

        0x71 => invoke(
            ins,pc,insns,dex,
            InvokeKind::Static
        ),

        0x72 => invoke(
            ins,pc,insns,dex,
            InvokeKind::Interface
        ),

        _ => {
                let size = opcode_width(opcode);
                (
                Instruction::Unknown {
                    opcode,
                    raw:ins
                },
                size
            )
        }
    }
}

fn invoke(
    first:u16,
    pc:usize,
    insns:&[u16],
    dex:&DexDocument,
    kind:InvokeKind

) -> (Instruction,usize) {

    let method_idx = get(insns,pc+1) as usize;
    let regs = decode_35c_registers( first, get(insns,pc+2)); 
    (
        Instruction::Invoke {
            kind,
            method:
                resolve_method(
                    method_idx,
                    dex
                ),
            registers:regs,
        },
        3
    )
}

fn resolve_method(idx:usize, dex:&DexDocument) -> String {
    let Some(m)= dex.method_ids.methods.get(idx)
    else {
        return format!(
            "<bad_method:{}>",
            idx
        );
    };

    format!(
        "{}->{}",
        resolve_type(
            m.class_idx as usize,
            dex
        ),

        dex.strings.strings
        .get(m.name_idx as usize)
        .unwrap_or(
            &"<bad>".into()
        )
    )
}

fn resolve_field(idx:usize, dex:&DexDocument) -> String {
    let Some(f)= dex.field_ids.fields.get(idx)
    else {
        return format!(
            "<bad_field:{}>",
            idx
        );
    };

    format!(
        "{}->{}",
        resolve_type(
            f.class_idx as usize,
            dex
        ),
        dex.strings.strings.get(f.name_idx as usize).unwrap_or(&"<bad>".into())
    )
}

fn resolve_type(idx:usize, dex:&DexDocument) -> String{
    let Some(t) = dex.type_ids.types.get(idx)
    else {
        return format!(
            "<bad_type:{}>",
            idx
        );
    };
    dex.strings.strings.get(t.descriptor_idx as usize).cloned().unwrap_or("<bad_descriptor>".into())
}

fn decode_35c_registers(first:u16, third:u16) -> Vec<u8> {
    let count = (first>>12) as usize;
    let regs=[
        (third&0xf) as u8,
        ((third>>4)&0xf) as u8,
        ((third>>8)&0xf) as u8,
        ((third>>12)&0xf) as u8,
        ((first>>8)&0xf) as u8,
    ];
    regs[..count.min(5)].to_vec()
}

fn get(data:&[u16], idx:usize) -> u16 {
    *data.get(idx).unwrap_or(&0)
}
