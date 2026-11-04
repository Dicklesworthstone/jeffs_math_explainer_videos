# Complex chapters 06–09: picture review of preview02

WhiteSwan, for BlueSalmon Mail 2135, bead `.39`. Date: 2026-11-04. Narration stays ElevenLabs Oliver; this review touches no narration.

**Status: sampled pictures only.** I read 381 contact sheets as images and opened 15 original frames. Nothing was watched or heard. Preview02 uses the scratch voice, so every time below is a scratch-voice time. This is not a final AV review and can't stand in for one.

**Verdict: no High defect is left, but I would fix five small things before these chapters go to 4K.** The numbers and formulas on screen held everywhere I checked them: `checks/number_check.py` recomputes the readouts, and all 85 checks pass. Of LavenderPike's 40 findings for these chapters, 27 are fixed. Five of the six Medium findings are new, and three of those come from labels the repair added (06-N1, 07-N1, 09-N2).

| Severity | Count | Meaning here |
|---|---|---|
| High | 0 | The screen says something false and nothing in the scene corrects it, or a claim the voice depends on can't be seen at all |
| Medium | 6 | The viewer is misled or loses a needed picture for several seconds, but can recover |
| Low | 40 | Polish, a style call, or a timing that only Oliver can settle |

## The six Medium findings

| ID | Frames | What the viewer gets | Proposed correction |
|---|---|---|---|
| 06-N1 | f442 (167.5) to f464 (175.35), 06 | The heading says doubling, `x + iy ↦ x + 2iy`, but the column beside it still says `up: −1` from the reflection | Fade the reflection's numbers at `one_derivative.rs:680`; move the new readout up |
| 07-N1 | f3 (0.8) to f84 (35.033), 07 | The map arrow's tail runs through the `e` of `Re` | `one_point.rs:238`: start the arrow at `v(-0.42, -0.15)` |
| 08-N1 | f88 (37.5) to f103 (45.0), 08 | The voice talks about "the turn of the step's position", and no position is marked on the ring | Bring the `z_k` dot and label back at `loop_sums.rs:443` |
| 08-N2 | f180 (78.0) to f204 (90.0), 08 | Sixteen circulation marks point the right way by measurement, but read as triangles pointing down, which looks clockwise | `loop_sums.rs:213`: `CIRCULATION.1 - 0.62` |
| 09-N2 | f223 (80.0) to f272 (94.167), 09 | The lower `−0.5` card covers the bottom half of `a = 0` for 14 s | `boundary.rs:284`: `p.p(C::new(1.05, -0.4))` |
| S2 | 07 f88 (36.5) to f189 (81.0); 09 f3 (0.8) on | Colour plates are stair-stepped | Root's call: `draw.rs:554`, no hunk proposed |

These are proposals. All corrections are desk-computed, uncompiled and unrendered. None removes a step of the mathematics or changes a cue word, so the film keeps its depth and the script stays at `2968b0fd7654`. Root decides and applies; nothing here is installed.

## Author overlap

This review is independent of LavenderPike's review and repair in authorship only:

- **The native chapters.** BronzeJaguar's lane wrote them. That lane's notes share this account's memory directory, so I can't call this review independent of it. LavenderPike's review also notes that the lanes share a model family.
- **My earlier complex work.**
  - `complex/clippy-repair05/` (Mail 1385) changed six files, including `one_point.rs` and `draw.rs`.
  - `complex-glyphs02-review01/` found the glyph items reported below as G1 to G3.
  - `complex-glyph-spacing-repair01/` (Mail 1987) is my unapplied `z_{\,0}` patch, so G1 reports on my own patch.
  - `complex-picture-repair-11-15/` shares a `chain_arrows` dependency with chapter 08.
  - I also reviewed the rigidity test-only repair (Mail 1846).
- **Anchoring.** I knew LavenderPike's findings and the repair's claims before I read the sheets, so I was looking for them. I may have looked less hard elsewhere.

## Coverage, stated exactly

| Chapter | Sheets read | Frames on them | Reads per sheet | Duration (s) | Last sample (s) | Hash of sheet hashes |
|---|---|---|---|---|---|---|
| `06_one_derivative` | 000–116 (117) | f0–f464 (465) | 1 | 175.6 | 175.35 | `180f8aa8285637a857bb8413d295fa84f7630e375ad20efe7bbe57b87ddefae2` |
| `07_one_point` | 000–065 (66) | f0–f261 (262) | 1 | 112.8 | 112.55 | `fbe34547c4d48decbbb02b5ebe95b08394cb50893fffb9709c105091d6ebd202` |
| `08_loop_sums` | 000–057 (58) | f0–f231 (232) | 1 | 102.133 | 101.883 | `ab45cfb27d43f15c51354ea562f100f6692fe8b1f70d028e6fe3d339d01fce19` |
| `09_boundary` | 000–139 (140) | f0–f558 (559) | 000–023 and 081–139 once; 024–080 twice | 176.133 | 176.0 | `356315b4e48b34e9eacf97e11ecb0431b725b52756b802d620cb34e2cc261452` |
| **Total** | **381** | **1,518** | | | | |

