use crate::binary::BinaryReader;
use crate::errors::ApkError;

#[derive(Debug)]
pub struct DexHeader {
    pub magic: [u8; 8],
    pub checksum: u32,
    pub signature: [u8; 20],
    pub file_size: u32,
    pub header_size: u32,
    pub endian_tag: u32,
    pub string_ids_size: u32,
    pub string_ids_off: u32,
    pub type_ids_size: u32,
    pub type_ids_off: u32,
    pub proto_ids_size: u32,
    pub proto_ids_off: u32,
    pub field_ids_size: u32,
    pub field_ids_off: u32,
    pub method_ids_size: u32,
    pub method_ids_off: u32,
    pub class_defs_size: u32,
    pub class_defs_off: u32,
    pub data_size: u32,
    pub data_off: u32,
    pub link_size: u32,
    pub link_off: u32,
    pub map_off: u32,
}

/// Expected DEX endian tag for little-endian files (the only layout this
/// parser supports).
const DEX_ENDIAN_CONSTANT: u32 = 0x1234_5678;

/// DEX magic is `"dex\n"` followed by a 3-digit version and a NUL byte,
/// e.g. `dex\n035\0`. We only check the fixed `"dex\n"` prefix and the
/// trailing NUL here; the exact version digits are not semantically
/// significant to this parser.
fn is_valid_dex_magic(magic: &[u8; 8]) -> bool {
    magic[0..4] == *b"dex\n" && magic[7] == 0 && magic[4..7].iter().all(u8::is_ascii_digit)
}

impl DexHeader {
    pub fn parse(reader: &mut BinaryReader) -> Result<Self, ApkError> {
        let magic = reader.read_array::<8>()?;
        if !is_valid_dex_magic(&magic) {
            return Err(ApkError::BadMagic(format!(
                "unrecognized DEX magic: {:02x?}",
                magic
            )));
        }

        let checksum = reader.read_u32()?;
        let signature = reader.read_array::<20>()?;
        let file_size = reader.read_u32()?;
        let header_size = reader.read_u32()?;
        let endian_tag = reader.read_u32()?;
        if endian_tag != DEX_ENDIAN_CONSTANT {
            return Err(ApkError::BadMagic(format!(
                "unsupported DEX endian tag: 0x{:08x}",
                endian_tag
            )));
        }

        Ok(Self {
            magic,
            checksum,
            signature,
            file_size,
            header_size,
            endian_tag,
            link_size: reader.read_u32()?,
            link_off: reader.read_u32()?,
            map_off: reader.read_u32()?,
            string_ids_size: reader.read_u32()?,
            string_ids_off: reader.read_u32()?,
            type_ids_size: reader.read_u32()?,
            type_ids_off: reader.read_u32()?,
            proto_ids_size: reader.read_u32()?,
            proto_ids_off: reader.read_u32()?,
            field_ids_size: reader.read_u32()?,
            field_ids_off: reader.read_u32()?,
            method_ids_size: reader.read_u32()?,
            method_ids_off: reader.read_u32()?,
            class_defs_size: reader.read_u32()?,
            class_defs_off: reader.read_u32()?,
            data_size: reader.read_u32()?,
            data_off: reader.read_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal, well-formed 0x70-byte DEX header buffer.
    fn valid_header_bytes() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"dex\n035\0"); // magic
        bytes.extend_from_slice(&[0u8; 4]); // checksum
        bytes.extend_from_slice(&[0u8; 20]); // signature
        bytes.extend_from_slice(&0x70u32.to_le_bytes()); // file_size
        bytes.extend_from_slice(&0x70u32.to_le_bytes()); // header_size
        bytes.extend_from_slice(&DEX_ENDIAN_CONSTANT.to_le_bytes()); // endian_tag
        for _ in 0..17 {
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn parse_accepts_well_formed_header() {
        let bytes = valid_header_bytes();
        let mut reader = BinaryReader::new(&bytes);
        let header = DexHeader::parse(&mut reader).expect("valid header should parse");
        assert_eq!(header.endian_tag, DEX_ENDIAN_CONSTANT);
    }

    #[test]
    fn parse_rejects_bad_magic() {
        let mut bytes = valid_header_bytes();
        bytes[0] = b'X';
        let mut reader = BinaryReader::new(&bytes);
        assert!(matches!(
            DexHeader::parse(&mut reader),
            Err(ApkError::BadMagic(_))
        ));
    }

    #[test]
    fn parse_rejects_bad_endian_tag() {
        let mut bytes = valid_header_bytes();
        // endian_tag is at offset 8 + 4 + 20 + 4 + 4 = 40
        bytes[40..44].copy_from_slice(&0u32.to_le_bytes());
        let mut reader = BinaryReader::new(&bytes);
        assert!(matches!(
            DexHeader::parse(&mut reader),
            Err(ApkError::BadMagic(_))
        ));
    }

    #[test]
    fn parse_does_not_panic_on_truncated_input() {
        let bytes = &valid_header_bytes()[..10];
        let mut reader = BinaryReader::new(bytes);
        assert!(DexHeader::parse(&mut reader).is_err());
    }
}
