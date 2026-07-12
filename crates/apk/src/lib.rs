//! APK parser layer: extracts APK/AXML/manifest/DEX data from raw bytes.
//!
//! Provides low-level parsing for binary file formats used in Android APK files,
//! with bounds-checked reads and format-complete parsing of all structures.

pub mod errors;
pub mod reader;
pub mod manifest;
pub mod axml;
pub mod binary;
pub mod dex;
