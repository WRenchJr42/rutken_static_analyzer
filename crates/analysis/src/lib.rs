//! Analysis crate: cross-reference (XREF) engine over the Rutken IR.
//!
//! This crate builds an in-memory cross-reference database describing
//! method call edges and field/string/class usage across a parsed APK's
//! DEX files, and exposes O(1)/O(k) queries over it. It depends only on
//! the `ir` crate: it never re-parses APK data and never mutates the IR.
//!
//! # Method identity limitation (deferred to Call Graph milestone)
//!
//! Methods are identified solely by `(class_descriptor, method_name)`. The
//! current IR's method definitions carry no parameter descriptor, and
//! invocation targets (`ir::MethodRef::descriptor`) only capture the DEX
//! "shorty" prototype -- a coarse type-shape summary, not a full signature.
//! Neither is sufficient to distinguish overloaded methods that share a
//! name. As a result, overloaded methods sharing `(class, name)` are merged
//! into a single [`MethodId`] node in the cross-reference graph.
//! Disambiguating overloads by full parameter/return descriptor is deferred
//! to the Call Graph milestone, when richer signature information is
//! expected to be available on `ir::Method`.
//!
//! # StringRef resolution
//!
//! `ir::StringRef` indices are only valid against the `strings` pool of the
//! specific `ir::DexFile` that produced them; the same index means a
//! different string in a different DEX file. The builder in this crate
//! always resolves every `StringRef`/`ClassRef`/`MethodRef`/`FieldRef` into
//! an owned `String` using the `DexFile` currently being walked. All keys
//! and query results in this crate are canonical resolved strings or opaque
//! interned handles -- never raw string-pool indices.

pub mod cfg;
pub mod graph;
pub mod metrics;
pub mod xref;

pub use cfg::Cfg;
pub use metrics::{ApkStats, compute as compute_stats};
pub use xref::{AnalysisContext, Callee, ClassId, FieldId, MethodId, XrefDatabase};
