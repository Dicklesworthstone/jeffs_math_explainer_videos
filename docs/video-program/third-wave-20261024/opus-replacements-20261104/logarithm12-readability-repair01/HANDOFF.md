# Logarithm 12: closing readability

BronzeJaguar, for BlueSalmon, answering the assignment relayed by the owner after root's review of all 95 sheets and 379 pictures of stage06 `12_what_it_forgets`. Narration is ElevenLabs Oliver and is unchanged.

**State: staged and frozen for root. Nothing here has been compiled, tested by Cargo, rendered or heard, and this is not a native, listening or independent acceptance.** Canonical is untouched. The patch changes 13 film lines in `what_it_forgets.rs`, moves none and changes no timed call or word. A small helper follows the film's code.

## The room

Chapters 10 and 11 could lift their footers alone. This chapter cannot. The closing column runs down to the family's row, so the footer at y −3.40 would stand 0.08 under `here: the whole family`.

The room is above the column. `so:` ends at y −0.311 and `how far` starts at −1.000, an empty 0.689. So the whole number-line block moves up 0.2, and the footer takes the space that opens below it.

All extents are measured on frame 378. "After" is arithmetic from them, not a render.

| Between | Was | After |
|---|---|---|
| `so:` and `how far` | 0.689 | **0.489** |
| `how far` and the rings | 0.233 | 0.233 |
| The rings and the labels | 0.077 | 0.084 |
| The labels and the tie | 0.244 | 0.211 |
| The tie and the convention | 0.211 | 0.201 |
| The convention and the family | 0.211 | 0.201 |
| The family and the footer | 0.300 | 0.280 |
| The footer and the frame's edge | 0.267 | **0.487** |
| The rod's foot and the footer | 0.944 | 0.724 |

**Sideways:**
- The labels grow to about 0.32 wide at most, under marks 0.62 apart, leaving about 0.30 between them.
- The convention's right edge reaches x 5.56, and the limit is 6.86.
- The footer stays ±6.31.

**The room is empty.** On all 47 sampled frames from 145.0 s to the chapter's end, a script found no ink in any of these:
- above the block: x 1.2 to 6.9, y −0.42 to −0.93
- left of it: x 0.6 to 1.3, y −0.42 to −2.9
- right of the rows: x 6.4 to 7.11, y −1.9 to −3.3
- under the cube, down to the footer's new row: x −7.11 to 0.9, y −2.95 to −3.45

## The repair

| Item | Line | Was | Now |
|---|---|---|---|
| Number-line labels, −3π to 3π | `:1244`–`:1246` | 16 pt, `dim`, 0.3 under the line | 20 pt, `ink`, 0.32 under it |
| The dots at each end | `:1251` | `dim` | `ink` |
| `for the number −1, Log(−1) = iπ by convention` | `:1297`–`:1299` | 18 pt, `dim`, y −2.7 | 20 pt, `ink`, y −2.5 |
| Higham footer | `:1283` | `precisely(`: `dim`, y −3.62 | `footer(`: `ink`, y −3.40, still 22 pt |
| `how far`, the line, the tie, the family | `:1266`, `:141`, `:1273`, `:104` | y −1.1, −1.6, −2.3, −3.1 | y −0.9, −1.4, −2.1, −2.9; size and colour unchanged |

`:50` drops the unused `precisely` import, which Clippy would refuse. `footer` and `FOOTER_Y` stand after the film's code, so no held line moves. `footer` builds the same `precisely: …` string as the shared helper, which I did not touch.

| Item | SHA-256 |
|---|---|
| `logarithm12-closing-readability.patch`, on canonical or stage06 `what_it_forgets.rs` `e938b750ce579fae…` (same bytes) | `b4662627c93ac4fe6d437568c33dfb274d4b5a6048aeba5b4572b07ad6ad2f19` |
| Result, staged in this folder at its crate path | `6951d3b2f157387683efaa3ec7c9d9d12a23c8448df247531cd5f0081bab1e28` |

Apply with `git apply` or `patch -p1` from the repo root. Below the film's code, the patch adds one test, moves two const bounds with the rows, and updates the convention's size in the width table.

