# Apportionment preview02, chapter 12: independent picture review

LavenderPike, 2026-11-04. Answers Mail 1504 (BlueSalmon), thread `mx-video-program-program-87do.30`.
Staged review only. Root owns .30. This is not acceptance.

## Verdict

- **No High finding.** Every number on every sampled frame agrees with my own arithmetic. The answer hold holds: no mark through f61 (20.133), first gold box f62 (20.433), and 12_2 starts at 19.933. The spoken proof holds.
- **The author's F1 to F5 are repaired on stills.** F1, the High one, no longer stands on these frames.
- **Three Medium teaching problems remain, all in the proof (12_8 and 12_9).** N1 and N2 are clear. N3 is a judgement call. All three can be fixed in the picture without changing a spoken word.
- **Recommendation:** one more picture-only source pass, N1 and N2 first, because a viewer meets them at the point where the proof has to land.
- **Limits:** these are stills at scratch-voice timing. Motion, listening, fine sync and final AV are all open.

| ID | Sev | Frames (scratch s) | Problem | Source |
|---|---|---|---|---|
| N1 | Medium | f563 (197.5) to f590 (207.5) | Gold rings land on "all those tiles", so the screen says 5 while the proof needs 8 | 1008 to 1019 |
| N2 | Medium | f532 (187.0) to f626 (221.5) | Grey veils are a third state the only key does not name | 895 to 902, 955 to 984 |
| N3 | Medium, judgement | f488 (170.5) to f498 (174.0) | Seven fills vanish on "Suppose one were"; the reason arrives 3.5 s later | 866, 887 |
| N4 | Low | f550 (194.0) to f626 (221.5) | The band tints the missing tile, its "?" and its caption; residue of F3 | 984 |
| N5 | Low | f519 (182.0) to f579 (203.4) | The chip row has no words on screen for 21.4 s | 932 to 940, 1028 |
| N6 | Low | f168 (60.0) | Cedar's unchanged "10" garbled by a needless morph | 499 to 504 |
| N7 | Low, optional | f307 (107.5) to about 113.4 | No picture of "as long as the stretch ... to earn one whole seat" | 632 to 636 |

The author's remaining open items are in the history table below: F6, F7 (tile flash only), F10, F11, F12, F13.

## How independent this is

Not clean-room, and root should weigh this.

- Before I had read Mail 1504's limits, I read my memory note `apportionment-lane-state.md` in the same parallel call. It lists the author review's finding headings: F1 High real fills during supposition, F2 solid green in 12_10, F3 four rings against "= 5", F4 about 29 s nearly still, F5 think ring cuts "2,000", F6 to F13 low, and Lesson 3. The memory index also carries a one-line version on every session load. So my first pass was primed on those headings.
- That note says the candidate and the author review were written "as WhiteSwan", and it sits in the memory directory this project shares. I can't establish what that means for authorship across sessions, so I'm stating it rather than resolving it. In this session my only work on this chapter is this review.
- I did not read the bodies of the earlier findings until all 178 sheets and the source were done. My notes from that pass are kept unedited in `checks/first-pass-observations.md`, including the mistakes withdrawn below. They were copied there after the pass, so the file's date is not evidence of order.
- N1 to N7 are mine. The author's handoff flagged N1's subject as an open question (risk 6); I raised N1 from the frames first and read that afterwards.

## Evidence

**Denominator (from `sampling.json`):** 178 of 178 sheets viewed, 709 frames (`frame-0000` to `frame-0708`), 499 on the half-second grid, largest sample gap 0.500 s, duration 249.566016, last sample 249.316. Sheet 177 holds f708 only. Scope text: "Half-second, cue-boundary and layout anchor/persistence samples; later observations may be nonconsecutive. Not continuous-motion or listening proof."

**Render:** `target/debug/apportionment render all --res 1280x720 --fps 30 --scratch-voice --lint --crf 26 --preset ultrafast --threads 4`, started 2026-11-04T09:34:27Z, exit 0. All times here are scratch voice, and phrase times inside a cue are interpolated. Picture order within a cue is bound to words through `wait_for`, so the order should survive Oliver takes but the seconds will not.

**Files:** under `narration_lab/third-wave-20261024/root/five-film-release-20261027/`, frames in `apportionment-opus-rebuild-frames02/12_both_promises/`, render in `apportionment-opus-rebuild-preview02/render/`.

