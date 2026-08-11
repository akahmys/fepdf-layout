//! Command history manager for Undo and Redo operations.

use crate::command::Command;

/// Undo/Redo command stack history manager.
#[derive(Debug, Default)]
pub struct CommandHistory {
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
}

impl CommandHistory {
    /// Create a new command history manager.
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Push a executed command onto the undo stack and clear the redo stack.
    pub fn push(&mut self, command: Command) {
        self.undo_stack.push(command);
        self.redo_stack.clear();
    }

    /// Pop a command from the undo stack to undo.
    pub fn undo(&mut self) -> Option<Command> {
        if let Some(cmd) = self.undo_stack.pop() {
            self.redo_stack.push(cmd.clone());
            Some(cmd)
        } else {
            None
        }
    }

    /// Pop a command from the redo stack to redo.
    pub fn redo(&mut self) -> Option<Command> {
        if let Some(cmd) = self.redo_stack.pop() {
            self.undo_stack.push(cmd.clone());
            Some(cmd)
        } else {
            None
        }
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
