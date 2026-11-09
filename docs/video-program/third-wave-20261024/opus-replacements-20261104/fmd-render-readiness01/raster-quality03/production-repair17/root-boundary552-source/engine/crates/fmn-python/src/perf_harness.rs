//! PG-8 binding-tax measurement harness (§17.2, fm-zoi).
//!
//! This module is the bridge-side producer for the PG-8 class table. It
//! drives the *real* bridge — an embedded CPython interpreter running the
//! production `manimlib` module — through the four declared scene classes,
//! and times full workload repetitions with a monotonic clock. It performs
//! no policy evaluation and no artifact I/O: `fmn-conformance`'s
//! `perf_pg8` module owns identity, statistics, and evidence; this module
//! returns raw per-repetition observations plus the final scene state
//! (the self-golden input proving the measured workload did the declared
//! work).
//!
//! The `native-builtins` class also measures a pure-Rust twin — the same
//! mobjects, the same engine-executable updaters, no interpreter — so the
//! gate's ratio-ppm value is the observed binding tax on a callback-free
//! scene rather than an absolute number on one host.
//!
//! Timing values are observations, not assertions: the harness marks a
//! repetition invalid only on a timer anomaly (zero elapsed), and every
//! repetition is retained for the robust statistics downstream.

use std::sync::Once;
use std::time::Instant;

use fmn_anim::{DeclaredOp, DeclaredUpdater};
use fmn_mobject::{Mobject, RecordBuffer, RecordSchema, Stage};
use fmn_scene::{RuntimeConfig, Scene};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// The four PG-8 scene classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pg8Class {
    /// Built-in mobjects + engine-executable updaters; no Python callback.
    NativeBuiltins,
    /// One Python updater per mobject, per frame, through `set_field`.
    PerFrameCallback,
    /// One Python updater per mobject, per frame, through the live
    /// zero-copy point view.
    PointTransformCallback,
    /// Dynamic Python subclasses: override-driven construction plus
    /// per-frame bound-method updater dispatch.
    DynamicSubclass,
}

impl Pg8Class {
    /// Fixed class order for producers and reports.
    pub const ALL: [Pg8Class; 4] = [
        Pg8Class::NativeBuiltins,
        Pg8Class::PerFrameCallback,
        Pg8Class::PointTransformCallback,
        Pg8Class::DynamicSubclass,
    ];

    /// Stable scenario name (the policy catalog's `scenario` column).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Pg8Class::NativeBuiltins => "native-builtins",
            Pg8Class::PerFrameCallback => "per-frame-callback",
            Pg8Class::PointTransformCallback => "point-transform-callback",
            Pg8Class::DynamicSubclass => "dynamic-subclass",
        }
    }

    /// Whether the timed repetition includes scene construction.
    #[must_use]
    fn rebuilds_per_repetition(self) -> bool {
        matches!(self, Pg8Class::DynamicSubclass)
    }
}

/// One retained workload repetition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepObservation {
    /// Wall time of one bridge-side workload repetition.
    pub elapsed_ns: u128,
    /// Wall time of the pure-Rust twin (`native-builtins` only).
    pub reference_ns: Option<u128>,
    /// Host-quality failure reason; the observation is retained regardless.
    pub invalid_reason: Option<String>,
}

/// Raw measurement output: retained repetitions plus the final scene
/// state bytes (self-golden input).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessRun {
    /// `warmup + repetitions` is excluded; exactly `repetitions` entries.
    pub repetitions: Vec<RepObservation>,
    /// Final bridge-side scene state, deterministic for a fixed workload.
    pub result_state: Vec<u8>,
    /// Final pure-Rust twin state (`native-builtins` only); must equal
    /// `result_state` — the twin runs bit-identical declared updaters.
    pub reference_state: Option<Vec<u8>>,
}

const PYTHON_PRELUDE: &str = r"
import numpy as np
import manimlib
from manimlib import Mobject, Scene
from manimlib import _NativeUpdater as NativeUpdater

VELOCITY = (0.1, -0.2, 0.3)

def state_bytes(scene):
    out = bytearray()
    for mob in scene.mobjects:
        fields = mob.field_names()
        for i in range(mob.n_records()):
            for f in fields:
                out += np.asarray(mob.get_field(f, i), dtype=np.float32).tobytes()
    return bytes(out)
";

