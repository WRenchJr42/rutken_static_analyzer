use crate::binary::BinaryReader;
use crate::errors::ApkError;

#[derive(Debug)]
pub struct EndNamespace {
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub line_number: u32,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub comment: u32,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub prefix: u32,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub uri: u32,
}

impl EndNamespace {
    pub fn parse(reader: &mut BinaryReader) -> Result<Self, ApkError> {
        Ok(Self {
            line_number: reader.read_u32()?,
            comment: reader.read_u32()?,
            prefix: reader.read_u32()?,
            uri: reader.read_u32()?,
        })
    }
}




