# Complex 11: counting wording review and reach label repair

WhiteSwan, for BlueSalmon, answering Mail 2586. Staged only. Nothing here has been built with Cargo, rendered or synthesized.

## Verdict

- **The wording is mathematically right, and the four sentence anchors are right.** I would keep root's candidate.
- **One blocker:** root's `script.tsv` fails the crate's own test until the 11_7 storyboard row changes. Patch 02 fixes it.
- **One picture gap to decide:** nothing on screen answers "Divide by its leading coefficient". The picture holds on the claim about 7 s longer than before.
- **The reach label is moved** above the output window. The disk cannot reach it anywhere in the sweep, and no timed call or on-screen string changes.

Decisions for root are in section 3. The label has been seen only in composites of preview02 pixels, never in a native picture.

## 1. What is here

| Item | Against | Result | SHA-256 |
|---|---|---|---|
| `patches/01-reach-label.patch`: the assigned fix and four tests | root's `counting.rs` `7c5a41c6fd37ad5d…` | staged `counting.rs` `f3c75f8cff9d06b39761e8ac3533db815de84a11b9e635ceb6d38cceccc19d9a` | `77737e5f928857894180125904a2eed85e98b0c728ca196162b3f5201687d9fe` |
| `patches/02-storyboard-11_7.patch`: root's exact 11_7 in the row | canonical `storyboard.md` `be30f73484bd69cb…` | `storyboard.md` `6803d39a27bbb2ba093d5e96e6bff3d0b12679c2d63d17e906436a3ca0762f96` | `310b1284d3d36a4e4893cec314fee2b7b3d9bf7ade9da90016aaff664b49bbe5` |
| `patches/03-optional-a-name.patch`: optional, see 4.2 | staged `counting.rs` | `counting.rs` `20ad375dcce1c1172a976a546507a108ce12e746c8fea939cefbc0f2bc6fecdb` | `23ab1e91b9665246feef592fc4705230550203653550e250d4e76111c5b9abf8` |

Root's `script.tsv` `1e24aba8b520f2e6…` is unchanged and is the one to use. Apply with `patch -p1` from the repo root: root's wording first, then 01, 02 and, if wanted, 03. My unapplied glyph patch `complex-glyph-spacing.patch` fits beside 01 and 03 in either order and gives the same file.

## 2. Review of root's candidate

| Asked | Result | Evidence |
|---|---|---|
| Nonconstant | Correct and needed. A constant makes no turns, and a nonzero constant has no roots. | Reasoned; new test runs the constant case |
| Complex | Correct. Real polynomials are included, and their roots may be complex. The screen says `nonconstant polynomials have roots` without "complex". That is still true in this film, and the voice says "complex" over it. | Reasoned |
| Monic step | Correct. The leading coefficient is nonzero by the definition of degree, and dividing by a nonzero constant keeps every root and its order. "Some positive integer n" is what makes "exactly n roots" yield at least one. | Reasoned; new test on `(2−3i)z³ + z − 5`: 3 turns before and after dividing, and the leash is shorter than the walker on `|z| = 50` |
| Targets 2, 3, 4, 6 | All land on the right words: `degree_n` on "Now it starts", the circle on "On a huge circle", the dog on "Picture", the walk's end on "So there are". | Source checks, new test, seven defects planted |
| Width: claim | About 428 px wide, x 115–544, against the output window at x 707. | Estimate calibrated on frame 360 (old claim measured at 320 px) |
| Width: card | One line of 58 characters, ending near x 785 of 1244. The band top drops from about y 610 to about 643, so it no longer sits against `one root inside` (frame 340). | Estimate and `lower_third` source |
| Glyphs | Every character is already in a film string. The semicolon is drawn natively in the same lower third on chapter 13, frame 456. | Source check; frame viewed |
| Card meaning | "Direction and turns around zero unchanged" is exact: `draw::squash` only rescales the radius. | Source read |
| Script lint | 0 errors, 35 warnings, the same count as before. 11_7's sentence warning goes from 6 to 7. | Lint run on both scripts |
| Unit limit | 96 words against the limit of 100. Following context around 11_7 was already over the limit before this change, so nothing new closes. | Source checks |
| Phrases | 11_7 uses no phrase anchor, and 11_8's words are unchanged. | Source check |
| Takes | Only 11_7 needs a new take. I found no `context_after` or `following_context` in root's ElevenLabs folder, so neighbouring takes do not bind to its old text. | Grep only, not a receipt audit |

