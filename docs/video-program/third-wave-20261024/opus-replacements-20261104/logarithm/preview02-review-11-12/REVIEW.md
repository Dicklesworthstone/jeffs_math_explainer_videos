# Logarithm preview02, chapters 11 and 12: picture review

RubyRaven, for BlueSalmon, Mail 1389. **This is an author review of my own repairs, not independent acceptance.**

I opened all 140 sheets of `11_unreachable` and all 91 sheets of `12_what_it_forgets` myself. The evidence is half-second samples, cue boundaries and lint anchors, plus 64 frames I decoded from the preview MP4 at four moments. I did not listen to anything, and nothing here is a claim about continuous motion. Every time is on the scratch voice. The narration is ElevenLabs Oliver, so no timing here qualifies final timing.

No canonical file was edited. No build, render, synthesis, Beads change or commit was made. Code below is a proposal and has not been compiled.

## Result

All ten original chapter 11 lint flags are gone, and the preview02 layout reports for both chapters are 0 bytes. Chapter 12's F1 to F7 all hold in this evidence, and both think bars reveal nothing early.

The review found fourteen items, none of them a wrong number. R1, R2 and R6 are the most important kind: the voice or a caption says something the picture doesn't show.

| ID | Cue | Defect | Frames (sheet) | Scratch time | Advice |
|---|---|---|---|---|---|
| R1 | 12_6 s1 | 1½-lap arrival lasts one frame; the voice says "also ends here" over the next run | decoded 110.833 and 110.867; f254, f255 (063) | 110.83; voice 111.33–112.40 | Fix |
| R2 | 12_2 s0 | "all land on one mark", but no mark exists where the pointer lands | f41 on (010) | 16.0 on | Fix |
| R3 | 11_3, 11_4 | Gold F hides the middle of the ×2 arrow | f168–f221 (042–055) | 76.5–100.0 | Fix |
| R4 | 11_5 s1 | Thick red gap appears 8 s before the sentence about the gap | f285–f301 (071–075) | 129.0–137.0 | Fix |
| R5 | 11_7 s4, s5 | 15.0 s and 10.5 s stills; plane unchanged for 42 s | f450–f480, f481–f502; f424–f512 | 206.5–232.5 | Root's call |
| R6 | 11_1 s4 | Voice says the whole plane goes flat; only an F-tall red stroke does | f43 on (010) | 20.0–27.5 | Worth fixing |
| R7 | 11_1 s2 | The F the voice names has 1.74:1 contrast | f20–f27 (005–006) | 8.5–12.0 | Worth fixing |
| R8 | 11_3 | Hollow arrow heads, grid lines through them | f161 on (040 on) | 73 on | Worth fixing |
| R9 | 12_6 s3 | Three reds crowd the end ring; tip dot touches it | decoded 126.667–126.867; f286–f296 | 124–131.5 | Root's call |
| R10 | 12_6 | Hard pose jumps at each reset | decoded 110.867; f269, f281, f297 | 110.87, 118, 124, 132 | Note |
| R11 | 11_6 | F's bar lies on the gold x axis, gold on gold | f361–f367 (090–091) | 164–167 | Note |
| R12 | 11_2, 11_5, 11_6 | Picture leads the words | see below | see below | Recheck on Oliver |
| R13 | 12_2 | Tight stack above the sentence | f62 (015) | 26.5 | Note |
| R14 | both | Smaller items and known leftovers | see below | | Note |

## Findings

### R1. 12_6 s1: the 1½-lap arrival isn't held

**Seen.** On the decoded frames, the readouts step .96, .97, .98, .99 and reach time 1.00, laps 1.50 at 110.833. At 110.867 they read 0.00 and 0.00. The arrival pose (purple and teal faces, dot in the end ring) lasts one frame at this preview's 30 fps, then the cube is back in its start pose (green and blue faces). The half-second samples miss it entirely: f254 110.5 reads .89 and 1.33, and f255 111.0 already reads "2½ laps".

