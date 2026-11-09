# ADR-0014 — fmn-dmath is the root of the crate DAG, because the funnel has to be reachable

**Status:** Accepted
**Date:** 2026-07-26
**Bead:** fm-ig3 (the certified CPU engine)
**Amends:** §19's crate map — the layering order of `fmn-core` and `fmn-dmath`,
and three crates' edge sets. No decision-log entry changes; D-17 ("fmn-dmath owns
certified transcendentals") is what this makes *possible* rather than what it
alters.

## Context

ADR-0010 resolved OQ-1 on measured evidence and made four properties binding, of
which the first is named "the load-bearing one":

> **fmn-dmath owns every transcendental on the certified path.** This is what
> removes the platform libm from the loop, and it is the load-bearing one: the
> three platforms have three different libms and agreed anyway.

fm-ig3 wrote the guard that checks it (`fmn-conformance/tests/certified_arithmetic.rs`)
and the guard found **sixteen live call sites**, in four crates, all of them
reaching pixels:

| Crate | Sites | What they decide |
|---|---|---|
| fmn-core | `srgb_eotf`/`srgb_oetf`'s `powf`, Oklab's `cbrt` | every colour, on the way in and on the way out |
| fmn-core | `wiggle`'s `sin`, `exponential_decay`'s `exp` | **α** — the rate functions every animation's geometry is a function of |
| fmn-geom | the cubic→quadratic converter's `cbrt` | a segment *count*, so a rounding difference changes how a curve is flattened |
| fmn-mobject | the trackers' `exp`/`ln` | tracked values that drive geometry |
| fmn-library | nine trig calls across the tip, arc and brace constructors | where points are |

Two of these were **inside the very frame G0-6 hashed**. `spikes/g0-8-accelerator/src/determinism.rs`
drives its geometry with `rate::wiggle` and `rate::exponential_decay`, and
converts every colour with `Srgb::to_linear`. So the three-platform agreement
G0-6 measured happened *despite* three libms being in the loop, not because they
were absent. The measurement stands — the digests are identical and the raw data
is committed — but the *argument* ADR-0010 made for it was stronger than the code
supported, and the gap was invisible because nothing checked.

Only one of the sixteen was a plain oversight. The other fifteen were
**unreachable from the funnel**: `fmn-core`, `fmn-mobject` and `fmn-library` had
no dependency edge to `fmn-dmath` at all, so no call site in them *could* route
through it. `fmn-core` had the worst version of the problem, because the edge it
would need pointed the wrong way: `fmn-dmath` declared a dependency on
`fmn-core`, so adding the reverse edge would have been a cycle.

That declared edge turns out to be **vestigial**. `fmn-dmath` uses nothing from
`fmn-core` — not a type, not a constant, not `canonicalize_f64`. Its kernels are
`f64::from_bits` coefficient literals evaluated in the published FDLIBM operation
order over its own range reduction. The dependency was inherited from the order
crates were stood up in, not from anything it consumes.

## Decision

**`fmn-dmath` becomes the root of the crate DAG.** It depends on nothing;
`fmn-core` depends on it. §19's listing order changes accordingly, and
`scripts/check_crate_dag.py` is trued up in the same commit.

Three crates gain the edge they needed:

| Crate | Edge added | For |
|---|---|---|
| `fmn-core` | `fmn-dmath` | the sRGB transfer functions, Oklab, and the rate functions |
| `fmn-mobject` | `fmn-dmath` | the exponential trackers |
| `fmn-library` | `fmn-dmath` | tip geometry, arc radii, brace angles |

All sixteen call sites now route through `fmn_dmath::*` (or `fmn-geom`'s
`scalar` funnel, which was already there and had one bypass).

The rule this generalizes to, stated so the next crate does not rediscover it:
**a crate that computes anything reaching a pixel needs an edge to `fmn-dmath`,
and `fmn-dmath` is below everything so that edge always points downward.**

`fmn-conformance/tests/certified_arithmetic.rs` is the enforcement. Its exemption
list is two entries long — `fmn-dmath` itself, whose `FAST` table names
`f64::sin` deliberately (§6.6: "`standard` may use fast paths"), and `fmn-python`,
whose PyO3 expansion is not ours to constrain — and growing that list is an ADR,
not an edit.

## Consequences

- **ADR-0010's evidence is unchanged; its argument is now true.** The G0-6
  digests still agree across three platforms. What changed is that the reason
  they agree is now the reason the ADR gives. Nothing needs re-measuring: every
  routed call moves a value by at most an ulp, and the whole test suite —
  including fmn-core's 418-row Reference colour parity fixtures — passed
  unchanged.
- **A latent portability hazard is closed, and it was not in the renderer.**
  `rate::wiggle` and `rate::exponential_decay` compute **α**, which every
  animated frame's geometry is a function of. A one-ulp libm difference there
  moves points, not just the last bit of a channel. This is the single most
  consequential of the sixteen and it lived in the substrate, two subsystems away
  from anything that looked like arithmetic.
- **The certified-vs-fast seam is still available and still unused.**
  `fmn_dmath::CERTIFIED` and `fmn_dmath::FAST` are function tables selected at
  engine construction, and §6.6 permits `standard` to take the fast one. Nothing
  in the tree does yet; the routed call sites take the certified functions
  directly, which is what `fmn-geom`'s funnel already documented as the right
  default for object-space semantics ("the standard/fast seam is a renderer-side
  choice and deliberately not plumbed here").
- **The cost is one edge and no code motion.** No file moved crates, no API
  changed, and `fmn-dmath` shrank by a dependency.
- **Forbidden:** re-adding `fmn-core` to `fmn-dmath`'s dependencies. It would
  re-create the cycle that made the hole unfixable, and it would do so silently,
  because nothing in `fmn-dmath` would fail to compile without it.
