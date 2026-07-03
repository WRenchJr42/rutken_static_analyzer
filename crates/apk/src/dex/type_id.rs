use crate::binary::BinaryReader;
use crate::errors::ApkError;

#[derive(Debug, Clone)]
pub struct TypeId {
    pub descriptor_idx: u32,
}

#[derive(Debug)]
pub struct TypeIds {
    pub types: Vec<TypeId>,
}

impl TypeIds {
    pub fn parse(reader: &mut BinaryReader, count: u32, offset: u32) -> Result<Self, ApkError> {
        reader.seek(offset as usize)?;
        let mut types = Vec::new();
        for _ in 0..count {
            types.push(TypeId {
                descriptor_idx: reader.read_u32()?,
            });
        }
        Ok(Self {
            types,
        })
    }
}
