# Apportionment chapter 12: picture review 01

**AUTHOR REVIEW, NOT INDEPENDENT.** I (WhiteSwan) wrote this chapter, so this is an author's read and does not count as independent review. It answers BlueSalmon's Mail 1169.

Nothing here was applied. I ran no build, render or TTS, and touched no Beads, commits, canonical files or frozen candidate files. Every fix below is a proposal and is uncompiled. Repairs and integration are root's.

## Verdict

**Hold the proof picture.** The numbers are right and the spoken proof holds, but from 171.0 s to 221.5 s the picture contradicts the sentences it sits under.

| | Finding | Severity |
|---|---|---|
| F1 | The real run's fills stay on screen during the proof, and they break what the voice says | **High** |
| F2 | The 12_10 quota boxes turn solid green and stay that way to the chapter's end | Medium |
| F3 | After the restore, 4 yellow rings sit beside "0 + 1 + 4 = 5" and "Cedar 4" | Medium |
| F4 | About 29 s of the densest proof sentences run over a nearly still picture | Medium |
| F5 | The think ring cuts "2,000" during the think hold (lint finding 2) | Medium |
| F6 | The 16-seat result is at full strength for about 0.6 s before the wipe | Low |
| F7 | 1.25× flashes misstate a tile's length and cover labels, about 1 s each | Low |
| F8–F13 | Strike legibility, silent walk, empty screen under voice, a 2 px near-touch, one ambiguous label, head at the safe-area line | Low |

**What held up:** every count agrees with the oracle, the answer hold holds, cue order is clean, and nothing is clipped.

## Evidence

| Item | Exact value |
|---|---|
| Folder | `narration_lab/third-wave-20261024/root/five-film-release-20261027/apportionment-opus-rebuild-frames01/12_both_promises/` |
| Sheets read | **178 of 178** (`sheet-000` to `sheet-177`; 177 sheets of 4 frames and a last sheet of 1) |
| Frames on those sheets | **709** (`frame-0000` to `frame-0708`) out of 7,487 rendered frames, about 9.5% |
| Samples | 499 on the half-second grid plus 210 others (cue boundaries and lint anchors) |
| Sample gaps | Largest 0.500 s, smallest 0.013 s |
| First and last sample | 0.000 s and 249.316 s, of a 249.566 s chapter |
| Frames opened at full size | **9:** 0048, 0057, 0170, 0300, 0305, 0430, 0545, 0640, 0700 |
| Frames receipt | `started` 08:02:52Z, `finished` 08:09:19Z, `exit` 0, no stop reason, `admitted` true, input hashes unchanged, PID 51129 |
| Preview receipt | `started` 07:25:20Z, `finished` 08:02:47Z, `exit` 0, no stop reason, input hashes unchanged, PID 13153 |
| Render flags | `--res 1280x720 --fps 30 --scratch-voice --lint --crf 26 --preset ultrafast --threads 4` |
| Jobs started by me | None. The existing controller ran the extraction |

Source read at these sha256 prefixes, in canonical `explainer_corpus/apportionment/`:

- `src/chapters/both_promises.rs` 81d2f0aa92091dc9
- `src/chapters/draw.rs` dd385ed5f45f09ff
- `script.tsv` cfcec06911c7d94e
- `oracle.py` 56009064259c3fba
- `src/facts.rs` ec55517c9544e275
- `docs/description.md` 0c87d6bcdabea112

All of these equal the frozen candidate except `draw.rs`, which differs only in root's retired `DIALS` constant and two `const { assert!(…) }` lines. Line numbers below are canonical.

**Limits:** this is a read of still frames and source. It is **not** listening, motion, fine-sync or independent AV acceptance. The render used a scratch voice, so every time here is a scratch estimate, and `wait_for` phrase times are interpolated inside a sentence. "Still" means no visible change across samples up to 0.5 s apart.

## Cue order and gaps