| Input | sha256 |
|---|---|
| `12_both_promises.mp4` | `30354ccd9664aa96a5b0628dc67395ceb296024ff97bf338a83b0f602f06db13` |
| `12_both_promises.cues.tsv` | `3a5b4ebfa23fe5f7f7f83a000f97bc1339372dcc136080dd2bb581e66dc1e8a8` |
| `12_both_promises.layout.tsv` | `b403b4d379ef19aea05c2b8a8d752d84225728c669ce5a9da54e359fda71ae12` |
| `sampling.json` | `aded023718729dea188879420750e57413663a8094469db9dc3e883a11652917` |
| `both_promises.rs` | `c667ad1a29a81ed9d880aa8d830a162380e7ebea7c2b347edb0dafd6d88de8e3` |
| `script.tsv` | `cfcec06911c7d94efa6b7b29a820be9bf73a608810b4b57085f02f5b95939313` |
| `draw.rs` | `dd385ed5f45f09ffb8c0d8563829601bb147460bc16c7d4cbbf08245a7b23ba4` |
| `model.rs` | `d7bb080faba88e63bf1261f81b49b54bf5c5b9b36191caf9fe321e1d140d6f6b` |

The four source files were re-hashed against canonical just before this report and have not moved; the two chapter files equal the preview02 inputs JSON. Canonical `both_promises.rs` differs from WhiteSwan's candidate `da41049bbaf9d9fe` only by `Timed` in the line 15 import.

**Reviewer's checks (mine, not native evidence):** `checks/arithmetic.py` uses exact fractions from the three populations, with no crate import and no read of `oracle.py` or `facts.rs`; output in `checks/arithmetic.out.txt`, exit 0. `checks/sheets.sha256` binds the 178 sheets. `HASHES.sha256` binds this folder.

**Cues (start, length):** 12_1 0.600 14.246; 12_2 19.933 30.785; 12_3 53.233 21.554; 12_4 76.000 18.862; 12_5 96.067 25.015; 12_6 121.700 21.554; 12_7 143.867 21.554; 12_8 167.433 29.246; 12_9 197.300 36.169; 12_10 234.700 14.246.

## New findings

All fix sketches are uncompiled proposals against `both_promises.rs` at `c667ad1a`. None changes a spoken word, so the existing Oliver takes are not disturbed by the text; cue starts could shift slightly, which needs a build to see. `bring_to_front` was read at engine pin a399d626 (`stage.scene_mut().bring_to_front(&[...])?`, as used in `bakery_algorithm/src/chapters/room.rs:289`).

### N1 Medium. The gold rings land on the wrong sentence

**Frames:** first ring f563 (197.5), all four by f566 (198.4), on "So all those tiles fit inside the stretch" (line 1008). They ring Birch 3 (8⅔ to 13) and Cedar 6, 7, 8 (7½ to 12), with Cedar 5 in red. The caption "7 seats from then to now: 7 + 1 = 8 tiles" arrives at f579 (203.4), full at f581 (204.0). "But each town's row fits at most" and the row bounds arrive at f590 (207.5).

**What goes wrong:** "those tiles" means the eight tiles the voice has just described, shown as eight chips. Rings arrive on those words and the viewer counts five. For 10 s nothing reconciles 5 and 8, and the first 5 s of that are a still.

The rings also can't be "those tiles" by the film's own definition. `then` is the last hand-out of a tile ending later than the missing one, so anything handed out since then ends by 7½. Every gold-ringed tile ends later than 7½ (`checks/arithmetic.out.txt`). The speech is sound, because it only uses "ends by now"; the mismatch is in the picture. The source already knows this: line 932 calls the chips "a count, not a place". The rings show row capacity, which is the next sentence's subject.

The author's handoff left this open (risk 6: "points at where they would have to go"). My answer is to move them.

**Fix:** let the chips answer "those tiles", and bring the rings in with the sentence they illustrate. The `fit` guard stays as it is.

```rust
s.wait_for(stage, "So all those tiles fit inside the stretch")?;
pulse_all(stage, &chips, None, 0.6)?;
// the `fit` rings are built here as now, guard included, but not yet shown
s.wait_for(stage, "one more of them than seats added")?;
// need_t as now
s.wait_for(stage, "But each town's row fits at most")?;
fade_in_lagged(stage, &fit, 1.0, 0.2)?;
fade_in_lagged(stage, &bounds, 1.2, 0.3)?;
```

