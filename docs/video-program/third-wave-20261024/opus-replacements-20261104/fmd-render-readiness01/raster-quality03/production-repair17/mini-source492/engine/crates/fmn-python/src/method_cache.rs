//! fm-zoi method-resolution cache: resolve each `(type, method)` once,
//! reuse the callable, invalidate on class mutation (§15.2 Rev 4).
//!
//! Invalidation mechanism — the boring one CPython itself uses. Every type
//! carries `tp_version_tag`; `PyType_Modified` (invoked on any class
//! mutation, including monkey-patching a method) zeroes the tag and
//! recursively invalidates every subclass, so the *leaf* type's tag covers
//! MRO mutation of bases. An entry is valid iff the leaf type's current tag
//! equals the tag recorded at resolution. Tag `0` is CPython's "invalid"
//! sentinel and never matches, so a mid-mutation read degrades to a
//! re-resolve, never a stale hit. A tag can only collide after exactly 2³²
//! intervening type mutations on a recycled type object — the same residual
//! risk CPython accepts for its own caches; the cache holds a strong
//! reference to the type, so the keyed pointer can never dangle or be
//! recycled while an entry lives.
//!
//! Correctness envelope (documented semantics):
//! - Resolution goes through `type.__getattribute__` (`PyType::getattr`), so
//!   the cached object is exactly what an unbound class attribute lookup
//!   yields — for plain functions and PyO3 methods, calling it with the
//!   instance as first argument is CPython's unbound-call semantics.
//! - Instance-`__dict__` shadowing is honored: if the instance carries the
//!   name in its own dict, the caller falls back to the ordinary
//!   `call_method` path. The cached path is for instance-method dispatch
//!   only (not staticmethod/classmethod contracts).
//!
//! The single `unsafe` read of `tp_version_tag` is the third ratified
//! project-authored unsafe item in this crate (ADR-0015, Amendment 1): a
//! read-only observation of CPython's own invalidation counter through
//! pyo3-ffi's non-limited-API `PyTypeObject` layout. It publishes no
//! pointer, extends no lifetime, and mutates nothing.
//!
//! Storage is thread-local: scene workers are single-threaded by ADR-0015,
//! so no synchronization exists on the hit path.

use std::cell::RefCell;
use std::collections::HashMap;

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};

/// One resolved `(type, method)` pair plus the tag it was resolved under.
struct CacheEntry {
    version_tag: u32,
    callable: Py<PyAny>,
    /// Keeps the keyed type pointer valid for the entry's lifetime.
    _owner: Py<PyType>,
}

#[derive(Default)]
struct Cache {
    entries: HashMap<(usize, &'static str), CacheEntry>,
    hits: u64,
    misses: u64,
    invalidations: u64,
}

thread_local! {
    static CACHE: RefCell<Cache> = RefCell::new(Cache::default());
}

/// Read-only observer statistics for tests and the detection report.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MethodCacheStats {
    pub entries: u64,
    pub hits: u64,
    pub misses: u64,
    pub invalidations: u64,
}

/// The leaf type's current version tag; `0` means "invalid" per CPython.
fn type_version_tag(ty: &Bound<'_, PyType>) -> u32 {
    // SAFETY: `ty` is a live `PyType`, so its pointer addresses a valid
    // `PyTypeObject` for the duration of the borrow. ADR-0015 pins abi3 off,
    // so pyo3-ffi's full `#[repr(C)]` PyTypeObject layout is the interpreter
    // ABI; this reads one scalar field and mutates nothing.
    unsafe { (*ty.as_type_ptr()).tp_version_tag }
}

/// Resolve `(type, name)` through the cache. A miss or tag mismatch
/// re-resolves through the full Python attribute machinery.
fn resolve(ty: &Bound<'_, PyType>, name: &'static str) -> PyResult<Py<PyAny>> {
    let key = (ty.as_type_ptr() as usize, name);
    let tag = type_version_tag(ty);
    let cached = CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let validity = cache
            .entries
            .get(&key)
            .map(|entry| entry.version_tag == tag && tag != 0);
        match validity {
            Some(true) => {
                cache.hits += 1;
                cache
                    .entries
                    .get(&key)
                    .map(|entry| entry.callable.clone_ref(ty.py()))
            }
            Some(false) => {
                cache.invalidations += 1;
                cache.entries.remove(&key);
                None
            }
            None => None,
        }
    });
    if let Some(callable) = cached {
        return Ok(callable);
    }
    let callable = ty.getattr(name)?.unbind();
    // Re-read after resolution: the lookup itself can assign a fresh tag to
    // a type that was mid-mutation. A still-zero tag is stored as-is and
    // simply never hits — correct, just uncached.
    let tag = type_version_tag(ty);
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.misses += 1;
        cache.entries.insert(
            key,
            CacheEntry {
                version_tag: tag,
                callable: callable.clone_ref(ty.py()),
                _owner: ty.clone().unbind(),
            },
        );
    });
    Ok(callable)
}

