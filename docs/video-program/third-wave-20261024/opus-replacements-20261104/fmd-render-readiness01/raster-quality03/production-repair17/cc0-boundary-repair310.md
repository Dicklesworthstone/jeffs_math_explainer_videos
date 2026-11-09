# cc0 boundary repair 310: chapter 09 zoom lines leave before the column fills

cc0, 2026-11-08. This is a narrow source repair, prepared only.

**Not done:**
- no canonical edit;
- no build, Clippy, test or render;
- no deploy;
- no permit;
- no Beads or TODO edits.

**Untouched:** worker and launcher 303, and the old source.

## Files

| File (in `production-repair17/`) | sha256 | Role |
|---|---|---|
| `boundary-label-repair52.rs` | `dc80c541799209f10d02701d18c7fab5a14327000719929e5e2fad0c6fd62f9f` | Base. Preserved, not edited. |
| `boundary-guide-repair310.rs` | `bda7d931498b23bd3203cd0c6106de69dc0a8c5e160e738d698376322ff39e53` | New candidate. |

**Why this is the right base.** Build268's own `source-copy.json` (`81d28272…`) is the record that worker 287/303 pins and that csd's build268 job holds. It lists `explainer_corpus/complex_analysis_visual/src/chapters/boundary.rs` at `dc80c541…`. Among the local copies, only `boundary-label-repair52.rs` has that hash. `production-repair17/boundary.rs` and the production214 workspace copy are `c9e6b5ba…`, which is not build268's file.

**Other build268 files I read** (by the same source-copy hashes):
- `draw.rs` = `cc2-plate-fade-candidate264-draw.rs` (`9bb07b3f…`);
- `mod.rs` = `mod-review-repair256.rs` (`e1f4d1b8…`).

## The defect, as recorded and as I saw it

**Root's record.** `boundary-proof-gap305.json` lists two crossings in the local preview274 movie `preview274-review305/preview/09_boundary.mp4` (`dba964d2…`):
- the upper gray zoom line crosses `|g| ≤ M` at 69.67 s and 90.28 s;
- the lower one crosses "inside edges cancel: walked both ways" at 69.67 s.

**My frames.** I pulled the same three instants from that movie (540p), plus 2 fps strips from 22 to 46 s, and looked at them as images. They show the same two crossings.

**The source.**
- **What the lines are.** These are the two `zoom_links` (source :271–279). They run from the "a little beyond" circle at angles ±0.7 to the close-up's left corners, at (2.4, 1.05) and (2.4, −2.95).
- **Their lifetime.** They are drawn at "It is undefined at a itself" (:542–552) and leave only at "The second piece contributes nothing" (old :814–816). So they stay across the column between the picture and the close-up (`GAP_X` = 1.15) for the whole 09_1 proof.
- **What else uses that column:**
  - `|g| ≤ M` at `GAP_TOP` 0.75, with "inside the dashes" below it;
  - the cancellation lines, then "both walked counterclockwise", the wobble lines and "for every small ρ", all at `GAP_LOW` −2.0;
  - the arc-and-chord card and the 2πρ·M bar around `GAP_MID`.
- **Where they cross.** The upper line passes x = 1.15 at about y = 0.88, through the top of `|g| ≤ M`. The lower line crosses the start of "walked both ways".

**Preview timings, read from the 2 fps strips (±0.5 s):**

| Time | Event |
|---|---|
| about 28.2 s | the lines fade in |
| about 34.8 s | "But as z approaches a": `g is undefined at a` fades out |
| about 39 s | `|g| ≤ M` fades in |

## The change

There are two hunks, and nothing else differs.

```diff
@@ -553,7 +553,12 @@
         fade_in_all(stage, &[outer, hole, ca, hole_name], 0.3)?;
         line.wait_for(stage, "But as z approaches a")?;
         let limit = proof_tex(&kit, stage, LIMIT_TEX, 26.0, output(), ROW_B)?;
-        fade_out_all(stage, &[hole_name], 0.2)?;
+        // The zoom lines have tied the close-up to the picture by now. Held longer they would
+        // run through the column's |g| <= M and cancellation lines, so they go with the hole's
+        // name, in the same 0.2 s fade: no time is added.
+        let mut named = vec![hole_name];
+        named.extend(&links);
+        fade_out_all(stage, &named, 0.2)?;
@@ -811,7 +816,6 @@
-        proof_gone.extend(links);
         proof_gone.extend(circle_arrows);
```