On scratch timing this leaves about 3.7 s before the sum at f605 (213.4).

### N2 Medium. The grey veils contradict the only key

**Frames:** the legend "solid: real up to then, hollow: only supposed" is up from f498 (174.0) and starts to fade at f627 (221.8). Veils start at f532 (187.0), full by f534 (188.0): Ash 1, Birch 1, Cedar 1 to 4. Then Birch 2 at f540 (190.5). Then Ash 2, Birch 4 and 5, Cedar 9 to 13 at f550 (194.0).

**What goes wrong:** the sets are correct (arithmetic check), and they mean "ruled out" (source comment, line 841). But the screen now has three states and the key names two. Ash has no seat at `then`, yet Ash 1 now looks filled, under a legend saying solid means real. Ash 1 and Ash 2 share a tone for different reasons. No word on screen says what grey means, and this lasts 34.8 s, through the whole contradiction.

**Fix:** extend the key just before the first veil. The legend test only constrains height, and the line is about 5.44 wide, centred, so it fits. `morph_all` keeps the `legend` handle rooted, so the later fade at line 1093 stands.

```rust
let legend3 = kit.text(
    stage,
    "solid: real up to then, hollow: only supposed, grey: ruled out",
    18.0,
    Palette::dim(),
    v(0.0, LEGEND_Y),
)?;
morph_all(stage, &[(legend, legend3)], 0.4)?;
// then the existing fade at line 959
```

### N3 Medium, judgement call. Seven fills vanish on "Suppose one were"

**Frames:** f488 to f489 (170.5 to 170.6) fading, gone by f490 (171.0): Ash 1, Birch 3, Cedar 5 to 9. One red "?" appears. `then` and the legend, which explain the change, arrive at f498 (174.0).

**What goes wrong:** the author's intent is right, since clearing fills before the ring avoids an impossible hole and repairs F1. But the voice has only asked about one tile, and the screen erases seven. For 3.5 s the film looks as if it has undone the run.

**Fix, with its cost:** at "Suppose one were" fade in only the ring, "?" and caption. At "Go back to the last moment", rewind one real hand-out per house, newest first, so every in-between state is a real run. This replaces lines 866 and 887.

```rust
fade_in_all(stage, &[gone, ask, suppose], 0.4)?;
s.wait_for(stage, "Go back to the last moment")?;
let then = stage.add_value_tracker(to as f64);
let then_mark = marker(stage, then, to as f64, Palette::ink())?;
fade_in_all(stage, &then_mark, 0.3)?;
let step = 1.6 / seats_n as f64;
for (i, &f) in since_then.iter().enumerate().rev() {
    fade_out_all(stage, &[f], step / 2.0)?;
    run_tracker(stage, then, (from_h + i) as f64, step / 2.0)?;
}
```

The cost is a red "?" over Cedar 5's green fill for about 2.5 s, itself a small contradiction. That needs checking on frames, and this is the least certain of the three. If root does only two, do N1 and N2. I rejected fading Cedar 5 alone first, because that shows the hole the author warned about.

### N4 Low. The band tints the missing tile (F3 residue)

**Frames:** from f550 (194.0) the gold band, rooted at line 984 after `gone`, `ask` and `wait_t`, turns the ring, the "?" and "6 to 7 1/2: not entered at 6" orange and lowers their contrast. From f566 (198.4) the ring's left edge lies under the white `then` and its right edge under Cedar 6's gold ring. Only two orange edges remain, so the row reads as 3 gold plus part of an orange box beside "Cedar 4". Line 1009 promises the missing tile's ring "stays red". Through f626 (221.5).

**Fix:** `stage.scene_mut().bring_to_front(&[gone, ask, wait_t])?;` after line 984, and `bring_to_front(&[gone])` again after the rings land, wherever N1 puts them.

### N5 Low. The chips have no name

**Frames:** first chips f519 (182.0), seven by f521 (183.0), red eighth f526 (185.0). First label f579 (203.4), so 21.4 s. The voice introduces them, but the screen has no words for them.

**Fix:** a dim `"handed out since then"` at `(NEED_X, PANEL_Y[0])` with the chip fade, morphed into `need_t` at line 1028. If it's built with `morph_all`, the fade-out list at line 1086 must name the kept handle. The red eighth chip is the missing tile, which the voice names as it appears.

