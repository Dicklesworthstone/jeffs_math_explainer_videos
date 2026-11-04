# Apportionment chapter 02 (`02_shares`): independent picture review 01

RubyRaven for BlueSalmon, Mail 1256, 2026-11-04.

## Verdict

The maths on screen is right. This review does not accept the chapter.

Two defects should be fixed before a narrated draft:

- **F1:** the chapter's last picture hides the words the voice is pointing at. The text measures 1.14 to 1.19:1 on gold.
- **F2:** the 14/15/16 table runs on a fixed clock. It finishes and flashes before the voice reaches it. "17" is on screen 9.75 s before "seventeen", and then the picture does not change for 14.5 s.

F3 is the main teaching gap: the division that defines a share is spoken but never drawn. F4 to F7 are smaller. All the proposed fixes are picture-only, so no cue text changes and no cue needs new synthesis. They all sit in `shares.rs`; none touches shared `draw.rs`. They are uncompiled proposals.

## Evidence

| Item | Value |
|---|---|
| Sheets read | 70 of 70: `sheet-000.png` to `sheet-069.png`, all opened as images |
| Frames carried by those sheets | 277 of 277: `frame-0000.png` to `frame-0276.png`; 4 per sheet, sheet 069 holds frame 276 alone |
| Frames also opened alone at 1280×720 | 31 and 276 |
| Other chapter opened | `01_one_more_seat/sheet-051.png` (frame 204, 85.183 s), for colour continuity only |
| Frame folder | `narration_lab/third-wave-20261024/root/five-film-release-20261027/apportionment-opus-rebuild-frames01/02_shares/` |
| Sheet set digest | `790a064acff57f95fb2af160c0f120b7af231f9766fc6144c54648c7312ae6e7` |
| Frame set digest | `d7d779b7e473bd24b6abda7fc8406d71176b1f1dce5151bab05386e92100292f` |

Each set digest is the SHA-256 of the lines `<sha256>  <file name>\n`, in file-name order.

| File | SHA-256 |
|---|---|
| `sampling.json` | `a726e3bf0c8b99180746d5c0c990e85ff765eac4e489874a70b7ccaa4f0f461b` |
| `02_shares.mp4` | `52dc436bce3c75c7fd6e22bf60688f64dc82712305777ffd439bdc1508d1092c` |
| `02_shares.cues.tsv` | `e23a30b16bcd931b718f7a4543ef5a73dd2b8d64696372d4902f01d64384d7cf` |
| `02_shares.layout.tsv` (0 bytes) | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `explainer_corpus/apportionment/script.tsv` | `cfcec06911c7d94efa6b7b29a820be9bf73a608810b4b57085f02f5b95939313` |
| `src/chapters/shares.rs` | `45e9521207b3a68c20c781bc93721cc161b395b35e70a4dbfc0f5bf1ee34ba49` |
| `src/chapters/draw.rs` | `dd385ed5f45f09ffb8c0d8563829601bb147460bc16c7d4cbbf08245a7b23ba4` |
| `src/chapters/visual.rs` | `27c2520b28079e9a3cb9329c63b7559163f1d5a4998e7e89572f6cf9ed20a032` |
| `src/chapters/mod.rs` | `050e940041bc0f3e60faa9ec65815a7da93abda4228d814cfec0a09f908b10cf` |
| `src/facts.rs` | `ec55517c9544e275a8ba3b639b07be2690e347dc564bac93849d6874f3d3faa6` |
| `src/model.rs` | `91baa9ca3f1d050de21375050f6f881401df7de7471c4fe0aa6e46f74ef2c5cf` |
| `frames01.py` | `7334372817752c85653b288eb54abbfe7ccbef48d23ada97bb772e13493dd8e6` |
| Preview input listing | `3407b58f869d39df6a45b500d69fc637e3701185f4f0a6810548d05f01d83bbc` |
| Frames input listing | `3e4e0f337b9b82907130502c95626c65a5ecc85160e1bbc0a2a863790fb8a180` |
| Preview `resource-receipt.json` | `eb4d581f84643596bfcba9498e5e305e0d9152da1ac530d19042c6c77508580e` |
| Frames `resource-receipt.json` | `15fbf08986358d80e28274d17483cd6f1de32270f6b954f6f19d347c276fe7c3` |
| `docs/storyboard.md` | `31bf5bf4fd382a4015bc28564e3aad6cddfa0ddbf025a500f12ecefb6ff98fa5` |

**Binding.** Both receipts report admitted, exit 0 and `inputs_sha256` equal to `final_inputs_sha256`:

