# Certified Determinism — the `--reproducible` User Guide

`--reproducible` is the user-facing switch for **certified determinism**
(plan §16.7; the normative enumeration is
[../INPUT_CLOSURE.md](../INPUT_CLOSURE.md)). This guide is the promise in
plain terms: what you get, what it costs, and what is deliberately outside.

## What it promises

From **the same content-hashed input closure**, a certified render produces:

- **bit-identical raw frames**,
- **bit-identical canonical PNGs**, and
- **bit-identical WAV audio**,

across the certified matrix — **linux-x86-64, linux-aarch64,
macos-aarch64** — **at any thread count, on any machine load, forever**. The
render also emits a **sidecar provenance manifest** recording the closure and
the output hashes, so a third party can verify a render without re-running it.

The input closure is enumerated exhaustively (INPUT_CLOSURE §3): scene
sources, engine identity and `SUITE.lock`, the pinned toolchain and SIMD
build tier, the fully-resolved config bytes, RNG seeds and substream layout,
content hashes of every asset and font actually read (bundled fonts
included), execution-engine/backend identities, locale/timezone, and the
capability policy. Two renders with the same closure hash are the same
render; a different closure hash names exactly what changed.

## What it requires

`--reproducible` is a CLI overlay that sets `determinism.mode: certified` and
`render.engine: cpu` — the certified CPU engine. Concretely, a certified
render:

1. **Uses bundled fonts only.** No system fonts, no fontconfig — the bundled
   faces are in the closure by content hash
   ([font_license_bundle.md](font_license_bundle.md)).
2. **Permits no asset fetching.** The `AssetFetcher` capability is off;
   every input asset must be local and content-hashed into the closure.
3. **Runs the owned arithmetic stack.** fmn-dmath transcendentals and
   canonical raster arithmetic — no FMA, no fast-math, fixed-order and
   lane-count-independent reductions (§10.5, ADR-0010).
4. **Runs from a pinned build.** Engine identity (commit or release build
   id) and the exact `SUITE.lock` contents are closure components; ad-hoc
   builds certify against their own identity, not someone else's.
5. **Refuses uncertified engines.** The Accelerator Annex backends
   (Metal/CUDA) are standard-mode only; requesting one in certified mode is a
   named configuration error, not a downgrade. `fmn doctor` rejects
   `determinism: certified` combined with an annex engine.

Warm caches change nothing: every cache key includes the complete semantic
inputs, so a hit is definitionally equivalent to a recompute, and certified
renders are bit-identical cold or warm (fmn-cache's never-an-oracle
discipline). `--clear-cache` is always safe to run
([cache_config_conventions.md](cache_config_conventions.md)).

## What it excludes by construction

- **ffmpeg products.** Encoded video (MP4/mov), audio mux, and transcode are
  equivalence-classed, never bit-promised — the encoder is a third-party,
  versioned, platform-specific binary outside the closure. The certified
  products are the native ones: raw frames, canonical PNG, WAV
  ([ffmpeg_optional.md](ffmpeg_optional.md)). Encode the certified y4m/PNG
  products afterward if you need a container; the manifest lists encoded
  artifacts as *uncertified*.
- **Cross-tier identity across different SIMD build tiers.** Bit-identity
  holds within one compiled tier across the certified matrix; tier-vs-scalar
  equivalence is proven per tier by the engine-equivalence harness. Two
  artifacts at different tiers are two certified identities (the tier is in
  the closure, C3/C7).
- **windows-x86-64 bit-certification.** Windows is functional from W1 and ruled
  out of the certified matrix by ADR-0019 until measured on native Windows
  hardware; one successor ADR reopens the question with that evidence.
- **Python scenes' own side effects.** Python scenes are Python programs:
  certification covers them only as far as the closure captures them (§15.5).
  Scene code that reads the network, the clock, or unlisted files steps
  outside the promise.
- **Anything the closure does not capture.** If a behavior difference traces
  to an input outside the enumeration, that is an INPUT_CLOSURE bug — report
  it; the document is versioned and reviewed like a schema change.

## Verifying a render

Render twice from the same closure (any certified-matrix machines, any thread
counts) and compare the sidecar manifests: identical closure hash and
identical per-output SHA-256 entries for `raw_frames`, `canonical_png`, and
`wav` *is* the certificate. `fmn doctor --robot` reports the machine's
hardware tier, the binary's compiled tier, the resolved determinism mode, and
the execution plan, which is everything needed to reproduce the certified
environment.
