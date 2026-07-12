//! Lowers `apk::dex` parse results into the stable `ir::dex` types.
//!
//! Class/method/field/string operands are already resolved to string-pool
//! indices by the `apk` disassembler, so lowering simply interns them as
//! `ir::StringRef`s pointing into the same pool (`DexModel::strings`), rather
//! than re-parsing or re-resolving anything.

use apk::dex::instruction as apk_instruction;
use apk::dex::model::{ClassModel, DexModel, FieldModel, MethodModel};
use ir::{
    BranchKind, Class, ClassRef, DexFile, Field, FieldRef, Instruction, InvokeKind, Method,
    MethodRef, StringRef,
};

/// Lower a parsed [`DexModel`] into the stable IR [`DexFile`].
pub(crate) fn convert_dex_model(model: DexModel) -> DexFile {
    DexFile {
        name: model.name,
        strings: model.strings,
        classes: model.classes.into_iter().map(convert_class).collect(),
    }
}

fn convert_class(class: ClassModel) -> Class {
    Class {
        name: class.name,
        methods: class.methods.into_iter().map(convert_method).collect(),
        fields: class.fields.into_iter().map(convert_field).collect(),
    }
}

fn convert_method(method: MethodModel) -> Method {
    Method {
        name: method.name,
        access_flags: method.access_flags,
        code_offset: (method.code_off != 0).then_some(method.code_off),
        instructions: method
            .instructions
            .into_iter()
            .map(convert_instruction)
            .collect(),
    }
}

fn convert_field(field: FieldModel) -> Field {
    Field {
        name: StringRef(field.name_idx),
        ty: StringRef(field.type_idx),
        access_flags: field.access_flags,
    }
}

fn convert_instruction(instruction: apk_instruction::Instruction) -> Instruction {
    match instruction {
        apk_instruction::Instruction::Const { register, value } => {
            Instruction::Const { register, value }
        }
        apk_instruction::Instruction::ConstString {
            register,
            string_idx,
        } => Instruction::ConstString {
            register,
            value: StringRef(string_idx),
        },
        apk_instruction::Instruction::Invoke {
            kind,
            class_idx,
            name_idx,
            descriptor_idx,
            registers,
        } => Instruction::Invoke {
            kind: convert_invoke_kind(kind),
            method: MethodRef {
                class: ClassRef {
                    name: StringRef(class_idx),
                },
                name: StringRef(name_idx),
                descriptor: StringRef(descriptor_idx),
            },
            registers,
        },
        apk_instruction::Instruction::FieldAccess {
            class_idx,
            name_idx,
            type_idx,
        } => Instruction::FieldAccess {
            field: FieldRef {
                class: ClassRef {
                    name: StringRef(class_idx),
                },
                name: StringRef(name_idx),
                ty: StringRef(type_idx),
            },
        },
        apk_instruction::Instruction::NewInstance { class_idx } => Instruction::NewInstance {
            class: ClassRef {
                name: StringRef(class_idx),
            },
        },
        apk_instruction::Instruction::CheckCast { class_idx } => Instruction::CheckCast {
            class: ClassRef {
                name: StringRef(class_idx),
            },
        },
        apk_instruction::Instruction::MoveResult { register } => {
            Instruction::MoveResult { register }
        }
        apk_instruction::Instruction::Return => Instruction::Return,
        apk_instruction::Instruction::Throw => Instruction::Throw,
        apk_instruction::Instruction::Nop => Instruction::Nop,
        apk_instruction::Instruction::Payload => Instruction::Payload,
        apk_instruction::Instruction::Branch { kind } => Instruction::Branch {
            kind: convert_branch_kind(kind),
        },
        apk_instruction::Instruction::Unknown { opcode, raw } => {
            Instruction::Unknown { opcode, raw }
        }
    }
}

fn convert_invoke_kind(kind: apk_instruction::InvokeKind) -> InvokeKind {
    match kind {
        apk_instruction::InvokeKind::Static => InvokeKind::Static,
        apk_instruction::InvokeKind::Virtual => InvokeKind::Virtual,
        apk_instruction::InvokeKind::Direct => InvokeKind::Direct,
        apk_instruction::InvokeKind::Super => InvokeKind::Super,
        apk_instruction::InvokeKind::Interface => InvokeKind::Interface,
    }
}

fn convert_branch_kind(kind: apk_instruction::BranchKind) -> BranchKind {
    match kind {
        apk_instruction::BranchKind::Goto => BranchKind::Goto,
        apk_instruction::BranchKind::IfEqz => BranchKind::IfEqz,
        apk_instruction::BranchKind::IfNez => BranchKind::IfNez,
    }
}