## 3. Decisions for root

### 3.1 Blocker: apply patch 02

`main.rs`'s `the_storyboard_the_registry_and_the_script_agree` compares `script.tsv` with `docs/storyboard.md`. I built the real `check_video` with rustc:

| Storyboard | Script | Result |
|---|---|---|
| canonical | canonical | agree |
| canonical | root's | fails: "script.tsv is not the storyboard's words" |
| patch 02 | root's | agree |

Separately, `tools/storyboard.py --check complex_analysis_visual` already fails on the canonical pair: three chapters report no prediction pause, and the script's header line differs. That is older than this work, so it is not a usable gate here.

### 3.2 Medium, not staged: sentence 1 has no picture

Nothing is keyed to sentence 1. While the voice says "Divide by its leading coefficient", the screen holds only the claim and the empty output plane, as on frame 360. At preview02's scratch rate, by character share, `degree_n` arrives about 11.4 s into the cue instead of 4.3 s. These are estimates, not Oliver.

If you want a picture there, this adds no film time because the next wait absorbs it. It is uncompiled, and `LEADING_TEX` also needs a place in `mod.rs`'s `TEX` list:

```rust
pub const LEADING_TEX: &str = r"p(z) = a\,z^n + \cdots";
// after the claim fades in:
line.wait_sentence(stage, 1)?;
let leading = kit.tex(stage, LEADING_TEX, 36.0, Palette::ink(), &[], v(-3.45, -0.3))?;
fade_in_all(stage, &[leading], 0.4)?;
line.wait_sentence(stage, 2)?;
fade_out_all(stage, &[leading], 0.2)?;
```

I left it out because it goes beyond the narrow fix you asked for. I can stage it if you want it.

### 3.3 Low, not staged: the card has no subject

"size compressed; direction and turns around zero unchanged" never says what is compressed. I suggest `output size compressed; direction and turns around zero unchanged`. It is 65 characters, still one line, and uses no new glyph.

### 3.4 Note: the 7 s window is kept, as you asked

The card nearly halved, so the test's numbers are tighter. Clean time is 7.2 s, against a floor of 3.87 s and a ceiling of 7.73 s. With the 7 s kept, a card of 53 characters or fewer fails `reading_windows_leave_time_for_what_the_voice_leaves_out`. The map's reasoning for this hold cites the card's length, so it is now slightly stale.

## 4. The reach label

### 4.1 What was wrong and what changed

Viewed on sheets 136–137 and enlarged crops of frames 541, 544, 546–553. The disk's line runs over `reach L × step` on frames 548–551. Measured from the frames, the sweep runs from about 186.9 s to 190.0 s, and by geometry the line is on the label for shares 0.645–0.959 of it, about 1 s. The preview02 layout lint did not flag this.

The label now sits at `out.p(0, 2.2)`, native (410.5, 171.9), the place `reach covers 0` has just left (`counting.rs:602`, `:778-779`, `:815`). Both labels now share one constant.

| Geometry, at 1280 × 720 | Value |
|---|---|
| Disk radius | π/4 window units, 49.5 px; the sweep reaches 112.5 px from the window centre, native y 198 at the top |
| Nearest the disk comes to the label, over 1,440 sweep steps | 18 px, using rounded-up label extents |
| Label ink, moved pixels | x 360–461, y 164–179 |
| Window border below it | rows 183–185, a 4 px gap |
| Title to its left | ink ends at x 328, a 32 px gap |
| Code panel to its right | starts at x 564 |

**Why not inside the window.** Fitted flush into the emptiest corner, the measured label misses the sweep by about 0.1 px while touching the border. The test proves every position inside the window is within the sweep once extents are rounded up. Below the window would also be clear, but above it continues the same label slot within one beat. A label carried with the disk would add motion and run into the axes.

**What it costs.** The label no longer sits near the disk. It names the disk by colour only: disk, label, row and code band are the only teal on screen. Please judge this on the native pictures.

### 4.2 Seen in the same sweep

