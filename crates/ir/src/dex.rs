use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DexFile {
    pub name: String,
    pub strings: Vec<String>,
    pub classes: Vec<Class>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Method>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub access_flags: u32,
    pub code_offset: Option<u32>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvokeKind {
    Static,
    Virtual,
    Direct,
    Super,
    Interface,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchKind {
    Goto,
    IfEqz,
    IfNez,
}
