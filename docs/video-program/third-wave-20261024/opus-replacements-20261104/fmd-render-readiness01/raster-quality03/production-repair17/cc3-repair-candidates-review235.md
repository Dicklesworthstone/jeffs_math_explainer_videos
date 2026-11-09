# Review 235: repair candidates for chapter 06 (review224 F1/F2) and chapter 08 (review219 M1/L3)

cc3, 2026-11-08, about 05:10Z. This is an independent source review. I wrote only this file. I ran
no build, test, Clippy, render, frame extraction or synthesis, and made no canonical, TODO, Beads
or memory edit. Root reports that build238 and rebind242 passed and are stable. I did not read
their logs, and no finding below rests on them.

## Verdict

- **Original compile defect (231/232):** real. The four `.rt(…)` calls had no `Timed` import. The
  238 copies fix it and change nothing else.
- **Remaining findings on the 238 copies:** no blocker and no Medium. Each claim below was checked
  in the source:
  - The native fade, removal and return semantics work as both handoff notes describe.
  - No time is added: list memberships change, and one fade becomes a same-length hand-over.
  - The two planes now share one scale exactly, so the drawn stretch equals |f′(z₀)|.
  - The qualifiers are mathematically accurate.
  - Each new note fades in once and out once.
  - The removed 08 labels return through an existing fade.
  - `mod-review-repair236.rs` supplies the registry changes that are needed.
- **Left for root's render and picture check:** four Lows (L1–L4) and one cosmetic item (C1).

## 1. Files reviewed

All paths are relative to `production-repair17/` (Q). Every hash was re-taken at 04:55Z and
matches the hashes taken when this review began.

| Role | File | sha256 |
|---|---|---|
| 06 base (production19; rendered `final06-terminal169`, 10,030 frames) | `production-source-preserved214/workspace/explainer_corpus/complex_analysis_visual/src/chapters/one_derivative.rs` | `4f0154ec82bb787cdbee4d11d308eea25ca6cc1164f7f4bdb1d908f8d54df29c` |
| registry base (production19) | `…/preserved214/…/src/chapters/mod.rs` | `4f4b17c35bf01aaa587d8d3c97b6e79059baef6925f68c1748a121776d841ce6` |
| 08 base (build164; rendered `final08-terminal218`, 14,959 frames) | `loop-sums-label-repair164.rs` | `1cf259683b29606e51f0fd25ffb35e26247f7adff1f5d5aee8334b57aa6dedec` |
| 06 candidate, original | `one-derivative-review-repair231.rs` | `fee77db8ab0c1d76300dbc49138816f94a43f26a590cf1e0bd0f541710470a27` |
| 08 candidate, original | `loop-sums-review-repair232.rs` | `a2831a48f56d14a91ca4df26aec6596414d2413a60e5597d9dd39fc6ec563b88` |
| **06 candidate, corrected (authoritative)** | `one-derivative-review-repair238.rs` | `7a4cc01acb497ad881c0d8478c696ab74db025494ab9381d0b414355fe3fd30d` |
| **08 candidate, corrected (authoritative)** | `loop-sums-review-repair238.rs` | `8a13e8a190cd502e7b58efbcb7c40872cdb0e6e702feb69e1b789e451a7a42b5` |
| **registry (authoritative)** | `mod-review-repair236.rs` | `b5817c501f31d7e6b35672804f14a7f61901fdb81d4669c4ef48324df3b30bc6` |

The 08 comparison base is repair164, not preserved214's `loop_sums.rs`. Preserved214's file is
production19's, sha `69042b14…`. Final08 was built from repair164, and candidate 232 is a two-hunk
edit of it. Compared with preserved214's file, the 238 copy also carries the earlier 164 label
repair, which this review does not cover.

**Limitation on note provenance.** cc0's note `one-derivative-review-repair231.md` (sha
`d9656f580af0dc60c317a38c9f317d245ee996ecf8ba82752f79710443ed7b5b`) is present and unchanged.

