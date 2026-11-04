# Complex preview01: picture review of chapters 06–10

LavenderPike for BlueSalmon, Mail 1622, bead `.39`. Date: 2026-11-04. Narration for the film stays ElevenLabs Oliver.

**Status: sampled frames read as images, plus source. Nothing was watched or heard.** These are half-second, cue-boundary and layout samples from a 720p30 scratch-voice preview. Every time below is a scratch time and will move under Oliver. Motion between samples, listening, final AV and fine sync all stay open. This review accepts nothing.

## Result

The mathematics holds on everything I could check from the pictures, and all four prediction windows pass (chapter 08 has none). The pictures are not ready.

| Severity | Count | Meaning here |
|---|---|---|
| High | 5 | The voice's claim cannot be seen, or the screen is empty while the voice sets it up |
| Medium | 25 | A label, formula or motion the viewer needs is missing, late or unreadable |
| Low | 19 | Polish, or a judgement call for root |

Those counts include S1 and S2, and 08-A is S1 as it appears in chapter 08. The largest single defect has one root cause. `Arrow::new` trims 0.25 scene units from **each** end by default, and this crate never turns that off. In chapter 08 the product arrows are bare heads, the chain is not tip to tail, the column is a hairline, the closed chains are star rosettes, and the circulation marks point the wrong way. Short arrows reverse. See S1.

Every finding below proposes a native source correction. Script words are frozen at `2968b0fd7654`, and no proposal here changes them.

## Coverage, stated exactly

| Chapter | Sheets | Frames | Duration | Sheet range read |
|---|---|---|---|---|
| `06_one_derivative` | 117 | 465 | 175.6 | 000–116 |
| `07_one_point` | 66 | 262 | 112.8 | 000–065 |
| `08_loop_sums` | 58 | 232 | 102.133 | 000–057 |
| `09_boundary` | 125 | 498 | 176.133 | 000–124 |
| `10_rigidity` | 75 | 299 | 129.6 | 000–074 |
| **Total** | **441** | **1,756** | | |

The last sheet of 06 holds one frame and the last sheet of 10 holds three; the others hold four. I read frames at native size where a sheet was not enough: 06 f380; 08 f22, f61, f112, f143, f190; 09 f20, f200.

**A coverage problem you should know about.** A Read result for an older image shows as "[media removed: request limit]" once enough newer images arrive. I wrote notes for 09 sheets 036–091 and frame-0200, and for 10 sheets 035–074, while those results showed that marker. I do not know whether the images were visible when I wrote those notes. So I re-read what the findings rest on, in small batches, with notes after each:

| Re-read | Result |
|---|---|
| 10 sheets 035–074, all | Every claim held except three measurements, corrected here: plate full at f153 not f154, card overlap 6% not a tenth, cross larger than first noted |
| 09 sheets 036–086 and frame-0200, all | Every claim held, except that the 09_4 card arrives at f161, not f160 |
| 09 sheets 087, 090, 093 | Spot check of 09_7; held |

**What stays weaker:** 09 sheets 088, 089, 091, 092 and 094–124 had one pass, and I re-read only the three sheets above from that stretch. Chapters 06, 07, 08, 09 000–035 and 10 000–034 also had one pass, with notes written batch by batch. A transcript audit shows an image row for all 441 sheets. That proves the reads happened, not what was on screen when I wrote each note. Treat 09_7–09_9 findings as the least re-verified.

**Not covered:** continuous motion between samples, audio, Oliver timing, 4K appearance. Colour and legibility judgements are at 720p. The layout lint's TSV is an input, not a pass: 07's is empty, yet 07 has a real axis-through-text collision.

## Author and reviewer overlap

LavenderPike wrote no source in this film. Agent Mail search finds only my apportionment handoff under my name for "complex". The chapters came from BronzeJaguar's lane, the Clippy repair from WhiteSwan's, and integration from root.

That makes this independent on authorship only. Notes from those lanes share a memory directory on this account, and I saw them as recalled context. I also read the complex reading-row rule earlier for the apportionment plan, and the lanes share a model family. I read BronzeJaguar's 02–05 review only after forming these findings, to see whether the arrow cause was already known. That review notes the symptoms (its 02-B, 04-E, 05-E) but does not name the `buff` default.

## Binding