- **Hash of sheet hashes:** SHA-256 over the sheets' hex digests, joined in sheet order. `checks/sheet-hashes.sha256` lists every sheet with its path and hash.
- **Layout:** sheet k holds frames 4k to 4k+3. The last sheets of 06, 07 and 09 are partly blank by construction.
- **Source of the sheets:** `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/`.

Originals opened to settle ambiguity. `checks/input-hashes.sha256` has their hashes.

| Chapter | Frame | Time (s) | Opened for |
|---|---|---|---|
| 06 | f74 | 32.5 | 06-F and 06-G: spiral readout, `z_0² = 2i` clear of the axis |
| 06 | f125 | 55.0 | G1 and 06-K |
| 06 | f130 | 57.0 | 06-G: label moved clear of the image square |
| 06 | f192 | 78.0 | 06-N5 |
| 06 | f450 | 171.0 | 06-N1 |
| 07 | f10 | 3.5 | 07-N1 |
| 07 | f195 | 84.0 | G3 and S3 on the caption |
| 07 | f206 | 89.0 | 07-N2 |
| 07 | f230 | 98.5 | Disks inside U |
| 08 | f34 | 15.5 | G3 and 08-N5 |
| 08 | f92 | 39.5 | 08-N1 |
| 08 | f186 | 81.0 | 08-N2 (8× crop), G1 and G3 (4× crop); crops not kept |
| 09 | f120 | 45.8 | 09-N1 |
| 09 | f240 | 85.0 | 09-N2, 09-N1, 09-I |
| 09 | f535 | 170.0 | `the bump` beside the right spike |

### A coverage problem you should know about

Image results older than about the newest 62 dropped out of my working context as new ones arrived. I noticed this while checking my notes after the first pass. So the reads were not equal:

| Sheets | Passes | What I can say |
|---|---|---|
| 09 024–080 | 2 | The first-pass images had gone by the time I checked my notes. I re-read these sheets in batches of 8–9 with notes after each batch. No first-pass finding changed |
| 09 081–139 | 1 | Still visible at the end of the first pass |
| 06, 07, 08, and 09 000–023 | 1 | Read once, with notes after each batch. Not re-read |

The session transcript shows an image row for all 381 sheets. That proves the reads happened, not what was on screen when I wrote each note.

**What stays weaker:** on the one-pass sheets, a small defect I missed has had no second chance. The findings there rest on reopened originals, so they are better supported than the absence of other findings is. `PROGRESS.md` holds the batch notes as written; its last sections qualify and correct them.

### Not covered

- **Motion between samples.** Samples are half-second, cue-boundary and layout-anchor stills, and later ones may be nonconsecutive. The fold, the turning dots and the growing circles were read as stills.
- **Sound.** I heard nothing, neither the scratch voice nor music. No Oliver audio exists in this render.
- **Oliver fine sync.** Every "on the words" below means scratch voice. All of it needs rechecking under Oliver, including the prediction windows.
- **4K appearance.** The source is 1280×720, CRF 26, ultrafast, and most reading was at 960×540 per sheet cell. Thin strokes, compression and S2 may look different in the final.
- **Anything outside chapters 06–09.**
- **The build link.** No hash ties the render to these sources. Source mtimes (14:28:45Z) precede the binary (15:15:32Z) and the preview receipt's start (15:50:12Z), and the pictures agree with the source at every anchor I read. Mail 2135 reports that preview02 and the extraction finished with exit 0; I did not verify that separately.

## How to read the times

Frames are `fN`, and the bracketed time comes from that chapter's `sampling.json`. `launch-preview02.py` passes `--scratch-voice`, so all times will move under Oliver.

Cue starts and ends come from `cues.tsv`. Times for single words are **not measured**. They are word-share estimates: cue start + cue length × (words before ÷ words in the cue). They can be at least a second off: 06_9's fade is keyed to "Squashing fails too" and starts at 167.0, against an estimate of 168.1. So treat a lead or lag under about 2 s as unproved.

