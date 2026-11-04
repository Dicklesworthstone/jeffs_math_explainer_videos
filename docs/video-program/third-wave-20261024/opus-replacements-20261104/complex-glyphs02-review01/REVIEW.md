# Complex glyphs02 page review

**Reviewer:** BronzeJaguar, for BlueSalmon Mail 1921, 2026-11-04.
**Scope:** all 67 pages of `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-glyphs02/glyphs/pages`, `frame_000000.png` to `frame_000066.png`.
**Author overlap:** I wrote part of this crate's source (complex repairs, BronzeJaguar, 87do.39), so this is not independent of the film's source.

## Verdict

The zero-failure report holds for missing symbols, but it cannot see the five findings below.

- No symbol is missing, and no literal command text or wrong symbol appears.
- No row is clipped by a page edge, and no two rows overlap.
- All 734 sources are on the pages as 754 ink rows.
- **One real collision:** the subscript in `z_0` overlaps the z's solid ink (F1).
- **Four shape or style findings:** F2 to F5. All are legible and none changes the mathematics.

**None of these needs to hold back root's repaired scene preview.** F1 and F3 are small string changes if root is rebuilding those chapters anyway. F2, F4 and F5 are style calls for root or the owner.

## Findings

Pixel figures are on the probe pages (1920×1080, Text size 34, TeX size 36). Crops are in `evidence/`.

### F1. The subscript in `z_0` overlaps the z

Each area below is `page:x0,y0,x1,y1`. Rows 597 and 605 each hold two occurrences.

| Row | Film | Area | Source |
|---|---|---|---|
| 594 | ch 12 | `51:235,350,272,395` | `residues.rs:31` |
| 597 | ch 06 | `51:100,695,136,740` and `51:335,695,372,740` | `one_derivative.rs:26` |
| 605 | ch 15 | `53:180,140,218,185` and `53:415,140,455,185` | `closing.rs:28` |
| 612 | ch 08 | `54:360,165,400,215` | `loop_sums.rs:51` |
| 637 | ch 06 | `56:145,720,182,766` | `one_derivative.rs:47` |
| 638 | ch 06 | `56:182,805,218,847` | `one_derivative.rs:44` |
| 676 | ch 08 | `61:405,312,442,354` | `loop_sums.rs:50` |
| 716 | ch 06 | `65:52,135,92,178` | `one_derivative.rs:211`, supplement `mod.rs:407` |
| 717 | ch 06 | `65:52,205,92,264` | `one_derivative.rs:217`, supplement `mod.rs:408` |

- **Evidence:** in all 10 occurrences in rows 594 to 716, z and 0 stay one connected shape even at the solid-ink level (`checks/outputs/touch_test.txt`). Row 717 lowers the 0 further: it stays joined to z at the two lighter levels and only abuts it at solid ink. Controls `c_0`, `c_1`, `p_1`, `u_y` and `z_k` stay separate, and the sampled superscripts are clear.
- **Cause:** `franken_markdown@126a530 fmd-math/src/layout.rs:1172-1183` places a lone subscript at the base's advance. In the bundled `cmunti.ttf`, z's ink runs 0.056 em past its advance. The other subscript bases measured run less: v 0.034, u 0.026, a 0.025, c 0.011, p 0 (`italic_face_check.txt`).
- **In the film:** the layout is in ems, so the same overlap should appear at film sizes. That is inferred; I did not measure a film frame.
- **Options:**
  - Change the strings to `z_{\,0}` at the sites above, including `z_{\,0}^2` in row 717. This is unbuilt and needs a re-probe, so try one row first.
  - Change the engine's script placement. That would affect every film.
  - Accept it, since `z_0` stays readable.

### F2. `\theta` draws as the open form ϑ

| Row | Film | Page | Source | Occurrences |
|---|---|---|---|---|
| 635 | ch 09 | 56 | `boundary.rs:36` | 1 |
| 649 | ch 09 | 58 | `boundary.rs:59` | 2 |
| 662 | ch 09 | 59 | `boundary.rs:48` | 2 |
| 705 | ch 09 | 64 | `boundary.rs:47` | 2 |

- **Cause:** `fmd-math/src/commands.rs:178` asks for U+03B8. Both `cmunti.ttf` and `cmunrm.ttf` draw that code point in the open form, and so does U+03D1 in `cmunti.ttf` (`evidence/F2-theta-from-the-faces.png`). Neither `\vartheta` nor `\mathrm{\theta}` should give a closed θ.
- **Effect:** it is the same letter and the meaning does not change. `script.tsv` never says "theta", so nothing spoken clashes with it.
- **Options:**
  - Accept it. This is my recommendation unless the owner objects.
  - Rename the angle on screen, for example to `t`, in those four strings.
  - Remap the glyph in the engine or font.

