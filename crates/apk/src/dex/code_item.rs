use crate::binary::BinaryReader;
use crate::errors::ApkError;

/// A single `encoded_type_addr_pair`: a caught exception type (a
/// string-pool-resolvable type-table index) and its handler entry point
/// (a code-unit offset within the method).
#[derive(Debug, Clone, Copy)]
pub struct CatchTypeAddr {
    pub type_idx: u32,
    pub handler_addr: u32,
}

/// A decoded `encoded_catch_handler`: the list of typed catches for a
/// `try_item`, plus an optional catch-all handler address.
///
/// Data only — these are exposed for a future exception-aware CFG pass;
/// no analysis is performed here (see `Instruction::Switch` for the
/// analogous rationale on branch/switch targets).
#[derive(Debug, Clone, Default)]
pub struct CatchHandler {
    pub catches: Vec<CatchTypeAddr>,
    pub catch_all_addr: Option<u32>,
}

/// A decoded `try_item`: the `[start_addr, end_addr)` code-unit range this
/// try block protects, and its associated handler.
#[derive(Debug, Clone)]
pub struct TryItem {
    pub start_addr: u32,
    /// Exclusive end of the protected range (`start_addr + insn_count`).
    pub end_addr: u32,
    pub handler: CatchHandler,
}

#[derive(Debug)]
pub struct CodeItem {
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub registers_size: u16,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub ins_size: u16,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub outs_size: u16,
    #[allow(dead_code)]
    // read locally during `parse` to gate try/catch parsing; not stored for
    // external consumption beyond that.
    pub tries_size: u16,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub debug_info_off: u32,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub insns_size: u32,
    pub instructions: Vec<u16>,
    /// Exception-handling ranges, as data (start/end PC + handler target
    /// PCs). Not yet consumed by any pass; reserved for a future
    /// exception-aware CFG milestone. Parse failures in this optional
    /// trailer are tolerated (empty `tries`) rather than failing the whole
    /// method, since the `insns` themselves already decoded successfully.
    pub tries: Vec<TryItem>,
}

impl CodeItem {
    pub fn parse(reader: &mut BinaryReader, offset: u32) -> Result<Self, ApkError> {
        reader.seek(offset as usize)?;
        let registers_size = reader.read_u16()?;
        let ins_size = reader.read_u16()?;
        let outs_size = reader.read_u16()?;
        let tries_size = reader.read_u16()?;
        let debug_info_off = reader.read_u32()?;
        let insns_size = reader.read_u32()?;
        let mut instructions = Vec::new();
        for _ in 0..insns_size {
            instructions.push(
                reader.read_u16()?
            );
        }

        let tries = if tries_size == 0 {
            Vec::new()
        } else {
            parse_tries(reader, insns_size, tries_size).unwrap_or_default()
        };

        Ok(Self {
            registers_size,
            ins_size,
            outs_size,
            tries_size,
            debug_info_off,
            insns_size,
            instructions,
            tries,
        })
    }
}

/// Parse the `try_item[tries_size]` array and its trailing
/// `encoded_catch_handler_list`, per the DEX `code_item` format.
///
/// Returns `Err` (tolerated by the caller as "no tries") if the trailer is
/// truncated or malformed; this never fails DEX parsing overall, since the
/// executable `insns` have already been read successfully.
fn parse_tries(
    reader: &mut BinaryReader,
    insns_size: u32,
    tries_size: u16,
) -> Result<Vec<TryItem>, ApkError> {
    // A two-byte alignment pad precedes `try_item[]` when `insns_size` is odd.
    if !insns_size.is_multiple_of(2) {
        reader.read_u16()?;
    }

    struct RawTry {
        start_addr: u32,
        end_addr: u32,
        handler_off: u16,
    }

    let mut raw_tries = Vec::with_capacity(tries_size as usize);
    for _ in 0..tries_size {
        let start_addr = reader.read_u32()?;
        let insn_count = reader.read_u16()?;
        let handler_off = reader.read_u16()?;
        raw_tries.push(RawTry {
            start_addr,
            end_addr: start_addr.saturating_add(insn_count as u32),
            handler_off,
        });
    }

    // `handler_off` values are byte offsets relative to the start of the
    // encoded_catch_handler_list, and may be shared by multiple try_items.
    // Parse the list once, recording each handler's byte offset so
    // try_items can be resolved by lookup.
    let list_start = reader.position();
    let handler_list_size = reader.read_uleb128()?;
    let mut handlers_by_offset = std::collections::HashMap::new();

    for _ in 0..handler_list_size {
        let handler_offset = (reader.position() - list_start) as u32;
        let handler = parse_catch_handler(reader)?;
        handlers_by_offset.insert(handler_offset, handler);
    }

    Ok(raw_tries
        .into_iter()
        .map(|raw| TryItem {
            start_addr: raw.start_addr,
            end_addr: raw.end_addr,
            handler: handlers_by_offset
                .get(&(raw.handler_off as u32))
                .cloned()
                .unwrap_or_default(),
        })
        .collect())
}