| Cue | Words | Estimate (s) |
|---|---|---|
| 06_1 | "Step from z" | 9.6 |
| 06_1 | "divide the change" | 13.5 |
| 06_1 | "As h shrinks" | 16.7 |
| 06_1 | "settle" | 19.8 |
| 06_5 | "which turns the answer" | 98.4 |
| 06_9 | "A reflection reverses" | 162.6 |
| 06_9 | "Squashing fails" | 168.1 |
| 07_2 | "critical point" | 26.9 |
| 07_6 | "small disk" | 100.3 |
| 08_3 | "dividing by z" | 38.9 |
| 08_4 | "survives" | 61.6 |
| 09_3 | "Eight sample" | 44.3 |
| 09_5 | "Each pole" | 80.1 |
| 09_6 | "the next coefficient" | 111.6 |
| 09_7 | "radius one" | 139.5 |

## Shared findings

| ID | Sev | Status | Frames | Finding | Anchor | Correction |
|---|---|---|---|---|---|---|
| S2 | Medium | Persists | 07 f88 (36.5) to f189 (81.0); 09 f3 (0.8) on | Colour plates are stair-stepped at band edges. The cell is a fixed scene size, about 3 px at 720p, so the steps stay the same size on a 4K screen | `draw.rs:554` `CELL = 0.035`; cell counts `:575-578` | No hunk. Halving `CELL` quadruples the cell count, so root should judge the cost |
| S3 | Low | New | 06 f224 (86.9); 07 f195 (84.0); 08 f162 (69.0); 09 f106 (40.0) | Variables on Text cards are upright beside italic maths: `u`, `x`, `i` in 06; `m`, `U`, `z`, `g`, `M` in 07; `f`, `f′`, `i` in 08; `f`, `a`, `r`, `i` in 09, including roman `a = 0` beside italic a | Text rows | Style call: set these rows in TeX, or accept |
| G1 | Low | Persists | 06 f125 (55.0); 08 f186 (81.0) | The `z_0` subscript touches the z, seen in 06 and 08. This is my F1 | `one_derivative.rs:26`, `:44`, `:47`, `:211`, `:217` | My patch in `complex-glyph-spacing-repair01/` is unapplied |
| G2 | Low | Persists | 09 f71 (26.7) on | Open θ. This is my F2 | `fmd-math/src/commands.rs:178`, fmn pin a399d626 | Style call |
| G3 | Low | Persists | 07 f195 (84.0); 08 f34 (15.5), f92 (39.5), f186 (81.0); 09 f240 (85.0) | Heavy `∮`, `−`, `≈`, `×`, and heavy `²` on Text rows. These are my F4 and F5 | `one_point.rs:42`, `:51` | Style call |

S1 (prior High, `Arrow::new` trimmed at each end) is **fixed** on every sampled use in 06_1, 07_5 and 08_1 to 08_4. On 08 f20 (8.5) the Δz_k arrow measures 142° against 142.5° expected, and the product 42 sheet px against 41 expected.

## Chapter 06: `one_derivative.rs`

### New

| ID | Sev | Cue | Frames | Problem | Anchor | Correction |
|---|---|---|---|---|---|---|
| 06-N1 | Medium | 06_9 | f442 (167.5) to f464 (175.35), 7.9 s | The heading is now `x + iy ↦ x + 2iy`, but the column beside it still shows `right: 1`, `up: −1`, a red X and `no complex derivative` from the reflection that has just left. Neither readout names its map. For doubling, up is 2, so the nearest number contradicts the heading until the eye reaches `right: 1   up: 2` lower down. Original f450 confirms it: blue height 90 px, green 183 px, ratio 2.03 | `:680`, `:691`, `:715`, `:725`, `:726`, `:732` | See below |
| 06-N2 | Low | 06_3, 06_4 | f118 (51.5) to f204 (81.2) | The planes have different scales and nothing says so: a right-plane unit is 0.75 of a left one. `stretch 2.83` therefore draws at 2.12, and the viewer can't check it by eye | `:67-68` | A one-unit tick on each plane, or the same range on both. Root's call |
| 06-N3 | Low | 06_5 | f239 (93.0) to f257 (100.5) | "which turns the answer back a quarter turn" has only a formula. No turn is drawn | `:544` | A small quarter-turn arc beside the row, or accept |
| 06-N4 | Low | 06_4 | f153 (63.5) to f179 (75.0) | Nothing moves for 11.5 s. It is on topic, with `2 + 2i` and the stretch readout on screen | None | Accept |
| 06-N5 | Low | 06_4 | f192 (78.0) | At the end of its shrink the blue square is about 22 px and its grid strokes keep their width, so it reads as a blob with four pinholes. Transitional | None | Accept |
| 06-N6 | Low | 06_3 | f113 (49.0) on | `× f'(z_0)` floats under `→ f'(z_0)`, with nothing tying it to the fan. Busy but readable | None | Accept |

**06-N1 correction.** At `wait_sentence(4)` ("Squashing fails too"):

