# test-repair01 for complex_analysis_visual

A repair of the two test failures in build03, made from the complete
`complex-opus-rebuild-build03/command.log` against current canonical. Only code under
`#[cfg(test)]` changes, so no rendered frame can change. Nothing here has been compiled or run:
`cargo test` is the real check, and root runs it.

## Files

| File | What it is |
|---|---|
| `test-repair01.patch` | Unified patch. From the repo root: `patch -p1 -i .../test-repair01.patch` |
| `explainer_corpus/complex_analysis_visual/src/chapters/real_integral.rs`, `two_planes.rs` | The two patched files, whole |
| `baseline.json` | sha256 of the two canonical files, the build log, the files read, and the script |
| `check_test_repair.py` | The checks. `python3 -I -B check_test_repair.py REPO EMPTY_SCRATCH` |
| `check-run01.txt` | The output of those checks |

## real_integral.rs:677, `None` against `Some(1)`

The contour's geometry is right. The test was asking the wrong question of it.

`contour(r)` is `[−R, R]` followed by the 120-step half circle, so the diameter is one chord of
a 122-point path. That is a correct drawing: the scene fades the path in as a static polyline
(`real_integral.rs:246-248`), so its straight part needs only its two ends, and nothing walks
along it.

`model::winding_count` is a sampled count. It refuses any step that turns the output more than
`MAX_STEP_TURN`, a quarter turn. From the first pole, `e^(iπ/4)`, the diameter chord turns
`z − q` by 2.3856 rad against the 1.5708 allowed, so the count returns `None`. The guard is doing
its job. All six points the test asks about hit the same refusal on step 0; the loop panicked on
the first.

**The repair.** The test walks the same chords in steps of at most 1/256 with a test-only helper,
`walked`. Every sample lies on the path as drawn. `contour` itself, used by the scene, is not
touched. The largest step is then 0.3779 rad.

**Coverage.** All six original assertions stay, byte for byte. Eight rows are added just inside
and just outside the loop: below the top of the arc and above it, just above and below the
diameter, and beside both corners. Those are the rows that would notice a wrong loop, which
points this far from the edge would not.

## two_planes.rs:689, `5.4` against `5.400000000000001`

`Plane::new` computes the height as `width * (y.1 - y.0) / (x.1 - x.0)`. With the walk's window,
`5.4 * 3.0 / 3.0` rounds one unit in the last place high. The frames are the same rectangle.

**The repair.** A test helper, `same_rectangle`, keeps the centre and the width exact: they are
passed through untouched, so any difference there is a real change. Only the computed height
gets a bound, and it comes from the rounding rather than from a chosen number. Two rounded
operations put a height within `f64::EPSILON × width` of the width, so two heights are within
twice that, 2.4e-15 scene units. A 4K pixel is about 3.7e-3.

**Negative cases.** Five layouts must be told apart from the input plane, and the plane rebuilt
from its own recipe must not be: the output plane, then the centre, the width and the height each
moved by 1e-9, and a window half as tall. A bound wide enough to pass the 1e-9 height is caught as
a planted defect.

**Not done.** Writing `width * (span_y / span_x)` in `Plane::new` would make a square window's
height exact with no tolerance at all, but it changes render code used by every plane. The
assignment asked for a tolerance, so this stays in the test.

## Evidence

No Rust ran. `check_test_repair.py` mirrors both tests in Python, reading its constants and rows
from the Rust text and using the same IEEE arithmetic as `model.rs`. On canonical the mirror
reproduces both panics, and the two frame tuples match the log character for character. On the
candidate it passes, including the assertions after each panic, which no build has reached:

- `(0, 1)` and `(0, −1)` in the contour test;
- the scale equality and the `WALK_R` check in the frame test.

Thirteen planted defects are all caught, among them a flipped row, raw chords, the lower half
circle, a zero bound (the build03 failure again) and a too-wide bound. The patch regenerates,
applies and reverses, no byte before `#[cfg(test)]` changes, and rustfmt accepts the whole module
tree.

## Gaps

- **Not compiled, not run.** The mirror checks geometry and rounding, not that the Rust compiles
  or that the test binary agrees.
- **Clippy and the oracle have never run** on this crate, and no lint has seen these lines.
- The test count stays at 95 until reading-repair01 is integrated.

## Order

The patch shares no file with reading-repair01, so the two apply in either order.
