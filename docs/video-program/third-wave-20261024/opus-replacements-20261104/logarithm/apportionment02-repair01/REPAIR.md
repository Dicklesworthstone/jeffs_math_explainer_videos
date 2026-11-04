# Apportionment chapter 02: staged repair 01 (F1–F7)

RubyRaven for BlueSalmon, Mail 1326, 2026-11-04. Findings are in `../apportionment02-independent-review01.md`, which stays frozen.

**Status: author repair, uncompiled and unrendered. Not an acceptance, and not independent review of this repair.** Root owns integration, the guarded build and new pictures. Narration stays ElevenLabs Oliver; nothing here was heard or synthesised.

## Files

Only `shares.rs` changes. The script, `draw.rs`, `mod.rs`, facts, model and storyboard are untouched, so no cue needs new synthesis.

| File | SHA-256 |
|---|---|
| Baseline `explainer_corpus/apportionment/src/chapters/shares.rs` (copy in `baseline/shares.rs`) | `45e9521207b3a68c20c781bc93721cc161b395b35e70a4dbfc0f5bf1ee34ba49` |
| Candidate `explainer_corpus/apportionment/src/chapters/shares.rs` in this folder | `836a2bc808714b2b463960ec2e6ca4e5ae9332cbe527aab4003fbd8efa648b2c` |
| `shares-repair01.patch` | `4a43b6725eb117892f1235f1004e6e3884b6bc7ba369cc74fe8e38d683be8afc` |
| `checks/check.py` | `092f9ca500a82709c1937931f34f1d0fca17994c89ff0709d0098c20c1702521` |
| `checks/check.out.txt` | `0eca8647e6ddee8f7bf5fe65a06ed24742a3b041f2aa586737172aee06e4aba6` |

Other baselines, all unchanged at handoff:

| File | SHA-256 |
|---|---|
| `script.tsv` | `cfcec06911c7d94efa6b7b29a820be9bf73a608810b4b57085f02f5b95939313` |
| `src/chapters/draw.rs` | `dd385ed5f45f09ffb8c0d8563829601bb147460bc16c7d4cbbf08245a7b23ba4` |
| `src/chapters/mod.rs` | `050e940041bc0f3e60faa9ec65815a7da93abda4228d814cfec0a09f908b10cf` |
| `src/model.rs` | `91baa9ca3f1d050de21375050f6f881401df7de7471c4fe0aa6e46f74ef2c5cf` |
| `src/facts.rs` | `ec55517c9544e275a8ba3b639b07be2690e347dc564bac93849d6874f3d3faa6` |

**To integrate:** confirm the baseline hash, then apply from the repository root with `git apply` on `shares-repair01.patch`, or copy the candidate over. `git apply --check` passes against the current file. Applying the patch to a scratch copy gave the candidate's exact hash.

The build should see one new test, `the_drawn_division_is_the_share`.

## What changed

Times are scratch estimates from `checks/check.out.txt`.

| # | Repair | Lands at |
|---|---|---|
| **F3** division | "39,000 people" slides over and "÷ 15 seats" joins it. The line now stays up after 02_1 ends, and "share = people ÷ 2,600" appears over it and stays through the rows. | 8.51; 19.73 |
| **F3** continuity | Ticks, brace and labels leave while the three bars wait between the rows. Each bar then morphs into its own share band as the voice names the town, with "4,000 ÷ 2,600 = 1.54" beside it. This builds the storyboard's "splits back into the three rows". | 21.93; 23.03, 25.89, 28.12 |
| **F4** | "→ ?" on each row. The "?" morphs to 2, 3, 10. The green sum gets its own `wait_for`. | 41.85; 52.36; 57.56 |
| **F2** table | The row just seen gathers into the table's row 15. Then row 14 by phrase: seat count, each share on its spoken number, rounded, "13" and flash. Then row 16: seat count and shares, rounded, "17" and flash. | 70.50; 71.20 to 81.93; 81.93 to 86.64 |
| **F7** | Rounded values are three aligned cells in town colours. | with F2 |
| **F6** | Rule left out of the dim. "1 short" and "1 over", computed from `h` and `sum`, appear in red beside the totals. | 96.78 |
| **F5** | Both empty frames appear when the cue starts; labels come on their phrases. | 101.97 |
| **F1** | Flash the whole card while the field is dark, then fill it gold and turn the words dark: 13.75:1 by palette values, against 1.19:1 before. | 113.92 to 115.42 |

All values come from `model`: `share`, `naive`, `total`, `decimal`, `grouped`. The new test pins "people ÷ 2,600" to the model's exact share.

### Where this departs from the review's proposals

