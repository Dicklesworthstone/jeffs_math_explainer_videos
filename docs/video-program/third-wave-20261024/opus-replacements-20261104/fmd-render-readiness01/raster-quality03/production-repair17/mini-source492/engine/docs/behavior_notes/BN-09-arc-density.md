# BN-09 — One arc-density rule

**Status:** Final. Drafted in W2 (fm-e3f) with the path model; applied
across the geometry lineage in W7 (fm-oab), where `Arc`, `Circle`, `Dot`,
`Ellipse`, the sectors, `ArcBetweenPoints`, and every `path_arc` line are
built through the one rule.

## What changed

Classic manim has **three mutually inconsistent conventions** for how many
quadratic components trace an arc of subtended angle θ:

| Site | Rule | Full circle | Quarter circle |
|---|---|---|---|
| `Arc.__init__` (and everything built on it) | `int(15·|θ|/TAU) + 1` | 16 | 4 |
| `VMobject.add_arc_to` | `ceil(8·|θ|/TAU)` | 8 | 2 |
| `quadratic_bezier_points_for_arc` default | fixed `8` | 8 | 8 |

The same quarter-arc is 4 components when drawn as an `Arc`, 2 when appended
with `add_arc_to`, and 8 when produced by the bare helper. Curve quality —
and point counts, which user code indexes — depend on which code path
happened to build the arc.

FrankenManim uses **one rule everywhere** (`fmn_geom::bezier::arc_n_components`):

```text
n(θ) = max(1, ceil(16·|θ|/TAU))
```

16 components for a full circle — the Reference's `Arc`/`Circle` convention,
its finest — and agreement with that convention at every common angle
(quarter → 4, half → 8, three-quarter → 12). The rule is monotone in |θ|,
never coarser than any of the three Reference conventions, and gives at
least one component to any arc, however small.

An explicit `n_components` argument is honored verbatim at every call site,
exactly as in the Reference.

## Migration guidance

- Arcs built through `Arc`-family constructors keep their Reference point
  counts at the standard angles; code indexing those points is unaffected.
- Paths built with `add_arc_to(..., n_components=None)` gain resolution
  (16/TAU density instead of 8/TAU): point counts along such paths grow.
  Pass an explicit `n_components` to reproduce old counts.
- Anything that consumed `quadratic_bezier_points_for_arc`'s fixed default
  of 8 regardless of angle now scales with the angle instead.
- An arced `Arrow` scales its point count with its stem angle, where the
  Reference's arrow always carries 43 points. A rebuild at a new length
  therefore changes the count too: `GrowArrow`'s zero-scale start copy of a
  `path_arc=PI` arrow has fewer points than the arrow. A `Transform` whose
  starting mobject no longer aligns with its mobject re-aligns the start, the
  mobject and its target copy before the first interpolation, so these
  animations run as in the Reference. Pointwise results are unchanged
  whenever the start already aligned.
- A non-finite implicit arc angle (`path_arc=NaN` or infinity) is rejected
  before geometry or the stored angle changes. The Reference can publish the
  invalid angle and clear the existing points before failing during component
  selection. Validate computed angles or pass a finite value; FrankenManim
  returns a typed error and leaves the prior line intact.

## Worked examples

| Construction | Reference | FrankenManim |
|---|---|---|
| `Circle()` | 16 components, 33 points | **same** |
| `Arc(angle=TAU/4)` | 4 components, 9 points | **same** |
| `Arc(angle=PI)` | 8, 17 | **same** |
| `Arc(angle=1.7)` | 5, 11 | **same** |
| `Arc(angle=0.13·TAU)` | `int(1.95)+1 = 2` | `ceil(2.08) = 3` — finer |
| `path.add_arc_to(p, TAU/4)` | `ceil(2) = 2` | `4` — matches `Arc` |
| `quadratic_bezier_points_for_arc(TAU/4)` | 8, regardless of angle | `4` — scales with angle |
| `Arc(angle=TAU, n_components=3)` | 3 | **3** — explicit wins |

The first four rows are the common cases, and they are unchanged: a
`Circle`'s 33 points are still 33 points, so code that indexes them is
unaffected. The rows that change are the ones where the Reference was
*inconsistent with itself*.

```python
# Before and after, in FrankenManim:
Circle().get_points().shape          # (33, 3) — as in the Reference
Arc(angle=TAU / 4).get_points()      # 9 points — as in the Reference

path = VMobject()
path.start_new_path(ORIGIN)
path.add_arc_to(RIGHT, TAU / 4)      # 9 points now, 5 in the Reference
path.add_arc_to(UP, TAU / 4, n_components=2)   # 5 points, either engine
```

## Evidence

- `crates/fmn-geom/src/bezier.rs` (`arc_n_components`, unit tests).
- `crates/fmn-library/src/arc.rs` (the lineage; `Arc::component_count`).
- `crates/fmn-library/tests/geometry_parity.rs`
  (`the_arc_density_rule_is_ours_and_never_coarser`, which asserts the
  rule's value *and* that it is never coarser than the Reference's, over
  the fixture corpus).
- `crates/fmn-python/tests/bridge.py` (portal `Line.set_path_arc` density,
  identity preservation, and failure atomicity).
- Reference sites: `manimlib/mobject/geometry.py` (`Arc.__init__`),
  `manimlib/mobject/types/vectorized_mobject.py` (`add_arc_to`),
  `manimlib/utils/bezier.py` (`quadratic_bezier_points_for_arc`), all at
  the pinned commit `6199a00d`.
