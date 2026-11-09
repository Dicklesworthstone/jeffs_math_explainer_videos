# The Behavior Notes — the register (§16.8)

The user-facing register of deliberate differences from classic manim: one
evidence-backed note per entry, **written as migration guidance, not as a
conformance ledger**. If your manim scene behaves differently under
FrankenManim, the reason is here, and so is what to do about it.

This file is the authoritative index. Plan §16.8 seeds the register; the
register grows past its seed as workstreams land, and §16.8 is trued up when it
does (ADR-0009).

## The numbering rules

Deliberately the ADR discipline, because a Behavior Note is cited by number
from the Parity Ledger's `evidence` column, from crate documentation, and from
migration guidance users read:

1. **One number per note.** Numbers are assigned monotonically, **never
   reused**, and never renumbered once anything outside the note cites them.
2. **A number may name a family** when the plan's own register entry does. BN-07
   is the standing example: §16.8 defines it as "Reference bugs fixed (Appendix
   C rulings)", and Appendix C carries eleven rulings owned by different
   workstreams, so BN-07 is several notes under one topic rather than one
   document that would have to be rewritten by every workstream that touches it.
3. **A reserved number stays reserved.** BN-06 was held for the renderer and
   written when Lumen's stages landed (W5, fm-5oi and fm-oac) — not filled with
   unrelated content in the meantime, which is how registers rot. There are no
   reserved numbers outstanding.
4. **Status vocabulary:** `Draft` → `Final`. A note goes Final when its
   subsystem's gate passes and its migration guidance has been reviewed against
   the real behaviour, not when the code lands.

## The register

