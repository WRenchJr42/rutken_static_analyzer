use serde::Serialize;

/// A decoded DEX instruction paired with its code-unit offset (PC) within
/// the owning method's `insns` array.
///
/// The offset is derived from the running program counter as instructions
/// are decoded (widths vary, so it cannot be recovered from a `Vec` index).
/// This is required to resolve branch/switch targets and, later, to build a
/// CFG.
#[derive(Debug, Clone, Serialize)]
pub struct InstructionAt {
    /// Code-unit offset of this instruction within the method's `insns`.
    pub offset: u32,
    pub instruction: Instruction,
}

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
    /// A `goto`/`goto-16`/`goto-32` or `if-*` branch.
    ///
    /// `target` is the absolute code-unit offset (within the method's
    /// `insns`) that the branch may jump to. For `BranchKind::Goto` this is
    /// the only successor (no fallthrough); for all other kinds this is the
    /// *taken* target and control also falls through to the next
    /// instruction when the condition is false.
    ///
    /// Out-of-range targets (corrupt/obfuscated input) are clamped rather
    /// than allowed to over/underflow; see `disasm::resolve_target`.
    Branch {
        kind: BranchKind,
        target: u32,
    },
    /// A `packed-switch` or `sparse-switch` instruction.
    ///
    /// `cases` holds the decoded `(key, target)` pairs from the switch
    /// payload; `target` is an absolute code-unit offset within the
    /// method's `insns`. There is always an implicit default fallthrough to
    /// the instruction immediately after the switch (not represented here).
    Switch {
        packed: bool,
        cases: Vec<SwitchCase>,
    },
    Unknown {
        opcode: u8,
        raw: u16,
    },
}

/// A single `(key, target)` case decoded from a switch payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SwitchCase {
    /// The switch value (an explicit key for `sparse-switch`, or
    /// `first_key + n` for `packed-switch`).
    pub key: i32,
    /// Absolute code-unit offset (within the method's `insns`) of this
    /// case's target instruction.
    pub target: u32,
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
///
/// The `if-*` mnemonic is preserved where cheap to do so; all `if-*`
/// variants share the same "conditional, with fallthrough" CFG shape.
#[derive(Debug, Clone, Serialize)]
pub enum BranchKind {
    Goto,
    IfEq,
    IfNe,
    IfLt,
    IfGe,
    IfGt,
    IfLe,
    IfEqz,
    IfNez,
    IfLtz,
    IfGez,
    IfGtz,
    IfLez,
}
