# Live implicit contours

`ImplicitFunction.init_points()` rebuilds the zero set of its current `func`.
The native Chisel contour extractor and path builder still own all sampling,
topology, smoothing and rendering. Refresh does not replace the scene object,
create an extra clock or install an updater implicitly.

```python
from manimlib import *

class MovingLevelSet(Scene):
    def construct(self):
        level = ValueTracker(-0.5)
        curve = ImplicitFunction(lambda x, y: y-level.get_value(),
                                 x_range=(-2, 2), y_range=(-1, 1),
                                 min_depth=3, max_quads=128, color=BLUE)
        curve.add_updater(lambda obj: obj.init_points(), call=False)
        self.add(curve)
        self.play(level.animate.set_value(0.5), run_time=2)
```

## What changes and what stays owned

Each refresh reads `func`, `x_range`, `y_range`, `min_depth`, `max_quads`, and
`use_smoothing` anew. Connected components may split, disappear, or return.
Scene membership, children, updaters, style, uniforms and saved states retain
their owners. No scene time advances from calling `init_points()` alone.

The existing native `set_points` protocol publishes the new points. Views follow
same-size record updates; resized generations detach old views. Style defaults
remain meaningful when a contour is empty, including changes made before it
becomes visible again. Per-record styles use the existing record-resizing rules;
there is no promised geometric correspondence between an old and new component.

Coordinates are those returned by the current implicit recipe's scene domain.
Earlier shifts, rotations or other affine edits of the extracted curve are not
reapplied on refresh. Encode such mappings in `func` and its domain as needed.
NaN and infinity keep their existing meaning: undefined field regions, not
invalid surface-point records. The extractor's normal subdivision and evaluation
budgets apply; this method does not turn sampled extraction into an exact solver.

## Failure and animation boundaries

The full candidate is extracted before target publication. Invalid controls and
callback errors leave prior geometry intact, and the original callback exception
is preserved. A failed refresh does not poison later refreshes.

Refresh refuses recursive updates and active direct animations of the same
contour. Animate a tracker used by the field, rather than combining a direct
`Transform(curve, ...)` with curve regeneration. If authored sampling changes the
target's recipe, records, family or scene owner, the candidate is discarded.
Arbitrary authored side effects are not undone. Callbacks must return normally
or raise; this synchronous method cannot preempt a callback that never returns.

`demo/python/live_implicit.py` shows a contour splitting and rejoining. The
native-backed acceptance suite `crates/fmn-python/tests/live_implicit.py` covers
topology, ownership, live views, failure recovery and independent world-coordinate
Y4M equivalence at one and four rendering threads.
