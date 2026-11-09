# BN-23: First-error execution for native-backed mathematical plots

**Status:** Draft.

ParametricCurve, FunctionGraph and ImplicitFunction delegate geometry to Atlas
and Chisel. Their legacy construction bridges retained the first Python error
but continued executing authored functions during the rest of native sampling.
On small requests this meant six calls instead of one for a curve/graph, and
twenty-five for an implicit field whose first evaluation already raised.

## New construction boundary

The ordinary Python constructors and Mobject native-builder dispatch now share
the first-error guard also used by surface construction. An authored exception,
including cancellation via KeyboardInterrupt or SystemExit, or a result-conversion
exception stops all further authored callback execution in that construction.
The same exception object crosses the native bridge unchanged. A nested build
gets an independent guard. Successful functions retain their identity in graph
metadata and remain available for ordinary graph queries and live bindings.

Range and discontinuity iterables are bounded before eager Python materialization
or conversion to native vectors. A curve range consumes at most four entries to
admit the normal two/three-value forms, and an implicit domain at most three to
admit its two bounds. Discontinuities consume at most 65,537 entries before
rejecting requests over the existing 65,536-entry limit. Native aggregate sample,
partition, isoline depth and evaluation budgets remain authoritative.

A ParametricCurve callback returns exactly three finite real f32-representable
coordinates. FunctionGraph returns a finite real f32-representable scalar; explicit
discontinuities are the way to exclude its undefined locations. Numeric conversion
protocols remain supported for scalar samples, and their first errors are retained;
text and complex values are not silently coerced into real coordinates.

**Implicit-field non-finite values have a different meaning.** NaN and infinity
mark undefined regions to the native contourer and remain supported. Field values
are not point records, so they are not subjected to a surface/curve f32 restriction.
The native undefined-region subdivision, evaluation order and extracted geometry
are unchanged. A raised exception is still an error, not an undefined sample.

Successful native sample order, endpoint inclusion, smoothing and discontinuity
partitioning remain unchanged. In particular, Atlas's finite non-positive-step
endpoint-only policy is preserved; this adapter does not replace it with a
positive-step-only rule. The user function is never probed to guess its type,
vectorization capability, output dimensions or purity.

## Native propagation, not sentinel completion

The Rust owners now accept fallible, borrowed callbacks directly:

* `fmn_library::graphs::ParametricCurveSpec::try_sample` samples curves (and
  scalar graphs expressed as `[x, f(x), 0]`) through the same admitted ranges and
  path kernel as the existing owned `ParametricCurve` and `FunctionGraph` APIs.
* `fmn_geom::try_plot_isoline` and `try_plot_isoline_with_stats` propagate a
  field error through quadtree construction, adaptive refinement, conforming
  triangulation and every crossing search. The existing infallible entry points
  delegate to the same implementation; no alternative contourer is introduced.
* `fmn_library::graphs::ImplicitFunctionSpec::try_sample` materializes these
  contours through the existing Atlas path and true-spline smoothing owner.

`GraphError<E>` and `IsolineError<E>` retain the original error value without
cloning it, requiring a boxed callback, or imposing `'static` on borrowed data.
No further sample, path smoothing, segment chaining, or partial-object publication
follows a callback failure. When extraction succeeds, memoization, evaluation
statistics, sample/contour ordering, and native arithmetic remain unchanged.
Stateful callbacks are useful for cancellation and diagnostics; deterministic
geometry still requires a stable value for each sampled coordinate. NaN and
infinity returned normally by an implicit field keep their undefined-region
meaning, while an `Err` aborts extraction.

The raw Python graph methods now call those fallible APIs. They no longer retain
an error in a side channel and finish native work using zero/NaN sentinels. A raw
parametric point must have exactly three components, and sampled/smoothed curve
points must be finite and f32-representable before becoming native records.

## Ownership and limits

Scene-bound objects cannot be reconstructed with __init__ or a construction seam;
use live binding or become. Rejected calls leave the existing scene owner, record
generation and exported views intact. Failed detached native builder calls do not
publish replacement geometry. Authored side effects made before failure are not
rolled back, nor is arbitrary unrelated host state transactionally restored.

Public Python iterable conversion and native sampling admission remain separate
layers. The native callback result channel is synchronous: it does not preempt a
callback that never returns, reverse authored side effects, or catch arbitrary
Rust panics. Native allocation and evaluation budgets remain in force; this does
not introduce an allocator-wide out-of-memory recovery mechanism. Finite point
validation is not a proof that all later renderer arithmetic is representable.

## Tests

`crates/fmn-python/tests/graph_admission.py` uses the actual compiled native module.
It covers first/middle-sample failures, cancellation, conversion protocols,
implicit undefined regions, bounded generators, native budget precedence,
discontinuities, endpoint policies, ownership, independent nested construction,
callable lifetime and idempotent installation. Successful points and animated Y4M
are compared with the original native builders at one and four threads.

`crates/fmn-library/tests/fallible_graph_sampling.rs` injects an error at every
evaluation of a nontrivial implicit extraction and verifies the exact successful
prefix, including refinement and crossing-search samples. It also checks curve
discontinuity order, admission precedence, non-clone errors, explicit budgets,
and existing owned-builder equivalence. `native_graph_sampling.py` exercises raw
PyO3 descriptors rather than the host guard, including cancellation, conversion
errors, nested construction, target/view preservation and an independently
authored line-animation Y4M oracle at one and four threads.
