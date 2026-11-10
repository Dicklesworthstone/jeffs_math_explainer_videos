//! G0-5: the prototype PyO3 bridge (fm-87q, plan §20.1 spike 5 → §15.2).
//!
//! Proves, against the G0-1 object model, that fmn-python's compatibility
//! claim — *source* compatibility with normal Python object semantics — is
//! implementable: the ENGINE calls back into Python overrides at the right
//! lifecycle points via real MRO dispatch, `__dict__` attributes participate
//! in manim copy remapping, a subclass-declared `data_dtype` flows through
//! the RecordBuffer schema machinery, live views alias engine memory under
//! the §8.2 protocol, proxy identity/hashing/weakref survive engine
//! collection round-trips (the G0-1 pin story), and exceptions map cleanly
//! across the boundary in both directions.
//!
//! **The reentrancy law this spike establishes** (the question G0-1
//! deferred here): the engine must never hold state borrows across a
//! Python callback. Every callback window — lifecycle inits, interpolate,
//! updaters — dispatches with the engine lock RELEASED, so the callback
//! may re-enter any engine API freely. The bridge therefore keeps its own
//! updater registry and drives the update loop itself; production Choreo
//! owns the interleaving with engine-native updaters under the same law.
//!
//! The Python-side suite is `py/test_extensibility.py`; crossing costs are
//! measured by `py/bench_crossing.py` and recorded in the ratification note
//! (they seed PG-8's class-tiered binding-tax budgets). This crate is a
//! spike: throwaway by charter, kept compiling; fmn-python (W10, fm-aqv)
//! implements the production bridge from the ratification note, not from
//! these internals.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use pyo3::create_exception;
use pyo3::exceptions::{PyKeyError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};

use fmn_spike_object_model::record::FieldSpec;
use fmn_spike_object_model::{Mob, Mobject, RecordBuffer, RecordSchema, RecordView, Stage};

create_exception!(
    fmn_spike_bridge,
    StaleHandleError,
    PyRuntimeError,
    "The engine entry behind this proxy no longer exists (stale or deleted handle)."
);
create_exception!(
    fmn_spike_bridge,
    ForeignStageError,
    PyRuntimeError,
    "This proxy belongs to a different Stage (the two-scene policy: content crosses stages only by copy)."
);

type Engine = Rc<RefCell<Stage>>;

/// The engine seam. Unsendable: the scene worker is single-threaded by
/// design (§13.2); the frame-pipeline GIL story is W10's.
#[pyclass(name = "Stage", unsendable)]
struct PyStage {
    engine: Engine,
    /// Proxy identity registry: Mob → weakref to the one live proxy.
    proxies: RefCell<HashMap<Mob, Py<PyAny>>>,
    /// Python updaters, bridge-side, per mobject in insertion order — so
    /// dispatch happens with the engine lock released (the reentrancy law).
    py_updaters: RefCell<HashMap<Mob, Vec<Py<PyAny>>>>,
}

/// The subclassable mobject proxy. `weakref` + `dict` make it a normal
/// Python object; `subclass` makes MRO dispatch real.
#[pyclass(subclass, weakref, dict, unsendable)]
struct BridgeMobject {
    engine: Option<Engine>,
    mob: Option<Mob>,
}

/// A live record view across the boundary — the Rust model of the exported
/// NumPy structured array (true buffer-protocol export lands with
/// fmn-python, where the numpy package is sanctioned).
#[pyclass(name = "RecordView", unsendable)]
struct PyRecordView {
    view: RecordView,
}

// --------------------------------------------------------------------------
// helpers
// --------------------------------------------------------------------------