**Why it matters.** The sentence's claim is that 1½ laps "also ends here". By character share, that phrase falls at about 111.33–112.40, when the 2½ run is already under way, so the picture never shows it.

**Source.** `what_it_forgets.rs` L875–882: the next iteration zeroes `pz.time` at once.

**Proposed fix.** Hold the first arrival to the end of that phrase, using the chapter's own `until_share` (L501):

```rust
const LAPS_ARRIVAL: f64 = 0.474; // share of 12_6 s1 at the end of "also ends here", by characters

let laps_began = stage.scene().time().to_f64(); // right after until_sentence(stage, "12_6", 1)
for (r, hold_to) in [(lap_and_half, Some(LAPS_ARRIVAL)), (two_and_half, None)] {
    // ... as now ...
    play!(stage; steady(pz.time, 1.0, 2.4 + 0.4 * r)?);
    if let Some(share) = hold_to {
        until_share(&kit, stage, "12_6", 1, laps_began, share)?;
    }
}
```

On scratch, the hold runs 110.83–112.40, the 2½ run ends about 116.10, and the chip settles about 116.5, inside s1's end at 117.78. Recheck on Oliver that the 2½ run and chip still fit inside s1, because `until_sentence` for s2 does nothing once its moment has passed.

**Evidence.** `crops/12_decoded_110p700_readouts.png`, `crops/12_decoded_110p800_cube.png`.

### R2. 12_2: "all land on one mark" has no mark

**Seen.** Ticks exist only at the digits 1 to 9. The pointer lands at log10(2.4), between "2" and "3", on bare rim, while the caption says "all land on one mark".

**Source.** `what_it_forgets.rs` L615–625 (ticks), L626–627 (pointer at `dial.mark`), L630 (caption).

**Proposed fix.** A gold tick wouldn't help, because it would be collinear with the pointer. Put a gold dot on the rim and a gold label outside it, and add both to the L634 fade:

```rust
let mark_dot = stage.arena_mut().add(
    Dot::new().point(dial_at(mark, DIAL_R)).radius(0.05).style(flat(gold, 1.0)).build(),
);
let mark_word = kit.text(stage, &format!("{:.1}", 10f64.powf(mark)), 14.0, gold, dial_at(mark, DIAL_R + 0.2))?;
```

Computed, not rendered: the label centres at (3.09, 0.61), clear of "2" at (3.39, 1.09), "3" at (2.46, 0.31) and the caption that starts at x 3.6. No narration changes.

**Evidence.** `crops/12_f041_dial.png`.

### R3. 11_3 and 11_4: gold F over the ×2 arrow

**Seen.** On the arrow's row (frame y 387, x 190–315), gold covers 0 of 126 px up to f167 76.0 and 75–76 px from f168 76.5. It stays that way until the frame before f222 100.5. Only the arrow's two ends show, across the end of 11_3 and the first 6 s of 11_4.

**Source.** `unreachable.rs` L536–541: `after` is built and faded last, so it paints over `e1`.

**Proposed fix.** After L541:

```rust
stage.bring_to_front(e1)?; // fmn-mobject stage.rs L941 at pin a399d626: re-adding promotes
```

`Stage<'_>` derefs to that type (`fmn/src/lib.rs` L1512). The cost is that the arrow's middle returns in one step at the end of the 0.5 s fade.

**Evidence.** `crops/11_f170_arrows.png`.

### R4. 11_5 s1: the red gap comes too early

**Seen.** There's no red at f284 128.5. At f285 129.0 the routes and thick red bar appear together. The caption "gap between the ends: 1.17" arrives at f302 137.5, with s2 at 137.06, and "end in different places" falls at about 140.86–142.57.

**Why it matters.** The bar shows the answer for 8 s while the voice is still setting up the two routes.

**Source.** `unreachable.rs` L206–212 draws the bar whenever the ends differ; L647 adds it all at s1; L651–666 is s2.

