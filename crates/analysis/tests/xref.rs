//! Integration tests for the XREF engine, built against synthetic
//! `ir::ApkIR` fixtures (no dependency on real APKs/DEX bytes).

use analysis::AnalysisContext;
use ir::{
    ApkIR, Class, ClassRef, DexFile, Field, FieldRef, Instruction, InstructionAt, InvokeKind,
    Metadata, Method, MethodRef, StringRef,
};

fn empty_metadata() -> Metadata {
    Metadata {
        sha256: None,
        size: None,
        dex_files: vec![],
        architectures: vec![],
    }
}

fn ir_with(dex_files: Vec<DexFile>) -> ApkIR {
    ApkIR {
        metadata: empty_metadata(),
        manifest: None,
        dex_files,
        findings: vec![],
    }
}

fn method(name: &str, access_flags: u32, instructions: Vec<Instruction>) -> Method {
    Method {
        name: name.to_string(),
        access_flags,
        code_offset: (!instructions.is_empty()).then_some(0x10),
        instructions: instructions
            .into_iter()
            .enumerate()
            .map(|(offset, instruction)| InstructionAt {
                offset: offset as u32,
                instruction,
            })
            .collect(),
        tries: vec![],
    }
}

// ============================================================================
// Method invocation: caller/callee
// ============================================================================

#[test]
fn resolves_internal_call_between_two_defined_methods() {
    // Lcom/example/A;->caller() invokes Lcom/example/B;->callee()
    let strings = vec![
        "Lcom/example/A;".to_string(),
        "caller".to_string(),
        "Lcom/example/B;".to_string(),
        "callee".to_string(),
        "()V".to_string(),
    ];

    let call = Instruction::Invoke {
        kind: InvokeKind::Virtual,
        method: MethodRef {
            class: ClassRef { name: StringRef(2) },
            name: StringRef(3),
            descriptor: StringRef(4),
        },
        registers: vec![0],
    };

    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings,
        classes: vec![
            Class {
                name: "Lcom/example/A;".to_string(),
                methods: vec![method("Lcom/example/A;->caller", 0x1, vec![call])],
                fields: vec![],
            },
            Class {
                name: "Lcom/example/B;".to_string(),
                methods: vec![method("Lcom/example/B;->callee", 0x1, vec![])],
                fields: vec![],
            },
        ],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let caller_id = db
        .method_id("Lcom/example/A;", "caller")
        .expect("caller should be interned");
    let callee_id = db
        .method_id("Lcom/example/B;", "callee")
        .expect("callee should be interned");

    let callees = db.find_callees(caller_id);
    assert_eq!(callees, vec![analysis::Callee::Internal(callee_id)]);

    let callers = db.find_callers(callee_id);
    assert_eq!(callers, vec![caller_id]);
}

#[test]
fn distinguishes_external_call_targets() {
    // Lcom/example/A;->caller() invokes Landroid/app/Activity;->onCreate,
    // which has no definition anywhere in the IR.
    let strings = vec![
        "Lcom/example/A;".to_string(),
        "caller".to_string(),
        "Landroid/app/Activity;".to_string(),
        "onCreate".to_string(),
        "(Landroid/os/Bundle;)V".to_string(),
    ];

    let call = Instruction::Invoke {
        kind: InvokeKind::Virtual,
        method: MethodRef {
            class: ClassRef { name: StringRef(2) },
            name: StringRef(3),
            descriptor: StringRef(4),
        },
        registers: vec![0, 1],
    };

    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings,
        classes: vec![Class {
            name: "Lcom/example/A;".to_string(),
            methods: vec![method("Lcom/example/A;->caller", 0x1, vec![call])],
            fields: vec![],
        }],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let caller_id = db.method_id("Lcom/example/A;", "caller").unwrap();
    let callees = db.find_callees(caller_id);

    assert_eq!(
        callees,
        vec![analysis::Callee::External(
            "Landroid/app/Activity;->onCreate".to_string()
        )]
    );
}

// ============================================================================
// Field access usage
// ============================================================================