const NATIVE_BUILTINS_SOURCE: &str = r"
def build(count):
    scene = Scene()
    mobs = []
    for _ in range(count):
        mob = Mobject()
        mob.resize(4)
        mobs.append(mob)
    scene.add(*mobs)
    scene._keep = mobs
    handles = [NativeUpdater.shift(mob, 'point', [0.1, -0.2, 0.3]) for mob in mobs]
    return scene, handles
";

const PER_FRAME_CALLBACK_SOURCE: &str = r"
def _make_updater():
    def update(mob, dt):
        x, y, z = mob.get_field('point', 0)
        mob.set_field('point', 0, [x + 0.1 * dt, y - 0.2 * dt, z + 0.3 * dt])
    return update

def build(count):
    scene = Scene()
    mobs = []
    for _ in range(count):
        mob = Mobject()
        mob.resize(4)
        mob.add_updater(_make_updater(), call=False)
        mobs.append(mob)
    scene.add(*mobs)
    scene._keep = mobs
    return scene
";

const POINT_TRANSFORM_CALLBACK_SOURCE: &str = r"
def _make_updater():
    velocity = np.array([0.1, -0.2, 0.3], dtype=np.float64)
    def update(mob, dt):
        points = mob.data['point']
        points[:] = points.astype(np.float64) + velocity * dt
    return update

def build(count):
    scene = Scene()
    mobs = []
    for _ in range(count):
        mob = Mobject()
        mob.resize(4)
        mob.add_updater(_make_updater(), call=False)
        mobs.append(mob)
    scene.add(*mobs)
    scene._keep = mobs
    return scene
";

const DYNAMIC_SUBCLASS_SOURCE: &str = r"
class DynamicPulse(Mobject):
    def init_points(self):
        self.resize(4)

    def init_uniforms(self):
        super().init_uniforms()
        self.uniforms['glow'] = 0.5

    def tick(self, m, dt):
        x, y, z = self.get_field('point', 0)
        self.set_field('point', 0, [x + 0.1 * dt, y - 0.2 * dt, z + 0.3 * dt])

def build(count):
    scene = Scene()
    mobs = []
    for _ in range(count):
        mob = DynamicPulse()
        mob.add_updater(mob.tick, call=False)
        mobs.append(mob)
    scene.add(*mobs)
    scene._keep = mobs
    return scene
";

impl Pg8Class {
    fn source(self) -> &'static str {
        match self {
            Pg8Class::NativeBuiltins => NATIVE_BUILTINS_SOURCE,
            Pg8Class::PerFrameCallback => PER_FRAME_CALLBACK_SOURCE,
            Pg8Class::PointTransformCallback => POINT_TRANSFORM_CALLBACK_SOURCE,
            Pg8Class::DynamicSubclass => DYNAMIC_SUBCLASS_SOURCE,
        }
    }
}

static PYTHON_INIT: Once = Once::new();