**Proposed fix.** Give `routes` a `show_gap: bool`, so the bar is drawn only when the ends differ and it's true, and the gold meet dot is unchanged. L589 passes `true`. L645 passes `value(s, gap_on) > 0.5`, from a tracker that starts at 0 and is set to 1 just before the L666 fade. The bar then pops on as the caption fades in.

**Evidence.** `crops/11_f285_routes.png`.

### R5. 11_7 s4 and s5: the long stills

**Seen.** Measured still spans are f450–f480 (206.5–221.5, 15.00 s) and f481–f502 (222.0–232.5, 10.50 s). The plane region has no pixel change from f424 193.5 to f512 235.83, about 42 s. Through s5, including "blocks for each negative stretch come in matching pairs" (about 228.66–232.46), the only new thing on screen is the citation at f481.

**Why it matters.** This is the chapter's largest teaching gap. Both sentences state a rule, and neither has a picture.

**Option, root's call.** These are new picture beats with no new words, using only matrices already on screen:

- **s4:** on the existing plane, a white direction sweeps round while its gold image under −I stays exactly opposite.
- **s5:** in the free area between the plane and the column (about px x 530–820), two rows of blocks: −1, −1 "a matching pair", and −2, −½ "unpaired".

This isn't a small repair. It needs its own staging, layout check and fine sync.

**Evidence.** `frame-0495.png`, and the spans in `MEASUREMENTS.md`.

### R6. 11_1 s4: "the whole plane"

**Seen.** The voice says "squashes the whole plane flat onto a line" (about 24.51–27.45) and the caption says the same. The red stroke is only as tall as the F, and the grid stays fully two-dimensional.

**Source.** `unreachable.rs` L305–309.

**Proposed fix.** Add a thin red line along the vertical axis for the grid's full height under `flat_f`, and add it to the L327 fade.

**Evidence.** `crops/11_f043_red.png`.

### R7. 11_1 s2: the faint "before" F

**Seen.** The F's fill measures [53, 61, 75] against the field [12, 17, 20], or 1.74:1. That's the darkest thing on screen when the voice reaches "the letter F" (about 10.15–11.01). The "before" word is 6.08:1 and the gold F is 12.73:1.

**Source.** `unreachable.rs` L281, and the same style for 11_3 at L500.

**Proposed fix.** `flat(Palette::dim(), 0.65)` at both places. Blending predicts 3.25:1; re-measure on the next frames.

**Evidence.** `crops/11_f024_before.png`.

### R8. 11_3: hollow arrow heads

**Seen.** Both arrows have outlined heads with grid lines running through them, while the routes in the same chapter have filled heads. I left this for root in my chapter 11 repair note, and it's still open.

**Source.** `draw.rs` L175–178 against L182–202.

**Proposed fix.** At `unreachable.rs` L516 and L527, use `arrow_parts(stage.arena_mut(), o, tip, colour, 4.0, 0.2)`. These are the file's only two `arrow` calls, so remove `arrow` from the `use` list or Clippy will fail. Don't change `draw::arrow` itself: it has 14 other call sites in six other chapters.

**Evidence.** `crops/11_f170_arrows.png`, `crops/11_f285_routes.png`.

### R9. 12_6 s3: reds crowd the end ring

**Seen.** My predicted crowding is real. The red line grazes the end ring's rim through the red spin. On decoded frames 126.667, 126.767 and 126.867, the pin's tip dot touches the ring from outside while the pin's shaft crosses the ring. At f293 the pin's arc crosses the red line beside it.

**Assessment.** The geometry is honest, and the corner dot's miss still reads because the dot is far from both rings at f293. It's busy, not wrong.

**Option.** End ring radius from 0.2 to about 0.15 at `what_it_forgets.rs` L436, then look again. The start ring is 0.13, so the "small start, big end" difference nearly goes. I'd leave it unless root finds it confusing at final resolution.

