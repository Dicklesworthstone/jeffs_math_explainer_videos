# BN-07 — Frame-marker constants are unit directions (C-16)

**Status:** Draft (W10, fm-iuow). Consumed by fmn-python (whose `manimlib`
surface exports these constants), both front doors' constant-resolution
pipelines, and the Parity Ledger.

One Appendix-C ruling owned by the constants surface. It is a deliberate,
correct divergence from the pinned Reference (D-05); the names, the module,
and every constant around them carry over exactly.

## C-16 — `TOP`/`BOTTOM`/`LEFT_SIDE`/`RIGHT_SIDE` work as direction arguments

The pinned Reference (`constants.py:56-59`) defines the four frame-marker
constants scaled by the frame radii:

```python
TOP:        Vect3 = FRAME_Y_RADIUS * UP
BOTTOM:     Vect3 = FRAME_Y_RADIUS * DOWN
LEFT_SIDE:  Vect3 = FRAME_X_RADIUS * LEFT
RIGHT_SIDE: Vect3 = FRAME_X_RADIUS * RIGHT
```

The Reference's own `manimlib` never uses these four names — zero internal
call sites at the pin. Their only observable effect is on user code, and
there they are a trap: the natural reading is "the direction of the top of
the frame", so scenes pass them as DIRECTION arguments. Every alignment
entry point (`Mobject.align_on_border`, `to_edge`, `to_corner`, and their
native engine equivalents) computes `- buff * direction` unnormalized, so
`to_edge(BOTTOM, buff)` misplaces the mobject by an extra
`(FRAME_Y_RADIUS - 1) * buff` inward — for the default 8-unit frame height,
a `buff=1` request lands the mobject three units off. No working scene can
depend on that placement; ManimCE-flavored code (`BOTTOM == DOWN`) silently
expects sane behavior.

**FrankenManim:** the four markers resolve to the unit directions —
`TOP = UP`, `BOTTOM = DOWN`, `LEFT_SIDE = LEFT`, `RIGHT_SIDE = RIGHT` — via
governed `[constants]` rulings in `API_OVERLAY.tsv` (C-16), so passing them
as direction arguments places the mobject exactly where the name promises.
This matches the unit diagonals (`UL`/`UR`/`DL`/`DR` are unscaled sums) and
ManimCE's spelling of the same names.

**Frame math is unchanged:** `FRAME_Y_RADIUS`, `FRAME_X_RADIUS`, `FRAME_HEIGHT`,
and `FRAME_WIDTH` keep their exact values. Code that used the marker values as
frame-extent arithmetic (e.g. `move_to(3 * BOTTOM)` meaning "three units below
the bottom edge") should spell that intent with `DOWN` and `FRAME_Y_RADIUS`
directly — which the Reference itself never did internally.

**Migration:** pass `BOTTOM`/`TOP`/`LEFT_SIDE`/`RIGHT_SIDE` to alignment calls
freely — that now does what it looks like it does. Replace extent arithmetic on
the markers with `DOWN * FRAME_Y_RADIUS`-style expressions over the unchanged
`FRAME_*` constants. Code using `UP`/`DOWN`/`LEFT`/`RIGHT` directly is
untouched.

Locked by the actual-extension bridge acceptance in
`crates/fmn-python/tests/bridge.py`: constant identity against the unit
vectors, detached-path border placement for all four markers, and bound-path
(`Stage::to_edge`) agreement.