/// Run a PG-8 workload with an exclusively owned temporary `manimlib`.
///
/// The bridge's scene and mobject proxies are intentionally thread-confined.
/// CPython cannot collect every cycle containing a PyO3 extension object, so
/// leaving the temporary module or workload globals alive until a later
/// measurement can make their finalizers run on that later measurement's OS
/// thread. Tear down the entire graph while the creating thread still owns
/// it, and fail the measurement if Python reports an unraisable destructor.
fn with_owned_python_module<T>(
    body: impl for<'py> FnOnce(
        Python<'py>,
        &Bound<'py, PyModule>,
        &Bound<'py, PyDict>,
    ) -> Result<T, String>,
) -> Result<T, String> {
    let _guard = crate::python_embedding_lock();
    PYTHON_INIT.call_once(Python::initialize);
    Python::attach(|py| {
        let sys = py.import("sys").map_err(|error| error.to_string())?;
        let gc = py.import("gc").map_err(|error| error.to_string())?;
        let weakref = py.import("weakref").map_err(|error| error.to_string())?;
        let modules = sys.getattr("modules").map_err(|error| error.to_string())?;
        let module_names = || -> Result<Vec<String>, String> {
            modules
                .call_method0("keys")
                .and_then(|keys| {
                    keys.try_iter()?
                        .map(|item| item.and_then(|name| name.extract::<String>()))
                        .collect()
                })
                .map_err(|error| error.to_string())
        };
        let existing = module_names()?;
        if existing
            .iter()
            .any(|name| name == "manimlib" || name.starts_with("manimlib."))
        {
            return Err("PG-8 harness found a pre-existing manimlib module".to_owned());
        }

        let hook_globals = PyDict::new(py);
        #[cfg(any(test, feature = "gauntlet"))]
        hook_globals
            .set_item(
                "_fmn_source_package",
                concat!(env!("CARGO_MANIFEST_DIR"), "/python"),
            )
            .map_err(|error| error.to_string())?;
        hook_globals
            .set_item("_fmn_unraisable", PyList::empty(py))
            .map_err(|error| error.to_string())?;
        let hook_source = std::ffi::CString::new(
            r#"import sys as _fmn_sys
_fmn_previous_path = _fmn_sys.path[:]
if "_fmn_source_package" in globals():
    _fmn_sys.path.insert(0, _fmn_source_package)
_fmn_old_unraisablehook = _fmn_sys.unraisablehook
def _fmn_capture_unraisable(event):
    _fmn_unraisable.append(
        f'{type(event.exc_value).__name__}: {event.exc_value}'
    )
_fmn_sys.unraisablehook = _fmn_capture_unraisable
"#,
        )
        .map_err(|_| "unraisable-hook source contains NUL".to_owned())?;
        py.run(
            hook_source.as_c_str(),
            Some(&hook_globals),
            Some(&hook_globals),
        )
        .map_err(|error| error.to_string())?;

        let module = PyModule::new(py, "manimlib").map_err(|error| error.to_string())?;
        modules
            .set_item("manimlib", &module)
            .map_err(|error| error.to_string())?;
        let module_weakref = weakref
            .getattr("ref")
            .and_then(|constructor| constructor.call1((&module,)))
            .map_err(|error| error.to_string())?;
        let globals = PyDict::new(py);
        globals
            .set_item("__name__", "__fmn_pg8_harness__")
            .map_err(|error| error.to_string())?;

        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::manimlib(py, &module).map_err(|error| error.to_string())?;
            body(py, &module, &globals)
        }));

        let lifetime_globals = PyDict::new(py);
        lifetime_globals
            .set_item("gc", &gc)
            .map_err(|error| error.to_string())?;
        lifetime_globals
            .set_item("weakref", &weakref)
            .map_err(|error| error.to_string())?;
        lifetime_globals
            .set_item("Bridge", py.get_type::<crate::BridgeMobject>())
            .map_err(|error| error.to_string())?;
        lifetime_globals
            .set_item("worker_namespace", "__fmn_pg8_harness__")
            .map_err(|error| error.to_string())?;
        let mut cleanup_errors = Vec::new();
        // CPython cannot see every native instance -> Python type ownership
        // edge. A worker-authored class attribute holding its own instance
        // therefore survives both clearing globals and draining the cache.
        // Release only data roots on classes authored in this owned workload;
        // imported classes, methods, descriptors and finalizers stay intact.
        if let Err(error) = py.run(
            c"import types\nproxies = [(weakref.ref(obj), type(obj).__name__) for obj in gc.get_objects() if isinstance(obj, Bridge)]\nowned_classes = [obj for obj in gc.get_objects() if isinstance(obj, type) and type.__getattribute__(obj, '__module__') == worker_namespace]\nprotected = {'__dict__', '__weakref__', '__slots__', '__module__', '__doc__', '__qualname__', '__name__', '__del__'}\ndescriptors = (types.FunctionType, types.BuiltinFunctionType, types.MemberDescriptorType, types.GetSetDescriptorType, classmethod, staticmethod, property)\nfor cls in owned_classes:\n    for name, value in tuple(type.__getattribute__(cls, '__dict__').items()):\n        if name not in protected and not isinstance(value, descriptors):\n            type.__delattr__(cls, name)\nowned_classes.clear()\nfor name in ('cls', 'name', 'value'):\n    globals().pop(name, None)\n",
            Some(&lifetime_globals),
            None,
        ) {
            cleanup_errors.push(format!("release owned class roots: {error}"));
        }
        // Python callbacks retain their globals and the globals retain the
        // scene graph. Cache entries strongly own those Python classes. A
        // class can retain module globals or even an unsendable native proxy;
        // release those owners while this same worker still holds the GIL.
        // Waiting for Rust thread-local destruction defers their decrefs to a
        // later worker and can both leak the module and finalize on the wrong
        // OS thread. This is required for production measurements too.
        crate::method_cache::clear_for_worker_teardown(py);
        globals.clear();
        if let Err(error) = py.run(
            c"_fmn_sys.path[:] = _fmn_previous_path",
            Some(&hook_globals),
            None,
        ) {
            cleanup_errors.push(format!("restore module search path: {error}"));
        }
        match module_names() {
            Ok(names) => {
                for name in names {
                    if name == "manimlib" || name.starts_with("manimlib.") {
                        if let Ok(value) = modules.get_item(&name)
                            && let Ok(owned_module) = value.cast_into::<PyModule>()
                        {
                            owned_module.dict().clear();
                        }
                        if let Err(error) = modules.del_item(&name) {
                            cleanup_errors.push(format!("remove module {name}: {error}"));
                        }
                    }
                }
            }
            Err(error) => cleanup_errors.push(format!("enumerate modules: {error}")),
        }
        module.dict().clear();
        drop(module);
        // As with the scene-worker teardown, a finalizer can release another
        // class/global cycle. Two collections are not a waiver: the module
        // weak reference and unraisable hook below must still be clean.
        for _ in 0..2 {
            if let Err(error) = gc.call_method0("collect") {
                cleanup_errors.push(format!("collect Python cycles: {error}"));
            }
        }
        if crate::method_cache::stats().entries != 0 {
            cleanup_errors.push("method cache retained entries after worker teardown".to_owned());
        }

        if let Err(error) = py.run(
            c"survivors = [name for ref, name in proxies if ref() is not None]",
            Some(&lifetime_globals),
            None,
        ) {
            cleanup_errors.push(format!("observe native worker lifetimes: {error}"));
        }
        match lifetime_globals.get_item("survivors") {
            Ok(Some(value)) => match value.extract::<Vec<String>>() {
                Ok(names) if names.is_empty() => {}
                Ok(names) => cleanup_errors.push(format!(
                    "native proxies survived worker teardown: {names:?}"
                )),
                Err(error) => cleanup_errors.push(format!("read native worker lifetimes: {error}")),
            },
            Ok(None) => {
                cleanup_errors.push("native worker lifetime observations missing".to_owned())
            }
            Err(error) => cleanup_errors.push(format!("read native worker lifetimes: {error}")),
        }

        match hook_globals.get_item("_fmn_old_unraisablehook") {
            Ok(Some(old_hook)) => {
                if let Err(error) = sys.setattr("unraisablehook", old_hook) {
                    cleanup_errors.push(format!("restore unraisable hook: {error}"));
                }
            }
            Ok(None) => cleanup_errors.push("prior unraisable hook is missing".to_owned()),
            Err(error) => cleanup_errors.push(format!("read prior unraisable hook: {error}")),
        }
        match hook_globals.get_item("_fmn_unraisable") {
            Ok(Some(errors)) => match errors.extract::<Vec<String>>() {
                Ok(errors) if errors.is_empty() => {}
                Ok(errors) => cleanup_errors.push(format!(
                    "Python teardown emitted unraisable errors: {errors:?}"
                )),
                Err(error) => {
                    cleanup_errors.push(format!("read unraisable errors: {error}"));
                }
            },
            Ok(None) => cleanup_errors.push("unraisable capture is missing".to_owned()),
            Err(error) => cleanup_errors.push(format!("read unraisable capture: {error}")),
        }
        match module_weakref.call0() {
            Ok(value) if value.is_none() => {}
            Ok(_) => cleanup_errors.push("temporary manimlib module survived teardown".to_owned()),
            Err(error) => cleanup_errors.push(format!("read module weak reference: {error}")),
        }

        let result = match outcome {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        };
        if cleanup_errors.is_empty() {
            result
        } else {
            let cleanup = cleanup_errors.join("; ");
            match result {
                Ok(_) => Err(cleanup),
                Err(primary) => Err(format!("{primary}; worker cleanup also failed: {cleanup}")),
            }
        }
    })
}

