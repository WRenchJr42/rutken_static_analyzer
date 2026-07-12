use crate::errors::ApkError;
use crate::binary::BinaryReader;

#[derive(Debug)]
pub struct EndElement {
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub line_number: u32,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub comment: u32,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub namespace: u32,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
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
