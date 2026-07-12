//! Intermediate representation (IR) layer: serializable data model for APK analysis.
//!
//! Defines the stable IR structures that bridge parsed APK data and analysis,
//! suitable for serialization and external processing.

pub mod apk;
pub mod dex;
pub mod findings;
pub mod manifest;
pub mod metadata;

pub use apk::ApkIR;
pub use dex::{
    BasicBlock, BranchKind, Class, ClassRef, DexFile, Field, FieldRef, Function, Instruction,
    InvokeKind, Method, MethodRef, StringRef,
};
pub use findings::{Finding, FindingSeverity};
pub use manifest::{Activity, Application, Manifest};
pub use metadata::Metadata;
