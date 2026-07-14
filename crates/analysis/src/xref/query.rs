//! Read-only query API over a built [`XrefDatabase`].
//!
//! All lookups on unknown/foreign handles return empty results rather than
//! panicking; internal indexing structures stay private, callers only see
//! opaque handles and owned `Vec`/`&str` results.

use super::database::{Callee, ClassId, FieldId, MethodId, XrefDatabase};

impl XrefDatabase {
    // ------------------------------------------------------------------
    // Handle lookup (canonical key -> Id)
    // ------------------------------------------------------------------

    /// Look up the [`MethodId`] for a method defined in the analyzed APK,
    /// identified by `(class_descriptor, method_name)`. Returns `None` if
    /// no such method was interned (e.g. it was only ever an unresolved
    /// call target, never a definition).
    ///
    /// # Panics
    /// Never panics; returns `None` for unknown methods.
    pub fn method_id(&self, class: &str, name: &str) -> Option<MethodId> {
        self.method_index.get(&format!("{class}->{name}")).copied()
    }

    /// Look up the [`ClassId`] for a class descriptor, if it was ever
    /// referenced (as a definition, `new-instance`, or `check-cast`
    /// target) while building the database.
    ///
    /// Returns `None` if the class was never encountered.
    pub fn class_id(&self, descriptor: &str) -> Option<ClassId> {
        self.class_index.get(descriptor).copied()
    }

    /// Look up the [`FieldId`] for a field identified by
    /// `(class_descriptor, field_name)`, if it was ever referenced (as a
    /// definition or a `FieldAccess` target) while building the database.
    ///
    /// Returns `None` if the field was never encountered.
    pub fn field_id(&self, class: &str, name: &str) -> Option<FieldId> {
        self.field_index.get(&format!("{class}->{name}")).copied()
    }

    // ------------------------------------------------------------------
    // Handle -> display name
    // ------------------------------------------------------------------

    /// The owning class descriptor of a method. Returns `None` for invalid handles.
    pub fn method_class(&self, id: MethodId) -> Option<&str> {
        self.methods.get(id.0 as usize).map(|m| m.class.as_str())
    }

    /// The simple (unqualified) name of a method. Returns `None` for invalid handles.
    pub fn method_name(&self, id: MethodId) -> Option<&str> {
        self.methods.get(id.0 as usize).map(|m| m.name.as_str())
    }

    /// The canonical `class->name` display string for a method. Returns `None` for invalid handles.
    pub fn method_display(&self, id: MethodId) -> Option<String> {
        self.methods
            .get(id.0 as usize)
            .map(|m| format!("{}->{}", m.class, m.name))
    }

    /// The class descriptor string for a class handle. Returns `None` for invalid handles.
    pub fn class_name(&self, id: ClassId) -> Option<&str> {
        self.classes.get(id.0 as usize).map(String::as_str)
    }

    /// The owning class descriptor of a field. Returns `None` for invalid handles.
    pub fn field_class(&self, id: FieldId) -> Option<&str> {
        self.fields.get(id.0 as usize).map(|f| f.class.as_str())
    }

    /// The simple (unqualified) name of a field. Returns `None` for invalid handles.
    pub fn field_name(&self, id: FieldId) -> Option<&str> {
        self.fields.get(id.0 as usize).map(|f| f.name.as_str())
    }

    /// The type descriptor of a field, if it was recorded. Returns `None` for invalid handles or if type is unrecorded.
    pub fn field_type(&self, id: FieldId) -> Option<&str> {
        self.fields.get(id.0 as usize).and_then(|f| f.ty.as_deref())
    }

    /// The canonical `class->name` display string for a field. Returns `None` for invalid handles.
    pub fn field_display(&self, id: FieldId) -> Option<String> {
        self.fields
            .get(id.0 as usize)
            .map(|f| format!("{}->{}", f.class, f.name))
    }

    // ------------------------------------------------------------------
    // Cross-reference queries
    // ------------------------------------------------------------------

    /// Methods that call the given method (reverse call adjacency).
    /// Results are deduplicated (each method appears at most once).
    /// Returns an empty vector for unknown or never-called methods.
    pub fn find_callers(&self, id: MethodId) -> Vec<MethodId> {
        self.callers.get(&id).cloned().unwrap_or_default()
    }

    /// Call targets reachable directly from the given method, distinguishing
    /// internal (defined-in-APK) targets from unresolved external symbols.
    /// Results are deduplicated (each target appears at most once).
    /// Returns an empty vector for unknown methods or methods with no
    /// `Invoke` instructions.
    pub fn find_callees(&self, id: MethodId) -> Vec<Callee> {
        self.callees.get(&id).cloned().unwrap_or_default()
    }

    /// Methods that reference the given string literal via `const-string`.
    /// Results are deduplicated (each method appears at most once).
    /// Returns an empty vector for unknown strings.
    pub fn find_string_usages(&self, value: &str) -> Vec<MethodId> {
        self.string_usages.get(value).cloned().unwrap_or_default()
    }

    /// Methods that access the given field.
    /// Results are deduplicated (each method appears at most once).
    /// Returns an empty vector for unknown fields.
    pub fn find_field_usages(&self, id: FieldId) -> Vec<MethodId> {
        self.field_usages.get(&id).cloned().unwrap_or_default()
    }

    /// Methods that reference the given class via `new-instance` or `check-cast`.
    /// Results are deduplicated (each method appears at most once).
    /// Returns an empty vector for unknown classes.
    pub fn find_class_references(&self, id: ClassId) -> Vec<MethodId> {
        self.class_references.get(&id).cloned().unwrap_or_default()
    }

    // ------------------------------------------------------------------
    // Aggregate metrics
    // ------------------------------------------------------------------

    /// Total number of call/usage edges recorded in this database: method
    /// call edges (both internal and external targets), plus field, string
    /// literal, and class-reference usage edges.
    ///
    /// Intended for coarse reporting (e.g. the CLI `stats` command), not
    /// as a precise graph-theoretic edge count.
    pub fn edge_count(&self) -> usize {
        let call_edges: usize = self.callees.values().map(Vec::len).sum();
        let field_edges: usize = self.field_usages.values().map(Vec::len).sum();
        let string_edges: usize = self.string_usages.values().map(Vec::len).sum();
        let class_edges: usize = self.class_references.values().map(Vec::len).sum();
        call_edges + field_edges + string_edges + class_edges
    }
}
