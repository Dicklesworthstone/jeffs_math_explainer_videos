# Candidate 232: loop_sums caption and sweep labels

Candidate only, for root to inspect and finalize. Nothing canonical was edited. Nothing was
compiled, built, rendered or synthesized. Narration stays ElevenLabs Frederick Surrey and no
narration timing changes. Listening, fine sync, motion review and independent final AV
acceptance remain open.

| File | sha256 | bytes |
|---|---|---|
| base `loop-sums-label-repair164.rs` (the source final08 was built from) | `1cf259683b29606e51f0fd25ffb35e26247f7adff1f5d5aee8334b57aa6dedec` | — |
| candidate `loop-sums-review-repair232.rs` | `a2831a48f56d14a91ca4df26aec6596414d2413a60e5597d9dd39fc6ec563b88` | 65,179 |

The candidate is the exact base bytes with two guarded replacements, each asserted to occur
once. Full diff:

```diff
--- loop-sums-label-repair164.rs
+++ loop-sums-review-repair232.rs
@@ -607,7 +607,7 @@
     let centre_name = kit.tex(stage, "c", 26.0, output(), &[], v(centre.re - 0.22, centre.im + 0.28))?;
     let zoom_name = kit.text(
         stage,
-        "Selected square Q_n, magnified",
+        "Selected square, magnified",
         21.0,
         Palette::dim(),
         v(SQUARE_CENTRE.0, 2.45),
@@ -638,11 +638,20 @@
         true,
     )?;
     line.wait_for(stage, "the steps add up to the perimeter")?;
-    stage.play(tween_linear(progress, 4.0, 2.0)?)?;
+    // The moving segment turns through every direction from c, so it crossed the D_n
+    // and c labels (picture review 219 L3; frames 10260-10320 viewed in 230). Both
+    // leave in the walk's first 0.3 s, inside its 2 s, and return with P_n in its
+    // 0.3 s; the dot at c and the diagonal stay. No time is added.
+    let walk = stage.prepare(tween_linear(progress, 4.0, 2.0)?)?;
+    let c_aside = fade_out(stage.arena_mut(), centre_name, ORIGIN, 1.0)?;
+    let c_aside = stage.prepare(c_aside.rt(0.3))?;
+    let d_aside = fade_out(stage.arena_mut(), diagonal_name, ORIGIN, 1.0)?;
+    let d_aside = stage.prepare(d_aside.rt(0.3))?;
+    stage.play_prepared(vec![walk, c_aside, d_aside])?;
     stage.clear_updaters(anchor, true);
     stage.remove_from_scene(anchor);
     let perimeter_name = kit.tex(stage, "P_n", 26.0, hero(), &[], v(-2.6, -2.5))?;
-    fade_in_all(stage, &[perimeter_name], 0.3)?;
+    fade_in_all(stage, &[perimeter_name, centre_name, diagonal_name], 0.3)?;
     line.wait_for(stage, "size at most epsilon, times its diagonal, times its perimeter")?;
     let bound = kit.tex(
         stage,
```

## 1. Caption (review 219 M1)

`kit.text` sets plain text, so `Q_n` is drawn as Q, an underscore and n. The caption is up
from 164.000 s to 217.000 s (frames 9840–13020). Root viewed the 164 s full frame and agrees.
The candidate keeps the same call, size, colour and centre, and only removes `Q_n`. The new
string is shorter, so the centred caption gets narrower inside its old footprint. I did not
use `kit.tex` for `Q_n`: that would change the font and the layout of the line, and this
fix is meant to be narrow.

### TEXT glyph registry update (not applied)

The glyph supplement list must follow the string. Replace one line in the `pub const TEXT`
list of whichever `chapters/mod.rs` root binds to the next build:

```diff
-    "Selected square Q_n, magnified",
+    "Selected square, magnified",
```

| mod.rs copy | sha256 | old line at |
|---|---|---|
| `production-source-preserved214/workspace/explainer_corpus/complex_analysis_visual/src/chapters/mod.rs` | `4f4b17c35bf01aaa587d8d3c97b6e79059baef6925f68c1748a121776d841ce6` | 97 |
| `Q/mod-repair18.rs` (same bytes as the snapshot) | `4f4b17c3…` | 97 |
| `Q/mod.rs` | `fedb9075d73901baefb03753d5b8c58417ec4dc458b0c1c55c612ddf501234ed` | 97 |

- The old string occurs once in each copy and the new string occurs nowhere, so
  `no_supplement_is_empty_or_duplicated` (snapshot mod.rs:737) still holds.
- The new string has no characters the old one lacked, so no new glyph is needed.
- Among the TEXT literals, this is the only one containing `_`, `^` or a backslash.
- The canonical `explainer_corpus/complex_analysis_visual/src/chapters/mod.rs` (`57c46d67…`)
  has no such entry. The canonical `loop_sums.rs` (`cf383504…`, 817 lines, last changed
  2fb1c0c7) has no zoom scene. Neither canonical file needs this line until root promotes
  the repair164 lineage.

## 2. Sweep labels (review 219 L3; root's frames 230)

### Evidence

- **Root, observed:** in `final08-crossing-viewed230.json`, root viewed all 61 frames
  10260–10320 on 8 sheets plus detail frames 10280 and 10303. The green distance line crosses
  D_n around 171.33 s and c around 171.72 s.