- **Moving `a`, optional patch 03.** The white distance line runs through `a` on frame 551, and `a` sits on the output circle's dot on frame 541. Turning the name with its sample keeps it off the line, inside the disk and off the circle. At the walk's start and end it stays exactly where it was. It changes film code in three places: a new constant and the two positions that use it. It does not change timing.
- **Stationary `0`, no patch.** The disk's line or the distance line touches it for about 40% of the sweep, by a geometry estimate (frames 546–548). Moving it cannot fix this: the disk's line crosses everything from 13.5 px to 112.5 px from zero, and the distance line crosses everything inside the circle. The hole is too small to hold the glyph clear of the axes. Painting `0` last would keep it on top, if you want that after seeing it natively.

## 5. Rebinding still needed (root's lane)

- `narration_units.json`: `script_sha256`, `direction_contract.script_sha256`, `direction_contract.map.sha256` and `units[76].text`.
- The direction map: `scripts`, and the 11_5, 11_6 and 11_8 rows of `visual_holds`. Their `sha256` changes, and their lines go 429 → 428, 492 → 491 and 547 → 546. The lines are the same in root's candidate, the staged file and patch 03.
- A new exact Oliver take of 11_7 with seven measured sentence starts. With any other count, `say` silently falls back to character shares.
- Everything after 11_7 moves. The estimate at preview02's scratch rate is 31.9 s → 39.9 s; Oliver's real length is unknown.

## 6. Native anchor regression

From the repo root, on sampled frames of a rerender:

```
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-review-repair01/checks/anchor_probe.py <frames-dir> BEFORE=<n> SWEEP=<n>,<n>,...
```

- `BEFORE`: a frame after the label arrives and before "cannot move".
- `SWEEP`: several frames from "cannot move" to the sentence "Then no turn can hide". Include one near 0.8 of the way through, because the probe requires a crossing moment.

It checks that the label is whole over the window, the old place is empty, the slot's pixels never change during the sweep, and the disk really travels. It fails on preview02, as it must, and passes on the composites. A pass is not picture acceptance.

## 7. Checks run

| Check | Result | Output |
|---|---|---|
| `checks/source_checks.py`: baselines, round trips, 112 timed calls unchanged by 01, 121 strings unchanged by 01, film code differs only in the label's 12 lines, 17 tests kept | 67 pass, 0 fail | `checks/source_checks.out.txt` |
| `checks/standalone_tests.py`: the new tests under real rustc with the real `model.rs` and a shim | 5 tests pass, 19 of 19 planted defects caught | `checks/standalone_tests.out.txt` |
| `checks/anchor_probe.py`, controls | preview02 fails, composites pass | `checks/anchor_probe.out.txt` |
| `rustfmt --check` on the staged file and on 01 + 03 | clean | run by hand |

`checks/desk-composites/` holds real preview02 pixels moved with PIL, each stamped as not a render. I viewed the composites from frames 535 and 550. `checks/baseline.sha256` pins the frames and sources used.

**Evidence.** Actual pictures viewed: chapter 11 sheets 136–137; frames 340, 360, 526, 535, 549; the enlarged crops listed in 4.1; chapter 13 frame 456. All are preview02, which predates this change and uses the scratch voice.

**Not done, and not claimed.** No Cargo, Clippy, `cargo test`, render or synthesis. `construct` has not been type-checked with fmn. No native picture shows the moved label. No listening, fine sync or final AV review; "nonconstant" has not been heard from Oliver.

## 8. Authorship overlap

This review is not independent for the label. My own `complex-picture-repair-11-15` (items 11-S and 11-V) put `reach covers 0` in this slot, moved `reach L × step` to (0.95, −1.2) and added the sweep. **The crossing is a regression from that repair.** My test then checked that the disk stayed in the window, not that it cleared the label. The glyph patch is also mine.

I did not write root's wording, the sentence shift, or their receipts, so that review is independent of them.

## 9. Remaining risks

1. All Rust here is unbuilt in the crate. The tests' logic ran under rustc with a shim; `coarse_loop`'s three changed lines have only been format-checked.
2. Native label extents may differ slightly from the moved pixels. The 4 px gap above the border is the tightest margin.
3. Naming the disk by colour alone may read less clearly in motion.
4. The sentence-1 hold may feel long under Oliver, depending on your decision in 3.2.
5. The test's label and title sizes are fixed numbers measured on preview02. If those strings or sizes change, the numbers must change too.
