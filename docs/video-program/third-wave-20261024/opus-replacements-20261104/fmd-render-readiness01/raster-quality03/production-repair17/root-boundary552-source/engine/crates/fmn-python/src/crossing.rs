//! The fm-zoi instrumentation seam: every control-flow crossing of the
//! native↔Python boundary routes through these counters (§15.2 Rev 4).
//!
//! A *crossing* is one transition of control across the language boundary.
//! The counters classify crossings by the operation that caused them:
//!
//! - [`CrossingClass::UpdaterCall`] — native→Python: one user updater
//!   callback invocation (rung 0 dispatches one crossing per updater).
//! - [`CrossingClass::MethodDispatch`] — native→Python: engine-driven method
//!   invocations (init hooks, lifecycle, `_dispatch_updater` in the batch
//!   path, transform `interpolate`/`__copy__`).
//! - [`CrossingClass::FieldWrite`] — Python→native: record-field mutation
//!   (`set_field`, `resize`, uniform writes). These are identical across
//!   ladder rungs; arbitrary Python updaters write through the same API.
//! - [`CrossingClass::DirtyPropagation`] — the batched dirty-consolidation
//!   crossing: rung 1 transfers the whole frame's accumulated dirty state to
//!   native in one return crossing per batch instead of flowing it per
//!   callback. (Per-write dirty state inside a callback rides the
//!   `field_write` crossing in every rung — RecordBuffer revisions are
//!   native-side, so no separate Python crossing exists to batch.)
//! - [`CrossingClass::Other`] — every remaining boundary crossing (attribute
//!   snapshots like `updaters`, reads, membership operations).
//!
//! [`CallbackPhaseBreakdown`] is measured at the phase seam in
//! `Scene.update`/`update_batched`: wall time spent inside Python callbacks
//! vs the native advance. Timing is instrumentation only — tests assert
//! counters, never wall-clock values.
//!
//! Storage is a thread-local of `Cell`s: recording is one TLS load and one
//! integer increment, non-allocating on the hot path. The bridge's proxies
//! are confined to their creating worker thread (ADR-0015), so per-thread
//! counters exactly partition crossings by scene worker.

use std::cell::Cell;

use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Crossing classification. Order is fixed: it indexes [`CrossingStats`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossingClass {
    UpdaterCall,
    MethodDispatch,
    FieldWrite,
    DirtyPropagation,
    Other,
}

impl CrossingClass {
    /// Fixed class order, also the index basis of [`CrossingStats::counts`].
    pub const ALL: [CrossingClass; 5] = [
        CrossingClass::UpdaterCall,
        CrossingClass::MethodDispatch,
        CrossingClass::FieldWrite,
        CrossingClass::DirtyPropagation,
        CrossingClass::Other,
    ];

    /// Stable snake_case name used by the Python detection-report getters.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            CrossingClass::UpdaterCall => "updater_call",
            CrossingClass::MethodDispatch => "method_dispatch",
            CrossingClass::FieldWrite => "field_write",
            CrossingClass::DirtyPropagation => "dirty_propagation",
            CrossingClass::Other => "other",
        }
    }

    #[must_use]
    fn index(self) -> usize {
        self as usize
    }
}

/// Wall-time split at the callback phase seam, in nanoseconds.
///
/// `python_callback_ns` accumulates time spent executing Python code under
/// engine control (updaters, lifecycle); `native_ns` accumulates the native
/// advance that follows in the same frame step.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CallbackPhaseBreakdown {
    pub python_callback_ns: u64,
    pub native_ns: u64,
}

/// Point-in-time copy of the crossing counters. `Copy` so the thread-local
/// can be a plain `Cell` and snapshots never allocate.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CrossingStats {
    /// Per-class counts, indexed by `CrossingClass as usize`.
    pub counts: [u64; 5],
    /// Phase-seam wall-time split.
    pub phase: CallbackPhaseBreakdown,
}

