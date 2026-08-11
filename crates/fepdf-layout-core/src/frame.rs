//! Frame primitive definitions for DTP layout.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FrameKind {
    Text { content: String },
    Image { asset_id: String },
    Shape,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub id: u64,
    pub bounds: Rect,
    pub kind: FrameKind,
}
