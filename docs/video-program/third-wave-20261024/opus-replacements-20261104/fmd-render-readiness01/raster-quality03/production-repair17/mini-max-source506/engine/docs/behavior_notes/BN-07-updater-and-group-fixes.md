# BN-07 — Corrected mobject behavior (C-5, C-6, C-14, C-15, C-17, C-18, C-19, C-20, C-21, C-23, C-24, C-26, C-29, C-30, C-31)

**Status:** Draft (W3, fm-yra; W10, fm-23ev, fm-5wq.4.39, and fm-easc). Consumed by Choreo (§9.1's
`suspend_mobject_updating` interaction), fmn-python (whose `manimlib`
surface presents these semantics), and the Parity Ledger.

The Appendix-C rulings owned by the mobject surface. All are deliberate,
correct divergences from the pinned Reference (D-05); the
API names and everything around them carry over exactly.

## C-5 — `add_updater(call=True)` runs the update pass exactly once

The Reference (`mobject.py`):

```python
def add_updater(self, update_func: Updater, call: bool = True) -> Self:
    self.updaters.append(update_func)
    if call:
        self.update(dt=0)
    self.refresh_has_updater_status()
    self.update()          # <-- unconditional second update(dt=0) pass
    return self
```

With `call=True`, every updater in the mobject's family runs **twice** at
registration (and even with `call=False`, the trailing `self.update()`
still runs one uninvited pass). An updater with side effects — a counter,
an appender, anything non-idempotent at dt=0 — observes double execution
on every registration.

**FrankenManim:** [`Stage::add_updater`] /
[`Stage::add_dt_updater`](../../crates/fmn-mobject/src/stage.rs) with
`call = true` run exactly one `update(dt=0)` pass over the mobject's
family (matching the Reference's *intended* `self.update(dt=0)` semantics,
including running pre-existing updaters — that part is Reference behavior,
not a bug); with `call = false`, no pass runs at all.

**Migration:** scene code that (knowingly or not) depended on the double
call — e.g. an updater that must run twice before the first frame — should
call `stage.update(0.0)` explicitly for the second pass. Non-idempotent
updaters simply stop double-firing; no action needed.

Locked by `tests/dynamics.rs::c5_call_runs_the_update_pass_exactly_once`
and `tests/scenarios.rs` (s9).

## C-6 — group addition always builds a new group

The Reference has two different semantics under one operator:

```python
class Mobject:
    def __add__(self, other):            # value semantics
        return self.get_group_class()(self, other)

class Group(Mobject):
    def __add__(self, other):            # in-place mutation
        return self.add(other)           # returns self
```

`square + circle` builds a fresh group, but `group + circle` **mutates the
existing group** and returns it — whether `a + b` aliases `a` depends on
`a`'s runtime type, which is exactly the kind of surprise that corrupts a
scene when a "combined" group is positioned independently.

**FrankenManim:**
[`Stage::group_add`](../../crates/fmn-mobject/src/dynamics.rs) always
builds a new group containing both operands, including when the left
operand is itself a group. Consistent value semantics; operands are never
mutated. (fmn-python's `Group.__add__` presents this corrected behavior
under the original name.)

**Migration:** code that relied on `group + mob` mutating `group` should
call `group.add(mob)` — the explicit in-place API, which keeps its
Reference semantics unchanged.

Locked by `tests/dynamics.rs::c6_group_add_is_a_value_operation`.

## C-14 — `get_grid(width=…)` sizes the width

The pinned Reference's `Mobject.get_grid` correctly routes `height` through
`grid.set_height(height)`, but its adjacent width branch also calls
`set_height`:

```python
if width is not None:
    grid.set_height(width)
```

The public argument therefore does not do what its name says. A 3-column by
2-row grid of 2-by-1 rectangles with zero buffer starts at 6-by-2; asking for
`width=6` turns the Reference result into 18-by-6 instead of leaving it at
6-by-2.

**FrankenManim:** `get_grid(width=value)` calls `set_width(value)`.
`height=value` continues to call `set_height(value)`, and row/column grouping
is unchanged.

**Migration:** code that worked around the Reference defect by passing the
desired height as `width=` should pass `height=`. Ordinary callers using the
argument according to its name now receive the requested width.

