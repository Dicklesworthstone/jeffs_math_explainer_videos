//! The opt-in acceleration ladder for Python updaters (§15.2 Rev 4, fm-zoi).
//!
//! Rung 0 — the always-correct default — is an ordinary Python callable in
//! `mobject.updaters`, dispatched by `Scene.update` (or, batched at the
//! dispatch level, `Scene.update_batched`). The three rungs declared here
//! are *explicit substitutions* a user constructs and attaches by hand.
//! Silent substitution is prohibited: nothing in the bridge rewrites a
//! rung-0 updater into a higher rung, and every rung object is a distinct,
//! inspectable Python type (D-05's spirit: labeled, never silent).
//!
//! | Rung | Type | Declared class | Crossings per frame |
//! |---|---|---|---|
//! | 1 | [`PyBatchedUpdater`] | one Python callback over a fixed mobject group, views only | 1 `updater_call` + 1 `dirty_propagation` per group |
//! | 2 | [`PyArrayUpdater`] | one [`DeclaredOp`] over one field | 0 inside the rung |
//! | 3 | [`PyNativeUpdater`] | one [`DeclaredOp`] over one field | 0 (engine-executable) |
//!
//! # Declared semantics
//!
//! Rungs 2 and 3 share `fmn_anim::ladder`'s declared operations (position
//! shifters, scale pulsers, color ramps). The arithmetic contract — binary64
//! over widened `f32` lanes, one round-to-nearest-even store, pure
//! arithmetic oscillators — is documented there; on that declared class
//! every rung produces bit-identical RecordBuffer state per frame, which
//! the ladder corpus (`tests/ladder.py`) proves against rung 0.
//!
//! Rung 1's declared class is wider: an arbitrary Python callback receiving
//! the group's writable structured NumPy views plus `dt`, once per frame.
//! The callback must mutate the views in place; per-field `set_field`
//! crossings are replaced by zero-copy writes, and the frame's dirty state
//! transfers in one batched `dirty_propagation` crossing when the callback
//! returns (conservative whole-field spans, exactly as the view protocol
//! forces for writable-view-affected fields). Bit-equality against rung 0
//! holds when the callback performs the same binary64 arithmetic
//! (documented in the corpus).
//!
//! # Instrumentation ownership
//!
//! The ladder records only the crossings it alone can see: the group
//! callback invocation (one `updater_call`) and the batched dirty transfer
//! (one `dirty_propagation`). Dispatch-side crossings are recorded by the
//! `Scene.update`/`update_batched` loops, and view export is native-side
//! work (no boundary transition). The ladder never records phase timings:
//! the frame-level phase seam in `Scene.update` already attributes the
//! callback wall time, and a second sub-frame record would double-count.

use std::cell::RefCell;
use std::rc::Rc;

use fmn_anim::{DeclaredOp, DeclaredUpdater, LadderError};
use fmn_mobject::{Mob, Stage, UpdaterId};
use pyo3::exceptions::{PyKeyError, PyRuntimeError, PyTypeError, PyValueError};
use pyo3::gc::{PyTraverseError, PyVisit};
use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::crossing::{self, CrossingClass};
use crate::{
    BridgeMobject, Engine, bound_parts, numpy_array, stage_error, with_buffer, with_buffer_ref,
};

fn ladder_error(error: LadderError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

/// Validate the declared operation against the target's live schema.
fn validate_schema(proxy: &Bound<'_, BridgeMobject>, op: &DeclaredOp) -> PyResult<()> {
    let width = with_buffer_ref(proxy, |buffer| buffer.schema().field_width(op.field()))?;
    match width {
        None => Err(PyKeyError::new_err(format!(
            "declared {} updater names field `{}`, which the mobject schema does not have",
            op.class_tag(),
            op.field()
        ))),
        Some(width) if width != op.width() => Err(PyValueError::new_err(format!(
            "declared {} updater field `{}` has {width} lanes, but the operation declares {}",
            op.class_tag(),
            op.field(),
            op.width()
        ))),
        Some(_) => Ok(()),
    }
}

fn declared_updater<'py>(
    mobject: &Bound<'py, PyAny>,
    op: DeclaredOp,
    rung: &str,
) -> PyResult<(DeclaredUpdater, Bound<'py, BridgeMobject>)> {
    let proxy = mobject
        .cast::<BridgeMobject>()
        .map_err(|_| PyTypeError::new_err(format!("{rung} target must be a Mobject instance")))?;
    let updater = DeclaredUpdater::new(op).map_err(ladder_error)?;
    validate_schema(proxy, updater.op())?;
    Ok((updater, proxy.clone()))
}