The order is 12_1 to 12_10 with no overlap, from `12_both_promises.cues.tsv`. The last cue ends 0.620 s before the chapter does.

| Cue | Start | End | Length | Gap before | First frame | Last frame |
|---|---|---|---|---|---|---|
| 12_1 | 0.600 | 14.846 | 14.246 | | f2 | f40 |
| 12_2 | 19.933 | 50.718 | 30.785 | **5.087** (think hold) | f59 | f138 |
| 12_3 | 53.233 | 74.787 | 21.554 | 2.515 | f146 | f212 |
| 12_4 | 76.000 | 94.862 | 18.862 | 1.213 | f217 | f269 |
| 12_5 | 96.067 | 121.082 | 25.015 | 1.205 | f275 | f344 |
| 12_6 | 121.700 | 143.254 | 21.554 | 0.618 | f347 | f407 |
| 12_7 | 143.867 | 165.421 | 21.554 | 0.613 | f411 | f470 |
| 12_8 | 167.433 | 196.679 | 29.246 | 2.012 | f478 | f558 |
| 12_9 | 197.300 | 233.469 | 36.169 | 0.621 | f562 | f659 |
| 12_10 | 234.700 | 248.946 | 14.246 | 1.231 | f664 | f705 |

**Answer hold: holds.** 12_1 asks who gets the fifteenth seat, and the hold runs from 14.846 to 19.933 s. Nothing marks Birch through f61 (20.133 s). The first mark is the hero glow on card 16 at f62 (20.433 s), after "Birch." has begun. 12_1 has the chapter's only think ring. The question opening 12_8 is answered at once by the next sentence, so it needs no hold.

## The five native lint findings

The linter looks every 0.2 s (`SAMPLE_S`, `crates/explainer_kit/src/layout.rs:70`), so looks × 0.2 s gives its window. The frames sometimes show a defect outlasting that window, so both are listed.

| # | Lint (`12_both_promises.layout.tsv`) | Lint window | On frames | Judgment |
|---|---|---|---|---|
| 1 | `edge`, group holding "Cedar seat 1", y up to 3.85, 367 later looks | 1.8 to 75.2 s | Head present from f7 until the wipe begins at f216 (75.6 s). Not clipped | Low, F13 |
| 2 | `crossed`, unlabeled stroke over "2,000", 15 later looks | 15.6 to 18.6 s | f43 (15.5 s, fading in) to f58 (19.6 s), clean at f59 (19.933 s). Full-size 0048 and 0057 show it | **Real**, F5 |
| 3 | `crossed`, stroke over "Cedar seat 11", 70 later looks | 58.4 to 72.4 s | f161 (58.4 s) until the fade at f204–f206 (73.2–73.5 s) | Deliberate strike, F8 |
| 4 | `crossed`, the same stroke over "2,364", 70 later looks | 58.4 to 72.4 s | Same | Deliberate strike, F8 |
| 5 | `crossed`, the same stroke over "15", 69 later looks | 58.4 to 72.2 s | Same | Deliberate; the rank is hard to read, F8 |

The linter did not flag F1, F2, F3 or F4. They are problems of meaning, visual weight and pacing, and only reading the frames against the words found them.

## Findings

### F1 (High). The proof picture is not an instance of the spoken proof

**Where:** f490–f626 (171.0–221.5 s), sheets 122–156; plainest on full-size frame 0545 (192.0 s). Source: `both_promises.rs:771-786`, with the comment at `:848-849`.

12_8 asks "Could a passed tile ever be left out? Suppose one were." The picture fades one fill (Cedar 8, from 10½ to 12) and marks it with a red ring and "?". The other 12 fills of the real run stay on screen at marker 13. `then` goes to 6 and `now` to 13. Against the words:

