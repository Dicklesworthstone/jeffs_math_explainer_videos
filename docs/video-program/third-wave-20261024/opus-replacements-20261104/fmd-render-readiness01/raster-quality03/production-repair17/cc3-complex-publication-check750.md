# Check 750: complex analysis publication package

Author: CC3 (WhiteSwan), 2026-11-10. Scope: the existing publication376 copy and the thumbnail208 still, checked against the frozen 15-chapter schedule, the reviewed script and the cited sources. Text, arithmetic and one still only. Nothing was uploaded, published or entered in any web form. No source, production, Beads or canonical file was edited, and nothing was rendered or launched. I did not listen to anything or watch any motion, and the master does not exist yet, so no seek on it was inspected.

**Result: the package is ready to upload once master606 succeeds.** All 15 marks are correct for the frozen schedule. Title, description and tags have no em dash or en dash. Every mathematical qualification and credit is supported. There is one optional Low wording correction in the description (C1). Nothing blocks upload.

## Package

| Upload field | File in Q | sha256 |
|---|---|---|
| Title | `publication376/title.txt` | `48a242e3e4209b446850be61f1868d8f6280881d00d34d2bc387e529c0edc920` |
| Description | `publication376/description.txt` | `868c83d3528fc71a6412040045ad01d031b4d0e70c18978874083b92d2b323aa` |
| Tags | `publication376/tags.txt` | `40e49939e363bb750de3884b2c8b0ed3664303887829417622e5ac79d52594a8` |
| Thumbnail | `publication19-thumbnail208/thumbnail.png` | `e5dfc2bd04aa3af148ee66ca8107594acf3015a237f5f36a08f1c3c72c4c2eab` |

These are the hashes root adopted in `root-publication376-review380.json` (`4f6a09dc…`), and the thumbnail is the one root viewed in `publication19-thumbnail208/review.json` (`ba2b9bcf…`). The author record `publication376/review376.json` is `6469ff2e…`.

Video and captions are not in this package. They are master606's outputs, `complex_analysis_visual_master.mp4` and `complex_analysis_visual.en.srt`/`.vtt`, and root owns them.

## 1. Chapter marks

### The frozen schedule

Master606's `film()` reads each chapter's `frames` header from the 15 timeline269 tables. It requires them to be byte-equal to timeline21 with only the narration path changed, and their sum to be 133662 (`combined-master606.py:344-355`). Its `gates()` puts each chapter at the sum of the frames before it (`1147-1154`). Its `master()` requires the movie to have exactly that many frames at 60/1 (`1361-1362`).

The frame counts below come from root's actual build-host run of that same `film()`, `master350-film-preflight385.json` (`a5b019ae…`). Its helper hash is `60575944…`, which is `cc0-combined-master350.py`. Master606's `film()` and `header()` are AST-identical to 350's. The `TIMELINES`, `TIMELINE269_INPUTS`, `BASELINE`, `NARRATION`, `ORIGINAL_NARRATION`, `KINDS` and `TOTAL_FRAMES` constants are identical too.

Each count also matches at least one more independent record:
- **01, 04, 05, 07, 09, 10, 11, 12, 14:** local copies of the timeline269 tables in `preview274-review305/preview/`. Their bytes equal the timeline269 pins in `cc1-caption-repair347.py:35-50` (`61fee2e9…`), and each `frames` header is the count below.
- **02, 03, 13, 15:** the retained finals' `frames` in master606's `RETAINED` (`179-203`).
- **06, 08:** both permits per chapter (`06_one_derivative-csd-permit375.json` `c43801cc…`, `06_one_derivative-css-permit577.json` `acd8b174…`, `08_loop_sums-csd-permit375.json` `c2d0b172…`, `08_loop_sums-css-permit474.json` `92119d3a…`). Each states these frames and carries the chapter's timeline269 hash.
- **09 and 10:** whichever row root chooses, the counts are fixed. `root-boundary607-gate.py:12,32` (`9293391f…`) requires 16504 for a native553 09. `root-native582-gate.py:12,32` (`a4e9f314…`) requires 7442 for a native521 10. A boundary313 09 or a combined 10 is held to the same timeline counts by `film()`. **Root's final09 decision does not move any mark.**

### Recomputation

The rule is the one draft01, draft306 and 376 use: mark = floor(start frame / 60) seconds. "Lead" is how many frames before the chapter's first frame a click on the mark lands.

