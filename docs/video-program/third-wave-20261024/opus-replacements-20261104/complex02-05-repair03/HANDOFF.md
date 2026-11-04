# complex02-05-repair03: picture repairs for chapters 02 to 05

BronzeJaguar, for BlueSalmon, answering Mail 2524. Bead 87do.39. Narration is ElevenLabs Oliver.

**State:** source repairs, Rust tests, patch and binding deltas are done and frozen for root. **Nothing here has been compiled, tested by Cargo, rendered or heard.** I wrote these scenes and the review they answer, so this is not independent. No canonical or shared file, Bead or commit was touched.

Cue words are unchanged: `script.tsv` stays `2968b0fd7654…`, 105 cues. The new on-screen labels are symbols (`|z|`, `|z|^2`, `\arg z`, `2 \arg z`), so no new printed number needs `facts.rs` or the oracle.

## Apply

```
git apply docs/video-program/third-wave-20261024/opus-replacements-20261104/complex02-05-repair03/complex02-05-repair03.patch
```

- Patch sha256 `49205a2a577c03b273935609701ddf04e602278245c6fd438fb08331f98b96d2`, +1041 −85, against the hashes in `checks/baseline.sha256`.
- It changes four files: `turning.rs`, `two_planes.rs`, `reading_color.rs`, `gallery.rs`. Full copies are under `explainer_corpus/complex_analysis_visual/src/chapters/`.
- The four files, `draw.rs`, `mod.rs`, `script.tsv` and `docs/storyboard.md` were byte-equal in canonical and in `combined-picture-stage01/candidate` when I wrote this, so the patch fits both. If Lavender's or anyone's edit has since landed in these four files, use `git merge-file` and regenerate the hold delta.
- `draw.rs` is not touched. `gallery.rs` uses the existing public `Painter.group` and `Painter.anchor`, so Lavender's repair must keep those two fields.

## Findings fixed

Lines are in the staged files. Every new beat waits on a phrase or sentence of the voice. Fixed durations are animation lengths and `until` floors; no final audio time is written anywhere.

| Review ID | What the picture now does | Where | New test |
|---|---|---|---|
| 04-still | Dot walks the loop and the needle turns on the words that name the cycle, with a live count. 04_3 walks at sentence 2 (twice around). 04_4 re-laps until "Once for a simple zero", then the order card comes on "twice for a zero of order two". 04_5 walks 1/z backwards after its card | `reading_color.rs:125` `LoopWalk`, `:309`, `:328`, `:366` | `the_needle_stands_on_the_colour_under_the_dot`, `the_walk_counts_what_the_cards_state`, `the_cycle_is_walked_on_the_words_that_name_it` |
| 03-empty | One point, length 1.4 at 30°, is drawn in both planes. Arrows and `|z|`, `|z|^2` come on "squares each length"; arcs and `\arg z`, `2 \arg z` on "doubles each angle"; all fade at sentence 2, before the ring | `two_planes.rs:145`, `:438` | `the_rule_is_shown_on_one_point_that_squaring_really_sends_there`, `the_planes_are_not_empty_under_the_rule` |
| 05-pole | Gold box is rooted first and the pole marker re-added over it, so the edge no longer crosses the marker from the box's first frame | `gallery.rs:815` | `the_strip_arrives_on_a_clear_column_and_under_the_pole` |
| 05-tile | The other panels clear together in `TILE_BEATS` (1.3 s), and the outline waits for "nearly square" | `gallery.rs:151`, `:930`, `:955` | `the_tile_is_ready_for_the_words_that_name_it` |
| 05-crowd | `converges`, `pole_says` and `zedge` fade at 05_7 sentence 5. `squashed` fades before the strip comes in | `gallery.rs:782`, `:861` | `the_strip_arrives_on_a_clear_column_and_under_the_pole` |
| 04-a | Card now reads "near a zero z = a of order m, …": 115 chars, so 7.667 s against 7.7 s clean. See the margin note below | `reading_color.rs:27` | `the_exact_card_names_its_point_and_fits_its_window` |
| 02-freeze | Angle arc sweeps and the row pulses on "an eighth of a turn" | `turning.rs:121`, `:137`, `:433` | `the_recalled_turn_moves_and_does_not_answer_the_question` |
| 02-G | 02_6 clears the old question, pulses the rows on "and angles add", and fades the grid on "Every complex derivative" | `turning.rs:477`, `:482` | `the_last_beat_reads_the_rule_off_the_rows_and_ends_on_the_arrows` |
| 03-flip | A ring and a straight chord per tick on "each point flips", cleared in 03_6 | `two_planes.rs:725` | `each_flip_goes_straight_across_the_horizontal_axis` |

