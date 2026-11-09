# G0-5 — Python extensibility: ratification note (fm-87q)

**Status:** Ratified, 2026-07-23. The executable form is
`spikes/g0-5-python-ext` (a non-member PyO3 cdylib over the G0-1 arena;
29-test Python suite green via `run.sh`; exact PyO3 pin in the spike's own
committed lockfile under ADR-0003's dev tier). **fmn-python (W10, fm-aqv)
implements the production bridge from this note**, not from the spike's
internals. Risk R10's pivot rule is served: the gap list below is the tier-in
input for the Parity Ledger *before* G4a.

## Security migration revalidation (fm-kg6g, 2026-08-11)

The original ratification and crossing-cost measurements below used PyO3
0.26.0. fm-kg6g advances both the maintained spike and production fmn-python
portal to exact PyO3 0.29.2, keeps `abi3` off, and declares
`#[pymodule(gil_used = true)]`; no free-threaded module claim is implied.
Extension linkage now comes from the process-scoped
`PYO3_BUILD_EXTENSION_MODULE=1` build setting rather than a unifiable Cargo
feature. The 29-test suite is re-run unchanged against that graph, so its
subclass/MRO, proxy identity, weakref, copy, live-view, detach, and exception
claims remain executable rather than documentary.

The ns/call table remains the historical PG-8 seed and was not silently
rebaselined onto different dependency bits or host conditions. Current
performance evidence stays in the canonical PG-8 bundles and is interpreted
under their own producer identity and comparability rules.

## Proven claims

1. **Real subclassing works against the Rust engine.** A Python subclass
   of the bridge base overrides `init_data` / `init_points` /
   `init_uniforms` / `interpolate`, and the **engine** dispatches those
   calls through ordinary Python MRO — mixins ahead of the bridge base in
   the MRO win, cooperative `super()` chains reach the Rust defaults, and
   the engine-defined call order (`init_data → init_points →
   init_uniforms`) is observable from Python. This is dynamic dispatch,
   not static binding.
2. **The proxy identity story (G0-1 A2) holds end-to-end.** A proxy holds
   one engine pin for its lifetime (`Drop` releases it); engine-side
   `delete()` defers under the pin and finalizes when the proxy is
   collected; identity, `hash`, and `weakref` behave as for any Python
   object; scene round-trips never change the proxy or its handle.
3. **`__dict__` participates in manim copy semantics.** `copy()` performs
   the engine family copy (data deep, edges remapped, updaters shared by
   reference), then applies the Reference's `Mobject.copy` rule to every
   mirrored proxy's `__dict__`: family-member-valued attributes remap to
   the copy's corresponding member, family-external mobjects and plain
   values stay shared. Attributes set before `_engine_init` survive.
4. **Subclass-declared `data_dtype` flows through the schema machinery.**
   A class attribute like `[("point", 3), ("rgba", 4), ("wobble", 2)]` is
   read off the type (MRO-resolved) by the engine seam and becomes the
   RecordBuffer schema; bad declarations are precise `ValueError`s.
5. **The §8.2 view protocol crosses the boundary intact.** Live views
   alias engine memory both ways, bump the revision on write, refuse
   writes when exported read-only, and detach on resize while keeping the
   old generation readable — exactly the G0-1 ratified semantics,
   observed from Python.
6. **Exceptions map both directions.** Python exceptions raised inside
   engine-driven callbacks (lifecycle inits, `interpolate` mid-transform,
   updaters mid-update) propagate out of the engine loop as themselves,
   message intact. Engine-side failures arrive as typed exceptions:
   `StaleHandleError` and `ForeignStageError`, real `RuntimeError`
   subclasses (the two-scene policy is a *checked error* from Python).

## The reentrancy law (the question G0-1 deferred here)

**The engine must never hold a state borrow across a Python callback.**
Every callback window — lifecycle inits, per-frame `interpolate`,
updaters — dispatches with the engine lock released, so the callback may
re-enter any engine API (the suite proves an updater mutating its own
records mid-update). Consequences for production:

- Choreo's updater/callback dispatch collects its worklist, drops the
  stage borrow, then calls out; callables re-enter through the same
  public seam as everyone else.
- The G0-1 updater signature (`FnMut(&mut Stage, Mob, dt)`) is fine for
  engine-native updaters but is NOT the shape Python callables can use
  (they would need the borrow the law forbids). fmn-python keeps a
  bridge-side registry (as the spike does) or W3 adds a re-entrant
  dispatch context; either way the law is the invariant, the mechanism is
  free.

## Measured crossing costs (PG-8 seed numbers)

Linux x86-64, Python 3.13.7, pyo3 0.26.0 (`abi3` off), release build,
medians over 7 batches (`py/bench_crossing.py`):

| Crossing | ns/call |
|---|---|
| py→rust no-op method (the floor) | **47** |
| py→rust `get_field` (3 lanes, entry resolution + copy-out) | 236 |
| py→rust `set_field` (3 lanes) | 178 |
| live-view write (3 lanes, no entry resolution) | 141 |
| live-view read (3 lanes) | 156 |
| rust→py updater dispatch (per `update`, 1 updater) | 325 |
| rust→py `interpolate` dispatch (per step, no-op override) | 408 |

Reading for §15.2's binding-tax program: the 47 ns floor means budget
pressure comes from *entry resolution + conversion*, not the FFI itself;
views beat method calls by ~⅓ (no handle resolution) — the NumPy
zero-copy path will beat both; a per-frame Python callback costs ~0.3–0.4
µs before user code, so PG-8's per-frame-callback class can budget ~1 µs
per crossing with headroom.

## Gap list (tier into the Parity Ledger before G4a — R10)

| # | Gap | Disposition |
|---|---|---|
| G1 | **No custom `tp_init` in pyo3**: engine-driven construction runs via the manimlib layer's pure-Python `__init__` calling `_engine_init` (the spike's `Mobject` base). | Sanctioned shim shape for fmn-python's `manimlib` layer; zero user-visible surface. |
| G2 | **NumPy structured views untested here** — the numpy package is outside the spike's closure. The spike proves the aliasing/lifetime semantics; the buffer-protocol skin is W10. | fmn-python maps `RecordView`'s `Arc` pinning onto buffer-protocol lifetimes (G0-1 §8.2 note anticipates exactly this). |
| G3 | **`FieldSpec` holds `&'static str`** — the spike leaks Python-declared dtype names. | W3's production `RecordSchema` takes owned/interned names (also needed for schema iteration, which the spike added as `field_names()`). |
| G4 | **`copy`/`deepcopy`/pickle protocols** (`__copy__`, `__deepcopy__`, `__reduce__`) not wired — the spike proves the remap *rule* via `copy()`. | fmn-python wires the protocols onto the same engine copy path (product surface, §15.2). |
| G5 | **Send/threading story**: every pyclass is `unsendable`; the scene worker is single-threaded by design. GIL-release windows for the frame pipeline are unmeasured. | W10 measures GIL windows against the §10.5 frame pipeline; the reentrancy law above is the constraint any design must satisfy. |
| G6 | **Sub-interpreter / free-threaded CPython** unprobed. | Out of scope for the compatibility claim; revisit trigger: CPython 3.14 free-threading adoption in the wild. |

Nothing in the gap list breaks a Reference-surface promise: every gap is
an implementation seam, not a semantics loss — no Parity-Ledger demotion
required, only the G1/G2/G4 wiring notes above.

## Governance record

- The spike is a **non-member crate** with its own committed `Cargo.lock`
  (the ADR-0003 fuzz-crate pattern). Its exact PyO3 0.29.2 graph is checked
  against the same reviewed allowlist records consumed by the shipped
  fmn-python crate, but the spike itself remains `class=dev` and can never
  reach a shipped artifact. ADR-0015 owns the production version, ABI,
  feature, and unsafe-boundary decision.
- `governed_closure` gained the ADR-0003 refinement (`audit_with_aux`):
  it now walks committed non-member lockfiles (this spike's, and
  `fuzz/Cargo.lock` when fm-ntp lands) under uniform admission/checksum
  rules; a consumed row is stale only if absent from every governed lock.
- G0-1 additions consumed by the bridge (additive, ratified semantics
  untouched): `RecordSchema::field_names()`, `Mob::token()`.
- The spike build is local by necessity (the cdylib links the host
  CPython; remote build workers refuse it at preflight) — `run.sh` sets
  `RCH_CARGO_WRAPPER_BYPASS=1` for exactly this build.