- Preview: 07:25:20Z to 08:02:47Z.
- Frames: 08:02:52Z to 08:09:19Z.

I re-read both listings against the tree while writing this: 234 of 242 and 282 of 290 entries still match. The same 8 differ in both, and none is a chapter 02 input:

- `apportionment/src/chapters/largest_tails.rs`
- `apportionment/src/chapters/sixteenth.rs`
- `complex_analysis_visual/script.tsv`
- `logarithm_mysteries/src/chapters/draw.rs`
- `logarithm_mysteries/src/chapters/ratio_ruler.rs`
- `logarithm_mysteries/src/chapters/unreachable.rs`
- `logarithm_mysteries/src/chapters/what_it_forgets.rs`
- `target/debug/logarithm_mysteries`

Two of them are the sources of apportionment chapters 03 and 04, so this preview no longer shows those chapters' current source. I reviewed neither.

For chapter 02, the script, the storyboard and the six source files in the table are in both listings and match. `02_shares.mp4`, `.cues.tsv` and `.layout.tsv` are in the frames listing and match. So the frames show the current `shares.rs`, and the proposals below apply to it.

## Limits on these numbers

- **Voice.** The preview ran with `--scratch-voice --lint`, 1280×720, 30 fps, CRF 26, preset ultrafast. Every voice time below is a scratch estimate, not Oliver. No Oliver take was opened or heard. The apportionment ElevenLabs lane folder, searched four levels deep, has no file named for cue 02_5.
- **Stills.** `sampling.json` says: "Half-second, cue-boundary and layout anchor/persistence samples; later observations may be nonconsecutive. Not continuous-motion or listening proof." Picture times are good to about half a second, and a change between samples would not show.
- **Voice time method.** I used the kit's own rule. Sentence starts are shares of `len − 0.4` by characters (`Narrator::scratch`, `estimated_starts`), and phrases are interpolated inside the sentence (`Spoken::at`).
- **Check on that rule.** It predicts these sampled events: "Ash's share" 23.033 (band visible at 23.5), "Nobody can sit" 38.177 (flash at 38.5), "that works" 56.725 (first arrow fading in at 57.0), "the total" 93.252 (dimming at 93.5), "The paradox lives" 113.925 (fill started at 114.0).
- **Contrast.** WCAG ratio, measured on frame pixels: the region's modal colour against its most distant pixels.
- **Static spans.** Consecutive samples with fewer than 30 pixels changing by more than 36 summed RGB levels.
- **Layout lint.** The report is empty, which clears nothing here: the lint cannot see F1.

Nothing was built, rendered or synthesised for this review. No source was edited.

## Findings

| # | Severity | Where | Defect |
|---|---|---|---|
| F1 | High | 02_7, frames 267–276, sheets 066–069 | Ink text on a solid gold card, 1.14 to 1.19:1, through the chapter's last picture |
| F2 | High | 02_5, frames 170–218, sheets 042–054 | Table on a fixed clock: totals and flash precede the voice by up to 9.75 s, then 14.5 s unchanged |
| F3 | Medium-high | 02_1/02_2, frames 31, 36–55, sheets 007, 009–013 | The division is only spoken; 2,600 leaves before the sentence that needs it; empty grids for 5.5 s |
| F4 | Medium | 02_3/02_4, frames 93–133, sheets 023–033 | "Become a whole number" and "round each share" play over an unchanged picture |
| F5 | Medium-low | 02_7, frames 239–246, sheets 059–061 | Empty screen under "There are two classic families." |
| F6 | Low | 02_6, frames 222–238, sheets 055–059 | Second sentence has no picture, misses are not marked, and the table rule brightens when "the rest dims" |
| F7 | Low | 02_5, frame 200, sheet 050 | Rounded column is uncoloured and its digits drift between rows |

### F1. The named family becomes unreadable at the moment it is named

**Seen.** At "The paradox lives" (scratch 113.925) the first card fills solid gold and its text stays ink.

| Frame | Time | Text and field | Contrast |
|---|---|---|---|
| 258 | 110.0 | Ink on dark, card still hollow | 15.03:1 |
| 267 | 114.5 | Ink on gold | 1.17:1 |
| 268 | 115.0 | Yellow on gold, during the text flash | 1.14:1 |
| 270 | 116.0 | Ink on gold | 1.18:1 |
| 276 | 117.35, last sample | Ink on gold | 1.19:1 |

The other family's card stays fully readable, so the last picture shows the rejected label clearly and the chosen one barely at all.