| Contrast on the ground, 720p, measured | Brightest tenth of inked pixels | Median inked pixel |
|---|---|---|
| Labels as staged, `dim` 16 pt | 5.16 | **3.12** |
| Convention as staged, `dim` 18 pt | 5.56 | **2.96** |
| Footer as staged, `dim` 22 pt | 5.90 | **3.74** |
| `how far`, a real `ink` 20 pt row on frame 378, the labels' and convention's new spec | 14.78 | 8.03 |
| A real `ink` 22 pt row on chapter 10's frame 229, the footer's new spec (from my sealed chapter 10 output) | 15.81 | 8.19 |

`checks/outputs/closing-block-NOT-A-RENDER.png` shows frame 378 as staged, the same frame with the rows moved, recoloured and scaled by hand, and both closing blocks at twice the size. The scaled labels and convention are blurred by that scaling, so judge the layout only.

## Decisions for you

1. **`ink` on the convention.** It is an aside between two `gold` rows, and `ink` is slightly brighter than they are (14.78 against 12.58 on the brightest tenth). I chose it because `dim` at any size that fits stays muted, and `how far` already uses neutral `ink` at 20 pt in the same block. The other palette colours carry meanings in this chapter.
2. **The footer stays 22 pt.** It is 12.62 wide. 24 pt would be 13.77, past the lint's 13.72. 23 pt would fit at 13.20, a 4.5% gain, but would differ from chapters 10 and 11. Two rows would need about 0.45 more height, which this column does not have without squeezing the rows.
3. **`so:` and `how far` are now closer.** The 0.489 is still more than twice the rows' own gaps, so the block should read as new. Only a frame can confirm that.
4. **The rest of the film now differs.** Once chapters 10, 11 and 12's patches land, five chapter files still use `draw::precisely` (`dim`, y −3.62): `branch_cut`, `product_rule`, `ratio_ruler`, `slide_rule`, `undoing_growth`. I have not checked those for room.

## One thing I saw and did not change

The footer is 128 characters. On the scratch voice it is fully shown for about 7.5 s, while the voice is still speaking, which is about 17 characters a second. You asked me to keep the timings, so I did. Check this against Oliver's real length.

## Scope

- **Words:** every string in the film's code is byte for byte, in order. That includes the footer's `eigenvalue −1, so no principal log (Higham §4 Fact 1)`, the convention and the tie. Every mark, both runs of dots and the family's dots stay.
- **Timing:** no timed call changes, so the chapter's length should not change and its stem should still fit. Only a native run can confirm that.
- **Narration:** no wording need, and no take is affected. `script.tsv`, `facts.rs`, `oracle.py` and `draw.rs` are untouched.
- **Not touched, since you did not flag them:** the ticks (`dim`), the line (`faint`), `time` and `laps so far` (`dim`, 20 pt), and the size and colour of the tie and the family.

## Bindings

**The one visual hold on this file keeps its line, marker and interval. Only the file's hash changes**, and `tools/narration_director.py:124` raises on a stale evidence hash.

| Map13 row | Lines | `sha256` and `evidence.sha256` | `interval_sha256` |
|---|---|---|---|
| 12_5 → 12_6, prediction | `:1099`–`:1123`, marker `predict(&kit, stage, v(2.6, -2.1), 3.4)?;` | `e938b750ce579faed820ae7ddd0d289819f7148bdfb991dc99f24d35d6f8491b` → `6951d3b2f157387683efaa3ec7c9d9d12a23c8448df247531cd5f0081bab1e28` | **Unchanged**: `b238a92968284627ce972479bb1c5a93ac5f5df33efee0b17ddea5d362c8387f` |

These are values only, good only for exactly the staged file; I did not patch the map. The formula reproduces the old value on canonical.

**Also in `combined-repair-stage06/bound-plan/`:**

| File | Row | Delta |
|---|---|---|
| `hold-audit.json` | The 12_5 prediction row | `source_sha256` as above; the rest unchanged |
| `hold-audit.json` | 8 cues, 3 other waits, 1 chapter finish | No change; line and marker verified |
| `hold-audit.json` | Reading holds | None on this file |
| `phrase-anchors.json` | 5 anchors: `:848`, `:1136`, `:1146`, `:1293`, `:1303` | `source_sha256` as above; cue, byte, const and hook line unchanged |
| `phrase-anchors.json` | 3 sentence-share hooks: `:1199`, `:1202`, `:1280` | No change |
| `inputs.sha256` | Line 22 | New hash |
| Its copy of map13 | The 12_5 row | As above |