#[test]
fn tracks_field_access_usage() {
    let strings = vec![
        "Lcom/example/A;".to_string(),
        "user".to_string(),
        "field".to_string(),
        "Ljava/lang/String;".to_string(),
    ];

    let field_access = Instruction::FieldAccess {
        field: FieldRef {
            class: ClassRef { name: StringRef(0) },
            name: StringRef(2),
            ty: StringRef(3),
        },
    };

    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings,
        classes: vec![Class {
            name: "Lcom/example/A;".to_string(),
            methods: vec![method("Lcom/example/A;->user", 0x1, vec![field_access])],
            fields: vec![Field {
                name: StringRef(2),
                ty: StringRef(3),
                access_flags: 0x2,
            }],
        }],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let method_id = db.method_id("Lcom/example/A;", "user").unwrap();
    let field_id = db
        .field_id("Lcom/example/A;", "field")
        .expect("field should be interned from its definition");

    assert_eq!(db.find_field_usages(field_id), vec![method_id]);
    assert_eq!(db.field_type(field_id), Some("Ljava/lang/String;"));
}

// ============================================================================
// const-string usage
// ============================================================================

#[test]
fn tracks_const_string_usage() {
    let strings = vec![
        "Lcom/example/A;".to_string(),
        "log".to_string(),
        "hello world".to_string(),
    ];

    let const_string = Instruction::ConstString {
        register: 0,
        value: StringRef(2),
    };

    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings,
        classes: vec![Class {
            name: "Lcom/example/A;".to_string(),
            methods: vec![method("Lcom/example/A;->log", 0x1, vec![const_string])],
            fields: vec![],
        }],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let method_id = db.method_id("Lcom/example/A;", "log").unwrap();
    assert_eq!(db.find_string_usages("hello world"), vec![method_id]);
    assert_eq!(db.find_string_usages("no such string"), Vec::new());
}

// ============================================================================
// Class references: NewInstance / CheckCast
// ============================================================================

#[test]
fn tracks_new_instance_and_check_cast_class_references() {
    let strings = vec![
        "Lcom/example/A;".to_string(),
        "build".to_string(),
        "Lcom/example/Widget;".to_string(),
    ];

    let new_instance = Instruction::NewInstance {
        class: ClassRef { name: StringRef(2) },
    };
    let check_cast = Instruction::CheckCast {
        class: ClassRef { name: StringRef(2) },
    };

    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings,
        classes: vec![Class {
            name: "Lcom/example/A;".to_string(),
            methods: vec![method(
                "Lcom/example/A;->build",
                0x1,
                vec![new_instance, check_cast],
            )],
            fields: vec![],
        }],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let method_id = db.method_id("Lcom/example/A;", "build").unwrap();
    let class_id = db.class_id("Lcom/example/Widget;").unwrap();

    // Two references from the same method dedup to a single entry.
    assert_eq!(db.find_class_references(class_id), vec![method_id]);
}

// ============================================================================
// Multi-DEX: same StringRef index means different strings in each DexFile
// ============================================================================

#[test]
fn multi_dex_string_indices_do_not_collide() {
    // Index 2 in dex1's pool is "Lcom/dex1/Target;->m", but index 2 in
    // dex2's pool is a totally different string. Both DexFiles have a
    // method that invokes index-2-in-their-own-pool; results must not mix.
    let dex1 = DexFile {
        name: "classes.dex".to_string(),
        strings: vec![
            "Lcom/dex1/Caller;".to_string(),
            "go".to_string(),
            "Lcom/dex1/Target;".to_string(),
            "hit".to_string(),
            "()V".to_string(),
        ],
        classes: vec![Class {
            name: "Lcom/dex1/Caller;".to_string(),
            methods: vec![method(
                "Lcom/dex1/Caller;->go",
                0x1,
                vec![Instruction::Invoke {
                    kind: InvokeKind::Static,
                    method: MethodRef {
                        class: ClassRef { name: StringRef(2) },
                        name: StringRef(3),
                        descriptor: StringRef(4),
                    },
                    registers: vec![],
                }],
            )],
            fields: vec![],
        }],
    };

    let dex2 = DexFile {
        name: "classes2.dex".to_string(),
        // Same indices (2, 3, 4) point at entirely different strings here.
        strings: vec![
            "Lcom/dex2/Caller;".to_string(),
            "go".to_string(),
            "Lcom/dex2/Other;".to_string(),
            "different".to_string(),
            "()V".to_string(),
        ],
        classes: vec![Class {
            name: "Lcom/dex2/Caller;".to_string(),
            methods: vec![method(
                "Lcom/dex2/Caller;->go",
                0x1,
                vec![Instruction::Invoke {
                    kind: InvokeKind::Static,
                    method: MethodRef {
                        class: ClassRef { name: StringRef(2) },
                        name: StringRef(3),
                        descriptor: StringRef(4),
                    },
                    registers: vec![],
                }],
            )],
            fields: vec![],
        }],
    };

    let ir = ir_with(vec![dex1, dex2]);
    let db = AnalysisContext::new(&ir).build();

    let caller1 = db.method_id("Lcom/dex1/Caller;", "go").unwrap();
    let caller2 = db.method_id("Lcom/dex2/Caller;", "go").unwrap();

    assert_eq!(
        db.find_callees(caller1),
        vec![analysis::Callee::External(
            "Lcom/dex1/Target;->hit".to_string()
        )]
    );
    assert_eq!(
        db.find_callees(caller2),
        vec![analysis::Callee::External(
            "Lcom/dex2/Other;->different".to_string()
        )]
    );
}

