use crate::dex::parser::DexDocument;
use crate::dex::opcode::opcode_width;
use crate::dex::instruction::{
    Instruction,
    InvokeKind,
    BranchKind,
};

/// Decode a single instruction at the given program counter.
///
/// Returns the decoded instruction and its size in u16s.
/// If `pc` is out-of-range, returns an Unknown instruction with size 0;
/// callers detect end-of-code by checking if size > 0.
pub fn decode_instruction(insns: &[u16], pc: usize, dex: &DexDocument) -> (Instruction, usize) {
    // `decode_instruction` is a public API; callers may pass an out-of-range
    // `pc`. Returning a zero-size `Unknown` instruction (rather than
    // panicking) lets callers detect the invalid position via `size == 0`,
    // matching the existing "stop on size == 0" convention used by callers.
    let Some(&ins) = insns.get(pc) else {
        return (Instruction::Unknown { opcode: 0, raw: 0 }, 0);
    };
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

        0x0e..=0x10 => (
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
            (
                Instruction::ConstString {
                    register: reg,
                    string_idx: idx as u32,
                },
                2
            )

        }

        0x1f => {
            let idx = get(insns,pc+1) as usize;
            (
                Instruction::CheckCast {
                    class_idx:
                        resolve_type_descriptor_idx(
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
                    class_idx:
                        resolve_type_descriptor_idx(
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
            let (class_idx, name_idx, type_idx) = resolve_field_components(idx, dex);
            (
                Instruction::FieldAccess {
                    class_idx,
                    name_idx,
                    type_idx,
                },
                2
            )
        }

        0x60..=0x6d => {
            let idx = get(insns,pc+1) as usize;
            let (class_idx, name_idx, type_idx) = resolve_field_components(idx, dex);
            (
                Instruction::FieldAccess {
                    class_idx,
                    name_idx,
                    type_idx,
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
    let (class_idx, name_idx, descriptor_idx) = resolve_method_components(method_idx, dex);
    (
        Instruction::Invoke {
            kind,
            class_idx,
            name_idx,
            descriptor_idx,
            registers:regs,
        },
        3
    )
}

/// Resolve a method ID to a human-readable name (class->method).
///
/// Out-of-range indices return `<bad_method:N>`.
pub(crate) fn resolve_method(idx:usize, dex:&DexDocument) -> String {
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

/// Resolve a type ID to a class name descriptor.
///
/// Out-of-range indices return `<bad_type:N>`; invalid string references return `<bad_descriptor>`.
pub(crate) fn resolve_type(idx:usize, dex:&DexDocument) -> String{
    let Some(t) = dex.type_ids.types.get(idx)
    else {
        return format!(
            "<bad_type:{}>",
            idx
        );
    };
    dex.strings.strings.get(t.descriptor_idx as usize).cloned().unwrap_or("<bad_descriptor>".into())
}

/// Sentinel string-pool index for a component that could not be resolved
/// (e.g. an out-of-range table index). Guaranteed to be out of range for any
/// string pool, so downstream `ir::StringRef::resolve` renders it as a
/// `<bad_string:N>` placeholder rather than panicking.
pub(crate) const BAD_STRING_IDX: u32 = u32::MAX;

/// Resolve a type-table index to its descriptor's string-pool index.
///
/// Out-of-range type indices resolve to [`BAD_STRING_IDX`].
pub(crate) fn resolve_type_descriptor_idx(idx: usize, dex: &DexDocument) -> u32 {
    dex.type_ids
        .types
        .get(idx)
        .map(|t| t.descriptor_idx)
        .unwrap_or(BAD_STRING_IDX)
}

/// Resolve a method-table index to its `(class, name, descriptor)`
/// string-pool indices.
///
/// Out-of-range indices resolve to [`BAD_STRING_IDX`] for the affected
/// component(s).
pub(crate) fn resolve_method_components(idx: usize, dex: &DexDocument) -> (u32, u32, u32) {
    let Some(m) = dex.method_ids.methods.get(idx) else {
        return (BAD_STRING_IDX, BAD_STRING_IDX, BAD_STRING_IDX);
    };

    let class_idx = resolve_type_descriptor_idx(m.class_idx as usize, dex);
    let descriptor_idx = dex
        .proto_ids
        .protos
        .get(m.proto_idx as usize)
        .map(|p| p.shorty_idx)
        .unwrap_or(BAD_STRING_IDX);

    (class_idx, m.name_idx, descriptor_idx)
}

/// Resolve a field-table index to its `(class, name, type)`
/// string-pool indices.
///
/// Out-of-range indices resolve to [`BAD_STRING_IDX`] for the affected
/// component(s).
pub(crate) fn resolve_field_components(idx: usize, dex: &DexDocument) -> (u32, u32, u32) {
    let Some(f) = dex.field_ids.fields.get(idx) else {
        return (BAD_STRING_IDX, BAD_STRING_IDX, BAD_STRING_IDX);
    };

    let class_idx = resolve_type_descriptor_idx(f.class_idx as usize, dex);
    let type_idx = resolve_type_descriptor_idx(f.type_idx as usize, dex);

    (class_idx, f.name_idx, type_idx)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dex::parser::DexDocument;
    use crate::dex::strings::DexStrings;
    use crate::dex::type_id::{TypeIds, TypeId};
    use crate::dex::field_id::FieldIds;
    use crate::dex::method_id::{MethodIds, MethodId};
    use crate::dex::proto_id::ProtoIds;
    use crate::dex::class_def::ClassDefs;
    use crate::dex::header::DexHeader;
    use crate::dex::string_id::StringIds;

    /// Helper to build a minimal DexDocument for testing.
    fn minimal_dex() -> DexDocument {
        DexDocument {
            header: DexHeader {
                magic: *b"dex\n035\0",
                checksum: 0,
                signature: [0; 20],
                file_size: 0x70,
                header_size: 0x70,
                endian_tag: 0x1234_5678,
                string_ids_size: 1,
                string_ids_off: 0,
                type_ids_size: 1,
                type_ids_off: 0,
                proto_ids_size: 0,
                proto_ids_off: 0,
                field_ids_size: 0,
                field_ids_off: 0,
                method_ids_size: 1,
                method_ids_off: 0,
                class_defs_size: 0,
                class_defs_off: 0,
                data_size: 0,
                data_off: 0,
                link_size: 0,
                link_off: 0,
                map_off: 0,
            },
            string_ids: StringIds {
                strings: vec![],
            },
            strings: DexStrings {
                strings: vec!["test_string".to_string()],
            },
            type_ids: TypeIds {
                types: vec![TypeId {
                    descriptor_idx: 0,
                }],
            },
            proto_ids: ProtoIds {
                protos: vec![],
            },
            method_ids: MethodIds {
                methods: vec![MethodId {
                    class_idx: 0,
                    proto_idx: 0,
                    name_idx: 0,
                }],
            },
            field_ids: FieldIds {
                fields: vec![],
            },
            class_defs: ClassDefs {
                classes: vec![],
            },
        }
    }

    #[test]
    fn decode_instruction_nop() {
        let insns = [0x0000u16];
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 1);
        assert!(matches!(instr, Instruction::Nop));
    }

    #[test]
    fn decode_instruction_out_of_range_pc_returns_size_zero() {
        let insns = [0x0000u16, 0x0001u16];
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 5, &dex);
        assert_eq!(size, 0);
        assert!(matches!(instr, Instruction::Unknown { opcode: 0, raw: 0 }));
    }

    #[test]
    fn decode_instruction_return() {
        let insns = [0x000eu16]; // return-void opcode 0x0e (little-endian, opcode in low byte)
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 1);
        assert!(matches!(instr, Instruction::Return));
    }

    #[test]
    fn decode_instruction_const_16() {
        let insns = [0x0013u16, 0x00ffu16]; // const/16 with register 0, value 255
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 2);
        assert!(matches!(instr, Instruction::Const { register: 0, .. }));
    }

    #[test]
    fn decode_instruction_const_string() {
        let insns = [0x001au16, 0x0000u16]; // const-string with register 0, string index 0
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 2);
        assert!(matches!(instr, Instruction::ConstString { register: 0, .. }));
    }

    #[test]
    fn decode_instruction_const_string_out_of_range_string_returns_sentinel() {
        let insns = [0x001au16, 0x0005u16]; // const-string with register 0, string index 5 (out of range)
        let dex = minimal_dex();
        let (instr, _) = decode_instruction(&insns, 0, &dex);
        assert!(matches!(instr, Instruction::ConstString { string_idx, .. } if string_idx == 5));
    }

    #[test]
    fn decode_instruction_check_cast() {
        let insns = [0x001fu16, 0x0000u16]; // check-cast with type index 0
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 2);
        assert!(matches!(instr, Instruction::CheckCast { .. }));
    }

    #[test]
    fn decode_instruction_check_cast_out_of_range_type() {
        let insns = [0x001fu16, 0x0005u16]; // check-cast with type index 5 (out of range)
        let dex = minimal_dex();
        let (instr, _) = decode_instruction(&insns, 0, &dex);
        assert!(matches!(instr, Instruction::CheckCast { class_idx } if class_idx == BAD_STRING_IDX));
    }

    #[test]
    fn decode_instruction_new_instance() {
        let insns = [0x0022u16, 0x0000u16]; // new-instance with type index 0
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 2);
        assert!(matches!(instr, Instruction::NewInstance { .. }));
    }

    #[test]
    fn decode_instruction_new_instance_out_of_range_type() {
        let insns = [0x0022u16, 0x0005u16]; // new-instance with type index 5 (out of range)
        let dex = minimal_dex();
        let (instr, _) = decode_instruction(&insns, 0, &dex);
        assert!(matches!(instr, Instruction::NewInstance { class_idx } if class_idx == BAD_STRING_IDX));
    }

    #[test]
    fn decode_instruction_throw() {
        let insns = [0x0027u16]; // throw opcode
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 1);
        assert!(matches!(instr, Instruction::Throw));
    }

    #[test]
    fn decode_instruction_goto_8() {
        let insns = [0x0028u16]; // goto/8 opcode
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 1);
        assert!(matches!(instr, Instruction::Branch { kind: BranchKind::Goto }));
    }

    #[test]
    fn decode_instruction_if_eqz() {
        let insns = [0x0038u16, 0x0000u16]; // if-eqz opcode
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 2);
        assert!(matches!(instr, Instruction::Branch { kind: BranchKind::IfEqz }));
    }

    #[test]
    fn decode_instruction_field_access() {
        let insns = [0x0052u16, 0x0000u16]; // iget opcode, field index 0
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 2);
        assert!(matches!(instr, Instruction::FieldAccess { .. }));
    }

    #[test]
    fn decode_instruction_invoke_virtual() {
        let insns = [0x106eu16, 0x0000u16, 0x0000u16]; // invoke-virtual 0x6e with method index 0, register count 1
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 3);
        assert!(matches!(instr, Instruction::Invoke { kind: InvokeKind::Virtual, .. }));
    }

    #[test]
    fn decode_instruction_invoke_out_of_range_method() {
        let insns = [0x106eu16, 0x0005u16, 0x0000u16]; // invoke-virtual with method index 5 (out of range)
        let dex = minimal_dex();
        let (instr, _) = decode_instruction(&insns, 0, &dex);
        assert!(matches!(
            instr,
            Instruction::Invoke { class_idx, name_idx, descriptor_idx, .. }
                if class_idx == BAD_STRING_IDX && name_idx == BAD_STRING_IDX && descriptor_idx == BAD_STRING_IDX
        ));
    }

    #[test]
    fn decode_instruction_invoke_static() {
        let insns = [0x1071u16, 0x0000u16, 0x0000u16]; // invoke-static 0x71 with method index 0
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 3);
        assert!(matches!(instr, Instruction::Invoke { kind: InvokeKind::Static, .. }));
    }

    #[test]
    fn decode_instruction_move_result() {
        let insns = [0x000cu16]; // move-result opcode
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 1);
        assert!(matches!(instr, Instruction::MoveResult { .. }));
    }

    #[test]
    fn resolve_type_valid_index() {
        let dex = minimal_dex();
        let result = resolve_type(0, &dex);
        assert_eq!(result, "test_string");
    }

    #[test]
    fn resolve_type_out_of_range_index() {
        let dex = minimal_dex();
        let result = resolve_type(5, &dex);
        assert!(result.contains("<bad_type:"));
    }

    #[test]
    fn resolve_method_valid_index() {
        let dex = minimal_dex();
        let result = resolve_method(0, &dex);
        assert!(result.contains("->"));
        assert!(!result.contains("<bad_"));
    }

    #[test]
    fn resolve_method_out_of_range_index() {
        let dex = minimal_dex();
        let result = resolve_method(5, &dex);
        assert!(result.contains("<bad_method:"));
    }

    #[test]
    fn decode_35c_registers_count_0() {
        let first = 0x0000u16; // count = 0
        let third = 0x1234u16;
        let result = decode_35c_registers(first, third);
        assert!(result.is_empty());
    }

    #[test]
    fn decode_35c_registers_count_1() {
        let first = 0x1000u16; // count = 1
        let third = 0x00abu16; // v5
        let result = decode_35c_registers(first, third);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0x0b);
    }

    #[test]
    fn decode_35c_registers_count_5() {
        let first = 0x5100u16; // count = 5, v1
        let third = 0xf3a9u16; // v9, v10, v3, va
        let result = decode_35c_registers(first, third);
        assert_eq!(result.len(), 5);
        // Order: regs[0] = third&0xf, [1] = (third>>4)&0xf, [2] = (third>>8)&0xf, [3] = (third>>12)&0xf, [4] = (first>>8)&0xf
        assert_eq!(result[0], 0x09);
        assert_eq!(result[1], 0x0a);
        assert_eq!(result[2], 0x03);
        assert_eq!(result[3], 0x0f);
        assert_eq!(result[4], 0x01);
    }
}
