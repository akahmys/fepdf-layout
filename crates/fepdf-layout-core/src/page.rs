//! Page presets, paper specifications, and active 1mm layout bounds.

use crate::units::{Mm, MmF64};
use serde::{Deserialize, Serialize};

/// Standard paper presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PagePreset {
    /// ISO A4 (210 x 297 mm) - Default
    A4,
    /// ISO A3 (297 x 420 mm)
    A3,
    /// ISO A5 (148 x 210 mm)
    A5,
    /// JIS B4 (257 x 364 mm)
    B4,
    /// JIS B5 (182 x 257 mm)
    B5,
    /// US Letter (Physical: 215.9 x 279.4 mm, Active: 215 x 279 mm)
    Letter,
    /// US Legal (Physical: 215.9 x 355.6 mm, Active: 215 x 355 mm)
    Legal,
    /// Custom page dimensions in whole millimeters (Width, Height)
    Custom { width: u32, height: u32 },
}

impl Default for PagePreset {
    fn default() -> Self {
        Self::A4
    }
}

/// Detailed specification for a page, including physical paper dimensions,
/// active 1mm layout area, and bottom-left origin offsets.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PageSpec {
    pub preset: PagePreset,
    /// Physical paper width in millimeters.
    pub paper_width: MmF64,
    /// Physical paper height in millimeters.
    pub paper_height: MmF64,
    /// Active layout width in integer millimeters (for 1mm grid snapping).
    pub layout_width: Mm,
    /// Active layout height in integer millimeters (for 1mm grid snapping).
    pub layout_height: Mm,
    /// Horizontal offset of active layout origin (0,0) from physical paper bottom-left.
    pub offset_x: MmF64,
    /// Vertical offset of active layout origin (0,0) from physical paper bottom-left.
    pub offset_y: MmF64,
}

impl PageSpec {
    /// Create a `PageSpec` for the given `PagePreset`.
    pub fn from_preset(preset: PagePreset) -> Self {
        match preset {
            PagePreset::A4 => Self {
                preset,
                paper_width: MmF64::new(210.0),
                paper_height: MmF64::new(297.0),
                layout_width: Mm::new(210),
                layout_height: Mm::new(297),
                offset_x: MmF64::ZERO,
                offset_y: MmF64::ZERO,
            },
            PagePreset::A3 => Self {
                preset,
                paper_width: MmF64::new(297.0),
                paper_height: MmF64::new(420.0),
                layout_width: Mm::new(297),
                layout_height: Mm::new(420),
                offset_x: MmF64::ZERO,
                offset_y: MmF64::ZERO,
            },
            PagePreset::A5 => Self {
                preset,
                paper_width: MmF64::new(148.0),
                paper_height: MmF64::new(210.0),
                layout_width: Mm::new(148),
                layout_height: Mm::new(210),
                offset_x: MmF64::ZERO,
                offset_y: MmF64::ZERO,
            },
            PagePreset::B4 => Self {
                preset,
                paper_width: MmF64::new(257.0),
                paper_height: MmF64::new(364.0),
                layout_width: Mm::new(257),
                layout_height: Mm::new(364),
                offset_x: MmF64::ZERO,
                offset_y: MmF64::ZERO,
            },
            PagePreset::B5 => Self {
                preset,
                paper_width: MmF64::new(182.0),
                paper_height: MmF64::new(257.0),
                layout_width: Mm::new(182),
                layout_height: Mm::new(257),
                offset_x: MmF64::ZERO,
                offset_y: MmF64::ZERO,
            },
            PagePreset::Letter => Self {
                preset,
                paper_width: MmF64::new(215.9),
                paper_height: MmF64::new(279.4),
                layout_width: Mm::new(215),
                layout_height: Mm::new(279),
                offset_x: MmF64::new(0.45),
                offset_y: MmF64::new(0.20),
            },
            PagePreset::Legal => Self {
                preset,
                paper_width: MmF64::new(215.9),
                paper_height: MmF64::new(355.6),
                layout_width: Mm::new(215),
                layout_height: Mm::new(355),
                offset_x: MmF64::new(0.45),
                offset_y: MmF64::new(0.30),
            },
            PagePreset::Custom { width, height } => Self {
                preset,
                paper_width: MmF64::new(width as f64),
                paper_height: MmF64::new(height as f64),
                layout_width: Mm::new(width),
                layout_height: Mm::new(height),
                offset_x: MmF64::ZERO,
                offset_y: MmF64::ZERO,
            },
        }
    }
}

impl Default for PageSpec {
    fn default() -> Self {
        Self::from_preset(PagePreset::A4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_letter_centering_offsets() {
        let spec = PageSpec::from_preset(PagePreset::Letter);
        assert_eq!(spec.layout_width, Mm::new(215));
        assert_eq!(spec.layout_height, Mm::new(279));
        assert!((spec.offset_x.0 - 0.45).abs() < 1e-6);
        assert!((spec.offset_y.0 - 0.20).abs() < 1e-6);
    }
}
