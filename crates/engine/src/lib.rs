//! Analysis engine: builds IR from parsed APK documents and performs analysis.
//!
//! Entry point for static analysis of APK files, combining parsed data into
//! the intermediate representation and running security analyzers.

pub mod analyzer;
pub mod error;

mod builder;
mod lower;

pub use analyzer::Analyzer;
pub use error::EngineError;
