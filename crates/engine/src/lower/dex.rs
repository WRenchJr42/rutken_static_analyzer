//! Lowers `apk::dex` parse results into the stable `ir::dex` types.
//!
//! Class/method/field/string operands are already resolved to string-pool
//! indices by the `apk` disassembler, so lowering simply interns them as
//! `ir::StringRef`s pointing into the same pool (`DexModel::strings`), rather
//! than re-parsing or re-resolving anything.

use apk::dex::instruction as apk_instruction;
use apk::dex::model::{CatchHandlerModel, ClassModel, DexModel, FieldModel, MethodModel, TryModel};
use ir::{
    BranchKind, CatchHandler, CatchTypeAddr, Class, ClassRef, DexFile, Field, FieldRef,
    Instruction, InstructionAt, InvokeKind, Method, MethodRef, StringRef, SwitchCase, TryRange,
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
            .map(convert_instruction_at)
            .collect(),
        tries: method.tries.into_iter().map(convert_try).collect(),
    }
}

fn convert_instruction_at(instruction_at: apk_instruction::InstructionAt) -> InstructionAt {
    InstructionAt {
        offset: instruction_at.offset,
        instruction: convert_instruction(instruction_at.instruction),
    }
}

fn convert_try(try_model: TryModel) -> TryRange {
    TryRange {
        start_addr: try_model.start_addr,
        end_addr: try_model.end_addr,
        handler: convert_handler(try_model.handler),
    }
}

fn convert_handler(handler: CatchHandlerModel) -> CatchHandler {
    CatchHandler {
        catches: handler
            .catches
            .into_iter()
            .map(|c| CatchTypeAddr {
                class: ClassRef {
                    name: StringRef(c.class_idx),
                },
                handler_addr: c.handler_addr,
            })
            .collect(),
        catch_all_addr: handler.catch_all_addr,
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
        apk_instruction::Instruction::Branch { kind, target } => Instruction::Branch {
            kind: convert_branch_kind(kind),
            target,
        },
        apk_instruction::Instruction::Switch { packed, cases } => Instruction::Switch {
            packed,
            cases: cases
                .into_iter()
                .map(|c| SwitchCase {
                    key: c.key,
                    target: c.target,
                })
                .collect(),
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
        apk_instruction::BranchKind::IfEq => BranchKind::IfEq,
        apk_instruction::BranchKind::IfNe => BranchKind::IfNe,
        apk_instruction::BranchKind::IfLt => BranchKind::IfLt,
        apk_instruction::BranchKind::IfGe => BranchKind::IfGe,
        apk_instruction::BranchKind::IfGt => BranchKind::IfGt,
        apk_instruction::BranchKind::IfLe => BranchKind::IfLe,
        apk_instruction::BranchKind::IfEqz => BranchKind::IfEqz,
        apk_instruction::BranchKind::IfNez => BranchKind::IfNez,
        apk_instruction::BranchKind::IfLtz => BranchKind::IfLtz,
        apk_instruction::BranchKind::IfGez => BranchKind::IfGez,
        apk_instruction::BranchKind::IfGtz => BranchKind::IfGtz,
        apk_instruction::BranchKind::IfLez => BranchKind::IfLez,
    }
}
