/// Undo/redo history for the editor.
/// Stores snapshots of (content, cursor_position) at each edit point.

const MAX_HISTORY: usize = 500;

#[derive(Clone)]
struct Snapshot {
    content: String,
    cursor: usize,
}

pub struct History {
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    /// Debounce: don't save a snapshot if the content hasn't changed
    last_saved: String,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_saved: String::new(),
        }
    }

    /// Push a snapshot onto the undo stack. Call this BEFORE making a change.
    pub fn push(&mut self, content: &str, cursor: usize) {
        // Don't push duplicate states
        if content == self.last_saved {
            return;
        }
        self.last_saved = content.to_string();
        self.undo_stack.push(Snapshot {
            content: content.to_string(),
            cursor,
        });
        // Any new edit clears the redo stack
        self.redo_stack.clear();
        // Limit history size
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
    }

    /// Undo: pop from undo stack, push current state onto redo stack.
    /// Returns the previous (content, cursor) or None if nothing to undo.
    pub fn undo(&mut self, current_content: &str, current_cursor: usize) -> Option<(String, usize)> {
        let prev = self.undo_stack.pop()?;
        self.redo_stack.push(Snapshot {
            content: current_content.to_string(),
            cursor: current_cursor,
        });
        self.last_saved = prev.content.clone();
        Some((prev.content, prev.cursor))
    }

    /// Redo: pop from redo stack, push current state onto undo stack.
    pub fn redo(&mut self, current_content: &str, current_cursor: usize) -> Option<(String, usize)> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(Snapshot {
            content: current_content.to_string(),
            cursor: current_cursor,
        });
        self.last_saved = next.content.clone();
        Some((next.content, next.cursor))
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
