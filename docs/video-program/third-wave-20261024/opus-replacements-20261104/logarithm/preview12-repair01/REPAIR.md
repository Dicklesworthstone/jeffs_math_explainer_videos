# Chapter 12 picture repair, preview01 (Mails 1157, 1166, 1168)

RubyRaven, bead `mx-video-program-program-87do.63`. Staging only; root owns canonical.

**Status: uncompiled, unrendered, unlistened.** Nothing here has been built or seen on screen.
The fixes are predicted from source, 18 sheets, WhiteSwan's report and a Python mirror. Root's
build, Clippy, previews, lint and frame look decide whether they worked.

**No cue changes.** `script.tsv` is untouched (`3e4e34af7090e448`), and so are `oracle.py`, facts,
`chapters/mod.rs` and the glyph list. No on-screen string changed. ElevenLabs Oliver remains the
narrator, and this repair adds no synthesis.

## What to apply

Two independent patches, for `git apply` or `patch -p1` at the repo root. They touch neither Mail
1144 candidate's files, and either can go in without the other.

| Patch | Hunks | sha256 start | Result file | Result | Canonical base |
|---|---|---|---|---|---|
| `chapter12-picture-repair01.patch` | 29 | `ca1a373df30a6d30` | `.../chapters/what_it_forgets.rs` | `97996601ae9f678d` | `8354492ba9d6a8cb` |
| `predict-think-bar-repair01.patch` | 1 | `49ccebe848f80fc0` | `.../chapters/draw.rs` | `59ebd09d2fa5c793` | `f9d2511334565991` |

`.../` is `explainer_corpus/logarithm_mysteries/src`. Both bases equal
`../snapshots/canonical-build04-preview01/`. **The second patch is the shared one:** it changes the
think bar at all 13 `predict` sites in 10 chapter files (audit below).

## The seven findings

All seven are fixed in the staged source. "Checked" says what I looked at myself. Everything else in
WhiteSwan's report I took from the report.

| # | Checked | Fix |
|---|---|---|
| F1 | Sheets 070–072; mirror | Corner and trace now show off the rod, in red; rings stay; pin replaces the unseeable path; red line reaches 3.0. **Departs from the review's recommended option; see below.** |
| F2 | Sheet 017: "2πik" at 463 px, border at 456 px (960 px cell) | `cut_word` and `rule` move to x −3.1 through `PLANE_WORDS_X`. Predicted end −0.49, border −0.35. |
| F3 | Sheet 026; `orbit_point` source | All flows start at `SHARED_START` (1.6, 0); `home` becomes a ring there. |
| F4 | Readouts on sheets 042, 064; `play.rs:49` | Chapter-local `steady` with `rate::linear` for every tracker tween in the chapter. `tween` and `M2` imports removed, since nothing else here uses them. |
| F5 | Sheets 047, 073, 074 | Run times now come from the sentence they illustrate. |
| F6 | Sheets 056–059; `narration.rs` `hold` and `window` | `kit.hold` before the bar is created; opacity 0 once under 0.01 units wide. |
| F7 | Sheets 079, 080, 084 | Line, window and dots at sentence 5; `tie` at 45%; `precisely` with `convention` at sentence 6. |

### F1: why a pin and not the x-ray

The review recommends drawing the top centre's path through the solid, dimmed. I took its other option
(a rigid point with a stem), for two reasons:

- **The x-ray undoes Mail 803 repair 3.** That repair made depth unambiguous by drawing nothing the
  solid hides. In `checks/sketch_pin_vs_xray.png` (top row pin, bottom row x-ray) the x-ray arc reads as a
  scratch painted on a face. That sheet is my own PIL sketch with approximate colours, not a render.
- **The path can never be seen honestly.** The top-face centre has radius 1, inside the inscribed ball, so
  it is inside the solid in every pose for any axis through the centre. The mirror gives drawn length 0.000
  over 200 poses. This is why the original trace never appeared, and no direction of spin changes it.

The pin is a red copy of the rod's upper stub, from the top-face centre to `PIN_TIP` (0, 2, 0), carried with
the cube. At radius 2 it is beyond the corners' √3, so it never enters the solid, and it goes through the same
depth test as everything else. At the start the gold rod lies over it, so it peels off the rod as the cube turns.

| t | Tip arc drawn | Pin stem drawn | Corner path drawn |
|---|---|---|---|
| 0.25 | 0.883 of 0.883 | 1.050 | 0.924 |
| 0.50 | 2.016 of 2.016 | 0.914 | 1.760 |
| 0.75 | 3.062 of 3.062 | 1.042 | 1.555 |
| 1.00 | 3.836 of 3.836 | 0.884 | 2.010 |

The corner column matches WhiteSwan's table. The arc starts at the rod's tip, screen (−3.00, 1.42), passes
(−4.71, 0.65) and ends at (−4.72, −0.96). The pin then stands out of the left face beside the corner's landing.

If root prefers the x-ray, it is one `add_runs` line, and the corner, red line and tests still stand.

