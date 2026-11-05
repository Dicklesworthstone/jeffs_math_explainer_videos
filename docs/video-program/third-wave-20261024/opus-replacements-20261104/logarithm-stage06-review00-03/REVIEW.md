# logarithm-stage06-review00-03: selected pictures of chapters 00–03

WhiteSwan, for BlueSalmon, answering Mail 2986. Reservation 1148, this folder only. Narration stays ElevenLabs Oliver. These pictures are the scratch voice at 720p30.

**Verdict: no blocker, and I found nothing wrong on screen.** Every number I checked is right, and every mark I measured sits within 1 px of its value. Two medium faults are worth fixing. Both are about how the picture meets the words, and neither changes a cue word:

- **F1:** the ruler is finished 12.7 s before the voice asks for it.
- **F2:** the dial's digits vanish while the voice says it remembers them.

F3 is a low-medium label fix, and the rest are low. This is not a listening, fine-sync or final-4K judgement.

| Chapter | Read | Verdict |
|---|---|---|
| `00_ratio_ruler` | 72 sheets, frames 0–285 | Fix F1. The keyboard spread and the half-octave prediction are strong |
| `01_building_ruler` | 76 sheets, frames 0–303 | Accept. The cleanest of the four |
| `02_slide_rule` | 60 sheets, frames 0–237 | Accept, with F3. Both current lint rows are motivated and need no change |
| `03_folded_decade` | 47 sheets, frames 0–187 | Fix F2. The bend is the best moment in the four |

## Findings

The fixes are proposals, uncompiled and unrendered, and I staged no patch. All times are the scratch voice. A word time ("about") is a share of letters from `checks/numbers.out.txt`, not measured and not Oliver.

### F1, medium: 00_1 to 00_2, the ruler arrives before the voice asks for it

| | |
|---|---|
| **Picture** | Axis at 1.5 s (frame 6), seventh dot at 6.5 s (16), labels at 7.0 s (17) |
| **Voice** | "Here are seven notes…", then the seven frequencies. "Mark them on an ordinary ruler of frequency" comes at 19.7 s |
| **Source** | `ratio_ruler.rs:160–161`, `:179–182`, `:209`, then `say 00_2` at `:212` |

This gives 15.0 s with no visible change (7.5–22.5 s, frames 18–54), the longest hold in these chapters, at the film's opening. The voice then asks for something done 12.7 s earlier. The lower two-thirds stay empty until the keyboard, about 33 s in.

**Fix, no word changes:**
1. Show the title alone for sentence 0.
2. Bring the green labels in through sentence 1, as they are read.
3. On `say("00_2")`, bring in the axis and ticks, then the seven dots and the leader line.

The ticks and dots take 2.5 s against 3.0 s to the next hook on the scratch voice.

**Cost:** hold 00_4 binds this file's `sha256` and line 371, so map13 needs rebinding. Its interval is untouched, so `interval_sha256` should survive.

### F2, medium: 03_3 sentence 0, the digits vanish on "remembers the digits"

| | |
|---|---|
| **Picture** | Numerals and the gold 2.4 are up on frame 100 (42.5 s) and gone on frame 101 (43.0 s). Peak brightness falls from 183 to 22 at the dial's 6, and from 240 to 20 at the gold 2.4. No digit returns until "the 2.4 mark" at 50.5–51.0 s (frames 119–120) |
| **Voice** | "The dial remembers the digits, and forgets the laps." runs to about 45.9 s |
| **Source** | `folded_decade.rs:339–343`: one `clear` list holds the lap objects and the digits |

Frame 103 (44.0 s) shows a bare tilted dial with unnumbered ticks, while the sentence says the opposite. This is the chapter's main idea. I missed it in my source review (Mail 1518), and only the frames showed it.

**Fix: split the list, so the picture does what the sentence says.**

```rust
// Sketch, uncompiled
fade_out_all(stage, &[pointer, trail, count, count_word, four, six, total, ten, lap_tag], 0.6)?;
// wait until sentence 1 minus 2.0 s; labels, land and land_word stay through "remembers the digits"
fade_out_all(stage, &digits, 0.4)?; // labels, land, land_word
play!(stage; tween(p_lay, 1.0, 1.6)?);
```

