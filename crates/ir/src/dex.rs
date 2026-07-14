use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// A single DEX file with classes and method instructions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DexFile {
    pub name: String,
    pub strings: Vec<String>,
    pub classes: Vec<Class>,
}

/// A class definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Method>,
    pub fields: Vec<Field>,
}

/// A method definition with its bytecode instructions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub access_flags: u32,
    pub code_offset: Option<u32>,
    pub instructions: Vec<InstructionAt>,
    /// Exception-handling ranges (try/catch), as data.
    ///
    /// Reserved for a future exception-aware CFG pass; CFG exception edges
    /// are explicitly out of scope for phase 2.3a. Empty for methods with
    /// no `try`/`catch` blocks (the common case).
    pub tries: Vec<TryRange>,
}

/// A decoded instruction paired with its code-unit offset (PC) within the
/// owning method's `insns`.
///
/// This is required to resolve `Instruction::Branch`/`Instruction::Switch`
/// targets against real instruction positions, and to build a CFG later.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionAt {
    /// Code-unit offset of this instruction within the method's `insns`.
    pub offset: u32,
    pub instruction: Instruction,
}

/// A single `try` range: the `[start_addr, end_addr)` code-unit range it
/// protects, and its handler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TryRange {
    pub start_addr: u32,
    /// Exclusive end of the protected range.
    pub end_addr: u32,
    pub handler: CatchHandler,
}

/// The handler for a `TryRange`: zero or more typed catches, plus an
/// optional catch-all handler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatchHandler {
    pub catches: Vec<CatchTypeAddr>,
    /// Code-unit offset of the catch-all handler, if present.
    pub catch_all_addr: Option<u32>,
}

/// A single caught exception type and its handler entry point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatchTypeAddr {
    pub class: ClassRef,
    /// Code-unit offset of the handler.
    pub handler_addr: u32,
}

/// A class field definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field {
    /// Field name (index into the DEX string pool).
    pub name: StringRef,
    /// Field type descriptor (index into the DEX string pool).
    pub ty: StringRef,
    /// Field access flags (e.g., public, private, static, final).
    pub access_flags: u32,
}

/// An index into the owning [`DexFile::strings`] pool.
///
/// References are interned as indices rather than copied `String`s so that
/// the IR can be built and passed around without re-parsing DEX data or
/// duplicating string content. Use [`StringRef::resolve`] to look up the
/// referenced string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StringRef(
    /// The 0-based index into the string pool.
    pub u32,
);

impl StringRef {
    /// Resolve this reference against a DEX string pool.
    ///
    /// Returns a borrowed string when the index is valid, or an owned
    /// `<bad_string:N>` placeholder when it is out of range. Never panics.
    pub fn resolve<'a>(&self, strings: &'a [String]) -> Cow<'a, str> {
        match strings.get(self.0 as usize) {
            Some(s) => Cow::Borrowed(s.as_str()),
            None => Cow::Owned(format!("<bad_string:{}>", self.0)),
        }
    }
}

/// A reference to a class, by its descriptor string (e.g. `Lcom/foo/Bar;`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassRef {
    /// Class descriptor string index.
    pub name: StringRef,
}

impl ClassRef {
    /// Resolve the class descriptor against a DEX string pool.
    pub fn display(&self, strings: &[String]) -> String {
        self.name.resolve(strings).into_owned()
    }
}

/// A reference to a method, by owning class, name, and shorty descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodRef {
    /// The owning class.
    pub class: ClassRef,
    /// Method name string index.
    pub name: StringRef,
    /// Method descriptor (prototype) string index.
    pub descriptor: StringRef,
}

impl MethodRef {
    /// Format as `class->name`, matching the previous composed-string
    /// representation used by the CLI and tests.
    pub fn display(&self, strings: &[String]) -> String {
        format!(
            "{}->{}",
            self.class.display(strings),
            self.name.resolve(strings)
        )
    }
}

/// A reference to a field, by owning class, name, and type descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldRef {
    /// The owning class.
    pub class: ClassRef,
    /// Field name string index.
    pub name: StringRef,
    /// Field type descriptor string index.
    pub ty: StringRef,
}

impl FieldRef {
    /// Format as `class->name`, matching the previous composed-string
    /// representation used by the CLI and tests.
    pub fn display(&self, strings: &[String]) -> String {
        format!(
            "{}->{}",
            self.class.display(strings),
            self.name.resolve(strings)
        )
    }
}