Your binder must take the new input. Map12 also names the old hash, and I have left it as history. `draw.rs` stays at the hash map13 names, and `narration_units.json` does not name this file's hash. This patch and those for chapters 09, 10 and 11 touch different files and can land in any order.

## Native picture checks

`source-markers.json` has these in full. Anchor on cue and phrase: the old times are stage06's scratch voice, not Oliver. Frame numbers are labels in `logarithm-stage06-frames01/12_what_it_forgets`, counted from 0.

| Anchor | When | Old frames | Expect | Fails if |
|---|---|---|---|---|
| L1 labels and dots | After `:1277`, to the end | 337, 340, 378 | −3π to 3π in ink, 20 pt, at y −1.72; both runs of dots in ink | Grey; a label touching a ring, a neighbour or the tie |
| F1 footer | 12_21 sentence 2, after `:1292`, to the end | 360–378 (157.5–165.016 s) | Ink at y −3.4, exact words | Grey; under 0.40 off the edge; touching the family; any word change |
| C1 convention | "a convention broke the same tie", after `:1301` | 364–378 | Ink, 20 pt, y −2.5, left edge x 1.4 | Grey; touching the tie or the family; any word change |
| S1 the whole stack | "the honest answer is the whole family", after `:1308` | 370–378 | All eight rows in order, none touching; the probe passes | Any two touching; `how far` reading as part of `so:` |
| N glyphs | A final-size frame after `:1308` | n/a | π − ½ ± § … and the exact words, read by eye. These are the same characters at new sizes | A clipped or fallback glyph |

```
python3 -B <this folder>/checks/closing_probe.py closing <frame.png>
```

The probe works at any frame width. Pick a frame after the family's fade. Its marks are my own: 11.0 on the brightest tenth, 0.40 off the edge, 0.20 from the family to the footer, and row heights and widths that prove the size. Frame 378 fails it on every row, and the hand-made mock-up passes. It measures rows; it does not read them.

## Evidence

Regenerate from the repo root with a scratch path that does not exist yet. It starts the probe with `python3 -B`, because that needs PIL, and regenerates the patch and `source-markers.json`.

```
/usr/bin/python3 -I -B <this folder>/checks/run_checks.py <fresh-scratch>
```

| Check | Result | What it cannot show |
|---|---|---|
| Patch round trip by `git apply` | Applies, gives the staged bytes, reverses | Anything about behaviour |
| Film-line diff | The 13 lines above; no timed call changed; every string identical and in order | That it looks right |
| Room mirror | The gaps above, from measured extents | No frame at the new rows |
| Empty-room scan | 47 frames, four regions, no ink | Between samples, though nothing moves in this block's room then |
| New test's conditions | Pass on staged, fail on canonical; 15 plants caught | The Rust has never run |
| `docs/check_cues.py` on a scratch copy | Passes both with identical output; refuses a removed `say` | Only cue order |
| `rustfmt --check` | Clean on both | Not a compile |

**What I read:** sheets 084–094 by eye, which is frames 336–378, and the mock-up sheet. Source lines `:1221`–`:1309`, the test module and the constants. This is not a chapter review; yours covers all 95 sheets.

My failed checks on the way were my own and are fixed in the delivered scripts:
- The probe first passed the old convention row, because its box caught part of the tie. It now checks row height.
- The checks' regex first matched the tick line instead of the label row.
- A sentence-share check was vacuous: it read a dict's keys and found no rows. It now requires all 3.

## Overlap and gaps

- **Not independent.** You named the defects, and I wrote the fix and every check.
- **Shared memory.** WhiteSwan and RubyRaven worked on this film, and we share one auto-memory directory, one model and the same skills. I did not trace who wrote this closing block.
- **Still yours:** Cargo, tests and Clippy; the glyph probe; a chapter 12 render and the anchors on it; rebinding; Oliver sync and listening; the final 4K look.

I changed nothing in canonical, Beads or any sealed folder, and started no agents.