| The voice says | The picture shows |
|---|---|
| "Hand out seats one at a time" (12_2) | Cedar fills 1–7, a hole at 8, then a fill at 9. Cedar's ninth seat cannot exist without its eighth |
| "the last moment the rule handed out a tile that ends later than the missing one" | At house 6 the rule gave Birch 2, ending at 8⅔, earlier than the missing tile's 12. Birch 3 (ends 13, given at 12) and Cedar 9 (ends 13½, given at 13) both end later and are both handed out after `then` |
| "Every tile handed out since then… had not been entered at that moment" | Ash 1, handed out at 9, starts at 0. Its fill visibly crosses the `then` line (px 498 on frame 0545) |
| "And every one of them ends by now" | Cedar 9's fill runs past the `now` line (px 906 against px 880 on frame 0545) |
| "one more of them than seats added": "7 + 1 = 8 tiles inside" | A viewer who counts inside the band finds 4 whole fills and a hole, plus 3 fills straddling its edges (Ash 1, Birch 2, Cedar 9) |

The real seats 7 to 13 are C5, C6, A1, C7, C8, B3, C9, so two of the seven visibly break the claims. A careful viewer, the one this section is written for, will check the sentence against the fills and conclude the proof is wrong.

**The maths is fine.** I re-derived it: the spoken argument holds, and `docs/description.md:127-137` has the precise form. No real run can look like the supposition, which is the theorem. The comment at `:848-849` already says so, but the code leaves the real run in view anyway.

**Proposed fix (picture only, no script change):** stop showing a run during the supposition.

1. At `:786`, fade all the fills held at `to`, not only `held`, leaving empty outlines, the band and the red ring. Keep the check at `:771-775`, rewritten as a plain test, because `held` is no longer faded by name.
2. At `:909-911`, fade out `ask`, `gone`, `need_t`, `cannot`, `then_t`, `then_mark`, `band`, `fit`, `bounds` and `sum`, then fade in fresh `tile_rect` fills for all 13 held tiles.

The real run then returns whole for "no passed tile is ever left out" and the green done line. All 13 fills also share one shade, and F3 disappears. `then` at 6 stays illustrative, which the red "suppose" line already signals. Sketch, uncompiled:

```rust
// at :786
let kept: Vec<Tile> = fills.iter().filter(|(h, _, _)| *h <= to as usize).map(|f| f.1).collect();
let kept_fills: Vec<Mob> = fills.iter().filter(|(h, _, _)| *h <= to as usize).map(|f| f.2).collect();
fade_out_all(stage, &kept_fills, 0.4)?;
// at :909-911
fade_out_all(stage, &[vec![ask, gone, need_t, cannot, then_t, band, sum], then_mark.clone(), fit.clone(), bounds.clone()].concat(), 0.3)?;
let back: Vec<Mob> = kept.iter().map(|t| tile_rect(stage, t, flat(town_color(t.town), 0.75))).collect::<fmn::Result<_>>()?;
fade_in_all(stage, &back, 0.4)?;
```

### F2 (Medium). The 12_10 boxes turn solid green

**Where:** f684 (241.5 s, starting), solid from f685 (241.8 s) through f708 (249.316 s), the last sample; full-size frame 0700 (247.5 s). Source: `both_promises.rs:966-967`.

`quota_box` draws a stroke-only rectangle (`draw.rs:205`). `refill` with opacity 1.0 uses `set_color_with_opacity`, and the frames show the rectangle filled as well as recoloured. The frames are the evidence here: in a local engine checkout that method only records a `SetColor` with opacity, and I did not trace it further or check it at the pin.

- The boxes become the heaviest thing on the chapter's last picture, on "Both keep the quota promise".
- They hide the cell lines inside them, and those lines are how a viewer reads where a stack ends.
- Cedar's narrow box is the same green as Cedar's tokens, so its tenth upper token merges into the bar.
- The flash at `:969` makes the Ash and Birch boxes solid yellow blocks at f692 (244.5 s).

The tokens do stay on top, so the counts remain readable. The teaching survives, but this reads as an accident, and it is one.

