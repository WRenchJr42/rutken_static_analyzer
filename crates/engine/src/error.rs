use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error(transparent)]
    Apk(#[from] apk::errors::ApkError),
}