// ============================================================================
// Empty / native / abstract methods
// ============================================================================

#[test]
fn empty_method_has_no_edges() {
    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings: vec!["Lcom/example/A;".to_string(), "empty".to_string()],
        classes: vec![Class {
            name: "Lcom/example/A;".to_string(),
            methods: vec![method("Lcom/example/A;->empty", 0x1, vec![])],
            fields: vec![],
        }],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let method_id = db.method_id("Lcom/example/A;", "empty").unwrap();
    assert_eq!(db.find_callees(method_id), Vec::new());
    assert_eq!(db.find_callers(method_id), Vec::new());
}

#[test]
fn native_method_with_no_instructions_is_interned_without_edges() {
    const ACC_NATIVE: u32 = 0x100;

    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings: vec!["Lcom/example/A;".to_string(), "nativeMethod".to_string()],
        classes: vec![Class {
            name: "Lcom/example/A;".to_string(),
            methods: vec![Method {
                name: "Lcom/example/A;->nativeMethod".to_string(),
                access_flags: ACC_NATIVE,
                code_offset: None,
                instructions: vec![],
                tries: vec![],
            }],
            fields: vec![],
        }],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let method_id = db
        .method_id("Lcom/example/A;", "nativeMethod")
        .expect("native method should still be interned as a definition");
    assert_eq!(db.find_callees(method_id), Vec::new());
}

#[test]
fn abstract_method_with_no_instructions_is_interned_without_edges() {
    const ACC_ABSTRACT: u32 = 0x400;

    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings: vec!["Lcom/example/A;".to_string(), "abstractMethod".to_string()],
        classes: vec![Class {
            name: "Lcom/example/A;".to_string(),
            methods: vec![Method {
                name: "Lcom/example/A;->abstractMethod".to_string(),
                access_flags: ACC_ABSTRACT,
                code_offset: None,
                instructions: vec![],
                tries: vec![],
            }],
            fields: vec![],
        }],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let method_id = db
        .method_id("Lcom/example/A;", "abstractMethod")
        .expect("abstract method should still be interned as a definition");
    assert_eq!(db.find_callees(method_id), Vec::new());
    assert_eq!(db.find_callers(method_id), Vec::new());
}

// ============================================================================
// Unknown handles / keys return empty results rather than panicking
// ============================================================================

#[test]
fn unknown_lookups_return_none_or_empty() {
    let ir = ir_with(vec![]);
    let db = AnalysisContext::new(&ir).build();

    assert_eq!(db.method_id("Lno/such;", "method"), None);
    assert_eq!(db.class_id("Lno/such;"), None);
    assert_eq!(db.field_id("Lno/such;", "field"), None);
    assert_eq!(db.find_string_usages("nothing"), Vec::new());
}

// ============================================================================
// Cross-DEX method resolution: a method in DEX A calling a method in DEX B
// ============================================================================

