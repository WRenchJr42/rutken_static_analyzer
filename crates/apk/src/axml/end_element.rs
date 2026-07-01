use crate::errors::ApkError;
use crate::binary::BinaryReader;

#[derive(Debug)]
pub struct EndElement {
    pub line_number: u32,
    pub comment: u32,
    pub namespace: u32,
    pub name: u32,
}

impl EndElement {
    pub fn parse(reader: &mut BinaryReader) -> Result<Self, ApkError> {
        Ok(Self {
            line_number: reader.read_u32()?,
            comment: reader.read_u32()?,
            namespace: reader.read_u32()?,
            name: reader.read_u32()?,
        })
    }
}
