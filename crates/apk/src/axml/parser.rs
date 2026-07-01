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
use crate::axml::end_namespace::EndNamespace;
use crate::axml::node::XmlNode;

#[derive(Debug)]
pub struct AxmlDocument {
    pub header: AxmlHeader,
    pub string_pool: StringPool,
    pub resource_map: ResourceMap,
    pub root: Option<XmlNode>,
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
    let mut stack: Vec<XmlNode> = Vec::new();
    let mut root: Option<XmlNode> = None;

    while reader.remaining() > 0 {
        let chunk = ChunkHeader::parse(&mut reader)?;
        match chunk.chunk_type {
            RES_XML_START_NAMESPACE => {
                let ns = StartNamespace::parse(&mut reader)?;
                println!("Namespace: {} -> {}", string_pool.strings[ns.prefix as usize], string_pool.strings[ns.uri as usize]);
            }
            RES_XML_START_ELEMENT => {
                let element = StartElement::parse(&mut reader)?;
                let mut attributes = Vec::new();
                for attribute in &element.attributes {
                    attributes.push(resolve_attribute(attribute, &string_pool));   
                }
                let node = XmlNode::new(string_pool.strings[element.name as usize].clone(), attributes);
                stack.push(node);
            }
            RES_XML_END_ELEMENT => {
                let _end = EndElement::parse(&mut reader)?;
                let node = stack.pop().unwrap();
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    root = Some(node);
                }
            }

            RES_XML_END_NAMESPACE => {
            let ns = EndNamespace::parse(&mut reader)?;
            println!("End: {}", string_pool.strings[ns.prefix as usize]);
            }

            _ => {
                println!("Unknown chunk: 0x{:04x}", chunk.chunk_type);
                break;
            }
        }
    }
    println!("{:#?}", root);    
    Ok(AxmlDocument {
        header,
        string_pool,
        resource_map,
        root,
    })
    }
}