fn parse_dtype(dtype: &Bound<'_, PyAny>) -> PyResult<RecordSchema> {
    let mut fields = Vec::new();
    for item in dtype.try_iter()? {
        let (name, width): (String, usize) = item?.extract().map_err(|_| {
            PyValueError::new_err("data_dtype entries must be (name: str, lanes: int) pairs")
        })?;
        if width == 0 {
            return Err(PyValueError::new_err(format!(
                "data_dtype field `{name}` has zero lanes"
            )));
        }
        // Spike-only: FieldSpec holds `&'static str`, so Python-declared
        // names are leaked. GAP for W3: the production RecordSchema takes
        // owned/interned names so user dtypes never leak.
        fields.push(FieldSpec {
            name: Box::leak(name.into_boxed_str()),
            width,
        });
    }
    if fields.is_empty() {
        return Err(PyValueError::new_err("data_dtype declares no fields"));
    }
    Ok(RecordSchema::new(fields))
}

impl BridgeMobject {
    fn bound(&self) -> PyResult<(Engine, Mob)> {
        match (&self.engine, self.mob) {
            (Some(engine), Some(mob)) => Ok((Rc::clone(engine), mob)),
            _ => Err(StaleHandleError::new_err(
                "proxy is not bound to a stage (construct via Mobject(stage) / _engine_init)",
            )),
        }
    }
}

/// Resolve the live entry or raise `StaleHandleError`. The borrow is scoped
/// to `f` — never held across a Python call.
fn with_buffer<T>(
    engine: &Engine,
    mob: Mob,
    f: impl FnOnce(&mut RecordBuffer) -> T,
) -> PyResult<T> {
    let mut stage = engine.borrow_mut();
    match stage.get_mut(mob) {
        Some(entry) => Ok(f(&mut entry.buffer)),
        None => Err(StaleHandleError::new_err(
            "engine entry was deleted out from under this proxy",
        )),
    }
}

/// Fetch the registered live proxy for `mob`, if any.
fn live_proxy<'py>(py: Python<'py>, stage: &PyStage, mob: Mob) -> Option<Bound<'py, PyAny>> {
    let proxies = stage.proxies.borrow();
    let weak = proxies.get(&mob)?;
    let target = weak.bind(py).call0().ok()?;
    (!target.is_none()).then_some(target)
}

/// Create a proxy of `cls` bound to `mob` WITHOUT running `__init__`
/// (the copy path: manim's `copy.copy` never re-runs constructors).
fn new_bound_proxy<'py>(
    py: Python<'py>,
    stage_ref: &Bound<'py, PyStage>,
    cls: &Bound<'py, PyType>,
    mob: Mob,
) -> PyResult<Bound<'py, PyAny>> {
    let proxy = cls.call_method1("__new__", (cls,))?;
    {
        let mut cell: PyRefMut<'_, BridgeMobject> = proxy.extract()?;
        let stage = stage_ref.borrow();
        stage.engine.borrow_mut().pin(mob);
        cell.engine = Some(Rc::clone(&stage.engine));
        cell.mob = Some(mob);
    }
    register_proxy(py, stage_ref, mob, &proxy)?;
    Ok(proxy)
}

fn register_proxy(
    py: Python<'_>,
    stage_ref: &Bound<'_, PyStage>,
    mob: Mob,
    proxy: &Bound<'_, PyAny>,
) -> PyResult<()> {
    let weakref = py.import("weakref")?.call_method1("ref", (proxy,))?;
    stage_ref
        .borrow()
        .proxies
        .borrow_mut()
        .insert(mob, weakref.unbind());
    Ok(())
}