For cc1's 08 note, my earlier record has `loop-sums-review-repair232-notes232.md` at sha
`42ec5b11c8be5136f19c6c0237c019151249d96f5a1f4d624d9e7aeae1507523`. That path no longer exists.
The file now present is `notes232.md`, sha
`1674f5ebd377fd3566dbd49007b843e08d5f53fe62f12f91e62caa6c7436644b`. It names the same candidate
hash (`a2831a48…`) and shows the same two-hunk diff. I could not establish whether the two texts
are the same. Both notes are context only: every finding here comes from the source files above
and the pinned engine.

**Engine and kit:** fmn pin `a399d626`; explainer_kit `589ab3df`.

## 2. Original compile defect (231/232, fixed in 238)

- **Cause:**
  - `.rt(run_time)` is not an fmn method. It comes from the kit trait `explainer_kit::Timed`
    (`crates/explainer_kit/src/play.rs:7`), whose blanket implementation for `A: Animation` calls
    `update_rate_info(Some(run_time), None, None)`.
  - Neither base calls `.rt(`; there are zero occurrences in both files. Neither base imports
    `Timed`.
  - The candidates add four calls without the import:
    - 231 lines 201 and 205, inside `hand_over`;
    - 232 lines 647 and 649, the two label fade-outs.
  - That gives the four E0599 errors build236 reported.
- **How it slipped:**
  - cc1's cited precedent, `cauchy.rs:56–60`, does import `Timed`, at `cauchy.rs:4`
    (`use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};`). The call pattern
    was copied but the import was not.
  - cc0's cited pattern is `draw.rs`, which also imports `Timed`.
  - Neither note claimed a compile.
- **The fix:** 238 against 231/232 differs in exactly one line per file:
  - one_derivative 238:12 `use explainer_kit::{Kit, Palette, Timed, v};`
  - loop_sums 238:17 `use explainer_kit::{Kit, Palette, Timed, tween, v};`

  rustfmt moved nothing else, and every other byte is equal. The line numbers below are the
  238 copies' and equal 231/232's.

## 3. Remaining findings on the 238 copies

### 3.1 Native API semantics (checked at `a399d626`)

- **`hand_over` (one_derivative 238:196):**
  - For each leaving mob, it calls `clear_updaters`, then prepares `fade_out(…, ORIGIN, 1.0)`
    with `.rt(rt)`.
  - For each coming mob, it prepares `fade_in(…).rt(rt)`.
  - It ends with one `play_prepared`.
  - These are the bodies of `draw::fade_out_all` and `draw::fade_in_all`, merged into one play.
    A play lasts as long as its longest animation (`fmn-anim/src/frame.rs:343`, `plan_play`), so
    this one lasts 0.3 s.
- **08 walk (loop_sums 238:645–650):**
  - `prepare(tween_linear(progress, 4.0, 2.0)?)` is valid because `AnimBuilder: IntoAnimation`
    (`fmn-anim/src/animation.rs:1004`). It is played with two 0.3 s fade-outs, and the play lasts
    max(2.0, 0.3, 0.3) = 2.0 s.
  - Alpha is `sample.time / run_time`, unclamped (`frame.rs:559`). `sub_alpha` clips it to [0, 1]
    (`animation.rs:311`). So both labels reach opacity 0 at 0.3 s and stay there for the remaining
    1.7 s.
- **End of play:** `finish` and `clean_up_from_scene` run back to back, after the last captured
  frame (`frame.rs:298`, `:679`).
  - `fade_out` is a remover with `final_alpha_value = 0.0` (`fmn-anim/src/fading.rs:94–95`). It
    restores the mob's original paint and then removes it (`animation.rs:912`).
  - Removal is "a draw-list edit, never a deletion" (`fmn-mobject/src/stage.rs:868–870`).
  - So there is no end-of-play flash, and the handles stay valid.
- **Return (loop_sums 238:654):**
  - `fade_in` (`fading.rs:52`) builds an opacity-0 start copy from the mob's current state.
  - `begin_animations` (`frame.rs:195`) roots any mob that is not in the scene.
  - `add_to_scene` (`stage.rs:811`) removes the family, appends it, then stable-sorts by z-index.
  - So c and D_n come back at full opacity, drawn above the distance segment and the walker.
    This is harmless, because after the walk the segment is back at c → `enlarged[0]` and crosses
    neither label.