**Why.** The sentence directs the viewer to the first family, and the only words naming it wash out at that instant. Chapter 03 opens on that family. Source: L220 `refill(... (cards[0][0], Palette::hero(), 1.0) ...)` fills the outline-only rectangle, and L221 flashes only the text.

**Proposed (uncompiled).** Flash while the field is still dark, then fill with the text turned dark. By palette values, background on hero is 13.75:1. This keeps the storyboard's "first card glows".

```rust
s.wait_for(stage, "The paradox lives")?;
flash_all(stage, &cards[0], 1.0)?;
let dark = explainer_kit::hex(explainer_kit::BACKGROUND);
refill(stage, &[(cards[0][0], Palette::hero(), 1.0), (cards[0][1], dark, 1.0)], 0.5)?;
pause(stage, 0.6)?;
```

Check on frames: how `indicate` looks on the hollow rectangle, and the text during the 0.5 s crossfade. Neither is verified.

### F2. The table finishes before the voice reads it

**Seen.** L131 discards the `say` handle and L154–185 contain no `wait_for`. Each row is a 1.6 s lagged fade plus 0.3 s, started about 1.2 s after the cue opens.

| Picture event | Frames, time | Voice (scratch) | Picture early by |
|---|---|---|---|
| "13" in red | 171–172, 72.5–73.0 | "a seat short" 80.828 | 8.3 s |
| Row 16 "2 4 11" | 178–179, 76.0–76.5 | "every share rounds up" 83.171 | about 7 s |
| "17" in red | 179, 76.5 | "seventeen" 86.249 | 9.75 s |
| Flash on 13 and 17 | 181 at 77.5; source 77.0–78.0 | Still on "…nine point three three", which ends at 77.823 | 3.8 s before "a seat short" |
| Nothing changes | 183–218, 78.5–93.0 | "Every one rounds down" 77.823 through "the total" 93.252 | 14.5 s static |

**Why.** The table answers the question the ring has just given the viewer four seconds to think about. The voice builds the answer in steps: the shares, all down, a seat short, then the same for sixteen. The picture has given all of it away within 7 s of "No.", so the voice is narrating a finished table.

**Oliver will not cure this.** Row 16 is complete at cue + 6.6 s whatever the voice does. The voice reaches "At sixteen" after 31 words, which would take 4.7 words per second to arrive in time. The scratch rate is 2.6.

**Proposed (uncompiled).** Keep the build loop, but store each row's lead cells (seats and shares), rounded cell and total instead of playing them. `totals` and `others` stay as they are. Row 15 arrives with the header: it is the case just shown and the voice does not read it.

```rust
let s = say(&kit, stage, "02_5")?;
fade_out_all(stage, &[q], 0.2)?;
explainer_kit::fade_all(stage, 0.5)?;
// ... build head and rows, unplayed ...
fade_in_all(stage, &head, 0.5)?;
fade_in_all(stage, &row15_all, 0.5)?;
s.wait_for(stage, "At fourteen")?;
fade_in_lagged(stage, &row14_lead, 1.6, 0.25)?;
s.wait_for(stage, "Every one rounds down")?;
fade_in_all(stage, &[row14_rounded], 0.5)?;
s.wait_for(stage, "a seat short")?;
fade_in_all(stage, &[row14_total], 0.4)?;
flash_all(stage, &[row14_total], 0.8)?;
s.wait_for(stage, "At sixteen")?;
fade_in_lagged(stage, &row16_lead, 1.2, 0.25)?;
s.wait_for(stage, "every share rounds up")?;
fade_in_all(stage, &[row16_rounded], 0.5)?;
s.wait_for(stage, "seventeen")?;
fade_in_all(stage, &[row16_total], 0.4)?;
flash_all(stage, &[row16_total], 0.8)?;
```

Under scratch times:

- "At fourteen" (70.319) has already passed when the row 15 fade ends at 71.8, so that `wait_for` falls through and row 14's shares appear from 71.8 to 73.4, while the voice reads them.
- "13" lands at 81.23 and "17" at 86.65.
- The last flash ends at 87.45, before the bound 02_6 start of 87.667.

All six phrases exist in the bound 02_5 text. Oliver will move these times. `wait_until` does nothing when the phrase has passed (`draw.rs` L985–992), but re-check that the last flash does not delay 02_6.

### F3. The division that defines a share is never drawn

**Seen.**