**Proposed fix:** recolour the stroke only. The crate already has the pattern at `no_rule.rs:309-311`: build `quota_box(..., Palette::green())` targets and `morph_all` the dim boxes into them. I did not check other chapters' `refill` calls; that is outside this scope.

### F3 (Medium). Four rings against "0 + 1 + 4 = 5"

**Where:** f629–f662 (222.5–234.0 s); full-size frame 0640 (226.8 s). Source: `both_promises.rs:865-870`, `:909-911`.

`fit` deliberately excludes the missing tile (`:867`) because the red ring marks it. That is right until `:909` removes the red ring. From then on Cedar 8 has no ring, so 3 Cedar rings sit beside "Cedar 4" and 4 rings in all beside "= 5". The restored fill also sits above the band, so Cedar 8 is a visibly different green from its neighbours.

**Proposed fix:** the F1 repair removes this. If F1 is not taken, add a hero `ring` on `missing` to the fade-in at `:911`.

### F4 (Medium). The densest sentences sit over a nearly still picture

**Where:** 174.0–203.4 s, about 29 s on scratch timing, spanning the end of 12_8 and the start of 12_9.

| Time | Picture | Voice, in order |
|---|---|---|
| f498–f530, 174.0–186.5 s | Still | "…or to the very start if it never did. Every tile handed out since then… had not been entered…" |
| f531–f532, 186.8–187.0 s | Band fades in | |
| f532–f561, 187.0–197.0 s | Still | "…or the rule would have picked a tile ending sooner. And every one of them ends by now." |
| f563–f566, 197.5–198.4 s | "suppose" out, band flash | "So all those tiles fit inside the stretch" |
| f566–f578, 198.4–203.0 s | Still | "…from that moment to now, and there is…" |
| f579, 203.4 s | `need_t` arrives | "one more of them than seats added" |

Phrase placement in this table is approximate, because `wait_for` times are interpolated inside a sentence.

"And every one of them ends by now" has no beat in the source at all. Root has said duration is not a defect, and I am not asking for cuts. The issue is that these are the sentences a viewer most needs help following.

**Proposed fix (picture only, on existing phrases):** at "had not been entered at that moment" (`:816`), mark the `then` line and the tile starts to its right. Add a `wait_for` on "And every one of them ends by now" that marks the `now` line and the tile ends to its left. Build these after F1, on the empty outlines, using a colour pulse rather than `flash_all` (see F7).

### F5 (Medium). The think ring cuts "2,000" (lint finding 2)

**Where:** f43–f58 (15.5–19.6 s); full-size frames 0048 (16.5 s) and 0057 (19.5 s), where even the last arc remnant lies across the digits. Source: `both_promises.rs:297`, `draw.rs:1161`.

The ring is centred at (6.1, −2.9) with radius 0.55, so its top reaches −2.35, inside card 18. That digit string matters: 12_3 says Birch's seat "beats Ash's second at two thousand", and the viewer may be making that very comparison during this hold.

Moving the ring down does not work. Clearing the card needs a centre near y −3.09, which puts the "pause for more time" note at about −3.92, outside the safe area.

**Proposed fix:** at `:297`, call `think(&kit, stage, v(0.4, -2.9))` for this one use, which also needs `think` added to the imports. The ring (px 627–726) and note (about px 612–740) then sit in free space left of the list, whose cards start at px 851. By my reading nothing else is there during the hold; `r15` arrives only in 12_3.

### F6 (Low). The 16-seat result barely lands

**Where:** f212 (74.787 s, cue end) with "16 seats: 1, 4, 11" only beginning; visible at f214 (75.0 s); fade begins at f216 (75.6 s). Source: `both_promises.rs:485-497`. The payoff of 12_3 gets about 0.6 s at full strength.

**Proposed fix:** `pause(stage, 1.0)` between `:492` and the wipe at `:497`. `pause` is already imported. Recheck on Oliver timing.

### F7 (Low). `flash_all` scales by 1.25

