use crate::binary::BinaryReader;
use crate::errors::ApkError;

#[derive(Debug)]
pub struct AxmlHeader{
    pub chunk_type: u16,
    pub header_size: u16,
    pub file_size: u32,
}

impl AxmlHeader {
    pub fn parse(reader: &mut BinaryReader) -> Result<Self, ApkError> {
        Ok(Self {
            chunk_type: reader.read_u16()?,
            header_size: reader.read_u16()?,
            file_size: reader.read_u32()?,
        })
    }
}