Preview01 `inputs_sha256` starts `f1a136fb7957e550`, frames `inputs_sha256` starts `8e27e5cdea75e1b4`. Canonical source equalled the preview's exact input paths when I checked; the crate files are untracked or uncommitted, so re-`cmp` before relying on a line number.

| Chapter | mp4 | cues | layout | sampling |
|---|---|---|---|---|
| 06 | `b3a98e5c82e818a4` | `84a952f33095ae2a` | `726f710f84a44525` | `6735a3464ed55af4` |
| 07 | `2c3a2c23b61ab187` | `faeb7dcac9f3b8dd` | empty | `988b5dda6a9f2a7d` |
| 08 | `163a3455f2749192` | `4ba82e269f26e8a9` | empty | `b926593153afbef7` |
| 09 | `1ee054d5a97c5ebf` | `760c12fdf887158f` | `a90a18974c6a87c6` | `1e9e0f29e389f99c` |
| 10 | `8c91a7df436d9634` | `ed20e0c4de7f2323` | `329eeaf67cbcd208` | `0d642abd34007af3` |

Source, by SHA-256 prefix: `one_derivative.rs` `ebeca9722668`, `one_point.rs` `6422b41859f6`, `loop_sums.rs` `bbf8e4bfbd4c`, `boundary.rs` `03880164adef`, `rigidity.rs` `c7d53ccac335`, `draw.rs` `fa1f5ddbc846`, `script.tsv` `2968b0fd7654`, `docs/storyboard.md` `6cc2c04286fe`. Every sheet's hash is in `checks/sheet-hashes.sha256`. Full hashes of these inputs are in `checks/input-hashes.sha256`.

Frames are `f<index>`, sheet k holds frames 4k to 4k+3, under `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-opus-rebuild-frames01/<chapter>/`. Anchors are `src/chapters/<file>:<line>` in the crate.

## Shared findings

### S1. High. Every `Arrow::new` is trimmed 0.25 at each end

Cause, from the engine at the crate's pin `a399d626`: `Arrow::new` defaults `buff` to `MED_SMALL_BUFF`, 0.25 (`crates/fmn-config/src/default_config.yml:102`). `crates/fmn-library/src/line.rs:623-627` moves both ends inward by it. So an arrow of length L is drawn L − 0.5 long, and for L < 0.5 it **reverses**. `Arrow::vector` uses `.buff(0.0)`, and `.buff` is public (line.rs:516-517). The crate has 29 `Arrow::new` sites and no `.buff(` call.

`checks/arrow_geometry.py` computes the cases from the crate's constants:

| Where | Anchor | Asked length | Drawn | On screen |
|---|---|---|---|---|
| 08_1, 08_3 Δz_k and product | `loop_sums.rs:151` | 0.63 | 0.13 | Bare head, about 11 px at 720p |
| 08_2 chain | `draw.rs:967` via `loop_sums.rs:315` | 0.78 | 0.28 | Separate arrows with 45 px gaps |
| 08_3 column | `draw.rs:967` via `loop_sums.rs:368` | 0.20 | reversed 0.30 | Hairline |
| 08_4 row | `draw.rs:967` via `loop_sums.rs:466` | 0.12 | reversed 0.38 | Star rosette |
| 08_5 circulation tip | `loop_sums.rs:207` | 0.14 | reversed 0.36 | Stray stick, head clockwise against an anticlockwise γ |
| 08_5 shared walks | `loop_sums.rs:559` | 0.80 | 0.30 | Stubby |
| 07_5 steps | `one_point.rs:480,495` | 0.81 | 0.31 | Detached stub |
| 06_1 step | `one_derivative.rs:138` | 0.86 | 0.36 | Detached stub, head larger than shaft |
| 06_1 change | `one_derivative.rs:144` | 2.12 | 1.62 | Detached from its dot |
| 06_9 orientation head | `one_derivative.rs:429` | 0.11 | reversed 0.39 | Smudge at f380 |

**Correction:** add `.buff(0.0)` at each site in scope. `chain_arrows` (`draw.rs:960-967`) is also called by `residues.rs:137`, so it needs its own patch with callers listed. Other sites are outside this review: `closing.rs:95,117`, `two_planes.rs:250`, `continuing.rs:602`, `gallery.rs:402,410,474,497`, `reading_color.rs:39`, `real_integral.rs:203,207,529,530,536`, `the_bump.rs:371`, `turning.rs:167`, `residues.rs:122`. Whoever owns them should check whether the gap was intended, since a gap off a dot may be deliberate on a long arrow.