/// Rung 1: one Python callback over a fixed mobject group, once per frame.
///
/// Construct with the group members and a callback
/// `f(views, dt) -> None`; attach to any mobject's `updaters` list (the
/// anchor) with `add_updater(group, call=False)`. Each frame the anchor's
/// dispatch exports one writable structured NumPy view per member (the
/// bridge's zero-copy `RecordView` machinery), invokes the callback once,
/// and transfers the group's dirty state in one crossing on return.
///
/// The declared class fixes the group at construction: members may not be
/// added, removed, or resized between frames (a resize swaps storage, and
/// a view exported before it observes the old generation — the ordinary
/// view protocol). The callback receives views in member order and must
/// mutate them in place.
#[pyclass(unsendable, name = "_BatchedUpdater")]
pub struct PyBatchedUpdater {
    members: Vec<Py<PyAny>>,
    callback: Option<Py<PyAny>>,
}

#[pymethods]
impl PyBatchedUpdater {
    #[new]
    fn new(members: Vec<Bound<'_, PyAny>>, callback: Py<PyAny>) -> PyResult<Self> {
        if members.is_empty() {
            return Err(PyValueError::new_err(
                "_BatchedUpdater requires at least one group member",
            ));
        }
        for member in &members {
            member.cast::<BridgeMobject>().map_err(|_| {
                PyTypeError::new_err("_BatchedUpdater members must be Mobject instances")
            })?;
        }
        let py = members[0].py();
        if !callback.bind(py).is_callable() {
            return Err(PyTypeError::new_err(
                "_BatchedUpdater callback must be callable",
            ));
        }
        Ok(Self {
            members: members.into_iter().map(pyo3::Bound::unbind).collect(),
            callback: Some(callback),
        })
    }

    // The anchor can also be a member or be captured by the callback.
    // Expose both native-owned edges so ordinary Python cycle collection
    // can reclaim an unreachable updater and its scene on the owner thread.
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        for member in &self.members {
            visit.call(member)?;
        }
        visit.call(&self.callback)
    }

    fn __clear__(&mut self) {
        self.members.clear();
        self.callback = None;
    }

    /// Number of group members fixed at construction.
    fn group_size(&self) -> usize {
        self.members.len()
    }

    /// Updater protocol entry point. The anchor argument is accepted for
    /// `Mobject._dispatch_updater` compatibility; the group operates on its
    /// construction-time member list, so one `BatchedUpdater` must anchor
    /// at exactly one mobject (attach with `call=False`).
    #[pyo3(signature = (_anchor, dt))]
    fn __call__(&self, py: Python<'_>, _anchor: &Bound<'_, PyAny>, dt: f64) -> PyResult<()> {
        let callback = self.callback.as_ref().ok_or_else(|| {
            PyRuntimeError::new_err("_BatchedUpdater has been cleared by garbage collection")
        })?;
        let views = PyList::empty(py);
        for member in &self.members {
            let proxy = member.bind(py).cast::<BridgeMobject>().map_err(|_| {
                PyTypeError::new_err("_BatchedUpdater members must be Mobject instances")
            })?;
            views.append(numpy_array(py, proxy, true)?)?;
        }
        crossing::record(CrossingClass::UpdaterCall);
        callback.call1(py, (views, dt))?;
        // The callback's view writes accumulate in RecordBuffer state with
        // no per-field crossings; the group's dirty transfer is this one
        // return crossing (conservative whole-field spans via the view
        // protocol's writable-view rule).
        crossing::record(CrossingClass::DirtyPropagation);
        Ok(())
    }
}

