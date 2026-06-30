use crate::axml::header::AxmlHeader;
use crate::errors::ApkError;
use crate::binary::BinaryReader;
use crate::axml::string_pool::StringPool;

pub struct AxmlParser;

impl AxmlParser {
    pub fn parse(bytes: &[u8]) -> Result<AxmlHeader, ApkError> {
    if bytes.len() < 8 {
        return Err(ApkError::InvalidFormat("Manifest too small".to_string()));
    }
    
    let mut reader = BinaryReader::new(bytes);
    
    let chunk_type = reader.read_u16()?;
    let header_size = reader.read_u16()?;
    let file_size = reader.read_u32()?;
    
    let header = AxmlHeader {
         chunk_type,
         header_size,
         file_size,
    };
    println!("{:#?}", header);
    let string_pool = StringPool::parse(&mut reader)?;
    println!("{:#?}", string_pool);
    Ok(AxmlHeader {chunk_type, header_size, file_size})
    }
}
