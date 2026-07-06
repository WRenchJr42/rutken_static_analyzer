use crate::binary::BinaryReader;
use crate::errors::ApkError;

#[derive(Debug, Clone)]
pub struct EncodedField {
    pub field_idx_diff: u32,
    pub access_flags: u32,
}

#[derive(Debug, Clone)]
pub struct EncodedMethod {
    pub method_idx_diff: u32,
    pub access_flags: u32,
    pub code_off: u32,
}

#[derive(Debug)]
pub struct ClassData {
    pub static_fields: Vec<EncodedField>,
    pub instance_fields: Vec<EncodedField>,
    pub direct_methods: Vec<EncodedMethod>,
    pub virtual_methods: Vec<EncodedMethod>,
}

impl ClassData {
    pub fn parse(reader: &mut BinaryReader, offset: u32) -> Result<Self, ApkError> {
        reader.seek(offset as usize)?;
        let static_fields_size = reader.read_uleb128()?;
        let instance_fields_size = reader.read_uleb128()?;
        let direct_methods_size = reader.read_uleb128()?;
        let virtual_methods_size = reader.read_uleb128()?;

        let mut static_fields = Vec::new();
        for _ in 0..static_fields_size {
            static_fields.push(
                EncodedField {
                    field_idx_diff: reader.read_uleb128()?,
                    access_flags: reader.read_uleb128()?,
                }
            );
        }

        let mut instance_fields = Vec::new();
        for _  in 0..instance_fields_size {
            instance_fields.push(
                EncodedField {
                    field_idx_diff: reader.read_uleb128()?,
                    access_flags: reader.read_uleb128()?,
                }
            );
        }

        let read_methods = |reader: &mut BinaryReader, count: u32| -> Result<Vec<EncodedMethod>, ApkError> {
            let mut methods = Vec::new();
            for _ in 0..count {
                methods.push(
                    EncodedMethod {
                        method_idx_diff: reader.read_uleb128()?,
                        access_flags: reader.read_uleb128()?,
                        code_off: reader.read_uleb128()?,
                    }
                );
            }
            Ok(methods)
        };

        let direct_methods = read_methods(reader, direct_methods_size)?;
        let virtual_methods = read_methods(reader, virtual_methods_size)?;

        Ok(Self {
            static_fields,
            instance_fields,
            direct_methods,
            virtual_methods,
        })
    }
}