/// Rung 2: one vectorized RecordBuffer operation per frame.
///
/// Construct through a classmethod naming the declared operation —
/// `_ArrayUpdater.shift(mobject, "point", velocity)`,
/// `_ArrayUpdater.scale_pulse(mobject, "point", center, amplitude, period)`,
/// `_ArrayUpdater.color_ramp(mobject, "rgba", from_rgba, to_rgba, period)` —
/// and attach with `add_updater(updater, call=False)`. The bridge executes
/// the declared operation natively as a single `write_range` over the
/// whole field column; no Python code runs inside the rung.
///
/// Following the Reference's updater rule, the operation applies to the
/// mobject it is dispatched on, so engine/Python copies sharing the updater
/// transform their own buffers. A copy whose schema no longer matches the
/// declaration raises `RuntimeError` at dispatch (schema drift is loud,
/// never silently skipped).
#[pyclass(unsendable, name = "_ArrayUpdater")]
pub struct PyArrayUpdater {
    updater: DeclaredUpdater,
}

#[pymethods]
impl PyArrayUpdater {
    #[staticmethod]
    fn shift(mobject: &Bound<'_, PyAny>, field: &str, velocity: Vec<f64>) -> PyResult<Self> {
        let (updater, _) = declared_updater(
            mobject,
            DeclaredOp::Shift {
                field: field.to_owned(),
                velocity,
            },
            "_ArrayUpdater",
        )?;
        Ok(Self { updater })
    }

    #[staticmethod]
    fn scale_pulse(
        mobject: &Bound<'_, PyAny>,
        field: &str,
        center: Vec<f64>,
        amplitude: f64,
        period: f64,
    ) -> PyResult<Self> {
        let (updater, _) = declared_updater(
            mobject,
            DeclaredOp::ScalePulse {
                field: field.to_owned(),
                center,
                amplitude,
                period,
            },
            "_ArrayUpdater",
        )?;
        Ok(Self { updater })
    }

    #[staticmethod]
    fn color_ramp(
        mobject: &Bound<'_, PyAny>,
        field: &str,
        from: Vec<f64>,
        to: Vec<f64>,
        period: f64,
    ) -> PyResult<Self> {
        let (updater, _) = declared_updater(
            mobject,
            DeclaredOp::ColorRamp {
                field: field.to_owned(),
                from,
                to,
                period,
            },
            "_ArrayUpdater",
        )?;
        Ok(Self { updater })
    }

    /// The declared operation's class tag (`shift`, `scale-pulse`,
    /// `color-ramp`).
    fn class_tag(&self) -> &'static str {
        self.updater.op().class_tag()
    }

    /// The rung's private time base; advances by `dt` after each frame.
    fn time(&self) -> f64 {
        self.updater.time()
    }

    #[pyo3(signature = (mobject, dt))]
    fn __call__(&mut self, mobject: &Bound<'_, PyAny>, dt: f64) -> PyResult<()> {
        let proxy = mobject
            .cast::<BridgeMobject>()
            .map_err(|_| PyTypeError::new_err("ArrayUpdater dispatch target is not a Mobject"))?;
        let applied = with_buffer(proxy, |buffer| self.updater.apply(buffer, dt))?;
        if !applied {
            return Err(PyRuntimeError::new_err(format!(
                "declared {} updater no longer matches the mobject schema (field `{}`)",
                self.updater.op().class_tag(),
                self.updater.op().field()
            )));
        }
        Ok(())
    }
}

