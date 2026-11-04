# complex13-radius-repair01: the radius and pole-name sweep in 13_3

BronzeJaguar, for BlueSalmon, answering Mail 2624. Bead 87do.39. Narration is ElevenLabs Oliver.

**State:** a narrow source repair is staged and frozen for root. **Nothing here has been compiled, tested by Cargo, rendered or heard.** I wrote this chapter, so this is not independent. No canonical or shared file, Bead or commit was touched. Cue words are unchanged (`script.tsv` `2968b0fd7654…`), and there are no new strings or numbers.

## What frame 93 shows

Confirmed folder: `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/13_real_integral/`. Sheet 023 holds frames 92–95. Frame 93 is local 39.000 s, walker at 98.75° (preview02, the scratch voice, canonical source).

- **The spoke near "i": near, not crossing.** It runs between the letter and the ring, 0.093 plane units (8.7 px at 720p) from the letter and 0.035 (3.3 px) from the ring.
- **"R" on the axis: yes.** R's centre is 0.028 off the axis, so the axis runs through the letter.
- **Not persistent: agreed.** Frames 92, 94 and 95 are clear. R is on the axis for about 0.12 s.

**The frame understates the sweep.** The sampling missed the worse moments. These come from the source geometry, whose R position matches frames 90–95 to 0.7 px; **no picture shows the four rows marked "unsampled"**:

| Contact in canonical | Walker angle | Time at scratch rate | What is drawn | On frames02 |
|---|---|---|---|---|
| Spoke over the pole ring | 83.3–96.7° | 0.24 s | **White line through the ring's centre** | Unsampled |
| R across the gold segment | 5.6–14.1° | 0.15 s | R on top of the gold line | Unsampled |
| Spoke over the segment's arrow head | 0.1–14.7° | 0.26 s | Spoke over the head | Unsampled (frame 90 is at 15.15°, just past it) |
| R on the imaginary axis | 96.4–103.3° | 0.12 s | Axis through the letter | **Frame 93** |
| Spoke over the letter i | 103.9–107.3° | 0.06 s | Spoke through the letter | Unsampled |

The repair is warranted for the ring above all: it is the loop's one pole, and the spoke passes through it on every take. Each contact is brief, and I would not hold a film for any of them alone.

**The pole names as R grows need no change.** Over R = 2 to 10 the smallest gap is 0.055 plane units (5 px at 720p), ring to lit segment. "−i" stays 0.105 left of the arrow head. Frame 130 agrees by eye.

## The repair

Only `real_integral.rs`. Pole, pole name, spoke, R and all narration stay.

