use crate::errors::ApkError;
use crate::binary::BinaryReader;

#[derive(Debug)]
pub struct StringPoolHeader {
    pub chunk_type: u16,
    pub header_size: u16,
    pub chunk_size: u32,
    pub string_count: u32,
    pub style_count: u32,
    pub flags: u32,
    pub strings_start: u32,
    pub styles_start: u32,
}

#[derive(Debug)]
pub struct StringPool {
    pub header: StringPoolHeader,
    pub strings: Vec<String>,
}

impl StringPoolHeader {
    pub fn parse(reader: &mut BinaryReader) -> Result<Self, ApkError> {
        let chunk_type = reader.read_u16()?;
        let header_size = reader.read_u16()?;
        let chunk_size = reader.read_u32()?;
        let string_count = reader.read_u32()?;
        let style_count = reader.read_u32()?;
        let flags = reader.read_u32()?;
        let strings_start = reader.read_u32()?;
        let styles_start = reader.read_u32()?;

        Ok(Self {
            chunk_type,
            header_size,
            chunk_size,
            string_count,
            style_count,
            flags,
            strings_start,
            styles_start,
        })
    }
}

impl StringPool {
    pub fn parse(reader: &mut BinaryReader) -> Result<Self, ApkError> {
        let chunk_start = reader.position();
        let header = StringPoolHeader::parse(reader)?;
        let mut offsets = Vec::new();
        let _strings_start = header.strings_start as usize;
        let mut strings = Vec::new();
        for _ in 0..header.string_count {
            offsets.push(reader.read_u32()?);
        }
        for offset in offsets {
            let absolute = chunk_start + header.strings_start as usize + offset as usize;
            let mut string_reader = reader.clone_at(absolute)?;
            let string = Self::read_utf16_string(&mut string_reader)?;
            strings.push(string);
        }
        reader.seek(chunk_start + header.chunk_size as usize)?;
        Ok(Self {
            header,
            strings,
        })
    }
    fn read_utf16_string(reader: &mut BinaryReader) -> Result<String, ApkError> {
        let length = reader.read_u16()? as usize;
        let mut chars = Vec::new();
        for _ in 0..length {
            chars.push(reader.read_u16()?);
        }
        reader.read_u16()?;
        String::from_utf16(&chars).map_err(|_| ApkError::InvalidFormat("Invalid UTF-16 string".to_string()))
    }
}