**Evidence.** `crops/12_decoded_126p667_ring.png`, `crops/12_f287_red.png`, `crops/12_f293_red.png`.

### R10. 12_6: hard resets

The pose cuts back at 110.867 (decoded), and between samples at f269 118.0, f281 124.0 and f297 132.0 (L933). The word swap is the only transition. R1's hold makes the first one less abrupt. I'd accept the others.

### R11. 11_6: gold on gold

From f361 to f367 the F's middle bar lies on the gold x axis (`crops/11_f365_axis.png`). Axes in another colour would separate them. It's still readable.

### R12. Picture leads the words (recheck on Oliver)

All phrase times below are character-share estimates inside a scratch sentence. Don't retime these until Oliver's word times exist.

| Cue | Picture | Words |
|---|---|---|
| 11_2 s3 | Backward run 38–41, text 41.5 | "run backwards" about 41.97–42.93 |
| 11_5 s1 | Routes at 129.0 | "send it along both routes" about 131.92–133.78 |
| 11_6 s3 | Two-stage journey 176–180.09 | "running one after the other" about 180.86–182.91 |

### R13. 12_2: tight stack

At f62, "−3πi" sits 10 px (0.11 units) above the sentence, and the axis stub ends 5 px (0.056 units) above it. It's clear but tight, and the lint passes it. `crops/12_f062_lower_left.png`.

### R14. Smaller items and known leftovers

Chapter 11:

- **Near the limit:** "same end point" at f230 ends at x 6.856 against the 6.86 limit, and it's far from its gold dot, linked only by colour.
- **Mixed notation:** caret text "e^X" and "e^(tp)" sits beside typeset e^X (f239 on, f357).
- **Notch look:** dash segments over the gold F's edge read as notches (`crops/11_f033_dashes.png`). The polygon itself isn't jagged.
- **Alignment:** the "suppose" tex is centred, not aligned to its column.
- **Empty plane:** the "u" plate stays over an empty plane, 127–129.
- **Dim purple:** the e^iπ line measures about 4.0:1 in thin type; readable on the crop (`crops/11_f524_purple.png`).

Chapter 12:

- **Family limit:** nothing on the turning line says the family continues past ±3π (f319 on).
- **Movers:** they pop on rather than fade, at L724.
- **Small labels:** the 16 pt purple "−3πi" measures 4.31:1.
- **Long hold:** still from 155.5 to the end at 163.016, under the last sentence.

Known and still open:

- **11_7, 11_8:** plane empty 188.0–190.5; title only about 236.33–239.3.
- **Title swaps:** headings overlap during the crossfade at f516 and at 12_4's f120–f121.
- **Headers:** the "uncompiled, unrendered, unlistened" comments are stale in both chapter files (`unreachable.rs` L41–42, `what_it_forgets.rs` L33–34).
- **From my chapter 12 "left alone" list:** "Fold the ruler" has no fold, 12_1 has no keyboard, "9 10" is crowded, the cut crosses the −1 target, and f111 shows ghosts.

## Original lint on chapter 11

Preview01 had these ten flags; preview02's report is 0 bytes. The 29.8 and 30.0 flags share a frame, as do the two at 233.1.

| Flag time | Original flag | Frame (sheet) | Measured now |
|---|---|---|---|
| 29.8 | Crossed: zero line through "0" | f69 (017) | "0" ends px 683, line starts px 697: 14 px clear |
| 30.0 | Overlap: "signed area" word and value | f69 (017) | Word y 106–124, value y 140–155: 16 px clear |
| 60.8 | Overlap: value and "det" word | f134–f137 (033–034) | At f137, word y 106–123, value y 140–155 |
| 142.8 | Edge: gap caption to x 7.08 | f313 (078) | Three lines, right edge x 6.000 |
| 168.2 | Behind: axis under "where A puts the F" | f369, f375 (092–093) | In the column: key ends px 899, word starts px 914 |
| 176.2 | Crossed: grid over the same label | f385, f386 (096) | Same: word starts px 914 at f386 |
| 197.4 | Outside: "equal stretches" to x 7.23 | f432 (108) | Right edge x 5.756 |
| 206.2 | Edge: "2×2" to x 7.04 | f449, f450 (112) | Right edge x 6.022 |
| 233.1 | Outside: rule to x 7.13 | f503 (125) | Four lines, right edge x 6.022 |
| 233.1 | Outside: rule to x 7.50 | f503 (125) | Same |

