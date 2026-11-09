# Review 467: the five apportionment layout rows in chapters 08 and 11 (CC3)

CC3, 2026-11-09 UTC. This review reads source and the timeline444 lint, and nothing else. No frame of any movie was looked at: preview458 and score445 do not exist yet, and timeline444 was still rendering chapter 12 at 04:25Z. Nothing in this review accepts a picture or claims clearance. The rows below say what each flag is, when it is on screen, and which preview458 frames a person has to look at to decide it.

## Inputs

All of these were read in place. Every source file named below equals its pin in build209's inputs (`build209-inputs.json`, `0f8565c5…`), which is the source of binary209 (`c7730356…`), the binary timeline444 ran.

| File | sha256 |
|---|---|
| timeline444 `08_signposts.layout.tsv` / `.timeline.tsv` / `.cues.tsv` | `692e0002…` / `329f1c49…` / `72a3b521…` |
| timeline444 `11_webster_quota.layout.tsv` / `.timeline.tsv` / `.cues.tsv` | `46cf8f06…` / `dc44dfc2…` / `f74aaec2…` |
| narration437 `phrase-timings.json` (the phrase marks the narrator loads) | `fc3c86fa…` |
| `explainer_corpus/apportionment/src/chapters/signposts.rs` | `16f6ff62…` |
| `explainer_corpus/apportionment/src/chapters/webster_quota.rs` | `14dab3d4…` |
| `explainer_corpus/apportionment/src/chapters/draw.rs` | `dd385ed5…` |
| `crates/explainer_kit/src/layout.rs`, `kit.rs`, `narration.rs` | `b1ada215…`, `ef035faf…`, `8072720d…` |
| CC0 `cc0-apportionment-preview-review462.py` (its `plan --dry-run-timeline444 --show`, read-only, exit 0) | `23575514…` |

## How a lint row maps to frames

The rules below come from `layout.rs` and `kit.rs`:
- **The stamp.** `kit.say` calls `begin_line` before the narrator waits for the cue, so a row's `+x s` counts from the `say` call, and `@y s` is chapter time (frame / 60).
- **Look timing.** Sampled looks fall every 12 frames. A problem a sampled look sees is reported only once it has stood for 1.0 s, but the row keeps the first look's stamp.
- **"Later looks"** counts every look after the report. That means sampled looks plus the checkpoint looks at each `say` and `hold`. It also counts any other problem that has the same key.
- **The key.** The key for `overlap` and `crossed` is the text, not the mobject, so the two tied "2,000" values in 08 share one key.

I replayed both chapters' construct order on the 60 fps grid. The replay used the timeline cue frames, the narrator's phrase marks and `BREATH` (0.6 s). It used one more rule: fmn rounds an animation up to whole frames on the exact f64, so 0.2 s costs 13 frames and 0.8 s costs 49. With that rule the replay reproduces every frame the timeline records:
- the predict marks at 3198 (08) and 399 (11);
- 08_4 starting at 3464 and 11_2 at 665, both exactly where the "?" ring finishes fading;
- 11_3 at 2758;
- 08's last frame at 6447.

It also reproduces all five stamps (`+0.8s`, `+10.3s`, `+10.3s`, `+2.3s`). Four of the five later-look counts come out exactly; 08-L2 is explained below. The replay is a scratch script, not saved in Q. Frames below are this replay's. Where a fade starts on a phrase, its first visible frame can differ by one.

Scale at preview458's 960×540: 1 scene unit = 67.5 px. A difference of 0.02 units, such as 0.23 against 0.25, is about 1.4 px there and about 5 px at 4K.

## The five rows

### 08-L1 `edge` @0.8 s: the chapter header

- **Object:** the header `head` (`signposts.rs:71`), "signposts: people per seat when a seat opens". It is 20 pt in `Palette::dim()`, centred at (`LIST_X` 4.45, 3.78). The lint box is x 2.49..6.41, y 3.69..3.87.
- **Position:** its top stands 0.13 below the frame top. That is 0.02 inside the 0.15 safe-area margin. A shape inside the margin is reported as `edge`, which is why there is no separate `tight` row.
- **Its only neighbour:** the top of card 1, 0.145 below it (card 1 top = 3.545).
- **At 540p:** the header is about 12 px tall and its top about 9 px from the frame edge.
- **On screen:** it fades in with the three town names at the start of 08_1, frames 36–72. It never moves, flashes or fades. It stays to frame 6447.
- **Count:** the first look is frame 48, and the row is reported at 108. The 535 later looks are exactly 528 sampled looks (frames 120..6444) plus 7 checkpoints: before 08_2 to 08_6, the hold in `think`, and the hold in `finish`. So it stands through the whole chapter.