/// A basic block within a method's control-flow graph.
///
/// Reserved for the future CFG milestone; no current pass populates
/// `insn_range` or `successors`. The shape is locked in now so later work can
/// depend on it without another breaking change to the IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicBlock {
    pub id: u32,
    pub insn_range: (u32, u32),
    pub successors: Vec<u32>,
}

/// A method together with its control-flow graph.
///
/// Reserved for the future CFG milestone; `blocks` is always empty for now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Function {
    pub method: Method,
    pub blocks: Vec<BasicBlock>,
}

/// A decoded DEX instruction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Instruction {
    Const {
        register: u8,
        value: i32,
    },
    ConstString {
        register: u8,
        value: StringRef,
    },
    Invoke {
        kind: InvokeKind,
        method: MethodRef,
        registers: Vec<u8>,
    },
    FieldAccess {
        field: FieldRef,
    },
    NewInstance {
        class: ClassRef,
    },
    CheckCast {
        class: ClassRef,
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
    /// *taken* target, and control also falls through to the next
    /// instruction when the condition is false.
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchCase {
    /// The switch value (an explicit key for `sparse-switch`, or
    /// `first_key + n` for `packed-switch`).
    pub key: i32,
    /// Absolute code-unit offset (within the method's `insns`) of this
    /// case's target instruction.
    pub target: u32,
}

/// Kinds of method invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // StringRef::resolve tests
    // ============================================================================

    #[test]
    fn test_stringref_resolve_in_range() {
        let strings = vec!["Hello".to_string(), "World".to_string()];
        let ref0 = StringRef(0);
        let ref1 = StringRef(1);

        assert_eq!(ref0.resolve(&strings), "Hello");
        assert_eq!(ref1.resolve(&strings), "World");
    }

    #[test]
    fn test_stringref_resolve_out_of_range() {
        let strings = vec!["Hello".to_string()];
        let out_of_range = StringRef(5);
        let resolved = out_of_range.resolve(&strings);

        // Should return a fallback string without panicking
        assert!(resolved.contains("<bad_string:"));
        assert!(resolved.contains("5"));
    }

    #[test]
    fn test_stringref_resolve_no_panic_on_large_index() {
        let strings = vec!["Hello".to_string()];
        let large_index = StringRef(u32::MAX);

        // Must not panic
        let _result = large_index.resolve(&strings);
    }

    // ============================================================================
    // display() helper tests for ClassRef, MethodRef, FieldRef
    // ============================================================================

    #[test]
    fn test_classref_display() {
        let strings = vec!["Lcom/example/Test;".to_string()];
        let class_ref = ClassRef {
            name: StringRef(0),
        };

        assert_eq!(class_ref.display(&strings), "Lcom/example/Test;");
    }

    #[test]
    fn test_classref_display_with_fallback() {
        let strings = vec![];
        let class_ref = ClassRef {
            name: StringRef(0),
        };

        let displayed = class_ref.display(&strings);
        assert!(displayed.contains("<bad_string:"));
    }

    #[test]
    fn test_methodref_display() {
        let strings = vec![
            "Lcom/example/Test;".to_string(),
            "onCreate".to_string(),
            "(Landroid/os/Bundle;)V".to_string(),
        ];

        let method_ref = MethodRef {
            class: ClassRef {
                name: StringRef(0),
            },
            name: StringRef(1),
            descriptor: StringRef(2),
        };

        assert_eq!(
            method_ref.display(&strings),
            "Lcom/example/Test;->onCreate"
        );
    }

    #[test]
    fn test_methodref_display_with_fallback() {
        let strings = vec![];
        let method_ref = MethodRef {
            class: ClassRef {
                name: StringRef(0),
            },
            name: StringRef(1),
            descriptor: StringRef(2),
        };

        let displayed = method_ref.display(&strings);
        assert!(displayed.contains("<bad_string:"));
        assert!(displayed.contains("->"));
    }

    #[test]
    fn test_fieldref_display() {
        let strings = vec![
            "Lcom/example/Test;".to_string(),
            "myField".to_string(),
            "I".to_string(),
        ];

        let field_ref = FieldRef {
            class: ClassRef {
                name: StringRef(0),
            },
            name: StringRef(1),
            ty: StringRef(2),
        };

        assert_eq!(field_ref.display(&strings), "Lcom/example/Test;->myField");
    }

    #[test]
    fn test_fieldref_display_with_fallback() {
        let strings = vec![];
        let field_ref = FieldRef {
            class: ClassRef {
                name: StringRef(0),
            },
            name: StringRef(1),
            ty: StringRef(2),
        };

        let displayed = field_ref.display(&strings);
        assert!(displayed.contains("<bad_string:"));
        assert!(displayed.contains("->"));
    }

    // ============================================================================
    // Serde round-trip tests
    // ============================================================================

    #[test]
    fn test_serde_stringref_roundtrip() {
        let original = StringRef(42);
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: StringRef =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_serde_classref_roundtrip() {
        let original = ClassRef {
            name: StringRef(5),
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: ClassRef =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_serde_methodref_roundtrip() {
        let original = MethodRef {
            class: ClassRef {
                name: StringRef(0),
            },
            name: StringRef(1),
            descriptor: StringRef(2),
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: MethodRef =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_serde_fieldref_roundtrip() {
        let original = FieldRef {
            class: ClassRef {
                name: StringRef(10),
            },
            name: StringRef(11),
            ty: StringRef(12),
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: FieldRef =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_serde_field_roundtrip() {
        let original = Field {
            name: StringRef(7),
            ty: StringRef(8),
            access_flags: 0x0019,
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: Field =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_serde_instruction_const_string_roundtrip() {
        let original = Instruction::ConstString {
            register: 0,
            value: StringRef(3),
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: Instruction =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
        // Verify the tag is "ConstString" in the JSON
        assert!(json.contains("\"type\":\"ConstString\""));
    }

    #[test]
    fn test_serde_instruction_invoke_roundtrip() {
        let original = Instruction::Invoke {
            kind: InvokeKind::Virtual,
            method: MethodRef {
                class: ClassRef {
                    name: StringRef(0),
                },
                name: StringRef(1),
                descriptor: StringRef(2),
            },
            registers: vec![0, 1, 2],
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: Instruction =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
        assert!(json.contains("\"type\":\"Invoke\""));
    }

    #[test]
    fn test_serde_instruction_field_access_roundtrip() {
        let original = Instruction::FieldAccess {
            field: FieldRef {
                class: ClassRef {
                    name: StringRef(4),
                },
                name: StringRef(5),
                ty: StringRef(6),
            },
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: Instruction =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
        assert!(json.contains("\"type\":\"FieldAccess\""));
    }

    #[test]
    fn test_serde_instruction_new_instance_roundtrip() {
        let original = Instruction::NewInstance {
            class: ClassRef {
                name: StringRef(9),
            },
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: Instruction =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
        assert!(json.contains("\"type\":\"NewInstance\""));
    }

    #[test]
    fn test_serde_instruction_checkcast_roundtrip() {
        let original = Instruction::CheckCast {
            class: ClassRef {
                name: StringRef(13),
            },
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: Instruction =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
        assert!(json.contains("\"type\":\"CheckCast\""));
    }

    #[test]
    fn test_serde_instruction_branch_roundtrip() {
        let original = Instruction::Branch {
            kind: BranchKind::IfLez,
            target: 42,
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: Instruction =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
        assert!(json.contains("\"type\":\"Branch\""));
    }

    #[test]
    fn test_serde_instruction_switch_roundtrip() {
        let original = Instruction::Switch {
            packed: true,
            cases: vec![
                SwitchCase { key: 0, target: 10 },
                SwitchCase { key: 1, target: 20 },
            ],
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: Instruction =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
        assert!(json.contains("\"type\":\"Switch\""));
    }

    #[test]
    fn test_serde_instruction_at_roundtrip() {
        let original = InstructionAt {
            offset: 4,
            instruction: Instruction::Nop,
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: InstructionAt =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_serde_try_range_roundtrip() {
        let original = TryRange {
            start_addr: 0,
            end_addr: 5,
            handler: CatchHandler {
                catches: vec![CatchTypeAddr {
                    class: ClassRef {
                        name: StringRef(1),
                    },
                    handler_addr: 5,
                }],
                catch_all_addr: Some(8),
            },
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: TryRange =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_method_with_tries_and_instruction_offsets_roundtrip() {
        let original = Method {
            name: "Lcom/example/A;->guarded".to_string(),
            access_flags: 0x1,
            code_offset: Some(0x100),
            instructions: vec![
                InstructionAt {
                    offset: 0,
                    instruction: Instruction::Nop,
                },
                InstructionAt {
                    offset: 1,
                    instruction: Instruction::Branch {
                        kind: BranchKind::Goto,
                        target: 0,
                    },
                },
            ],
            tries: vec![TryRange {
                start_addr: 0,
                end_addr: 2,
                handler: CatchHandler {
                    catches: vec![],
                    catch_all_addr: Some(2),
                },
            }],
        };
        let json = serde_json::to_string(&original).expect("serialize failed");
        let deserialized: Method = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, deserialized);
    }
}