- **No early appearance:** `Kit::text` and `Kit::text_left` return detached text (kit `kit.rs:403`,
  `:449`, "Detached text (not yet in the scene)"). No 06 note can show before its fade. This
  includes `matrix_unless_note`, which is created before the 06_7 symbols' 0.5 s fade.
- **Updaters:** the c and D_n labels have none; the anchor updater moves only the walker and the
  distance line. The inline fade-outs in loop_sums therefore need no `clear_updaters`.
  `hand_over` clears updaters on `scale_note`, which has none either, so the call does nothing.

### 3.2 Duration preservation

I listed every time-bearing call in each file's scene code with its timing arguments, using a
read-only Python comparison. The calls counted were fades, shifts, plays, tweens, sentence and
phrase waits, `until`, holds, `predict`, `wait_until` and `finish`.

- **06** (`construct` body):
  - 103 calls in both base and candidate, in the same order with the same arguments, once the
    single `hand_over(…, 0.3)` is mapped to the base's `fade_in_all(stage, &zoomed, 0.3)`.
  - The 52 fade calls have equal run times.
  - cc0's count of 95 leaves out the tween arguments and `finish`; it agrees with mine.
- **08:** 168 calls in both. The only difference is that the walk's
  `stage.play(tween_linear(progress, 4.0, 2.0)?)` becomes `prepare` plus
  `play_prepared(vec![walk, c_aside, d_aside])`, still 2.0 s. No `wait_for` phrase changed.
- **Frame counts:** with the narration binding unchanged, the expected counts are 10,030 (06)
  and 14,959 (08). `review-repair-timeline239.py` encodes the same two numbers. Root reports
  rebind242 passed; I have not reviewed rebind239, rebind242 or timeline239.

### 3.3 Mathematical scale (review224 F1)

- `Plane::new(center, width, x, y)` draws `width / (x range)` scene units per unit.
  - Input `(−0.6, 2.6)²`: 4.6 / 3.2 = 1.4375. The base's `(−0.2, 2.2)²` gave 4.6 / 2.4 = 1.9167.
  - Output `(−1.6, 1.6) × (0.4, 3.6)`, unchanged: 4.6 / 3.2 = 1.4375.
  - The drawn stretch is now exactly |f′(z₀)| = |2(1 + i)| = 2√2 = 2.828. The base drew
    2√2 × 1.4375 / 1.9167 = 2.121; review224 measured 2.11.
- The window centres are unchanged:
  - the input centre (1, 1) = z₀ stays at scene (−3.6, −0.6);
  - the output centre (0, 2) = z₀² = 2i stays at (2.4, −0.6).

  Both panels are 4.6 × 4.6, with bottoms at −2.9 and tops at 1.7. The input's right edge (−1.3)
  is still left of the output's left edge (0.1). Content around z₀ shrinks to 0.75 of its old
  size.
- **Fan arrows (06_3)** use the native `Arrow` with `thickness(4.0)` and `buff(0.0)` (238:453).
  Its key dimensions are in `fmn-library/src/line.rs:581–595`:
  - Shaft width is min(4 × 0.015, 0.1 × length). The 0.1 × length cap binds both at the old
    input length (0.575) and at the new one (0.431).
  - So the head width (5 × the shaft) and head length (0.187 at the new size) scale with the
    length. The head is 43% of the arrow both before and after.
  - The input fan is therefore a uniform 0.75× copy of the old one, and its heads do not crowd
    more in proportion. At 4K (270 px per scene unit), each arrow is about 116 px and its shaft
    about 12 px.
  - cc0's optional `FAN_LEN` 0.3 → 0.4 is not needed on proportion grounds. Whether 116 px reads
    well is a render call.
- **Tests:**
  - New: `both_pictures_share_one_scale` (238:1087) and `the_drawn_stretch_is_the_derivatives_size`
    (238:1096). The fields they use are public: `Plane.frame` and `Frame2 { center, width, height,
    x, y }` (kit `plot.rs:81`).
  - Existing tests still hold by arithmetic. In `the_windows_hold_the_pictures`, the side-0.5
    square around z₀ spans 0.75–1.25. In `the_fan_ends_as_the_derivative_times_each_step`,
    z₀ + 0.3e^{iθ} lies inside (−0.6, 2.6)².

