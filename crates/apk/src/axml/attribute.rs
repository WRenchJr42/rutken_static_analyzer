use crate::errors::ApkError;
use crate::binary::BinaryReader;

#[derive(Debug)]
pub struct Attribute {
    pub namespace: u32,
    pub name: u32,
    pub raw_value: u32,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub value_size: u16,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub res0: u8,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub data_type: u8,
    pub data: u32,
}

impl Attribute {
    pub fn parse(reader: &mut BinaryReader) -> Result<Self, ApkError> {
        Ok(Self {
            namespace: reader.read_u32()?,
            name: reader.read_u32()?,
            raw_value: reader.read_u32()?,
            value_size: reader.read_u16()?,
            res0: reader.read_u8()?,
            data_type: reader.read_u8()?,
            data: reader.read_u32()?,
        })
    }
}