/// The pure-Rust twin of the `native-builtins` workload: same mobjects,
/// same declared shift updaters, no interpreter in the loop.
struct RustTwin {
    scene: Scene,
}

impl RustTwin {
    fn build(mobjects: usize) -> Result<Self, String> {
        let mut scene = Scene::new(RuntimeConfig::default(), 0)
            .map_err(|error| format!("rust twin scene: {error}"))?;
        for _ in 0..mobjects {
            let buffer = RecordBuffer::new(RecordSchema::mobject(), 4)
                .expect("the harness twin buffer cannot overflow");
            let mob = scene.stage_mut().add(Mobject::from_buffer(buffer));
            scene
                .stage_mut()
                .add_to_scene(mob)
                .map_err(|error| format!("rust twin root: {error}"))?;
            let updater = DeclaredUpdater::new(DeclaredOp::Shift {
                field: "point".to_owned(),
                velocity: vec![0.1, -0.2, 0.3],
            })
            .map_err(|error| format!("rust twin updater: {error}"))?;
            let mut cell = updater;
            scene
                .stage_mut()
                .add_dt_updater(
                    mob,
                    move |stage: &mut Stage, mob: fmn_mobject::Mob, dt: f64| {
                        if let Some(entry) = stage.get_mut(mob) {
                            cell.apply(&mut entry.buffer, dt);
                        }
                    },
                    false,
                )
                .map_err(|error| format!("rust twin registration: {error}"))?;
        }
        Ok(Self { scene })
    }