`every_hook_is_said_and_they_run_in_the_voices_order` (in `turning.rs`) reads all four files and guards every literal hook.

## Low findings

**Fixed**

| Cue | Edit | Where | New or changed test |
|---|---|---|---|
| 03_2 | Second lap drawn in ink at the walk's halfway point, so the picture also says "twice" | `two_planes.rs:162`, `:555` | `each_half_of_the_walk_is_one_lap_of_the_image` |
| 03_2 | Radius labels 20 → 24 pt | `two_planes.rs:157` | `the_walk_radii_stand_off_their_circles_and_the_axes` (changed) |
| 03_3 | Predict ring moved to `ASK_AT` (0.55, −0.65), off both axes and off every scale mark. The card still lies over a stretch of the imaginary axis | `two_planes.rs:121` | `the_question_and_its_ring_stand_clear_of_the_axes` |
| 03_4 | Foot labels now say "horizontal axis", "vertical axis", as the voice does | `two_planes.rs:37` | `the_labels_say_what_the_voice_says` |
| 03_4 | Fold marker is a dot, not a vertical stub | `two_planes.rs:627` | none new |
| 03_6 | The unspoken 18 pt "new windows" note fades at sentence 2. **Partial:** it is still 18 pt | `two_planes.rs:806` | `the_labels_say_what_the_voice_says` |
| 04_3, 04_5, 04_6 | Each title arrives after its crossfade | `reading_color.rs:293`, `:348`, `:383` | `the_cycle_is_walked_on_the_words_that_name_it` |
| 04_2 | "dark lines" note is 22 pt ink | `reading_color.rs:282` | none new |
| 04_5 | "simple pole" is ink text with a pole-coloured edge | `reading_color.rs:351` | none new |
| 04_7 | Gold is pre-rooted, the marks re-added over it, and only then the dim circle fades: no strike, no gap | `reading_color.rs:421` | `the_gold_never_crosses_the_marks_and_never_leaves_a_gap` |
| 05_2 | Zero at sentence 2. Pole, card and output box on "and at minus one" | `gallery.rs:452` | `the_zero_and_the_pole_come_on_their_own_words` |
| 05_3 | One exp tile outlined on "exact squares", bracket on "repeating every two pi" | `gallery.rs:114`, `:509` | `the_exponential_tile_is_an_exact_square_on_the_band_lines` |
| 05_4 | The arrow stops at 2.8, short of the dot | `gallery.rs:133`, `:574` | `the_climb_stops_short_of_the_dot` |
| 05_8 | RH card on "is the Riemann hypothesis" | `gallery.rs:856` | `the_strip_arrives_on_a_clear_column_and_under_the_pole` |
| 05_8 | "this strip shows four zeros…", still 55 chars | `gallery.rs:64` | `the_tile_is_ready_for_the_words_that_name_it` |

**Not fixed, and why**

