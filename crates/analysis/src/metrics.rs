//! Coarse whole-APK metrics, intended for summary reporting (e.g. the CLI
//! `stats` command).
//!
//! Kept separate from the XREF/CFG internals so presentation layers never
//! need to reach into `ir` internals or `XrefDatabase` internals directly.

use ir::ApkIR;

use crate::xref::XrefDatabase;

/// Aggregate counts over an analyzed APK (classes, methods, instructions, strings, fields, xref edges).
///
/// Suitable for summary reporting and metrics display. All fields are counts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ApkStats {
    /// Total number of classes across all DEX files.
    pub classes: usize,
    /// Total number of methods across all classes.
    pub methods: usize,
    /// Total number of instructions across all methods.
    pub instructions: usize,
    /// Total number of strings in all DEX string pools.
    pub strings: usize,
    /// Total number of fields across all classes.
    pub fields: usize,
    /// Total number of cross-reference edges (call/usage graph edges) in the XrefDatabase.
    pub xref_edges: usize,
}

/// Compute [`ApkStats`] from the IR and an already-built [`XrefDatabase`].
///
/// `O(n)` in the number of classes/methods/instructions/fields across all DEX files.
/// Never panics; returns zero stats for empty IR.
pub fn compute(ir: &ApkIR, xref: &XrefDatabase) -> ApkStats {
    let mut stats = ApkStats {
        xref_edges: xref.edge_count(),
        ..Default::default()
    };

    for dex in &ir.dex_files {
        stats.strings += dex.strings.len();
        for class in &dex.classes {
            stats.classes += 1;
            stats.fields += class.fields.len();
            for method in &class.methods {
                stats.methods += 1;
                stats.instructions += method.instructions.len();
            }
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AnalysisContext;

    fn empty_ir() -> ApkIR {
        ApkIR {
            metadata: ir::Metadata {
                sha256: None,
                size: None,
                dex_files: vec![],
                architectures: vec![],
            },
            manifest: None,
            dex_files: vec![],
            findings: vec![],
        }
    }

    #[test]
    fn empty_ir_yields_zero_stats() {
        let ir = empty_ir();
        let db = AnalysisContext::new(&ir).build();
        let stats = compute(&ir, &db);
        assert_eq!(stats, ApkStats::default());
    }

    #[test]
    fn counts_classes_methods_instructions_strings_fields() {
        use ir::{Class, DexFile, Field, Instruction, InstructionAt, Method, StringRef};

        let mut ir = empty_ir();
        ir.dex_files.push(DexFile {
            name: "classes.dex".to_string(),
            strings: vec!["a".to_string(), "b".to_string()],
            classes: vec![Class {
                name: "Lfoo;".to_string(),
                fields: vec![Field {
                    name: StringRef(0),
                    ty: StringRef(1),
                    access_flags: 0,
                }],
                methods: vec![Method {
                    name: "Lfoo;->m".to_string(),
                    access_flags: 0,
                    code_offset: Some(0),
                    instructions: vec![
                        InstructionAt {
                            offset: 0,
                            instruction: Instruction::Nop,
                        },
                        InstructionAt {
                            offset: 1,
                            instruction: Instruction::Return,
                        },
                    ],
                    tries: vec![],
                }],
            }],
        });

        let db = AnalysisContext::new(&ir).build();
        let stats = compute(&ir, &db);

        assert_eq!(stats.classes, 1);
        assert_eq!(stats.methods, 1);
        assert_eq!(stats.instructions, 2);
        assert_eq!(stats.strings, 2);
        assert_eq!(stats.fields, 1);
    }
}