/// Engine-side copy + the Reference `__dict__` remap rule. Shared by
/// `BridgeMobject.copy()` and the transform begin-state.
fn copy_proxy<'py>(
    py: Python<'py>,
    proxy: &Bound<'py, BridgeMobject>,
) -> PyResult<Bound<'py, PyAny>> {
    let (engine, handle) = proxy.borrow().bound()?;

    // 1. Engine copy: record data deep-copies, family-internal edges remap,
    //    updater callables share by reference (G0-1 ratified semantics).
    let (old_family, new_root, new_family) = {
        let mut stage = engine.borrow_mut();
        let old_family = stage.family(handle);
        let new_root = stage
            .copy_family(handle)
            .ok_or_else(|| StaleHandleError::new_err("copy source vanished mid-copy"))?;
        let new_family = stage.family(new_root);
        (old_family, new_root, new_family)
    };
    debug_assert_eq!(old_family.len(), new_family.len());

    let stage_obj = proxy.getattr("_stage")?;
    let stage_ref = stage_obj
        .cast::<PyStage>()
        .map_err(|_| PyRuntimeError::new_err("proxy _stage attribute is not a Stage"))?;

    // 2. Mirror proxies for every old family member that has one, of the
    //    same Python type, `__dict__` shallow-copied (copy.copy semantics).
    let mut mirrors: Vec<Bound<'_, PyAny>> = Vec::new();
    for (old, new) in old_family.iter().zip(new_family.iter()) {
        let Some(old_proxy) = live_proxy(py, &stage_ref.borrow(), *old) else {
            continue;
        };
        let cls = old_proxy.get_type();
        let new_proxy = new_bound_proxy(py, stage_ref, &cls, *new)?;
        let old_dict = old_proxy.getattr("__dict__")?;
        let new_dict = new_proxy.getattr("__dict__")?;
        new_dict.call_method1("update", (old_dict,))?;
        mirrors.push(new_proxy);
    }

    // 3. Python updaters share by reference, like manim's
    //    `result.updaters = list(self.updaters)`.
    {
        let stage_cell = stage_ref.borrow();
        let mut updaters = stage_cell.py_updaters.borrow_mut();
        let shared: Vec<(Mob, Vec<Py<PyAny>>)> = old_family
            .iter()
            .zip(new_family.iter())
            .filter_map(|(old, new)| {
                updaters
                    .get(old)
                    .map(|v| (*new, v.iter().map(|c| c.clone_ref(py)).collect()))
            })
            .collect();
        for (new, callables) in shared {
            updaters.insert(new, callables);
        }
    }

    // 4. The Reference remap rule (Mobject.copy): a `__dict__` value that
    //    is a family-member mobject remaps to the copy's corresponding
    //    member; family-external mobjects and plain values stay shared.
    for new_proxy in &mirrors {
        let dict = new_proxy.getattr("__dict__")?;
        let dict = dict.cast::<PyDict>().expect("__dict__ is a dict");
        let mut remaps: Vec<(Py<PyAny>, Py<PyAny>)> = Vec::new();
        for (key, value) in dict.iter() {
            let Ok(value_proxy) = value.cast::<BridgeMobject>() else {
                continue;
            };
            let Some(value_mob) = value_proxy.borrow().mob else {
                continue;
            };
            if let Some(pos) = old_family.iter().position(|m| *m == value_mob) {
                if let Some(mapped) = live_proxy(py, &stage_ref.borrow(), new_family[pos]) {
                    remaps.push((key.unbind(), mapped.unbind()));
                }
            }
        }
        for (key, mapped) in remaps {
            dict.set_item(key, mapped)?;
        }
    }

    live_proxy(py, &stage_ref.borrow(), new_root)
        .ok_or_else(|| PyRuntimeError::new_err("copy produced no root proxy"))
}

// --------------------------------------------------------------------------
// PyStage
// --------------------------------------------------------------------------

#[pymethods]
impl PyStage {
    #[new]
    fn new() -> Self {
        Self {
            engine: Rc::new(RefCell::new(Stage::new())),
            proxies: RefCell::new(HashMap::new()),
            py_updaters: RefCell::new(HashMap::new()),
        }
    }

    fn time(&self) -> f64 {
        self.engine.borrow().time()
    }

    /// Number of scene roots (test observability).
    fn root_count(&self) -> usize {
        self.engine.borrow().roots().len()
    }

    fn add_to_scene(&self, mob: &Bound<'_, BridgeMobject>) -> PyResult<bool> {
        let (engine, handle) = mob.borrow().bound()?;
        if !Rc::ptr_eq(&engine, &self.engine) {
            return Err(ForeignStageError::new_err(
                "cannot add: proxy belongs to a different stage",
            ));
        }
        Ok(self.engine.borrow_mut().add_to_scene(handle))
    }