## Repairs that held

### Chapter 11 labels found by looking

| Repair | Evidence |
|---|---|
| "before" and "after" centred in grid cells, no grid line touching | `crops/11_f024_before.png`, `crops/11_f033_dashes.png` |
| "×2, flipped" and "×½, flipped" on plates that cut the grid line cleanly | `crops/11_f170_arrows.png` |
| "u = (1, 1)" on its plate | `crops/11_f285_routes.png` |
| "det < 0" back in the column, clear of the readout | f145 |
| Target F key in the column, a readable match for the outline | `crops/11_f375_key.png` |

11_5's off-diagonal run also reads correctly: the gap falls from 1.16 to 0.00 by f337 155.0, then the gold dot and "the ends meet only when b = c = 0". The two-stage journey lands in the red outline at f393.

### Chapter 12 F1 to F7

| Fix | Result | Evidence |
|---|---|---|
| F1 red corner, pin, line | Holds. Red corner dot stays out of the end ring in samples. At f293 the cube outline lands on itself with green on top, correct for a half turn about (−1, 1, 0). Gold top-centre dot leaves the rod. | f281–f296 |
| F2 Log line inside border | Holds. Text ends px 594, border starts px 607: 13 px clear. | f62 |
| F3 flows share a start | Holds. All three movers appear together in the ring at decoded 32.533 and separate from there; all in the ring at f86. | `crops/12_decoded_32p400_flows.png`, f86 |
| F4 linear readouts | Holds in the windows read: 12_4 s3 steps .0645 per 0.5 s, 12_6 s3 steps .09, and .01 per frame in the decoded 1½ run. | f165–f180, f282–f292, decoded |
| F5 run times from sentences | Holds. Lap 61–70.7 (s2 60.53–71.09); follow 71.5–79.5 (s3 71.09–80.20); gold line f182 80.5 (s4 80.20). Red spin reaches 1.00 by f293; computed end 129.6. | Sheets 035–045, 070–073 |
| F6 think bar | Holds; see below. | f231–f239 |
| F7 turning-line timing | Holds. Line, window, dots at f317–f319 (s5 141.75); tie f328–f329 (share .45 = 147.45); `precisely` and convention f342–f344 (s6 154.42). Chip absent f301 134.0, settled f302 134.5; voice about 134.37. | Sheets 075, 079–086 |

### Prediction timing

| Cue | Line ends | Think bar | Next cue | Revealed early? |
|---|---|---|---|---|
| 12_5 | 99.69 | Linear fit 100.30–104.30 (eight samples, f231–f238); no bar at f230, gone at f239 | 104.567 | No |
| 11_3 | 89.177 | Starts about 89.81 (from f198); gone at f206 94.0 | 94.067 | No |

## Reviewed evidence

Paths are relative to `narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-opus-rebuild-frames02/`. A set digest is the SHA-256 of `<sha256>  <file name>\n` lines in file-name order, so it equals the hash of the list file here.

