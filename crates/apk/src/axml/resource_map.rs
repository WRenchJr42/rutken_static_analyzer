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

        if chunk_type != RES_XML_RESOURCE_MAP {
            return Err(ApkError::BadChunk("Expected Resource Map chunk".to_string()));
        }
        if header_size != 8 {
            return Err(ApkError::BadChunk("Invalid Resource Map header".to_string()));
        }

        let payload_size = chunk_size
            .checked_sub(header_size as u32)
            .ok_or_else(|| ApkError::BadChunk("Resource Map chunk_size smaller than header".to_string()))?;
        let count = (payload_size / 4) as usize;

        // Guard against a maliciously large `count` triggering a huge allocation
        // before we know the buffer actually contains that much data.
        let available = reader.remaining() / 4;
        if count > available {
            return Err(ApkError::Truncated(
                "Resource Map declares more entries than remaining buffer".to_string(),
            ));
        }

        let mut resources = Vec::new();
        for _ in 0..count {
            resources.push(reader.read_u32()?);
        }
        Ok(Self {
            resources,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_bytes(chunk_size: u32, header_size: u16) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&RES_XML_RESOURCE_MAP.to_le_bytes());
        bytes.extend_from_slice(&header_size.to_le_bytes());
        bytes.extend_from_slice(&chunk_size.to_le_bytes());
        bytes
    }

    #[test]
    fn parse_rejects_chunk_size_smaller_than_header_without_panicking() {
        // chunk_size < header_size would previously underflow `chunk_size - header_size`.
        let bytes = header_bytes(4, 8);
        let mut reader = BinaryReader::new(&bytes);
        assert!(ResourceMap::parse(&mut reader).is_err());
    }

    #[test]
    fn parse_rejects_declared_count_larger_than_buffer() {
        // Declares a huge entry count but provides no backing data.
        let bytes = header_bytes(8 + 4 * 1_000_000, 8);
        let mut reader = BinaryReader::new(&bytes);
        assert!(ResourceMap::parse(&mut reader).is_err());
    }

    #[test]
    fn parse_accepts_well_formed_resource_map() {
        let mut bytes = header_bytes(8 + 8, 8);
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&2u32.to_le_bytes());
        let mut reader = BinaryReader::new(&bytes);
        let map = ResourceMap::parse(&mut reader).expect("well formed map should parse");
        assert_eq!(map.resources, vec![1, 2]);
    }
}