#[test]
fn cross_dex_method_invocation_resolves_as_internal() {
    // Lcom/dex1/Caller;->invoke() calls Lcom/dex2/Target;->method()
    // Both DEXes define these methods; the invocation should resolve as Internal.
    let dex1 = DexFile {
        name: "classes.dex".to_string(),
        strings: vec![
            "Lcom/dex1/Caller;".to_string(),
            "invoke".to_string(),
            "Lcom/dex2/Target;".to_string(),
            "method".to_string(),
            "()V".to_string(),
        ],
        classes: vec![Class {
            name: "Lcom/dex1/Caller;".to_string(),
            methods: vec![method(
                "Lcom/dex1/Caller;->invoke",
                0x1,
                vec![Instruction::Invoke {
                    kind: InvokeKind::Static,
                    method: MethodRef {
                        class: ClassRef { name: StringRef(2) },
                        name: StringRef(3),
                        descriptor: StringRef(4),
                    },
                    registers: vec![],
                }],
            )],
            fields: vec![],
        }],
    };

    let dex2 = DexFile {
        name: "classes2.dex".to_string(),
        strings: vec![
            "Lcom/dex2/Target;".to_string(),
            "method".to_string(),
            "()V".to_string(),
        ],
        classes: vec![Class {
            name: "Lcom/dex2/Target;".to_string(),
            methods: vec![method("Lcom/dex2/Target;->method", 0x1, vec![])],
            fields: vec![],
        }],
    };

    let ir = ir_with(vec![dex1, dex2]);
    let db = AnalysisContext::new(&ir).build();

    let caller_id = db.method_id("Lcom/dex1/Caller;", "invoke").unwrap();
    let callee_id = db.method_id("Lcom/dex2/Target;", "method").unwrap();

    let callees = db.find_callees(caller_id);
    assert_eq!(callees, vec![analysis::Callee::Internal(callee_id)]);

    let callers = db.find_callers(callee_id);
    assert_eq!(callers, vec![caller_id]);
}

// ============================================================================
// Recursive calls: a method invoking itself
// ============================================================================

#[test]
fn recursive_method_appears_in_callers_and_callees_once() {
    // Lcom/example/A;->recursive() invokes itself.
    let strings = vec![
        "Lcom/example/A;".to_string(),
        "recursive".to_string(),
        "()V".to_string(),
    ];

    let call_self = Instruction::Invoke {
        kind: InvokeKind::Virtual,
        method: MethodRef {
            class: ClassRef { name: StringRef(0) },
            name: StringRef(1),
            descriptor: StringRef(2),
        },
        registers: vec![0],
    };

    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings,
        classes: vec![Class {
            name: "Lcom/example/A;".to_string(),
            methods: vec![method("Lcom/example/A;->recursive", 0x1, vec![call_self])],
            fields: vec![],
        }],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let method_id = db.method_id("Lcom/example/A;", "recursive").unwrap();

    // Verify dedup: the method should appear exactly once in both lists.
    let callees = db.find_callees(method_id);
    assert_eq!(
        callees,
        vec![analysis::Callee::Internal(method_id)],
        "recursive method should have itself as callee exactly once"
    );

    let callers = db.find_callers(method_id);
    assert_eq!(
        callers,
        vec![method_id],
        "recursive method should have itself as caller exactly once"
    );
}

// ============================================================================
// Self references: a method referencing its own class via NewInstance/CheckCast
// ============================================================================

#[test]
fn method_can_reference_its_own_class() {
    let strings = vec!["Lcom/example/A;".to_string(), "selfRef".to_string()];

    let new_self = Instruction::NewInstance {
        class: ClassRef { name: StringRef(0) },
    };
    let cast_self = Instruction::CheckCast {
        class: ClassRef { name: StringRef(0) },
    };

    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings,
        classes: vec![Class {
            name: "Lcom/example/A;".to_string(),
            methods: vec![method(
                "Lcom/example/A;->selfRef",
                0x1,
                vec![new_self, cast_self],
            )],
            fields: vec![],
        }],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let method_id = db.method_id("Lcom/example/A;", "selfRef").unwrap();
    let class_id = db.class_id("Lcom/example/A;").unwrap();

    // Two references to the same class in the same method should deduplicate.
    let references = db.find_class_references(class_id);
    assert_eq!(references, vec![method_id]);
}

// ============================================================================
// Duplicate references: same invoke multiple times in one method collapses
// to a single edge, with stable ordering
// ============================================================================

#[test]
fn duplicate_invocations_deduplicate_in_edges() {
    let strings = vec![
        "Lcom/example/A;".to_string(),
        "caller".to_string(),
        "Lcom/example/B;".to_string(),
        "target".to_string(),
        "()V".to_string(),
    ];

    let call = Instruction::Invoke {
        kind: InvokeKind::Virtual,
        method: MethodRef {
            class: ClassRef { name: StringRef(2) },
            name: StringRef(3),
            descriptor: StringRef(4),
        },
        registers: vec![0],
    };

    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings,
        classes: vec![
            Class {
                name: "Lcom/example/A;".to_string(),
                methods: vec![method(
                    "Lcom/example/A;->caller",
                    0x1,
                    vec![call.clone(), call.clone(), call],
                )],
                fields: vec![],
            },
            Class {
                name: "Lcom/example/B;".to_string(),
                methods: vec![method("Lcom/example/B;->target", 0x1, vec![])],
                fields: vec![],
            },
        ],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let caller_id = db.method_id("Lcom/example/A;", "caller").unwrap();
    let callee_id = db.method_id("Lcom/example/B;", "target").unwrap();

    let callees = db.find_callees(caller_id);
    // Three identical invocations should deduplicate to a single edge.
    assert_eq!(callees.len(), 1);
    assert_eq!(callees, vec![analysis::Callee::Internal(callee_id)]);

    let callers = db.find_callers(callee_id);
    // The caller should appear exactly once.
    assert_eq!(callers, vec![caller_id]);
}

