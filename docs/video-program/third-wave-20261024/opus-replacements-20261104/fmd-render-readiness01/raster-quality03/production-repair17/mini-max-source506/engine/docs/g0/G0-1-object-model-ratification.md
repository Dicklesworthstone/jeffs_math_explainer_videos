# G0-1 — Object model & buffer lifetime: ratification note (fm-dzv)

**Status:** Ratified with amendments, 2026-07-22. Updates decision **D-11**
from "arena + handles PENDING G0 ratification" to **ratified as amended
below**. The executable form is `spikes/g0-1-object-model` (ten scenario
tests + the fluent-API prototype, all compiling and green); W3 implements
fmn-mobject from this note without consulting the spike's internals.

## Ratified: the §8.1 ownership model

`Stage` arena + generational `Mob` handles + rooted lifetime + CoW
snapshots, exactly as sketched, with these decisions now fixed:

1. **Entries are arena-owned; scene membership is a root set.** `remove()`
   from a scene unroots; it never destroys. Every outstanding handle stays
   valid across scene round-trips.
2. **Handles are `Copy`, generational, and stage-scoped.** A `Mob` is
   `(stage_id, index, generation)`. Stale and foreign handles resolve to
   `None` — never to a recycled stranger's data (generation bump on free)
   and never across stages (the **two-scene policy**: content crosses
   stages only by copy; handles are meaningless outside their stage).
3. **Explicit delete is the only destructor, and it defers under pins.**
   `delete()` on a pinned entry marks `pending_delete` and returns; the
   last `unpin()` finalizes (unlink from parents/children/roots, free the
   slot, bump the generation). This is the Python-proxy identity story:
   a proxy holds one pin for its lifetime, so handle → object identity
   survives collection round-trips, and engine-side deletion can never pull
   memory out from under a live proxy.
4. **Multiple parents are first-class.** `submobjects`/`parents` form a
   DAG; family traversal visits each member once (diamond-safe). Copy
   remaps family-internal edges and drops family-external ones (a copy is
   a detached family — external parents keep the original).
5. **Copy semantics are manim's** (§8.3): record data deep-copies,
   family-internal references remap, **updater callables are shared by
   reference** (`Rc<RefCell<dyn FnMut>>`), captured handles inside those
   callables are *not* remapped — exactly the Reference's behavior.
6. **Updaters receive `(&mut Stage, Mob, dt)`.** Closures capture plain
   `Copy` handles and resolve them at call time, so the borrow checker and
   the arena never fight. `add_updater(call_now = true)` runs the updater
   exactly once (the Reference's double-call is a bug; Behavior-Noted with
   fm-8dx's W3 sibling).

## Ratified with amendment: the §8.2 view protocol (risk R12)

The zero-copy view protocol is **sound in safe Rust** and R12's fallback
(copy-based export) is **not needed**. The load-bearing construction:

- **V1. Storage generations are fixed-capacity allocations** (`Arc`-owned,
  never growable). *Reallocation under a live view is impossible by
  construction* — growth is a fresh allocation swapped in (copy-on-resize,
  null-padded), never `realloc`.
- **V2. Views pin their generation.** A view holds the `Arc`; the memory a
  NumPy view points at outlives anything the engine does to the mobject.
- **V3. Live views alias the current generation**: engine writes are
  visible through them, their writes are visible to the engine. After a
  resize (or a snapshot restore) they are **detached**: still readable,
  pinned to the old generation, no longer tracking — NumPy-natural.
- **V4. Every write, through buffer or view, bumps the generation's
  revision counter.** Render state is dirty *by comparison* (the lazy
  revisioned mirrors of §8.2/§10.8 poll revisions); nothing notifies
  eagerly.
- **V5 (the amendment — the snapshot/view exclusion rule).** A generation
  is **never simultaneously shared by a snapshot and aliased by a live
  view**, because their requirements contradict (snapshots must not see
  writes; live views must). Discipline:
  - `snapshot()` **eagerly copies** buffers that have live views; all
    others share CoW.
  - Writers and view-exporters **unshare first**: if any snapshot holds
    the generation, clone it before writing / before attaching the view.
  - Consequence: snapshot cost is O(live-viewed objects) eager copy +
    O(1) share for everything else. Live-viewed objects are exactly the
    conservatively-invalidated set of §8.2, so this composes with the
    mirror rule rather than adding a new tax.
- **V6. Restore never mutates visible memory**: it swaps generations in,
  so outstanding views detach exactly as under resize (V3).

For W3: the spike models the view as a safe Rust type over
`Arc<Storage>`; fmn-python's NumPy export later maps the same `Arc`
pinning to buffer-protocol lifetimes. The one deferred question — the
concurrency story for revision/pin counters under the frame pipeline — is
bounded by §10.5's single-writer frame semantics and lands with W3's
production implementation, not here.

## Ratified: the §15.1 fluent surface

The "scoped stage context + deferred-command `.animate` recording" lean is
confirmed by compilation and reads well:

- Builders are `Default` config structs with by-value chained setters
  (`Square::new().side_length(2.0).color(BLUE)`); `stage.add(...)` accepts
  `impl Into<Mobject>` and moves the detached value into the arena.
- `mob.animate()` returns a recorder owning only the `Copy` handle —
  no stage borrow — so chains like
  `stage.play(square.animate().rotate(PI/4).set_opacity(0.5))` are
  borrow-checker clean by construction.
- `stage.play(...)` accepts one recording or a tuple; it **refuses stale
  or foreign target handles with a typed error** rather than silently
  dropping work.
- Mutation between plays goes through `stage.get_mut(handle)` — the
  README's `stage.get_mut(label).next_to(square, UP)` shape holds.

## Amendments summary (what changed vs. the §8 sketch)

| # | Amendment |
|---|---|
| A1 | The snapshot/view exclusion rule (V5) is new: §8.2 did not say how CoW snapshots coexist with live views. It is now part of the view protocol. |
| A2 | Delete-defers-under-pins is the ratified lifetime for proxy identity (§8.1 listed the scenario, not the mechanism). |
| A3 | Handles carry a `stage_id`; the two-scene policy is "checked error + copy transfer", not UB or implicit sharing. |
| A4 | Restore detaches views (V6) under the same semantics as resize — one rule, two triggers. |

## What the spike deliberately does not decide

- The six-step frame order, clocks, and real interpolation (Choreo).
- The QuadPath wiring into record fields and null-padding family
  alignment (W3, with fmn-geom).
- The GIL/reentrancy rules and true NumPy buffer export (G0-5, fm-87q,
  which builds its prototype bridge against this model).
- Concurrency hardening of the counters (W3, under §10.5).
