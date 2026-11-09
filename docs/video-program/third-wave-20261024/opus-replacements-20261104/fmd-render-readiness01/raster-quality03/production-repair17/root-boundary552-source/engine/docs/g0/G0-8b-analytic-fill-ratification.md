# G0-8b — The analytic fill's GPU mapping report (fm-orn)

**Status:** Ratified, 2026-07-25. **Verdict: GO** — §10.2's analytic fill maps
onto Metal-via-frankentorch cleanly, in *two* dispatch shapes, neither of which
needs an atomic, a barrier, or a byte of threadgroup memory. The executable form
is `spikes/g0-8-accelerator` (75 tests, green on linux-x86-64 and
macos-aarch64); the rendered evidence is `docs/g0/g0-orn-renders/`.

**Not a ninth spike.** §20.1 has eight, and this is the follow-on
`G0-8-accelerator-ratification.md` §5 named: *"[the fill] is the next thing to
spike, and it should be spiked before W5 commits to a fill layout."* Everything
in G0-8's mapping report still stands; this report adds the fill's tables, its
findings (numbered from F11, continuing G0-8's sequence), and the go/defer
decision the bead asked for.

**What ran, and where.** A fill-only frame compiled to the same prototype IR,
rendered by the CPU reference engine in `f64`, by the identical algorithm in
`f32` (the arithmetic floor, measurable on a machine with no GPU), and by two
Metal kernels over the same tables. The Metal frames were produced on
**mac-mini-max** (Apple M4 Pro, 14 cores, 64 GiB unified memory, Darwin 25.2.0
arm64, macOS 26.2), built with the `SUITE.lock` toolchain pin
`nightly-2026-07-05`.

---

## 1. The verdict, in one paragraph

The assessment G0-8 recorded — *"harder but tractable: the accumulation is
per-tile and the tile is already a threadgroup, so the natural shape is a
threadgroup-local scanline reduction"* — is **refuted, in the favourable
direction**. No reduction is needed, because the accumulation never has to cross
a thread at all. The literal §10.2 shape gives each thread a whole scanline and a
private accumulator; the alternative gives each thread one pixel that re-derives
the winding to its left. Both are barrier-free, atomic-free and deterministic
within a run, both land at the `f32` arithmetic floor against the CPU reference
(**max Δ8-bit = 1**, which is what `f32` alone already costs), and the two GPU
shapes are **byte-identical after 8-bit encoding**. Neither is measurably faster
than the other — at preview resolution both sit just above a ~1.1 ms floor made
of buffer allocation and readback (F17) — so the choice is made on structure, and
there the per-pixel shape wins outright: full threadgroup occupancy, no
per-thread arrays, no compile-time cap on tile size, and the same dispatch model
the stroke kernel already uses. The fill does not force a second dispatch model
into the annex.

---

## 2. The measurement

Fill frame, 960×540, tile 16×16 — 42 paths, 530 segments compiled to 536
monotone pieces, 10 styles, 2078 tile commands. The scene exercises
doubly-monotone splitting (a lobed blob), the nonzero winding rule twice (an
annulus, whose hole comes from opposing orientation, and a pentagram, whose core
is wound twice and therefore *filled* — where even-odd would differ), the tile
carry (shapes far wider than a tile), §10.4's interior class, occlusion, gradients
and painter-ordered alpha, an edge-dense cluster of small discs, and a sliver
thinner than a pixel.

| Run | max abs | mean abs | rms | max Δ8-bit | components differing | SSIM |
|---|---|---|---|---|---|---|
| **CPU f32 vs CPU f64** — the arithmetic floor | 5.117e-5 | 9.155e-8 | 1.180e-6 | **1** | 15 / 2 073 600 (0.0007 %) | 1.000000 |
| **Metal scanline vs CPU f64**, `Safe` | 5.117e-5 | 7.056e-8 | 9.365e-7 | **1** | 17 / 2 073 600 (0.0008 %) | 1.000000 |
| **Metal per-pixel vs CPU f64**, `Safe` | 5.111e-5 | 7.053e-8 | 9.367e-7 | **1** | 17 / 2 073 600 (0.0008 %) | 1.000000 |
| **Metal scanline vs CPU f64**, `Fast` | 6.247e-5 | 1.278e-7 | 7.900e-7 | 1 | 13 / 2 073 600 (0.0006 %) | 1.000000 |
| **Metal per-pixel vs CPU f64**, `Fast` | 5.978e-5 | 1.436e-7 | 8.793e-7 | 1 | 22 / 2 073 600 (0.0011 %) | 1.000000 |
| **Metal scanline vs Metal per-pixel** — the reorder alone | 2.754e-5 | 2.088e-10 | 4.325e-8 | **0** | **0 / 2 073 600** | 1.000000 |

Read rows 1–3 together:

> **Under safe math, both Metal fill kernels land on the arithmetic floor.**
> Their divergence from the `f64` reference is `f32`'s divergence from it, to
> within two differing components out of 2 073 600 — and the largest 8-bit
> error anywhere in the frame, on any engine, is one level.

This is a materially better result than the stroke stage's, where G0-8 measured
max Δ8-bit = 152 and traced it to `f32` conditioning at curvature extrema
(finding F10). The fill's arithmetic is better conditioned because its
load-bearing quantity is a **difference of two scanline ordinates** — bounded by
one pixel, and exactly representable at the band edges — rather than a distance
recovered from a cubic root solve.

**Timings**, medians of 33 warm dispatches on the M4 Pro, across three
independent runs of the whole harness (a single median is not a measurement
here — see below):

| Engine | run 1 | run 2 | run 3 | threads/threadgroup |
|---|---|---|---|---|
| CPU reference, `f64`, single-threaded, unoptimized | 4.6 ms | 4.6 ms | 4.4 ms | — |
| **Metal dispatch floor — empty scene, same grid** | **1.18 ms** | **1.05 ms** | **1.31 ms** | 256 |
| Metal, scanline, `Safe` | 2.21 ms | 1.72 ms | 1.29 ms | **16** of 1024 |
| Metal, scanline, `Fast` | 1.62 ms | 1.32 ms | 1.50 ms | 16 |
| Metal, per-pixel, `Safe` | 1.59 ms | 1.35 ms | 1.59 ms | **256** of 1024 |
| Metal, per-pixel, `Fast` | 1.53 ms | 1.78 ms | 1.59 ms | 256 |

**The only defensible timing conclusion is the floor row.** An empty scene on the
same grid — no paths, no commands, a kernel that clears and stores — costs
~1.1 ms, and every one of the four kernel configurations lands within
0.2–1.1 ms of it, with run-to-run spread larger than the gaps between them. So
at preview resolution the fill's *dispatch shape is not the cost*: allocating,
clearing and reading back a 7.9 MiB `f32` surface is. That is G0-8's F6 restated
as a fraction rather than a ratio, and it is the finding that should drive
optimization (F17).

The measurement discipline mattered twice, both times against a conclusion this
report would otherwise have printed:

- a single-sample run reported per-pixel/`Fast` at **5.66 ms**, which reads as a
  finding about fast math and was noise;
- a 9-sample run made per-pixel/`Safe` look 30 % faster than scanline/`Safe`,
  which did not survive 33 samples or a second run.

The IR compile, monotone derivation and tile classification together cost
0.59 ms; occlusion pruning 0.04 ms. **None of this is a speedup claim**: the CPU
number is a reference implementation with no tiling parallelism, no SIMD, no span
fast path and no adaptive AA.

Per-frame traffic: **36 772 B uploaded** (35.9 KiB — the whole fill IR including
the classification word), **8 294 400 B read back**.

---

## 3. The mapping, table by table — what fm-gw7 inherits

| §10.8 structure | Device form | Verdict |
|---|---|---|
| `SegmentTable` (fill's view) | **a second, derived table**: `f32[6]` per doubly-monotone piece | **Derived, never shared** — see F11 |
| `PathTable` | `u32[4]` (`first_piece, count, style, pad`) + `f32[4]` (slab) | **Clean**, and the same split F1 already required |
| `StyleTable` (fill's view) | `f32[12]`: `rgba, rgba_end, gradient_axis` | **Stage-specific** — see F12 |
| per-tile command lists | CSR `u32` offsets + `u32` draws, **plus a parallel `u32` flag word** | **Clean; the flag word is new** — see F13 |
| conservative slab | `f32[4]`, per-row reject rather than per-pixel | **Clean**, and it serves a second purpose here |
| primitive hints | not exercised by the fill | Assessed in §5 |
| `InstanceTable`, `ImageTable` | not exercised | Still assessed, not proven (G0-8 §5 stands) |

Per-path piece counts are bounded: a quadratic has at most one extremum per
axis, so **one segment yields at most three pieces**. A device-side allocator can
size the fill's geometry table at `3 × segments` without a second pass.

---

## 4. Findings

Numbered from F11, continuing G0-8's sequence.

### F11 — The fill needs its own geometry table; it must not reshape the SegmentTable

§10.2 wants monotone pieces. §10.3 wants whole segments. These are not the same
table and merging them costs the stroke stage correctness, not just work:

- a segment carries its normalized arc-length span `(s0, s1)`, which §10.3
  interpolates width and colour along; splitting a segment invalidates that span
  unless every piece re-derives it, and
- the stroke's nearest-point search over more, shorter pieces is strictly more
  work for the same answer.

So `MonoTable` is a **derived artifact keyed by the geometry revision alone** —
the same rule §10.8 already states for the arc-length LUT, and the same rule F3
generalized. A style change or a camera move must not rebuild it. On the device
it is one flat `f32` array at stride 6; there is no arc-length span in it,
because §10.2 never asks where along the path a pixel is.

### F12 — The StyleTable is stage-specific, and pretending otherwise costs a bitcast

Fills read `rgba`, `rgba_end` and `gradient_axis`; strokes read `rgba`,
`width_start`, `width_end`, `aa_width`. One union-shaped row serving both wastes
a third of every entry or reintroduces exactly the mixed-type access F1 exists to
forbid. The IR should keep **one interned style table in host form** — dedup is
semantics, per §8.5's bitwise `BatchKey` rule — and derive a **per-stage device
row** from it. The dedup index is shared; the layout is not.

### F13 — §10.4's tile classification wants to live in the command list, and occlusion pruning does not work without it

The single most consequential finding here, because it ties two optimizations
that read as independent.

§10.4 classifies tiles as "empty, fully covered, simple-edge, or complex-edge" so
that "interiors fill as vectorized spans". §10.8 separately asks for
painter-order-safe occlusion pruning "when the result is provably unchanged".
Implemented independently, **the second one silently changes pixels**:

> An interior tile's coverage, *accumulated*, is `1 − ε`, not `1`. So an opaque
> fill drawn over another layer lets that layer show through by an ulp. Drop the
> hidden command and the ulp goes away — the frames differ in the last bit of a
> channel, and an "optimization that provably changes nothing" has changed
> something.

The first draft of the pruning pass had exactly this bug and the test caught it.
The fix is not a tolerance: it is to let the classification **short-circuit the
accumulation**, writing exactly `1` for an interior command. That is also the
*more* accurate answer — analytically the coverage of an interior cell is one,
and the `1 − ε` is the numerical error — and it deletes the per-pixel evaluation
for the commands that dominate any large filled shape.

Two consequences for fm-gw7:

1. The per-tile command list should carry `(path_index, flags)`, not
   `path_index`. Four bytes per command; the spike keeps the flags in a parallel
   array only to avoid reshaping buffers G0-8 has already published measurements
   against.
2. Interiority is a property of a **(path, tile) pair**, not of a tile. The flags
   are parallel to the *draws*, not to the tiles.

Measured on the fill frame: **21.8 %** of commands classified interior; occlusion
pruning removed **16.5 %** of all commands across 333 tiles, with the pruned and
unpruned frames byte-identical — asserted in the runner, not hoped for.

The soundness test is deliberately narrow: a closed fill whose control polygon is
convex, whose tile-plus-margin corners all have nonzero winding, and whose
gradient is opaque at both ends. Concave covers, translucent covers, and open
paths are simply not pruned. Failing the test costs an opportunity; passing it
cannot be wrong.

### F14 — The tile carry is free, and it does not change binning

A scanline entering a tile is already inside whatever the path deposited to its
left, so each row needs

```
carry(row) = Σ over pieces of (signed dy of the part in this row left of the tile)
```

That is one closed-form root solve per piece — never a walk from the frame's left
edge. It does mean the fill iterates **every** piece of a path, not only those
whose bounds meet the tile, but binning is unaffected: a path enclosing any pixel
of a tile has hull bounds containing that tile, so the existing slab-keyed
count/prefix/scatter already lists it. **No shadow binning, no clip geometry, no
extra commands.**

### F15 — The scanline walk has a fragile stepping predicate; the per-pixel form has none

The scanline form advances column by column, and the next boundary is
`ceil(x) − 1` (or `floor(x) + 1`), which is discontinuous exactly at integers.
The walk's entry point is the parameter where the piece crosses the tile edge —
an integer — re-evaluated through `x(t)`, which lands an ulp to one side or the
other. Landing on the wrong side names the column the walk is already in, the
step makes no progress, and the fallback deposits a **three-column span as a
single trapezoid**.

This was a real defect, and the shape of it is the lesson: it was wrong in the
**f64 reference** and right in `f32`, purely because the two rounded opposite
ways. A two-width transcription exists to make exactly this visible, and it
nearly hid it — the f32-vs-f64 comparison reported a `max Δ8-bit` of 78 on two
pixels, which reads like conditioning, and the honest reading was "one of these
two is wrong and it is not obvious which".

What resolved it was the **per-pixel form disagreeing with the scanline form at
both widths**, which is only diagnostic because the two shapes exist. The fix is
to clamp the walk's endpoints into the tile — exact, since the span is inside the
tile by construction — plus a secant fallback that always advances. Both are
mirrored in the MSL.

Two rules for W5 out of this:

- **Where an exact value is known, use it rather than re-deriving it.** The
  crossing's `x` *is* the boundary; re-evaluating `x(t)` to recover it is how the
  ulp got in.
- **A tiling test must sweep tile alignments.** The original test compared the
  two shapes over one full-width window and saw nothing, because no piece ever
  entered at an edge. `the_two_orders_agree_on_every_tile_alignment` sweeps tile
  sizes 4/8/16 at every offset.

### F16 — Fast math buys nothing measurable on the fill, and costs accuracy

G0-8's F9 refused Metal's default fast math because it doubled rms error on the
stroke stage. On the fill the accuracy cost is smaller but consistent (per-pixel:
17 → 22 differing components, mean abs 7.05e-8 → 1.44e-7) and the speed benefit
is **not measurable at all** — across three runs `Fast` is faster twice and
slower once, inside a spread the dispatch floor dominates.

So the ruling is unchanged and now rests on two stages, with a cleaner
justification on this one: there is nothing to trade. `MathMode::Safe`,
explicitly, journaled into the backend identity.

### F17 — At preview resolution the boundary is the cost, not the kernel

An empty scene on the same grid costs ~1.1 ms; the real scene costs 1.3–2.2 ms.
**The fixed per-dispatch cost — allocate an 8.3 MB device buffer, clear it, read
it back — is the majority of a preview frame**, and no choice of kernel shape,
math mode, or tile classification moves it.

Three levers, in the order they should be pulled, none of which is in the kernel:

1. **Do not reallocate the surface per frame.** The spike calls
   `buffer_zeroed(8.3 MB)` on every dispatch because a spike should not cache;
   production holds one surface for the engine's lifetime. This is pure waste and
   the easiest to remove.
2. **Do not read back linear `f32`.** §17.6 already prescribes GPU-side
   NV12/P010 conversion before any export readback — a 4× to 8× cut in the only
   large transfer. The spike reads `f32` deliberately, to keep the equivalence
   comparison in the space the compositing happened in.
3. **For Studio preview, do not read back at all.** The frame is going to a
   display; §13.5's preview path should present rather than round-trip.

The corollary for PG-A is that a per-frame annex number is meaningless without
the floor beside it, and this harness now prints both.

---

## 5. What this spike did not exercise, with an assessment

- **Mixed stroke-and-fill frames.** Each annex kernel refuses the other kind by
  name, so the frames here are fill-only exactly as G0-8's were stroke-only.
  Composition of the two stages is a W5 question about pass ordering (§8.5's
  `PassOrder`), not a mapping question, and answering it here would have
  pre-empted it.
- **Primitive hints for fills.** §10.8 routes rectangles, circles and dots to
  specialized coverage. Assessed as *straightforward*: a hinted fill's coverage is
  a closed form with no walk and no accumulator, so it is strictly simpler than
  what already works — and the interior class (F13) is already the biggest of
  those wins, taken generically.
- **The interior class for non-convex fills.** The convexity test is a
  conservative sufficient condition. A general interiority test (an inscribed
  rectangle per path, or a per-tile winding-range bound) would raise the 21.8 %,
  and is a pure optimization behind an unchanged contract.
- **`fill_border_width`** (§10.2's inner border stroke), **3D/perspective fills**
  (projected quadratics are rational in screen space), and **mean-value-coordinate
  interior colour** — all fm-5oi's, none of them layout questions. The gradient
  here is the same *defined stand-in* `crate::fill` documents.
- **The macrotile level** and **multi-frame pipelining** — unchanged from G0-8 §5.

---

## 6. Decisions recorded

- **Go/defer on whether fills join the annex's first production duty:** **GO.**
  The fill is ready to be part of the Metal preview backend (fm-xsz) rather than
  staying CPU-only for G3. It lands at the arithmetic floor rather than above it
  (unlike the stroke stage), needs no dispatch model the stroke stage does not
  already use, and adds no new upstream requirement on frankentorch. This is a
  recommendation to fm-xsz and G3, not a G3 verdict — OQ-12 remains G3's under
  its declared fallback.
- **The recommended dispatch shape is per-pixel**, not the literal scanline form
  — on structure, since the two are not distinguishable on speed (F17): 16× the
  threadgroup occupancy, no per-thread arrays and so no compile-time `MAX_TILE`
  cap, identical output after 8-bit encoding, the same dispatch model as the
  stroke kernel, and no stepping predicate to get wrong (F15). The scanline form
  stays in the spike as the definitional reference and as the **CPU** engine's
  shape, where a serial prefix over a row is the right thing and there is no
  occupancy to lose.
- **D-18 stands unamended.** Standard-mode only, never certified; excluded from
  the core gates; PG-A only; frankentorch the only gateway; no wgpu. The fill
  annex refuses a non-`Fill` IR by name and an oversized tile by name, both
  before the device is opened.

---

## 7. Follow-ups

| Bead | What |
|---|---|
| fm-gw7 | The compiled render IR — consumes §3's verdicts and F11–F14 (the derived monotone table, the per-stage style row, the command flag word, the carry rule) |
| fm-5oi | Analytic fill — inherits this algorithm, its tests, and F15's two rules; owes the real interior colour field, `fill_border_width`, and 3D |
| fm-xsz | The Metal annex production backend — the fill is in scope per §6's GO |
| fm-tg6 | Two-level binning and occlusion pruning — F13's classification is a precondition, not an independent optimization |
| fm-xsz | Also owns F17's first two levers: a surface held for the engine's lifetime, and GPU-side NV12/P010 before any export readback |
