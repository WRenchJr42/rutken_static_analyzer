use std::path::Path;

use apk::reader::{ApkContainer, ApkReader};
use ir::ApkIR;

use crate::builder::IrBuilder;
use crate::error::EngineError;

#[derive(Debug, Clone)]
pub struct Analyzer {
    container: ApkContainer,
}

impl Analyzer {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        Ok(Self {
            container: ApkReader::read(path)?,
        })
    }

    pub fn analyze(&self) -> Result<ApkIR, EngineError> {
        IrBuilder::new(&self.container).build()
    }
}
