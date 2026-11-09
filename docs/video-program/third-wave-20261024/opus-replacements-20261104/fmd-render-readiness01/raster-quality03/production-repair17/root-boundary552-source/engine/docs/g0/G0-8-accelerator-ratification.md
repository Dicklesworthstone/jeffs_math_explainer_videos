# G0-8 — Accelerator proof: the IR→GPU mapping report (fm-ekx)

**Status:** Ratified, 2026-07-25. **Verdict: GO** for the Accelerator Annex on
Metal-via-frankentorch, under every constraint §10.7 already placed on it.
The executable form is `spikes/g0-8-accelerator` (36 tests, green on
linux-x86-64 and macos-aarch64); the rendered evidence is
`docs/g0/g0-8-renders/`.

**What ran, and where.** The compiled render IR's highest-ROI stage — stroke
signed-distance evaluation plus AA resolve (§10.7's own ranking: "the dominant
cost of typical 2D scenes, per-pixel independent") — expressed once as a
backend-neutral IR, rendered by a CPU reference engine in `f64` and by a Metal
kernel in `f32`, and compared. The Metal frame was produced on **mac-mini-max**
(Apple M4 Pro, 14 cores, 64 GiB unified memory, Darwin 25.2.0 arm64, macOS
26.2), built with the `SUITE.lock` toolchain pin `nightly-2026-07-05`.

This report is the deliverable **fm-gw7 consumes**. §20.1 put spike 8 in G0
precisely so its layout findings reach the IR design *before* W5 freezes it.

---

## 1. The verdict, in one paragraph

The IR maps onto a GPU cleanly, and the mapping is not close-run. Every table
the stroke stage reads survives the trip as a flat, single-typed device array;
the per-tile command lists become one threadgroup per tile with no
synchronization and no atomics; painter order is structural rather than
enforced; and the resulting frame is visually indistinguishable from the CPU
reference (SSIM 0.999949, 0.0097 % of 8-bit components differing at all). The
one thing that had to change was **not in FrankenManim**: frankentorch had no
route to a GPU for a consumer's own kernel, and D-22 makes frankentorch the only
route there is. That is now fixed upstream (ledger row 8) rather than worked
around here.

---

## 2. The measurement

Preview frame, 960×540, tile 16×16 — 30 paths, 164 segments, 14 styles, 2040
tiles, 3898 tile-draws. The scene deliberately exercises curvature (arcs built
by `QuadPath::arc`, a Lissajous with osculating handles), the `Line` primitive
hint, sharp joins, round caps, arc-length width tapers, translucent overlap in
painter order, and a sub-pixel hairline thinner than the AA band.

| Run | max abs | mean abs | rms | max Δ8-bit | components differing | SSIM |
|---|---|---|---|---|---|---|
| **CPU f32 vs CPU f64** — the arithmetic floor | 7.214e-1 | 6.703e-6 | 1.310e-3 | 152 | 200 / 2 073 600 (0.0096 %) | 0.999949 |
| **Metal vs CPU f64**, `MathMode::Safe` | 7.214e-1 | 6.701e-6 | 1.310e-3 | 152 | 201 / 2 073 600 (0.0097 %) | 0.999949 |
| **Metal vs CPU f64**, `MathMode::Fast` | 7.977e-1 | 1.328e-5 | 2.502e-3 | 184 | 235 / 2 073 600 (0.0113 %) | 0.999790 |

Read the first two rows together, because that pairing is the point:

> **Under safe math, the Metal engine's entire divergence from the reference is
> what `f32` does to the algorithm. The GPU itself contributes one differing
> component out of 2 073 600.**

That is established by a controlled experiment, not by inference: `sdf::
distance_to_quadratic_f32` is a literal `f32` transcription of the same
function, run on the CPU, on a machine with no GPU. Its divergence from the
`f64` reference (row 1) and Metal's divergence from the same reference (row 2)
agree to four significant figures.

**Timings** (same M4 Pro): IR compile 0.29 ms; CPU reference 999 ms
single-threaded and deliberately unoptimized; Metal 10.4 ms warm, including a
full 8.29 MB `f32` RGBA readback. **This is not a speedup claim.** The CPU
number is a reference implementation with no tiling parallelism, no SIMD, no
span fast path and no adaptive AA — comparing it to a GPU says nothing about
the CPU engine W5 will actually build. What it *does* say is that the stage fits
in ~10 ms at preview resolution with a wasteful readback, which is the signal
G3's 60 fps preview requirement needed.

---

## 3. The mapping, table by table — what fm-gw7 inherits

| §10.8 structure | Device form | Verdict |
|---|---|---|
| `SegmentTable` | `f32[8]` per segment: `p0.xy, p1.xy, p2.xy, s0, s1` | **Clean.** 32 bytes, naturally aligned, sequentially read. |
| `PathTable` | split into `u32[4]` (`first_segment, count, style, hint`) and `f32[4]` (the slab) | **Needs the split** — see finding F1. |
| `StyleTable` | `f32[8]`: `rgba, width_start, width_end, aa_width, pad` | **Clean.** Interned by exact bits, matching `BatchKey`. |
| per-tile command lists | CSR: `u32` offsets + `u32` draws | **Clean, and structurally superior on the GPU** — see F2. |
| `InstanceTable`, `ImageTable` | not exercised | Assessed in §5, not proven. |
| primitive hints | `u32` in the path header; kernel branches to a capsule evaluator | **Clean**, and measurably real: the hint is read and taken. |
| arc-length parameterization | precomputed host-side into `(s0, s1)` per segment | **Clean, and the right division of labour** — see F3. |

Per-frame traffic: **30 444 B uploaded** for the whole IR (29.7 KiB — the entire
scene description), **8 294 400 B read back** (the `f32` surface). The asymmetry
is the finding of §4's F6.

### F1 — One scalar type per buffer, or pay a bitcast in the inner loop

A Metal buffer binding is a typed pointer. A `PathHeader` mixing `u32` indices
with `f32` bounds must either be bound twice with different types (aliasing the
same memory, which is legal but obscures the layout) or unpacked with `as_type`
casts inside the per-command loop. The IR therefore derives **one flat array per
table, each array a single scalar type** — `RenderIr::flatten()` → `FlatIr`.

This is cheap to honour if the IR is designed for it and expensive to retrofit,
which is exactly why the spike ran before fm-gw7. **Recommendation for W5:** keep
the backend-neutral IR as host structs (readable, one source of truth) and treat
the flat single-typed arrays as a *derived* layout, one derivation per backend —
which is what §10.8 already says ("SoA/AoSoA sized to SIMD lanes on CPU;
contiguous device arrays for CUDA; packed shared/private buffers for Metal").
The spike confirms that sentence survives contact with a real GPU.

### F2 — The CSR command list makes painter order structural

§10.8 requires "stable per-tile command runs" with "stable draw indices" and
forbids "unordered atomic appends" on the annex. The CSR shape delivers all
three by construction rather than by discipline: binning is a deterministic
count → prefix-sum → scatter on the **host**, and the kernel only *reads*
`draws[offsets[t] .. offsets[t+1]]` in index order. There is no ordering
decision at dispatch time to get wrong, no atomic, and no synchronization
anywhere in the kernel.

A consequence worth stating plainly: **both engines consumed byte-identical
command lists.** Any divergence the comparison finds is therefore provably in
the kernel, never in the binning — which is what made the arithmetic floor of §2
measurable at all.

### F3 — Precompute arc length into the IR; never ship the machinery

§10.3 interpolates stroke width and colour by **arc length**. §7.3's arc-length
layer is transcendental-dense and branchy — precisely the wrong shape for a
per-pixel kernel. The IR resolves this by storing, per segment, the normalized
arc-length span `(s0, s1)` computed on the host by fmn-geom's exact closed form.
The GPU reads two floats and lerps.

Because the span is a function of geometry alone, it is invalidated by the
**geometry revision** and survives a style change or a transform — the same
retained-cache rule `arclength::CachedArcLength` already implements. **This
generalizes:** any IR field that is expensive, branchy, and a pure function of a
single revision belongs on the host side of the boundary, keyed by that
revision. It is the cheapest lever the annex has.

### F4 — One threadgroup per tile, sized from pipeline introspection

16×16 tiles → 256 threads per threadgroup, against the pipeline's reported
maximum of 1024 and a SIMD execution width of 32 — read from
`Pipeline::max_threads_per_threadgroup()` and `thread_execution_width()`, not
assumed (§17.6: "threadgroup sizes taken from pipeline introspection, never CUDA
habit"). A tile larger than the pipeline allows is a **typed error**, not a
silently reshaped dispatch: quietly halving the threadgroup would make the
pixel-to-thread mapping a mystery at the exact moment someone is debugging it.

The accumulator stays in registers for the whole tile and the surface takes
exactly one store per pixel — the tile-local compositing Apple's TBDR
architecture rewards.

### F5 — The conservative slab is the load-bearing early-out

Every path carries `[min_x, min_y, max_x, max_y]`, the control-polygon hull
(which contains the curve) grown by `half_width + aa_width`. It serves as the
host's binning key and the kernel's per-pixel early-out at once, from one field.
Most pixels of most tiles leave the loop there. Keeping it *in the IR* rather
than recomputing per dispatch is what makes it free.

### F6 — Readback, not upload, is the boundary cost

The entire scene uploads in **29.7 KiB**; the frame reads back **7.9 MiB** — a
272× asymmetry, on a machine where §10.7 calls the handoff "nearly free" because
unified memory is real (`has_unified_memory()` confirmed `true` on the M4 Pro).
The spike reads back linear-light `f32` RGBA deliberately, to keep the
equivalence comparison in the space the compositing happened in. **Production
must not.** §17.6 already prescribes the fix — "GPU-side NV12/P010 conversion
before any export readback" — and this measurement is the argument for it: the
conversion is a 4× to 8× cut in the only large transfer in the pipeline.

---

## 4. Findings that changed code, and one that changed a default

Three defects surfaced. Two were in the spike's own shared mathematics and would
have been inherited by W5; the third is a platform default that would have
silently taxed every annex frame.

### F7 — `sign(0)` is not portable, and it silently broke a root solver

Rust's `f64::signum(0.0)` returns `1.0`. Metal's `sign(0.0f)` returns `0.0f`.
The stable quadratic pairing `q = -½(a₁ + sgn(a₁)·√Δ)` therefore collapsed to
`q = 0` on the GPU alone whenever `a₁` was exactly zero, turning two real roots
into the double root `{0, 0}`. Both engines now write the predicate out
longhand (`x >= 0 ? 1 : -1`), and `sdf.rs` carries the regression test.

**The general lesson for W5:** a kernel mirrored across two languages is only
mirrored where both standard libraries agree, and standard-library disagreement
at edge values is invisible until it is a wrong pixel. Every mirrored primitive
needs a named, tested predicate rather than a builtin.

### F8 — An absolute degeneracy threshold is a bug only one engine can see

The solvers originally tested `|a₃| <= 1e-12` to decide whether a cubic had
degenerated to a quadratic. Screen coordinates run to ~10³, so a *nearly*
straight curve's leading coefficient lands around `1e-8` in `f32` purely from
cancellation — comfortably above `1e-12`. The `f32` engine entered the
genuine-cubic branch, divided by that noise, and produced garbage roots; the
`f64` engine sailed through the identical test. Both solvers now use a threshold
**relative to the polynomial's own coefficient scale**, and **sized per engine
precision** (`1e-14` on the CPU, `1e-6` in the shader). Two further robustness
changes rode along: forming `c = (p₂−p₁) − (p₁−p₀)` instead of `p₀ − 2p₁ + p₂`
(algebraically identical, numerically not close), and treating a
near-zero discriminant as the multiple-root case instead of testing the sign of
a cancellation error.

**The general lesson for W5:** a single shared absolute constant across engines
of different precision is not a shared semantics. Numerical thresholds are part
of an engine's identity and belong next to its arithmetic.

### F9 — Metal compiles fast-math by default; the annex should not

`MTLCompileOptions` enables fast math unless told otherwise, licensing the
compiler to assume no NaNs or infinities, flush denormals, contract
multiply-adds, and substitute reduced-precision reciprocals and square roots.
Measured, on this frame: fast math **doubles** rms error (1.310e-3 → 2.502e-3),
raises the worst 8-bit error from 152 to 184, and adds 17 % more differing
components — for no measurable time saving at this size.

The upstream gateway therefore **defaults to `MathMode::Safe`** and makes the
fast path an opt-in named `MathMode::Fast`. That is a deliberate reversal of the
platform default, taken because the annex's kernels feed comparisons — root
solves, nearest-point searches — where "no NaNs, and reciprocals are close
enough" is not a free assumption.

This matters beyond the annex. D-18 permanently refuses GPU work in the
certified path, so no bit promise was ever at stake; but §16.3's
visual-equivalence budget is a real number that somebody has to meet, and half
of it was being spent on a compiler flag nobody had chosen.

### F10 — The residual is `f32` conditioning at curvature extrema, characterized

The 200 surviving components are not scattered. They sit at two symmetric spots
on one path — the Lissajous's upper turning points, where the distance function
has a near-degenerate critical point and the general quadratic solve is
ill-conditioned. There, `f32` evaluation misses the true distance by enough to
move coverage from 1.0 to ~0.1 across a handful of pixels; everywhere else in
the frame it is sub-quantization noise.

**This is the finding with the longest reach, and it is not about the annex.**
The annex is standard-only and 0.0096 % at SSIM 0.99995 is comfortably inside
any budget §16.3 will write. But the **fast CPU engine** (fm-4wt) is licensed to
use mixed precision under §6.1, and this measurement is the quantified warning:
`f32` in the stroke SDF costs visible error at curvature extrema, on a
mainstream curve, at ordinary stroke widths. The fast engine must keep the
distance solve in `f64` — or adopt a conditioned reformulation and re-measure
with `cpu::Precision::AnnexF32`, which exists in the spike for exactly this
purpose and needs no GPU to run.

---

## 5. What the spike did not exercise, with an assessment

Named rather than silent, so fm-gw7 inherits an evaluation instead of a gap:

- **Fills** (§10.2's analytic winding coverage). Per-scanline rather than
  per-pixel, with a signed-area accumulation whose ordering matters. Assessed as
  *harder but tractable*: the accumulation is per-tile and the tile is already a
  threadgroup, so the natural shape is a threadgroup-local scanline reduction.
  It is the next thing to spike, and it should be spiked before W5 commits to a
  fill layout.

  **Spiked, 2026-07-25 (fm-orn) — see `G0-8b-analytic-fill-ratification.md`.**
  Verdict GO; the assessment above is refuted in the favourable direction, since
  no threadgroup-local reduction is needed in either of the two dispatch shapes
  that work. Both land at the `f32` arithmetic floor (max Δ8-bit = 1, against
  this stage's 152), and the recommended shape is one thread per *pixel* — the
  same shape as this kernel.
- **Glyph instancing** (`InstanceTable`). Assessed as *the easiest remaining
  win*: an instance is a transform plus a style index against an interned
  outline, which is exactly the flat-table shape that already works, and
  text-heavy mathematical scenes are where the duplication lives.
- **Images** (`ImageTable`). Needs `MTLTexture` and a sampler, which the
  gateway does not expose — a genuine upstream extension, not a layout question.
- **The macrotile level.** The spike bins one level. The second level is host-side
  pruning that changes no kernel contract; nothing here suggests it will not
  transfer.
- **Occlusion pruning.** Untouched. It is a host-side mask over painter order and
  should be assessed with the fill spike, since it only pays where fills are.

  **Settled there (fm-orn, finding F13):** it is host-side as expected, and it is
  *not* an independent optimization — without §10.4's interior tile class writing
  exactly `1` for a covered command, a pruned frame differs from an unpruned one
  in the last bit of a channel. Implemented and measured: 16.5 % of commands
  removed, frames byte-identical.
- **3D, depth, lighting.** Untouched.
- **Multi-frame pipelining.** The gateway is synchronous by design (dispatch
  waits). §17.4 wants 2–4 annex frames in flight, which needs asynchronous
  completion in the gateway — a known, bounded upstream extension, deferred until
  a consumer needs it rather than speculated now.

---

## 6. The upstream contribution (ledger row 8)

`ft-kernel-metal::compute` — a generic Metal compute gateway: compile
caller-supplied MSL, allocate unified-memory buffers, dispatch a grid, read
back, with every `unsafe` Metal FFI call contained in frankentorch. Before it,
ft exposed only its own tensor kernels behind fixed entry points, so **"ft is the
only GPU gateway" and "the annex runs its own kernels" were contradictory** —
standing pressure on the D1 closure to admit a second, much larger GPU
dependency. It adds no dependency (`metal` was already ft's), is stubbed off
macOS so consumers compile everywhere, and let this spike keep
`#![forbid(unsafe_code)]`.

**Ritual status:** landed upstream as `5dbcc1d2` on frankentorch `main` (step 2
complete; `master` synced per that repo's branch policy). The `SUITE.lock` pin
bump, the Gauntlet diff, and the `class=ffi` allowlist rows for the Metal
closure are steps 3–5, owned by **fm-xsz** — which will also switch the spike
from its path dependency to a git dependency at the pinned rev, removing the
sibling-checkout requirement. Until then the spike consumes the crate by path,
under ADR-0003's non-shipped tier, with its own committed `Cargo.lock` and
`class=dev` allowlist rows.

---

## 7. Decisions recorded

- **OQ-10** (does CUDA-via-ft reach annex quality now, or wait on upstream ft
  device work?) — **resolved: it waits.** See **ADR-0007**. The ledger row
  carries the two named preconditions: the gateway's *buffer* API must
  generalize to device-resident handles with delta upload (unified memory is an
  Apple luxury; §17.6's CUDA playbook wants a resident scene), and the D1
  closure must rule on shipping precompiled PTX before any code is written. The
  dispatch API and the kernel itself port by construction.
- **OQ-12** (does G3 exercise its annex-preview fallback?) — **recommendation:
  no**, the annex is ready to be G3's preview path. The decision remains G3's
  under its declared fallback and R21; this note is the evidence, not the
  verdict.
- **D-22 stands unamended.** Every constraint held: standard-mode only, never
  certified; excluded from the core gates; measured under PG-A only; backend
  identity journaled (`AnnexReport` carries device, unified-memory status,
  threadgroup sizing, math mode, and both transfer sizes); frankentorch the only
  gateway; no wgpu.

---

## 8. Follow-ups filed

| Bead | What |
|---|---|
| fm-xsz | The Metal annex production backend — inherits this mapping; owns the §2.9 ritual steps 3–5 for ledger row 8 |
| fm-gw7 | The compiled render IR — consumes §3's table verdicts and F1–F6 |
| fm-4wt | The fast CPU engine — must act on F10 before choosing `f32` anywhere in the stroke SDF |
| fm-orn | Analytic-fill GPU mapping spike, before W5 commits to a fill layout (§5) — **done 2026-07-25**, `G0-8b-analytic-fill-ratification.md`, findings F11–F16 |
