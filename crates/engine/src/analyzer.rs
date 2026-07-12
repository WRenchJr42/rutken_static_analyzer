use std::path::Path;

use apk::reader::{ApkContainer, ApkReader};
use ir::ApkIR;

use crate::builder::IrBuilder;
use crate::error::EngineError;

/// Static analyzer for APK files.
#[derive(Debug, Clone)]
pub struct Analyzer {
    container: ApkContainer,
}

impl Analyzer {
    /// Open and parse an APK file.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        Ok(Self {
            container: ApkReader::read(path)?,
        })
    }

    /// Build IR and run analysis on the loaded APK.
    pub fn analyze(&self) -> Result<ApkIR, EngineError> {
        IrBuilder::new(&self.container).build()
    }
}