The lay still ends on sentence 1, so nothing after that hook moves. On the scratch voice the digits start to fade at about 43.9 s.

**Cost:** no hold binds this file, so map13 is untouched. The test at `:544–550` names `tween(p_lay, 1.0, 2.0)` and must follow.

### F3, low-medium: 02_2, 2.5 and 3.2 are never named at their marks

- **Frame 40:** "0.398 of a full length" sits under a bar ending at an unlabelled half tick.
- **Frame 65:** the ring on 3.2 sits between the 3 tick and the 3.5 half tick, with a bare "0.505" above. `draw.rs:414` draws no tick at 3.2.
- **Voice:** "Find three point two on the moving ruler."

The position is right: the ring is at x 750.0 and D's 8 at 750.2. But the viewer has only the card at top right to connect those numbers to these marks.

**Fix:** use "2.5: 0.398 of a full length" and "3.2: 0.505" (`slide_rule.rs:158`, `:190–197`). The first should still fit its 286 px bar, by my estimate; the width is not measured. Both strings are mine (R4).

**Cost:** the test at `:515` pins the template and its 9.06 ems. The edit is inside 02_2, so that hold's `interval_sha256` changes and map13 needs rebinding.

### Low

- **F4, 00_4 and 00_5, picture about 3 s ahead of its words.** The arc, "6 semitones" and "? Hz" arrive at 56.5–57.0 s (frames 134–135), with "go up six semitones" at about 59.7 s. "×√2" arrives at 79.5–80.0 s (186–187), with "the square root of two" at about 82.5 s. Both hooks are sentence starts (`ratio_ruler.rs:346`, `:386`). **Decide on real Oliver marks.** `until_phrase` exists only in `slide_rule.rs:96` and handles first sentences only, and a new hook would need an anchor in root's phrase gate.
- **F5, "Hz" is never introduced.** The voice and axis say "vibrations per second" (`ratio_ruler.rs:154`), then "? Hz" (`:358`), "622.25 Hz" (`:393`), the hedge (`:436`) and `building_ruler.rs:273` use Hz. **Fix:** "vibrations per second (Hz)". Frame 18 has room.
- **F6, 00_6, the closing three pictures arrive left, right, middle** (99.5, 104.5, 112.0 s; frames 232, 242, 257). **Optional fix:** swap the grid and the dial, and move the red card under the middle, so arrival reads left to right. The strips are bare lines with no marks, so the caption carries them. The card says "no steady flow reaches" and the voice says "no steady motion produces". Leave that, because "steady flow" is the film's later term (`unreachable.rs:762`, cue 11_1).
- **F7, 03_4, the column dots miss the helix rows by 4–5.5 px** (frame 187: y 184, 287.5, 391, 494.5 against 188.5, 292.5, 396.5, 498.5). That alignment is the argument, so it should be exact. **Fix:** at `folded_decade.rs:422` and the fold lines at `:434`, take y from `spiral().at(k + mark)[1]`, looping over `laps`.
- **F8, 03_2, the ring and tick read as a crossed-out ring** (frames 71–100, crop `03_f075_ring24`), just as the voice says "stop on two point four's mark". **Fix:** light the tick when the ring leaves. The test at `:530–531` pins today's order.

## Earlier open questions, answered from frames