| Item | Why |
|---|---|
| 04-A, 04-A2 (stripes) | Lavender owns `draw.rs` |
| 05_5 flat highlight | Depends on 04-A, so wait for it |
| 05_1 lap tones, 05_2 "+0.0", wheel "negative"/"positive" | Live in `draw.rs` or the kit, outside my four files |
| 05_7 term underlines | Needs TeX layout measures that only a build gives; a guessed underline would be worse |
| Upright maths, Unicode `z³`, heavy title minus, the upright quiz cards | Glyph lane, and already in `complex-glyphs02-review01` |
| 02 and 03 short stills, 02 grid caption hold, 03 lone arrow | Judge on Oliver. They may be fine at his pace |
| 04 band-ratio still and the "once" row | Would add motion beyond the review's Medium items. The card now lands on "twice", which resolves the contradiction |
| 02-D half, 03 r = 1 gap, `e^{2πi}` card corners, mosaic names, 05-L columns, "a single pole" pink, dim unit circle | Layout changes that should be judged on pixels. I left them rather than guess blind |
| 02 "1+i" a sentence early | Deliberate. Left |
| Fade residue, gold-loop specks | Unresolved at CRF 26, recheck on the 4K. Not source edits |

## Bindings

**Hold rows:** `bindings/visual-holds-delta.json`, against `docs/narration-direction-map-oliver10.json` (`341356cda34a…`). Replace only the listed fields.

| Cue | Old line | New line | Marker |
|---|---|---|---|
| 02_5 | 384 | 435 | same |
| 03_3 | 494 | 607 | **changes** to `let asked = predict(&kit, stage, &line, &ASK, out.p(ASK_AT))?;` |
| 04_4 | 239 | 341 | same |
| 04_6 | 279 | 389 | same |
| 05_8 | 827 | 866 | same |

The candidate's copy of this map (`bcb6a31bf026…`) differs from canonical only in the `sha256` of rows 10_4 and 10_5. All five `old` sides match there too, so the delta fits both maps. The rows are valid only for these exact file hashes: any later edit to the four files moves them again.

**Phrase hooks:** `bindings/phrase-hooks.tsv` lists kept 71, added 14, removed 3.

The 14 added are 12 phrase hooks and 2 sentence hooks (04_3 sentence 2, 04_4 sentence 2). The 12 phrases need new phrase marks, or they run on sentence share:

| Cue | Phrase | Byte in cue |
|---|---|---|
| 02_5 | an eighth of a turn | 37 |
| 02_6 | and angles add | 139 |
| 02_6 | Every complex derivative | 172 |
| 03_2 | squares each length | 66 |
| 03_2 | doubles each angle | 90 |
| 03_5 | each point flips | 140 |
| 04_4 | Once for a simple zero (`until`) | 81 |
| 04_4 | twice for a zero of order two | 105 |
| 05_3 | exact squares | 151 |
| 05_3 | repeating every two pi | 166 |
| 05_8 | is the Riemann hypothesis | 71 |
| 05_9 | nearly square | 134 |

Removed: 04_4 sentence 1, 05_3 sentence 2, 05_8 sentence 1. Variable hooks (`NAMED`, the phrase in `walk_and_count`) are not in the table and did not change.

I found no `phrase-timings*` in `combined-picture-stage01` or `narration-request-preparation01` (depth ≤ 6), so I could not check any mark on Oliver.

**Binder:** the frozen `bind_oliver_plan.py` `build()` refuses on canonical with 8 problems that this repair did not make, and the same 8 with it: added 0, gone 0. They are `closing.rs:278`, `closing.rs:319`, `counting.rs:215`, `counting.rs:277`, `gallery.rs:345`, `reading_color.rs:170`, and two stale `DYNAMIC` entries for `gallery.rs`. **Root must fix or re-classify these before the binder can write a map.** The delta above comes from calling `build()` read-only on an overlay, not from a written map.

## For root: files I may not edit