`flash_all` uses `indicate(..., 1.25)` (`draw.rs:876`). Each case lasts about a second or less, and the linter is silent on all of them.

| Line | Frames | Effect |
|---|---|---|
| `:584` | f304–f306, 106.5–107.0 s; full-size 0305 | Tile outlines grow while the voice says "so each tile is as long as". Ash's reaches about 11 on the axis instead of 9¾, and the outlines cross all three "per seat" labels |
| `:852` | f564–f565, 197.8–198.0 s | The band grows over "then", "now" and tick labels 5 and 10 |
| `:649` | f389–f390, 136.8–137.0 s | Passed bars grow leftward under "1.85" and "5.33"; no text is crossed |
| `:969` | f692, 244.5 s | Ash and Birch boxes; see F2 |

**Proposed fix:** a colour-only pulse at `:584` and `:852`. The first is worth doing because it misstates the very length being described. Leave the others.

### F8 (Low). The strike on card 15 (lint findings 3–5)

**Where:** f161–f206; full-size frame 0170 (60.5 s). The strike is what "so it is skipped" asks for. "Cedar seat 11" and "2,364" stay readable through it. The rank "15" is hard to read, since it is dim and the line runs straight through it. **Keep it.** The three findings describe one deliberate line.

### F9 (Low). The silent walk past 12_2's line (known open item, now measured)

12_2 ends at f138 (50.718 s) with the picture at 8 seats. Seats 9 to 14 run after the voice: f139 (50.918 s) shows 9 seats, f145 (53.0 s) about 14, and f146 (53.233 s) lands on 15 exactly as 12_3 begins. The pace is 0.3 s a seat (`both_promises.rs:407-419`), against about 10 s spent on the skip at seat 6.

12_3 starts when the picture finishes, so this 2.515 s gap should survive a voice change, and I see no overlap risk. All counts agree with the walk. The 7-seat state, where the skipped seat is finally handed out, falls between f134 and f135, so at this sampling I cannot say how it reads. **Acceptable unless listening says otherwise.**

### F10–F13 (Low)

- **F10. Empty screen under the voice.** f275–f281 (96.067–97.5 s) are empty while 12_5 has begun; names and axis start fading in at f282 (98.0 s). This is deliberate, tied to "along the house sizes" at `:566`, and fits "say before the picture". Listen before changing it.
- **F11. "+26/39 per seat" near-touch.** On full-size frame 0300 the label ends about 2 px (0.02 units) from Cedar's first tile, with no crossing, through f348 (`:573-581`). Use a slightly smaller size or the shorter "+26/39".
- **F12. "never with 3 towns" (`:514`).** At size 15 beside Webster's ✗, it can read as "fails: never keeps it", the opposite of the fact. The same label is at `webster_quota.rs:395` and `recap.rs:110`, and chapter 11 sets up its meaning, so change all three or none.
- **F13. Lint finding 1.** "Jefferson's list" is at the safe-area line and not clipped on any sample. Moving it from y 3.78 to about 3.70 at `:281` should clear the lint and still sit above card 1, whose top is at 3.545. Optional.

**Noted, no action:**

- The "marker at" read-out rounds while moving, for example "15" at about 14.6 on f468; settled values are right.
- The "?" reads orange on the band but stays readable.
- 12_10 uses only the left part of the frame.
- In 12_10, Ash's single upper token sits left of its box and Birch's three lower tokens stop at its edge. The film reads a stack by where it ends, and `quota_box` appears in earlier chapters, but I did not re-read how they teach that convention.

## Native quota-proof teaching, beat by beat