Locked by the actual-extension bridge acceptance in
`crates/fmn-python/tests/bridge.py`, including the 6-by-2 planted negative and
row/column grouping checks.

## C-15 — `FunctionGraph(color=…)` styles the graph

The pinned Reference declares `color=YELLOW` on `FunctionGraph.__init__`, then
constructs its parametric function and calls the parent without forwarding
`color`. The default happens to look ordinary only when surrounding defaults
also select yellow; an explicit `FunctionGraph(f, color=RED)` request is
silently ignored.

**FrankenManim:** the declared color reaches the native Atlas graph style.
The default is still `YELLOW`, and all other `ParametricCurve` sampling,
discontinuity, and smoothing options keep their original meaning.

**Migration:** remove any post-construction recoloring that existed solely to
work around the ignored constructor argument. Code that did not pass `color`
is unchanged.

Locked by the actual-extension FunctionGraph coverage in
`crates/fmn-python/tests/bridge.py` and the production Python PNG Gauntlet row.

## C-17 — `NumberPlane.get_y_unit_size()` measures the y axis

The pinned Reference's `get_y_unit_size` body reads
`return self.get_x_axis().get_unit_size()`: a copy-paste defect that answers
a y-axis query with the x-axis scale. On any plane whose x and y ranges or
dimensions differ, the reported unit size is simply wrong.

**FrankenManim:** the method measures the axis the name names, over live
geometry (`axis length / (x_max − x_min)`), so rescaling or restretching the
plane is reflected exactly.

**Migration:** code that worked around the defect by calling
`get_x_unit_size()` where it wanted the y scale can call `get_y_unit_size()`
directly. Code on default square-aspect planes sees no change.

Locked by the actual-extension bridge acceptance in
`crates/fmn-python/tests/bridge.py` (equal-units assertion on a square-aspect
plane alongside the x-axis measurement).

## C-18 — `SingleStringTex`, `OldTex` and `OldTexText` construct

At the pin, `SingleStringTex` supplies its SVG through
`get_svg_string_by_content` (`old_tex_mobject.py:77`). `SVGMobject.__init__`
never calls that method; it accepts only `svg_string` or `file_name`
(`svg_mobject.py:86-93`). `SingleStringTex` passes neither, so every
`SingleStringTex`, `OldTex` and `OldTexText` construction raises
`Exception: Must specify either a file_name or svg_string SVGMobject`,
whatever the input. The corpus differential (fm-5wq.33) observed this in
every 3b1b scene that uses them.

**FrankenManim:** these classes construct as their evident intent: native
typesetting (BN-05), with the Reference's part-splitting semantics
(`tex_strings`, isolates and `tex_to_color_map` keys, one part per piece,
and part lookup) kept exactly.

**Migration:** none needed. Scenes that could not run under the pinned
Reference now run.

Locked by the OldTex/OldTexText assertions in
`crates/fmn-python/tests/bridge.py` (part splitting, part lookup).

## C-19 — a caller's `color=` is not masked by constructor defaults

The pinned Reference resolves a VMobject's channels as
`fill_color or color or DEFAULT` and `stroke_color or color or DEFAULT`
(`vectorized_mobject.py:100-102`). A constructor that passes its own
default channel color therefore masks the caller's `color=`:

- `StringMobject.__init__` (`string_mobject.py:46-72`) passes `color=`
  through `**kwargs` to `SVGMobject`, then re-applies its white
  `fill_color`/`stroke_color` defaults with `set_fill`/`set_stroke`. So
  `Text("What is this", color=RED)`, `Tex(..., color=BLUE)` and every other
  `StringMobject` given only `color=` render white.
- `Dot(color=RED)` passes `fill_color=DEFAULT_MOBJECT_COLOR` and
  `stroke_color=BLACK` to `Circle`, so it renders white with a black stroke.
  The corpus has 147 `Dot(..., color=...)` call sites in 60 files.

The corpus differential (fm-5wq.33) found the text case in dozens of 3b1b
scenes whose authors asked for a color. For example, `_2021/newton_fractal.py`
`WhatIsThis` is red in the portal and white in the pinned Reference.