Even with `.buff(0.0)`, heads on very short arrows are small: 5 px at 720p for the 08_4 row. Check that on a render.

### S2. Medium. Colour plates are stair-stepped with raster banding

Seen at 07 f89 to f188 (37.0 to 80.5), 09 f20 (6.0), 10 f152 (65.5) and later plates. `CELL = 0.035` (`draw.rs:523`) is about 3 px at 720p and about 9 px at 4K, in row strips. Phase lines step and fine centres alias, as in the `|z|²` picture's centre. Halving `CELL` quadruples cells and repaint cost, which affects every chapter using `Painter`. This is root's call on a real 4K sample. I propose no patch.

## Chapter 06

Cues (scratch start): 06_1 0.600, 06_2 23.167, 06_3 39.167, 06_4 63.267, 06_5 81.200, 06_6 104.133, 06_7 119.367, 06_8 137.700, 06_9 155.533.

| ID | Sev | Cue | Frames | Problem | Anchor | Proposed correction |
|---|---|---|---|---|---|---|
| 06-A | High | 06_3 | f80 to f126 (35.5 to 55.5) | Nothing changes for 16.8 s of voice, from the cue's start at 39.167 until the square at f127 (56.0). "Moves every small arrow the same way" and "multiplies it by the derivative" have no picture | `one_derivative.rs:215-222`; `wait_sentence(4)` at :216 | Start earlier, keyed to sentences: at "the function moves" a fan of small blue arrows at z₀ and their green images, each the blue one scaled and turned by f′(z₀); at sentence 3 "stretch × turn"; keep the square at "So a tiny square" |
| 06-B | High | 06_5 | f207 to f222 (81.6 to 86.0) | Empty screen under "Now write the rule in coordinates. Call the output's parts u and v". `f = u + iv` never appears, yet rows use u and v | `one_derivative.rs:270-288` | At the cue's start show `f = u + iv`, keep it above the rows |
| 06-C | High | 06_8 | f365 to f376 (138.2 to 142.5) | Empty screen under "Reflect the plane". The map `x + iy → x − iy` never appears, though the prediction depends on it. Axes arrive at f377 (143.0) | `one_derivative.rs:324-340`; `wait_sentence(2)` at :334 | Draw the mirror plane at the cue's start; show the map formula at "It sends", keep the square's flip where it is |
| 06-D | Medium | 06_7 | f315 to f331 (120.0 to 126.5) | No change for 7.5 s under "In matrix form". The symbolic Jacobian in the storyboard never appears. The caption "a stretch and a turn" lands at f346 (132.5), 13 s after those words. `= 2 + 2i` and the storyboard's morph never appear (f332 to f362) | `one_derivative.rs:312-320` | Symbolic grid at the cue's start with the caption; numbers at sentence 2; `= 2 + 2i` at sentence 3 |
| 06-E | Medium | 06_1 | f25 to f53 (11.0 to 23.167) | Frozen 12 s under "As h shrinks to zero". Arrows are unlabelled though the formula names h. `f(z) = z²` appears nowhere in the chapter | `one_derivative.rs:115-151` | Add the map label; label h and the output change; shrink h and its image at "As h shrinks", keeping them above the kit's floor |
| 06-F | Medium | 06_2 | f55 to f79 (23.5 to 35.0) | The teal inset has no title, axes or number for 11.5 s. The storyboard's live per-path readout never appears; one result lands at f79 (35.0). Trails cover the white target | `one_derivative.rs:160-211` | Title "the ratio", a live readout during each path, target drawn above the trails |
| 06-G | Medium | 06_3, 06_4 | f127 to f186 (56.0 to 76.5), f127 to f204 (56.0 to 81.2), f180 (75.2) onward, f74 (32.5) | Three layout findings confirmed on frames: blue square through "+ i" of `z_0 = 1 + i`; green and teal through `z_0^2 = 2i`, never freed; axis through "zoomed in about 2i". The spiral also crosses the first label | `one_derivative.rs:121-131`, `:239-240` | Move names clear of the drawn square and spiral; move the zoom caption off x = 0 |
| 06-H | Medium | 06_5, 06_6 | f224 to f331 (86.9 to 126.5) | Formula rows sit at the top with the lower half empty. `u_x` is never glossed. No link from the rows to the box, and the box at f277 (106.5) arrives whole while the voice gives one equation at a time | `one_derivative.rs:289-305` | Gloss "u_x: rate of u along x"; lower the block; bring the two equations in on their sentences |
| 06-I | Medium | 06_8, 06_9 | f380 (144.5) | Orientation mark is a thin arc with a smudged head, so "reverses every little square" cannot be read, and nothing animates the reversal | `one_derivative.rs:419-436`, Arrow at :429 | S1, larger mark, and a mark that sweeps on the blue square and the opposite way on the green one |
| 06-J | Low | 06_9 | f444 (168.5) | `x + 2iy` never shown, grey caption low contrast, red X unlabelled | `one_derivative.rs:366,386` | Show the map; raise contrast; label "no complex derivative" once |
| 06-K | Low | all | f79 (35.0), f332 (126.9) | Three-decimal readouts where the voice says "two"; upright i | `one_derivative.rs:204-211` | Judgement call for root; facts must stay pinned |

