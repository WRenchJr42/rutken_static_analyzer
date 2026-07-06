use crate::binary::BinaryReader;
use crate::errors::ApkError;

#[derive(Debug, Clone)]
pub struct ClassDef {
    pub class_idx: u32,
    pub access_flags: u32,
    pub superclass_idx: u32,
    pub interfaces_off: u32,
    pub source_file_idx: u32,
    pub annotations_off: u32,
    pub class_data_off: u32,
    pub static_values_off: u32,
}

#[derive(Debug)]
pub struct ClassDefs {
    pub classes: Vec<ClassDef>,
}

impl ClassDefs {
    pub fn parse(reader: &mut BinaryReader, count: u32, offset: u32) -> Result<Self, ApkError> {
        reader.seek(offset as usize)?;
        let mut classes = Vec::new();

        for _ in 0..count {
            classes.push(ClassDef {
                class_idx: reader.read_u32()?,
                access_flags: reader.read_u32()?,
                superclass_idx: reader.read_u32()?,
                interfaces_off: reader.read_u32()?,
                source_file_idx: reader.read_u32()?,
                annotations_off: reader.read_u32()?,
                class_data_off: reader.read_u32()?,
                static_values_off: reader.read_u32()?,
            });
        }
        Ok(Self {
            classes,
        })
    }
}
