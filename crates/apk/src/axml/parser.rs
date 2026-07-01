use crate::axml::header::AxmlHeader;
use crate::errors::ApkError;
use crate::binary::BinaryReader;
use crate::axml::string_pool::StringPool;
use crate::axml::resource_map::ResourceMap;
use crate::axml::chunks::ChunkHeader;
use crate::axml::namespace::StartNamespace;
use crate::axml::element::StartElement;
use crate::axml::constants::*;
use crate::axml::resolve::resolve_attribute;
use crate::axml::end_element::EndElement;

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
        match chunk.chunk_type {
            RES_XML_START_NAMESPACE => {
                let ns = StartNamespace::parse(&mut reader)?;
                println!("Namespace: {} -> {}", string_pool.strings[ns.prefix as usize], string_pool.strings[ns.uri as usize]);
            }
            RES_XML_START_ELEMENT => {
                let element = StartElement::parse(&mut reader)?;
                println!("{:#?}", element);
                println!("Element: {}", string_pool.strings[element.name as usize]);
                for attribute in &element.attributes {
                    let resolved = resolve_attribute(attribute, &string_pool);
                    if let Some(ns) = resolved.namespace {
                        println!("{}:{} = {}", ns, resolved.name, resolved.value);
                    } else {
                        println!("{} = {}", resolved.name, resolved.value);
                    }
                }
            }
            RES_XML_END_ELEMENT => {
            let end = EndElement::parse(&mut reader)?;
            println!("End: {}", string_pool.strings[end.name as usize]);
            }

            _ => {
                println!("Unknown chunk: 0x{:04x}", chunk.chunk_type);
                break;
            }
        }
    }
        
    Ok(AxmlDocument {
        header,
        string_pool,
        resource_map,
    })
    }
}
