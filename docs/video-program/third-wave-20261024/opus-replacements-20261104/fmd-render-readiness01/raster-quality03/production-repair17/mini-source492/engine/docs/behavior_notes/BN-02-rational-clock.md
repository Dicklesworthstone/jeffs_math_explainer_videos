# BN-02 — The rational clock: drift-free time, exact frame counts

**Status:** Final (W4, fm-wuq). The six-step frame order landed (fm-x79),
and G1 (fm-o3j) passed on 2026-08-20. The migration table was checked
against real behaviour on 2026-10-04:
- the Reference's `np.arange(0, d, 1/fps)` gives 3, 6, 30, 10, 90 and 15
  frames for the rows below;
- a portal `wait(0.1)` publishes 4 frames at 30 fps and 7 at 60;
- `wait(2.9999)` publishes 90 and 180, equal to the Reference.

## What changed

Classic manim advances a float accumulator over
`np.arange(0, run_time, 1/fps) + 1/fps`. Two consequences:

1. **Drift.** `1/30` is not representable; an hour at 30 fps accumulates
   error visibly, and the same scene disagrees with itself under different
   play/wait chunkings.
2. **Float-boundary frame counts.** `arange`'s length depends on float
   division rounding, so the number of emitted frames for a duration is an
   artifact of the float pipeline, not of the duration.

FrankenManim's clock is `(frame_index, fps)` — a `RationalTime` whose
value is exactly `frames / fps`. Time *derives*; it never accumulates.
A million frames at 30 fps is exactly `1_000_000/30` seconds, bit-equal
to the closed form (locked by test).

The segment frame count is `n = ceil(run_time · fps)`, computed on the
**exact rational value of the received f64 duration**:
`(n−1)/fps < run_time ≤ n/fps`, exactly (locked by a 20 000-duration
property test).

## What deliberately did NOT change

The Reference's emission semantics are kept verbatim as API behavior:

- samples are `t_k = k/fps`, `k = 1..=n` — no alpha-zero frame in the
  emission sequence (`begin()` interpolates at zero separately);
- upward duration rounding: the final sample may exceed `run_time`;
- alpha = `t/run_time`, clamped to `[0, 1]` (the Reference's
  `interpolate` clip);
- skipped playback advances the whole segment in one step.

Adaptive or variable frame sampling remains **permanently refused**
(D-18): `RationalTime` is only constructible as whole frames over fps —
there is no API for an off-grid sample.

## Migration guidance

Frame counts can differ **by one** from the Python engine exactly where
binary f64 representation puts the requested duration off its decimal
intent:

| duration | fps | Python frames | FrankenManim | why |
|---|---|---|---|---|
| `0.1` | 30 | 3 | 4 | f64(0.1) > 1/10; three frames end at 0.1 exactly, which is *less* than the requested duration |
| `0.1` | 60 | 6 | 7 | same excess |
| `1.0`, `1/3`, `2.9999`, `0.5`, whole numbers… | any | equal | equal | representable-boundary cases agree |

If a scene needs the old count, request a grid-exact duration
(`n/fps`). Everything downstream gains: `wait()` chains of any length
hold A/V sync exactly, and a scene's timing is identical however its
plays are chunked.

## Evidence

- `crates/fmn-anim/src/clock.rs`; `crates/fmn-anim/tests/clock.rs`
  (count fixtures cross-checked against Python `fractions.Fraction` and
  the Reference's `arange` output, drift test, coverage property).
- Reference: `manimlib/scene/scene.py::get_time_progression` at the
  pinned commit.
- The corpus differential counts each Reference segment by this rule
  (`time_bn02` in `structural_facts.py`); a scene whose portal clock equals
  that count differs only under this note (exclusion row
  `bn02-frame-count`). `UniformSamples` (`_2023/convolutions2/continuous.py`)
  waits 0.1 s a hundred times: 35.000 s in the Reference, 38.333 s here.

## Native tracing cadence and time windows

Native `TracedPath` and `TracingTail`, including the Python portal's
already-bound tail source, measure history in elapsed seconds.
`time_per_anchor` spaces observations on a regular temporal grid; the
moving `time_traced` boundary and the current endpoint interpolate between
those observations. Smoothing and the true-arc-length stroke and opacity
tapers still operate on that path. Scene frame sampling and the rational
clock are unchanged.

The Reference chooses a recent-point count from the latest `dt`, so a
change of update size changes the duration represented by older samples.
The native tracers retain the configured time window under tiny,
irregular, and large positive updates. Their storage and sampling work are
bounded by the window/cadence ratio, including the predecessor needed to
clip the boundary. Expired samples within a large update are skipped;
they are never allocated. Configurations exceeding the shared 100,000
anchor budget fail before stage adoption.

`TracedPath` begins at its first positive-dt observation. Construction and
zero-dt updates neither call its source function nor invent earlier
history. Its default infinite duration retains all regular grid samples
up to the same anchor budget. A step that would exceed that budget fails
before changing the stored history or geometry; the caller can choose a
finite window or a larger `time_per_anchor` for a longer recording.

`TracingTail` instead seeds stationary history at the source's current
center while keeping public geometry empty. Zero-dt updates leave it
empty; the first positive update publishes the seeded path. This preserves
the Reference's construction lifecycle while correcting the previous
native eager path.

**Migration:** use `time_per_anchor` to choose temporal detail and
`time_traced` for duration. Native `TracedPath` previously stored
`time_per_anchor` without applying it; direct point inspection now reflects
that cadence instead of one anchor per update. Code that inspects native
trace points directly after construction should advance by the intended
positive scene step first. A caller supplying negative or non-finite
native updater deltas, or non-finite source coordinates, gets an explicit
panic before trace history or geometry changes. The infallible
`Stage::update` API does not roll back its own clock or arbitrary source
callback effects.

The native regressions are in `crates/fmn-library/src/fields.rs`; the
bound-source portal cases are in `crates/fmn-python/tests/bridge.py` and
`crates/fmn-python/tests/detached_tracing_tail.py`. The registered
`lifecycle.tracing_tail_empty_until_update.v1` scenario covers stage
adoption, zero-dt behavior, seed/endpoints, tapers, and stateful scheduling.
`lifecycle.traced_path_cadence.v1` drives the native timeline and renderer,
checking first observation, regular anchors, the exact retained window,
tapers, stateful scheduling, and visible output.
