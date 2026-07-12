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
    pub instructions: Vec<Instruction>,
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
    Branch {
        kind: BranchKind,
    },
    Unknown {
        opcode: u8,
        raw: u16,
    },
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchKind {
    Goto,
    IfEqz,
    IfNez,
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
}
