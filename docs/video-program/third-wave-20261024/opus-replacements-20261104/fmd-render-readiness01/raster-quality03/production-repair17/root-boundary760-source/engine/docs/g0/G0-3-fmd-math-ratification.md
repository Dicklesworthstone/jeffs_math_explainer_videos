# G0-3 — fmd-math architecture: ratification note (fm-nh0)

**Status:** Ratified, 2026-07-23. Resolves **OQ-2** (ADR-0005) and issues
the **go** on the Appendix-G parameter approach. The executable form is
`spikes/g0-3-fmd-math` (18 tests green; the visual gallery under
`docs/g0/g0-3-renders/`); **fmd-math implements from this note without
consulting the spike's internals**, and its public API freezes at the
shape recorded here until G2 (R8).

## The proven engine shape

One pipeline, each stage a real seam:

```
source &str
  → parse           tokens → Node tree; EVERY node carries its byte span
  → classify        the eight atom classes; Bin→Ord degradation in context
  → layout(Ctx)     style (D/T/S/SS × cramped) threaded top-down;
                    Appendix-G constructions build MBoxes bottom-up
  → Layout          positioned {glyphs, rules, drawn paths}, each glyph
                    naming its FACE and carrying its source span (§11.3)
```

- **Spacing** is the TeX inter-atom table (thin 3 mu / medium 4 mu /
  thick 5 mu, 1 mu = 1/18 em at current size), medium/thick suppressed in
  script styles; font kerning applies between adjacent same-face Ord
  character glyphs.
- **Style propagation** is exact TeX: scripts go D/T→S→SS (subscripts
  cramped), fraction interiors go D→T→S→SS (denominators cramped); glyph
  sizes 1.0 / 0.7 / 0.5 (CM's 10/7/5 pt family).
- **Constructions implemented and structurally asserted:** ruled
  fractions (rule 15: axis-centered bar, 3θ/θ clearances), scripts
  (rule 18: σ13–σ19 shifts, the 4θ sub/sup clearance and ⅘x-height
  lift), radicals with indices (rule 11: ψ clearance, overbar,
  scriptscript index), `\left…\right` (rule 19: the 901/1000 coverage
  target), big-operator limits (rules 13/13a: ξ9–ξ13 gaps, display
  limits vs text side-scripts), and a `matrix` environment (`\vcenter`
  on the axis, 1.2 em baselines, 1 em columns).

## Verdict 1 — metrics synthesis: **GO**

The method: compile in the **published TFM fontdimen family**
(cmr10/cmsy10 σ, cmex10 ξ — exactly the parameters Appendix G consumes)
as em-valued constants, and **validate** them against geometry decoded
from the bundled faces by fmd-font. Measured on `cmunrm.ttf` (2048 upm):

| Parameter | Published | Measured | Δ |
|---|---|---|---|
| x-height (σ5) | 0.4306 | 883/2048 = 0.43115 | **+0.13 %** |
| axis height (σ22), from '+' | 0.2500 | 512/2048 = 0.25000 | **exact** |
| axis height, from '=' | 0.2500 | 512/2048 = 0.25000 | **exact** |

The bundled CM Unicode faces *are* the Computer Modern the TFM family
describes. fmd-math ships the full σ/ξ table as calibration constants
with these validation tests promoted to its own suite; per-face
recalibration only ever happens through the same measure-and-validate
seam.

## Verdict 2 — extensible delimiters (OQ-2): **drawn-path mainline**

Probed fact: CM Unicode maps **no** bracket-extension pieces
(U+239B…U+23AE all unmapped) and carries **no size-variant sets** — the
cmex10-style glyph-assembly repertoire does not exist in the bundled
fonts. Glyph assembly is therefore rejected as the mechanism (there is
nothing to assemble). The ruling (ADR-0005), proven in the spike at
three sizes:

1. **Natural glyph** when the authored delimiter covers the target;
2. **Uniform scale up to 1.25×** natural (stroke weight stays plausible);
3. **Parametric drawn-path construction beyond** — the *mainline*, not a
   fallback — stroke weights calibrated against the authored glyph so
   the seam at the threshold is invisible at a glance. No requested size
   can fail, by construction.

The spike draws parens (one closed quadratic contour each) and the
drawn-path radical; fmd-math generalizes the construction to brackets,
braces (three-lobe), vert bars, and angle brackets.

## Verdict 3 — multi-face layout is structural

`∑ ∫ ∏` do not exist in CM Unicode at all; they resolve through the Noto
Math fallback subset. Consequently **every positioned glyph names its
face** in the core output type — face selection is data, not a rendering
afterthought — and the layout engine consumes multiple `fmd-font::Font`
instances from day one.

## The frozen API sketch (until G2, R8)

```rust
parse(&str) -> Result<Node, MathError>          // spans on every node
enum MathError { UnsupportedCommand{name, span},// the ratchet's unit
                 Malformed{what, at}, UnmappedChar{ch, span} }
enum Style { Display, Text, Script, ScriptScript }
Engine::new(faces…) ; typeset(&str, Style) -> Result<Layout, MathError>
struct Layout { glyphs: Vec<PlacedGlyph>,       // face, gid, ch, x, y,
                rules:  Vec<PlacedRule>,        //   size, span
                paths:  Vec<PlacedPath>,        // drawn constructions
                width, height, depth }          // ems, y-up, baseline 0
```

Consumers: fmn-tex builds VMobjects from `Layout` (glyph outlines via
fmd-font + rules + paths are all already quadratic); fmd renders the
same `Layout` to HTML/PDF. The span map on `PlacedGlyph` is what
`isolate`/`t2c`/`TransformMatchingTex` consume — no second render.

## Visual review (vs LaTeX renders of the same input)

Gallery in `docs/g0/g0-3-renders/`. Verdicts: nested display fraction,
simultaneous scripts, ∑ limit stacks (display and text), and the 2×2
matrix — **at-least-as-good at a glance**; radicals and the three
delimiter sizes — **different-but-fine** (noted: drawn-paren waist a
touch narrow at 4-line sizes; overbar/up-stroke joint acceptable).
Formal Look-Gallery review rides G0-2/W6 as planned.

## Spike simplifications = fmd-math work items (not architecture)

Recorded so nothing silently becomes load-bearing: italic-correction
sourcing (TFM per-glyph values as calibration data — affects sup
placement on slanted bases); the char→math-codepoint mapping table
(`-` → U+2212 etc.); calibrated display-size big operators (the spike
scales uniformly); drawn constructions for brackets/braces/angles;
proper glue (the spike kerns fixed mu values); matrix `arraycolsep`
refinement; radical index kern constants; `\mathchoice`-class deferred
lists. None of these moves a seam.