### 3.4 Qualifier accuracy (review224 F2)

- **"unless the derivative is 0"** (238:64), used at 06_3 and 06_7.
  - 06_3's voice says "multiplying by a complex number is a stretch and a turn. So a tiny square
    around z lands as a tiny square, bigger and turned, but still square."
  - Multiplying by c ≠ 0 is a stretch by |c| and a turn by arg c. Multiplying by 0 collapses
    everything to a point.
  - 06_7 says "the local map must be a stretch and a turn, with no squash and no reflection". The
    holomorphic Jacobian is [[a, −b], [b, a]]. It is a stretch and a turn exactly when
    (a, b) ≠ (0, 0), and is the zero matrix otherwise.
  - The note states the exact exception in both places. It agrees with the later script: 07_1
    and 07_2 treat critical points, and 15_2 says "away from critical points".
- **"bigger here: stretch 2.83"** (238:67, 512) is `BIGGER_HERE` followed by `fact("deriv.scale")`.
  That fact is `fixed(|d|, 2)` (`facts.rs:195`, pinned to "2.83" at `facts.rs:458`), the same one
  06_4's readout prints. "Bigger" holds because 2.83 > 1, and "here" ties it to this example.
  Accurate, and it adds no new number.
- **"both pictures drawn to the same scale"** (238:62) is true from 06_1 until the zoom, where
  `hand_over` takes it away as the existing zoom note arrives. One wording nuance is listed as L3.

### 3.5 Fades, removal and reappearance

| Mob | Created | Fades in | Fades out | Faded out twice? |
|---|---|---|---|---|
| `scale_note` | 238:272 (06_1) | 279, with both frames, 0.4 s | 543, `hand_over` at the 06_4 zoom, 0.3 s | no; not in 06_5's `gone` |
| `unless_note` | 476 (06_3, sentence 3) | 477, with `named`, 0.3 s | 06_5 `gone` (580), 0.4 s | no |
| `bigger_note` | 512 (06_3, sentence 4) | 519, with `sq`, `img`, `img_name`, 0.5 s | 06_5 `gone` (581), 0.4 s | no |
| `matrix_unless_note` | 656 (06_7) | 664, with `kind`, 0.4 s | 06_8 `gone` (686), 0.4 s | no |
| 08 `c`, `D_n` | existing | existing; again at 654 with P_n, 0.3 s | 646–650 during the walk; then the existing final fade at "The same shrinking argument works for a triangle" | no; no other fade lists them between 654 and that fade |