1. Add `r1`, `r2` and both `cross1` parts to the fade at `:680`.
2. Take them out of the teardown at `:725`, `:726` and `:732`.
3. Keep `no_derivative`, which is just as true of doubling.
4. Move `s1` (`:691`) to `v(4.2, 1.3)` and `cross2` (`:715`) to `v(4.2, 0.3)`, so the new readout sits above the kept label.

If root prefers to keep both readouts on screen, label the blocks `reflection` and `doubling` instead.

### Prior findings

| ID | Was | Now | Evidence |
|---|---|---|---|
| 06-A | High | Reduced (Low) | Nothing changes from 39.167 until the fan at f106 (45.5), about 6.3 s, down from 16.8 s. `× f'(z_0)` f113 (49.0), `a stretch and a turn` f118 (51.5) |
| 06-B | High | Fixed | `z = x + iy` from f208 (81.7), `f = u + iv` at f217 (84.0) on "Call the output's parts" |
| 06-C | High | Fixed | Axes f365 (138.2), square f366 to f367 (138.5 to 139.0), Re turns gold f368 (139.5), fold f369 to f375 (140.0 to 142.0), map f377 to f378 (143.0 to 143.5) |
| 06-D | Medium | Fixed, residual (Low) | Symbolic grid f314 (119.867), caption f316 to f317 (120.2 to 120.5), numbers f332 (126.9), `= × (2 + 2i)` f346 to f347 (132.5 to 133.0). Residual: `= ×` reads as an equals sign followed by a times sign |
| 06-E | Medium | Fixed, residual (Low) | h labelled f23 (10.0), `f(z) = z²` named, shrink f35 to f38 (16.0 to 17.5) on "As h shrinks". Residual: nothing moves f6 to f22 (1.5 to 9.5) under the first two sentences; the quotient f25 (11.0) leads "divide the change" by about 2.5 s; `change in the output` stays at its first height while the arrow shrinks away from it, f37 to f53 (17.0 to 23.167) |
| 06-F | Medium | Fixed | Inset titled `the ratio`; right f63 to f67 (27.0 to 29.0), above f68 to f72 (29.5 to 31.5), spiral f74 to f78 (32.5 to 34.5), result f79 (35.0). Target stays on top |
| 06-G | Medium | Fixed | On original f130 (57.0), `z_0 = 1 + i` clears the blue square and `z_0² = 2i` clears the image. On original f74 (32.5), `z_0² = 2i` clears the Im axis. The zoom note clears the axis on the sheets |
| 06-H | Medium | Partly (Low) | Gloss on `u_x` f224 (86.9), equations arrive apart f277 (106.5) and f288 (111.0), `must agree` bar f258 to f259 (101.0 to 101.5). The lower half stays empty f223 (86.5) to f277 (106.5) |
| 06-I | Medium | Partly (Low) | Marks now read: blue anticlockwise, green clockwise. Nothing moves f428 (161.9) to f440 (166.9) under "A reflection reverses", and the fold was 20 s earlier. A one-time sweep on both arcs would answer the rest |
| 06-J | Low | Mostly (Low) | `x + 2iy` at f442 (167.5), first X labelled at f428 (161.9). The second X has no label of its own; the 06-N1 correction settles that |
| 06-K | Low | Persists (Low) | Three decimals and upright i at f79 (35.0) and f125 (55.0). Root's call |

### Prediction window

**06_8 to 06_9 holds.** Voice ends 149.638. Ring f398 (150.5) to f409 (155.2), gone f410 (155.5). 06_9 starts 155.533, and the answers start at f415 (156.5). No answer is visible before that.

### Sound

- `2 + 2i`, stretch 2.83, 45°. The fan measures about +47° on f123 (54.0).
- `2.000 + 2.000 i` matches the limit.
- Grid values 2, −2, 2, 2 match my own derivatives of z² at 1 + i.
- Gap readout: 0.088 and 0.0088 from 0.0884 and 0.00884.
- Reflection gives right 1 and up −1. Doubling gives right 1 and up 2.
- The chapter ends cleanly, fading from f463 (175.2).

## Chapter 07: `one_point.rs`

### New

