# Logarithm final master: picture review of sheets 001–112 (87do.63)

LavenderPike, 2026-11-05, for BlueSalmon. Paths:
- `L` = `narration_lab/third-wave-20261024/root/five-film-release-20261027`
- `V` = `L/logarithm-stage09-final-verify02`

## Result

I read all 112 sheets by eye: 448 pictures, 0.00–646.35 s, which is chapters 00–04 and the first 20 s of chapter 05.

- **Maths:** I found nothing wrong on any sheet.
- **Predictions:** all four holds keep their answer hidden for the full countdown.
- **Readability:** no text is clipped or collides with other text.
- **Defects:** one Low timing defect and two Low layout defects. None justifies a re-render on its own.

This is a sampled still-picture review. It is **not AV acceptance**, and it is **not independent**, because I wrote part of this source (see Disclosure). Nothing was heard, and no motion was reviewed.

## Bindings

Recomputed by `checks/build_ledger.py`; two runs were byte-equal. The output is `checks/outputs/binding.txt`.

| Item | sha256 | Result |
|---|---|---|
| Master, 116,245 frames at 60 fps | `00236d1a6915ff968235a30ab59769b02125ece0fee482e4fa3154c265ac7c24` | matches verification.json |
| SRT | `c9321cd7a2912d4457f2dc617735fd06f0b2fc7d684442815fa46b989bc31b7c` | matches |
| VTT | `45833988985413f3cfd8c44d5e18959be47ff5003d0dcbc570ce0b74ddfc0b45` | matches |
| `V/verification.json` | `22fb77188d6e9d86a0c9499e7a81db782d4b29830d1ddb4be40fe4dd3003fd62` | read as the binding |
| 339 sheets, 1,356 pictures | see `viewed-sheets.tsv` for 001–112 | all match |

The sheets are named 001–339; there is no sheet-000. `viewed-sheets.tsv` gives each sheet's full hash, frames, seconds, status and note. The session transcript shows 112 Read calls, one per sheet from 001 to 112, each returning an image, and none outside that range. I did not open sheet 113 or later.

## Observed defects

| ID | Severity | Where | What I saw | Evidence | Suggestion |
|---|---|---|---|---|---|
| O1 | Low, timing | 04_7, sheets 107–108 | "≈ 138" is on screen by 610.73 s, within 0.5 s of sentence 5's mark. Sentence 6, which holds that number, has its mark at 618.70 s. The result leads its sentence by about 8 s. This is not a quiz, so no hold rule is broken, but it goes against say-before-picture. `logarithm05-08-teaching-repair01` staged the same fix for the same pattern in 05_20. | f36614 has no fraction; f36644 has the whole line | If chapter 04 is rendered again, reveal `≈ 138` at sentence 6. Not worth a render alone. |
| O2 | Low, layout | 02_4, sheets 056–059 | C's "10" ends 52 px from the right edge of the 3840 frame (1.4%). It is not clipped. This is the layout lint's "tight" flag. | Native crop of f18914 | Accept. |
| O3 | Low, layout | 02_5, sheets 060–065 | C's left end and its "1" are off-frame. This is the lint's "outside" flag. It follows from putting C's 10 above D's 4. The marks the sentence uses, C 6, C 10, D 2.4 and D 4, are all in frame. | Native crop of f19766; sheets 062–065 | Accept. |

**Cosmetic, no action proposed**
- **C1:** `×`, `≈` and `≠` look heavier than the Computer Modern around them (f28980, f37256).
- **C2:** captions set some variables upright where formulas use italic: "slope 1 at x = 1" (095), `z`, `w`, `zw`, `iz` (111–112), and `e` on the ratio ruler (097–100).
- **C3:** in 03_2 the "0" appears before its label "laps past the top:" (f22412, 373.53 s). Both are visible by 373.83 s (f22430).

**Tool nit, not in the master:** the caption bars on sheets 056, 057, 060 and 061 show tab characters as boxes and truncate long lint reasons. Those bars come from my `verify_stage09_final.py`.

## Prediction holds

Cue times are from `placements`. Each gap between cues is about 4.85 s. Each countdown bar is 4.05 s from its first to last sampled picture, inside the 4–6 s standard.

| Cue | Sheets | Question held | Cue ends | Bar | Next cue | First answer on screen |
|---|---|---|---|---|---|---|
| 00_4 | 012–013 | "? Hz" | 71.71 s | 72.32–76.37 s | 76.57 s | √2 at 82.35 s; 622.25 at 87.23 s |
| 01_3 | 030–031 | "3 ?", triangle resting at 1.50, the midpoint of 2 and 4, not the answer | 178.56 s | 179.17–183.22 s | 183.42 s | 3¹² at 186.28 s; pos(3) ≈ 19/12 on sheet 034 |
| 02_2 | 049–052 | "2.5 × 3.2 = ?" | 292.04 s | 292.65–296.70 s | 296.90 s | "= 8" at 297.40 s |
| 04_6 | 103–105 | "×2 when?" | 584.65 s | 585.25–589.30 s | 589.50 s | t ≈ 13.9 at 600.73 s |

