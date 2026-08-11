//! Asset import utilities for `fepdf-layout`.

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

pub type Result<T> = std::result::Result<T, ImportError>;