| ID | Sev | Cue | Frames | Problem | Anchor | Correction |
|---|---|---|---|---|---|---|
| 07-N1 | Medium | 07_1, 07_2 | f3 (0.8) to f84 (35.033) | The left plane's `Re` (x 570–592) collides with the map arrow's tail, which starts about x 585 at y 373 and runs through the `e`. Original f10 confirms it. It came from 07-C (Re added) meeting S1 (arrow no longer trimmed). The layout TSV is empty, so lint did not catch it | `:237-243`; `draw.rs:468` | `:238`: start at `v(-0.42, -0.15)`. Tail at x 602, 10 px clear of `Re`. The arrow's midpoint moves 0.09 right of the map label |
| 07-N2 | Low | 07_5 | f203 (88.0) to f210 (89.667) | Fixed-size sample dots saturate. Original f206 shows a 20 px white disk on the plate and a green blob in the inset, with the ring barely visible. The ring is the claim | `:147` and its callers | Shrink dots with rho, or stop near rho 0.12 |
| 07-N3 | Low | 07_6 | f220 (93.5) to f224 (95.5) | Disks grow about 5 s before "small disk" | `:698-714` | Key on `line.until(stage, "small disk", …)`. Recheck under Oliver |
| 07-N4 | Low | 07_4 | f148 (63.5) on | `ratio → 0.0000`: four decimals for a limit that is exactly 0. Kin of 06-K | None | Root's call |
| 07-N5 | Low | 07_4 | f144 (61.5) | The spiral's h segment touches the `0` card, on one sample | None | Accept |
| 07-N6 | Low | 07_6 | f213 (90.167) to f217 (92.0) | Blank for about 2 s under "The strong results ahead need more." | None | Accept as a breath, or bring U forward |

### Prior findings

| ID | Was | Now | Evidence |
|---|---|---|---|
| 07-A | Medium | Fixed | Lines and rays colour-paired and labelled, f17 (7.0) to f29 (13.0) |
| 07-B | Medium | Fixed | `180°` clear of the axis, f33 to f34 (15.0 to 15.5) |
| 07-C | Medium | Fixed | Carded `0` f92 (38.5), carded `1` f161 (67.0), Re and Im present. See 07-N1 |
| 07-D | Medium | Fixed | Step arrows start at the dot, f163 (68.0) and f169 (71.0) |
| 07-E | Medium | Fixed | `|g| ≤ 1 = M` in the caption, f192 (82.5) to f210 (89.667) |
| 07-F | Low | Fixed | Red dot and caption 12 sheet px apart and bright, f224 (95.5) |
| 07-G | Low | Persists (Low) | Still f173 (73.0) to f188 (80.5) and f193 (83.0) to f197 (85.0); squeeze moves at f198 (85.5). Root's call |
| 07-H | Low | Persists (Low) | Names card f51 (21.0) to f55 (23.0), gone f56 (23.5); "critical point" is estimated at 26.9. It is structural: `:387` shows the card and `:390` removes it at `wait_sentence(2)`. Softened because the replacement line at `:389` also says `a critical point`. Move the card to the reading window (`:396`) if root wants it |

### Prediction window

**07_3 to 07_4 holds.** Question f97 (41.0), voice ends 44.079, ring f108 (45.0) to f117 (49.5), gone f118 (49.967), answer f123 (51.0).

### Sound

- Rays and arcs: arc radii 65 and 33 sheet px, as squaring needs.
- The small cross's image lands at (−0.7 + 0.6i)² = 0.13 − 0.84i.
- h̄ mirrors h on every spiral sample checked, f142 (60.5) to f145 (62.0).
- Limits 2 and 0, and `ratio → 0`.
- Both squeeze rings are at true radius on f193 (83.0).
- Disks sit inside U on original f230, with gaps about 9 px and 4 px.
- The `precisely` card is true and stays on f57 (24.0) to f84 (35.033).
- The chapter ends cleanly at f261 (112.55).

## Chapter 08: `loop_sums.rs`

### New

| ID | Sev | Cue | Frames | Problem | Anchor | Correction |
|---|---|---|---|---|---|---|
| 08-N1 | Medium | 08_3 | f88 (37.5) to f103 (45.0) | Under "dividing by z undoes the turn of the step's position" the ring has no z_k dot, though `f(z_k)` and `Δz_k` are drawn. The viewer can't compare the position's turn with f's. 08_1 had the dot. Original f92 confirms it is absent | `:443`; 08_1's dot at `:311-312`; `gone` at `:449-451` | Add the green dot and `z_k` label from `:311-312` beside `:443`, and push both onto `gone` |
| 08-N2 | Medium | 08_5 | f180 (78.0) to f204 (90.0), 12 s | Sixteen marks. Original f186 at 8× shows the geometry is right: the tip points up-right on the anticlockwise tangent. But the head's length and width are both 0.16, and the 0.35 rad chord (0.0905) is shorter than the head. So the near-equilateral head reads as a triangle pointing **down**. On the right of a circle that reads clockwise, which was prior 08-A's complaint, and it sits against the blue-up, green-down shared edge. The test at `:757` proves the sign only, so it can't catch this | `:211-214`; `draw.rs:479-490`; test `:757` | `:213`: `point(CIRCULATION.1 - 0.62)`. Chord 0.1586, about the head's length. The film's sign test still passes (cross 0.248), and the base midpoint lands within 0.0014 of the arc |
| 08-N3 | Low | 08_3 | f120 (53.5) to f127 (55.0) | `a bigger circle: the same column` lives about 1.5 s | `:516-520` | Recheck under Oliver; lengthen if still short |
| 08-N4 | Low | 08_4 | f144 (63.0) to f145 (63.5) | `only 1/z survives` trails "survives" by about 2 s | `:578` | Recheck under Oliver |
| 08-N5 | Low | 08_1 | f34 (15.5) | The ring's direction head sits over a bead, on the original | `:293` | Accept, or move it between beads |
| 08-N6 | Low | 08_3, 08_4 | f105 (46.0) to f156 (66.5) | Column arrows are 13 sheet px against the 42 px step arrow, with no scale. The smallest rings' heads are ticks at this size | None | Recheck at 4K |
| 08-N7 | Low | 08_5 | f162 (69.0) to f174 (75.0) | Still under the statement of the theorem; on topic. So is f183 to f193 | None | Accept |