| | `11_unreachable` | `12_what_it_forgets` |
|---|---|---|
| Sheets viewed | `sheet-000`–`sheet-139` (140) | `sheet-000`–`sheet-090` (91) |
| Frames | `frame-0000`–`frame-0556` (557) | `frame-0000`–`frame-0362` (363) |
| Sheet hash list | `sheets-11.sha256` | `sheets-12.sha256` |
| Sheet set digest | `0203d191bbef5d0b5df7d333ab0a491c1b64792ab424b73399b6740a771abb68` | `3525c2b4c9a25b9831d5b7ac219225eb931d84575f50e32dbb2e0026ef228d3a` |
| Frame set digest | `ac23161ce5e32279289334398b1b4d61d58a034b25168f711139d9a4bd8ae8dc` | `fe4ce9b120152de37c3e77684aab84e6b2dc91a8544552996d6ff17b79f1c0d1` |
| `sampling.json` | `8c6dd49bf9dc47df21f7fb203ee37ee9bd6a3812b3864d98375963ea96153fa2` | `c682706699ea98ca8b6ed9c0644fe560dafab140d418170752328e56aadba7dd` |
| Duration, s | 255.3 | 163.266016 |

`sheet-frame-map.tsv` gives each sheet's frames and times.

**Render** (`logarithm-opus-rebuild-preview02/render/`):

| File | SHA-256 |
|---|---|
| `11_unreachable.mp4` | `044b47fcb8f3eeffa8a70ae83187c3edb72a808aeabb1c18836da4051278467a` |
| `11_unreachable.cues.tsv` | `ed4207e693011d87ea207360aa6db889b29b35bf93dd1a62aa9eaaa1e11219c2` |
| `12_what_it_forgets.mp4` | `286bb90396c3eec827c07b72738669d2421877e5d002fe2814d535382ec58ccd` |
| `12_what_it_forgets.cues.tsv` | `aea8fbbc53a98896909c407ad065e992670e58a224101d66846f9e0b7e51a811` |

Both `.layout.tsv` files are 0 bytes.

**Receipts:** preview02 `b8542ec30825caac8edcd5cf7bf5c50345bd8162a59a9aa03a3684d8cbed2b35` and frames02 `db87e6c0ec4d5b024d06ccadaab7cd31e08f5c8a2b075a38c7efc94035f68884`, both admitted with exit 0. Their input listings re-checked same: 174 of 174 and 216 of 216.

**Source read** (`explainer_corpus/logarithm_mysteries/`):

| File | SHA-256 |
|---|---|
| `src/chapters/unreachable.rs` | `b725f3688334b821b5e69e1d0e731a95120b6956e4e330f2d61e71de9a4cecd4` |
| `src/chapters/what_it_forgets.rs` | `97996601ae9f678d42688b73dc3c9b858808b8c1c589865e3fef8fdd51507a5d` |
| `src/chapters/draw.rs` | `59ebd09d2fa5c793fbb2f9ab414a4b951325cbc68e69d15c488a949eba88f9a3` |
| `script.tsv` | `3e4e34af7090e448d65a2731f0d0d78673679730c0f95fdc695e3e8f2e6f5243` |
| `docs/storyboard.md` | `c5bc6e7e9fbd079793e597cc56ac76723a3dc5c4966053cf4f779e86ab20ddb6` |

## Not reviewed

- **Sound:** no listening, and nothing about Oliver's delivery, sync or cue fit.
- **Motion:** nothing between samples beyond the four decoded runs. A flash or jitter shorter than a sample could be missed, as R1 nearly was.
- **Final picture:** this is 1280×720, 30 fps, CRF 26. Colours, thin type and one-frame events may differ at 4K60.
- **Other chapters,** and any independent check of the mathematics. This review compares pictures with the script and source; it doesn't re-derive the facts.
- **The proposed code:** uncompiled and unrendered, and the R2 layout is computed only.

## Files here

- `REVIEW.md`: this file.
- `MEASUREMENTS.md`: every measurement with its frame, box and method, the discarded ones, and the crop index.
- `sheets-11.sha256`, `sheets-12.sha256`, `crops.sha256`: hash lists.
- `sheet-frame-map.tsv`: sheet, frames and times.
- `NOTES.md`: running notes made while viewing, not checked as findings.
- `crops/`: kept detail.
- `checks/`: the read-only scripts, run from the repo root with `python3 -I -B`.
