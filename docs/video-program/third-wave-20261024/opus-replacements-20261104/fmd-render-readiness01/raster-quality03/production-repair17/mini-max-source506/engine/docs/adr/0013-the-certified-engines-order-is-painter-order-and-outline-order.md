# ADR-0013 — The certified engine's accumulation order is painter order and outline order, never table order

**Status:** Accepted
**Date:** 2026-07-26
**Bead:** fm-ig3 (the certified CPU engine)
**Resolves:** the open question fm-5oi and fm-oac each handed forward — whether
the certified path fixes the accumulation *order* across pieces and segments.
**Amends:** nothing in the decision log. §10.5(c) requires fixed-order
reductions; this records *which* order, and why that choice is not a property of
the synchronization.

## Context

Two landed stages accumulate, and both closed with the same unresolved note.

`fill_row` sums a signed area contribution per monotone piece into a row
accumulator and then prefix-sums along x. `stroke_nearest` takes a minimum over
segments, and — because ties are broken by `excess < best`, which keeps the
incumbent — the *arc-length coordinate* it reports depends on which segment came
first. Floating-point addition is not associative and a tie-broken minimum is not
commutative, so in both cases the order is part of the answer.

fm-5oi put it exactly: "fill_row accumulates pieces in table order and then
prefix-sums the row, which is deterministic for a given plan — but the plan's
piece order is a property of the sync, so PG-5's thread-count sweep will want
that pinned explicitly rather than inherited." fm-oac added the same caveat for
the stroke's minimum.

The concern is real and worth stating precisely, because "deterministic for a
given plan" is *weaker than certification needs*. The certified promise is that
two runs of the same scene produce the same bytes. If the order were a property
of the sync, then a scene reached by a different route — an object added and
removed, a mobject created earlier, a plan rebuilt rather than retained — could
present the same picture through a differently-ordered plan and render different
last bits. That is not a hypothetical: `RenderPlan`'s shape and style tables are
*interned* and append-only, so an index records **when an outline was first
compiled**, not where it draws. On a retained plan — the normal path — an object
added behind ones already compiled draws first and interns last, so the two
orders come apart permanently.

## Decision

**The certified engine's accumulation order is fixed by two orders, and neither
of them is a table order.**

1. **Across draws: painter order.** The engine iterates
   `plan.shapes().instances()`, which `RenderPlan::sync` rebuilds every frame
   from `Stage::draw_plan()` — §8.5's two-level model, documented normatively in
   `docs/RENDER_ORDER.md` and already *semantics* rather than scheduling. Within
   a tile the binner preserves that order (its runs ascend in instance index by
   construction, at every partition count). Compositing therefore happens in the
   order the scene declares.

2. **Within a draw: the outline's own order.** A shape's pieces and segments are
   `first_segment .. first_segment + segment_count`, a contiguous run produced by
   `compile_shape` from the `QuadPath`'s point array. That order is a property of
   the *geometry* — the order the author's own path visits its anchors — and it
   survives interning, reuse, restyling and instancing untouched.

**What is deliberately not in the order: the interning indices.** `Instance::shape`
and `Instance::style` are append-only table indices whose values depend on
compilation history. Nothing in the engine iterates a table; every loop is over
instances (painter order) or over a shape's own contiguous slice (outline order).

Two consequences follow and are asserted rather than argued:

- `a_retained_plan_and_a_fresh_one_render_the_same_frame` builds one picture two
  ways and requires the frames to be byte-identical. The experiment took two
  attempts and the first one is worth recording, because it proved nothing while
  looking like it proved something: setting creation order against `z_index` on a
  *fresh* plan varies neither table, since `RenderPlan::sync` walks the draw plan
  in painter order and interns as it goes, so interning order **is** painter
  order. The indices diverge only when a plan is **retained** and an object is
  inserted behind ones already compiled — the newcomer draws first and interns
  last. That is also the normal path. The test now asserts the divergence as an
  explicit precondition before comparing a single pixel.
- `the_frame_is_identical_at_every_thread_count` and
  `every_locked_frame_is_thread_count_invariant` cover the schedule at {1, 4, 16}.

**Tile dimensions are declared, not invariant.** One thing this ADR explicitly
does *not* claim: that tile size cannot move a bit. The fill's per-tile carry is
computed in closed form from the pieces to the left of the tile, which is a
different *association* of the same sum than a longer row's prefix — mathematically
equal, and equal to floating-point tolerance rather than to bits. `fmn-render`'s
own `tile_alignment_never_changes_the_answer` measures that agreement at `1e-9`,
not at zero. This is exactly why `docs/INPUT_CLOSURE.md`'s C10 puts "fixed tile
dims" in the **declared certified configuration**: a certified run pins the
dimensions and hashes them, rather than resting on an invariance the arithmetic
does not owe. `fmn_render::engine::journal` carries them, and
`the_identity_journal_separates_what_it_must` fails if it stops.

## Consequences

- **fm-4wt (the fast CPU engine) inherits the harder half.** Order across draws
  and within a shape is settled here; §10.5(c)'s *lane-count independence* is
  not, and a horizontal sum whose association varies with vector width would
  break the property from inside a single segment. `Tier::ALL` and
  `every_tier_reproduces_the_scalar_definition` are the tripwire: a tier lands in
  the list and is held to the scalar frame over the whole corpus.
- **fm-3df (the frame pipeline) is unaffected.** Frame-level parallelism cannot
  reach this: a frame's order is internal to the frame.
- **Forbidden:** any engine loop that iterates `ShapeTable::shapes()`,
  `StyleTable::rows()`, or `MonoTable::pieces()` to *accumulate*. Reading them by
  index is fine and is what the engine does; walking them in table order and
  summing is the failure this ADR names.
- **The retained compositor (fm-uql) gains a precondition it already met.**
  `TileKey` deliberately excludes the instance index for the same reason this
  decision excludes table order — "an object inserted earlier shifts every later
  index without changing what this tile draws." The two rulings agree, and now
  they agree on purpose.
