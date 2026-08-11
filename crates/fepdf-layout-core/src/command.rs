//! Command pattern for reversible document mutations and Undo/Redo operations.

use crate::element::{Element, ElementId};
use crate::page::PagePreset;
use serde::{Deserialize, Serialize};

/// Reversible document modification command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// Add a new element to the document.
    AddElement(Element),
    /// Remove an element by ID.
    RemoveElement(Element),
    /// Move an element by offset (dx, dy) in millimeters.
    MoveElement { id: ElementId, dx: i32, dy: i32 },
    /// Update an existing element.
    UpdateElement { old: Element, new: Element },
    /// Align multiple elements.
    AlignElements { old_elements: Vec<Element>, new_elements: Vec<Element> },
    /// Change the page preset.
    SetPagePreset { old: PagePreset, new: PagePreset },
}
