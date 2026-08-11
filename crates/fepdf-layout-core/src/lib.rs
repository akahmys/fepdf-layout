//! `fepdf-layout-core`
//!
//! DTP Layout Engine & Document Model for `fepdf-layout`.

pub mod command;
pub mod frame;
pub mod history;

#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("Invalid frame bounds")]
    InvalidBounds,
    #[error("Document operation error: {0}")]
    EngineError(String),
}

pub type Result<T> = std::result::Result<T, LayoutError>;