**FrankenManim:** the precedence is explicit channel keyword > `color=` >
constructor default. A caller's `color=` colors every channel the caller
left unset, over any constructor default: the evident intent and manim's
long-standing behavior. An explicit `fill_color`/`stroke_color` still wins
over `color=`, as in the Reference. So `Square(color=RED, fill_color=BLUE)`
fills blue with a red stroke, and `Dot(color=RED, fill_color=BLUE)` is a
blue dot with a red stroke. (Until fm-qead, `color=` also overrode explicit
channels.) Constructors mark their own channel defaults as such, which is how
the portal tells them from a caller's keyword. `BackgroundRectangle` keeps
the Reference's fixed style (its `set_style` changes only fill opacity).

Locked by the C-19 and fm-qead assertions in
`crates/fmn-python/tests/bridge.py`.

**Migration:** scenes that relied on the defect to keep text or dots white
while passing `color=` should drop the keyword.

## C-20 — `make_number_changeable(value, index=k)` returns the number it installed

The pinned Reference (`tex_mobject.py:249-273`) narrows `parts` to
`[parts[index]]` when `replace_all` is false, splices that one
`DecimalNumber` into the formula, and then returns `decimal_mobs[index]`
from the one-element list. Any `index > 0` raises `IndexError`, after the
formula has already been changed. `Tex("1 + 1").make_number_changeable("1",
index=1)` leaves a `DecimalNumber` in the family and hands the caller an
exception. Negative indices happen to work.

**FrankenManim:** the call returns the `DecimalNumber` it installed at the
`index`-th occurrence.

Locked by `test_index_selects_the_matching_source_occurrence` in
`crates/fmn-python/tests/live_tex.py`.

**Migration:** none needed. Such calls could only have raised in the
Reference.

## C-21 — a changeable number reports the style it draws with

The Reference styles the new number with `decimal_mob.match_style(part)`,
where `part` is the temporary `VGroup` that `select_parts` built. The
number's root takes that group's defaults: white, stroke width 4. Its digits
take the glyphs' style through the recursive match. In
`Tex("x = 1").set_color(RED)`, the number's digits render red while
`number.get_color()` reports white and `get_stroke_width()` reports 4. Code
that reads the number's color to style something else gets white.

**FrankenManim:** the root takes the selected glyph's style, so
`get_color()` reports the color the digits draw with, and the root's stroke
width is the glyph's (0). The rendered digits are the same in both engines.
After `set_value` the Reference restyles the root from its first glyph, so
from then on the two engines agree.

Locked by `test_preserves_live_scene_identity_and_style` in
`crates/fmn-python/tests/live_tex.py` (`first.get_color() == RED`).

**Migration:** none needed, unless a scene depended on reading white from a
number inside a colored formula.

## C-23 — subpath ends follow the points after `match_points`

The Reference caches `VMobject.subpath_end_indices` and clears it only in
the setters wrapped by `triggers_refresh`. `Mobject.match_points` resizes
and rewrites the points without clearing it, so after matching a path with
fewer points the cached ends index past the points. The next joint refresh,
which every animation's `begin` runs, raises `IndexError`. The corpus scene
`_2024/puzzles/added_dimension.py` `StruggleWithStrips` dies this way: its
strip updater calls `match_points`.

**FrankenManim:** subpath ends are derived from the live points on every
read, so no write path can leave them stale.

Locked by the C-23 assertions in `crates/fmn-python/tests/bridge.py`
(`get_subpath_end_indices()` is `[4]` after matching a 5-point path).

**Migration:** none needed. Such scenes could only have raised in the
Reference.

## C-24 — a partial `background_line_style` keeps the other defaults

The Reference's `NumberPlane` copies a caller's `background_line_style` in
place of its default dict, then reads `stroke_color` from it to style the
faded lines. `NumberPlane(background_line_style={"stroke_width": 3})`
raises `KeyError: 'stroke_color'`. Two corpus scenes do this.

**FrankenManim:** the caller's keys merge over the default style, so that
plane draws its background lines in the default `BLUE_D` at width 3.

Locked by the C-24 assertions in `crates/fmn-python/tests/bridge.py`.

**Migration:** none needed. Such calls could only have raised in the
Reference.