Sound: three-path trails, the curved green image against the teal square, the shrink converging at f180 to f193, the names card, and all numbers. **Prediction 06_8 → 06_9 passes:** ring from f398 to f409, first answer f415 (156.5).

## Chapter 07

Cues: 07_1 0.600, 07_2 18.167, 07_3 34.833, 07_4 49.967, 07_5 65.967, 07_6 89.667.

| ID | Sev | Cue | Frames | Problem | Anchor | Proposed correction |
|---|---|---|---|---|---|---|
| 07-A | Medium | 07_1 | f17 to f29 (7.0 to 13.0) | Both rays are one green on one axis, so they read as one line. Which blue line goes to which ray is not shown, though the voice says "two rays" | `one_point.rs:249-285` | Colour each blue line and its ray as a pair; label "real line", "imaginary line" or mark the two ray ends |
| 07-B | Medium | 07_1 | f33 (15.0) to wipe | "180°" sits on the right panel's vertical axis, which runs through it. The layout TSV is empty, so lint does not catch this | `one_point.rs:322`, label at `C::new(0.0, 0.5)` | Move the label off x = 0 |
| 07-C | Medium | 07_3, 07_5 | f91 (38.0), f159 (66.467) | The dot at 0 and the point 1 are unlabelled, though the voice says "at zero" and "the point one". No axis or scale label on the painted plane | `one_point.rs:390`, `:475` | Label "0" and "1" on backing |
| 07-D | Medium | 07_5 | f163 (68.0), f168 (70.5) | Step arrows are stubs detached from their dot | `one_point.rs:480,495` | S1 |
| 07-E | Medium | 07_5 | f191 (82.0) onward | The formula uses `M`, never defined; the caption says `|g| ≤ 1` | `one_point.rs:41-42` | Say what M is, e.g. "`|g| ≤ 1 = M`" in the caption. `the_jagged_multiplier_is_bounded_with_no_limit_at_zero` checks that `|g| ≤ 1` stays printed |
| 07-F | Low | 07_6 | f223 (95.0) onward | Red dot and "|z|²: one point, no disk" are about 100 px apart and dim | `one_point.rs`, `lone` at v(2.0, −2.4), `LONE` at v(4.1, −2.4) | Set the caption left-aligned just right of the dot |
| 07-G | Low | 07_5 | f173 to f188 (73.0 to 80.5) | 7.5 s still; squeeze first moves about 2.5 s after it appears, f198 (85.5) | `one_point.rs:516-573` | Judgement call; both follow the words |
| 07-H | Low | 07_2 | f51 (21.0) | Names card shows "critical point" about 2.7 s before the voice. Voice-dependent, recheck under Oliver | `one_point.rs:362-363` | Show "conformal map" first, or hold to sentence 2 |