- No "÷" appears on any of the 277 frames, and `shares.rs` contains none.
- The ticks in 02_1 carry no "15". The picture shows "39,000 people" and "2,600 people = 1 seat" with nothing relating them (frame 31).
- "2,600" is last seen whole at 16.0 (frame 38), is fading at 16.5 (frame 39), and the frame is blank at 16.833 (frame 40). The label at L83–89 is the only `per_seat` text, so the number never returns in this chapter. The voice says "divided by two thousand six hundred" at 19.727.
- Frames 44–55 (17.5–23.0) show names, populations and empty grids, unchanged for 5.5 s, under the whole sentence that defines the share.
- Frame 31 already holds the answer, and the wipe discards it. Measured on the line: 16 ticks, 70.2 px a piece, Ash's colour ends at 1.538 pieces, Birch's ends at tick 5, and Cedar runs the remaining 10.
- The storyboard's 02_2 says "The long bar splits back into the three rows". L93 wipes instead.

**Why.** This is the definition the whole film rests on: "share" is population over 2,600. A viewer who misses the spoken clause sees 1.54 arrive with no visible source.

**Proposed (uncompiled).**

(a) Preferred. Replace the L93 `wipe` with `kit.hold`, so the line stays through the first sentence. At "population divided" add `share = people ÷ 2,600` at 26 pt, about (−2.0, 2.75). About 1.0 s before "Ash's share" (`wait_until(stage, s.at("Ash's share")? - 1.0)`; `wait_until` is in `draw.rs`, needs importing), fade the line, ticks, brace and labels out, and the names and grids in. Keep the formula up. "÷" is in the crate's glyph list (`mod.rs:30`) and is already used by other chapters. "People" matches the row labels.

(b) Smaller fallback, if the wipe must stay. Add the same formula over the empty grids at "population divided". This cures the static grids but loses the pieces picture.

(c) With either. At "Fifteen seats" add `÷ 15 seats` beside "39,000 people", left aligned at about x = 1.05, 30 pt, so the top line reads as the division and the brace as its result. Check the fit with the 0.01-per-em-per-point width rule before rendering.

### F4. The task is stated over an unchanged picture

**Seen.**

- 02_3: frames 93–109, 39.5–47.5, 8.0 s unchanged. "Each share has" starts at 41.848. The sum waits for "add up to exactly" at 47.933 (L112).
- 02_4: frames 111–133, 48.5–56.5, 8.0 s unchanged. "round each share" starts at 52.360. The arrows wait for "that works" at 56.725 (L123), 4.4 s later.

Between 39.5 and 56.5 the only change is the "? + ? + ? = 15" line.

**Why.** Rounding is the action under test in this chapter, and it is first shown after the voice has moved on from describing it.

**Proposed (uncompiled).**

- At "Each share has", show `→ ?` in town colour at x = 2.45 on each row, where the arrows already sit.
- At "round each share", morph those to `→ 2`, `→ 3`, `→ 10`.
- Give the green sum morph (L125–127) its own `wait_for("because two plus")` at 57.557. It starts at 57.925 today only because it follows the arrows, and it would otherwise move earlier with them.

No question depends on the rounded values, so showing them earlier leaks nothing.

### F5. Empty screen under "two classic families"

**Seen.** Frames 239–246 (101.967–104.0) are blank. The sentence runs 101.967 to 104.135, and the first card starts fading at "One keeps". This is the only spoken sentence in the chapter with an empty frame throughout.

**Proposed (uncompiled).** At the cue's start, fade both empty outlines in (`fade_in_all(stage, &[cards[0][0], cards[1][0]], 0.5)`), so "two" is visible as two. Then fade each label in at its existing phrase.

### F6. "A second idea" has no picture, and the rule brightens

**Seen.**

- Frames 222–238 (95.0–101.5) are unchanged under "So every rule needs a second idea, a way to settle the seats that rounding gets wrong".
- Seeing that a total is wrong means comparing the seats column with the total column, 9.8 scene units apart.
- The table rule goes from `[53,61,73]` on frame 200 (1.73:1) to `[87,90,101]` on frame 232 (2.76:1). By source, the rest of `others` is ink, town-coloured or dim text at full opacity going to dim at 0.35, so the rule is the one element that gets brighter when "the rest dims". Cause: L150 pushes the rule into `head`, L153 clones `head` into `others`, and L190 recolours `others` to `dim`.

**Proposed (uncompiled).**

- Leave the rule out of `others`.
- At "a way to settle", add `one short` beside 13 and `one over` beside 17, red, 20 pt, left aligned at about x = 5.15, and not dimmed. Derive both from `sum` and `h`, not from literals.

### F7. Rounded column

**Seen.** On frame 200 (86.938 s), "1 3 9" and "2 3 10" and "2 4 11" are ink, in one centred string (L168). Glyph runs at 1280 px:

