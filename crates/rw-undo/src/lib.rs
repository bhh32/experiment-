//! # rw-undo
//!
//! Undo/redo system for Rust Writer using the command pattern.
//!
//! Every user action that modifies the document is represented as a
//! `Command` that knows how to apply itself and reverse itself.
//! Commands are stored on an undo stack, allowing unlimited undo/redo.
//!
//! ## Architecture
//!
//! - `Command` trait: defines `execute`, `undo`, and `description`
//! - `UndoManager`: manages the undo/redo stacks
//! - `CompoundCommand`: groups multiple commands into a single undoable unit
//! - Commands are generic over the document type to avoid coupling

use std::fmt;

/// A command that can be executed and undone.
///
/// Commands encapsulate a reversible document modification.
/// The type parameter `D` is the document/state type.
pub trait Command<D>: fmt::Debug + Send {
    /// Execute this command, modifying the document.
    fn execute(&mut self, doc: &mut D);

    /// Undo this command, restoring the document to its prior state.
    fn undo(&mut self, doc: &mut D);

    /// A human-readable description of this command (for UI display).
    fn description(&self) -> &str;

    /// Whether this command can be merged with the previous command.
    /// Used for coalescing rapid keystrokes into a single undo unit.
    fn can_merge(&self, _other: &dyn Command<D>) -> bool {
        false
    }

    /// Merge another command into this one (if `can_merge` returned true).
    fn merge(&mut self, _other: Box<dyn Command<D>>) {}
}

/// Manages undo and redo stacks.
pub struct UndoManager<D> {
    /// Commands that can be undone
    undo_stack: Vec<Box<dyn Command<D>>>,
    /// Commands that can be redone
    redo_stack: Vec<Box<dyn Command<D>>>,
    /// Maximum number of undo levels (0 = unlimited)
    max_levels: usize,
    /// Whether a compound command is being recorded
    recording: bool,
    /// Commands accumulated during compound recording
    recording_buffer: Vec<Box<dyn Command<D>>>,
    /// Flag indicating unsaved changes exist
    dirty: bool,
    /// The undo level at which the document was last saved
    save_point: Option<usize>,
}

impl<D: fmt::Debug + 'static> UndoManager<D> {
    /// Create a new UndoManager with unlimited undo levels.
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_levels: 0,
            recording: false,
            recording_buffer: Vec::new(),
            dirty: false,
            save_point: Some(0),
        }
    }

    /// Create a new UndoManager with a maximum number of undo levels.
    pub fn with_max_levels(max: usize) -> Self {
        let mut mgr = Self::new();
        mgr.max_levels = max;
        mgr
    }

    /// Execute a command and push it onto the undo stack.
    pub fn execute(&mut self, mut cmd: Box<dyn Command<D>>, doc: &mut D) {
        cmd.execute(doc);

        if self.recording {
            self.recording_buffer.push(cmd);
        } else {
            // Try to merge with the previous command
            if let Some(prev) = self.undo_stack.last_mut() {
                if prev.can_merge(cmd.as_ref()) {
                    prev.merge(cmd);
                    self.redo_stack.clear();
                    self.dirty = true;
                    return;
                }
            }

            self.undo_stack.push(cmd);
            self.redo_stack.clear();
            self.dirty = true;

            // Enforce max levels
            if self.max_levels > 0 && self.undo_stack.len() > self.max_levels {
                self.undo_stack.remove(0);
                // Invalidate save point if it was in the removed range
                if let Some(sp) = self.save_point {
                    if sp == 0 {
                        self.save_point = None;
                    } else {
                        self.save_point = Some(sp - 1);
                    }
                }
            }
        }
    }

    /// Undo the most recent command.
    pub fn undo(&mut self, doc: &mut D) -> bool {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(doc);
            self.redo_stack.push(cmd);
            self.dirty = self.save_point != Some(self.undo_stack.len());
            true
        } else {
            false
        }
    }

    /// Redo the most recently undone command.
    pub fn redo(&mut self, doc: &mut D) -> bool {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(doc);
            self.undo_stack.push(cmd);
            self.dirty = self.save_point != Some(self.undo_stack.len());
            true
        } else {
            false
        }
    }

    /// Whether there are commands to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Whether there are commands to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the description of the next undo command.
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().map(|c| c.description())
    }

    /// Get the description of the next redo command.
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|c| c.description())
    }

    /// Begin recording a compound command.
    /// All commands executed until `end_compound` are grouped.
    pub fn begin_compound(&mut self) {
        self.recording = true;
        self.recording_buffer.clear();
    }

    /// End recording and push the compound command.
    pub fn end_compound(&mut self, description: impl Into<String>) {
        self.recording = false;
        if !self.recording_buffer.is_empty() {
            let compound = CompoundCommand {
                commands: std::mem::take(&mut self.recording_buffer),
                desc: description.into(),
            };
            self.undo_stack.push(Box::new(compound));
            self.redo_stack.clear();
            self.dirty = true;
        }
    }

    /// Mark the current state as the save point.
    pub fn mark_saved(&mut self) {
        self.save_point = Some(self.undo_stack.len());
        self.dirty = false;
    }

    /// Whether the document has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear all undo/redo history.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.save_point = Some(0);
        self.dirty = false;
    }
}

impl<D: fmt::Debug + 'static> Default for UndoManager<D> {
    fn default() -> Self {
        Self::new()
    }
}

/// A compound command that groups multiple commands into one undo unit.
#[derive(Debug)]
struct CompoundCommand<D> {
    commands: Vec<Box<dyn Command<D>>>,
    desc: String,
}

impl<D: fmt::Debug + 'static> Command<D> for CompoundCommand<D> {
    fn execute(&mut self, doc: &mut D) {
        for cmd in &mut self.commands {
            cmd.execute(doc);
        }
    }

    fn undo(&mut self, doc: &mut D) {
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(doc);
        }
    }

    fn description(&self) -> &str {
        &self.desc
    }
}