impl CrossingStats {
    /// Count for one crossing class.
    #[must_use]
    pub fn count(&self, class: CrossingClass) -> u64 {
        self.counts[class.index()]
    }

    /// Sum over all crossing classes.
    #[must_use]
    pub fn total(&self) -> u64 {
        self.counts.iter().sum()
    }

    /// Python-callback phase nanoseconds (convenience getter).
    #[must_use]
    pub fn python_callback_ns(&self) -> u64 {
        self.phase.python_callback_ns
    }

    /// Native phase nanoseconds (convenience getter).
    #[must_use]
    pub fn native_ns(&self) -> u64 {
        self.phase.native_ns
    }
}

thread_local! {
    static STATS: Cell<CrossingStats> = const { Cell::new(CrossingStats {
        counts: [0; 5],
        phase: CallbackPhaseBreakdown {
            python_callback_ns: 0,
            native_ns: 0,
        },
    }) };
}

/// Record one boundary crossing. Hot path: one TLS load, one increment.
#[inline]
pub fn record(class: CrossingClass) {
    STATS.with(|stats| {
        let mut snapshot = stats.get();
        snapshot.counts[class.index()] += 1;
        stats.set(snapshot);
    });
}

/// Accumulate one phase-seam measurement: `python_callback_ns` spent inside
/// Python callbacks, `native_ns` spent in the native advance.
#[inline]
pub fn record_phase(python_callback_ns: u64, native_ns: u64) {
    STATS.with(|stats| {
        let mut snapshot = stats.get();
        snapshot.phase.python_callback_ns += python_callback_ns;
        snapshot.phase.native_ns += native_ns;
        stats.set(snapshot);
    });
}

/// Point-in-time copy of this thread's counters (LadderWorker's detection
/// report reads this).
#[must_use]
pub fn snapshot() -> CrossingStats {
    STATS.with(Cell::get)
}

/// Zero this thread's counters. Tests and per-scene measurement windows
/// reset before the measured section.
pub fn reset() {
    STATS.with(|stats| stats.set(CrossingStats::default()));
}

/// `manimlib._crossing_stats_snapshot()`: the detection-report getter.
///
/// Returns a plain dict with one integer key per crossing class plus the two
/// phase keys. Keys are stable; consumers must not require dict order.
#[pyfunction]
pub(crate) fn _crossing_stats_snapshot(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
    let stats = snapshot();
    let out = PyDict::new(py);
    for class in CrossingClass::ALL {
        out.set_item(class.as_str(), stats.count(class))?;
    }
    out.set_item("total", stats.total())?;
    out.set_item("python_callback_ns", stats.python_callback_ns())?;
    out.set_item("native_ns", stats.native_ns())?;
    Ok(out)
}

/// `manimlib._crossing_stats_reset()`: zero this thread's counters.
#[pyfunction]
pub(crate) fn _crossing_stats_reset() {
    reset();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_record_snapshot_reset() {
        reset();
        record(CrossingClass::UpdaterCall);
        record(CrossingClass::UpdaterCall);
        record(CrossingClass::FieldWrite);
        record_phase(10, 5);
        record_phase(1, 2);
        let stats = snapshot();
        assert_eq!(stats.count(CrossingClass::UpdaterCall), 2);
        assert_eq!(stats.count(CrossingClass::FieldWrite), 1);
        assert_eq!(stats.count(CrossingClass::MethodDispatch), 0);
        assert_eq!(stats.total(), 3);
        assert_eq!(stats.python_callback_ns(), 11);
        assert_eq!(stats.native_ns(), 7);
        reset();
        assert_eq!(snapshot().total(), 0);
        assert_eq!(snapshot().phase, CallbackPhaseBreakdown::default());
    }

    #[test]
    fn class_names_are_stable() {
        let names: Vec<&str> = CrossingClass::ALL
            .iter()
            .map(|class| class.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "updater_call",
                "method_dispatch",
                "field_write",
                "dirty_propagation",
                "other"
            ]
        );
    }
}
