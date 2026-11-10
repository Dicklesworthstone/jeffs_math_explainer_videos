# G0-6 — Determinism: the certified arithmetic, decided (fm-zn9)

**Status:** Ratified, 2026-07-25. **OQ-1 resolves in favour of floating-point**
(ADR-0010); §10.5's canonical fixed-point raster boundary is **retired, not
deferred**. The certified matrix is frozen into `docs/INPUT_CLOSURE.md` §5.

The executable form is `spikes/g0-8-accelerator` — the prototype renderer both
G0-8 and G0-6 needed — with `src/determinism.rs` and the
`g0_6_determinism` binary. Raw data: `docs/g0/g0-6-hashes/`.

---

## 1. The result

| Platform | libc | ISA | Execution | Record digest (of the 18 rows) |
|---|---|---|---|---|
| linux-x86_64 | glibc | x86-64 | native | `cca55de642d54f2e…` |
| linux-aarch64 | musl | aarch64 | qemu-user | `cca55de642d54f2e…` |
| macos-aarch64 | Darwin libSystem | aarch64 (Apple M4 Pro) | native | `cca55de642d54f2e…` |

**Every one of the eighteen digests is identical on all three platforms** — the
`f64` frame, the `f32` frame, and each of the sixteen fmn-dmath functions.

```
frame.f64   3ff2cac55c33a8b2f460b5e3d338a542a736a6093f9519b712535e31bbe675f7
frame.f32   9e94bf5e95765eca5927178ac70da2fd21133d1a4f173cafa0c2428042595698
```

The three legs vary things independently rather than repeating one comparison:
x86-64-glibc against aarch64-Darwin crosses **both** the ISA and the OS boundary
at once; aarch64-musl shares an ISA with the second and an OS family with the
first. Agreement across all three shows the ISA, the libc, and the operating
system each not to be in the loop.

---

## 2. What the frame contains, and why each piece is there

fm-zn9 enumerates the contents. Every one is a place two platforms could
disagree, and every one reaches pixels — a feature that does not affect the
output cannot be tested by a frame hash.

| Element | What it exercises |
|---|---|
| Gradient-filled disc, radius driven by `smooth` | nonzero-winding coverage, per-pixel gradient, a rate function in the geometry |
| Gradient-filled **ring**, inner subpath wound backwards | the winding *rule* — a crossing-direction or sort-order bug renders a solid disc instead of a ring |
| Zigzag stroke with `miter_gain` on | **per-vertex `atan2`**, through the joint-angle stand-in, so a wrong `atan2` moves the hash |
| Tapered gradient stroke over the fills | arc-length width and colour interpolation, alpha compositing in painter order |
| Five glow discs, alternating `glow_factor` 2.0 / 0.0 | the Reference's `true_dot` profile **and its `pow` falloff** — the only per-pixel `pow` in the picture — plus its hard-dot branch |
| Sub-pixel hairline | the regime where an AA profile error is most visible |
| `smooth`, `there_and_back`, `wiggle`, `exponential_decay`, `rush_into` | rate functions driving geometry, so `sin` and `exp` are in the frame, not just in a log line |

The frame is 480×270 on purpose: the linux-aarch64 leg runs under emulation at
roughly 30× native cost, and evidence nobody can afford to re-run is evidence
that rots. Bit-identity over 518 400 components either holds or it does not.

**The fmn-dmath shakedown is separate from the frame, deliberately.** fm-zn9 is
also "fmn-dmath's first accuracy/portability shakedown", and a single frame hash
cannot say *which* function moved. Sixteen per-function digests over fixed grids
— deliberately awkward grids, crossing argument-reduction boundaries, spanning
decades, including the `atan2` quadrant axes and signed zeros — mean a
divergence lands on a named row instead of on "the frame".

---

## 3. Methodology

**Hashing.** The surface goes through fmn-hash's versioned canonical container
(schema `FMND/1.0`), and every component through
`fmn_core::types::canonicalize_f32` at the boundary. That last part matters: it
makes `-0.0` and `+0.0` hash alike and collapses every NaN to the one canonical
NaN, so the spike cannot report a divergence between two runs that produced the
same picture. A test pins it.

**Reproducibility of the record itself.** The runner prints `key<TAB>value`
rows and nothing else — no timestamps, no timings, no host names — to stdout,
with everything human-facing on stderr. Two platforms' files therefore diff
cleanly, and a re-run on one platform is byte-identical to the last.

