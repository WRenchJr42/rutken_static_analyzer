use crate::binary::BinaryReader;
use crate::errors::ApkError;

#[derive(Debug, Clone)]
pub struct EncodedField {
    /// Absolute index into `field_ids`, reconstructed from the DEX file's
    /// cumulative `field_idx_diff` encoding.
    pub field_idx: u32,
    pub access_flags: u32,
}

#[derive(Debug, Clone)]
pub struct EncodedMethod {
    pub method_idx: u32,
    pub access_flags: u32,
    pub code_off: u32,
}

#[derive(Debug)]
pub struct ClassData {
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
    pub static_fields: Vec<EncodedField>,
    #[allow(dead_code)]
    // parsed for format completeness; not yet consumed
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

        let read_fields = |reader: &mut BinaryReader, count: u32| -> Result<Vec<EncodedField>, ApkError> {
            let mut fields = Vec::new();
            let mut field_idx = 0u32;
            for _ in 0..count {
                field_idx = field_idx.wrapping_add(reader.read_uleb128()?);
                fields.push(
                    EncodedField {
                        field_idx,
                        access_flags: reader.read_uleb128()?,
                    }
                );
            }
            Ok(fields)
        };

        let static_fields = read_fields(reader, static_fields_size)?;
        let instance_fields = read_fields(reader, instance_fields_size)?;

        let read_methods = |reader: &mut BinaryReader, count: u32| -> Result<Vec<EncodedMethod>, ApkError> {
            let mut methods = Vec::new();
            let mut method_idx = 0u32;
            for _ in 0..count {
                method_idx = method_idx.wrapping_add(reader.read_uleb128()?);
                methods.push(
                    EncodedMethod {
                        method_idx,
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
