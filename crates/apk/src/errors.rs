use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}