The 05_1 question ("So what is i times i?") falls after sheet 112. It is in BronzeJaguar's range.

**Two judgment calls for root.** Neither is stated as an answer, and I read both as deliberate:
- During 02_2, D's printed "8" is legible under the yellow line. The question asks the viewer to read the rule.
- During 04_6, the level-2 line crosses the growth line with no number beside it.

## What was checked

- **Maths:** I compared each sheet with the script and worked the on-screen values by hand. This covered ruler positions, dial angles, curve points, tangents and every printed equation; `viewed-sheets.tsv` names them. No oracle was run, and `facts.rs` was not opened.
- **Readability and overlaps:** every sheet at panel scale, which is 1/4 of 4K linear, plus the full-size frames below.
- **Transitions:** chapters 01–05 each open black in the sampled pictures. The two cross-fades that were caught, at 212.28 s and 539.98 s, show ghost overlap only during the fade.
- **Full-size frames:** 15 pulled from the master, listed in `pulled-frames.tsv`. I opened 14. The 15th, f22500, has the same bytes as f22460. The 6 that are also sealed pictures reproduced their sealed hash at 960×540, so the pull reads the same frames as the sheets.

**Concerns these frames closed**
- Between 101.38 and 106.85 s, where no picture was sampled, the screen shows two strips and "multiply by sliding" at 103.00 and 105.50 s. That fits "sliding one strip of wood along another".
- The keyboard's green dots sit on the A keys, and 13 blue dots mark 12 semitones (f2745).
- The struck "660 by difference?" does not touch the 880 label (f5755).
- The level line stays below the `y = e^x` label at every height I saw: 2.16, 2.13, 1.03, 0.1, −0.84 and 1.5.

## Unverified concerns

Nothing below is a known fault. I could not check these from stills.

- **U1, between pictures:** 448 of the 38,782 frames in range were sampled (1.2%). The largest gaps are 9.55 s (10.37–19.92 s), 7.77 s (610.93–618.70 s), 7.15 s (542.42–549.57 s) and 7.03 s (593.20–600.23 s). Gaps can hide real content: at 485.00 s the level line drops to about −0.84, turns red and the count reads 0. That is correct, but no sheet shows it. In f28980 the moving line also passes over the axis label "1"; other transient contacts could exist.
- **U2, motion:** easing, pointer sweeps and reveal order were not reviewed.
- **U3, voice against picture:** sentence labels come from timeline marks. Whether a word lands with its picture was not heard, and that includes O1.
- **U4, silence in the holds:** the holds are shown only by cue placements.
- **U5, 4K quality:** only the 14 opened frames were judged at or near full size. Text was crisp in those. Compression in fades and on dark gradients was not judged.
- **U6, fonts:** whether C1 is a fallback face was not established.
- **U7, small screens:** the grey footnotes and thin `years`-axis labels were not judged at phone size.

## Disclosure

This review is not independent of the source.

- **Source I wrote:** the `logarithm-repair01-04` patch, which touches chapters 01–04, and `folded_decade.rs`, the candidate behind chapter 03 (sheets 065–080). I also wrote `logarithm05-08-teaching-repair01`; its handoff lists 05_1 as unchanged, so I did not write the part of chapter 05 on sheets 109–112 through that repair.
- **Earlier reviews and tools I wrote:**
  - the 01–04 picture review (Mail 1448);
  - `final-verification02`, the tool that made these sheets and their captions (sealed `e8e19275`);
  - `render-preparation04` and the render03 review;
  - the stage09 audio preparation and timing inspection.
- **Others' work on this source:** WhiteSwan's follow-ups and repairs touch `slide_rule.rs` and the openings, BronzeJaguar repaired 05–07, and root bound `building_ruler.rs`. I did not audit the full authorship of every line.
- **Other reviewers:** I did not read `logarithm-final-review-pane169-01` or any review of sheets 113 and later.

## Limits

- **No claims:** nothing was heard, no motion was reviewed, and no acceptance is given or implied. Fine sync and final AV sign-off remain open.
- **No mutations:** nothing was rendered, decoded for production, synthesized or published. The only decodes were single frames, written to a private temporary path. Production files, Beads and TODO were not edited.
- **Notes:** I wrote the sheet notes as I read the sheets. `checks/build_ledger.py` proves the hashes, not the notes.

## Files and replay

| File | What it is |
|---|---|
| `REVIEW.md` | This note |
| `viewed-sheets.tsv` | The 112 sheets read: hash, frames, seconds, status, note |
| `pulled-frames.tsv` | The 15 full-size frames: hash, how each was read, what it settled |
| `checks/sheet-notes.tsv` | The per-sheet notes, input to the ledger |
| `checks/build_ledger.py` | Rebuilds the ledger and re-checks every hash |
| `checks/pull_frames.py` | Pulls named frames and compares sealed pictures |
| `checks/outputs/binding.txt` | The hash results at seal |

From the repository root, where OUT must not exist:

```
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-final-review-pane167-01/checks/build_ledger.py OUT
```