### N6 Low. Needless Cedar morph

**Frames:** at f168 (60.0) Cedar's unchanged "10" is garbled while Birch's 3 to 4 legitimately morphs; clean by f169 (60.4). One sampled frame, inside a 0.5 s morph.

**Fix:** at lines 499 to 504, `morph_all(stage, &[(c_j[BIRCH], c15[BIRCH])], 0.5)?;`. `c15` here is used nowhere else. Lines 409, 418, 453, 463 and 529 to 534 pair all three counters too. No sampled frame caught an unchanged digit garbled there (f209 to f210 show only Cedar's real change), but half-second sampling is weak evidence for a 0.5 s morph, so root may want the same treatment there.

### N7 Low, optional. No picture of the length sentence

**Frames:** f307 (107.5) to about 113.4, the frame is still under "so each tile is as long as the stretch of house sizes it takes to earn one whole seat".

**Option:** at "to earn one whole seat", show "1 1/2 × 26/39 = 1" over Cedar's first tile. This needs a facts key and an oracle entry, so it is more than a picture edit. Skipping it is reasonable.

## Earlier findings: author and independent history

F-numbers are from the **author's** review (`apportionment/preview12-picture-review01.md`, WhiteSwan, on frames01), read after my first pass. "First pass" says what my notes recorded before I read those bodies, subject to the priming disclosed above. "Status" is my reading of frames02.

| ID | Author's sev | Author's subject | First pass | Status on frames02 |
|---|---|---|---|---|
| F1 | High | Real fills under "suppose" | Checked, heading primed | **Repaired on stills.** f490 to f626 solid only 0, 2, 4 with no fills after `then`; missing tile now Cedar 5 (6 to 7½); `then` is the real h6, Birch 2; f628 to f629 (222.0 to 222.5) real run 1, 3, 9 returns, captioned. Residue N3 |
| F2 | Medium | Solid green in 12_10 | Checked, heading primed | **Repaired.** f685 to f708 stroked green, cells and tokens visible; f692 (244.5) flash is outline only, scale 1.25, harmless |
| F3 | Medium | Four rings against "= 5" | Checked, heading primed | **Repaired in its old form.** No rings after f627. Residue N4 |
| F4 | Medium | About 29 s nearly still | Saw statics | **Repaired on stills.** Beats at 182, 185, 187, 190.5, 194, 197.5, 203.4, 207.5, 213.4, 216.8. Longest stills now 5.0 s (198.4 to 203.4, 208.4 to 213.4) and 4.8 s (217.0 to 221.8), against the old 12.5 s. Whether that feels right is a motion and listening question, open |
| F5 | Medium | Think ring cuts "2,000" | Saw no cut | **Repaired.** f43 to f58 ring is left of the list; lint edge finding gone (layout.tsv now 4 lines: 1 edge, 3 crossed) |
| F6 | Low | 16-seat result too brief | Saw times, not raised | **Open.** Full f214 (75.0), fading f216 (75.6), about 0.6 s. Fix sketch: `pause(stage, 1.0)` between lines 542 and 547; adds 1 s to the chapter. Outside the F1 to F5 grant |
| F7 | Low | Flashes over labels | Raised tile flash | **Half repaired.** Band flash gone. Tile flash open at line 634, f304 to f306 (106.5 to 107.0): at f305 Ash's first tile spans about −1.2 to 10.95 on the axis against true 0 to 9.75, Birch −0.5 to 4.88 against 0 to 4.33, Cedar −0.17 to 1.7 against 0 to 1.5, crossing "per seat" on the words "as long as". This is the one moment lengths are the subject, so I'd fix it: `pulse_all(stage, &outlines, None, 1.0)` |
| F8 | Low | Strike crosses text | Noted as deliberate | **Keep.** f161 to f204 (58.4 to 73.2), gone by f206 (73.5). The lint's 3 crossed lines are this stroke |
| F9 | Low | Walk runs past the voice | Noted | **Acceptable.** Past 12_2's end at f138 (50.718); f139 (50.918) to f146 (53.233). h7 falls between f134 (50.0) and f135 (50.5), unsampled; source 457 to 469 draws it, and the tile walk shows Cedar 5 at f459 (161.5) |
| F10 | Low | Blank opening of 12_5 | Noted | **Open, deliberate.** f275 to f281 (96.067 to 97.5), line 616. Listen before changing |
| F11 | Low | "+26/39" near Cedar's row | Noted as touching; corrected below | **Open, near-touch.** About 2 px clear on frame-0300, f299 (104.5) to f349 (121.9), lines 621 to 631 |
| F12 | Low | "never with 3 towns" label | **Not raised by me** | **Open, unchanged** at line 564. I have no independent view beyond the author's |
| F13 | Low | "Jefferson's list" at safe-area line | Not raised | **Open.** Lint edge, line 331. Not clipped on any sample |

## What holds

- **Numbers:** Jefferson list, 18 rows. Walk with skips only at h6 (Cedar 5) and h15 (Cedar 11). 15 seats 1, 4, 10; 16 seats 1, 4, 11. Tile lengths 9¾, 4⅓, 1½. Ranks 1 to 17 by tile end. "39,000 ÷ 2,250 = 17 1/3". Shares at 8 and 13. Under-bars and outlines at 8 (floor 0, 1, 5; ceiling 1, 2, 6). 12_10 upper 1, 4, 10 and lower 2, 3, 10.
- **Proof labels:** "6 to 7 1/2: not entered at 6", "15 to 16 1/2: not entered at 15", "Birch's seat 2 ends at 8 2/3, later than 7 1/2", "7 + 1 = 8", "Ash 0 ≤ 7 × 4/39", "Birch 1 ≤ 7 × 9/39", "Cedar 4 ≤ 7 × 26/39", "0 + 1 + 4 = 5 ≤ 7 × 39/39 = 7". The bracket under 7 chips and "8 tiles cannot fit" at f614 to f615 (216.8 to 217.0). The veil at "not entered at 13" f636 to f638 (225.5 to 226.0). The done line f648 to f662 (230.0 to 234.0).
- **Spoken proof, re-derived:** say T is missing with end e ≤ h, and t0 is the last house that handed out a tile ending after e, or 0. Every tile handed out since then, plus T, starts at or after t0 and ends by e ≤ h. That is h − t0 + 1 whole tiles in the stretch, but the rows hold at most Σ floor((h − t0) f_i) ≤ h − t0. Contradiction. "Entered at h" is the same test as k ≤ ceil(h f) (checked for all towns, h and k below 40), so the 12_7 rule equals the 12_2 rule. The film does not state that an entered unheld tile always exists; that's minor under "one way to see why".
- **Frame safety:** no clipped text on any sample.

**Seen, no change asked:**
- Rank numbers crossed in passing by the marker, f451 to f452.
- Live readouts round while moving.
- Red wait rings and captions stay until 12_8, which is true as history.
- "share is exactly ten" has a flash and "at most 10", with no drawn share.
- The 12_7 walk overruns the voice by the 2.0 s gap.
- The screen holds 22.0 to 31.5 under the credit.
- "or 0" arrives at f511 (179.0).
- 70.0 to 73.2 shows "16 seats" with the old strike and counts, matching the sentence.
- Ring edges touch seat boxes in 12_10, and 12_10 uses the left part of the frame, as the author noted.

## Candidates I raised and withdrew

All are still visible in `checks/first-pass-observations.md`.

- **"h9 Cedar 7 and h12 Cedar 9 skipped, unmarked":** false. Jefferson's next at h9 is Ash 1 (rank 9) and at h12 Birch 3 (rank 12). Only h6 and h15 skip (source 383 to 393; `arithmetic.out.txt`).
- **"Line ahead of the label in the 12_2 walk":** sequential `move_all` then `morph_all` (467 to 468) caught in transit. Not a defect.
- **"Birch 5 veil short of the edge":** the normal 0.03 inset of `tile_rect`, visible only on the open tile.
- **"+26/39 touches Cedar's edge":** measured on frame-0300, the text ends at px 168 and the row stroke starts at px 171. A near-touch, not contact (F11).
- **"6 fills vanish":** the correct count is 7 fills vanish, 6 solid remain.

## Not done

No listening, motion review, fine sync or final AV judgement. No build, render, synthesis or oracle run. No canonical, memory or Beads edits. Every fix above is an uncompiled proposal. The arithmetic script is mine and is not native evidence.
