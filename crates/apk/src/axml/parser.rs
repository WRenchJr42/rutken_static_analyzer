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
                let _ns = StartNamespace::parse(&mut reader)?;
            }
            RES_XML_START_ELEMENT => {
                let element = StartElement::parse(&mut reader)?;
                let mut attributes = Vec::new();
                for attribute in &element.attributes {
                    attributes.push(resolve_attribute(attribute, &string_pool));   
                }
                let name = string_pool
                    .strings
                    .get(element.name as usize)
                    .cloned()
                    .unwrap_or_else(|| format!("<bad_string:{}>", element.name));
                let node = XmlNode::new(name, attributes);
                stack.push(node);
            }
            RES_XML_END_ELEMENT => {
                let _end = EndElement::parse(&mut reader)?;
                let Some(node) = stack.pop() else {
                    return Err(ApkError::InvalidFormat(
                        "unbalanced XML end element".into(),
                    ));
                };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    root = Some(node);
                }
            }

            RES_XML_END_NAMESPACE => {
            let _ns = EndNamespace::parse(&mut reader)?;
            }

            _ => {
                break;
            }
        }
    }
    Ok(AxmlDocument {
        header,
        string_pool,
        resource_map,
        root,
    })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axml::constants::RES_XML_END_ELEMENT;

    /// Builds a minimal AXML document: header + empty string pool + empty
    /// resource map, followed by a single trailing chunk of `trailing_type`.
    fn minimal_axml_with_trailing_chunk(trailing_type: u16) -> Vec<u8> {
        let mut bytes = Vec::new();

        // AxmlHeader (8 bytes): chunk_type, header_size, file_size.
        bytes.extend_from_slice(&0x0003u16.to_le_bytes());
        bytes.extend_from_slice(&8u16.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());

        // StringPool header (28 bytes), zero strings, zero styles.
        bytes.extend_from_slice(&0x0001u16.to_le_bytes()); // chunk_type
        bytes.extend_from_slice(&28u16.to_le_bytes()); // header_size
        bytes.extend_from_slice(&28u32.to_le_bytes()); // chunk_size
        bytes.extend_from_slice(&0u32.to_le_bytes()); // string_count
        bytes.extend_from_slice(&0u32.to_le_bytes()); // style_count
        bytes.extend_from_slice(&0u32.to_le_bytes()); // flags
        bytes.extend_from_slice(&28u32.to_le_bytes()); // strings_start
        bytes.extend_from_slice(&0u32.to_le_bytes()); // styles_start

        // Resource map: header only, zero entries.
        bytes.extend_from_slice(&0x0180u16.to_le_bytes()); // chunk_type
        bytes.extend_from_slice(&8u16.to_le_bytes()); // header_size
        bytes.extend_from_slice(&8u32.to_le_bytes()); // chunk_size

        // Trailing chunk header + a plausible EndElement-sized payload.
        bytes.extend_from_slice(&trailing_type.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(&24u32.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 16]); // line_number/comment/namespace/name

        bytes
    }

    #[test]
    fn parse_rejects_unbalanced_end_element_without_panicking() {
        let bytes = minimal_axml_with_trailing_chunk(RES_XML_END_ELEMENT);
        let result = AxmlParser::parse(&bytes);
        assert!(result.is_err(), "unbalanced END_ELEMENT should be a parse error, not a panic");
    }
}
