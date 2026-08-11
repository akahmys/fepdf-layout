//! `fepdf-layout-core`: DTP Layout engine, 1mm grid snapping, document model, and command history.

pub mod align;
pub mod command;
pub mod document;
pub mod element;
pub mod export;
pub mod history;
pub mod page;
pub mod units;

pub use align::{align_elements, AlignMode};
pub use command::Command;
pub use document::{Document, DocumentManager};
pub use element::{
    Color, Element, ElementId, FormFieldElement, FormFieldKind, LineElement, RectMm, StrokeStyle,
    TextAlign, TextBoxElement,
};
pub use history::CommandHistory;
pub use page::{PagePreset, PageSpec};
pub use units::{Mm, MmF64, MM_PER_INCH, POINTS_PER_INCH};
