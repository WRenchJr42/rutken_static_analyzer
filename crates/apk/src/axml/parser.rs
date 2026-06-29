use crate::axml::header::AxmlHeader;
use crate::errors::ApkError;

pub struct AxmlParser;

impl AxmlParser {
    pub fn parse(bytes: &[u8]) -> Result<AxmlHeader, ApkError> {
    if bytes.len() < 8 {
        return Err(ApkError::InvalidFormat("Manifest too small".to_string()));
    }
    let chunk_type = u16::from_le_bytes([bytes[0], bytes[1]]);
    let header_size = u16::from_le_bytes([bytes[2], bytes[3]]);
    let file_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

    Ok(AxmlHeader {chunk_type, header_size, file_size})
    }
}