    fn remove_from_scene(&self, mob: &Bound<'_, BridgeMobject>) -> PyResult<()> {
        let (engine, handle) = mob.borrow().bound()?;
        if !Rc::ptr_eq(&engine, &self.engine) {
            return Err(ForeignStageError::new_err(
                "cannot remove: proxy belongs to a different stage",
            ));
        }
        self.engine.borrow_mut().remove_from_scene(handle);
        Ok(())
    }

    /// The engine-driven update: rooted families in traversal order, each
    /// target's Python updaters in insertion order, called with `(proxy,
    /// dt)` and NO engine borrow held (the reentrancy law) — an updater may
    /// re-enter any engine API, and an updater exception propagates as
    /// itself. Time advances afterwards.
    fn update(slf: &Bound<'_, Self>, dt: f64) -> PyResult<()> {
        let py = slf.py();
        let targets: Vec<Mob> = {
            let cell = slf.borrow();
            let stage = cell.engine.borrow();
            let mut out = Vec::new();
            for &root in stage.roots() {
                for member in stage.family(root) {
                    if !out.contains(&member) {
                        out.push(member);
                    }
                }
            }
            out
        };
        for target in targets {
            let callables: Vec<Py<PyAny>> = {
                let cell = slf.borrow();
                let updaters = cell.py_updaters.borrow();
                match updaters.get(&target) {
                    Some(list) => list.iter().map(|c| c.clone_ref(py)).collect(),
                    None => continue,
                }
            };
            for callable in callables {
                let proxy = {
                    let cell = slf.borrow();
                    live_proxy(py, &cell, target)
                };
                let Some(proxy) = proxy else { continue };
                callable.bind(py).call1((proxy, dt))?;
            }
        }
        let cell = slf.borrow();
        let mut stage = cell.engine.borrow_mut();
        stage.update(dt); // engine-native updaters + time advance
        Ok(())
    }

    /// The engine-driven transform loop: for each alpha, dispatch
    /// `interpolate(start, target, alpha)` through the proxy's MRO — a
    /// Python override replaces the Rust default per ordinary Python rules.
    fn run_transform(
        &self,
        mob: &Bound<'_, BridgeMobject>,
        target: &Bound<'_, BridgeMobject>,
        steps: usize,
    ) -> PyResult<()> {
        let (engine, _) = mob.borrow().bound()?;
        let (target_engine, _) = target.borrow().bound()?;
        if !Rc::ptr_eq(&engine, &self.engine) || !Rc::ptr_eq(&target_engine, &self.engine) {
            return Err(ForeignStageError::new_err(
                "transform endpoints must live in this stage",
            ));
        }
        let py = mob.py();
        // The begin-state snapshot: an engine-side copy (manim's
        // Animation.begin() captures starting_mobject the same way).
        let start = copy_proxy(py, mob)?;
        for k in 0..=steps {
            let alpha = if steps == 0 {
                1.0
            } else {
                k as f64 / steps as f64
            };
            mob.call_method1("interpolate", (&start, target, alpha))?;
        }
        Ok(())
    }
}

// --------------------------------------------------------------------------
// BridgeMobject
// --------------------------------------------------------------------------

#[pymethods]
impl BridgeMobject {
    /// tp_new tolerates the subclass constructor's arguments; binding
    /// happens in `_engine_init` (pyo3 has no custom `tp_init`, so the
    /// manimlib layer's pure-Python `__init__` drives the engine seam —
    /// recorded as the sanctioned shim shape for fmn-python).
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn py_new(_args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        Self {
            engine: None,
            mob: None,
        }
    }

    /// The default record layout — a Python subclass overrides this class
    /// attribute and the engine honors the override (scenario S3).
    #[classattr]
    fn data_dtype() -> Vec<(&'static str, usize)> {
        vec![("point", 3), ("rgba", 4)]
    }