## C-26 — a proportion along a zero-length path is its start point

The Reference's `point_from_proportion` handles an empty path, but not a
non-empty one of zero length: it computes a curve index one past the last
curve and asserts. `Arc(angle=0).pfp(0.3)` raises `AssertionError`. A
corpus scene hits this with an arc that follows a line's angle while the
angle is 0.

**FrankenManim:** the call returns the path's start point.

Locked by the C-26 assertions in `crates/fmn-python/tests/bridge.py`.

**Migration:** none needed. Such calls could only have raised in the
Reference.

## C-29 — `restore()` and `become()` carry a point-less member's style

A point-less Reference mobject keeps its colors in `_data_defaults`, not in
`data` (`mobject.py:1329`, `vectorized_mobject.py:264`), and `become()`
copies `data` only (`mobject.py:730`). So `restore()` leaves every point-less
member of the family (an `Integer` root, the group inside it, a `VGroup`)
with the style it had before the call, while members with points return to
the saved state. `get_fill_opacity()` on such a member then reads the stale
value: in `_2020/hamming.py` `OneGroupPerParityBit`, a grid of `Integer`
bits is faded by subgroup and restored four times, and every root a fade
ever reached stays at opacity 0, though its glyph is visible again.

**FrankenManim:** `become()` and `restore()` carry each member's style,
with or without points.

Locked by the C-29 assertions in `crates/fmn-python/tests/bridge.py`. The
same rule shows in `_2023/convolutions2/continuous.py` `RepeatedConvolution`
(and its three subclasses). `prod_rhs = fg_label[6:]` is a sliced `VGroup`,
and `prod_rhs.become(...)` leaves its point-less root grey in the Reference
but carries the source's style in FrankenManim. The corpus differential
excludes it by the `bn07-c29-become-point-less-style` row.

**Migration:** none needed. A scene that reads a restored member's style
gets the saved value, matching what is drawn.

## C-30 — `BackgroundRectangle.get_fill_color()` reports the rectangle's color

The Reference's `BackgroundRectangle.get_fill_color()` returns
`Color(self.color)` (`shape_matchers.py:96`). `VMobject.__init__` consumes
`color=` itself, so `self.color` is `Mobject`'s default white. The getter
reports `Color("white")`, a `colour.Color` rather than a hex string, whatever
the rectangle is filled with; by default that is the camera background,
`#333333`.

**FrankenManim:** the getter reports the rectangle's own color as a hex
string: the constructor's `color=`, the camera background by default. That is
the fill it draws until a `set_style` call forces black (the Reference's
fixed style, kept).

Excluded from the structural diff by the `bn07-c30-background-rectangle-fill-getter` row,
pinned to the Reference's `<Color white>`.

**Migration:** none needed. A scene that read this getter got white; it
now gets the rectangle's color.

## C-31 — `match_style` styles a point-less mobject

The Reference's `VMobject.set_style` writes `fill_rgba` and `stroke_rgba`
into `data` only (`vectorized_mobject.py:232`, `:242`). Every other color
setter falls back to `_data_defaults` when there are no points
(`mobject.py:1329`), so `set_fill` and `set_stroke` style a point-less
mobject, but `match_style` (`vectorized_mobject.py:276`), which passes the
source's arrays through `set_style`, is silently dropped. `DashedVMobject`
calls `self.match_style(vmobject, recurse=False)` on its point-less root
(`vectorized_mobject.py:1414`), which keeps VMobject's default colors, and
`VGroup().match_style(Circle())` stays grey. `cube.match_style(square)` on a
`VCube` restyles the faces but leaves the root's BLUE_D fill
(`_2026/spheres_talk/random_puzzles.py` `SumOfTwoSquares`). A `match_style`
inside a play leaves a `ThreeDAxes` tick group's root grey
(`_2024/puzzles/max_rand.py` `VisualizeMaxOfPairCDF`, row
`bn07-c31-axes-ticks-match-style`).

**FrankenManim:** a point-less member takes the matched style, as C-29
carries it through `become()` and `restore()`.

Excluded from the structural diff by the `bn07-c31-point-less-match-style` row.

**Migration:** none needed. The dashes keep the source's style in both
engines; only the point-less root's getters differ.
