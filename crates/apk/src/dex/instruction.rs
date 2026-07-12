use serde::Serialize;

/// A decoded DEX instruction.
///
/// Class/method/field/string operands are represented as raw indices into
/// the enclosing DEX file's string pool (`DexDocument::strings`) rather than
/// pre-composed strings. This lets IR lowering intern them directly as
/// `ir::StringRef`s without re-parsing the DEX data. Table lookups (method
/// ids, field ids, type ids, proto ids) are already resolved to string-pool
/// indices at decode time; out-of-range table indices resolve to `u32::MAX`,
/// which is itself out of range for any string pool and is handled by
/// `ir::StringRef::resolve`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Instruction {
    Const {
        register: u8,
        value: i32,
    },
    ConstString {
        register: u8,
        /// String-pool index.
        string_idx: u32,
    },
    Invoke {
        kind: InvokeKind,
        /// String-pool index of the target class's descriptor.
        class_idx: u32,
        /// String-pool index of the method name.
        name_idx: u32,
        /// String-pool index of the method's shorty descriptor.
        descriptor_idx: u32,
        registers: Vec<u8>,
    },
    FieldAccess {
        /// String-pool index of the target class's descriptor.
        class_idx: u32,
        /// String-pool index of the field name.
        name_idx: u32,
        /// String-pool index of the field's type descriptor.
        type_idx: u32,
    },
    NewInstance {
        /// String-pool index of the class descriptor.
        class_idx: u32,
    },
    CheckCast {
        /// String-pool index of the class descriptor.
        class_idx: u32,
    },

    MoveResult {
        register: u8,
    },

    Return,
    Throw,
    Nop,
    Payload,
    Branch {
        kind: BranchKind,
    },
    Unknown {
        opcode: u8,
        raw: u16,
    },
}

/// Kinds of method invocation.
#[derive(Debug, Clone, Serialize)]
pub enum InvokeKind {
    Static,
    Virtual,
    Direct,
    Super,
    Interface,
}

/// Kinds of control flow branches.
#[derive(Debug, Clone, Serialize)]
pub enum BranchKind {
    Goto,
    IfEqz,
    IfNez,
}