/// Whether the instance's own `__dict__` shadows `name` (rare path; falls
/// back to ordinary `call_method` to preserve exact attribute semantics).
fn instance_shadows(obj: &Bound<'_, PyAny>, name: &str) -> bool {
    let Ok(dict) = obj.getattr(intern!(obj.py(), "__dict__")) else {
        return false;
    };
    let Ok(dict) = dict.cast_into::<PyDict>() else {
        return false;
    };
    dict.contains(name).unwrap_or(false)
}

/// Call the cached instance method `obj.name()` with no arguments.
pub fn call_cached0<'py>(
    obj: &Bound<'py, PyAny>,
    name: &'static str,
) -> PyResult<Bound<'py, PyAny>> {
    if instance_shadows(obj, name) {
        return obj.call_method0(name);
    }
    let callable = resolve(&obj.get_type(), name)?;
    callable.bind(obj.py()).call1((obj,))
}

/// Call the cached instance method `obj.name(*args)`.
pub fn call_cached1<'py>(
    obj: &Bound<'py, PyAny>,
    name: &'static str,
    args: &Bound<'py, PyTuple>,
) -> PyResult<Bound<'py, PyAny>> {
    if instance_shadows(obj, name) {
        return obj.call_method1(name, args);
    }
    let callable = resolve(&obj.get_type(), name)?;
    let mut full = Vec::with_capacity(args.len() + 1);
    full.push(obj.clone());
    full.extend(args.iter());
    callable.bind(obj.py()).call1(PyTuple::new(obj.py(), full)?)
}

/// Call a cached `staticmethod` on `obj`'s type: `type(obj).name(*args)`.
/// Used for engine machinery functions (no `self`, no instance shadowing).
pub fn call_static_cached1<'py>(
    obj: &Bound<'py, PyAny>,
    name: &'static str,
    args: &Bound<'py, PyTuple>,
) -> PyResult<Bound<'py, PyAny>> {
    let callable = resolve(&obj.get_type(), name)?;
    callable.bind(obj.py()).call1(args)
}

/// Observer statistics for this thread's cache.
#[must_use]
pub fn stats() -> MethodCacheStats {
    CACHE.with(|cache| {
        let cache = cache.borrow();
        MethodCacheStats {
            entries: cache.entries.len() as u64,
            hits: cache.hits,
            misses: cache.misses,
            invalidations: cache.invalidations,
        }
    })
}

/// Zero the observer counters (entries stay cached; correctness never
/// depends on the counters).
pub fn reset_stats() {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.hits = 0;
        cache.misses = 0;
        cache.invalidations = 0;
    });
}

/// Release this embedding worker's cached Python owners while its thread is
/// still attached. ADR-0015's strong type references remain intact throughout
/// execution; they must not outlive the isolated worker's module teardown.
pub(crate) fn clear_for_worker_teardown(_py: Python<'_>) {
    let entries = CACHE.with(|cache| std::mem::take(&mut cache.borrow_mut().entries));
    // A decref can invoke Python finalizers. Release the RefCell borrow before
    // dropping any callable or type so finalization cannot reenter that borrow.
    drop(entries);
}

/// `manimlib._method_cache_stats()`: deterministic cache observability.
#[pyfunction]
pub(crate) fn _method_cache_stats(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
    let stats = stats();
    let out = PyDict::new(py);
    out.set_item("entries", stats.entries)?;
    out.set_item("hits", stats.hits)?;
    out.set_item("misses", stats.misses)?;
    out.set_item("invalidations", stats.invalidations)?;
    Ok(out)
}

/// `manimlib._method_cache_reset()`: zero the observer counters.
#[pyfunction]
pub(crate) fn _method_cache_reset() {
    reset_stats();
}

/// `(tag(cls), tag(type(cls)))`, the same ratified read the method cache
/// uses. A class's tag changes (or reads `0`) after any mutation of it or of
/// a base; the metaclass's tag covers the metaclass MRO. The Python
/// authored-override checks key their per-class verdicts on this pair
/// (fm-xte3): a verdict is reused only while both tags are unchanged and
/// nonzero.
#[pyfunction]
pub(crate) fn _type_version_tags(cls: &Bound<'_, PyType>) -> (u32, u32) {
    (type_version_tag(cls), type_version_tag(&cls.get_type()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_start_zeroed() {
        reset_stats();
        let stats = stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.invalidations, 0);
    }
}