### F3. `|-3z|` reads like a subtraction

- **Where:** row 723, ch 11, page 65, y 692 to 747, `counting.rs:50`. The ink reads `| − 3z|`.
- **Cause:** `|` is an ordinary atom, so the minus keeps binary spacing. TeX's atom rules give the same result.
- **Option:** `|{-3z}|`. This is unbuilt, and I did not confirm that the engine treats the group that way. Any test that uses `LEASH_TEX` needs checking too.

### F4. Superscripts in Text rows come from IBM Plex Sans

| Row | Film | Page | Ink | Source |
|---|---|---|---|---|
| 18 | ch 02 | 1 | `(1 + i)² = ?` | `turning.rs:33` |
| 242 | ch 11 | 19 | `dog: z⁵ − 3z + 1` | `counting.rs:56` |
| 342 | ch 05 | 28 | `one tile of z³ − 1, traced exactly` | `gallery.rs:65` |
| 411 | ch 03 | 34 | `straight after z²?` | `two_planes.rs:35`, `:472` |
| 475 | ch 11 | 39 | `walker: z⁵` | `counting.rs:56` |
| 503 | ch 11 | 41 | `z⁵ − 3z + 1: how many roots` | `counting.rs:49` |
| 508 | ch 07 | 42 | `\|z\|² × jagged g, \|g\| ≤ 1 = M: derivative 0 at 0` | `one_point.rs:42` |
| 509 | ch 07 | 42 | `\|z\|²: one point, no disk` | `one_point.rs:51` |

- **Effect:** the digits are sans and heavy beside Computer Modern letters. Chapter 11 also sets `z^5` in TeX (row 723), so the viewer sees both forms of `z⁵` (`evidence/F4-plex-superscripts.png`).
- **Cause:** Computer Modern Roman has no ², ³ or ⁵, so the engine falls back per character (`franken_manim@a399d62 crates/fmn-text/src/font.rs`, `glyph_fallback`). That is also why the probe cannot fail on them.
- **Related, same class:** math in Text rows uses upright letters where TeX rows use italic, and the Text radical has no bar (rows 527 to 531).
- **Options:** set these strings in TeX, which changes letter style and width and needs a layout re-check. Or accept them.

### F5. The symbol face is heavier than Computer Modern, in both Text and TeX

- **Symbols:** `− × → ⇒ ≈ ≠ ≤ ≥` come from Noto Sans Math in Text rows by cmap. In TeX rows the same symbols, plus `↦ ∫ ∮ ∑`, show the same heavy look. `fmd-math/src/parse.rs:170` turns `-` into U+2212, and `faces.rs` documents the math-symbol fallback.
- **Measured stroke widths** (`stroke_widths.txt`; a scan across a slanted or diagonal stroke overstates its true thickness, so the × and ∫ figures are upper bounds):

| Stroke | Width |
|---|---|
| CM `+` and `=` | 1 to 2 px |
| Fraction rule, drawn `\Big\|` bar | 3 px |
| Noto minus | 4 px |
| Noto ×, scanned across its centre | 6 px |
| ∫ stem, scanned horizontally | 9 px |

- **Effect:** the symbols are correct and legible, but a minus is visibly heavier than the `+` and `=` beside it (rows 31, 75, 549).
- **Fix:** routing Text through TeX would not change this. It belongs to the engine and font on this pin and is not specific to this film, so I propose no change in this crate.
- **Minus forms in the film:** there are two. Number mobjects use a thin U+2013 en dash, following the Reference (`fmn-library/src/numbers.rs:21`, rows 512 to 517), and everything else uses the Noto minus. `crates/explainer_kit/src/kit.rs:597` records those numbers as plain strings, so the probe shows the dash's shape but not its height in the film.

## Result by class

| Class | Result |
|---|---|
| Missing symbol or dropped term | None. Each row read against its source; the 7 vertical gaps of 18 px or more are all inside fractions |
| Literal command text | None by eye. A static scan of Text and Mono rows finds no TeX markup; TeX rows have no bare 3+ letter runs |
| Wrong symbol | None. F2 is a same-letter variant |
| Clipping at a page edge | None. No row within 60 px of the right, 40 px of the bottom, or 40 px of the left |
| Overlap between rows | None. Ink bands found 754, predicted 754 |
| Overlap within a row | F1. `a_{-1}` (row 611) joins only at the edge blur. Other spots tested are clear (`h→0`, `−∞` by the bar, `p_k f`) |
| Mathematics on the page | Read as right. Values recomputed by hand are listed below |

