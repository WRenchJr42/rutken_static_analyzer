use crate::errors::ApkError;
use crate::binary::BinaryReader;
use crate::axml::constants::*;

#[derive(Debug)]
pub struct ResourceMap {
    pub resources: Vec<u32>,
}

impl ResourceMap {
    pub fn parse(reader: &mut BinaryReader) -> Result<Self, ApkError> {
        let chunk_type = reader.read_u16()?;
        let header_size = reader.read_u16()?;
        let chunk_size = reader.read_u32()?;
        let count = ((chunk_size - header_size as u32) / 4) as usize;
        
              if chunk_type != RES_XML_RESOURCE_MAP {
            return Err(ApkError::InvalidFormat("Expected Resource Map chunk".to_string()));
        }
        if header_size != 8 {
            return Err(ApkError::InvalidFormat("Invalid Resource Map header".to_string()));
        }

        let mut resources = Vec::with_capacity(count);
        for _ in 0..count {
            resources.push(reader.read_u32()?);
        }
        Ok(Self {
            resources,
        })
    }
}
