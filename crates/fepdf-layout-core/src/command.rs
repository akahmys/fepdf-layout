//! Undoable layout commands.

use crate::frame::{Frame, Rect};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayoutCommand {
    AddFrame(Frame),
    MoveFrame { id: u64, new_bounds: Rect },
    DeleteFrame(u64),
}
