# Reviewer: engine feasibility (R1 only)

You check every planned picture against what franken_manim can actually do
at the pinned revision (see `FRANKEN_MANIM.pin` and the root `Cargo.toml`),
through this repository's kit (`crates/explainer_kit`). The franken_manim
skill (`~/.claude/skills/franken-manim-mega-skill/`) has the API, the
cookbook and the known engine bugs; read its STATUS-LEDGER and
AUTHORING-COOKBOOK before judging. Code wins over docs: when in doubt, read
the source at the pin (cargo's git checkout of franken_manim, or
`~/projects/franken_manim` at that revision; never edit it).

## Rules the chapters must obey (plan §6.6)

- Motion by placement only (shift, rotate); never rewrite a shape's points
  per frame.
- Never transform a group of more than about 2,000 members per frame
  (positional ops dedupe in O(M²)).
- `always_redraw` for small readouts only; grids, maps and tables are fixed
  mobjects recoloured in place, one VMobject per row strip.
- Chapters at most about 85 s; a 3D shot is its own camera-route segment
  through the kit's encoder (spike S-5 proved the route; S-9 the join).
- Glyphs: what the S-7 probe passed (plan §6.5; the briefs' Notation rows).

## The spikes

Done: S-1, S-5 (camera route, 44×11 lit bars at 4K60), S-7 (glyphs), S-9
(two-route chapter master). Open: S-2 (600–1,500 fish), S-3 (256×256 table as
row strips), S-4 (4,845 netful glyphs), S-6 (322 staircases), S-8 (arena
growth over 80 s). A beat that depends on an open spike names its fallback.

## How to read

For each beat whose picture or features column asks for something
non-trivial, say: feasible as written / feasible with a change (say which) /
not feasible (say the fallback). Estimate the mobject counts and per-frame
work for the heavy beats. Flag anything that would break the 4K60 render
budget (this Mac renders Hoeffding's 4K60 at about 15 fps with x264).