- **Storyboard picture cells drift** for the rows at lines 114, 115, 122–125, 135–139, 146, 147, 151–153. Lines 136 and 152 quote the old `NEAR_ZERO` and `FOUR` text. The `main.rs` tests check hold cells only, so nothing will flag this.
- **`mod.rs` `TEX`:** its doc says it lists every formula. Add `two_planes::RULE_LENGTH_TEX[0]`, `[1]`, `RULE_ANGLE_TEX[0]`, `[1]`, near line 520. The probe captures the actual strings anyway, so this is for the list's truth, not coverage. The comment near line 63 ("how the plot was made") is stale wording.
- **04_4 window margin is 0.033 s.** Optional: raise it to 8.0 in `mod.rs` and the storyboard.
- **03_3 test copies `predict`'s ring geometry** (radius 0.26, drop 0.45). If `predict` changes, that test must change with it.

## Correction to my review

`REVIEW.md` proposed `stage.bring_to_front(&pole1)?`. That method is on `Scene` (`fmn-scene/src/runtime.rs:966`), not on this `Stage`, so the edit would not have compiled. The repair uses this crate's idiom: `add_to_scene` re-appends a mob, so a re-add promotes it. `begin_animations` roots only mobs not yet in the family, and `fade_in` starts at opacity zero with no frame captured in `pre_play`, so pre-rooting and re-adding hold from the first frame.

## Checks run

All from `checks/`, outputs in `checks/outputs/`. Paths there are written as `<repo>`, `<fmn>`, `<scratch>`.

| Check | Result | What it cannot show |
|---|---|---|
| `rustfmt --check`, four files, plus `main.rs` of canonical + repair (parses every module) | exit 0 | Types, borrows, names |
| Patch round trip on scratch copies | apply equals staged, `-R` equals canonical | That it merges with later edits |
| `mirror_tests.py`: Python mirrors of the 19 new tests, the 1 changed test and the repeated `mod.rs` reading windows | 21 pass on staged. All 44 planted defects are caught. On canonical 19 fail; the 2 that pass are the general hook-order guard and the `mod.rs` windows | **A mirror is not the Rust test.** It does not check Rust types or the tests' own helpers. One real helper bug got past it this session and I found it by reading |
| Frozen desk checks, canonical against repair | arity 0 problems in 4717 calls; scope, shape, desk none new; unused only the existing `cauchy.rs:21` | Anything needing the compiler |
| `binder_compare.py` | added 0, gone 0, hold rows derived | Any audio time |
| `timing_budget.py`: scratch cue lengths × 0.85, 1, 1.15, char share | Every new hook 0.00 s late. 02_6 sentence 1 is 0.40 s. Over 0.5 s: 04_3 sentence 1 (1.56) and 05_9 sentence 1 (1.52), both existing fixed runs this patch did not lengthen | **Scratch voice, not Oliver.** An estimate, not a measure |

Canonical equalled `checks/baseline.sha256` (9 files) at sealing.

## Not run, and the limits

- No `cargo` of any kind: no compile, Clippy or test. **All 19 new tests and the 1 changed test are unrun.**
- No render, ASR, TTS or listening. No picture of this repair exists, so every visual claim above is what the source should do. Nothing was sampled, and no continuous AV exists.
- Label half sizes in the new tests are estimates from the font-width rule, not measured glyphs.

**Please look at these first on the next render:**

1. 04_4 and 04_6: a stopped live count fades as a container. `fade_out` on a family container is what `Painter::leave` does, but a faded live value is new to this crate.
2. 04_4: the count visibly restarts from +2.0 to +0.0 on the re-lap. Does it read as a restart or as a glitch?
3. 03_2: is the second lap visible in ink over the first?
4. 05_3: the tile outline at exp's small scale is 0.279 scene units wide. Is it legible at 4K?
5. 03_5: are the chords legible among the ticks?
6. 04_3 sentence 1 and 05_9 sentence 1 on Oliver's real lengths.

## Regenerate

```
python3 -I -B checks/run_checks.py REPO FMN_CRATES FRESH_SCRATCH_DIR
```

It writes under this folder and the scratch directory only, stops if canonical has moved, and needs a new scratch directory each run. The desk tools and binder are read from the frozen `../complex` copies. It does not rewrite this file or `SHA256SUMS`.