**Build.** The `SUITE.lock` pin `nightly-2026-07-05` on all three legs.
linux-aarch64 is cross-compiled from the x86-64 host to
`aarch64-unknown-linux-musl` (static, `rust-lld`, `link-self-contained`) and
executed through a `qemu-aarch64` binfmt handler. Compiling natively under
emulation would have taken ~30× longer for an identical binary.

**The musl choice is not a compromise, it is a stronger test.** If fmn-dmath
owns every transcendental on the certified path, the C library cannot influence
the result — and running one leg on musl, one on glibc and one on Darwin
libSystem is exactly how that claim gets tested rather than assumed.

---

## 4. The no-FMA evidence (§10.5d)

`llvm-objdump -d` on the aarch64 release build:

```
scalar FP instructions (fmul/fadd/fdiv/fsqrt)   1652
fmadd / fmsub / fnmadd / fnmsub / fmla / fmls      0
```

on a target where FMA is baseline and the compiler would contract if permitted.
x86-64: 1112 scalar FP ops, zero `vfmadd`/`vfmsub`/`vfnmadd`/`vfnmsub`.

**The scalar-op count is reported because it is what makes the zero
meaningful.** An earlier pass of this check used plain `objdump`, which on this
host cannot disassemble aarch64 at all: it emitted three lines and duly reported
zero of every instruction, including the ones that were certainly there. A
"zero" from a tool that found nothing is not evidence. The sanity counter is the
difference between a measurement and a comfortable number.

rustc performs no FP contraction by default, so this confirms a default rather
than a setting. The realistic regression is therefore a hand-written `mul_add`,
which `determinism::fma_guard` now refuses across the certified source roots.

**Fixed-order reductions (§10.5c)** are trivially satisfied today — the
prototype is scalar and every accumulation runs in index order. Stating that
plainly is more useful than claiming a property was verified: the fill's
crossing list is sorted on a *total* key so ordering cannot depend on sort
stability, and that is the only place an order could have crept in. Lane-count
independence becomes a real obligation when §17.3's SIMD tiers land, and
ADR-0010 makes it binding there.

---

## 5. What this decides, and what it does not

**Decided.** Floating-point suffices; no fixed-point boundary; no precision or
rounding constants; the certified matrix is linux-x86-64, linux-aarch64,
macos-aarch64, with windows-x86-64 functional-CI only under OQ-6.

**Not decided, and not implied:**

- **That `f32` is safe for the fast CPU engine.** The `f32` frame hashed
  identically across all three platforms, so f32 is *portable* — but G0-8
  measured that f32 changes the *picture* at curvature extrema. Portable and
  correct are different questions, and fm-4wt is bounded by the second.
- **That the SIMD tiers will preserve any of this.** They are now the risk.
  PG-5's per-commit sweep tests thread counts, not lane counts; W5 should add a
  tier-vs-scalar bit-identity check when the first tier lands.
- **That the shipped renderer is this renderer.** The fill here is exact in x
  and supersampled in y — a defined, deterministic stand-in, *not* §10.2's
  analytic area evaluator, which is fm-5oi's. The joint-angle widening is a
  stand-in for §10.3's join model, which is fm-oac's. Both are labelled as such
  in the source. What transfers is the *arithmetic* result, which is what OQ-1
  asked about; a spike that had claimed to be W5's renderer would have produced
  a much less useful answer to a question nobody asked.
- **That linux-aarch64 hardware behaves like linux-aarch64 emulation.** See §3.

---

## 6. The frame is now a cross-platform self-golden

The `f64` and `f32` digests are bit-locked as constants in the spike's tests.
This is the strongest form of the §16.2 self-golden rig: because the arithmetic
is portable, one constant serves every platform, so the property fails on
whichever machine breaks it rather than waiting for someone to re-run the
three-platform sweep. A drift is a finding to adjudicate, never a number to
re-bless (GOVERNANCE §5).

---

## 7. Follow-ups

| Bead | What |
|---|---|
| fm-sol | W1's CI matrix — replace the emulated aarch64 leg with real hardware if any becomes reachable; wire the three-platform sweep as a scheduled job |
| fm-ig3 | The certified CPU engine — builds on floating arithmetic; the four binding properties of ADR-0010 are its acceptance criteria |
| fm-4wt | The fast CPU engine — f32 is portable but not free; see G0-8 F10 |
| fm-yp0 | G4b — enforces the frozen matrix end to end |
