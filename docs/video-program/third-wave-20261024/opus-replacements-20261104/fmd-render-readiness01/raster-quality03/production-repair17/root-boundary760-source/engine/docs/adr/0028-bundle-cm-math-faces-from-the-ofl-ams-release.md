# ADR-0028 — Bundle Computer Modern's math faces from the OFL AMS release

**Status:** Proposed
**Date:** 2026-10-05
**Bead:** fm-tex-metrics-glyphs-ru72
**Amends:** §11.1's bundled face list (D-08's "bundled Computer Modern" is
unchanged in intent). The §11.4 metrics table stays synthesized.

## Context

fm-tex-metrics-glyphs-ru72 measured native `Tex` formulas 2-18% narrower than
the pinned Reference, and with visibly different glyphs. The bead comment of
2026-10-05 has per-glyph traces. Upright glyphs agree (digits within 0.5%). The
rest of the gap comes from the faces themselves:

- **Math letters** are set in CM Unicode's *text* italic (cmunti). TeX sets them
  in cmmi10, a different, wider design. For example, cmmi10's f advances
  0.49 em against cmti's 0.31 em, and 'x' inks at 0.93x the Reference. α, β and
  γ read as Latin letters (fm-5wq.60).
- **Relations and operators** (≤ ≥ ≠ ≡ ≈ ∈ ⊂ ∫ …) are missing from CM Unicode and
  fall through to Noto Sans Math, a heavier sans design.
- **Scripts** are a 0.7x scale of the 10 pt face. TeX uses the optically
  designed 7 pt and 5 pt sizes.
- **cmex pieces** are absent, so ADR-0005 draws delimiters past 1.25x.

Rule 17's italic corrections (franken_markdown 4c8caa2, UPSTREAM_LEDGER row 20)
were the face-independent part. They lifted the Reference box oracle from 26 to
39 formulas within 5%. What remains needs the faces TeX actually uses. That
means a new bundled asset, which is a licensing and closure decision. The
§11.4 quality bar is "indistinguishable at a glance from LaTeX output".

## Decision (proposed)

1. **Source: the AMS Type 1 Computer Modern release, SIL OFL 1.1.** The faces
   are:
   - cmmi10, cmmi7 and cmmi5 (math italic);
   - cmsy10, cmsy7 and cmsy5 (symbols and relations);
   - cmex10 (extension);
   - cmr7 and cmr5 (upright script sizes).

   Bold math (cmmib10, cmbsy10) is a later tier, scheduled by corpus rank. These
   are the designs the Reference's LaTeX path renders, so they meet the look bar
   by construction. They keep the bundle single-licence: all OFL, like CM
   Unicode, Plex and Noto.
2. **Offline, one-time conversion to TrueType glyf outlines.** The conversion
   uses a fixed, recorded cubic-to-quadratic tolerance of at most 1/1000 em, so
   the runtime keeps its single mature outline path (glyf, quadratic, §11.4
   output). The tool, its version, the tolerance and the input and output
   SHA-256 enter `dist/FONT_BUNDLE.json`'s provenance. No font tooling is ever
   invoked at runtime, so D-02 is untouched.
3. **Renamed derivatives.** The AMS release reserves its font names (cmmi10,
   cmex10, …), so the converted files and their name tables carry new names, for
   example "FMD CM Math Italic 10". The OFL text and the original copyright line
   ship alongside, under `dist/licenses/fonts/`.
4. **Role mapping in fmd-math.**
   - Math letters: cmmi. This replaces FACE_ITALIC's cmunti role.
   - Relations, binary operators and arrows: cmsy.
   - Big operators and delimiter variants: cmex. Drawn paths remain ADR-0005's
     fallback beyond cmex's largest variant.
   - Script and scriptscript styles: the 7 and 5 sizes.

   Noto Sans Math stays the last-resort fallback, and every fallback use is
   counted in the ratchet.
5. **Metrics.** Advances come from the faces; the Type 1 widths match the TFM
   widths. Italic corrections and Appendix-G parameters stay synthesized
   (§11.4), and the bead's width oracle checks them. Embedding TFM-derived
   italic-correction tables is a separate follow-up: it requires its own licence
   check of Knuth's CM metric files.

**Alternative considered: Latin Modern Math** (one OpenType font with CFF
outlines, a MATH table and `ssty` optical variants; the GUST Font License, an
LPPL instance). It is technically convenient, but it brings a second licence
family into the bundle. Modified versions must be renamed under the GFL, and
fmd-math would need cubic outlines on its path pipeline. **Rejected for now:**
the AMS faces are closer to what the Reference renders and stay within OFL.

**Rejected:**
- drawn-path relation glyphs alone, because they fix item 2 only;
- the status quo with a Behavior Note, because it fails §11.4's quality bar.

## Consequences

- **Upstream (franken_markdown):** fmd-font bundles the converted faces with
  provenance, and fmd-math remaps its roles. The layout goldens move, and each
  diff is adjudicated per glyph. The franken_manim repin follows the SUITE.lock
  ritual; scene goldens and PG-5, PG-6 and PG-7 digests move with adjudication.
- **franken_manim:**
  - the ru72 acceptance items that need CM-family glyphs become reachable: glyph
    source asserted for ≤ ≥ ≠, and `f(x)` within 2% of TeX's box;
  - the Tex oracle floors rise;
  - the `fm-tex-metrics-glyphs-ru72-extent` structural-facts envelope row
    becomes stale and is retired with evidence;
  - fm-5wq.60's Greek-letter regressions are expected to resolve.
- **Bundle size** grows by about 10 faces of roughly 20-40 KiB each after
  conversion. That counts against the wasm package budget (fm-8j70), and the
  wasm audit records the measured delta.
- **Plan true-up:** §11.1's face list gains "CM math (cmmi/cmsy/cmex, 10/7/5)"
  when this ADR is Accepted. The true-up lands with the acceptance commit; it is
  deferred while the ADR is Proposed.