### Prior findings

| ID | Was | Now | Evidence |
|---|---|---|---|
| 08-A | High | Fixed | Fixed for every chain, product and column arrow. The circulation marks are now 08-N2 |
| 08-B | Medium | Fixed | Second lap captioned and drawn over the first, f63 (28.0) to f70 (31.5) |
| 08-C | Medium | Fixed | `|total| = 0.000000` alone at f75 (34.0) |
| 08-D | Medium | Fixed | Labels and head f5 to f6 (1.1 to 1.5), step shrinks f36 to f38 (16.5 to 17.5), cleared at f85 (36.5) |
| 08-E | Low | Accept (Low) | Still f21 (9.0) to f30 (13.5). The adding happens in 08_2 |
| 08-F | Low | Accept (Low) | Still f91 (39.0) to f103 (45.0). Acceptable now the arrow is visible |
| 08-G | Low | Partly (Low) | Double colon gone. Orphan `assumed` remains on the second line of the Goursat card, f213 (93.9) to f230 (101.5) |

There is no prediction in chapter 08.

### Sound

- Chain readouts 0.26, 0.50, 0.87, 1.01, 0.97, 0.71, 0.26, 0.00 match 2R sin(15n°).
- `|total|` is 0 and the product points straight up.
- Column 0.26, 1.57, 2.87, 4.18, 5.48, 6.27 against a limit of 2π.
- The bigger circle keeps the same column, as scale invariance requires.
- Closing powers: ring radii 1.000, 0.504 and 0.341 on one scale.
- On the shared edge, blue goes up on the left square and green goes down on the right.
- The Goursat card is true. The chapter ends cleanly at f231 (101.883).

## Chapter 09: `boundary.rs`

### New

| ID | Sev | Cue | Frames | Problem | Anchor | Correction |
|---|---|---|---|---|---|---|
| 09-N2 | Medium | 09_5 | f223 (80.0) to f272 (94.167), about 14 s | The lower `−0.5` card (native x 374–457, y 455–502) covers the lower half of `a = 0` (x 384–472, y 428–473). Original f240 confirms it, and the second pass saw it on every frame. Two repairs landed on one spot: 09-F's card and 09-B's card. Lint 09_5 @80.0 reports it | `:278-287`; `:369-381` | `:284`: `p.p(C::new(1.05, -0.4))`. Spans x 404.8–492.8, y 395.8–440.8: 14.2 px above the card and 9.0 px clear of the r = 1/2 ring with its dots. `CENTRE_ZERO` is used only at `:281` |
| 09-N1 | Low | 09_3 to 09_5 | f118 (45.5) to f272 (94.167) | The panel's axis (x 893) runs through the first `o` of `opposite samples share a value` (text starts at x 890). Readable. Originals f120 and f240; lint 09_3 @45.6 is true | `:300` | `v(right + 0.2, -0.75)`: spans x 908–1128, panel right edge 1225 |
| 09-N3 | Low | 09_7 on | f354 (119.333) to f554 (175.0) | `x` is 0.18 from the right edge, where title-safe wants 0.25. Lint 09_7 @119.4 is true | `:577` | `axis_end[0] + 0.11`: desk margin 0.25 |
| 09-N4 | Low | 09_1 | f11 (2.9) to f25 (8.0) | Still for 5.1 s; on topic | None | Accept |
| 09-N5 | Low | 09_2 | f81 (31.0) to f90 (35.0) | "the average of the values on the circle" has a formula and a caption but no samples on the circle; they arrive in 09_3 | None | Root's call |
| 09-N6 | Low | 09_6, 09_7 | f317 (108.2); f409 to f410 (136.0 to 136.5) | Each leads its words by about 3 s: the `the next coefficient` row before "the next coefficient", and `radius 1` before "radius one" | `:506`; `:633` | Recheck under Oliver |
| 09-N7 | Low | 09_1, 09_6 to 09_9 | f280 (95.5) to f287 (97.5); f377 to f378; f503 to f506 | `2πi f(a)` is tight. The `1` mark sits under the dot cloud. Growing rings pass over `a` and `1`, transiently | None | Accept |
| 09-N8 | Low | 09_7, 09_9 | f416 (138.4) to f432 (143.5); f516 (164.0) to f532 (169.0) | On-topic holds. On original f535, `the bump` ends 5 native px from the right spike, with no contact | None | Accept |