- **Mine, predicted from existing stills only, with no extraction:** I fitted a constant-rate
  walker to the 171 s still (frame 10260, progress 1.433) and the 172 s still (frame 10320,
  progress 3.433). That gives 1.9995 sides/s, a walk start near 170.283 s (frame ≈10217) and
  an end near 172.283 s (frame ≈10337). The label ink boxes are in 4K pixels: c from the
  168 s still, D_n from the 171 s still.

| label | ink box (x0, y0, x1, y1) | predicted covered | frames | root |
|---|---|---|---|---|
| D_n | (1423, 727, 1498, 784) | 171.310–171.402 s | 10279–10284 | ≈171.33 s; detail 10280 |
| c | (1146, 1016, 1170, 1046) | 171.633–171.824 s | 10298–10309 | ≈171.72 s; detail 10303 |

The sampled stills near the walk are frames 10200, 10260, 10320 and 10380, and none falls in
either window. That is why review 219 recorded L3 as predicted, not observed. Review 219 also
said "about 8 frames near 171.7 s" for c; this better fit gives 12 frames.

### Mechanism (read from fmn at the pinned rev `a399d626`)

- **The walk's play still lasts 2.0 s.** A play's run time is the maximum over its
  animations (`fmn-anim/src/frame.rs`, `plan_play`). The frame-order comment there calls this
  "the Reference's mechanism for unequal run_times in one play".
- **The labels stay hidden after 0.3 s.** A shorter animation's alpha runs past 1 and
  `sub_alpha` clips it to [0, 1] (`fmn-anim/src/animation.rs`). Both labels are fully
  transparent from 0.3 s into the walk until its end.
- **There is no end-of-play flash.** `fade_out` is a remover with `final_alpha_value = 0`
  (`fmn-anim/src/fading.rs`). At the end of the play, `finish` puts the entry back in its
  begin state and `clean_up_from_scene` removes it from the draw list. The two run back to
  back with no capture between them.
- **The labels can return.** `remove_from_scene` is "a draw-list edit, never a deletion",
  so the entry and its handles stay valid (`fmn-mobject/src/stage.rs`). The existing 0.3 s P_n
  `fade_in_all` brings them back at full opacity and re-roots them, which puts them above the
  distance segment and the diagonal.
- **Precedent:** a prepared `fade_out(..).rt(..)` inside `play_prepared` already appears at
  `cauchy.rs:56–60` in the preserved214 snapshot. I found no earlier use in this crate of
  unequal run times in one `play_prepared`, so this would be the first.

### Timeline

No time is added:

- The walk play is max(2.0, 0.3, 0.3) = 2.0 s, and the P_n fade stays 0.3 s.
- No `wait_for` phrase or other duration changes.
- `goursat_phrase_waits_match_the_installed_recording_text` reads only the `wait_for`
  phrases, which are unchanged.
- The 08_5 reading-window test in mod.rs searches from `say(&kit, stage, "08_5")` (base
  line 1171) onward, and both edits sit before that.

Predicted visibility, from the fit; root's frames are the authority:

| span | c and D_n |
|---|---|
| fade-ins at "the distance to c is at most the square's diagonal" → walk start ≈170.283 s (≈10217) | fully visible, as now; the static segment does not cross them |
| ≈170.283 → ≈170.583 s (≈10217–10235) | fading out, 0.73 s (≈44 frames) before the first crossing (D_n, 10279) |
| ≈170.583 → ≈172.283 s (≈10235–10337) | hidden; the last c crossing frame, 10309, is 0.46 s before the end |
| ≈172.283 → ≈172.583 s (≈10337–10355) | returning with P_n |
| afterwards | visible until the existing `fade_out_all` at "The same shrinking argument works for a triangle" (base 723–739), which lists both |

After the walk the segment stands at c → `enlarged[0]` again, because progress 4.0 maps to
vertex 0. Before the walk it held that same position without crossing either label, so the
restored labels are not crossed. `centre_dot`, the diagonal, the inset, the walker, the
distance segment and the caption are all untouched.

### Alternatives considered

- **Hide during the 0.4 s distance/walker fade-in** (root's first idea): D_n would never
  appear before the walk, and c would vanish while the voice names it. I rejected this given
  the requirement to preserve both labels before and after the walk.
- **Move the labels:** the segment turns through all directions from c, so any spot near c
  inside the square gets crossed. Spots outside the square bring back review 219 L2's
  distance problem.
- **Draw the labels above the segment:** the line would still pass through them visibly.

## What root should check on pixels

- ≈10217–10240: the labels fade out with no flash. The walker and segment start as before.
- 10279–10284 and 10298–10309: no label under the segment.
- ≈10337–10356: c and D_n return with P_n, with no flash or double draw.
- 164–217 s: the caption reads "Selected square, magnified" and is centred as before.
- The later triangle-transition fade still clears c and D_n.

## Checks and limits

- `rustfmt --check --edition 2024 --config-path rustfmt.toml` passes on scratch copies of both
  the candidate and base164. This checks parsing and formatting only.
- **Not done:** cargo check, Clippy, tests, build or render. The candidate is uncompiled.
- `fade_out`, `ORIGIN` and `stage.prepare` resolve through `fmn::prelude::*`, as in
  `draw.rs` and `cauchy.rs`. That is read from the source, not compiled.
- The clipped-alpha and remover behaviour were read from the fmn source at the pin, not
  exercised. The crossing windows above are predictions from stills; root's frames 230 are
  the observation.
- No listening, fine sync, motion review or independent final AV acceptance.