| Item | Frames | Answer |
|---|---|---|
| 1.13 "stretched" against the picture | 01: 270–293 | No contradiction. The marks stay and only the step bar grows, to 1→10 on frame 287, so "stretched" lands on the one thing that moved. **Keep.** The picture is my design, so a fresh eye should confirm |
| 1.14 no keyboard, no picture for closer matches | 01: 197–211 | Still so. The text line reads clearly (frame 200). "Closer matches" gets only 1.583 → 1.585, and the marker moves 0.29 px, which cannot be seen. **Accept for one clause.** If the owner wants more, 3^53 / 2^84 = 1.00209 is a new fact and needs the oracle |
| 2.8 C's "10" margin | 02: 137–172 | **Accept.** See the lint table |
| 3.8 unvoiced "240 / 2 laps" | 03: from 132 | **Keep.** 2.4 × 10² is right, and 03_4's column needs the fourth row |
| 3.11 busy end state | 03: 180–187 | I count 13 texts and two pictures, with no overlap. **Keep.** The rows joining the pictures are the point (F7) |
| 3.12 pointer pop | 03: 46–48 | Still there: pointer and "0" are full on frame 47 while their label is faint. **Cosmetic, keep** |
| 4.7 hollow arrows | 02: 60; 03: 20 | **Gone.** Both are solid, because `draw.rs:176` now uses `arrow_parts` |
| R3 nudge | 02: 12–15 | Lands at 4.5 s, against "one that slides" at about 4.6 s. My own repair |
| 4.18 | | Chapter 04, out of scope |

**Canonical 01_5 says "true position" while the picture shows ≈ 1.585** (log₂ 3 = 1.58496). As I read Mail 2872, root's staged rounding wording agrees with the picture, so no picture change follows.

## Lint against the pictures

| Row | Source | Now | Judgement |
|---|---|---|---|
| 02_4 @54.8 "10" 0.24 from the edge | current | Whole on frames 138 and 172. Ink ends 18 px (0.20 units) from the edge | **Motivated.** C's 1 on D's 4 forces it. The shortfall against 0.25 is under 5 px. No change |
| 02_5 @68.4 group outside the frame | current | C hangs off the left on purpose. Its visible range is about 1.82–10, and C's 2 is whole (ink from x 26) | **Motivated.** The voice asks for this, a real rule overhangs the same way, and no spoken value is hidden. No change |
| 02 @30.7 hairline over "8" | older | Stops 7 px above the printed 8 (frame 72) | Cleared |
| 02 @56.8 line over "6" | older | Starts 63 px below C's 6 (frame 143) | Cleared |
| 02 @70.8 hairline over "6" | older | Starts 9 px below C's 6 (frame 190) | Cleared |
| 01 @75.6 "19 doublings" tight | older | Ends 69 px from the edge (frame 200) | Cleared |
| 03 @50.2 "one mark on the dial" | older | String reworded and moved. Clean on frame 120 | Cleared |
| 03 @60.4 "a spiral, and its shadow" | older | 24 px below the dial (frame 144) | Cleared |

Chapters 00, 01 and 03 have empty current `layout.tsv` files.

## What teaches well, keep it

- **00:** the half and eighth brackets land on their sentences. The same seven dots walk down and spread over the keys (about 34–36.5 s). The strike and the 622 mark arrive together at 85.0 s, and the 12 px between red and gold shows the size of the error (frame 206).
- **01:** the 19 and 12 rows meet within about 1 px (frame 180), and the marker waits at 1.5 until they justify a better place. The answer stays hidden until 76 s.
- **02:** the 0.505 bar drops to abut 0.398 (frames 100–103), which shows the lengths adding. In 02_4 the dashed extension and "off the end" make the failure plain. In 02_5 the 2.4 ring, the bracket and "one mark" answer the question directly (frame 225).
- **03:** the bend is a real morph, not a swap (13.0–15.5 s, frames 29–34). The counter flips from 0 to 1 as the pointer crosses the top. The readings arrive on their sentences (frames 136, 187).

**Think time.** Answers stay hidden through all three bars:

| Cue | Bar | Answer appears |
|---|---|---|
| 00_4 | 70.0–73.5 s, frames 164–171 | 84.5 s |
| 01_3 | 52.5–56.0 s, frames 123–130 | 76.0 s |
| 02_2 | 34.0–37.5 s, frames 85–92 | about 38.2 s |

In 02_2 the hairline already points at the printed 8. That is the question itself ("which number sits directly below it?"), not an early answer.