| Chapter | Start frame | Frames | Start (s) | Mark computed | Mark in copy | Length (s) | Lead |
|---|---:|---:|---:|---|---|---:|---:|
| 01_the_bump | 0 | 6311 | 0.000 | 0:00 | 0:00 | 105.2 | 0 |
| 02_turning_numbers | 6311 | 5372 | 105.183 | 1:45 | 1:45 | 89.5 | 11 |
| 03_two_planes | 11683 | 7249 | 194.717 | 3:14 | 3:14 | 120.8 | 43 |
| 04_reading_color | 18932 | 6774 | 315.533 | 5:15 | 5:15 | 112.9 | 32 |
| 05_gallery | 25706 | 9879 | 428.433 | 7:08 | 7:08 | 164.7 | 26 |
| 06_one_derivative | 35585 | 10030 | 593.083 | 9:53 | 9:53 | 167.2 | 5 |
| 07_one_point | 45615 | 6491 | 760.250 | 12:40 | 12:40 | 108.2 | 15 |
| 08_loop_sums | 52106 | 14959 | 868.433 | 14:28 | 14:28 | 249.3 | 26 |
| 09_boundary | 67065 | 16504 | 1117.750 | 18:37 | 18:37 | 275.1 | 45 |
| 10_rigidity | 83569 | 7442 | 1392.817 | 23:12 | 23:12 | 124.0 | 49 |
| 11_counting | 91011 | 11932 | 1516.850 | 25:16 | 25:16 | 198.9 | 51 |
| 12_residues | 102943 | 3943 | 1715.717 | 28:35 | 28:35 | 65.7 | 43 |
| 13_real_integral | 106886 | 10641 | 1781.433 | 29:41 | 29:41 | 177.3 | 26 |
| 14_continuation | 117527 | 11756 | 1958.783 | 32:38 | 32:38 | 195.9 | 47 |
| 15_closing | 129283 | 4379 | 2154.717 | 35:54 | 35:54 | 73.0 | 43 |

- The total is 133662 frames = 2227.700 s = 37:07.7. The last mark, 35:54, is inside it.
- **All 15 marks equal the copy.** The labels and start frames also equal 376's record, row by row.
- YouTube's chapter rules hold: the first mark is 0:00, there are 15 marks, they ascend, and the shortest chapter, 12_residues, runs 65.7 s (the minimum is 10 s).
- The floor rule never lands after a chapter's first frame. The largest lead is 51 frames (0.85 s, chapter 11), so a click shows under a second of the previous chapter's end. That is the safe side, and no change is needed.

## 2. Title, description and tags: dashes and form

I scanned every code point for U+2010 to U+2015, U+2212, U+2E3A, U+2E3B, U+FE58, U+FE63 and U+FF0D. **None appear** in the title, description or tags. "Cauchy-Riemann" uses the ASCII hyphen.