Sound: conformal cross and image; precisely card about 10 s for 103 chars, correct; ratio inset with the conjugate direction f132 to f147; shrinking cloud f193 to f205 (83.0 to 88.959), the best demonstration here; disks inside U f218 to f224; definition and names cards f237 to f239; "from here on" f245 to f246. **Prediction 07_3 → 07_4 passes:** ring f108 to f117 (45.0 to 49.5); answer f123 (51.0), after 07_4 starts.

## Chapter 08

Cues: 08_1 0.600, 08_2 22.000, 08_3 36.100, 08_4 54.800, 08_5 66.200. No prediction.

| ID | Sev | Cue | Frames | Problem | Anchor | Proposed correction |
|---|---|---|---|---|---|---|
| 08-A | High | 08_1 to 08_5 | f19 to f54 (8.0 to 23.5), f54 to f84 (23.5 to 36.3), f105 to f127 (46.0 to 55.0), f130 to f156 (56.0 to 66.5), f180 to f204 (78.0 to 90.0) | S1. "Lengths multiply, angles add" and "points straight up" rest on 11 px heads (f22). "Tip to tail" is separate arrows with gaps (f61). "Stack into a column" shows a hairline (f112). "Always close up" shows star rosettes (f143). Circulation marks read clockwise under an anticlockwise γ (f190) | `loop_sums.rs:151,207,283,521,559`, `draw.rs:967` | S1. Check the circulation mark's head and the row's heads on a render; the mark may need a larger drawn arc |
| 08-B | Medium | 08_2 | f61 to f69 (27.0 to 31.0) | The second lap redraws the first, so the picture stops while the readout runs. The left shows 24 steps, the right 12 arrows, and nothing says the chain goes round twice (storyboard: "doubled polygon") | `loop_sums.rs:315-342` | Say it on screen ("round twice") and mark the lap, without covering the chain |
| 08-C | Medium | 08_2 | f75 (34.0) onward | "|total| = 0.000000" sits under the stale label "distance from the start:" | `loop_sums.rs:333-342` | Fade the label when the total arrives |
| 08-D | Medium | 08_1 | f0 to f15 (0.0 to 6.0), f36 to f37 (16.5 to 17.0) | Axes and unit circle unlabelled; no direction of travel under "Walk around"; Δz_k does not shrink when the steps do, and the gold circle goes under the beads | `loop_sums.rs:283-297` | Label the plane and circle; a direction arrowhead; shrink or drop Δz_k as steps multiply |
| 08-E | Low | 08_1 | f23 to f30 (10.0 to 13.5) | Still under "Then add all those products" | `loop_sums.rs:288-290` | Accept: the adding happens in 08_2. Root's call |
| 08-F | Low | 08_3 | f91 to f103 (39.0 to 45.0) | Still 6 s, acceptable once the arrow is visible | `loop_sums.rs:357-368` | Recheck after S1 |
| 08-G | Low | 08_5 | f214 (94.0) onward | Double colon in "precisely: Goursat's lemma:", orphan "assumed" on a wrap; upright f in the hypothesis caption | `loop_sums.rs:581`, `GOURSAT` | Reword the card, not the voice |

Sound: left trace thickening with the chain; growth f116 to f120 with "the same column"; 08_4 saving z⁻¹ for last, box and 2πi at f143 (62.5), "only 1/z survives" at f144 (63.0); inner edges fading at f204 (90.0); precisely card after the voice, f215 (94.108) on.

## Chapter 09

Cues: 09_1 0.600, 09_2 26.233, 09_3 38.800, 09_4 56.367, 09_5 71.867, 09_6 93.967, 09_7 118.833, 09_8 144.833, 09_9 158.667.