**Values recomputed by hand:**

- `11 terms` at z = 1.2, 1/2, 2 give 32.15, 1.999023, 2047.
- sin 3i ≈ 10.0 i, and sin² 3i ≈ −100.36 with cos² 3i ≈ 101.36.
- sin i ≈ 1.18 i.
- ζ(−1) ≈ −0.083333.
- 1/(2i) = −0.5 i.
- p⁴ = −1 gives 1/p³ = −p.
- π/√2 ≈ 2.2214.

## Looks wrong but is not

- **Words split mid-word** (for example row 428, `int` / `egral`): the probe chunks non-TeX rows every 64 characters. This is why 734 sources make 754 ink rows. These are not film breaks.
- **Rows 727 and 728, all zeros:** `CodePanel::build` uses them only to measure one column (`crates/explainer_kit/src/props.rs:440-441`). They never join the panel's group.
- **Comment `#` columns on pages 65 and 66:** the probe drops the leading spaces that the panel restores as an offset. Row 733's mark sits 97 px right of row 732's, and four columns measure 97.6 px, so they align in the panel (`code_columns.txt`).
- **Row 562, `= × (2 + 2 i)`:** an authored operator note after the matrix (`one_derivative.rs:589-596`), not a dropped value.
- **Fragments such as `turns:` or `total stretch:`:** supplement rows for characters, not film strings. Of the 734 sources, 94 are supplement only and prove glyphs, not film presence.
- **Small gap after a prime, `f′ (z)`, and the spaced ` :` in rows 681, 683 and 705:** both read right, so I do not count them as defects.

## Evidence and method

- **Pages:** I opened all 67 full size and read each against its expected rows. Doubtful spots were enlarged. `coverage/pages.tsv` has each exact path, SHA-256, row range and observation. `coverage/rows.tsv` maps every ink row to its page and registry row.
- **Log correction:** `coverage/viewed-log.tsv` is the raw log. Four counts I typed there were wrong (pages 47, 57, 61, 62) and happened to cancel in the total. I recounted those pages against their images, and `pages.tsv` carries the corrected counts.
- **Receipt:** `resource-receipt.json` was stable through this review, with SHA-256 `e2f62483…` unchanged at delivery.
  - Phase preview, admitted, exit 0, 12 samples, 15:15:54Z to 15:18:52Z.
  - `inputs_sha256` equals `final_inputs_sha256`, `ed62350f…`.
  - `command.log` reports 734 sources and 0 failures; `failures.txt` is empty.
  - Page hashes matched at delivery; all are in `coverage/run-files.sha256`.
  - The newest chapter source (10:14:27 local) and the binary (10:15:32) predate the run.
- **Limit of the overlap test:** `touch_test.py` covers the named spots only. Everywhere else, overlap within a row was judged by eye at 1080p, so a one-pixel touch elsewhere could have been missed.

**Re-running the checks:** copy `checks/` to a scratch folder first and run from the repo root.

- `python3 -B` (needs PIL): `band_gaps`, `code_columns`, `crop`, `inner_gaps`, `italic_face_check`, `page_map`, `stroke_widths`, `touch_test`.
- `python3 -I -B`: `font_faces`, `log_page`, `show_rows`, `source_scan`, `trace_rows`.
- `show_rows` and `inner_gaps` read `page_rows.tsv` beside them; that file is `coverage/rows.tsv`.
- `italic_face_check` needs `font_faces` beside it, and takes the `fmd-font/fonts/computer-modern` folder as an argument.

## Not covered

- **These pages cannot show:** film size, colour or position, or overlap and clipping inside a scene.
- **Scene motion:** not reviewed.
- **Listening and narrated sync:** nothing was listened to. Narration remains ElevenLabs Oliver and nothing here touches it; no suggested change alters cue words.
- **Independent final AV:** still open.
- **Live readouts:** the probe skips `TextKind::Live`, so they are covered only through the supplement rows, such as row 75.
- **`glyphs/capture/*`:** these 32×18 frames were not opened.
- **My runs:** I did not compile, render, run the probe, or edit canonical files. I read the pages, the registry, sources, the binary and the bundled fonts. Every suggested string change above is unbuilt.