| Cue | What it teaches | Read |
|---|---|---|
| 12_1 | Jefferson's 1, 3, 11 breaks Cedar's ceiling of 10; think hold | Good, except F5 |
| 12_2 | The capped walk, including the real skip at seat 6 | Good; path right all the way, F9 |
| 12_3 | Cedar skipped at 15, Birch's 2,250 beats Ash's 2,000; 1, 4, 10 then 1, 4, 11 | Good, F6, F8 |
| 12_4 | Scoreboard: both promises, Theorem 2 cited | Good, F12 |
| 12_5 | Tile lengths 9¾, 4⅓, 1½ | Good; boundaries measure correctly on the sheets, F7, F11 |
| 12_6 | Passed = rounded down, entered = rounded up, at marker 8 | **Strong.** Bars, rings and veils match floor and ceiling in every row |
| 12_7 | List order is end order; the walk in tiles, with both waits | **Strong.** Rank numbers legible on frame 0430; "not entered at 6" and "at 15" teach the strict edge; fill order matches the walk through 16 seats |
| 12_8 | Suppose a tile is missing; go back to `then` | **Picture misleads,** F1, F4 |
| 12_9 | 8 tiles needed, at most 7 fit; quota promise | Numbers right; picture F1, F3, F4. The done line holds about 4 s (f648, 230.0 s, to f662, 234.0 s), which is good |
| 12_10 | Same promise, two answers | Right values and order, F2 |

## Counts against the oracle

`oracle.py` printed 429 keys. I also recomputed these with my own scratch Python, which is still the author checking the author. All agree.

| On screen | Frames | Oracle |
|---|---|---|
| 1, 3, 11 at 15 seats | f9 | `jefferson.15.*` |
| Cedar at most 10; Birch 2,250, Ash 2,000 | f20–f23, list | `quotamethod.15.cedar_ceiling`, `.birch_priority`, `.ash_priority` |
| 1, 3, 10 arriving at 15 seats | f146 | `quotamethod.14.*` |
| "15 seats: 1, 4, 10" | f189–f190 | `quotamethod.15.*` |
| "16 seats: 1, 4, 11" | f214 | `quotamethod.16.*` |
| 9 3/4, 4 1/3, 1 1/2 | f321–f338 | `tile.len.*` |
| "39,000 ÷ 2,250 = 17 1/3" | f437 | `tile.end.birch.4` |
| "15 to 16 1/2: not entered at 15" | f469 | `tile.end.cedar.11` |
| `then` 6, `now` 13, 7 seats | f498, f579 | `stretch.from`, `.to`, `.seats` |
| 0, 1, 4; total 5 | f592–f605 | `stretch.inside.*` |
| Quota tokens 1, 4, 10 | f672 | `quotamethod.15.*` |
| Hamilton tokens 2, 3, 10 | f682 | `hamilton.15.*` |

Checked against the scratch recompute only:

- The 18 card values on the list.
- The whole walk through 16 seats, including the skips at 6 and 15.
- Live shares at 8, 13, 15 and 16: 0.82, 1.85, 5.33; 1.33, 3.00, 8.67; 1.54, 3.46, 10.00; 1.64, 3.69, 10.67.
- Row bounds 0.718, 1.615, 4.667.
- Rank numbers 1–17 against end order.
- Allowed ranges "Ash 1 or 2, Birch 3 or 4, Cedar 10".

## Borders and overlap

- **Frame borders:** nothing is clipped on any of the 178 sheets. The lowest text, "8 tiles cannot fit" at y −3.45, is inside the safe area.
- **Real overlaps:** F5, and the F7 transients.
- **Deliberate overlap:** F8.
- **Near-touch:** F11.
- **Clear on the frames:** top-line text against "marker at"; `then` and `now` labels against the Ash row; bound lines against tick labels; the red wait labels against ticks and each other; 12_10's lines against the rows. The marker line does cross rank numbers in passing during the 12_7 walk.

## Not done

- No listening, motion, fine-sync or independent AV acceptance, and no claim of having watched the chapter. No Oliver audio was involved: this render used the scratch voice.
- No fix has been compiled or applied.
- Other chapters were read only for the `quota_box`, `refill` and "never with 3 towns" call sites.
- The linter's narrower windows are reported as observed; I did not investigate why they differ from the frames.