| ID | Sev | Cue | Frames | Problem | Anchor | Proposed correction |
|---|---|---|---|---|---|---|
| 09-A | Medium | 09_1 | f12 to f45 (3.0 to 17.4), f46 (17.5) | The split arrives before "divide it by", then nothing moves for 14.4 s, no term lit, right half empty. The caption at f46 has no mark on the term it names (storyboard: second piece marked "holomorphic → 0") | `boundary.rs:158-162` | The quotient alone at "divide it by"; split and first term lit at sentence 2 ("Near a"); mark the second term at sentence 3 |
| 09-B | Medium | 09_5 | f193 (73.5) to f243 (90.344) | Magenta "−0.5" on a rose plate is unreadable for the whole hold (f200 at native size), and it arrives one sentence before "Each pole pulls" | `boundary.rs:263-271` | Move the flash after `wait_sentence(2)`; ink on a dark card, as the zero and pole marks use |
| 09-C | Medium | 09_6 | f260 (94.6), f261 (95.0), f282 (103.0), f311 (113.5) to f328 (118.833) | `c_n` uses undefined `avg_k` and `θ_k` (storyboard: "average of", θ). Panel has no title or "1" mark. "Slope" never appears at `c_1`. No event for "Every derivative at the centre is hidden in the boundary values" | `boundary.rs:39`, `:298-303`, `:345-353`, `:370-381` | "average of" in the formula; panel title and "1"; "the slope at a" at `c_1`; pulse the ring and samples on the last sentence |
| 09-D | Medium | 09_7 | f333 to f362 (120.0 to 131.5), f363 (132.0), f374 (136.0) to f399 (144.433) | Graph, curve and poles carry no labels. 11.5 s unchanged across "largest disk" and "opening puzzle". The radius-1 circle only fades in, with no "radius 1". No −1, 1 on the bar. "No further" has no picture of its own | `boundary.rs:393-431` | At "the largest disk" grow the circle from the centre until it hits the poles (same device as 09_9); label ±i, "radius 1", −1, 1, "bump", "recipe" |
| 09-E | Medium | 09_9 | f452 to f460 (160.0 to 163.2), f477 (169.5) | The growth is the chapter's best picture, but no radius segment, "√2" or ±i, though the storyboard has `recentre1.radius`. "60 terms at 2: 0.200000" and "at 2.6: 212" compare with nothing | `boundary.rs:464-497`, `:509-527` | Radius segment and √2 label from the fact; on the rows say "bump at 2: 0.2" and "2.6 is outside" |
| 09-F | Medium | 09_3 | f105 to f114 (39.5 to 43.5), f115 (44.0) | Empty panel 4 s; radius ½ unlabelled; "a" stays though the voice says "around zero"; bump unnamed. 8 samples but 4 dots in the panel, unexplained (same at radius 2: 16 and 8). The even function folds them, which a viewer cannot guess | `boundary.rs:186-216` | "r = ½", "a = 0", "bump"; a small note that samples pair up |
| 09-G | Low | 09_4 | f148 (56.867) onward | Two gold rings unlabelled; no "f(0) = 1" | `boundary.rs:238-239` | Dim the small ring; "r = 2" |
| 09-H | Low | 09_4 | f152 (58.0) to f184 (71.5) | The panel's cloud sits near 0 during the think time. No number shown. A fair clue; root's call | `boundary.rs:240` | None proposed |
| 09-I | Low | 09_5 | f220 (83.2) onward | Precisely card covers the plate's foot by about 25 sheet px; hides no sample. Wording correct | `boundary.rs:56`, `:287` | Shift the plate up or shrink it; check test tolerances |
| 09-J | Low | 09_8 | f405 to f408 (145.5 to 147.0) | The ring collapses just before the true radius grows to √2; no mark at x = 1. The storyboard asks for the shrink. The lint's hit under "a" here is this transient | `boundary.rs:438-456` | Plain fade once the dot leaves; label the point 1. Storyboard row needs the same edit |
| 09-K | Low | 09_7 onward | f333 (120.0) to f401 | Left spike tops out lower than the right, from a grid not symmetric about 0 | `boundary.rs:107-108`, `n = 600` | `n = 920`, step 0.005; check test near :631 |
| 09-L | Low | 09_1 | f9 to f10 (2.5 to 2.6) | Dim outer circle unlabelled ("a little beyond"). Lint's hit under "a" at f8 is a false positive on legibility (f20) | `boundary.rs:153-157` | "a little beyond", small |

Sound: f268 to f275 (97.5 to 100.0) the cloud turning with its ring sliding to 0, which is excellent; 8 then 16 samples with readouts; f212 (80.0) tally; precisely wording; numbers. **Prediction 09_4 → 09_5 passes:** card f161 (62.4), ring f174 to f184 (67.0 to 71.5), answer ring f188 (72.367), readout f191 (73.0). **Prediction 09_8 → 09_9 passes:** ring f428 to f446 (152.4 to 158.5), answer ring from f449 (159.0).

## Chapter 10