### Prior findings

| ID | Was | Now | Evidence |
|---|---|---|---|
| 09-A | Medium | Fixed | Quotient f26 (8.5), first term gold f34 to f35 (12.4 to 12.5), second term and caption f47 (18.0), result f54 to f55 (21.5 to 22.0) |
| 09-B | Medium | Fixed | Pole rings f203 (73.5). Dark cards at f223 (80.0), on "Each pole". The lower card's position causes 09-N2 |
| 09-C | Medium | Fixed | `c_n` f276 (94.6), θ line, panel title and `1` f277 (95.0), slope f302 (103.0), next row f317 (108.2), coefficients f331 to f332 (113.2 to 113.4), ring lights f334 (114.0), caught on one sample |
| 09-D | Medium | Fixed | Labels f354 to f359 (119.333 to 120.0). Unchanged f360 to f375 (120.4 to 124.5), 4.5 s, down from 11.5 s. Circle grows f376 to f384 (125.0 to 128.0), `±i` f396 (132.0), segment, `radius 1` and bar f409 to f410 (136.0 to 136.5), pole ringed f433 (144.0) |
| 09-E | Medium | Fixed | `√2` card and segment f499 to f500 (159.0 to 159.167), circle f501 to f512 (159.4 to 163.0), recipe f512 to f514 (163.0 to 163.4), bar and range f515 to f516 (163.5 to 164.0), rows f533 (169.4) |
| 09-F | Medium | Fixed | Labels f106 (40.0), panel empty f105 (39.5) to f114 (43.5) under sentence 1 only ("Eight sample" is about 44.3), caption f118 (45.5). The `a = 0` card causes 09-N2 |
| 09-G | Low | Fixed | One ring and `r = 2`, f155 to f156 (56.867 to 57.0). No `f(0) = 1` mark, but the grey mark at 1 serves |
| 09-H | Low | Persists (Low) | The cloud sits near 0 from f159 (58.0), before the question at f168 (62.0), and through the think time. The number stays hidden until f201 (73.0). The skill's rule is that a quiz answer stays hidden until the think time ends, and this clue gets close. Root's call |
| 09-I | Low | Reduced (Low) | The plate's foot runs 20 native px under the card from f232 (83.0): plate bottom 630, card top 610. It was about 25 sheet px. Hides no sample. `:76`'s comment isn't quite met |
| 09-J | Low | Fixed | Plain fade f440 to f441 (145.0 to 145.033), slide f442 to f449 (145.333 to 147.0), `1` card f449 to f450 (147.0 to 147.1) |
| 09-K | Low | Fixed | Spikes level from f360 (120.4) |
| 09-L | Low | Fixed | `a little beyond` from f8 to f10 (2.4 to 2.6) |

### Prediction windows

- **09_4 to 09_5 holds for the number.** Voice ends 65.998 at f179. Ring f183 (67.0) to f194 (71.5), empty at f195 (71.867). Gold ring f198 (72.367), `average of 16` f201 (73.0), `limit: 0` f202 (73.4). See 09-H for the clue.
- **09_8 to 09_9 holds.** Voice ends 151.771 at f472. Ring f475 (152.4) until it goes at f497 (158.667). `√2` f499 (159.0), range f515 to f516 (163.5 to 164.0).

### Sound

- The split identity, the mean-value form, and `dz = i(z − a) dθ`.
- Panel dots at true positions for r = 1/2 and r = 2.
- Means 1.0039, 1.000015 and −0.000015, which agree with "within half a percent" and "about fifteen millionths".
- Residues −0.5.
- `c_0` to `c_4` are 1, 0, −1, 0, 1. Unrounded, `c_2` is −1.0000153.
- The turned rings are at true scale: 16 distinct dots once turned, 8 twice turned, gold mean −0.25.
- Radius 1. `√2` draws at 75 and 75 sheet px. The ring reaches both poles at f512 (163.0).
- Bar ends are true, plus 4 px of round cap: −0.41 and 2.41.
- 60 terms at 2 give 0.200000. 60 terms at 2.6 give 212, against a bump of 0.129.
- The `precisely` card is true.
- The ending: cue end f545 (172.529), hold to f554 (175.0), overlays fade from f555 (175.5), plate still fading at f558 (176.0).

