use crate::dex::instruction::{BranchKind, Instruction, InvokeKind, SwitchCase};
use crate::dex::opcode::opcode_width;
use crate::dex::parser::DexDocument;

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
        // packed-switch-payload: ushort ident, ushort size, int first_key,
        // int[size] targets.
        0x0100 => {
            let size = get(insns, pc + 1) as usize;
            return (Instruction::Payload, 4 + size * 2);
        }
        // sparse-switch-payload: ushort ident, ushort size, int[size] keys,
        // int[size] targets.
        0x0200 => {
            let size = get(insns, pc + 1) as usize;
            return (Instruction::Payload, 2 + size * 4);
        }
        // fill-array-data-payload: ushort ident, ushort element_width,
        // uint size, ubyte[] data (data rounded up to a whole code unit).
        0x0300 => {
            let element_width = get(insns, pc + 1) as usize;
            let size = read_u32(insns, pc + 2) as usize;
            let data_units = element_width
                .checked_mul(size)
                .map(|bytes| bytes.div_ceil(2))
                .unwrap_or(insns.len().saturating_sub(pc));
            return (Instruction::Payload, 4 + data_units);
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

        // goto (10t): signed 8-bit offset in the high byte of the opcode unit.
        0x28 => {
            let offset = ((ins >> 8) as i8) as i64;
            (
                Instruction::Branch {
                    kind: BranchKind::Goto,
                    target: resolve_target(pc, offset),
                },
                1
            )
        }

        // goto/16 (20t): signed 16-bit offset in the next code unit.
        0x29 => {
            let offset = read_i16(insns, pc + 1) as i64;
            (
                Instruction::Branch {
                    kind: BranchKind::Goto,
                    target: resolve_target(pc, offset),
                },
                2
            )
        }

        // goto/32 (30t): signed 32-bit offset across the next two code units.
        0x2a => {
            let offset = read_i32(insns, pc + 1) as i64;
            (
                Instruction::Branch {
                    kind: BranchKind::Goto,
                    target: resolve_target(pc, offset),
                },
                3
            )
        }

        // packed-switch / sparse-switch (31t): signed 32-bit offset (in code
        // units, relative to this instruction) to the switch payload.
        0x2b => (decode_switch(insns, pc, true), 3),
        0x2c => (decode_switch(insns, pc, false), 3),

        // if-test (22t): two registers, then a signed 16-bit offset.
        0x32..=0x37 => {
            let offset = read_i16(insns, pc + 1) as i64;
            (
                Instruction::Branch {
                    kind: if_test_kind(opcode),
                    target: resolve_target(pc, offset),
                },
                2
            )
        }

        // if-testz (21t): one register, then a signed 16-bit offset.
        0x38..=0x3d => {
            let offset = read_i16(insns, pc + 1) as i64;
            (
                Instruction::Branch {
                    kind: if_testz_kind(opcode),
                    target: resolve_target(pc, offset),
                },
                2
            )
        }

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

/// Read a signed 16-bit value from a single code unit.
fn read_i16(insns: &[u16], idx: usize) -> i32 {
    get(insns, idx) as i16 as i32
}

/// Read an unsigned 32-bit value from two code units (low unit first).
fn read_u32(insns: &[u16], idx: usize) -> u32 {
    let lo = get(insns, idx) as u32;
    let hi = get(insns, idx + 1) as u32;
    (hi << 16) | lo
}

/// Read a signed 32-bit value from two code units (low unit first).
fn read_i32(insns: &[u16], idx: usize) -> i32 {
    read_u32(insns, idx) as i32
}

/// Resolve a signed code-unit offset relative to `pc` into an absolute
/// code-unit target, clamping to a safe `u32` range instead of
/// overflowing/underflowing on malformed or adversarial input. Never
/// panics.
fn resolve_target(pc: usize, offset: i64) -> u32 {
    let target = (pc as i64).saturating_add(offset);
    target.clamp(0, u32::MAX as i64) as u32
}

/// Map a `22t` if-test opcode (`if-eq` .. `if-le`, 0x32..=0x37) to its
/// `BranchKind`.
fn if_test_kind(opcode: u8) -> BranchKind {
    match opcode {
        0x32 => BranchKind::IfEq,
        0x33 => BranchKind::IfNe,
        0x34 => BranchKind::IfLt,
        0x35 => BranchKind::IfGe,
        0x36 => BranchKind::IfGt,
        _ => BranchKind::IfLe,
    }
}

/// Map a `21t` if-testz opcode (`if-eqz` .. `if-lez`, 0x38..=0x3d) to its
/// `BranchKind`.
fn if_testz_kind(opcode: u8) -> BranchKind {
    match opcode {
        0x38 => BranchKind::IfEqz,
        0x39 => BranchKind::IfNez,
        0x3a => BranchKind::IfLtz,
        0x3b => BranchKind::IfGez,
        0x3c => BranchKind::IfGtz,
        _ => BranchKind::IfLez,
    }
}

/// Decode a `packed-switch`/`sparse-switch` instruction at `pc`, resolving
/// its payload (located elsewhere in the same method's `insns`, at a
/// signed offset relative to `pc`).
///
/// The payload's own ident tag (not the calling opcode) determines which
/// payload format is parsed, so a mismatched/corrupt reference resolves to
/// an empty case list rather than misinterpreting unrelated data.
fn decode_switch(insns: &[u16], pc: usize, packed: bool) -> Instruction {
    let offset = read_i32(insns, pc + 1) as i64;
    let payload_pc = resolve_target(pc, offset) as usize;
    let cases = match get(insns, payload_pc) {
        0x0100 => decode_packed_switch_payload(insns, payload_pc, pc),
        0x0200 => decode_sparse_switch_payload(insns, payload_pc, pc),
        _ => Vec::new(),
    };
    Instruction::Switch { packed, cases }
}

/// Decode a `packed-switch-payload` at `payload_pc`. Targets are absolute
/// code-unit offsets, resolved relative to the *switch instruction's* PC
/// (`switch_pc`), per the DEX spec.
fn decode_packed_switch_payload(insns: &[u16], payload_pc: usize, switch_pc: usize) -> Vec<SwitchCase> {
    let size = get(insns, payload_pc + 1) as usize;
    let first_key = read_i32(insns, payload_pc + 2);

    (0..size)
        .map(|i| {
            let target_offset = read_i32(insns, payload_pc + 4 + i * 2) as i64;
            SwitchCase {
                key: first_key.wrapping_add(i as i32),
                target: resolve_target(switch_pc, target_offset),
            }
        })
        .collect()
}

/// Decode a `sparse-switch-payload` at `payload_pc`. Targets are absolute
/// code-unit offsets, resolved relative to the *switch instruction's* PC
/// (`switch_pc`), per the DEX spec.
fn decode_sparse_switch_payload(insns: &[u16], payload_pc: usize, switch_pc: usize) -> Vec<SwitchCase> {
    let size = get(insns, payload_pc + 1) as usize;
    let keys_start = payload_pc + 2;
    let targets_start = keys_start + size * 2;

    (0..size)
        .map(|i| {
            let key = read_i32(insns, keys_start + i * 2);
            let target_offset = read_i32(insns, targets_start + i * 2) as i64;
            SwitchCase {
                key,
                target: resolve_target(switch_pc, target_offset),
            }
        })
        .collect()
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
        let insns = [0x0028u16]; // goto/8 opcode, offset 0 -> target == pc
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 1);
        assert!(matches!(instr, Instruction::Branch { kind: BranchKind::Goto, target: 0 }));
    }

    #[test]
    fn decode_instruction_goto_8_negative_offset() {
        // goto/8 opcode 0x28 with AA = -4 (0xfc): target = pc(0) + (-4) -> clamped to 0.
        let insns = [0xfc28u16];
        let dex = minimal_dex();
        let (instr, _) = decode_instruction(&insns, 0, &dex);
        assert!(matches!(instr, Instruction::Branch { kind: BranchKind::Goto, target: 0 }));
    }

    #[test]
    fn decode_instruction_goto_8_positive_offset() {
        // goto/8 opcode 0x28 with AA = 5: target = pc(1) + 5 = 6.
        let insns = [0x0000u16, 0x0528u16];
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 1, &dex);
        assert_eq!(size, 1);
        assert!(matches!(instr, Instruction::Branch { kind: BranchKind::Goto, target: 6 }));
    }

    #[test]
    fn decode_instruction_if_eqz() {
        let insns = [0x0038u16, 0x0000u16]; // if-eqz opcode, offset 0
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 2);
        assert!(matches!(instr, Instruction::Branch { kind: BranchKind::IfEqz, target: 0 }));
    }

    #[test]
    fn decode_instruction_if_eq_target() {
        // if-eq (0x32), offset = 10 -> target = pc(0) + 10 = 10.
        let insns = [0x0032u16, 0x000au16];
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 2);
        assert!(matches!(instr, Instruction::Branch { kind: BranchKind::IfEq, target: 10 }));
    }

    #[test]
    fn decode_instruction_goto_16_target() {
        // goto/16 (0x29), offset = 3 -> target = pc(0) + 3 = 3.
        let insns = [0x0029u16, 0x0003u16];
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 2);
        assert!(matches!(instr, Instruction::Branch { kind: BranchKind::Goto, target: 3 }));
    }

    #[test]
    fn decode_instruction_goto_32_target() {
        // goto/32 (0x2a), offset = 0x0001_0000 (lo=0x0000, hi=0x0001) -> target = 65536.
        let insns = [0x002au16, 0x0000u16, 0x0001u16];
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 3);
        assert!(matches!(instr, Instruction::Branch { kind: BranchKind::Goto, target: 65536 }));
    }

    #[test]
    fn decode_instruction_packed_switch() {
        // packed-switch at pc 0, offset (in code units) = 3 -> payload at pc 3.
        // payload: ident 0x0100, size 2, first_key 100, targets [10, -5] (relative to switch pc 0).
        let insns = [
            0x002bu16, // packed-switch opcode
            0x0003u16, 0x0000u16, // offset = 3 (lo, hi)
            0x0100u16, // payload ident
            0x0002u16, // size = 2
            0x0064u16, 0x0000u16, // first_key = 100 (lo, hi)
            0x000au16, 0x0000u16, // target[0] offset = 10
            0xfffbu16, 0xffffu16, // target[1] offset = -5
        ];
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 3);
        match instr {
            Instruction::Switch { packed, cases } => {
                assert!(packed);
                assert_eq!(cases.len(), 2);
                assert_eq!(cases[0], SwitchCase { key: 100, target: 10 });
                assert_eq!(cases[1], SwitchCase { key: 101, target: 0 });
            }
            other => panic!("expected Switch, got {other:?}"),
        }
    }

    #[test]
    fn decode_instruction_sparse_switch() {
        // sparse-switch at pc 0, offset = 3 -> payload at pc 3.
        // payload: ident 0x0200, size 1, key [7], target offset [3] (relative to switch pc 0).
        let insns = [
            0x002cu16, // sparse-switch opcode
            0x0003u16, 0x0000u16, // offset = 3
            0x0200u16, // payload ident
            0x0001u16, // size = 1
            0x0007u16, 0x0000u16, // key[0] = 7
            0x0003u16, 0x0000u16, // target[0] offset = 3
        ];
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 3);
        match instr {
            Instruction::Switch { packed, cases } => {
                assert!(!packed);
                assert_eq!(cases, vec![SwitchCase { key: 7, target: 3 }]);
            }
            other => panic!("expected Switch, got {other:?}"),
        }
    }

    #[test]
    fn decode_instruction_switch_with_mismatched_payload_ident_is_safe() {
        // packed-switch pointing at a payload location whose ident isn't a
        // recognized payload tag: must resolve to an empty case list, not panic.
        let insns = [0x002bu16, 0x0003u16, 0x0000u16, 0xffffu16];
        let dex = minimal_dex();
        let (instr, _) = decode_instruction(&insns, 0, &dex);
        assert!(matches!(instr, Instruction::Switch { cases, .. } if cases.is_empty()));
    }

    #[test]
    fn decode_instruction_switch_out_of_range_payload_is_safe() {
        // Offset pointing far beyond the insns array must not panic; the
        // payload ident lookup safely returns 0 (via `get`'s OOB fallback),
        // which doesn't match a known payload tag, yielding empty cases.
        let insns = [0x002bu16, 0xffffu16, 0x7fffu16];
        let dex = minimal_dex();
        let (instr, size) = decode_instruction(&insns, 0, &dex);
        assert_eq!(size, 3);
        assert!(matches!(instr, Instruction::Switch { cases, .. } if cases.is_empty()));
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
