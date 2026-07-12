use crate::binary::BinaryReader;
use crate::errors::ApkError;

#[derive(Debug)]
pub struct ChunkHeader {
    pub chunk_type: u16,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub header_size: u16,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub chunk_size: u32,
}

impl ChunkHeader {
    pub fn parse(reader: &mut BinaryReader) -> Result<Self, ApkError> {
        Ok(Self {
            chunk_type: reader.read_u16()?,
            header_size: reader.read_u16()?,
            chunk_size: reader.read_u32()?,
        })
    }
}


