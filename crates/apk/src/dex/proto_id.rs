use crate::binary::BinaryReader;
use crate::errors::ApkError;

#[derive(Debug, Clone)]
pub struct ProtoId {
    pub shorty_idx: u32,
    pub return_type_idx: u32,
    pub parameters_off: u32,
}

#[derive(Debug)]
pub struct ProtoIds {
    pub protos: Vec<ProtoId>,
}

impl ProtoIds {
    pub fn parse(reader: &mut BinaryReader, count: u32, offset: u32) -> Result<Self, ApkError> {
        reader.seek(offset as usize)?;
        let mut protos = Vec::new();
        for _ in 0..count {
            protos.push(
                ProtoId {
                    shorty_idx: reader.read_u32()?,
                    return_type_idx: reader.read_u32()?,
                    parameters_off: reader.read_u32()?,
                }
            );
        }
        Ok(Self {
            protos,
        })
    }
}
