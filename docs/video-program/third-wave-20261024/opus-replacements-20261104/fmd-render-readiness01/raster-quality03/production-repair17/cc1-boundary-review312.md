# CC1 review 312: chapter 09 zoom-line repair 310

This is an independent review of CC0's `boundary-guide-repair310.rs` against its base, with
`cc0-boundary-repair310.md` read in full. I did not write the repair.

**Not done:**
- no build, Clippy, test, render, deploy or permit;
- no Beads or TODO edits;
- no edits to any other file.

Narration is ElevenLabs Frederick Surrey and is not touched by this change. I listened to nothing.

| File | SHA-256 |
|---|---|
| base `boundary-label-repair52.rs` | `dc80c541799209f10d02701d18c7fab5a14327000719929e5e2fad0c6fd62f9f` |
| candidate `boundary-guide-repair310.rs` | `bda7d931498b23bd3203cd0c6106de69dc0a8c5e160e738d698376322ff39e53` |
| `cc0-boundary-repair310.md` | `c9e7f56518f050888a909f5b6a86e4c4372905e37ad64684c1188c8b2a3327e1` |
| movie used for stills: `preview274-review305/preview/09_boundary.mp4` (the build268 base) | `dba964d2b0730b0e327eb32b9b9bb81496ddb4ec3422f043d356a390f1de3fb4` |

## Verdict

**I found no High or Medium defect.** The two hunks do what the notes say:
- the zoom lines leave in the existing 0.2 s fade with "g is undefined at a";
- they are no longer re-faded by the 0.4 s `proof_gone` fade.

I see no route by which the lines can come back. Reading the code, the timing, narration and maths are unchanged. Three Low items and one editorial note follow; none blocks root's integration.

## What I checked, and how

1. **The base is build268's file.** Root's terminal copy `review-build268-terminal272/source-copy.json` (`81d28272…`, the hash 303 and 309 pin, and also re-hashed on csd during 309) lists `chapters/boundary.rs` at `dc80c541…`. That equals the base.
2. **Only two hunks.** I ran `diff -u` myself. With the link and `named` lines filtered out, the files are byte-identical except for the one replaced `fade_out_all(stage, &[hole_name], 0.2)` call. So these are unchanged:
   - every `wait_for`, `wait_sentence`, `until`, `pause`, `play` and run time;
   - all TeX and text constants, numbers, colours and positions;
   - the tests (they start at line 1434, and none mentions the lines, the column or the hole name).
3. **`Mob` is `Copy` at the pin.** At fmn `a399d626`, `crates/fmn-mobject/src/stage.rs:89–91` reads `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct Mob`.
   - `kit.text` returns `Mob` (the base already passes `hole_name` in a `[Mob]` array).
   - So `vec![hole_name]` is a `Vec<Mob>`, and `named.extend(&links)` uses `Extend<&Mob> for Vec<Mob>`, which needs `Copy`.
   - `&named` coerces to `&[Mob]`.
   - `links` is still pushed to, so its `mut` stays used.
   - This is by reading only; nothing was compiled.
4. **Pinned fade semantics.** I read the source at `a399d626`.
   - `fade_out` (`fmn-anim/src/fading.rs:80–97`) is a remover with `final_alpha_value = 0`. `finish_animations` (`frame.rs:298–308`) finishes, then removes, with no frame capture in between. This is the same path every fade-out in the film takes. In the base movie the hole name's own fade-out (about frames 2087–2099) leaves nothing behind in frames 2097–2107.
   - `fade_out_all` (draw264 `:315–328`) prepares one fade per mob at the same `rt` and plays them together. Two more mobs therefore keep the play at 0.2 s (13 frames). `proof_gone` keeps 0.4 s (25 frames).
   - The lines are vector polylines, so no `plate_fade` record is added.
   - **The second hunk is required, not cosmetic.** `begin_animations` (`frame.rs:192–209`) roots any animated mob that is not on the scene. Leaving `proof_gone.extend(links)` in place would re-root the removed lines at "The second piece contributes nothing" and fade them out again, which is a visible reappearance.
5. **Reappearance.**
   - After 310, `links` is used only at `:552` (fade in) and `:560` (fade out).
   - Nothing in the file iterates scene members: there is no `top_level`, `bring_to_front` or "fade everything".
   - The lines are top-level, not attached to any group.
   - From about 34.8 s on, the lines are off the scene. In the base they were off from about 133 s on, so every later code path already ran with them off. The only code that touched them in between was `proof_gone`, which 310 removes.