/// Rung 3: an engine-executable native updater.
///
/// Same declared operations as rung 2, registered into the Stage's native
/// updater list at construction: the updater runs inside the engine's
/// update pass with zero boundary crossings per frame and persists through
/// engine-side family copies like any native updater slot.
///
/// Construction requires a bound mobject (one already added to a Scene);
/// the returned handle is the removal capability — `detach()` unregisters
/// by updater id. Dropping the handle leaves the updater registered,
/// exactly as dropping a Reference updater function reference leaves it in
/// `mobject.updaters`.
///
/// Registration runs no immediate `dt = 0` pass (the native convention;
/// the Reference's `add_updater(call=True)` double-pass is C-5-corrected
/// engine-side). Ladder corpus reference updaters attach with `call=False`
/// so every rung observes the same frame sequence.
#[pyclass(unsendable, name = "_NativeUpdater")]
pub struct PyNativeUpdater {
    engine: Engine,
    mob: Mob,
    id: UpdaterId,
    updater: Rc<RefCell<DeclaredUpdater>>,
    attached: bool,
}

impl PyNativeUpdater {
    fn build(mobject: &Bound<'_, PyAny>, op: DeclaredOp) -> PyResult<Self> {
        let (updater, proxy) = declared_updater(mobject, op, "_NativeUpdater")?;
        let (engine, mob) = bound_parts(&proxy.borrow())?;
        let shared = Rc::new(RefCell::new(updater));
        let inner = Rc::clone(&shared);
        let id = engine
            .borrow_mut()
            .stage_mut()
            .add_dt_updater(
                mob,
                move |stage: &mut Stage, mob: Mob, dt: f64| {
                    if let Some(entry) = stage.get_mut(mob) {
                        inner.borrow_mut().apply(&mut entry.buffer, dt);
                    }
                },
                false,
            )
            .map_err(stage_error)?;
        Ok(Self {
            engine,
            mob,
            id,
            updater: shared,
            attached: true,
        })
    }
}

#[pymethods]
impl PyNativeUpdater {
    #[staticmethod]
    fn shift(mobject: &Bound<'_, PyAny>, field: &str, velocity: Vec<f64>) -> PyResult<Self> {
        Self::build(
            mobject,
            DeclaredOp::Shift {
                field: field.to_owned(),
                velocity,
            },
        )
    }

    #[staticmethod]
    fn scale_pulse(
        mobject: &Bound<'_, PyAny>,
        field: &str,
        center: Vec<f64>,
        amplitude: f64,
        period: f64,
    ) -> PyResult<Self> {
        Self::build(
            mobject,
            DeclaredOp::ScalePulse {
                field: field.to_owned(),
                center,
                amplitude,
                period,
            },
        )
    }

    #[staticmethod]
    fn color_ramp(
        mobject: &Bound<'_, PyAny>,
        field: &str,
        from: Vec<f64>,
        to: Vec<f64>,
        period: f64,
    ) -> PyResult<Self> {
        Self::build(
            mobject,
            DeclaredOp::ColorRamp {
                field: field.to_owned(),
                from,
                to,
                period,
            },
        )
    }

    /// The engine updater id (the durable removal token).
    fn updater_id(&self) -> u64 {
        self.id.raw()
    }

    /// The declared operation's class tag.
    fn class_tag(&self) -> &'static str {
        self.updater.borrow().op().class_tag()
    }

    /// The rung's private time base; advances by `dt` after each frame.
    fn time(&self) -> f64 {
        self.updater.borrow().time()
    }

    /// Whether the native updater is still registered.
    fn is_attached(&self) -> bool {
        self.attached
    }

    /// Unregister the native updater. Returns `True` on the first detach,
    /// `False` if already detached (idempotent, loud by inspection rather
    /// than by exception).
    fn detach(&mut self) -> bool {
        if !self.attached {
            return false;
        }
        self.engine
            .borrow_mut()
            .stage_mut()
            .remove_updater(self.mob, self.id);
        self.attached = false;
        true
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    /// Run the ladder acceptance corpus (`tests/ladder.py`) against the
    /// production module: per-rung bit-equality, crossing reduction, and
    /// the explicit opt-in surface.
    #[test]
    fn ladder_acceptance_suite() {
        crate::with_python_test_module("acceleration ladder acceptance", |py, _module, globals| {
            let source = CString::new(include_str!("../tests/ladder.py"))
                .expect("test source contains no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .expect("ladder acceptance suite");
        });
    }
}
