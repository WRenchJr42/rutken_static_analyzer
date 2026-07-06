use crate::binary::BinaryReader;
use crate::errors::ApkError;

#[derive(Debug)]
pub struct CodeItem {
    pub registers_size: u16,
    pub ins_size: u16,
    pub outs_size: u16,
    pub tries_size: u16,
    pub debug_info_off: u32,
    pub insns_size: u32,
    pub instructions: Vec<u16>,
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

        Ok(Self {
            registers_size,
            ins_size,
            outs_size,
            tries_size,
            debug_info_off,
            insns_size,
            instructions,
        })
    }
}
