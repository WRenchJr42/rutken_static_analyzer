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

impl DexHeader {
    pub fn parse(reader: &mut BinaryReader) -> Result<Self, ApkError> {
        Ok(Self {
            magic: reader.read_array::<8>()?,
            checksum: reader.read_u32()?,
            signature: reader.read_array::<20>()?,
            file_size: reader.read_u32()?,
            header_size: reader.read_u32()?,
            endian_tag: reader.read_u32()?,
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
