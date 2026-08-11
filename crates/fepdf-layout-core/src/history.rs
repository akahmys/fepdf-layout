//! History manager for Undo/Redo operations.

use crate::command::LayoutCommand;

#[derive(Debug, Default)]
pub struct HistoryStack {
    undo_stack: Vec<LayoutCommand>,
    redo_stack: Vec<LayoutCommand>,
}

impl HistoryStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, command: LayoutCommand) {
        self.undo_stack.push(command);
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
