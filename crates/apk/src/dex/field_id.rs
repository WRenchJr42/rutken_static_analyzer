use crate::binary::BinaryReader;
use crate::errors::ApkError;

#[derive(Debug,Clone)]
pub struct FieldId {
    pub class_idx: u16,
    pub type_idx: u16,
    pub name_idx: u32,
}

#[derive(Debug)]
pub struct FieldIds {
    pub fields: Vec<FieldId>,
}

impl FieldIds {
    pub fn parse(reader: &mut BinaryReader, count: u32, offset: u32) -> Result<Self, ApkError> {
        reader.seek(offset as usize)?;
        let mut fields = Vec::new();
        for _ in 0..count {
            fields.push(FieldId {
                class_idx: reader.read_u16()?,
                type_idx: reader.read_u16()?,
                name_idx: reader.read_u32()?,
            });
        }
        Ok(Self {
            fields,
        })
    }
}