6. **Geometry.**
   - The picture plane maps 5 units to 5.6. The circle points are (−2.615, 0.377) and (−2.615, −0.777); the close-up corners are (2.4, 1.05) and (2.4, −2.95).
   - **Upper line.** At x = 1.15 it is at y = 0.882, through `|g| ≤ M` at `GAP_TOP` 0.75.
   - **Lower line.** Across x = 0.3–0.6 it runs from y = −2.04 to −2.17, which is the left ends of the `GAP_LOW` rows.
   - **Where else they could cross.** The arc-and-chord card, the bar and "inside the dashes" lie between the two lines and are not crossed.
   - **The remaining lifetime.** Between their fade-in and the new exit, the column holds nothing. Above them is the split formula at y = 3.2, and the hole name is at (4.3, 1.5), right of the corners.
7. **Stills from the base movie** (540p, viewed as images; extraction only, no render):
   - **27–40.5 s at 2 fps.**
     - The lines fade in at about 28.5 s.
     - From 29 to 34.5 s the column holds only the lines.
     - "g is undefined at a" fades out at about frames 2087–2099 (34.8–35.0 s, ±2 frames by eye). That is where 310's exit lands.
     - `|g| ≤ M` arrives at about 39 s, and the upper line already crosses it then.
   - **40–136 s every 4 s.** The upper line crosses `|g| ≤ M` in every sample from 40 to 132 s. By 136 s the proof has cleared, so the base crossing lasts about 93 s.
   - **Full-resolution crops at frames 4180 (69.67 s), 5417 (90.28 s) and 6240 (104 s).**
     - These confirm root's two crossings.
     - At 104 s they also show a third: the lower line runs into the start of "L: arc + chord lengths".
8. **Format.** `rustfmt --edition 2024 --config-path rustfmt.toml --check` (repo root: max_width 120) exits 0 on both files. That is a parse and format check only.

## Findings

**L1. Integration: the existing preview and final helpers cannot take this file.** This is not a defect in 310.
- **The preview helper.** CC3's `cc3-preview-fade274.py` pins `boundary` at `dc80c541…` (`:54`) and site E2 at line 1398. In 310, `painter.leave(stage, 0.5)` is at line 1402 (E1 is still at 474). That helper would stop at its own source check, so a repaired-09 preview needs a new preview helper version, or a site table re-read for 310.
- **The final helpers.** CC0 notes that 303 pins binary 268, build268, preview274 and scene-file equality. 309 is the same, so 09 needs new final helper versions on both hosts.

**L2. The layout lint missed the original defect.**
- **What the lint reported.** The base preview's `09_boundary.layout.tsv` has only a 09_3 "behind" line and a 09_7 "tight" line. It did not report either zoom-line crossing.
- **So** a clean lint on a rebuilt 09 is not evidence that the fix works.
- **What the repaired preview must show instead:**
  - frames around 34.8–35.0 s, where the lines should fade with the hole name and nothing should remain;
  - the arrival of `|g| ≤ M` at about 39 s;
  - 69.67, 90.28 and 104 s;
  - a coarse sweep of 39–133 s.

**L3. Root's record is incomplete.** `boundary-proof-gap305.json` lists only 69.67 s and 90.28 s. The base also crosses:
- `|g| ≤ M` continuously, from about 39 to 132 s;
- "L: arc + chord lengths" at 104 s.

310 removes all of these, but they belong on the repaired preview's check list (L2).

**N1. Editorial (root's call).**
- **Before:** the lines were visible for about 104 s.
- **After:** about 6.3 s, from 28.5 to 34.8 s, during "It is undefined at a itself, so it has a hole in the middle…".
- **Why that is acceptable:**
  - the narration never refers to the lines;
  - the picture's circles and the close-up both stay on screen;
  - they leave in a fade that already exists.
- **Rerouting instead.** I agree it is not narrower. The upper corner sits at y = 1.05, just above `|g| ≤ M`. Any straight route into that corner clears the formula by a few hundredths of a unit at best.

## Not established

- **No compile.** It has not been compiled or type-checked by a compiler, run through Clippy or the crate tests, rendered or linted. The `Copy`/`Extend` reasoning is by reading the pinned source.
- **Stills are of the base only.** Every still is from the build268 base movie. The repaired behaviour, with the lines gone from about 34.8 s and no crossing at 39–133 s, is predicted from source, not observed.
- **Timing is by reading.** Unchanged tables are predicted from unchanged plays and run times. Only a timeline comparison of a rebuilt 09 against timeline269 confirms it.
- **The exit window is by eye.** Frames 2087–2099 were read from a 540p crop (±2 frames). The exact frames come from the phrase mark for "But as z approaches a", which I did not read.
- **Nothing heard.** No listening, fine sync, motion, 4K or final AV judgement. Narration (ElevenLabs Frederick Surrey) is untouched by the change and was not heard.
