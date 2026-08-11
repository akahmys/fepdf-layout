//! Millimeter measurement units and PostScript Point conversions for PDF 2.0.

use serde::{Deserialize, Serialize};

/// Exact PostScript Points per inch (PDF 2.0 standard).
pub const POINTS_PER_INCH: f64 = 72.0;

/// Millimeters per inch (ISO standard).
pub const MM_PER_INCH: f64 = 25.4;

/// Millimeter value stored as an integer for 1mm grid snapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Mm(pub u32);

impl Mm {
    /// Zero millimeters.
    pub const ZERO: Self = Self(0);

    /// Create a new millimeter value.
    pub const fn new(mm: u32) -> Self {
        Self(mm)
    }

    /// Convert millimeters to PDF PostScript points (`f64`).
    /// 1 mm = 72 / 25.4 pt ≈ 2.83464567 pt.
    pub fn to_pt(self) -> f64 {
        (self.0 as f64) * POINTS_PER_INCH / MM_PER_INCH
    }
}

impl From<u32> for Mm {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// Floating point millimeter value for high-precision offset calculations.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MmF64(pub f64);

impl MmF64 {
    /// Zero millimeters.
    pub const ZERO: Self = Self(0.0);

    /// Create a new float millimeter value.
    pub const fn new(mm: f64) -> Self {
        Self(mm)
    }

    /// Convert millimeters to PDF PostScript points (`f64`).
    pub fn to_pt(self) -> f64 {
        self.0 * POINTS_PER_INCH / MM_PER_INCH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mm_to_pt() {
        let mm = Mm::new(254);
        // 254 mm = 10 inches = 720 points
        let pt = mm.to_pt();
        assert!((pt - 720.0).abs() < 1e-6);
    }
}