| Change | Effect | Where (staged) |
|---|---|---|
| Spoke rooted first, then the segment's arrow, i's backing, the ring and "i" re-added over it | The spoke passes behind them | `:409` |
| Dark backing under "i" for the walk (the pole mark's own device) | Spoke passes behind the letter, never through it | `:395`, radius `:238` |
| Dark backing under R, moving with it | Axis passes behind R | `:394`, `:428`, radius `:237` |
| R and its backing come in over 0.28–0.42 rad (16–24°), tied to the walker's angle | R never lies across the gold segment. Until then the fixed "R" names the spoke lying along the segment | `:225`, `:228`, `:430` |
| Both backings fade with the spoke at sentence 2 | Nothing dark is left behind, and the poles close in as before | `:454` |
| `r_name_at` split into `r_name_z` + `r_name_at`; `POLE_NAME_X`, `MINUS_POLE_NAME_X` replace literals | Lets the tests read the real numbers. Same values | `:213`, `:233` |

No hook changed, and nothing is timed to the audio. R's arrival follows the walk, which ends on "so the bump" as before.

The kit's layout lint already treats this as the right device ("the disc under a node's numeral", `crates/explainer_kit/src/layout.rs:21`). The backings are painted before their letters, so `covered` should not fire; I have not run the lint.

**The cost:** the backings hide more of the spoke than the letter did. At height 1 the spoke is hidden over 82.9–97.1° (0.26 s) and 100.5–110.6° (0.18 s), against the letter's 0.06 s. Around 101° the "i" backing takes a small bite out of the spoke's side. In the synthetic sheet it reads as passing behind. **Anchor A9 asks you to judge this on the native picture.** If it looks like a break, drop `i_back` and `I_BACK`, with the test lines that name them; "i" still stays on top of the spoke.

**Seen, not changed:** the lit segment later runs over the arrow head (frame 130), and the arc walker passes over the top arrow head. Both are outside this brief.

## Apply

```
git apply docs/video-program/third-wave-20261024/opus-replacements-20261104/complex13-radius-repair01/complex13-radius-repair01.patch
```

- Patch sha256 `39eae1d3b7dcd1cf6802f1392691c7a09a5784da728d05d895df21eae829a0cf`, +184 −10.
- Base `real_integral.rs` is `906d4bd803b1…`, byte-equal in canonical and in `combined-picture-stage01/candidate` when I checked. Staged result is `a5ef573dfa68…`.
- It uses the existing public `dot` and `ground` from `draw.rs`, unchanged. It does not depend on Lavender's work or on complex02-05-repair03, which touches other files.

## Bindings

**Hold rows:** `bindings/visual-holds-delta.json`, against `docs/narration-direction-map-oliver10.json` (`341356cda34a…`). Replace only the listed fields.

| Cue | Kind | Old line | New line | Marker |
|---|---|---|---|---|
| 13_5 | prediction | 593 | 631 | same |
| 13_11 | reading | 775 | 813 | same |

The candidate's copy of this map (`bcb6a31bf026…`) differs from canonical only in rows 10_4 and 10_5, as I reported in repair03, so these rows fit both. Sections other than `visual_holds` do not change.

**Phrase hooks:** `bindings/phrase-hooks.tsv` lists 32 kept, none added or removed, so no new phrase marks are needed.

**Binder:** the frozen `build()` still refuses on canonical with the same 8 existing problems (added 0, gone 0). The delta comes from calling `build()` read-only on an overlay, as in repair03.

## Tests

Five new Rust tests, all unrun, in `real_integral.rs`:

| Test | Guards |
|---|---|
| `the_walks_r_is_never_shown_across_the_segment` (`:1007`) | At 0.2° steps, R is at zero wherever it would touch the gold line, and its backing never notches it |
| `the_walks_r_and_its_backing_keep_off_everything_else` (`:1032`) | Backing covers the letter; clear of spoke, ring, "i", half circle and arrows at every angle |
| `the_poles_name_keeps_its_backing_off_the_axis_and_the_ring` (`:1057`) | Backing covers "i", leaves the axis and ring alone, and the spoke really does reach both |
| `the_poles_names_stay_clear_as_they_close_in` (`:1070`) | i, −i and the ring stay clear of the lit segment and arrow head for all R from 2 to 10 |
| `the_spoke_passes_behind_the_pole_and_both_stay_on_screen` (`:1089`) | Source order of the re-add, fades and hooks; poles and names outlast the radius |

The half sizes in them are measured on frames 88, 100 and 130, not estimated. They are 720p measures, so each is good to about one pixel (0.011 plane units).

## Native review anchors

`native-review-anchors.tsv` lists 13 poses, with what each must show. Please **extract at these exact times, not on the 0.5 s grid**: on preview02 that grid landed in only one of the five contact bands.

For the walk, on any voice:

```
T0 = (13_3 start in the render's cues.tsv) + 1.1 s          fades of 0.3 + 0.4 + 0.4 before the walk
W  = max(time of "so the bump" − T0, 1.5 s)                 until(stage, "so the bump", 1.5)
t(deg) = T0 + W × deg / 180                                 tween_linear, a steady rate
```

On preview02 this gives T0 = 37.2 s (13_3 at 36.100). The frames measure 37.23 s, and the walker reaches 180° at 40.46 s, so the formula holds on the scratch voice. Those times are in the table only to compare with the old frames, and **they are not Oliver's**. "So the bump" has an existing hook, so it is on the render's own clock.

## Checks run

All in `checks/`, outputs in `checks/outputs/`.

| Check | Result | What it cannot show |
|---|---|---|
| `rustfmt --check`, staged file and `main.rs` of canonical + repair (parses every module) | exit 0 | Types, borrows, names |
| Patch round trip on scratch copies | apply equals staged, `-R` equals canonical | Later merges |
| `sweep_check.py`, reading numbers and z-order from both source files | Canonical: all 5 contacts faulted. Repair: no faults. **All 16 planted faults caught** | Pixels. It models z-order from source text, and backings as ideal discs |
| `frame_calibration.py` on frames02 | R within 0.008 plane units (0.7 px) of the formula on frames 90–95 | **Old source only**: nothing about the repair |
| `synthetic_sweep.py`, `outputs/synthetic-sweep-NOT-A-RENDER.png` | Drawn before and after, for judging the look | **Not a render.** Line weights are the script's |
| Desk checks, canonical against repair | arity 0 problems in 4603 calls; shape, unused and desk nothing new. Scope has one new row, `Item` in `impl Iterator<Item = f64>` at `:1002`, which I read as a false alarm: it is an associated-type name | Anything needing the compiler |
| `binder_compare` (in `run_checks.py`) | added 0, gone 0, two hold rows derived | Any audio time |

**Pictures read:** sheets 023, 025, 027, 028 and 031; crops of frames 86–97; zooms of frames 93 and 130. The growth was not read frame by frame; its clearances are from geometry.

## Not run, and the limits

- No `cargo` of any kind, so **all 5 new tests are unrun**. No render, ASR, TTS or listening. No native picture of this repair exists. Sampled frames are not continuous AV, and the synthetic sheet and geometry are not native proof.
- All times use scratch-voice rates. Oliver's walk can differ, down to the 1.5 s floor.
- **Compile risks, first on the list:** `a.set_opacity` inside an updater appears in no chapter yet. I read `impl VStyle for Stage` in `fmn-library/src/style.rs` at the pin and expect it to resolve, but only rustc can say. Pre-rooting relies on the same reading of `begin_animations` as repair03's 05-pole, also uncompiled.
- **Behaviour to check on the first render:** R's ramp (A3, A4), then that `fade_out_all` takes R and both backings cleanly (A11). I expect both to work, since `fade_out` restores begin-time state and the backings have zero stroke width, but neither is seen.

## Regenerate

```
python3 -B checks/run_checks.py REPO FMN_CRATES FRESH_SCRATCH_DIR
```

It writes under this folder and the scratch directory only, stops if canonical has moved, and needs a new scratch directory each run. PIL needs site-packages, so the driver and the two picture checks run without `-I`. It does not rewrite this file, `native-review-anchors.tsv` or `SHA256SUMS`.