    /// THE ENGINE SEAM. Reads the (possibly overridden) `data_dtype` off
    /// the type, allocates the arena entry, pins it for proxy lifetime,
    /// then drives `init_data` → `init_points` → `init_uniforms` through
    /// the instance — i.e. through Python MRO, so subclass overrides win.
    /// No engine borrow is held across any of the three calls.
    fn _engine_init(slf: &Bound<'_, Self>, stage: &Bound<'_, PyStage>) -> PyResult<()> {
        if slf.borrow().mob.is_some() {
            return Err(PyRuntimeError::new_err("proxy is already bound"));
        }
        let py = slf.py();
        let dtype = slf.getattr("data_dtype")?; // MRO: subclass attr wins
        let schema = parse_dtype(&dtype)?;
        let mob = {
            let stage_cell = stage.borrow();
            let mut engine = stage_cell.engine.borrow_mut();
            let mob = engine.add(Mobject {
                buffer: RecordBuffer::new(schema, 0),
                submobjects: Vec::new(),
            });
            engine.pin(mob); // the proxy holds one pin for its lifetime
            mob
        };
        {
            let mut cell = slf.borrow_mut();
            cell.engine = Some(Rc::clone(&stage.borrow().engine));
            cell.mob = Some(mob);
        }
        register_proxy(py, stage, mob, slf.as_any())?;
        slf.setattr("_stage", stage)?;
        // The lifecycle callbacks, engine-driven, MRO-dispatched. A Python
        // exception in any of them aborts construction and propagates
        // (scenario S7).
        slf.call_method0("init_data")?;
        slf.call_method0("init_points")?;
        slf.call_method0("init_uniforms")?;
        Ok(())
    }

    // ---------------------------------------------------- default lifecycle

