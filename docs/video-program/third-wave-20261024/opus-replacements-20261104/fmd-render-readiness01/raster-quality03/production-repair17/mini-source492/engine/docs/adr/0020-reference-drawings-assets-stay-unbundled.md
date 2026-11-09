# ADR-0020 — Reference-derived drawings assets stay unbundled; the classes take user-supplied files

**Status:** Accepted
**Date:** 2026-08-25
**Bead:** fm-7lx1 (W11/W7)
**Resolves:** the bundled-asset question surfaced by fm-3kr's census recon
(2026-08-25, FuchsiaDove)
**Amends:** nothing. This *applies* §15.3's corpus policy (and risk row R13's
CC BY-NC-SA fixture rule) to W7's drawings shelf; it invents no new doctrine.

## Context

fm-3kr's drawings census splits cleanly: `Clock`, `DieFace`, `Dartboard`,
`Speedometer`, `Piano`/`Piano3D` are pure Marionette+Chisel compositions and
landed as tranches 1–2 without any asset question. The remaining shelf —
`Lightbulb`, `VideoIcon`/`VideoSeries`, the Bubble family
(`Bubbles_speech.svg`, `Bubbles_double_speech.svg`, `Bubbles_thought.svg`),
and `VectorizedEarth` — are `SVGMobject` subclasses whose default appearance
is an **asset file loaded from the Reference repository**, and those files are
video-content derivatives under **CC BY-NC-SA**: the same license class, and
therefore the same private-fixture policy, as the gallery captures governed by
§15.3 and R13.

Bundling them is not an attribution gap that a manifest could paper over.
**NC** forbids the commercial use this project's MIT+rider license grants;
**SA** makes any shipped derivative a share-alike obligation the MIT artifact
cannot carry; and the font-bundle discipline (`docs/dist/font_license_bundle.md`)
exists precisely so every shipped byte has a complete, permissive license
inventory. There is also a mechanical guard to honor: the bundle manifest's
no-corpus-leak tooth checks shipped bytes against Reference-capture digests.

## Decision

1. **OUT of every shipped artifact.** No Reference-derived SVG enters the
   binary, wheel, npm package, `dist/`, or any compiled include — now or at
   any later gate. This is settled by license class, not by case-by-case
   review of individual files.
2. **Asset-backed classes resolve user-supplied files.** Each family takes
   its SVG path (VideoSeries: a directory or explicit file list) through the
   live SVGMobject loading mechanism (fm-5wq.4.50). Default constructors
   resolve through one shared policy resolver that fails with a **precise,
   named capability error**: the missing file name, the owning policy
   (§15.3 / this ADR), and the remedy — point at your own legally obtained
   3b1b checkout via the asset-root configuration. No silent substitution,
   no placeholder art, matching D2's fail-closed posture for absent tools.
3. **Original replacement artwork is the sanctioned IN-path for defaults.**
   Native-drawn (or commissioned) equivalents may land later as deliberate
   Behavior-Noted replacements under D-05 — each with its own bead, its own
   provenance, and full ownership of the new default. Until such an ADR
   lands, these classes exist with their full API, positional, and animation
   semantics but **no bundled default art**.
4. **The refusal path is tested, and the bundle stays provably clean.** When
   the first asset-backed class lands (fm-3kr tranche 3): unit tests assert
   the named-refusal behavior for every family, and the conformance suite's
   no-corpus-leak check gains the digests of the four asset families so a
   future accidental `include_str!`/copy fails CI mechanically.

## Consequences

- **fm-3kr's asset half unblocks with this shape:** implement the families as
  thin resolvers over SVGMobject loading per Decision 2 — API parity without
  shipped bytes. `Lightbulb`'s glow, the Bubbles' tail variants, and
  VideoSeries' frame enumeration operate on whatever file the user supplies.
- No change to `dist/FONT_BUNDLE.json` or the license inventory; the
  drawings assets never appear in any manifest because they never ship.
- README Limitations gains its bullet when the classes land (a Behavior
  Note), not before — this ADR governs the mechanism; user-facing copy ships
  with the feature.
- Revisit trigger: original replacement artwork commissioned → one successor
  ADR rules the new defaults IN with provenance and updates the census note
  in `crates/fmn-library/src/drawings.rs`.