#[test]
fn duplicate_field_accesses_deduplicate() {
    let strings = vec![
        "Lcom/example/A;".to_string(),
        "user".to_string(),
        "field".to_string(),
        "I".to_string(),
    ];

    let field_access = Instruction::FieldAccess {
        field: FieldRef {
            class: ClassRef { name: StringRef(0) },
            name: StringRef(2),
            ty: StringRef(3),
        },
    };

    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings,
        classes: vec![Class {
            name: "Lcom/example/A;".to_string(),
            methods: vec![method(
                "Lcom/example/A;->user",
                0x1,
                vec![field_access.clone(), field_access.clone(), field_access],
            )],
            fields: vec![Field {
                name: StringRef(2),
                ty: StringRef(3),
                access_flags: 0x2,
            }],
        }],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let method_id = db.method_id("Lcom/example/A;", "user").unwrap();
    let field_id = db.field_id("Lcom/example/A;", "field").unwrap();

    // Three identical field accesses should deduplicate to a single entry.
    let usages = db.find_field_usages(field_id);
    assert_eq!(usages, vec![method_id]);
}

#[test]
fn duplicate_string_references_deduplicate() {
    let strings = vec![
        "Lcom/example/A;".to_string(),
        "logger".to_string(),
        "debug_msg".to_string(),
    ];

    let const_string = Instruction::ConstString {
        register: 0,
        value: StringRef(2),
    };

    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings,
        classes: vec![Class {
            name: "Lcom/example/A;".to_string(),
            methods: vec![method(
                "Lcom/example/A;->logger",
                0x1,
                vec![
                    const_string.clone(),
                    const_string.clone(),
                    const_string.clone(),
                ],
            )],
            fields: vec![],
        }],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let method_id = db.method_id("Lcom/example/A;", "logger").unwrap();

    // Three identical string references should deduplicate to a single entry.
    let usages = db.find_string_usages("debug_msg");
    assert_eq!(usages, vec![method_id]);
}

// ============================================================================
// Mixed invoke kinds in a single method
// ============================================================================

#[test]
fn different_invoke_kinds_to_same_target_deduplicate() {
    // A single method makes multiple invocations to the same target,
    // but with different InvokeKind. They should still deduplicate to one edge.
    let strings = vec![
        "Lcom/example/A;".to_string(),
        "multi".to_string(),
        "Lcom/example/B;".to_string(),
        "target".to_string(),
        "()V".to_string(),
    ];

    let method_ref = MethodRef {
        class: ClassRef { name: StringRef(2) },
        name: StringRef(3),
        descriptor: StringRef(4),
    };

    let invoke_virtual = Instruction::Invoke {
        kind: InvokeKind::Virtual,
        method: method_ref.clone(),
        registers: vec![0],
    };

    let invoke_direct = Instruction::Invoke {
        kind: InvokeKind::Direct,
        method: method_ref,
        registers: vec![0],
    };

    let dex = DexFile {
        name: "classes.dex".to_string(),
        strings,
        classes: vec![
            Class {
                name: "Lcom/example/A;".to_string(),
                methods: vec![method(
                    "Lcom/example/A;->multi",
                    0x1,
                    vec![invoke_virtual, invoke_direct],
                )],
                fields: vec![],
            },
            Class {
                name: "Lcom/example/B;".to_string(),
                methods: vec![method("Lcom/example/B;->target", 0x1, vec![])],
                fields: vec![],
            },
        ],
    };

    let ir = ir_with(vec![dex]);
    let db = AnalysisContext::new(&ir).build();

    let caller_id = db.method_id("Lcom/example/A;", "multi").unwrap();
    let callee_id = db.method_id("Lcom/example/B;", "target").unwrap();

    let callees = db.find_callees(caller_id);
    // Even though there are two different invoke kinds, they target the same method,
    // so they should deduplicate to a single edge.
    assert_eq!(callees.len(), 1);
    assert_eq!(callees, vec![analysis::Callee::Internal(callee_id)]);
}
