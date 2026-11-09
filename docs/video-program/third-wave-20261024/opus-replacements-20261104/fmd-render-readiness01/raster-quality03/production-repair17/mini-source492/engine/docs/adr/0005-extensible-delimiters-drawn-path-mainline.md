# ADR-0005 — OQ-2 resolved: extensible delimiters ride a drawn-path mainline

**Status:** Accepted
**Date:** 2026-07-23
**Bead:** fm-nh0 (the G0-3 fmd-math architecture spike)
**Amends:** resolves OQ-2 (extensible delimiters: CM glyph assembly,
drawn paths, or hybrid — and the metrics-synthesis calibration method;
owner G0-3)

## Context

TeX assembles oversized delimiters from cmex10's size-variant glyphs and
extension pieces. The G0-3 spike probed the bundled faces for that
repertoire: **CM Unicode maps none of it** — U+239B…U+23AE (all bracket
hooks/extensions) are unmapped, and no size-variant sets exist; the Noto
Math symbol subset carries only scattered pieces (⌠/⌡ without the
extension bar). Glyph assembly has nothing to assemble. Meanwhile §11.4
already required drawn-path construction as the universal fallback so no
requested size can fail.

## Decision

The delimiter mechanism, proven in `spikes/g0-3-fmd-math` at three sizes
and frozen for fmd-math:

1. **Natural authored glyph** when it covers the rule-19 target
   (delimiterfactor 901/1000 semantics kept);
2. **Uniformly scaled glyph up to 1.25× natural** — stroke weight
   remains visually plausible in that band;
3. **Parametric drawn-path construction beyond** — the *mainline*
   mechanism, not a fallback — quadratic contours whose stroke weights
   are calibrated against the authored glyph so the mechanism seam at
   the 1.25× threshold is invisible at a glance.

Glyph assembly is **rejected** for the bundled set (nothing to
assemble); it may be revisited only if a future bundled face ships a
real extension repertoire, via a new ADR.

The same ruling covers the radical sign past its natural size, and the
companion **metrics-synthesis method** the spike validated is recorded
with its measurements in `docs/g0/G0-3-fmd-math-ratification.md`: the
published TFM fontdimen family compiled in as em constants, validated
against fmd-font-decoded geometry (x-height within 0.13 %, axis exact).

## Consequences

fmd-math implements drawn constructions for the full delimiter family
(parens, brackets, braces, bars, angles) with per-family calibration
against the authored glyphs; delimiter sizing never fails by
construction (§11.4's promise becomes structural); the Look Gallery
reviews the threshold seam. The plan's §23 OQ-2 entry is trued up in
this commit. fmd-math's public API freezes at the spike's recorded shape
until G2 (R8) — see the ratification note.
