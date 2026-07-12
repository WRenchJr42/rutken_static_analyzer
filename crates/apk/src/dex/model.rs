use serde::Serialize;

use crate::binary::BinaryReader;
use crate::dex::class_data::{ClassData, EncodedField, EncodedMethod};
use crate::dex::disasm::{decode_instruction, resolve_field_components, resolve_method, resolve_type};
use crate::dex::instruction::Instruction;
use crate::dex::parser::{DexDocument, DexParser};
use crate::errors::ApkError;
use crate::reader::should_skip_class;

#[derive(Debug, Clone, Serialize)]
pub struct DexModel {
    pub name: String,
    pub strings: Vec<String>,
    pub classes: Vec<ClassModel>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClassModel {
    pub name: String,
    pub methods: Vec<MethodModel>,
    pub fields: Vec<FieldModel>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MethodModel {
    pub name: String,
    pub access_flags: u32,
    pub code_off: u32,
    pub instructions: Vec<Instruction>,
}

/// A class field definition, with `name`/`ty` already resolved to
/// string-pool indices so IR lowering can intern them without re-parsing.
#[derive(Debug, Clone, Serialize)]
pub struct FieldModel {
    /// String-pool index of the field's name.
    pub name_idx: u32,
    /// String-pool index of the field's type descriptor.
    pub type_idx: u32,
    pub access_flags: u32,
}

/// Parse a DEX file and build its model.
///
/// Out-of-range indices in the DEX file resolve to sentinel values like
/// `<bad_type:N>`, `<bad_method:N>`, and `<bad_string:N>` instead of panicking.
pub fn build_dex_model(name: impl Into<String>, bytes: &[u8]) -> Result<DexModel, ApkError> {
    let dex = DexParser::parse(bytes)?;
    let mut classes = Vec::new();

    for class in &dex.class_defs.classes {
        let class_name = resolve_class_name(&dex, class.class_idx as usize);

        if should_skip_class(&class_name) {
            continue;
        }

        if class.class_data_off == 0 {
            continue;
        }

        let class_data = ClassData::parse(&mut BinaryReader::new(bytes), class.class_data_off)?;
        let mut methods = Vec::new();

        methods.extend(build_methods(
            &dex,
            bytes,
            &class_data.direct_methods,
        )?);
        methods.extend(build_methods(
            &dex,
            bytes,
            &class_data.virtual_methods,
        )?);

        let mut fields = build_fields(&dex, &class_data.static_fields);
        fields.extend(build_fields(&dex, &class_data.instance_fields));

        classes.push(ClassModel {
            name: class_name,
            methods,
            fields,
        });
    }

    Ok(DexModel {
        name: name.into(),
        strings: dex.strings.strings.clone(),
        classes,
    })
}

pub fn resolve_class_name(dex: &DexDocument, class_idx: usize) -> String {
    resolve_type(class_idx, dex)
}

pub fn resolve_method_name(dex: &DexDocument, method_idx: usize) -> String {
    resolve_method(method_idx, dex)
}

pub fn decode_method_instructions(bytes: &[u8], dex: &DexDocument, code_off: u32) -> Result<Vec<Instruction>, ApkError> {
    let code = crate::dex::code_item::CodeItem::parse(&mut BinaryReader::new(bytes), code_off)?;
    let mut instructions = Vec::new();
    let mut pc = 0;

    while pc < code.instructions.len() {
        let (instruction, size) = decode_instruction(&code.instructions, pc, dex);
        instructions.push(instruction);
        if size == 0 {
            break;
        }
        pc += size;
    }

    Ok(instructions)
}

fn build_methods(dex: &DexDocument, bytes: &[u8], encoded_methods: &[EncodedMethod]) -> Result<Vec<MethodModel>, ApkError> {
    let mut methods = Vec::new();

    for encoded in encoded_methods {
        let name = resolve_method_name(dex, encoded.method_idx as usize);
        let instructions = if encoded.code_off == 0 {
            Vec::new()
        } else {
            decode_method_instructions(bytes, dex, encoded.code_off)?
        };

        methods.push(MethodModel {
            name,
            access_flags: encoded.access_flags,
            code_off: encoded.code_off,
            instructions,
        });
    }

    Ok(methods)
}

fn build_fields(dex: &DexDocument, encoded_fields: &[EncodedField]) -> Vec<FieldModel> {
    encoded_fields
        .iter()
        .map(|field| {
            let (_class_idx, name_idx, type_idx) =
                resolve_field_components(field.field_idx as usize, dex);
            FieldModel {
                name_idx,
                type_idx,
                access_flags: field.access_flags,
            }
        })
        .collect()
}