/// Parse a single `encoded_catch_handler`.
fn parse_catch_handler(reader: &mut BinaryReader) -> Result<CatchHandler, ApkError> {
    let size = reader.read_sleb128()?;
    let count = size.unsigned_abs();

    let mut catches = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let type_idx = reader.read_uleb128()?;
        let handler_addr = reader.read_uleb128()?;
        catches.push(CatchTypeAddr {
            type_idx,
            handler_addr,
        });
    }

    let catch_all_addr = if size <= 0 {
        Some(reader.read_uleb128()?)
    } else {
        None
    };

    Ok(CatchHandler {
        catches,
        catch_all_addr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_code_item_without_tries() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&2u16.to_le_bytes()); // registers_size
        bytes.extend_from_slice(&0u16.to_le_bytes()); // ins_size
        bytes.extend_from_slice(&0u16.to_le_bytes()); // outs_size
        bytes.extend_from_slice(&0u16.to_le_bytes()); // tries_size
        bytes.extend_from_slice(&0u32.to_le_bytes()); // debug_info_off
        bytes.extend_from_slice(&1u32.to_le_bytes()); // insns_size
        bytes.extend_from_slice(&0x0000u16.to_le_bytes()); // insns[0]: nop

        let mut reader = BinaryReader::new(&bytes);
        let code = CodeItem::parse(&mut reader, 0).unwrap();

        assert_eq!(code.instructions, vec![0x0000]);
        assert!(code.tries.is_empty());
    }

    /// Builds the bytes for a single `try_item` with one typed catch and no
    /// catch-all handler, protecting one instruction, in a method whose
    /// `insns_size` is odd (exercising the alignment pad).
    #[test]
    fn parse_code_item_with_single_typed_catch() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&2u16.to_le_bytes()); // registers_size
        bytes.extend_from_slice(&0u16.to_le_bytes()); // ins_size
        bytes.extend_from_slice(&0u16.to_le_bytes()); // outs_size
        bytes.extend_from_slice(&1u16.to_le_bytes()); // tries_size
        bytes.extend_from_slice(&0u32.to_le_bytes()); // debug_info_off
        bytes.extend_from_slice(&1u32.to_le_bytes()); // insns_size (odd -> padding required)
        bytes.extend_from_slice(&0x0000u16.to_le_bytes()); // insns[0]

        // Alignment padding (insns_size is odd).
        bytes.extend_from_slice(&0u16.to_le_bytes());

        // try_item[0]: start_addr=0, insn_count=1, handler_off=1 (the first
        // handler entry starts right after the 1-byte list-size uleb128,
        // which itself sits at offset 0 of the encoded_catch_handler_list).
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());

        // encoded_catch_handler_list: size=1
        bytes.push(0x01);
        // encoded_catch_handler[0]: size=1 (one typed catch, no catch-all)
        bytes.push(0x01);
        // encoded_type_addr_pair: type_idx=5, handler_addr=2
        bytes.push(0x05);
        bytes.push(0x02);

        let mut reader = BinaryReader::new(&bytes);
        let code = CodeItem::parse(&mut reader, 0).unwrap();

        assert_eq!(code.tries.len(), 1);
        let try_item = &code.tries[0];
        assert_eq!(try_item.start_addr, 0);
        assert_eq!(try_item.end_addr, 1);
        assert_eq!(try_item.handler.catches.len(), 1);
        assert_eq!(try_item.handler.catches[0].type_idx, 5);
        assert_eq!(try_item.handler.catches[0].handler_addr, 2);
        assert_eq!(try_item.handler.catch_all_addr, None);
    }

    #[test]
    fn parse_code_item_with_catch_all_only() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&2u16.to_le_bytes()); // registers_size
        bytes.extend_from_slice(&0u16.to_le_bytes()); // ins_size
        bytes.extend_from_slice(&0u16.to_le_bytes()); // outs_size
        bytes.extend_from_slice(&1u16.to_le_bytes()); // tries_size
        bytes.extend_from_slice(&0u32.to_le_bytes()); // debug_info_off
        bytes.extend_from_slice(&2u32.to_le_bytes()); // insns_size (even -> no padding)
        bytes.extend_from_slice(&0x0000u16.to_le_bytes()); // insns[0]
        bytes.extend_from_slice(&0x0000u16.to_le_bytes()); // insns[1]

        // try_item[0]: start_addr=0, insn_count=2, handler_off=1
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());

        // encoded_catch_handler_list: size=1
        bytes.push(0x01);
        // encoded_catch_handler[0]: size=0 (SLEB128 0x00) -> no typed catches, catch-all present
        bytes.push(0x00);
        // catch_all_addr = 7
        bytes.push(0x07);

        let mut reader = BinaryReader::new(&bytes);
        let code = CodeItem::parse(&mut reader, 0).unwrap();

        assert_eq!(code.tries.len(), 1);
        let handler = &code.tries[0].handler;
        assert!(handler.catches.is_empty());
        assert_eq!(handler.catch_all_addr, Some(7));
    }

    #[test]
    fn parse_code_item_with_truncated_tries_trailer_is_tolerated() {
        // `tries_size` claims one try_item, but the buffer ends right after
        // the insns. This must not fail DEX parsing (the executable insns
        // decoded fine); it should simply resolve to no tries.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&2u16.to_le_bytes()); // registers_size
        bytes.extend_from_slice(&0u16.to_le_bytes()); // ins_size
        bytes.extend_from_slice(&0u16.to_le_bytes()); // outs_size
        bytes.extend_from_slice(&1u16.to_le_bytes()); // tries_size (claims a try, but none follow)
        bytes.extend_from_slice(&0u32.to_le_bytes()); // debug_info_off
        bytes.extend_from_slice(&1u32.to_le_bytes()); // insns_size
        bytes.extend_from_slice(&0x0000u16.to_le_bytes()); // insns[0]

        let mut reader = BinaryReader::new(&bytes);
        let code = CodeItem::parse(&mut reader, 0).unwrap();

        assert_eq!(code.instructions, vec![0x0000]);
        assert!(code.tries.is_empty());
    }

    #[test]
    fn parse_code_item_with_shared_handler_across_try_items() {
        // Two try_items pointing at the same handler_off must both resolve
        // to the same (single, deduplicated) parsed handler.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&2u16.to_le_bytes()); // registers_size
        bytes.extend_from_slice(&0u16.to_le_bytes()); // ins_size
        bytes.extend_from_slice(&0u16.to_le_bytes()); // outs_size
        bytes.extend_from_slice(&2u16.to_le_bytes()); // tries_size
        bytes.extend_from_slice(&0u32.to_le_bytes()); // debug_info_off
        bytes.extend_from_slice(&4u32.to_le_bytes()); // insns_size (even -> no padding)
        for _ in 0..4 {
            bytes.extend_from_slice(&0x0000u16.to_le_bytes());
        }

        // try_item[0]: start_addr=0, insn_count=1, handler_off=1
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        // try_item[1]: start_addr=2, insn_count=1, handler_off=1 (shared)
        bytes.extend_from_slice(&2u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());

        // encoded_catch_handler_list: size=1
        bytes.push(0x01);
        // encoded_catch_handler[0]: size=1
        bytes.push(0x01);
        bytes.push(0x09); // type_idx = 9
        bytes.push(0x03); // handler_addr = 3

        let mut reader = BinaryReader::new(&bytes);
        let code = CodeItem::parse(&mut reader, 0).unwrap();

        assert_eq!(code.tries.len(), 2);
        for try_item in &code.tries {
            assert_eq!(try_item.handler.catches.len(), 1);
            assert_eq!(try_item.handler.catches[0].type_idx, 9);
            assert_eq!(try_item.handler.catches[0].handler_addr, 3);
        }
    }
}
