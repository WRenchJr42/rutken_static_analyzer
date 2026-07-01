use crate::binary::BinaryReader;
use crate::errors::ApkError;

#[derive(Debug)]
pub struct StringId {
    pub offset: u32,
}

#[derive(Debug)]
pub struct StringIds {
    pub strings: Vec<StringId>,
}

impl StringId {
    pub fn parse(reader: &mut BinaryReader) -> Result<Self, ApkError> {
        Ok(Self {
            offset: reader.read_u32()?,
        })
    }
}

impl StringIds {
    pub fn parse(reader: &mut BinaryReader, count: u32, offset: u32) -> Result<Self, ApkError> {
        reader.seek(offset as usize)?;
        let mut strings = Vec::new();
        for _ in 0..count {
            strings.push(StringId::parse(reader)?);
        }
        Ok(Self {
            strings,
        })
    }
}
        