**Effect.**
- **Before the change:** the lines show for about 6.5 s, while the close-up, the loop and the hole appear and the hole is named. That is the zoom context.
- **When they go:** they leave with "g is undefined at a", in the fade that already exists.
- **From `|g| ≤ M` on (about 39 s):** nothing crosses the column.
- **Unchanged:** the 33.48 s sample still shows them, because the change comes later.
- **Expected:** neither line appears at 69.67 s or 90.28 s. This is predicted from the source and has not been rendered.

**Timing is unchanged, by reading the code.**
- **The fade itself.** `fade_out_all` (draw264:315–328) prepares one fade per mob, each with the same `rt`, and plays them together. Adding the two lines to the 0.2 s call keeps it at 0.2 s. Removing them from the 0.4 s `proof_gone` call keeps that at 0.4 s, which still holds its 12 named mobs and the circle arrows.
- **Nothing else moved.** No `wait_for`, `wait`, `pause`, `play`, text, number, colour or position changed. The narration cue marks and the wording are untouched.
- **Tables.** 09's timeline, cues and cuelog tables are therefore expected to stay byte-equal to timeline269's. That is still to be confirmed by root's next timeline comparison.

**Why the second hunk is required, not cosmetic.**
- At the pinned engine (`a399d626`, `fmn-anim/src/fading.rs:73–97`), `fade_out` is a remover that puts the mob back in its begin-time state when done.
- Once the lines have left the scene, fading them out again in `proof_gone` would at best be an empty animation. At worst it would root them again for 0.4 s at "The second piece contributes nothing".
- `links` has no other use in the file (grep), and no test refers to `zoom_links` or `links`.

**Why not reroute.**
- **Every straight route crosses the column.** Any straight line from the picture's circle to the close-up crosses the column. That column carries text at all three heights during the proof.
- **What a reroute would need.** It would have to bend around changing text, or re-time with it.
- **What was asked.** The lines are decorative. Letting them go once the zoom is established is the narrowest fix that "simply avoids persistent crossing without adding animation time".

## Checks run

- **Diff:** `diff -u` against `boundary-label-repair52.rs` shows only the two hunks above.
- **Format:** `rustfmt --edition 2024 --config-path rustfmt.toml --check` passes on the base and on the candidate (max width 120). It parses the file, but it does not compile it.
- **Types:**
  - `Mob` is `Copy` at the pin (`fmn-mobject/src/stage.rs:89–91`), so `named.extend(&links)` type-checks by reading.
  - `links` stays `Vec<Mob>`.
  - `named` is a `Vec<Mob>` passed as `&[Mob]`.

## Not established, and for root

- **Not run at all.** It is not compiled, and has not been through Clippy, the crate tests, a render, the layout lint or a draft.
- **No acceptance.** There is no picture or motion check of the change, no audio heard or checked, and no fine-sync, 4K or AV acceptance.
- **Narration.** It stays the current ElevenLabs Frederick Surrey set. Nothing in narration, units or script changes.
- **Limited look.** I looked for other crossings only in the three sampled frames and the 22–46 s strips, and saw none.
- **Integration (root's call):**
  - put this file in place as `chapters/boundary.rs` in a new build workspace;
  - build, test and Clippy;
  - render a 540p preview of 09;
  - review 28–40 s (the lines' new exit), and the 69.67 s and 90.28 s instants;
  - compare the tables with timeline269.
- **A new final version is needed.**
  - A new build means a new binary.
  - 303 pins binary 268, build268 and preview274, and requires the scene file to be equal across the preview's and final's builds. So 303 cannot render the repaired 09.
  - I left 303 untouched.
- **Coordination.** For CC3's independent 308 audit, root can pass on the exact paths: the base `production-repair17/boundary-label-repair52.rs` (`dc80c541…`, build268's bound `boundary.rs`) and this candidate `production-repair17/boundary-guide-repair310.rs` (`bda7d931…`).
- **Not independent.** I wrote this repair.
