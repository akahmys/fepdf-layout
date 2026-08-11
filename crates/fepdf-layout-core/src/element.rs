//! Layout elements (Line, TextBox, FormField) and their properties.

use crate::units::Mm;
use serde::{Deserialize, Serialize};

/// Unique identifier for a layout element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ElementId(pub u64);

/// RGBA Color representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const GRAY_LIGHT: Self = Self { r: 204, g: 204, b: 204, a: 255 };
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };
}

/// Line stroke style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrokeStyle {
    Solid,
    Dashed,
    Dotted,
}

/// PDF 2.0 Line Cap Style (ISO 32000-2 Section 8.4.3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineCap {
    /// Butt cap: stroke ends abruptly at endpoint (平頭 - デフォルト)
    Butt,
    /// Round cap: semicircular end with radius = half line width (丸頭)
    Round,
    /// Square cap: extends beyond endpoint by half line width (突き出し角頭)
    Square,
}

impl Default for LineCap {
    fn default() -> Self {
        Self::Butt
    }
}

/// Text alignment within a text box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    /// Equal horizontal distribution of characters (均等割付).
    Justify,
}

/// Straight line element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineElement {
    pub id: ElementId,
    pub x1: Mm,
    pub y1: Mm,
    pub x2: Mm,
    pub y2: Mm,
    pub stroke_width: Mm,
    pub stroke_color: Color,
    pub stroke_style: StrokeStyle,
    pub line_cap: LineCap,
}

/// Text box element with horizontal scaling and auto-fit options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextBoxElement {
    pub id: ElementId,
    pub x: Mm,
    pub y: Mm,
    pub width: Mm,
    pub height: Mm,
    pub text: String,
    pub font_family: String,
    pub font_size_pt: f64,
    pub text_color: Color,
    pub align: TextAlign,
    /// Horizontal scaling percentage (100 = default, <100 = 長体, >100 = 平体).
    pub horizontal_scaling: u32,
    /// Auto-adjust horizontal scaling to fit text box width.
    pub auto_fit_horizontal: bool,
}

/// PDF 2.0 Form Field kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormFieldKind {
    TextField,
    CheckBox,
    RadioButton,
    ComboBox,
}

/// Interactive PDF form field widget element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormFieldElement {
    pub id: ElementId,
    pub kind: FormFieldKind,
    /// Data aggregation identifier tag / key (集計用データタグ).
    pub field_tag: String,
    pub x: Mm,
    pub y: Mm,
    pub width: Mm,
    pub height: Mm,
    pub border_color: Color,
    pub border_width: Mm,
    pub bg_color: Color,
    pub font_size_pt: f64,
    pub text_color: Color,
}

/// Bounding rectangle in integer millimeters (Bottom-Left origin).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RectMm {
    pub x: Mm,
    pub y: Mm,
    pub width: Mm,
    pub height: Mm,
}

/// Wrapper enum for all supported layout elements (Line, TextBox, FormField).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Element {
    Line(LineElement),
    TextBox(TextBoxElement),
    FormField(FormFieldElement),
}

impl Element {
    /// Get the unique ID of the element.
    pub fn id(&self) -> ElementId {
        match self {
            Self::Line(e) => e.id,
            Self::TextBox(e) => e.id,
            Self::FormField(e) => e.id,
        }
    }

    /// Calculate the 1mm integer bounding box of the element.
    pub fn bounds(&self) -> RectMm {
        match self {
            Self::Line(e) => {
                let min_x = Mm::new(e.x1.0.min(e.x2.0));
                let min_y = Mm::new(e.y1.0.min(e.y2.0));
                let max_x = Mm::new(e.x1.0.max(e.x2.0));
                let max_y = Mm::new(e.y1.0.max(e.y2.0));
                let w = Mm::new((max_x.0 - min_x.0).max(1));
                let h = Mm::new((max_y.0 - min_y.0).max(1));
                RectMm { x: min_x, y: min_y, width: w, height: h }
            }
            Self::TextBox(e) => RectMm {
                x: e.x,
                y: e.y,
                width: e.width,
                height: e.height,
            },
            Self::FormField(e) => RectMm {
                x: e.x,
                y: e.y,
                width: e.width,
                height: e.height,
            },
        }
    }

    /// Translate the element's position by (dx, dy) in millimeters.
    pub fn translate(&mut self, dx: i32, dy: i32) {
        let apply_shift = |val: Mm, shift: i32| -> Mm {
            let new_val = (val.0 as i64) + (shift as i64);
            Mm::new(new_val.max(0) as u32)
        };

        match self {
            Self::Line(e) => {
                e.x1 = apply_shift(e.x1, dx);
                e.y1 = apply_shift(e.y1, dy);
                e.x2 = apply_shift(e.x2, dx);
                e.y2 = apply_shift(e.y2, dy);
            }
            Self::TextBox(e) => {
                e.x = apply_shift(e.x, dx);
                e.y = apply_shift(e.y, dy);
            }
            Self::FormField(e) => {
                e.x = apply_shift(e.x, dx);
                e.y = apply_shift(e.y, dy);
            }
        }
    }
}