    /// Default: the engine already allocated the buffer from `data_dtype`;
    /// nothing to do. Subclasses may resize/prefill.
    fn init_data(_slf: &Bound<'_, Self>) {}

    /// Default: no geometry. Subclasses fill points.
    fn init_points(_slf: &Bound<'_, Self>) {}

    /// Default uniforms — a plain Python dict attribute, as manim's are.
    fn init_uniforms(slf: &Bound<'_, Self>) -> PyResult<()> {
        let uniforms = PyDict::new(slf.py());
        uniforms.set_item("opacity", 1.0)?;
        slf.setattr("uniforms", uniforms)
    }

    /// Default interpolation: per-record, per-field lerp between the two
    /// endpoint buffers (the engine's certified lerp stands here in
    /// production). Python overrides replace this wholesale.
    fn interpolate(
        slf: &Bound<'_, Self>,
        start: &Bound<'_, BridgeMobject>,
        target: &Bound<'_, BridgeMobject>,
        alpha: f64,
    ) -> PyResult<()> {
        let (engine, mob) = slf.borrow().bound()?;
        let (start_engine, start_mob) = start.borrow().bound()?;
        let (target_engine, target_mob) = target.borrow().bound()?;
        if !Rc::ptr_eq(&engine, &start_engine) || !Rc::ptr_eq(&engine, &target_engine) {
            return Err(ForeignStageError::new_err(
                "interpolate endpoints must live in one stage",
            ));
        }
        let mut stage = engine.borrow_mut();
        let flat = |stage: &Stage, m: Mob| -> PyResult<Vec<f32>> {
            let entry = stage
                .get(m)
                .ok_or_else(|| StaleHandleError::new_err("interpolate endpoint vanished"))?;
            let buffer = &entry.buffer;
            let names: Vec<&'static str> = buffer.schema().field_names().collect();
            let mut cells = Vec::with_capacity(buffer.len() * buffer.schema().stride());
            for i in 0..buffer.len() {
                for f in &names {
                    cells.extend(buffer.read(i, f).expect("schema field"));
                }
            }
            Ok(cells)
        };
        let a = flat(&stage, start_mob)?;
        let b = flat(&stage, target_mob)?;
        if a.len() != b.len() {
            return Err(PyValueError::new_err(format!(
                "interpolate endpoints disagree in size ({} vs {} lanes) — align first",
                a.len(),
                b.len()
            )));
        }
        let entry = stage
            .get_mut(mob)
            .ok_or_else(|| StaleHandleError::new_err("interpolate target vanished"))?;
        let buffer = &mut entry.buffer;
        if buffer.len() * buffer.schema().stride() != a.len() {
            return Err(PyValueError::new_err(
                "interpolating mobject disagrees with endpoints in size",
            ));
        }
        let names: Vec<&'static str> = buffer.schema().field_names().collect();
        let mut cursor = 0usize;
        for i in 0..buffer.len() {
            for f in &names {
                let width = buffer.schema().field_width(f).expect("schema field");
                let mixed: Vec<f32> = (0..width)
                    .map(|k| {
                        let (x, y) = (a[cursor + k], b[cursor + k]);
                        x + (y - x) * alpha as f32
                    })
                    .collect();
                buffer.write(i, f, &mixed);
                cursor += width;
            }
        }
        Ok(())
    }

    // ------------------------------------------------------------- data api

    fn resize(slf: &Bound<'_, Self>, n: usize) -> PyResult<()> {
        let (engine, mob) = slf.borrow().bound()?;
        with_buffer(&engine, mob, |buffer| buffer.resize(n))
    }

    fn n_records(slf: &Bound<'_, Self>) -> PyResult<usize> {
        let (engine, mob) = slf.borrow().bound()?;
        with_buffer(&engine, mob, |buffer| buffer.len())
    }

    fn revision(slf: &Bound<'_, Self>) -> PyResult<u64> {
        let (engine, mob) = slf.borrow().bound()?;
        with_buffer(&engine, mob, |buffer| buffer.revision())
    }

    fn field_names(slf: &Bound<'_, Self>) -> PyResult<Vec<String>> {
        let (engine, mob) = slf.borrow().bound()?;
        with_buffer(&engine, mob, |buffer| {
            buffer.schema().field_names().map(String::from).collect()
        })
    }

    fn set_field(
        slf: &Bound<'_, Self>,
        field: &str,
        index: usize,
        values: Vec<f32>,
    ) -> PyResult<()> {
        let (engine, mob) = slf.borrow().bound()?;
        let ok = with_buffer(&engine, mob, |buffer| buffer.write(index, field, &values))?;
        if ok {
            Ok(())
        } else {
            Err(PyKeyError::new_err(format!(
                "no field `{field}` at record {index} with {} lanes",
                values.len()
            )))
        }
    }

    fn get_field(slf: &Bound<'_, Self>, field: &str, index: usize) -> PyResult<Vec<f32>> {
        let (engine, mob) = slf.borrow().bound()?;
        with_buffer(&engine, mob, |buffer| buffer.read(index, field))?
            .ok_or_else(|| PyKeyError::new_err(format!("no field `{field}` at record {index}")))
    }

    /// Export a live view under the §8.2 protocol (scenario S4). The NumPy
    /// structured-array skin over this lands with fmn-python.
    #[pyo3(signature = (writable = true))]
    fn data_view(slf: &Bound<'_, Self>, writable: bool) -> PyResult<PyRecordView> {
        let (engine, mob) = slf.borrow().bound()?;
        let view = with_buffer(&engine, mob, |buffer| buffer.export_view(writable))?;
        Ok(PyRecordView { view })
    }

    // ------------------------------------------------------ family & scene

    fn attach(slf: &Bound<'_, Self>, child: &Bound<'_, BridgeMobject>) -> PyResult<()> {
        let (engine, mob) = slf.borrow().bound()?;
        let (child_engine, child_mob) = child.borrow().bound()?;
        if !Rc::ptr_eq(&engine, &child_engine) {
            return Err(ForeignStageError::new_err(
                "cannot attach across stages (copy instead)",
            ));
        }
        engine.borrow_mut().attach(mob, child_mob);
        Ok(())
    }

    fn family_size(slf: &Bound<'_, Self>) -> PyResult<usize> {
        let (engine, mob) = slf.borrow().bound()?;
        let stage = engine.borrow();
        Ok(stage.family(mob).len())
    }

    /// manim `copy()` semantics end to end (scenario S5).
    fn copy<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyAny>> {
        copy_proxy(slf.py(), slf)
    }

    /// Register a Python updater; the ENGINE calls it on every
    /// `stage.update(dt)` with `(proxy, dt)` (scenario S9; the
    /// per-frame-callback PG-8 class).
    fn add_updater(slf: &Bound<'_, Self>, callable: Py<PyAny>) -> PyResult<()> {
        let (_, mob) = slf.borrow().bound()?;
        let stage_obj = slf.getattr("_stage")?;
        let stage_ref = stage_obj
            .cast::<PyStage>()
            .map_err(|_| PyRuntimeError::new_err("proxy _stage attribute is not a Stage"))?;
        stage_ref
            .borrow()
            .py_updaters
            .borrow_mut()
            .entry(mob)
            .or_default()
            .push(callable);
        Ok(())
    }

    /// Explicit engine-side delete. With this proxy still alive the delete
    /// DEFERS (the pin story): the entry survives until the proxy is
    /// collected (scenario S6).
    fn delete(slf: &Bound<'_, Self>) -> PyResult<bool> {
        let (engine, mob) = slf.borrow().bound()?;
        Ok(engine.borrow_mut().delete(mob))
    }

    fn is_alive(slf: &Bound<'_, Self>) -> bool {
        match slf.borrow().bound() {
            Ok((engine, mob)) => engine.borrow().contains(mob),
            Err(_) => false,
        }
    }

    /// Stable engine identity `(stage_id, index, generation)` — test
    /// observability for the S6 identity assertions.
    fn handle_token(slf: &Bound<'_, Self>) -> PyResult<(u64, u32, u32)> {
        let (_, mob) = slf.borrow().bound()?;
        Ok(mob.token())
    }

    /// A no-op crossing for the bench harness.
    fn noop(_slf: &Bound<'_, Self>) {}
}

impl Drop for BridgeMobject {
    /// The proxy's pin releases on collection; a deferred delete completes
    /// here — engine memory can never be pulled out from under a live
    /// proxy, and a dead proxy never leaks the entry.
    fn drop(&mut self) {
        if let (Some(engine), Some(mob)) = (&self.engine, self.mob) {
            engine.borrow_mut().unpin(mob);
        }
    }
}

// --------------------------------------------------------------------------
// PyRecordView
// --------------------------------------------------------------------------

#[pymethods]
impl PyRecordView {
    fn __len__(&self) -> usize {
        self.view.len()
    }

    fn read(&self, index: usize, field: &str) -> PyResult<Vec<f32>> {
        self.view
            .read(index, field)
            .ok_or_else(|| PyKeyError::new_err(format!("no field `{field}` at record {index}")))
    }

    /// Writes through the view are visible to the engine while attached and
    /// bump the revision (protocol rule 4).
    fn write(&self, index: usize, field: &str, values: Vec<f32>) -> PyResult<()> {
        if self.view.write(index, field, &values) {
            Ok(())
        } else {
            Err(PyValueError::new_err(format!(
                "cannot write `{field}` at record {index} ({} lanes) — read-only view, bad field, or bad width",
                values.len()
            )))
        }
    }

    /// Whether the view still aliases the proxy's current storage
    /// generation (false after a resize swapped generations — rule 3).
    fn attached(&self, mob: &Bound<'_, BridgeMobject>) -> PyResult<bool> {
        let (engine, handle) = mob.borrow().bound()?;
        with_buffer(&engine, handle, |buffer| self.view.is_attached_to(buffer))
    }
}

// --------------------------------------------------------------------------
// module
// --------------------------------------------------------------------------

#[pymodule(gil_used = true)]
fn fmn_spike_bridge(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyStage>()?;
    m.add_class::<BridgeMobject>()?;
    m.add_class::<PyRecordView>()?;
    m.add("StaleHandleError", py.get_type::<StaleHandleError>())?;
    m.add("ForeignStageError", py.get_type::<ForeignStageError>())?;
    m.add("__engine__", "g0-1-object-model prototype")?;
    Ok(())
}
