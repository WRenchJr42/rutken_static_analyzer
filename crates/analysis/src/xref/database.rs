//! Storage for the cross-reference database: interned symbol tables and
//! precomputed adjacency/usage maps.
//!
//! All internal indexing structures are private. Callers only ever see
//! opaque handles ([`MethodId`], [`FieldId`], [`ClassId`]) and resolve them
//! back to display strings through the accessors in [`super::query`].

use std::collections::HashMap;

/// Opaque handle to an interned method, identified by `(class, name)`.
///
/// See the crate-level docs for the overload-merge limitation this implies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MethodId(pub(super) u32);

/// Opaque handle to an interned field, identified by `(class, name)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldId(pub(super) u32);

/// Opaque handle to an interned class, identified by its descriptor string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassId(pub(super) u32);

/// A resolved invocation target: either an internal method defined
/// somewhere in the analyzed APK, or an unresolved external symbol (most
/// commonly a framework/library method with no definition in the APK).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Callee {
    /// The target resolves to a method defined in the analyzed APK.
    Internal(MethodId),
    /// The target does not resolve to any defined method in the APK.
    /// Carries the canonical `class->name` descriptor of the call target.
    External(String),
}

/// A method symbol: its owning class descriptor and simple name.
#[derive(Debug, Clone)]
pub(super) struct MethodSymbol {
    pub(super) class: String,
    pub(super) name: String,
}

/// A field symbol: its owning class descriptor, name, and (if known) type.
#[derive(Debug, Clone)]
pub(super) struct FieldSymbol {
    pub(super) class: String,
    pub(super) name: String,
    pub(super) ty: Option<String>,
}

/// Precomputed cross-reference database over an [`ir::ApkIR`].
///
/// Built via [`crate::AnalysisContext::build`]. All lookups are `O(1)` or
/// `O(k)` in the number of matching results.
#[derive(Debug, Default)]
pub struct XrefDatabase {
    pub(super) methods: Vec<MethodSymbol>,
    pub(super) method_index: HashMap<String, MethodId>,

    pub(super) fields: Vec<FieldSymbol>,
    pub(super) field_index: HashMap<String, FieldId>,

    pub(super) classes: Vec<String>,
    pub(super) class_index: HashMap<String, ClassId>,

    /// Forward adjacency: caller method -> resolved call targets.
    pub(super) callees: HashMap<MethodId, Vec<Callee>>,
    /// Reverse adjacency: internal callee method -> methods that call it.
    pub(super) callers: HashMap<MethodId, Vec<MethodId>>,

    /// String literal value -> methods that reference it via `const-string`.
    pub(super) string_usages: HashMap<String, Vec<MethodId>>,
    /// Field -> methods that access it.
    pub(super) field_usages: HashMap<FieldId, Vec<MethodId>>,
    /// Class -> methods that reference it (`new-instance` / `check-cast`).
    pub(super) class_references: HashMap<ClassId, Vec<MethodId>>,
}