    fn step(&mut self, dt: f64, frames: usize) {
        for _ in 0..frames {
            self.scene.stage_mut().update(dt);
        }
    }

    fn state_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let roots = self.scene.stage().roots().to_vec();
        for mob in roots {
            let Some(entry) = self.scene.stage().get(mob) else {
                continue;
            };
            let fields: Vec<String> = entry
                .buffer
                .schema()
                .fields()
                .iter()
                .map(|field| field.name.clone())
                .collect();
            for index in 0..entry.buffer.len() {
                for field in &fields {
                    if let Some(values) = entry.buffer.read(index, field) {
                        for value in values {
                            out.extend_from_slice(&value.to_le_bytes());
                        }
                    }
                }
            }
        }
        out
    }
}

fn observation(
    elapsed: std::time::Duration,
    reference: Option<std::time::Duration>,
) -> RepObservation {
    let elapsed_ns = elapsed.as_nanos();
    let invalid_reason = if elapsed_ns == 0 {
        Some("zero elapsed time: clock resolution below workload".to_owned())
    } else {
        None
    };
    RepObservation {
        elapsed_ns,
        reference_ns: reference.map(|value| value.as_nanos()),
        invalid_reason,
    }
}

/// Measure one PG-8 scene class against the real bridge.
///
/// Builds the class's canonical workload (`mobjects` built-in mobjects),
/// runs `warmup` untimed repetitions, then times `repetitions`
/// repetitions of `frames` `scene.update(dt)` steps. `dynamic-subclass`
/// rebuilds its scene inside every timed repetition (construction is part
/// of the class's declared workload); the other classes build once.
///
/// # Errors
/// Returns a string describing any interpreter, bridge, or workload
/// construction failure before or during measurement.
pub fn measure(
    class: Pg8Class,
    repetitions: usize,
    warmup: usize,
    frames: usize,
    mobjects: usize,
    dt: f64,
) -> Result<HarnessRun, String> {
    with_owned_python_module(|py, _module, globals| {
        let source = std::ffi::CString::new(format!("{}{}", PYTHON_PRELUDE, class.source()))
            .map_err(|_| "workload source contains NUL".to_owned())?;
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .map_err(|error| format!("{} workload: {error}", class.name()))?;
        let build = globals
            .get_item("build")
            .and_then(|item| {
                item.ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("workload defines no build")
                })
            })
            .map_err(|error| error.to_string())?;
        let state_bytes = globals
            .get_item("state_bytes")
            .and_then(|item| {
                item.ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        "workload defines no state_bytes",
                    )
                })
            })
            .map_err(|error| error.to_string())?;

        let mut twin = (class == Pg8Class::NativeBuiltins)
            .then(|| RustTwin::build(mobjects))
            .transpose()?;

        // Build once for the classes whose construction is not timed.
        let mut scene: Option<Bound<'_, PyAny>> = None;
        let mut handles: Option<Bound<'_, PyAny>> = None;
        if !class.rebuilds_per_repetition() {
            let built = build
                .call1((mobjects,))
                .map_err(|error| format!("{} build: {error}", class.name()))?;
            if class == Pg8Class::NativeBuiltins {
                let pair = built
                    .extract::<(Bound<'_, PyAny>, Bound<'_, PyAny>)>()
                    .map_err(|error| error.to_string())?;
                scene = Some(pair.0);
                handles = Some(pair.1);
            } else {
                scene = Some(built);
            }
        }

        let total = warmup + repetitions;
        let mut observations = Vec::with_capacity(repetitions);
        for repetition in 0..total {
            let bridge_start = Instant::now();
            if class.rebuilds_per_repetition() {
                scene = Some(
                    build
                        .call1((mobjects,))
                        .map_err(|error| format!("{} rebuild: {error}", class.name()))?,
                );
            }
            let active = scene
                .as_ref()
                .ok_or_else(|| "workload scene missing".to_owned())?;
            for _ in 0..frames {
                active
                    .call_method1("update", (dt,))
                    .map_err(|error| format!("{} update: {error}", class.name()))?;
            }
            let bridge_elapsed = bridge_start.elapsed();
            let reference_elapsed = twin.as_mut().map(|twin| {
                let reference_start = Instant::now();
                twin.step(dt, frames);
                reference_start.elapsed()
            });
            if repetition >= warmup {
                observations.push(observation(bridge_elapsed, reference_elapsed));
            }
        }

        let active = scene
            .as_ref()
            .ok_or_else(|| "workload scene missing".to_owned())?;
        let result_state: Vec<u8> = state_bytes
            .call1((active,))
            .and_then(|bytes| bytes.extract())
            .map_err(|error| format!("{} state: {error}", class.name()))?;
        let reference_state = twin.as_ref().map(RustTwin::state_bytes);
        drop(handles);
        Ok(HarnessRun {
            repetitions: observations,
            result_state,
            reference_state,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_names_match_the_policy_catalog() {
        let names: Vec<&str> = Pg8Class::ALL.iter().map(|class| class.name()).collect();
        assert_eq!(
            names,
            [
                "native-builtins",
                "per-frame-callback",
                "point-transform-callback",
                "dynamic-subclass"
            ]
        );
    }

    #[test]
    fn rust_twin_runs_the_declared_shift() {
        let mut twin = RustTwin::build(2).expect("twin");
        twin.step(0.5, 2);
        let state = twin.state_bytes();
        assert!(!state.is_empty());
        // After two 0.5 s frames at velocity 0.1, record 0 lane x is 1.0.
        let lane = f32::from_le_bytes([state[0], state[1], state[2], state[3]]);
        let expected = (0.1 * 0.5 + 0.1 * 0.5) as f32;
        assert_eq!(lane, expected);
    }

    #[test]
    fn zero_elapsed_is_an_invalid_observation() {
        let sample = observation(std::time::Duration::ZERO, None);
        assert!(sample.invalid_reason.is_some());
        let sample = observation(std::time::Duration::from_nanos(7), None);
        assert!(sample.invalid_reason.is_none());
    }

    #[test]
    fn failed_workloads_release_cached_class_owners_before_thread_exit() {
        for _ in 0..2 {
            std::thread::spawn(|| {
                let error = with_owned_python_module(|py, _module, globals| {
                    py.run(
                        c"from manimlib import Mobject\nclass Owned(Mobject):\n    pass\nOwned.retained = Owned()\nOwned.nested = {'objects': [Owned()]}\n",
                        Some(globals),
                        Some(globals),
                    )
                    .map_err(|error| error.to_string())?;
                    assert!(crate::method_cache::stats().entries > 0);
                    py.run(
                        c"raise RuntimeError('authored owned workload failed')",
                        Some(globals),
                        Some(globals),
                    )
                    .map_err(|error| error.to_string())
                })
                .expect_err("authored error must survive teardown");
                assert_eq!(error, "RuntimeError: authored owned workload failed");
                assert_eq!(crate::method_cache::stats().entries, 0);
            })
            .join()
            .expect("failing workload owner thread");
        }
    }

    #[test]
    fn repeated_measurements_teardown_before_the_owner_thread_changes() {
        for class in Pg8Class::ALL {
            let first = std::thread::spawn(move || measure(class, 1, 0, 1, 1, 1.0 / 30.0))
                .join()
                .expect("first owner thread")
                .expect("first measurement");
            let second = std::thread::spawn(move || measure(class, 1, 0, 1, 1, 1.0 / 30.0))
                .join()
                .expect("second owner thread")
                .expect("second measurement");
            assert_eq!(first.result_state, second.result_state, "{}", class.name());
        }
    }
}