**The top-centre dot still vanishes for part of the spin** (t 0.237–0.649), when the top face looks away.
That is correct depth. The review noted this as a blink; the pin should carry the eye through it. Reversing the
spin would keep the dot in view throughout, but it swings the arc across the front faces at screen (−3.01, −0.19),
so I left the direction alone.

### F5 and F7: timing

**WhiteSwan's preview ran on the scratch voice** (`--scratch-voice` in preview01's receipt), so its sentence times
are character shares, not Oliver. The code reads `kit.sentence_starts` at render, so it follows Oliver's real
sentences when they exist. The numbers below are scratch-voice predictions from `checks/timeline.py`. That model
agrees with the old source's beats on the sheets to within the half-second sampling.

| Beat | Sentence (scratch) | Before | After |
|---|---|---|---|
| 12_4 sentence 2 | 60.64–71.32 | Quarter lap out and back, ends about 64.0 | One steady lap, 61.04–70.72, then an unseen reset (a whole lap is the same pose) |
| 12_4 sentence 3 | 71.32–80.53 | Spin ends 74.92 | Spin ends 79.73 |
| 12_6 sentence 3 | 123.91–136.98 | Red spin ends 127.01, reset at about 128 | Red spin ends 129.79, holds to 131.75, reset, chip 134.23 |
| 12_6 sentence 5 | 142.01–154.77 | `tie` 143.31, `precisely` 143.71 | `tie` 147.75, `precisely` 154.77 |
| Think bar | Line ends 99.69 | Drains from about 97.45 to 101.45, then a stub | 100.29–104.29, fade ends 104.54 |

The shares in 12_6 sentence 3 are `AXIS_SPIN_END` 0.45, `AXIS_RESET` 0.60 and `AXIS_CHIP` 0.79. By even word spacing
over its 34 words, "where it was" ends at 47%, "only the rod's points" starts at 62% and "so any other axis" starts
at 79%. The review suggested resetting at the second clause (47%), which leaves almost no hold on the miss, so I hold
through "and our half turn leaves". **These shares are estimates, and fine sync is unreviewed.** They are named
constants at the top of the file.

I moved `axis_chip` from straight after the spin to 79%. The review did not ask for this, but "axis: only the rod" is
the sentence's conclusion, and showing it at 45% would be F7's defect again.

The 12_6 spins in sentences 1 and 2 keep their old lengths and only lose their easing. F5 did not list them.

## `predict` audit (F6)

**No cue moves.** `hold` and `window`'s `hold_breath` wait for the same frame under `LineGaps::Constant`, which this film
uses (`main.rs:21`), and `Thought` gaps are never longer than `BREATH`. Without a narrator `hold` does nothing. So the
next `kit.say` lands where it did. What changes is when the bar appears and how much of it drains in silence.

At all 13 sites `predict` is the last beat before the next `kit.say`, so nothing else waits on it. Lines are canonical.

| Site | Cue | Sentence anchored before the call | Its words | Timed beats between anchor and call |
|---|---|---|---|---|
| `building_ruler.rs:143` | 01_3 | second to last (the last has 3 words) | not counted | three tweens, 2.8 s |
| `complex_turns.rs:167` | 05_1 | last | 6 | one fade |
| `complex_turns.rs:547` | 05_5 | last | 6 | one fade |
| `branch_cut.rs:326` | 07_3 | last | 8 | one fade |
| `lifting_loops.rs:238` | 06_2 | last | 11 | tween 1.6 s, one fade |
| `lifting_loops.rs:349` | 06_4 | last | 9 | one fade |
| `product_rule.rs:211` | 08_1 | last | 9 | one fade |
| `product_rule.rs:336` | 08_4 | last | 3 | none |
| `slide_rule.rs:150` | 02_2 | last | 6 | one fade |
| `unreachable.rs:525` | 11_3 | last | 7 | none |
| `ratio_ruler.rs:362` | 00_4 | last | 9 | one fade |
| `what_it_forgets.rs:744` | 12_5 | last | 11 | lagged fade 2.4 s |
| `undoing_growth.rs:539` | 04_6 | last | 4 | one fade |

By this reading the voice is usually still going at the call, so the other chapters probably have the same defect.
**That is inferred from source. I confirmed it only on chapter 12's sheets; I did not look at the other chapters' bars.**

Two side effects for root to expect:

- `kit.hold` adds one layout-lint sample per site, labelled `hold`. It could raise a finding at a moment the lint did not
  sample before, though the picture there is one the film already shows.
- Wherever the voice was still going at the call, the bar now first appears after the voice stops, where it used to
  appear during the line. Its drain always takes the full `THINK` seconds, in silence.

## Tests

Two new, one rewritten, one extended: 11 tests in this file, up from 9. Every threshold was run in the mirror.

- **New `a_spin_about_another_axis_draws_the_drag_and_the_miss`:** guards drawn length, the check that was missing. It also
  asserts that the top centre's path draws nothing, which is why the pin exists. All eight corners miss by one edge and land
  on corners.
