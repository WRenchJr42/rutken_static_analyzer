use crate::binary::BinaryReader;
use crate::errors::ApkError;
use crate::axml::attribute::Attribute;

#[derive(Debug)]
pub struct StartElement {
    pub line_number: u32,
    pub comment: u32,
    pub namespace: u32,
    pub name: u32,
    pub attribute_start: u16,
    pub attribute_size: u16,
    pub attribute_count: u16,
    pub id_index: u16,
    pub class_index: u16,
    pub style_index: u16,
    pub attributes: Vec<Attribute>,
}

impl StartElement {
    pub fn parse(reader: &mut BinaryReader) -> Result<Self, ApkError> {
        let line_number = reader.read_u32()?;
        let comment = reader.read_u32()?;
        let namespace = reader.read_u32()?;
        let name = reader.read_u32()?;
        let attribute_start = reader.read_u16()?;
        let attribute_size = reader.read_u16()?;
        let attribute_count = reader.read_u16()?;
        let id_index = reader.read_u16()?;
        let class_index = reader.read_u16()?;
        let style_index = reader.read_u16()?;
        let mut attributes = Vec::new();
        for _ in 0..attribute_count {
            attributes.push(Attribute::parse(reader)?);
        }
        Ok(Self {
            line_number,
            comment,
            namespace,
            name,
            attribute_start,
            attribute_size,
            attribute_count,
            id_index,
            class_index,
            style_index,
            attributes,
        })
    }
}
