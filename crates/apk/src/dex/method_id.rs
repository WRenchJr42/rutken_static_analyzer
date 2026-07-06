use crate::binary::BinaryReader;
use crate::errors::ApkError;

#[derive(Debug, Clone)]
pub struct MethodId {
    pub class_idx: u16,
    pub proto_idx: u16,
    pub name_idx: u32,
}

#[derive(Debug)]
pub struct MethodIds {
    pub methods: Vec<MethodId>,
}

impl MethodIds {
    pub fn parse(reader: &mut BinaryReader, count: u32, offset: u32) -> Result<Self, ApkError> {
        reader.seek(offset as usize)?;
        let mut methods = Vec::new();
        for _ in 0..count {
            methods.push(MethodId {
                class_idx: reader.read_u16()?,
                proto_idx: reader.read_u16()?,
                name_idx: reader.read_u32()?,
            });
        }
        Ok(Self {
            methods,
        })
    }
}