### 08-L2 `crossed` @53.4 s: the think ring over rank 18's "2,000"

- **Stroke:** the think ring from `draw.rs` `think()`, at `THINK_AT` (6.1, −2.9) with `THINK_R` 0.55 and a 4.0-wide hero stroke. The lint's box (x 5.53..6.67, y −3.47..−2.33) is exactly that ring plus half its stroke.
- **Text:** the crossing point (6.10, −2.35) is the ring's top. It falls on the value of rank 18, Ash seat 2, "2,000", which is right-aligned at x 6.43 with centre y −2.346.
- **Why the crossing lasts:** the drain shortens the arc from its far end, and the top is the arc's fixed start. So the crossing lasts until the ring fades. With the arc full, the ring passes through the value on both sides of the top.
- **On screen:**
  - mark `predict` at 3198;
  - the ring, the "?" and "pause for more time" fade in over 3198–3211;
  - the ring drains over 3211–3451 (`THINK_S` 4.0 s, linear);
  - the ring and the note fade out over 3451–3464;
  - 08_4 starts at 3464.
- **Count:** 18 later looks. Only 16 of them are ring looks (frames 3276..3456). The other two, about, come from the cutoff line. That line slides through the tied values twice: in 08_5 down to rank 19 (frames 4877–4974), and in 08_6 up to rank 18 (frames 5662–5711). About two sampled looks catch it inside a "2,000": 4944 and 5688 with fmn's smooth easing, or 4932 and 5688 with smoothstep. Both tied values centre to the place the key uses (`crossed|"2,000"|6.0,-2.5`), so these looks count on this row.
- **Consequence:** the row mixes a held picture (the ring, about 4.4 s) with a moving line passing through, which is how a cutoff moves. For this row, the "consecutive span end" (58.0 s) is a coincidence.

### 08-L3 `overlap` @53.4 s: the "?" over rank 19's "2,000"

- **Objects:** the think "?" (70 pt, hero, centred at (6.1, −2.9)) and, by geometry, the value of rank 19, Cedar seat 13, "2,000" at y −2.684.
- **The geometry:** I estimated the glyph boxes from CMR10 metrics × size × 0.01. Rank 19's value box is about y −2.76..−2.61 and the "?" ink about −3.15..−2.65, so they overlap. Rank 18's value ends about 0.2 above the "?".
- **What the row can't say:** it cannot say which "2,000" it means, because the overlap key is the pair of texts.
- **On screen:** the "?" fades in over 3198–3211 and stays after the ring is gone. It fades out with the blank rank-16 outline over 3464–3482, the start of 08_4.
- **Count:** 19 = 18 sampled looks (3276..3480) plus the checkpoint before 08_4 at 3464, exactly.
- **Source context:**
  - `THINK_AT` is the default place every chapter uses ("unless a chapter needs it elsewhere").
  - In 08 the list reaches into it: the values of ranks 18 and 19 (x up to 6.43, card bottom −2.83) lie inside the ring's box.
  - Those two cards are the exact 2,000 tie that 08_6 highlights 40 s later. The ring and the "?" sit over the very values the tie is about.

### 08-L4 `tight` @93.2 s: the red "="

- **Object:** the red "=" (26 pt). It sits at (`LIST_X + CARD_W/2 + 0.3` = 6.85, −2.515), between the red glows on ranks 18 and 19. Its ink stops 0.17 from the right edge, so its right edge is about 6.94.
- **On screen:**
  - it fades in with the two glows at "exactly tied" (phrase mark at 93.087 s), frames 5586–5616;
  - the cutoff moves under rank 18 over 5662–5711;
  - the "=" flashes (`indicate`, scale 1.25) over 5711–5766, with its peak near 5738;
  - a 0.6 s pause runs to 5802;
  - it stays to 6447.
- **The flash peak:** at 1.25× the right edge comes to about 0.15. That is still counted as `tight` (0.15 to 0.25), because the 67 later looks are every look.
- **Count:** 67 = 66 sampled looks (5664..6444) plus the `finish` hold, exactly.
- **Not a lint finding, but worth looking at:** at rest under rank 18, the cutoff line (x 2.20..6.70, y −2.515) ends about 0.06 left of the "=", at its height.

### 11-L1 `tight` @13.4 s: the three-town citation