- **New `a_motion_ends_before_the_next_sentence_starts`:** `room` and `sentence_len`, including no narrator and a short sentence.
- **Rewritten `every_ellipse_flow_leaves_one_place_parts_and_comes_home_there`:** shared start and end, the meeting at t = ½ (true
  and harmless, as the review noted), each on its own ellipse, and at least 0.3 apart at t = 0.3 (mirror: 0.761, 1.227, 1.361).
- **Extended `the_panels_hold_their_pictures`:** the two panel C strings, orbit clearance, red line and pin sweep bounds.
- **Width estimator:** the `0.55 × size / 144` closure is gone. The CMR10 table reads +0.6% on the `Log zw` string against
  WhiteSwan's measured 2.61, and within 2.5% of four more strings whose ends I read by eye off sheet 080. I added `'L' => 0.625`
  to the table Mail 1144 used.

## What I checked, and how

- **Base:** `what_it_forgets.rs`, `draw.rs` and `script.tsv` hashes as above. Canonical `src` and `script.tsv` equal the snapshot.
- **Frozen files:** nothing under `logarithm/` or canonical is newer than Mail 1144's `REPAIR.md` except this directory. The Mail 1144
  patches still hash `592989d16fd57f4f` and `f5e9242445255958`.
- **Patches:** `git apply --check` is clean, and applying both to a scratch copy reproduces the staged files byte for byte. No absolute
  home paths in the patches, `checks/` or this report.
- **Format:** `rustfmt --edition 2024 --check` passes on both candidate files, which also proves they parse. Nothing more about the Rust
  is proven.
- **Arithmetic:** `python3 -I -B <this dir>/checks/verify.py <this dir>/checks` from the repo root reports `FAILS: 0`. It compares against
  canonical, so run it before integration. `checks/timeline.py` prints the timing table, also from the repo root.
- **Literals:** scene-code strings differ only by six cue ids passed to the new helpers. `until_sentence` (19) and `kit.say` (6) anchors keep
  their order. No new characters on screen.
- **Numbers:** `SHARED_START`, the shares and `PIN_TIP` are positions and timings, not shown numbers. No oracle key changes.
- **Pictures read:** 18 of 91 sheets: 017, 026, 036, 042, 047, 056, 058, 059, 064, 070, 071, 072, 073, 074, 079, 080, 084, 089. Also
  `frame-0316.png`, where a dark wedge under the cube on sheet 079 proved to be background (pixels within one level).

## Predicted crowding to check on root's frames

Computed, not seen:

- **Red line and end ring:** the line passes 0.187 from the ring's centre (radius 0.2), so it grazes the ring for the whole red spin. The
  ring is at screen (−4.46, 0.53).
- **Pin tip and end ring:** the tip's dot clears the ring by 0.003 at t = 0.488, then its trace stays beside the ring.
- **Corner trace, red line and pin arc are all red.** The sketch reads clearly, but the sketch is not the film.
- **Reset at about 60% of 12_6 sentence 3:** the pose jumps from the other axis's end to the rod's half turn. The old code jumped too.

## Left alone

From the review's lower-priority list, none blocking and none changed: hard resets at other points; spoken "its own circle" and "the whole
family"; "Fold the ruler" with no fold; no keyboard picture in 12_1; "9 10" crowding; the cut crossing the −1 target; dim 16 pt tick labels; the
title-only hold; frame 111's ghosts. `docs/storyboard.md:165` says "(red trace)", which is still true of the pin's tip and the corner.

## Remaining limits

- **Uncompiled.** No Cargo, tests or Clippy ran. The Rust has only been parsed by rustfmt and mirrored in Python.
- **Unrendered.** No frame of this repair exists. Whether the pin reads as part of the cube can only be judged on root's frames.
- **No motion review.** WhiteSwan's and my evidence is half-second stills, and the steadiness of `steady` is predicted from `rate::linear`.
- **No listening, no fine sync, no independent AV.** All timings above use scratch-voice shares. How the beats sit against Oliver is unknown until his
  takes exist and someone hears them.
- **Other chapters' think bars are unseen,** both before and after this change.
- **Higham** was not reopened for this repair; the on-screen citation is unchanged.

## What root needs to run

1. Apply the patches, then build, tests and strict Clippy. Compile-risk spots: `steady`'s path `fmn::core::rate::linear` (as in golay's `draw.rs`),
   `stage.scene().time()` in a chapter (as in `abstract_algebra_visual`'s `parity.rs`), and the two `const { assert!(..) }` on float constants.
2. Fresh preview of chapter 12, then frames at the F1 spin, the panel C and D text, the think bar and 12_6 sentence 5.
3. Because of the shared patch, fresh previews of the think bar in chapters 00, 01, 02, 04, 05, 06, 07, 08 and 11, with cue tables compared against the old ones:
   they should match exactly.
4. Keep motion, listening, fine sync and independent AV open.
