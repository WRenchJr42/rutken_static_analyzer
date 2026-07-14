//! Cross-reference (XREF) engine: builds and queries call/usage graphs
//! derived from the Rutken IR.

pub mod builder;
pub mod database;
pub mod query;

pub use builder::AnalysisContext;
pub use database::{Callee, ClassId, FieldId, MethodId, XrefDatabase};