- **Title:** "The Hidden Geometry of Complex Functions", 40 characters, all ASCII. It is root's choice in 380.
- **Description:** 2660 characters (YouTube's limit is 5000). It has no `<` or `>`, which YouTube rejects. The only non-ASCII character is the superscript two in `1/(1+x²)`. LF line endings.
- **Tags:** 18, no duplicates. The longest is "fundamental theorem of algebra" (30 characters). YouTube counts 385 of 500 (each multi-word tag gains two quote characters, plus the commas).

## 3. Mathematical qualifications, against the script

Script: `script.tsv` `234f7257…` (105 cues), read in full.

| Description says | Script | Verdict |
|---|---|---|
| A power series fails where the curve is smooth; the answer lies above and below the line | 01_3 to 01_5: works between -1 and 1, fails past 1, poles at i and -i, radius one | holds |
| Moving grids show multiplication and the complex derivative as a stretch and a turn | 02_4, 06_3 | holds |
| Colour shows zeros and poles | 04_4, 04_5 | holds |
| Cauchy's theorem for squares and triangles, by shrinking squares and cancelling edges | 08_5: squares are quartered; triangles are cut at their midpoints into four triangles; shared edges cancel | see C1 |
| Values on the boundary determine the inside | 09_1, 09_2, 09_6 | holds |
| Loops count roots without finding them; residues at complex poles give real integrals | 11_8; 13_2 uses one pole, 13_6 two | holds |
| Topic list: polynomials, rational functions, exponential, sine, tangent, gamma, zeta | 05_1 to 05_7 | holds |
| The Cauchy-Riemann equations; critical points and holomorphic functions | 06_6; 07_2, 07_6 | holds |
| Cauchy's integral formula and the radius of a power series | 09_2, 09_7 | holds |
| The identity theorem and Liouville's theorem | 10_1 (the theorem, not named in narration), 10_5, 10_6 | holds; the names are the standard ones |
| The argument principle and the fundamental theorem of algebra | 11_2, 11_7 (not named in narration) | holds |
| A derivative at one point is not enough; the strong results need every point of an open region | 07_5, 07_6 | holds |
| A loop that counts zeros and poles must not pass through one | 11_2, 11_4 | holds |
| Continuation can depend on the path, can meet a natural boundary, never makes a divergent series converge; 1 + 2 + 3 + ... still diverges | 14_6, 14_7, 14_5 | holds |
| A picture of zeta cannot settle the Riemann hypothesis | 05_8 | holds |
| Some calculus helps; the geometry is built from scratch | 13_1 ("You may know it from calculus"); 02_1 onward | holds |

Each chapter label also names that chapter's subject: 01_1 to 01_6, 02_4, 03_1, 04_1, 05_1 to 05_7, 06_1/06_2, 07_2 to 07_6, 08_1 to 08_5, 09_1, 10_1, 11_8, 12_1, 13_1, 14_1 to 14_7, 15_1.

### C1 (Low, optional): "shrinking squares" covers only half of the proof

Paragraph 2 says "We prove Cauchy's theorem for squares and triangles by shrinking squares and cancelling edges". In 08_5 the triangle case shrinks triangles, not squares: "Join the midpoints of its sides, and you get four triangles at half the size". Suggested exact replacement for that clause:

> We prove Cauchy's theorem for squares and triangles by cutting them into smaller and smaller copies and cancelling shared edges, then use it to show how values on the boundary determine the inside.

This changes `description.txt` and so its hash. If root adopts it, 380's adoption would need a new record. The marks are unaffected.

## 4. Source credits

| Credit | Checked against | Verdict |
|---|---|---|
| NIST DLMF, Calculus of a Complex Variable: dlmf.nist.gov/1.9 | page fetched 2026-11-10: "§1.9 Calculus of a Complex Variable", chapter 1 | exact |
| NIST DLMF, Functions of a Complex Variable: dlmf.nist.gov/1.10 | "§1.10 Functions of a Complex Variable"; subsections include analytic continuation and the residue theorem | exact |
| NIST DLMF, Gamma Function: dlmf.nist.gov/5.2 | "§5.2 Definitions" in "Chapter 5 Gamma Function" | fair; the label names the chapter |
| NIST DLMF, Riemann Zeta Function: dlmf.nist.gov/25.2 | "§25.2 Definition and Expansions", under the part heading "Riemann Zeta Function" in chapter 25 | exact to the part |
| Sister video, Logarithms Made Visual: From Slide Rules to Matrices, youtu.be/lgpTDROHgQY | root's logarithm publication receipt (`publication-receipt01.json`, `6fcfbc5c…`): that title, visibility public; `docs/video-program/catalog.json` (`6d409784…`) has the same URL, public, recorded 2026-11-06 | matches; live page not opened |
| Narration: ElevenLabs Frederick Surrey, a synthetic voice | `script.tsv` line 1 names "ElevenLabs Frederick Surrey" | holds |
| Music: procedural score generated in code | master606 mixes score21's stems (`--score`); score21 runs the native `score all` (`score21.py:12`); `crates/explainer_score/src/lib.rs:1-8`: std-only Rust synthesis, deterministic from seeds | holds |
| Animation: FrankenManim | the film's animation engine, franken_manim (the binary master606 runs is built on it) | holds; the spelling is a styling of the engine's name |

The DLMF pages were read with plain GET requests, and no data was sent. Earlier reviews 197 and 304 reached the same titles on 2026-11-08.

## 5. Thumbnail

- **File:** PNG, 3840x2160 (16:9), RGBA at 8 bits, 679205 bytes (0.65 MiB, under YouTube's 2 MB). The alpha channel is 255 at every pixel (ffmpeg `alphaextract,signalstats`: minimum 255, maximum 255), so it is fully opaque.
- **I viewed the still once, displayed at 2576x1449.** Its text reads "THE BEAUTY OF", "COMPLEX", "ANALYSIS", "A PLANE IN COLOR" and "z²: HUE = OUTPUT ANGLE". The text is legible with clear margins, and there is no dash.
- **The colour field matches the film's wheel (04_1) for z².** Right and left are red. Upper right and lower left are yellow green. Straight up and down are teal. Upper left and lower right are violet. Each colour appears twice around the centre, as 04_3 says.

The thumbnail is not a movie frame, so the schedule does not affect it.

## 6. Left as root's choices (unchanged, already recorded in 376)

- The thumbnail says "THE BEAUTY OF COMPLEX ANALYSIS", while the title is "The Hidden Geometry of Complex Functions". The two differ in wording, not in substance.
- The description and narration use British "colour", while the thumbnail says "COLOR" and the tag is "domain coloring".
- YouTube Studio asks about altered or synthetic content at upload; that answer is root's. The description already says the narration is a synthetic voice.

## Not established

- **Master606 has not run.** The marks are computed from the schedule its gates enforce, so a successful master606 has these chapter starts by construction. Nobody has read its `rows[].start_frame` or tried a seek on the actual file.
- **If root changes any chapter's timeline** (for example a retimed 09 instead of the 16504-frame one both gates require), the marks from that chapter on must be recomputed.
- **The captions are master606's;** they are not checked here.
- **No listening, motion review, fine sync, final picture, independent AV, upload or publication acceptance is claimed.** The only picture viewed was the thumbnail still.
- **The sister link's live state** was taken from root's receipt of 2026-11-06, not from the page today.

The checking scripts stay outside the repo (paths in the pane report).
