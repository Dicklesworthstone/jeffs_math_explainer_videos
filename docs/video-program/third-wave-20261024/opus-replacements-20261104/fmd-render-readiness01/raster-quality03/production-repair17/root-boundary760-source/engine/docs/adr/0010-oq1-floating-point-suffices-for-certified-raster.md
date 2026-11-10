# ADR-0010 — OQ-1: floating-point suffices; the fixed-point raster boundary is not adopted

**Status:** Accepted
**Date:** 2026-07-25
**Bead:** fm-zn9 (G0-6, the determinism spike)
**Resolves:** OQ-1 — "Floating + fmn-dmath vs fixed-point raster boundary for
certification."
**Amends:** nothing in the decision log. §10.5 named both options and left the
choice to this gate; this records which one the measurement chose, and retires
the other.

## Context

§10.5 sketches a fallback the entire program is shaped around. If floating
screen-space arithmetic cannot be made bit-identical across platforms, the
certified engine adopts a **canonical fixed-point raster boundary**: fixed-point
screen coordinates, integer coverage accumulation at a defined precision,
explicit rounding. That is not a tuning knob — it is a different renderer, with
different kernels, different tolerances, and different self-goldens. §20.1 put
the decision in G0 precisely because making it after W5 exists would invalidate
every golden already locked.

The hypothesis under test: `f64`/`f32` screen-space arithmetic **plus** fmn-dmath
transcendentals **plus** fixed-order reductions **plus** no FMA is already
bit-identical across platforms, making the fixed-point boundary unnecessary.

## Evidence

One frame — gradient fills under the nonzero winding rule, a filled ring whose
hole depends on winding direction, strokes with joins and arc-length tapers,
per-vertex `atan2` joint angles that reach pixels, the Reference's `true_dot`
glow profile with its `pow` falloff, rate functions driving the geometry, alpha
compositing throughout — rendered at 480×270 and hashed through fmn-hash's
canonical container, alongside per-function digests of all sixteen fmn-dmath
entry points over fixed grids.

| Platform | libc | ISA | Execution | Record digest |
|---|---|---|---|---|
| linux-x86_64 | glibc | x86-64 | native | `cca55de642d54f2e…` |
| linux-aarch64 | musl | aarch64 | qemu-user | `cca55de642d54f2e…` |
| macos-aarch64 | Darwin libSystem | aarch64 (M4 Pro) | native | `cca55de642d54f2e…` |

**All eighteen digests are identical on all three platforms** — the `f64` frame,
the `f32` frame, and every one of the sixteen fmn-dmath functions. Raw data:
`docs/g0/g0-6-hashes/`. Methodology and caveats:
`docs/g0/G0-6-determinism-ratification.md`.

The three legs are chosen to vary things independently: x86-64-glibc against
aarch64-Darwin crosses **both** the ISA and the OS boundary at once, while
aarch64-musl shares an ISA with one and an OS family with the other. Agreement
across all three is not one comparison repeated; it is the ISA, the libc, and
the operating system each shown not to be in the loop.

**No-FMA evidence.** `llvm-objdump -d` on the aarch64 build: 1652 scalar
floating-point instructions, **zero** `fmadd`/`fmsub`/`fnmadd`/`fnmsub`/
`fmla`/`fmls`, on a target where FMA is baseline and the compiler would
contract if permitted. The scalar-op count is reported because it is what makes
the zero meaningful — an earlier pass used `objdump`, which cannot disassemble
aarch64 and reported zero of everything.

## Decision

**Floating-point suffices for the certified path. The canonical fixed-point
raster boundary of §10.5 is NOT adopted.** No precision or rounding constants
are specified, because none are needed.

The result stands on four properties, and it is only as durable as they are —
so they are hereby binding on W5 rather than incidental:

1. **fmn-dmath owns every transcendental on the certified path.** This is what
   removes the platform libm from the loop, and it is the load-bearing one: the
   three platforms have three different libms and agreed anyway.
2. **No FMA contraction** (§10.5d), verified in object code, guarded in source.
3. **Fixed-order reductions** (§10.5c). Trivially satisfied today — the
   prototype is scalar and every accumulation runs in index order — and this
   ADR is where that stops being trivial: the SIMD tiers of §17.3 must keep it.
4. **IEEE-754 basic operations**, which are correctly rounded by the standard
   and therefore identical everywhere by construction. Nothing on the certified
   path may use an operation weaker than that.

## Consequences

- **fm-ig3** (the certified CPU engine) builds on floating arithmetic. The
  fixed-point design is retired, not deferred: it should not reappear in a W5
  review as an unresolved option.
- **fm-4wt** (the fast CPU engine) inherits a sharper obligation than before.
  The `f32` frame *also* hashed identically across all three platforms, so f32
  is portable — but G0-8 measured that f32 changes the *picture* at curvature
  extrema. Portable and correct are different questions, and §6.1's
  mixed-precision licence is bounded by the second, not the first.
- **The SIMD tiers are now the risk.** Every property above survives
  vectorization only if §17.3's lane rules are honoured; PG-5's per-commit
  {1,4,16}-thread sweep tests schedules, not lane counts. W5 should add a
  tier-vs-scalar bit-identity check when the first tier lands.
- **The frame is bit-locked as a cross-platform self-golden** in the spike's own
  tests, so the property fails on any single machine the moment it breaks,
  rather than waiting for someone to re-run the sweep.
- **One caveat, stated rather than buried:** the linux-aarch64 leg runs under
  qemu-user emulation, because no aarch64 Linux hardware is reachable. QEMU's
  softfloat is IEEE-754 correct for the basic operations, and fmn-dmath uses
  only those plus its own polynomials, so the leg is valid evidence about the
  *arithmetic*. It is not evidence about an aarch64 Linux *machine*. The G0-6
  note carries this, and W1's CI matrix (fm-sol) should replace it with real
  hardware when any is available.
- **windows-x86-64 is unaffected** by this ruling. Its matrix membership was
  OQ-6's separate declared decision — now resolved by **ADR-0019**: out of the
  certified matrix until measured on native Windows hardware. See
  `docs/INPUT_CLOSURE.md` §5 for the frozen list.
