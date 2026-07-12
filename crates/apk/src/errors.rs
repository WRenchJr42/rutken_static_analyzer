use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Invalid APK format: {0}")]
    InvalidFormat(String),

    /// The input buffer ended before all expected data could be read.
    #[error("Truncated data: {0}")]
    Truncated(String),

    /// A magic number / signature did not match the expected value.
    #[error("Bad magic: {0}")]
    BadMagic(String),

    /// An index (string id, type id, etc.) fell outside the valid range.
    #[error("Index out of range: {0}")]
    IndexOutOfRange(String),

    /// A chunk header was malformed (wrong type, bad size, etc.).
    #[error("Bad chunk: {0}")]
    BadChunk(String),
}
