//! Builds an [`XrefDatabase`] by walking an [`ir::ApkIR`].
//!
//! The enclosing method of each instruction is known purely from the
//! iteration position (`dex_files -> classes -> methods -> instructions`);
//! there is no per-instruction backpointer in the IR, so the walk tracks
//! the current method's interned [`MethodId`] as it descends.

use ir::{ApkIR, DexFile, Instruction};

use super::database::{ClassId, FieldId, FieldSymbol, MethodId, MethodSymbol};
use super::{Callee, XrefDatabase};

/// Borrows an [`ApkIR`] and builds a cross-reference database from it.
///
/// ```
/// use analysis::AnalysisContext;
/// # let ir = ir::ApkIR {
/// #     metadata: ir::Metadata {
/// #         sha256: None,
/// #         size: None,
/// #         dex_files: vec![],
/// #         architectures: vec![],
/// #     },
/// #     manifest: None,
/// #     dex_files: vec![],
/// #     findings: vec![],
/// # };
/// let db = AnalysisContext::new(&ir).build();
/// ```
pub struct AnalysisContext<'ir> {
    ir: &'ir ApkIR,
}

impl<'ir> AnalysisContext<'ir> {
    /// Create a new analysis context over the given IR.
    pub fn new(ir: &'ir ApkIR) -> Self {
        Self { ir }
    }

    /// Build the cross-reference database.
    ///
    /// Two passes are made over the IR:
    ///
    /// 1. Intern every method *definition* across all DEX files, so that
    ///    call-target resolution in pass 2 is independent of declaration
    ///    order (a method may be called before it is defined, or defined
    ///    in a different DEX file than its caller).
    /// 2. Walk every instruction, resolving all `StringRef`/`ClassRef`/
    ///    `MethodRef`/`FieldRef` operands against the *current* DEX file's
    ///    string pool, and record call/usage edges.
    pub fn build(&self) -> XrefDatabase {
        let mut db = XrefDatabase::default();

        for dex in &self.ir.dex_files {
            for class in &dex.classes {
                for method in &class.methods {
                    intern_method(&mut db, &method.name);
                }
            }
        }

        for dex in &self.ir.dex_files {
            walk_dex(&mut db, dex);
        }

        db
    }
}

fn walk_dex(db: &mut XrefDatabase, dex: &DexFile) {
    let strings = dex.strings.as_slice();

    for class in &dex.classes {
        intern_class(db, &class.name);

        for field in &class.fields {
            // Resolve StringRefs to owned Strings using this DEX's string pool.
            let name = field.name.resolve(strings).into_owned();
            let ty = field.ty.resolve(strings).into_owned();
            intern_field(db, &class.name, &name, Some(ty));
        }

        for method in &class.methods {
            let Some(&caller) = db.method_index.get(&method.name) else {
                // Interned in pass 1; absence here would indicate a bug in
                // `build`, not a malformed IR. Skip defensively rather than
                // panic.
                continue;
            };

            for instruction_at in &method.instructions {
                // Walk instruction operands, resolving all StringRef/ClassRef/MethodRef/FieldRef
                // against this DEX's string pool, and record edges.
                record_instruction(db, caller, &instruction_at.instruction, strings);
            }
        }
    }
}

fn record_instruction(
    db: &mut XrefDatabase,
    caller: MethodId,
    instruction: &Instruction,
    strings: &[String],
) {
    match instruction {
        Instruction::Invoke { method, .. } => {
            let target_key = method.display(strings);
            let callee = match db.method_index.get(&target_key) {
                Some(&id) => Callee::Internal(id),
                None => Callee::External(target_key),
            };
            if let Callee::Internal(target) = callee {
                push_dedup(db.callers.entry(target).or_default(), caller);
            }
            push_dedup(db.callees.entry(caller).or_default(), callee);
        }
        Instruction::FieldAccess { field } => {
            let class = field.class.display(strings);
            let name = field.name.resolve(strings).into_owned();
            let ty = field.ty.resolve(strings).into_owned();
            let field_id = intern_field(db, &class, &name, Some(ty));
            push_dedup(db.field_usages.entry(field_id).or_default(), caller);
        }
        Instruction::ConstString { value, .. } => {
            let s = value.resolve(strings).into_owned();
            push_dedup(db.string_usages.entry(s).or_default(), caller);
        }
        Instruction::NewInstance { class } | Instruction::CheckCast { class } => {
            let name = class.display(strings);
            let class_id = intern_class(db, &name);
            push_dedup(db.class_references.entry(class_id).or_default(), caller);
        }
        _ => {}
    }
}

/// Intern a method definition keyed by its already-composed
/// `"class->name"` string (matching `ir::Method::name` and
/// `ir::MethodRef::display`). Idempotent.
fn intern_method(db: &mut XrefDatabase, composed_name: &str) -> MethodId {
    if let Some(&id) = db.method_index.get(composed_name) {
        return id;
    }
    let (class, name) = split_composed(composed_name);
    let id = MethodId(db.methods.len() as u32);
    db.methods.push(MethodSymbol { class, name });
    db.method_index.insert(composed_name.to_string(), id);
    id
}

/// Intern a class by descriptor string. Idempotent.
fn intern_class(db: &mut XrefDatabase, descriptor: &str) -> ClassId {
    if let Some(&id) = db.class_index.get(descriptor) {
        return id;
    }
    let id = ClassId(db.classes.len() as u32);
    db.classes.push(descriptor.to_string());
    db.class_index.insert(descriptor.to_string(), id);
    id
}

/// Intern a field by `(class, name)`, recording its type descriptor if
/// known. Idempotent; the first-seen type is kept if interned again.
fn intern_field(db: &mut XrefDatabase, class: &str, name: &str, ty: Option<String>) -> FieldId {
    let key = format!("{class}->{name}");
    if let Some(&id) = db.field_index.get(&key) {
        return id;
    }
    let id = FieldId(db.fields.len() as u32);
    db.fields.push(FieldSymbol {
        class: class.to_string(),
        name: name.to_string(),
        ty,
    });
    db.field_index.insert(key, id);
    id
}

/// Split a composed `"class->name"` string into its two components.
/// Falls back to an empty class name if the separator is absent, rather
/// than panicking.
fn split_composed(composed: &str) -> (String, String) {
    match composed.split_once("->") {
        Some((class, name)) => (class.to_string(), name.to_string()),
        None => (String::new(), composed.to_string()),
    }
}

fn push_dedup<T: PartialEq>(vec: &mut Vec<T>, item: T) {
    if !vec.contains(&item) {
        vec.push(item);
    }
}