- **06 test:** `the_review224_notes_ride_existing_fades` (238:1122) pins the 06 rows from the
  source text: three mentions per note, one `hand_over`, no bare zoom fade, and `scale_note` absent
  from 06_5's list. Its slicing anchors (`say(&kit, stage, "06_x")` and `fade_out_all(stage,
  &gone, 0.4)?;`) match the source as written.
- **08 visibility times** come from cc1's constant-rate fit to existing stills. I checked the
  arithmetic, not the frames.
  - The labels fade from about 170.283 s, are hidden from about 170.583 s to 172.283 s, and are
    back by about 172.583 s.
  - This covers root's observed crossings: D_n at about 171.33 s and c at about 171.72 s
    (`final08-crossing-viewed230.json`).
  - The margins are 0.75 s before the first crossing and 0.56 s after the last.
  - The centre dot, diagonal, inset, walker and caption are untouched.
- **08 caption (review219 M1):** the caption changes from "Selected square Q_n, magnified" to
  "Selected square, magnified" (loop_sums 238:610). This removes the literal `Q_n` shown from
  164 s to 217 s, and no other code reads the string.

### 3.6 Registry needs (`mod-review-repair236.rs`)

- **Changes against production19's `mod.rs`:**
  - lines 90–92 add `one_derivative::SAME_SCALE`, `one_derivative::UNLESS_ZERO` and
    `one_derivative::BIGGER_HERE` to `TEXT`;
  - line 100 replaces the old caption with "Selected square, magnified".

  These are the only text changes the candidates need:
  - `matrix_unless_note` reuses `UNLESS_ZERO`.
  - The runtime "2.83" uses digits and a full stop that the existing supplements already cover,
    for example 06_4's "stretch 2.83, turn 45°".
  - No other module refers to `one_derivative::` items.
- **Tests:**
  - The new strings are non-empty and distinct, so `no_supplement_is_empty_or_duplicated`
    (`mod.rs:740`) holds.
  - The notes add no hold, so no `READING_WINDOWS` entry is needed.
  - The 08_5 reading-window test slices from `say(&kit, stage, "08_5")` onward. The loop_sums
    edits sit earlier in the file (607–655), so the test is unaffected.
- **Docs:** the on-screen strings are not listed in the preserved workspace's docs.
  `docs/storyboard.md` and the direction maps contain neither the old strings ("Selected square
  Q_n, magnified", "exact stretch and turn") nor the new ones; a grep found them only in `.rs`
  files. No docs change is needed for consistency.
- **C1 (cosmetic):** mod236 puts the three 06 entries at the top of `TEXT`, before the "Native
  Goursat proof" comment. cc0 suggested placing them beside the existing `one_derivative` entries.
  There is no functional effect.

### 3.7 Lows for root's render and picture check

All four rest on scene-unit arithmetic and CMR10 width estimates, not on pixels.

- **L1, 06_2: "z₀ = 1 + i" label near the "from above" approach.**
  - At the new scale, the approach path's tip ends near scene (−3.6, 0.05). The label's right end
    is now near x ≈ −3.62, with its bottom at about 0.11–0.16. That leaves a gap of about 0.06–0.11
    scene units, roughly 16–30 px at 4K.
  - In the old scale, the label ended near x ≈ −3.88.
  - This is not an overlap by estimate, but it is the closest approach the rescale creates. cc0's
    "no new contact" is right but leaves this narrow margin unmentioned. Check the 06_2 approach
    frames.
- **L2, 06_3: notes pair with legends across the axis.**
  - In sentence 3, `unless_note` (x 0.25–2.20, y 1.45) and `named` ("a stretch and a turn", about
    x 2.48–4.42, y 1.35) sit on nearly one row, either side of the axis at x = 2.4. Read left to
    right, the qualifier comes before the claim it qualifies.
  - From sentence 4 the rows pair "unless the derivative is 0" with "the image", and "bigger here:
    stretch 2.83" with "exact stretch and turn".
  - The dim 18 pt style sets the notes apart. Confirm on pixels that the second row does not read
    as one phrase.
- **L3, 06_2 to 06_4: "both pictures" while a third picture is on screen.** The ratio inset
  (238:338, `Plane::new(v(5.85, 2.75), 1.9, …)`, at 1.1875 scene units per unit) is visible during
  this span, at a different scale. "Both" most naturally means the two large planes. The ambiguity
  is mild, and "both planes drawn to the same scale" would remove it, but I do not require it.
- **L4, 08_5: hidden labels and the voice.** Removing c and D_n during the walk reads cleanly
  only if no word naming c or the diagonal falls inside the hidden span, about 170.58–172.28 s.
  cc1 places the labels' fade-in at "the distance to c is at most the square's diagonal", before
  the walk, so the walk likely runs under "and the steps add up to the perimeter". This needs
  word timing (fine sync), which I have not checked.

## 4. Not established by this review

- I compiled, tested and linted nothing. The build238 and rebind242 results are root's report; I
  did not read their logs.
- I rendered, decoded and viewed nothing. The layout figures are arithmetic, not measurements.
- **No listening, fine-sync, final-AV, pronunciation or picture acceptance.** Narration remains
  ElevenLabs Frederick Surrey. These candidates change no script line, cue, take or timing. The
  new 06 and 08 renders still need an independent picture review, and the existing listening and
  fine-sync gaps remain open.
- I did not review `review-repair-rebind239.py`, rebind242 or `review-repair-timeline239.py`,
  beyond noting that timeline239 expects 10,030 and 14,959 frames.
- The provenance of cc1's earlier note path is unrecovered (§1).
