use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Instruction {
    Const {
        register: u8,
        value: i32,
    },
    ConstString {
        register: u8,
        value: String,
    },
    Invoke {
        kind: InvokeKind,
        method: String,
        registers: Vec<u8>,
    },
    FieldAccess {
        field: String,
    },
    NewInstance {
        class: String,
    },
    CheckCast {
        class: String,
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

#[derive(Debug, Clone, Serialize)]
pub enum InvokeKind {
    Static,
    Virtual,
    Direct,
    Super,
    Interface,
}

#[derive(Debug, Clone, Serialize)]
pub enum BranchKind {
    Goto,
    IfEqz,
    IfNez,
}