Cues: 10_1 0.600, 10_2 17.767, 10_3 43.767, 10_4 64.400, 10_5 83.767, 10_6 109.433.

| ID | Sev | Cue | Frames | Problem | Anchor | Proposed correction |
|---|---|---|---|---|---|---|
| 10-A | Medium | 10_4 | f152 to f165 (65.5 to 72.0), f196 (84.5) | The plate is alone for 6.5 s under the first sentence, with no `sin(1/z)`, axes or scale, and the right half empty. The first on-screen `sin(1/z)` is f196, in the next cue. Zeros arrive at f166 (72.5) after the voice has listed them | `rigidity.rs:371-385` | Name the plate as it arrives; add Re, Im; zeros one at a time on the sentence's own words |
| 10-B | Medium | 10_6 | f257 to f274 (111.0 to 119.0) | The new plate has no name, axes or scale for 8 s. The pinwheels at 0, ±π are unlabelled | `rigidity.rs:424-428` | Name it `sin z`; label 0, ±π, and Re, Im |
| 10-C | Medium | 10_6 | f277 to f296 (120.5 to 128.5) | Static 8 s. "Must grow huge somewhere else" has no growth. Dot not labelled 3i, though 10_2 labels it at :259 | `rigidity.rs:437-454` | Walk the dot up from the real line to 3i with a live \|sin z\|, ending on the 10.0 reading; label 3i |
| 10-D | Medium | 10_2 | f45 to f47 (18.267 to 19.0), f55 to f65 (23.0 to 28.0), f71 to f77 (31.0 to 34.0) | Blank square with no Re, Im or ticks; then 5 s and 3 s unchanged while the voice builds the argument | `rigidity.rs:224-225`, `:237-257` | Label the plane; mark the real line "agree here"; keep the good band growth at f67 to f70 |
| 10-E | Low | 10_5 | f195 (84.267) onward | The red X is about 60 sheet-px on a 378 sheet-px plate and hides the innermost zeros, the very pile-up | `rigidity.rs:404`, `:502-511` | Smaller arms, or set off from 0 with a pointer. `d` at :502 is local to this file |
| 10-F | Low | 10_3 | f103 to f110 (44.267 to 47.5), f120 to f127 (52.5 to 56.0) | Bare axes; "yet it is not zero" gets no mark | `rigidity.rs:300-309`, `:324-330` | Label x; mark a point where the curve leaves 0 |
| 10-G | Low | 10_1 | f3 to f16 (0.8 to 6.5), f26 (11.5) | f, g and U's agreement unlabelled at first; a green margin stays uncovered under "agree everywhere" | `rigidity.rs:172`, `:193-198` | Final fill of U on the last sentence's words |
| 10-H | Low | 10_5 | f212 (92.2) onward | Precisely card overlaps the plate by about 22 of 378 sheet px, 6%; text readable | `rigidity.rs:140`, `:411` | Lift the plate slightly; check its tests |
| 10-I | Low | 10_2 | f80 (35.5) onward | "sum = 1.000000" and formula left in x | `rigidity.rs:30`, `:262-289` | Root's call |

Sound: overlapping disks f20 to f26 (8.5 to 11.5), very good; band growth f67 to f70 (29.0 to 30.5), excellent; seven red gaps f129 (57.0); theorem card with gold condition f210 to f211 (91.5 to 92.0); sin(3i) ≈ 10.0 i is correct. **Prediction 10_4 → 10_5 passes:** ring f181 to f191 (78.5 to 83.5), first answer f195 (84.267). The reading hold after 10_5 ends at f230 (100.321) runs to f250 (109.433), 9.1 s against the storyboard's 8.5; authored, low.

## Limits

This is a picture review from sampled frames. It cannot judge motion quality, audio, or anything timed to the voice beyond scratch. All proposed corrections are unrendered here. Any source change stales the direction map's hashes, so root must rebind after integrating. The next step, per Mail 1654, stages native repairs in a separate folder; those repairs are mine and are not accepted by this review.

## Reproduce

From the repo root: `python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-preview01-review-06-10/checks/check_review.py`. It checks every frame and time pair against `sampling.json`, the severity counts, the sheet hash list, the input hash list, and that this report has no absolute home path. `checks/arrow_geometry.py` prints the S1 table's numbers.