## The layout lint, line by line

The TSVs for 06, 07 and 08 are empty, and 09's has five lines. An empty TSV is not evidence of clean frames: 07's is empty and 07-N1 runs for 34 s.

| Line in `09_boundary.layout.tsv` | Disposition |
|---|---|
| `09_1 +2.4s @2.4s behind`: plate strip under `a` | False on legibility. It is the fade-in; `a` is clear from f10 |
| `09_3 +6.8s @45.6s behind`: stroke under the caption | True: 09-N1 |
| `09_5 +8.1s @80.0s crossed`: lower `−0.5` card over `a = 0` | True: 09-N2 |
| `09_7 +0.6s @119.4s tight`: `x` near the right edge | True: 09-N3 |
| `09_8 +1.8s @146.6s behind`: moving label over a plate row | False on legibility. Transient |

## What ran and what did not

- **`checks/number_check.py`:** RESULT 85 PASS, 0 FAIL, Python standard library only. Output is in `checks/number_check.out.txt`. Two planted wrong strings fail as they should, and a PASS line records that; those two `FAIL control` lines are expected. It also holds the desk geometry for the corrections above.
- **`checks/check_review.py`:** RESULT PASS. It tests this file's frame and time pairs against `sampling.json`, voice quotes against `script.tsv`, word-share estimates, source anchors, severity counts, the coverage table and the binding table. Output is in `checks/check_review.out.txt`.
- **`checks/sheet-hashes.sha256`** and **`checks/input-hashes.sha256`:** 381 and 41 rows.
- **Read audit, on the session transcript:** an image row for every sheet, and two distinct reads for 09 sheets 024–080 only. The audit script was scratch and isn't kept here.

**Not done:** no compile, Clippy, test run, render, frame extraction, TTS or listening. No corrections were applied or tried. No canonical edit, Bead, commit or publication. I did not watch the video, hear any audio, or check Oliver sync, and this is not final AV acceptance. Numbers were recomputed by my own script, and I did not run the crate's `facts-check` or oracle.

## What this review binds

Sheet and original-frame hashes are in `checks/`. The four chapter sources below equal LavenderPike's staged repair (`complex-picture-repair-06-10/`). `draw.rs` differs from that staged copy, so it has moved on since. `one_derivative.rs` still has `z_0`, so my spacing patch is not in it.

| File | SHA-256 |
|---|---|
| `explainer_corpus/complex_analysis_visual/script.tsv` | `2968b0fd76540cbcd4f50290925e8acfec1aa995bea76a04934dac12f959929e` |
| `explainer_corpus/complex_analysis_visual/src/chapters/one_derivative.rs` | `b67231d94cafe4f5e93b103b3ed7179e5008aeb63c9a64f2abe5777b44fe5981` |
| `explainer_corpus/complex_analysis_visual/src/chapters/one_point.rs` | `43b6f1c323c5f71506e618bdcec5dec71ddc302c24e604186259fb026963e80c` |
| `explainer_corpus/complex_analysis_visual/src/chapters/loop_sums.rs` | `cf383504cdc80652acf4dacf13ba6b99cc6c679cb859749618dd559ab95f695d` |
| `explainer_corpus/complex_analysis_visual/src/chapters/boundary.rs` | `328d2acbc247eccc090340b825cc6764582ebe4337a8dc20089ea9fa001e1b59` |
| `explainer_corpus/complex_analysis_visual/src/chapters/draw.rs` | `4eba4bb18b989ac7075519031913ad4974e471a495a56a80fde4f796d9159545` |
| `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-preview02/render/06_one_derivative.mp4` | `fd887f7fab2814da93b1ca19a88d624d708bc990ff93209ab4c564625e9cb8bb` |
| `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-preview02/render/07_one_point.mp4` | `94625ae5433e5cdceea769307994174c7b0837af6c4a3f1167593766142f9370` |
| `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-preview02/render/08_loop_sums.mp4` | `6c86558f9ed22b47fda1df52fdafa2f3a6370a18b27d528d2d13ffc16ee799b9` |
| `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-preview02/render/09_boundary.mp4` | `6618d1660889042d47d2799801ca1581ab08317fb1c401436dfd2b79d7728a7f` |

Canonical source, root's staging and my earlier reviews were not edited. Other files in this folder are listed in `SHA256SUMS`.