- **"17" lands on "hand out", not "seventeen".** My proposed sequence in the review ended 0.41 s past the line. That is an error in the review: `Narrator::say` allows no overrun at a coherent seam. It now ends 0.40 s inside the line, and "17" is whole about 0.3 s before its word.
- **Row 15 morphs into place** instead of fading in with the header, for object continuity.
- **Each readout shows its division**, not only the general rule. The arrows move right to x = 3.75 to make room, and Cedar's readout starts one digit left so the "÷" and "=" line up.
- **The tags use digits** ("1 short"), so the whole label is computed.
- **Small layout changes:**
  - `LINE_Y` goes from 0.6 to 0.75, so the waiting bars clear both grids.
  - "39,000 people" goes up with it, to keep the same gap.
  - "share = quota" moves to the rule's height.
  - The table rule grows to x = 5.8, to span the tags.
- **Both table flashes are 0.7 s**, not 0.8 s.

No script change is proposed. The storyboard's picture cells for 02_2 and 02_6 now understate the picture. `check_video` compares words, not pictures, so nothing fails; updating them is root's call.

## Native APIs checked, at pin a399d626

| Fact | Where | Why it matters |
|---|---|---|
| `Transform` keeps its source; the target stays detached | `largest_tails.rs` L138 comment, root's `result-handle-repair01` | After a morph the code uses only `bars`, `reads`, `marks`, `need`. `check.py` fails if a target is used later. |
| `indicate` changes RGB and scale, not alpha | `fmn-anim/src/transform.rs` L350, L311 (`set_family_rgb`) | The hollow card stays hollow during the flash. |
| `set_color_with_opacity` also fills a hollow rectangle | `fmn-mobject/src/animate.rs` L584, and review frames 266–276 | Solid gold is the seen behaviour; the fix turns the text dark rather than fighting the fill. |
| `.rt()` comes from `explainer_kit::Timed` | `explainer_kit/src/play.rs` L7 | The new local helper needs it, so the import is added. |
| A late `wait_until` does nothing | `draw.rs` L985–992 | Row 14's seat count starts at 71.20, past "At fourteen" (70.32). |
| `say` rejects any gap at a coherent seam; this film uses `LineGaps::Constant` | `narration.rs` L1182–1217, `main.rs` L41 | Why every step ends inside its line. |
| `pause(0.6)` is exactly 18 frames at 30 fps | `frames_covering`, `narration.rs` L605 | `kit.hold` plus `pause` keeps the wipe's timing. |
| `÷`, `→`, `?` are in the glyph list | `chapters/mod.rs` L28–30 | No glyph change needed. |
| `share_readouts` still has other callers | `webster_quota.rs`, `rounding_rules.rs` | Dropping its import here leaves no dead code. |

## Cheap checks run

- `rustfmt --check` on the candidate with the repo config: exit 0. That parses the file and checks its format, and does nothing more.
- `python3 -I -B checks/check.py` from the repository root: all pass.
  - The arithmetic, in exact fractions.
  - The seven `say` ids in order.
  - All 28 phrases, each once in its own line.
  - The morph-target rule.
  - Every scheduled step ends inside its line. The tightest are "17" with 0.40 s of room and the final card with 0.96 s.
  - Text boxes in each new view against the lint's limits. The width estimate matched two texts measured on frames: 1.77 against 1.78, and 2.25 against 2.27.

**Expected on the scratch build:** 3528 frames and the same `cues.tsv` starts as preview01. A difference would mean a step overran.

Longest stills under the scratch voice are now 7.35 s (58.26–65.61, the question before the ring) and 7.13 s (12.59–19.73, the finished division). The baseline's were 14.5 s and 8 s.

## What to look at first on new pictures

Scratch times:

| Time | Look for |
|---|---|
| 9–10 | "39,000 people ÷ 15 seats" reads as one line, with no gap or collision |
| 22.5–23 | Bars clear of both grids and of Ash's "4,000 people" and Birch's name |
| 23–29 | Bars land exactly on the band positions. **Cedar's bar crosses Birch's row** on its way down; it should pass behind |
| 24–70 | **Grid lines over the bands.** The bars enter the scene before the grids, so lines should now paint over the bands; the baseline had bands over lines. I expect it to help seat counting, but it is a change |
| 42.5, 53 | "→ ?" and the "?" to digit morph, with the digit left-aligned |
| 70.5–71.2 | **The gather is the largest risk.** A readout of about 20 glyphs collapses into 4 as it travels. If it looks messy on motion, the simple fallback is to fade row 15 in with the header and fade `reads`, `marks` and `need` out; the row-15 handles in 02_6 then change too |
| 83–87 | "17" and the flash, then the join into 02_6 |
| 100 | Rule no darker or brighter than before the dim; tags readable |
| 114–116 | Flash on the hollow card at scale 1.25 (x −6.80 to 0.20, clear of the second card at 0.50). The words cross the gold during the 0.5 s fill. Then a contrast measure on the settled card |

## Not done

- No compile, Clippy, tests, render or frames. Compile errors remain possible; I read every line against the helpers' signatures, and that caught the missing `Timed`, but it is not a build.
- No motion review, listening, Oliver timing, fine sync, independent AV, score or 4K. Every time above is interpolated from the scratch voice. Re-check the 0.40 s of room on "17" against the real take.
- No canonical edits, Beads, commits or memory edits. The review, root's chapter 03/04/05 edits and WhiteSwan's chapter 12 staging were not touched.