| # | Note | File | Workstream | Status |
|---|---|---|---|---|
| BN-01 | One RNG: PCG64DXSM with named substreams; seeded scenes reproduce within FrankenManim, not across engines | [BN-01-single-rng.md](BN-01-single-rng.md) | W1 | Draft |
| BN-02 | The rational clock on manim's nominal sample points — no drift | [BN-02-rational-clock.md](BN-02-rational-clock.md) | W4 | Final |
| BN-03 | True arc length under the original names: constant-speed paths, length-true dashes and tips | [BN-03-true-arc-length.md](BN-03-true-arc-length.md) | W2 | Draft |
| BN-04 | **Colour** — a family, see rule 2: linear-light compositing with manim's gradient formulas kept; callable color fields on every native record schema | [compositing and gradients](BN-04-color.md) · [callable color fields](BN-04-functional-color-fields.md) | W1/W10 | Draft |
| BN-05 | Native typesetting: metrics differ from LaTeX; the quality bar is documented | [BN-05-native-text-typesetting.md](BN-05-native-text-typesetting.md) | W6 | Draft |
| BN-06 | **The renderer** — a family, see rule 2: analytic coverage, round caps, arc-length stroke width | [the fill: analytic coverage, a defined gradient field, a border that does not grow the shape (fm-5oi)](BN-06-analytic-fill.md) · [strokes: curve distance, round caps, joins that mean what they are called (fm-oac)](BN-06-strokes.md) | W5 | Draft |
| BN-07 | **Reference bugs fixed** (Appendix C rulings) — a family, see rule 2 | [stroke uniforms (C-2, C-7)](BN-07-stroke-uniform-fixes.md) · [updaters, group addition, grid sizing, FunctionGraph color, plane unit size, OldTex construction, text color keyword, live TeX numbers, subpath ends after match_points, partial plane styles, zero-length proportions, restored point-less styles, the background rectangle's fill getter, point-less match_style (C-5, C-6, C-14, C-15, C-17, C-18, C-19, C-20, C-21, C-23, C-24, C-26, C-29, C-30, C-31)](BN-07-updater-and-group-fixes.md) · [frame-marker constants are unit directions (C-16)](BN-07-frame-marker-constants.md) · [functional movement lifecycle and flow replay](BN-07-functional-movement.md) | W3/W10 | Draft |
| BN-08 | The de-TeX'd classes (§11.6): Brace, Matrix delimiters, DecimalNumber, the drawn marks | [BN-08-de-texed-natives.md](BN-08-de-texed-natives.md) | W7 | Draft |
| BN-09 | One arc-density rule, never coarser than the Reference's | [BN-09-arc-density.md](BN-09-arc-density.md) | W2/W7 | **Final** |
| BN-10 | Skip mode delivers the same updater time and clock as playback (§9.3, C-28) | [BN-10-skip-mode-updater-time.md](BN-10-skip-mode-updater-time.md) | W4 | Draft |
| BN-11 | Composition honours its declared timing (C-10, C-11) | [BN-11-composition-timing.md](BN-11-composition-timing.md) | W4 | Draft |
| BN-12 | The Animation contract's typed edges (§9.1) | [BN-12-animation-contract.md](BN-12-animation-contract.md) | W4 | Draft |
| BN-13 | Cubic curves use one C1, error-bounded reduction; the simple shortcut cannot lower fidelity | [BN-13-error-bounded-cubic-reduction.md](BN-13-error-bounded-cubic-reduction.md) | W2 | Final |
| BN-14 | Sound is sample-exact on the rational frame clock; pre-zero audio clips instead of raising | [BN-14-sample-exact-sound.md](BN-14-sample-exact-sound.md) | W8 | Draft |
| BN-15 | The CLI is generated, validated, capability-aware, and machine-readable | [BN-15-cli-contract.md](BN-15-cli-contract.md) | W9/W10 | Draft |
| BN-16 | Polygon default corner rounding measures the complete cyclic edge set | [BN-16-polygon-corner-radius.md](BN-16-polygon-corner-radius.md) | W10 | Draft |
| BN-17 | Image sampling rejects every outside point; image acquisition is explicit and bounded | [BN-17-image-input-and-sampling.md](BN-17-image-input-and-sampling.md) | W10 | Draft |
| BN-18 | `PMobject.ingest_submobjects` consumes children instead of double-drawing stacked points | [BN-18-pmobject-ingest.md](BN-18-pmobject-ingest.md) | W10 | Draft |
| BN-19 | InteractiveScene gestures without a window toolkit | [BN-19-interactive-editing.md](BN-19-interactive-editing.md) | W9/W10 | Draft |
| BN-20 | Calculus helpers use transformed axes, bounded intervals and signed-area semantics | [BN-20-calculus-geometry.md](BN-20-calculus-geometry.md) | W7/W10 | Draft |
| BN-21 | Live surfaces retain the axes chart; wireframes refresh in place | [BN-21-live-surface-plotting.md](BN-21-live-surface-plotting.md) | W7/W10 | Draft |
| BN-22 | Bounded surface construction and first-error callback execution | [BN-22-surface-construction-admission.md](BN-22-surface-construction-admission.md) | W7/W10 | Draft |
| BN-23 | First-error execution for native-backed mathematical plots | [BN-23-graph-construction-admission.md](BN-23-graph-construction-admission.md) | W7/W10 | Draft |
| BN-24 | Live affine axes invert their chart instead of projecting onto each axis | [BN-24-live-affine-coordinate-inversion.md](BN-24-live-affine-coordinate-inversion.md) | W7/W10 | Draft |

BN-10 and BN-12 through BN-24 grew past §16.8's seed list, which is expected — the
seed names the differences the plan could foresee, and a workstream that finds
another deliberate divergence writes it a note rather than filing it nowhere.
ADR-0009 trues §16.8 up to this table and fixes the numbering rules above so
the next addition does not need an ADR of its own.

## Citing a note

The Parity Ledger's `evidence` column carries the bare number (`BN-08`), which
is why rule 1 exists: a number that names two documents makes the ledger
ambiguous exactly where it is meant to be authoritative. Ledger cells are
generated from `API_OVERLAY.tsv`'s `[status]` section — edit the overlay, then
regenerate with

```
UPDATE_API_ARTIFACTS=1 cargo test -p fmn-conformance --test api_schema
```

Crate documentation cites notes inline (`grep -rn 'BN-0' crates/`), and those
citations are the reason a note that anything references is never renumbered.