**Seen, no change:**
- The dim 880, 1760 and 3520 labels are doubled by the green ones.
- 03_1's "one full ruler length" plays over a three-decade ruler; the caption "one decade along" rescues it, and it is script level.
- In 03_2 the pointer rests for 3 s after "×6" until "pass the top".
- `ratio_ruler.rs:8–9`, `building_ruler.rs:9`, `slide_rule.rs:13` and `folded_decade.rs:14` still say "unrendered, unlistened" in their headers.
- Glyph review F5's fallback faces show on frame 200's ½ and on × and ≈, and are legible at 720p.

**For the final-4K pass, not judged here:** the small grey lines on 00 frame 206, 01 frame 225, 02 frame 225 and 03 frame 100. Chapters 00 and 01 end on a full held picture with no fade and the next chapter opens blank; the master's join is not in this evidence.

## What was run

All in `checks/`, each with a saved `.out.txt`. All three report `RESULT PASS`.

| Check | Result | What it cannot show |
|---|---|---|
| `receipt_and_inventory.py` | Both receipts terminal with exit 0 and inputs unchanged. 255 sheets, 1016 selected pictures, widest gap 0.500 s | What the pictures show |
| `viewing_proof.py`, from this session's transcript | 255 of 255 sheets came back as images, all byte-identical to disk. Writes `coverage.tsv` | Delivery only. What I noticed is my claim |
| `numbers.py` | 24 displayed strings recomputed. 12 mark groups measured on frames, worst 0.79 px; dial ticks worst 0.17°. Clearances above, sentence and phrase shares, static holds. 5 of 5 planted faults caught | Strings are what I read by eye. Motion between samples. Any Oliver time |

**Pictures read by eye:** all 255 sheets. Every row of `coverage.tsv` names a sheet, frame or crop, with indices and times. I also opened full frames and enlarged crops wherever a sheet was small or unclear; both are listed there.

**Bindings:** every path is relative to `narration_lab/third-wave-20261024/root/five-film-release-20261027/`.

| Item | Value |
|---|---|
| `logarithm-stage06-preview01` | exit 0, finished 22:54:33Z, inputs `23aaf2130ea9776a` unchanged |
| `logarithm-stage06-frames01` | exit 0, finished 23:03:47Z, inputs `773778f91bf2b4a8` unchanged |
| `script.tsv` | `f201d2dc57fa9cca` |
| `ratio_ruler.rs` | `92cf028b124f8a37` |
| `building_ruler.rs` | `ae125e5ec9ec9bd6` |
| `slide_rule.rs` | `eea4d01d96fd4853` |
| `folded_decade.rs` | `acfc7f9c198b3d3d` |
| `draw.rs` | `2a4e84504bab6364` |
| Video hashes | Per chapter in `checks/receipt_and_inventory.out.txt` |

## Limits and independence

- **I am not independent of these chapters.**
  - `slide_rule.rs` is byte-identical to my `logarithm-repair01-04-followup02` candidate, so R3 and R4 are my own code, and F3 criticises my own R4.
  - The "no squeeze" step bar in 01_6 is my design, and `building_ruler.rs` is root's fact binding on top of it.
  - `folded_decade.rs` is LavenderPike's candidate. I source-reviewed it and `building_ruler.rs` in Mail 1518, and missed F2 there.
  - I approved root's `predict` mark at source.
  - Chapter 00 is the only one I had not reviewed before.
- **My reading was not blind.** I knew the frozen findings and the lint rows before reading the sheets.
- **Scratch voice, 720p30, sampled frames.** Samples are at most 0.5 s apart, plus cue boundaries and lint anchors. Nothing here proves continuous motion.
- **Oliver:** no time here is Oliver's. Hook timings are the kit's character share by construction, so "lands on its sentence" is only as good as that share. F1, F2 and F4 rest on the order of events, which holds under any voice, but their sizes in seconds will change.
- **Not run:** no Cargo, render, synthesis, ASR, listening, fine sync or final-4K review, and no canonical or Beads edit. The proposed fixes have never been compiled.
- **Other chapters:** 04–12 are out of scope, and other agents and root cover them.