- **Object:** `BY_VIA_GPP` (`webster_quota.rs:50`), "Balinski & Young 1982, via Gölz, Peters & Procaccia 2024". The row quotes its first 32 characters; the line ends with "2024". It is 20 pt in `Palette::dim()`, centred at (`GAP_X` 4.3, 2.1), under "3 towns: Webster keeps the quota promise".
- **Position:** it stands 0.23 from the right edge, so by symmetry it spans about x 1.72..6.88. The source comment calls it "the three-town result's short source; the description cites it fully".
- **On screen:**
  - at "Balinski and Young proved it" (phrase mark 13.023 s), `fade_in_lagged([proved, cited], 0.8, 0.3)` starts the heading at 782 and the citation at about 793;
  - both are fully in by 831;
  - there is no move or flash;
  - it stays through 11_2, until the wipe: the hold ends at 2722 and `fade_all` runs 2722–2758;
  - 11_3 starts at 2758.
- **Count:** 155 = 154 sampled looks (876..2712) plus the hold checkpoint at 2577, exactly. The wipe clears the lint probe's updater at 2722, so no look sees the fade.

## Preview458 frames: what 462 plans and what to add

CC0's 462 plan for 08 (115 samples) and 11 (one anchor, 93 samples in all) already covers every row's first look, +0.2 s, +1.0 s, +2.0 s, the 2 s grid, the predict hold, and the cue starts. Controller 468 (`6b08e2c0…`) pins 462 unchanged. Adding the frames below therefore needs a new helper version or a small extraction afterwards. That is CC0's call.

| Row | Already in 462 | Add (frame, what it shows) |
|---|---|---|
| 08-L1 | 48, 60, 108, 168, the 2 s grid, 6447 | none needed; any frame after 72 shows the header at full opacity |
| 08-L2 | 3198, 3204, 3216, 3264, 3318, 3324, 3360, 3463, 3464, 3480 | **3211**: full ring at full opacity, crossing the value on both sides of the top. **3450**: the shortest arc, the last drain frame. **3457**: the ring half faded. **4932, 4944, 5688**: the cutoff passing through a tied value (the extra looks). **4974**: the cutoff at rest under rank 19 |
| 08-L3 | 3204–3480 as above, 3492, 3494 | **3482**: the first frame after the "?" fade, the overlap gone |
| 08-L4 | 5592, 5604, 5652, 5712, 5760, the grid, 6447 | **5616**: "=" fully in. **5711**: the cutoff at rest beside the "=". **5738**: the flash peak, nearest the edge. **5766**: the flash over |
| 11-L1 | 804, 816, 864, 924, the 2 s grid 960..2640, 2722, 2724, 2757, 2760 | **793**: the citation's first visible frame. **831**: both lines fully in |

Three label notes for CC0. None changes what 462 extracts:
- **"+2.0 s (PREDICT_HOLD_S)":** in these films `think()` drains over `THINK_S` 4.0 s (`draw.rs:1106`), so the +2.0 s frame shows a half ring. The drain ends at mark + 13 + 240 frames.
- **"+0.3 s ('?' faded in)":** the fade takes 13 frames, so +0.3 s is 5 frames into the drain.
- **"End if the later looks were consecutive":** later looks include checkpoints and same-key looks. The real last frames are 3463 for the 08-L2 ring, 3481 for the 08-L3 "?", 6447 for 08-L1 and 08-L4, and 2757 for 11-L1.

## What only the preview can decide

1. **08-L1:** does the header read clear of the frame's top edge at 0.13, or cramped against it?
2. **08-L2 / 08-L3:** with the ring and "?" over the two tied values (3211–3481), are the values unreadable during the think hold? Is that acceptable for a hold where the viewer is meant to think about rank 16, not ranks 18 and 19?
3. **08-L4:** does the "=" read as cramped at the right edge, at rest (0.17) and at the flash peak (about 0.15)? Does the cutoff line ending just short of it read as part of the sign?
4. **11-L1:** does the citation's "2024" read clear of the right edge at 0.23?

The 0.02-unit questions (08-L1, 11-L1) are about 1.4 px at 540p. The preview can show whether a line looks cramped, but it cannot measure 0.23 against 0.25. That margin is the lint's rule, and whether it applies is root's decision.

## Not done

- No movie frame was looked at, and nothing was extracted or launched.
- No source, helper, TODO or Beads file was edited.
- Glyph boxes not printed by the lint (the "?", the two "2,000" values, the "=" ink) are estimates from glyph metrics.
- Nothing here is listening, sync, motion or AV evidence.
