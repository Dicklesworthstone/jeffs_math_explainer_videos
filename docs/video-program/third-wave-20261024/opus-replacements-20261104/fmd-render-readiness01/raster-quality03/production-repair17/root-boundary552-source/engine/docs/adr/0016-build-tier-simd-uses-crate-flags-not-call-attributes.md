# ADR-0016 — Select SIMD build tiers with crate flags, not call attributes

**Status:** Accepted
**Date:** 2026-07-28
**Bead:** fm-4wt (the fast CPU engine and SIMD build tiers)
**Amends:** D-15 and §17.3's function-level `#[target_feature]` mechanism. The
build-tier, `std::simd`, governed-feature-set, safe-code, and no-runtime-dispatch
decisions remain unchanged.

## Context

D-15 assumed target-feature-1.1 would make a safe function carrying
`#[target_feature]` safely callable whenever the same feature was enabled in
the build configuration. The exact pinned compiler disproves that premise.
With rustc `1.98.0-nightly (c397dae80 2026-07-02)`, this invocation:

```text
rustc --edition 2024 -C target-feature=+avx2,+bmi2,+fma \
  --emit=metadata -o /dev/null -
```

still reports E0133 when ordinary safe code calls a safe
`#[target_feature(enable = "avx2")]` function:

```text
the avx2 target feature being enabled in the build configuration does not
remove the requirement to list it in #[target_feature]
```

Annotating the caller only moves the obligation outward. Eventually a public
safe boundary, trait method, or entry point must call the annotated function,
and that call is unsafe unless its own context carries the feature. The
[Rust Reference](https://doc.rust-lang.org/stable/reference/attributes/codegen.html#the-target_feature-attribute)
also defines safe calls in terms of the caller's target-feature context, not
the artifact's build flags. Because every authoritative crate has
`#![forbid(unsafe_code)]`, a function-attribute dispatch chain cannot provide
the promised safe library boundary.

The build tier already gives a simpler proof. SUITE.lock fixes one feature set
for the entire artifact. Code selected by `cfg(target_feature = "...")` is
compiled only into that artifact, and W11 selects an artifact before launch.
There is no binary-local choice to make at a call site.

## Decision

The old D-15 text was:

> Exact pinned nightly; scalar-first as ordering; **SIMD is `std::simd` with
> safe `#[target_feature]` in build tiers** governed by SUITE.lock's certified
> target-feature sets, under the §10.5 lane rules — the abstraction is now
> named, and no per-call `unsafe` dispatch exists.

It becomes:

> Exact pinned nightly; scalar-first as ordering; **SIMD is `std::simd` in
> crate-wide build tiers** selected by SUITE.lock's certified target-feature
> sets and `cfg(target_feature)`, under the §10.5 lane rules. Authoritative
> code uses neither function-level `#[target_feature]` nor runtime feature
> detection, and no per-call `unsafe` dispatch exists.

Concretely:

1. CI and W11 build each artifact with exactly one SUITE.lock feature set:
   portable, x86-64-v3, x86-64-v4, or aarch64 + NEON.
2. `cfg(target_feature)` selects the compiled tier and its `std::simd` kernels.
   Dispatch between the scalar oracle and the compiled tier occurs once at the
   frame/kernel boundary, never per element and never by runtime detection.
3. Function-level `#[target_feature]` is forbidden in authoritative code. It
   does not create a safe boundary on the pinned compiler and is unnecessary
   when every shipped binary is already ISA-specific.
4. Certified kernels remain lane-count-independent and bit-equal to the scalar
   definition. A build flag may permit FMA at the ISA level for standard mode,
   but certified source still forbids `mul_add` and rustc's strict floating
   operations do not contract by default.

## Consequences

- The project keeps `#![forbid(unsafe_code)]` without relying on a compiler
  behavior the pinned compiler does not implement.
- A build cannot dynamically widen itself on a newer CPU. W11 must select the
  correct artifact before execution, and the selected tier remains part of the
  input closure.
- Portable, v3, v4, and NEON artifacts compile different code from the same
  sources. `Tier::ALL` exposes the scalar oracle plus the artifact's one
  compiled tier, and the engine-equivalence harness compares those certified
  routes byte-for-byte.
- Per-tier CI must use the exact SUITE.lock flags in its job environment and
  keep tier-specific build caches. Partial or improvised feature sets are not
  supported tiers.
- This ADR and the §17.3 / D-15 plan true-up land with fm-4wt; no follow-up
  compatibility wrapper or unsafe dispatch layer is created.
