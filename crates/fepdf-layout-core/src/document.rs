//! Single-page document model and command executor.

use crate::command::Command;
use crate::element::{Element, ElementId};
use crate::history::CommandHistory;
use crate::page::{PagePreset, PageSpec};
use serde::{Deserialize, Serialize};

/// Single-page business form document model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Active page specification.
    pub page_spec: PageSpec,
    /// List of layout elements on the page.
    pub elements: Vec<Element>,
    /// Next ID counter for new elements.
    next_element_id: u64,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            page_spec: PageSpec::default(),
            elements: Vec::new(),
            next_element_id: 1,
        }
    }

}

impl Document {
    /// Create a new empty single-page document with the given preset.
    pub fn new(preset: PagePreset) -> Self {
        Self {
            page_spec: PageSpec::from_preset(preset),
            elements: Vec::new(),
            next_element_id: 1,
        }
    }

    /// Generate a unique `ElementId`.
    pub fn next_id(&mut self) -> ElementId {
        let id = ElementId(self.next_element_id);
        self.next_element_id += 1;
        id
    }

    /// Get an element reference by ID.
    pub fn get_element(&self, id: ElementId) -> Option<&Element> {
        self.elements.iter().find(|e| e.id() == id)
    }

    /// Get a mutable element reference by ID.
    pub fn get_element_mut(&mut self, id: ElementId) -> Option<&mut Element> {
        self.elements.iter_mut().find(|e| e.id() == id)
    }

    /// Apply a command directly to the document model (internal execution).
    pub fn apply_command(&mut self, command: &Command) {
        match command {
            Command::AddElement(elem) => {
                self.elements.push(elem.clone());
            }
            Command::RemoveElement(elem) => {
                self.elements.retain(|e| e.id() != elem.id());
            }
            Command::MoveElement { id, dx, dy } => {
                if let Some(elem) = self.get_element_mut(*id) {
                    elem.translate(*dx, *dy);
                }
            }
            Command::UpdateElement { new, .. } => {
                if let Some(elem) = self.get_element_mut(new.id()) {
                    *elem = new.clone();
                }
            }
            Command::AlignElements { new_elements, .. } => {
                for new_elem in new_elements {
                    if let Some(elem) = self.get_element_mut(new_elem.id()) {
                        *elem = new_elem.clone();
                    }
                }
            }
            Command::SetPagePreset { new, .. } => {
                self.page_spec = PageSpec::from_preset(*new);
            }
        }
    }

    /// Revert an executed command (internal undo execution).
    pub fn revert_command(&mut self, command: &Command) {
        match command {
            Command::AddElement(elem) => {
                self.elements.retain(|e| e.id() != elem.id());
            }
            Command::RemoveElement(elem) => {
                self.elements.push(elem.clone());
            }
            Command::MoveElement { id, dx, dy } => {
                if let Some(elem) = self.get_element_mut(*id) {
                    elem.translate(-*dx, -*dy);
                }
            }
            Command::UpdateElement { old, .. } => {
                if let Some(elem) = self.get_element_mut(old.id()) {
                    *elem = old.clone();
                }
            }
            Command::AlignElements { old_elements, .. } => {
                for old_elem in old_elements {
                    if let Some(elem) = self.get_element_mut(old_elem.id()) {
                        *elem = old_elem.clone();
                    }
                }
            }
            Command::SetPagePreset { old, .. } => {
                self.page_spec = PageSpec::from_preset(*old);
            }
        }
    }

    /// Save document to a JSON file.
    pub fn save_json(&self, path: impl AsRef<std::path::Path>) -> Result<(), String> {
        let json_str = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json_str).map_err(|e| e.to_string())
    }

    /// Load document from a JSON file.
    pub fn load_json(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let json_str = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json_str).map_err(|e| e.to_string())
    }
}

/// Document Manager wrapping the `Document` and `CommandHistory` for full Undo/Redo support.
#[derive(Debug, Default)]
pub struct DocumentManager {
    pub doc: Document,
    pub history: CommandHistory,
}

impl DocumentManager {
    /// Create a new document manager with default A4 page spec.
    pub fn new(preset: PagePreset) -> Self {
        Self {
            doc: Document::new(preset),
            history: CommandHistory::new(),
        }
    }

    /// Execute a command on the document and record it in history.
    pub fn execute(&mut self, command: Command) {
        self.doc.apply_command(&command);
        self.history.push(command);
    }

    /// Undo the last command.
    pub fn undo(&mut self) -> bool {
        if let Some(cmd) = self.history.undo() {
            self.doc.revert_command(&cmd);
            true
        } else {
            false
        }
    }

    /// Redo the last undone command.
    pub fn redo(&mut self) -> bool {
        if let Some(cmd) = self.history.redo() {
            self.doc.apply_command(&cmd);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{Color, LineElement, StrokeStyle};
    use crate::units::Mm;

    #[test]
    fn test_document_command_undo_redo() {
        let mut mgr = DocumentManager::new(PagePreset::A4);
        let id = mgr.doc.next_id();

        let line = Element::Line(LineElement {
            id,
            x1: Mm::new(10),
            y1: Mm::new(10),
            x2: Mm::new(50),
            y2: Mm::new(10),
            stroke_width: Mm::new(1),
            stroke_color: Color::BLACK,
            stroke_style: StrokeStyle::Solid,
        });

        mgr.execute(Command::AddElement(line.clone()));
        assert_eq!(mgr.doc.elements.len(), 1);

        assert!(mgr.undo());
        assert_eq!(mgr.doc.elements.len(), 0);

        assert!(mgr.redo());
        assert_eq!(mgr.doc.elements.len(), 1);
    }

    #[test]
    fn test_document_json_serialization() {
        let mut doc = Document::new(PagePreset::A4);
        let id = doc.next_id();
        doc.elements.push(Element::Line(LineElement {
            id,
            x1: Mm::new(10),
            y1: Mm::new(20),
            x2: Mm::new(100),
            y2: Mm::new(20),
            stroke_width: Mm::new(2),
            stroke_color: Color::BLACK,
            stroke_style: StrokeStyle::Solid,
        }));

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_doc.fepdf-layout");

        assert!(doc.save_json(&temp_file).is_ok());
        let loaded = Document::load_json(&temp_file);
        assert!(loaded.is_ok());

        let loaded_doc = loaded.unwrap();
        assert_eq!(loaded_doc.page_spec, doc.page_spec);
        assert_eq!(loaded_doc.elements.len(), 1);

        let _ = std::fs::remove_file(&temp_file);
    }
}
