use crate::errors::ApkError;
use crate::dex::string_id::StringIds;
use crate::binary::BinaryReader;

#[derive(Debug)]
pub struct DexStrings {
    pub strings: Vec<String>,
}

impl DexStrings {
    pub fn parse(reader: &mut BinaryReader, ids: &StringIds) -> Result<Self, ApkError> {
        let mut strings = Vec::new();
        for id in &ids.strings {
            reader.seek(id.offset as usize)?;
            let _utf16_len = reader.read_uleb128()?;
            let string = reader.read_cstring()?;
            strings.push(string);
        }
        Ok(Self {
            strings,
        })
    }
}
