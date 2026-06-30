use crate::axml::header::AxmlHeader;
use crate::errors::ApkError;
use crate::binary::BinaryReader;
use crate::axml::string_pool::StringPool;
use crate::axml::resource_map::ResourceMap;
use crate::axml::chunks::ChunkHeader;
use crate::axml::namespace::StartNamespace;

#[derive(Debug)]
pub struct AxmlDocument {
    pub header: AxmlHeader,
    pub string_pool: StringPool,
    pub resource_map: ResourceMap,
}

pub struct AxmlParser;

impl AxmlParser {
    pub fn parse(bytes: &[u8]) -> Result<AxmlDocument, ApkError> {
    if bytes.len() < 8 {
        return Err(ApkError::InvalidFormat("Manifest too small".to_string()));
    }
    
    let mut reader = BinaryReader::new(bytes);
    let header = AxmlHeader::parse(&mut reader)?;
    let string_pool = StringPool::parse(&mut reader)?;
    let resource_map = ResourceMap::parse(&mut reader)?;
    
    while reader.remaining() > 0 {
        let chunk = ChunkHeader::parse(&mut reader)?;
        println!("{:#?}", chunk);
        let namespace = StartNamespace::parse(&mut reader)?;
        println!("{} -> {}",string_pool.strings[namespace.prefix as usize], string_pool.strings[namespace.uri as usize]);
        break;
    }
        
    Ok(AxmlDocument {
        header,
        string_pool,
        resource_map,
    })
    }
}
