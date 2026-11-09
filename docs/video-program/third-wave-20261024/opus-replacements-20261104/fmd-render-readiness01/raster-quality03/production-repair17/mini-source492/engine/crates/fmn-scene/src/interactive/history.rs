//! Bounded native editor history over Marionette's in-memory CoW snapshots.
//!
//! Callables stay shared by identity in those snapshots. This is editing
//! history, not replay of the Scene clock, I/O or mutable closure captures.

use std::collections::VecDeque;
use std::rc::Rc;

use fmn_mobject::Stage;

use super::{InteractionState, InteractiveScene, UndoState};

/// Default number of retained native editing transitions.
pub const DEFAULT_HISTORY_LIMIT: usize = 50;

pub(super) struct EditHistory {
    past: VecDeque<UndoState>,
    future: VecDeque<UndoState>,
    limit: usize,
}

impl Default for EditHistory {
    fn default() -> Self {
        Self {
            past: VecDeque::new(),
            future: VecDeque::new(),
            limit: DEFAULT_HISTORY_LIMIT,
        }
    }
}

impl EditHistory {
    fn trim(&mut self) {
        // Keep the nearest undo/redo transitions. Past front is the oldest
        // undo state; future front is the most distant redo state.
        while self.past.len().saturating_add(self.future.len()) > self.limit {
            if self.past.len() >= self.future.len() {
                self.past.pop_front();
            } else {
                self.future.pop_front();
            }
        }
    }
}

impl InteractionState {
    fn capture_edit(&self, stage: &Stage) -> UndoState {
        UndoState {
            stage: Rc::new(stage.snapshot()),
            selection: self.selection.clone(),
            clipboard: self.clipboard.clone(),
        }
    }

    pub(super) fn save_undo(&mut self, stage: &Stage) {
        if self.history.limit == 0 {
            return;
        }
        let state = self.capture_edit(stage);
        self.history.future.clear();
        self.history.past.push_back(state);
        self.history.trim();
    }

    fn restore_edit(&mut self, stage: &mut Stage, state: UndoState) {
        stage.restore(&state.stage);
        self.selection = state.selection;
        self.clipboard = state.clipboard;
        // A historical geometry edit must not leave a live grab/resize
        // gesture capable of immediately overwriting the restored state.
        self.grab = None;
        self.resize = None;
        self.color_picking = false;
        self.selection_start = None;
        self.selection_sweeping = false;
        self.prune(stage);
    }

    pub(super) fn restore_undo(&mut self, stage: &mut Stage) -> bool {
        if self.history.past.is_empty() {
            return false;
        }
        let current = self.capture_edit(stage);
        let Some(previous) = self.history.past.pop_back() else {
            return false;
        };
        self.history.future.push_back(current);
        self.restore_edit(stage, previous);
        true
    }

    pub(super) fn restore_redo(&mut self, stage: &mut Stage) -> bool {
        if self.history.future.is_empty() {
            return false;
        }
        let current = self.capture_edit(stage);
        let Some(following) = self.history.future.pop_back() else {
            return false;
        };
        self.history.past.push_back(current);
        self.restore_edit(stage, following);
        true
    }
}

impl InteractiveScene {
    /// Number of retained undo and redo transitions, respectively.
    #[must_use]
    pub fn history_depths(&self) -> (usize, usize) {
        let state = self.state.borrow();
        (state.history.past.len(), state.history.future.len())
    }

    /// Bound the total retained editing transitions. Zero disables history
    /// and releases both branches. Reducing a limit retains nearby states;
    /// it never mutates the live Stage or executes an updater.
    pub fn set_history_limit(&mut self, limit: usize) -> &mut Self {
        {
            let mut state = self.state.borrow_mut();
            state.history.limit = limit;
            state.history.trim();
        }
        self
    }

    /// Discard undo/redo history without altering the live scene.
    pub fn clear_history(&mut self) {
        let mut state = self.state.borrow_mut();
        state.history.past.clear();
        state.history.future.clear();
    }

    /// Save the pre-edit state for a host-authored native Stage mutation.
    /// Built-in editing commands call the same operation themselves.
    /// Recording a new edit abandons the previous redo branch.
    pub fn save_undo_state(&mut self) {
        self.state.borrow_mut().save_undo(self.scene.stage());
    }

    /// Undo one native editing transition. Returns false at the oldest state.
    /// Scene time, event sequencing and external closure state are not rewound.
    pub fn undo(&mut self) -> bool {
        self.state.borrow_mut().restore_undo(self.scene.stage_mut())
    }

    /// Redo one native editing transition. Returns false at the newest state.
    pub fn redo(&mut self) -> bool {
        self.state.borrow_mut().restore_redo(self.scene.stage_mut())
    }
}