| Digit | Row 14 | Row 15 | Row 16 |
|---|---|---|---|
| First | 812–821 | 805–815 | 806–816 |
| Second | 841–851 | 835–845 | 835–846 |

That is a drift of up to 7 px. The shares are coloured by town and their rounded values are not.

**Why.** "Every one rounds down" asks the viewer to match 1.44 to 1, 3.23 to 3 and 9.33 to 9 across a colour break.

**Proposed (uncompiled).** Use three cells at `cols[4]` −0.62, 0 and +0.62, in `town_color(i)`. That gives alignment and town identity from one change, and the three become F2's `rounded` group.

## What held up

- **Arithmetic, recomputed with exact fractions:**
  - 39,000 ÷ 15 = 2,600.
  - 14 seats: 1.4359, 3.2308, 9.3333 give 1 + 3 + 9 = 13. All three move down.
  - 15 seats: 1.5385, 3.4615, 10 give 2 + 3 + 10 = 15.
  - 16 seats: 1.6410, 3.6923, 10.6667 give 2 + 4 + 11 = 17. All three move up.
  - "Every one rounds down" and "every share rounds up" are literally true.
  - The displayed two-decimal shares also add to 14.00, 15.00 and 16.00.
- **Bars match numbers.** Share bands on frame 76 end at px 293, 366 and 613, against 293.2, 365.8 and 613.0 expected. Ash's tail is visibly longer than Birch's, as 0.54 and 0.46 require. Cedar has no visible tail, matching "exactly ten".
- **Line geometry.** 16 ticks make 15 equal pieces, and Cedar starts exactly on tick 5, because 13,000 ÷ 2,600 = 5.
- **Think ring.** The voice ends at 65.013. The ring is up by frame 155 (66.0) and has drained by frame 163 (70.0), and the next voice is at 70.100. Nothing on frames 155–163 reveals the answer: the only total visible is the correct 15, and the table does not start until frame 168 (71.0).
- **On-phrase beats under scratch times:** bands per town, the flash on Ash's tail at "Nobody can sit", the sum at "add up to exactly", the total-column highlight at "the total", and the two cards at their phrases.
- **Persistence.** Names and grids stay from 17.0 to the fade at 70.5. "share = quota" stays from 31.0 to the same fade. The sum stays through the ring. The table stays from 71 s to the wipe. 2,600 (F3) is the only item that leaves before a sentence needs it.
- **Colour continuity.** Chapter 01 frame 204 (85.183 s) names the towns in the same blue, gold and green.
- **Layout.** I saw no clipping or overlap on any of the 277 frames, and the widest text stays inside the 6.86 limit (the "pause for more time" right edge is about x = 6.81).
- **Storyboard.** 02_1, 02_3, 02_4, 02_5, 02_6 and 02_7 are drawn as described, subject to the timing and legibility findings above. 02_2 is not (F3).

## Lower-priority notes, not findings

- **Unnamed bars.** Frames 3–13 show three coloured bars with no names or populations. Colour continuity carries them, so I do not call this a defect. There is room for the `name()` helper at x −6.5 to −4.5 if root wants "4,000 + 9,000 + 26,000" visible before "39,000".
- **Green has two jobs.** Cedar and the "good" total both use `Palette::green` (`draw.rs` L41, `shares.rs` L175), so "15" and Cedar's "10.00" share a colour. This is a film-wide choice, flagged only.
- **"share = quota".** It reads well (6.13:1) but floats about 3.4 units from the readouts it names. Over the readout column (about x = 1.6, y = 2.35) it would double as the column head.
- **Birch's band.** The whole/tail boundary is 1.43:1 (Ash's 1.46:1), carried mostly by hue. It is visible on every sheet at this size. I did no colour-vision check.
- **Brace label.** "2,600 people = 1 seat" spans px 137–341 under a brace at px 113–184. Left aligning it at `LINE_X0` would tie it to the brace. Cosmetic.
- **Bands fade in.** Frame 61 (26.0) shows Birch's whole band faint, where the storyboard says "filled to its share". Whether a growing fill is wanted is a motion question I cannot judge from stills.
- **Dimmed table.** 2.76 to 2.79:1 on frame 232. It reads as deliberately receded and is still legible on the frame, and the totals stay at 6.28:1. I would leave it.

## Not reviewed

- Listening, or any Oliver take or Oliver timing.
- Fine sync: every voice time here is a scratch estimate.
- Continuous motion, including the slide, the morph, the ring's smoothness and the wipes.
- Independent AV review, the score, captions, and the 4K render.
- Chapter 03's opening, so the join out of F1's card is unchecked.
- Whether any proposal compiles or looks right: none was built or rendered.
