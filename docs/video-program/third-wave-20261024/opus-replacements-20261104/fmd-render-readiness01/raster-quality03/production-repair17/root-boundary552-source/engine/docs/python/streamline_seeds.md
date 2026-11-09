# Authored streamline seeds and live native integration

`StreamLines.get_sample_coords()` returns a fresh NumPy array with one row per
seed and one column per coordinate-system dimension. The default implementation
replays Atlas's named deterministic RNG substream: querying it does not advance
NumPy randomness or alter the draw offset used by `AnimatedStreamLines`.
The current detached portal construction seed remains zero; this is not new
scene-specific seed or cross-platform certification plumbing.

Override `get_sample_coords` to choose explicit coordinates. The hook is called
on the actual authored instance, once per `draw_lines()` operation, including
construction. Arrays, finite lists and bounded generators are supported. Explicit
seeds retain caller ordering and introduce no implicit jitter; an override that
calls `super()` retains that sampler's native draw accounting.

```python
class MyLines(StreamLines):
    def get_sample_coords(self):
        return [[-1, 0], [0, 0], [1, 0]]

lines = MyLines(field, axes, noise_factor=0)
self.add(lines)
lines.func = changed_field
lines.draw_lines()
```

Each rebuild captures the controls, obtains and validates seeds, integrates and
styles a detached candidate, and then uses the existing native family-replacement
operation. The container, scene membership and root updaters retain their
identities. Individual line children are replaced. Existing exported point views
continue to refer to their old native buffers, not the new paths. Per-line
`virtual_time` and RNG metadata change only with successful adoption.

Two-dimensional fields receive two-dimensional rows; three-dimensional fields
receive three-dimensional rows. The existing adaptive RK45 solver, quadratic
smoothing, true-arc re-spacing and rational scene clock remain authoritative.
Dense samples retain the existing half-open `[0, solution_time)` convention:
`solution_time=.5, dt=.125` samples through `.375`. `arc_len` remains a true-length
cap. No Python ODE solver or second RNG is used.

Malformed/nonfinite seeds, exceeded budgets, invalid controls, failed field or
paint callbacks, reentrant edits and callback changes to the displayed family
refuse before replacement. The original callback exception is retained; the
native bridge stops calling a poisoned callback. Authored external side effects
are not rolled back. Finish animations, release updater suspension, and clear an
`AnimatedStreamLines` controller before replacing its source children.

The live paint adapter uses the native arclength width-profile function. Record
correctness does not establish full pixel parity for arbitrary interior stroke
profiles: the existing endpoint-only style projection is a separate renderer
limitation. The demonstration `demo/python/seeded_streamlines.py` uses uniform
widths and colors and exercises changing actual integrated paths.

The acceptance suite is `crates/fmn-python/tests/live_streamlines.py`. It checks
analytic constant-field endpoints, hook identity, native RNG offsets, failed
rebuild preservation and actual PNG frames across one/four renderer threads.
