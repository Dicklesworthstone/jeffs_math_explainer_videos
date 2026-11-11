# Native 4K still review of final995 (CC3, independent)

Reviewer: CC3 (WhiteSwan), independent of the final995 overview review. Written 2026-11-10.

## Inputs, bindings and scope

- Stills: `Q/final-review995/native-retrieved01/native4k/frame-000001.png` to `frame-000128.png`, where Q is
  `docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17`.
  Every still is 3840x2160. Still k is `natives[k-1]` of `sampling.json`.
- `sampling.json` sha256 `74c64ebff91594915ec6aa31a75a695e64cab545748eb1f7b421c962dcc5f9b5` (hashed by me).
- `retrieval-verified01.json` sha256 `0007ebf0289e454b3dfe2a5f3a10e6b5006e851dc7de5b55a49348cf5915da33` (hashed by me).
- Master sha256 `80244e88ca5970d0da07a5d96d6d825ae1851bae703d75bd200f8810b04e7b90`, as `sampling.json` states it. I did not hash the master.
- Every PNG's sha256 was recomputed when its progress file was written and had to equal its `sampling.json` entry (the writer asserts it).
- Evidence types. Each still was opened as an image. The viewer shows the whole frame downscaled to 2576x1449 (0.67x), and that display is not pixel evidence. Pixel evidence here is either a crop cut from the native PNG at 1:1 or nearest-neighbour 2x/3x/4x and viewed (91 crop files in the scratch folder `native995-cc3-crops/`), or a scan of the raw RGB pixels (ink runs, radial scans, hue winding, model fits). Each finding says which kind it rests on; "display" marks an estimate read from the downscaled view.
- Not claimed: listening, motion, fine sync, AV acceptance or whole-film coverage. 128 stills are 128 instants. Hold lengths and pacing (root O3, O14, O18) cannot be judged from stills. Narration is ElevenLabs Frederick Surrey; I did not hear it.
- Existing final995 defects stay open. This report adds native measurements of them and new items only. My earlier picture report `independent-final995-cc3-pictures.md` (sha256 `8921bfd3795c58bd1dada64b556b14925bec57608f9a7460861d0533527b45b7`) is unchanged.

## Coverage: 128/128 stills, 32 progress files

Each progress file holds, per still: index, PNG sha256, master frame, chapter, local frame and seconds, sampling reasons, the native crops viewed and the finding. Each was created exclusively (never overwritten) from a matching `native995-cc3-notes-AAA-BBB.json`.

| Progress file | sha256 |
|---|---|
| native995-cc3-progress-001-006.json | 883663e5732c2c110d403b1ec5c5c16df78b05c928c73a2cf6da4bca33169688 |
| native995-cc3-progress-007-010.json | 8346eb0d991f7bcf3d748f9a04ce623fbc2766aa3f6f6a699f75567dfa2705b5 |
| native995-cc3-progress-011-014.json | f17a4f8a1a2c0c9511f792f246e3a361f73c6b38aa6ec3a8868d6a338f19e3f3 |
| native995-cc3-progress-015-018.json | 2b68ca74f1a947ef6375ddaeef48711b0542a721e0e5cf9aa7c121139236a62e |
| native995-cc3-progress-019-022.json | 616eb6611d1f9de9261666bfabfa12f2529ea634582a42a77300d623e27b745d |
| native995-cc3-progress-023-026.json | fd720bacb25418651a76aa2c676bb050e274ae6f526f99dd002d5b1823838ea9 |
| native995-cc3-progress-027-030.json | 859b9ad4e3773699c42a065c2f0bb36a5a2d362d45b6165fd007aadd226c579a |
| native995-cc3-progress-031-034.json | d3a3768ff300c33eba7bc905f11202982f69ea84e636b7b311af3144d1635c29 |
| native995-cc3-progress-035-038.json | 8831d556e81dec82709dfcbbf76035e9ae4b46427bd9afa627bee9ec51e932a6 |
| native995-cc3-progress-039-044.json | e5e9d6d47b54b8dc2a6e83508366e946396fb25b45c076507c1512e9b626cefe |
| native995-cc3-progress-045-048.json | 7ce1c7b7c07500f80c5bc0931093ca52cdafa571c10bc610065e63b52ca04976 |
| native995-cc3-progress-049-052.json | 50efb9dfda83cc661f69eedc8b02458603ba8c440cc734fa7279ea3ccb0c090a |
| native995-cc3-progress-053-056.json | fbf042f47db4d63d59a8a4735a97fa190658f14e807adcfef68486db8af719eb |
| native995-cc3-progress-057-060.json | cda67fb28ff54af2a779e7efd6f830f6fc78fdb7f342fccc58561a75f7661ea6 |
| native995-cc3-progress-061-064.json | 739d906a2000e5946dfd9320e8583c32e30ec377153d985595829b55e0670ee8 |
| native995-cc3-progress-065-068.json | 55d96296aa0b384476524784cb8c0f16a1b3bfb8dc63764c86201aede988fb9d |
| native995-cc3-progress-069-072.json | 4a6a892ac8deb7f231eafa755ced62713b7ad9db7db78364c2d5ef79fd454e99 |
| native995-cc3-progress-073-076.json | 6b547946483a85fd298e41899e0b1696640fc66dd288a4394231f5a070c580be |
| native995-cc3-progress-077-080.json | 50aaaefaaa12f308406af6bfaea4a76941f855d2c7e03c8af00205e0495253dd |
| native995-cc3-progress-081-082.json | d19848c4e8949eb911bf5556a83e49766cb2f022716395f2de93200102e52f9a |
| native995-cc3-progress-083-086.json | 70de815cd4b231d024fe54e7fa94b6356588363d666a4000858ee97c01a9f506 |
| native995-cc3-progress-087-090.json | 8380aadeed95821136a8adddc5c182dc5e7195ddfe7b6dadada19cb7146fb847 |
| native995-cc3-progress-091-094.json | f5663f92baa9a22fd2d3528b4f785b4282a06dfd1b56dc26977885475f9dc945 |
| native995-cc3-progress-095-098.json | c615964f1cbe6ac31a836fa1a7629da01d32f83a1a6c13145b3d895d433968ec |
| native995-cc3-progress-099-102.json | 0df61b4ecae95a2797185facd611f947e4f88608017464fcb0694c83631756a3 |
| native995-cc3-progress-103-106.json | b9c80b3761e594204b914c47c440e8d94240af601d463199db408113db288762 |
| native995-cc3-progress-107-110.json | cd886b4860c1f0e60ac8fb12727b060cd28c862e9e3ef019cc67427ce0eafd9c |
| native995-cc3-progress-111-114.json | 7baa3238aa9ab687f61b618fd3711b1d9e6cefb203ec703d9ad02e0e51fc80f1 |
| native995-cc3-progress-115-118.json | 1e07f94c1ac303e53eb29f9f2ecfdb15dbcecc4428144fea3f47df7542f30019 |
| native995-cc3-progress-119-122.json | 591eb2453aeaf91b0a131ec16440ca4f2c657f76f6d7cdf035b1e609a69f157a |
| native995-cc3-progress-123-126.json | 9d5d337dbc8ef043668d88ec2de2c7a8940e446b7621e9b7d0fe07d53e900644 |
| native995-cc3-progress-127-128.json | e9140aa58374d7a94c037ca3873adb38b5e584f0aebd609f122d80ecf7f83da9 |

The 32 files list 128 distinct indices, 1 to 128.

## Findings by severity

No High and no Medium. None of the new items changes a number, a statement or a turn count; all are Low. Coordinates are native pixels (x right, y down). "Master f" is the frame in the master; "local f" is the frame and time within the chapter.

### New Lows

**N1. Fraction bar far wider than its expression (still 060).** 08_loop_sums, master f64442, local f12336 (205.6 s). Pixel scan: the fraction bar spans x 2768-3501 (733 px, rows 807-812), while its numerator `epsilon D_0 P_0` spans x 3024-3245 (222 px). The bar overhangs numerator and denominator by about 255 px on each side, and the struck `4^n` on the left stands about 150 px before the bar, so the expression reads as three pieces rather than one product over `4^n`. Repair idea: size the bar to the wider of numerator and denominator plus the usual padding.

**N2. A long subscript detaches the integrand (still 070, milder at 072-074).** 09_boundary, master f71505, local f4440 (74.0 s). Column scan of rows 575-780: in the first term the integral sign's ink ends at x 2707 and g starts at x 2765 (58 px, with P under it). In the second term the sign ends at x 3066, the subscript `p clockwise` spans x 3074-3301 and g starts at x 3321, a 255 px gap, so it reads as an integral sign, a caption and a detached `g dz`. The same mechanism, milder: still 072 (local f6060), 137 px after `chord` against 85 px after `arc`; still 073 (local f6240), 123 and 173 px; still 074 (local f6900), about 202 px for `|z - a| = r` and `|z - a| = rho`. Repair ideas for root: put the orientation in a note outside the formula (for example "p walked clockwise"), or write the relation as the contour over P equals the contour over p.

**N3. Chapter 11 output panel: unlabelled compression and changing scale laws (stills 091, 092, 095).** 11_counting.
- Still 091 (master f95061, local f4050, 67.5 s): f(z) = (z - 0.45)/(z - 1.35) on |z| = 1.7. Storyboard 11_3 says "rescaled to fit (at radius 1.7 the image is a circle off zero)". The true image is a circle (centre 2.138, radius 1.433).
  - Measured about the axis crossing (2797,1174): the curve runs from 247 px left of the axis to the image dot 474 px right of it, and 263 px up and down. Ratios: width/height 0.432, left/right 0.521, top/right 0.555.
  - A uniform rescale would give 1.000 and 0.197. The radial compression w/(1 + |w|) at 607 px per unit gives 0.431, 0.524 and 0.553.
  - So the panel draws a tall D shape at a compressed scale. A crop of rows 1760-2160 shows no footer or label saying so.
- Still 092 (master f95359, local f4348, 72.47 s): the same panel is linear at 450.8 px per unit. The image dot of f(-1.35) = 0.667 sits at 300.5 px; under 091's law it would sit at 243 px. Nothing cues the change.
- Still 095 (master f98459, local f7448, 124.13 s): the panel fits w/(2 + |w|) at 606.7 px per unit. Five measured edges fall within 5 px of that law's predictions; w/(1 + |w|) and linear are rejected. Here the footer does say "output size compressed; direction and turns around zero unchanged".
- Turn counts are unaffected, because every one of these maps keeps the argument.
- Repair ideas: label 091's compression as 095 does and use one law for the chapter, or draw the true rescaled circle that the storyboard describes.

**N4. Code highlight band taller than its line (still 102, also 103).** 11_counting, master f101811, local f10800 (180.0 s); still 103 at local f11880. The band behind `if L * step >= |a|: refuse  # L bounds |f'| on this subarc` fills rows 1150-1225 (76 px, x 1851-3442) on a 50 px line pitch. Line 3's ink ends at row 1157 and line 5's starts at row 1216, so the band runs under 8 rows of the descenders of `a = f(start); b = f(end)` and 10 rows of the tops of `turns += angle(b / a)`. The text is drawn above the band and stays legible, but the band does not isolate its line. Repair idea: band height equal to the pitch, centred on the line.

**N5. `Res_{p_k} f` has no operator space (still 104).** 12_residues, master f104914, local f1971 (32.85 s). Column ink runs: R 2373-2437, e 2441-2476, s 2482-2512, subscript p 2517-2548, k 2553-2572, f 2577-2621. The f begins 4 px after the k, the same as the letter gaps inside `Res`. TeX's `\operatorname{Res}_{p_k} f` puts 3 mu there, about 16 px at this size (cap height 66 px, so an em of about 96 px). At 2x the f's lower hook nearly meets the k. Still readable.

**N6. Pole cards sit on the circle stroke (still 116).** 14_continuation, master f118667, local f1140 (19.0 s). Pixel scan and 4x crops:
- The `i` card (fill x 1083-1228, y 744-878, border about 3 px) has its rounded bottom-left corner on the yellow circle's stroke: at 78 and 76 degrees the stroke centre, (1091,879) and (1102,881), falls on the card's bottom border.
- The `-i` card (fill x 1089-1270, y 1470-1604) has its top-left corner on the stroke at -78 degrees (1091,1470).
- In both places the card is drawn over the stroke and hides part of its width.

Text is unaffected. Repair idea: move both cards about 20 px right, or out to the circle's outside.

**N7. Arrow joints: one construction, recurring (stills 005, 010, 012, 013, 046, 113, 127).** Cosmetic. Seen in 2x/3x crops; stills 113 and 127 also have pixel scans.

| Still | Chapter | Master f | Local f | What the crops show |
|---|---|---|---|---|
| 005 | 01 | 6291 | 6291 | The ring icon's small grey arrowhead floats inside the ring (x 3457-3491, y 1027-1057). At 3x a dark gap about 4 px wide splits it, with a 5 px stub on top, so it reads as a broken glyph. |
| 010 | 04 | 19205 | 273 | The curved arrow's shaft ends in a square stub about 15 x 20 px beside the head base (x 3240-3260, y 255-285). The head's lower corner overlaps the wheel rim by about 20 px. |
| 012 | 04 | 22319 | 3387 | A white rectangular neck about 20 x 15 px, wider than the loop stroke, stands above each of the three loop heads (left head x 795-885, y 1045-1140). |
| 013 | 04 | 23132 | 4200 | The same necks as still 012. |
| 046 | 06 | 45265 | 9680 | The round-capped shaft pokes about 15 px beyond the arrowhead's tip (x 1210-1225, y 640-655), so the tip reads blunt. |
| 113 | 13 | 112206 | 5320 | Straight shaft stubs are thicker than the contour (16-18 rows against 14 on the arc, 15 against 14 on the axis). The arc stub ends about 85 px from the head (near x 1193) with a 4-5 px step onto the arc. |
| 127 | 15 | 131472 | 2189 | Panel 3. A radial scan about (1920,1134) gives a stroke 7-8 px wide at a radius of 238-239.5 px, except at 27-37.5 degrees just before the head (head from 39 degrees). There the inner edge steps in from 235 to 229-231 px, so the last 11 degrees (about 46 px) are 10-13 px wide, with a visible step near (2133,1025). |

**N8. "precisely:" card over the plate (still 013).** 04_reading_color, master f23132, local f4200 (70.0 s). The card (y about 1830-2105, display estimate) overlaps the bottom of the domain-colour square by about 180 px. It covers only colour field, no marker or label. Cosmetic.

### Known items, with native measurements (not new)

**Heavy glyph faces (cc0 L1 class).** Symbol glyphs are set in a heavier face than the CM text around them. Pixel scans:

| Still | Glyph | Measurement |
|---|---|---|
| 003 | minus | 46 px wide, 8 px thick (rows 400-407), against the plus's 65 px width and 3 px bar |
| 006 | approx | 33 px wide with two 6 px strokes, against the `=` beside it, 47 px with two 2 px strokes |
| 027 | superscript minus in `2^-s` | x 828-849, 4 px thick, against the full-size plus's 3 px bar |
| 110 | minus in `residue -1` | 4 px thick, 28 px wide |
| 127 | numerator minus | 6-7 rows, against the plus bar's 2 rows |

The same face appears for the approx signs in still 003, the multiplication sign in `size x 1.69` (still 012) and the text superscript three in `z^3` (still 035). If cc0 L1 names only the minus, those three extend it.

The minus is heavy at the following stills (1:1 or 2x crops):

| Chapter | Stills |
|---|---|
| 01 | 003 |
| 04 | 013 |
| 05 | 016, 027, 029, 033, 035 |
| 06 | 038 |
| 08 | 051 |
| 09 | 061 |
| 10 | 083 |
| 11 | 088, 094, 095, 097 |
| 12 | 104, 105, 108, 110 |
| 13 | 113 |
| 14 | 116 |
| 15 | 127, 128 |

**Text-face math (cc0 L3, L4).** Still 043 (06, master f37265, local f1680, 28.0 s): the readout `2.00 + 2.45 i` is set in four separate slots. The gaps are about 53, 35 and 45 px, against about 15 px for a word space, so it reads as four tokens: native numbers for the cc0 L3 fixed-slot class. Upright i, f, r, a, z or x appear as text where formulas use math italic (cc0 L4) at stills 045, 047, 051, 055, 078, 081, 083, 089, 092, 094, 097, 098, 102, 104, 105, 116, 118 and 128. The same applies to `Re s` (027) and the upright z in the 035 cards.

**cc1 classes.** The heavy `<=` (L9) appears at stills 056, 066, 083 and 095. The heavy minus in chapter 09 formulas (L6) appears at 056, 061, 072 and 077. L1 appears at 054 and 055, L4 at 054, and L12 at 060.

**cc2 chapter 09 items.**
- L1, still 062 (local f1914): both connector lines end at x 2570 (rows 796 and 1876), while the zoom circle spans x 2802-3413 and y 1032-1641. The lines stop 232 px short, about 236 px above the circle's top and 235 px below its bottom.
- L2: rho stays outside the yellow circle with no leader at stills 067, 068, 069, 071, 074 and 076.
- L3, still 071: the card top sits just under "inside the dashes".
- L5, still 073: `-> 0` ends at x 3769, 71 px from the right frame edge.
- L6, still 074: the small circle's stroke is 11 px wide at radius about 104 px, and its four heads span 18-23 px radially, so each barb stands about 6 px beyond the stroke. At 3x they read as counterclockwise heads; at display scale they are small notched barbs.
- L9, still 079: the card covers 5 of the 7 rows of the plane's green bottom frame line (rows 1349-1353 of 1349-1355) and hides the foot of the vertical axis.
- M1, stills 072 and 073: the closing bar sits inside the ink run of the second dz's z, 0 px apart (x 3337-3340 inside 3311-3340; x 3168-3171 inside 3143-3171).
- M2: the rho move comes at about 117.7-118.4 s, and no still falls inside it (076 at 117.0 s is before it).

**My final995 items.**
- L1, still 080 (09, master f80457): the x label's ink ends at x 3792, 47 px from the right frame edge. It is already visible at this frame, earlier than sheets 0784-0788.
- L2, still 082 (master f82725): the circle's stroke runs about 20 px into the `1` card. It is already visible here, earlier than f82742.
- L6, still 088: the condition box under `turns: -1`.
- L7: still 089, under root O17 below.

### Native answers to root's open items (cc1-chapter-picture-review357)

- **O1** (stills 092 and 093, 11 local f4348 and f4408). At 4348 the readout shows `|f| =` with no value. At 4408 it shows `|f| = 0.7`, the rounded 0.72. The transient is confirmed and resolves within 60 frames.
- **O2** (stills 095 and 096, local f7448 and f7478).
  - The bottom root marker has its centre at (958,1676) and an outline of outer radius 29 px, 10 rows thick.
  - At 7448 the chip's top border starts at row 1700, so only rows 1696-1699 of the outline's bottom show.
  - In still 096, with the chip gone, all 10 rows show (1696-1705).
  - So the chip is drawn over the bottom 6 rows of the outline, and at 3x the ring is cut flat.
- **O3** (stills 098 and 099, local f8121 and f8303). The sentence `0 / 2i = 0, and nothing else lands on 0` is fully drawn and unclipped at both frames. How long it is held is a timing question that stills cannot settle.
- **O5** (stills 101 and 103, local f9731 and f11880). At 1:1 the monospaced type is crisp, with no blur and no broken stems.
  - Metrics: x-height 22 px, advance 25.6 px, stems 3-4 px, line pitch 50 px.
  - Scaled, the x-height is 11 px at 1080p, 7 px at 720p and 5.5 px at 540p: legible at 4K and 1080p, marginal at 720p, which is consistent with O5's 540p concern.
- **O7** (stills 106, 108 and 110, 12 local f2532, f2766 and f3002). All the residue labels sit on row 2085 (ink rows 2044-2086), 74 px or 3.4% above the bottom edge. That is inside a 5% title-safe margin (108 px) and just at a 3.5% action-safe margin (76 px).
- **O8** (still 108, local f2766). The third plate is already faintly present. Inside x 2050-2600, y 1050-1650 its mean is (14.3,16.5,22.2) against the background (12,16,20), with its top edge at row 976 in column 2300. That is about 2 levels, invisible at normal viewing.
- **O9** (still 116, 14 local f1140). On three sides the plate's frame is 5-7 px of grey. At the bottom the plate colour runs to row 1925, and rows 1926-1929 are the same grey (59,68,80), shared by the plate's frame and the footer's top border; the footer fill starts at 1930. The plate is 1506 x 1505 px, so nothing of it is covered; the two borders read as one line.
- **O11** (stills 123, 124 and 125, 14 local f10260, f10278 and f10301).
  - At 10260 and 10278 there is no box outline yet. Along the box rows and the left columns in x 300-1400, no pixel is 2 or more code values from the background.
  - What is there is a 2 px fringe just outside all four edges of the plate, not only the bottom: rows 1734-1735 below, rows 694-695 above, column 1450 at the left and 2230-2231 at the right.
  - The fringe's colour follows the plate colour next to it while its brightness stays near the background: (20,14,23) under red, (16,14,35) under violet, (9,19,17) under yellow-green.
  - That is what 4:2:0 chroma subsampling of the encoded master gives at a sharp edge; a plate leak would carry the plate's brightness.
  - At 10301 the grey box line (about 5 px; rows 693-697 and 1732-1736, columns 202-208 and 2228-2233) lies over the plate's outer rows and covers the fringe, as O11 says for 10290-10301.
  - Answer: a chroma fringe, not a plate leak. Informational.
- **O12** (still 088, 11 local f2010). In columns 600, 1200 and 1700 the plate's last row is 1925, the footer border fills rows 1926-1929 and the fill starts at 1930. The plate shows 1512 x 1508 px, so the footer meets it with no gap and probably covers its bottom 3-4 rows.
- **O13** (still 091, local f4050). The inset chip spans x 3233-3711, y 206-738. The panel's top border (rows 497-501) shows up to x 3239 and its right border (x 3470-3475) from y 733 down, so the chip hides a 236 x 236 px corner of the panel frame.
- **O17** (stills 089 and 090, local f2280 and f2400).
  - The three chips are separated by 12-20 px of background and do not touch each other.
  - The third chip spans rows 1068-1222, across the panel's horizontal axis.
  - Its bottom border (rows 1216-1221 at x 2743) covers the top of the `0` tick label. The visible ink (x 2731-2755, rows 1221-1257) starts as two separate side strokes, so at 3x the 0 reads open at the top.
  - Same item as my final995 L7, measured more precisely.
- **Not covered by any still:**
  - O10 (frames 8876-8877): still 120 at 8908 shows no edge line.
  - O14 (2160-2340).
  - O15 (3911-3916): still 112 is at 3926.
  - O16 (1253-1254): still 117 is at 1240.
  - O18 (8304-8344): still 099 is at 8303, one frame before.

### Observations (no severity)

- 029 (05, local f7362): the zeta strip has unequal axes, about 49.6 px per unit up against about 170-180 across, so the rings about the zeros read as wide ellipses. No grid angles are shown, so nothing is wrong.
- 033 (05, local f9169): the sin z markers at +-2 pi touch the plate's side borders (left ring over x 136-140, right ring to x 1274) without passing beyond them.
- 084 (10, local f3767): within about 30 px of the essential singularity the plate is point-sampled speckle, with a thin dotted seam along the real axis inside |x| < 120 px. This is inherent to sampling, and its behaviour in motion was not judged.
- 102 (11): the neighbouring image at 45 degrees lies only 3.8 px inside the reach ring's centre line, so the ring cuts its 27 px dot. Correct, but the margin is too small to show.
- 104 (12): where the direction arrow overlaps the dimmed loop, from about 101 to 119 degrees, the loop reads brighter: (176,149,71) against (137,115,54).
- 113 (13): the graph's x scale is 236 px per unit and the plane's 282, both centred on x 1109. So x = +-1 on the graph does not sit above the unit circle, and the graph spans [-4, 4] while the contour's segment is [-2, 2]. The graph has no ticks, so nothing is mislabelled.
- 127 (15), panel 4: the white real axis (x 2324-2947) is drawn over the inner 4 of the frame's 6 stroke columns at both ends; it reads as intended.
- 127 (15), panel 5: the blue and green paths end at two separate dots, (3147,1100) and (3147,1167), and meet at about row 1130 with a hard colour step. Schematic.
- 128 (15): each pole marker has a dark knockout ring about 6 px wide that cuts the axis and the circle around it. Deliberate, and it reads clean.

## Maths checks that passed (pixel measurement unless marked display)

- 01: 1/(1 + 1.21) = 0.45249; the degree-40 sum (1 + 1.1^42)/(1 + 1.1^2) = 25.23 (003).
- 03: grid images, y = +-1 maps to u = x^2 - 1, v = +-2x (008).
- 04: bands with |z| growing by 1.30 give |f| = |z|^2 growing by 1.69 (012).
- 05:
  - Zeta zero heights in the ratios 1 : 1.485 : 1.770 : 2.155, against 1 : 1.487 : 1.769 : 2.152 for 14.135, 21.022, 25.011 and 30.425 (029).
  - Gallery zeros and poles by hue winding (030).
  - Cube roots of 1 at 120 degrees (033, display).
- 06: difference quotients 2 z0 + h for f = z^2 (038, 040, 043, 044; partly display).
- 08: 48 sin(7.5 degrees) i = 6.2652i, shown as 6.27i (051).
- 09:
  - Triangle and ring cancellation (067-069).
  - Arc-versus-chord bounds (073).
  - Mean of the samples 1/(1 - 2^-8) = 1.0039 (078).
  - 1/(1 + x^2) (080).
  - |i - 1| = sqrt 2 (082).
- 10:
  - Zeros at 573/k px for k = 1 to 5 (084).
  - sin z zeros 636 px apart, winding 3 (086).
- 11:
  - Chips -2, +1 and turns -1 by winding (088).
  - f(-0.219 - 1.327i) = 0.665 + 0.284i, within 1 px, and Re w = 2/3 for the circle through the pole (093).
  - Roots of z^5 - 3z + 1 within 0.002 units, each winding +1, and the Rouche bound on |z| = 1 (095).
  - 64 steps and 8 turns for z^8 (102, 103).
- 12: windings +1, +2 and -1 for z, z^2 and 1/z; the poles of 1/(1 + z^2) at +-i (108, 110, 112).
- 13: the four poles of 1/(1 + z^4) at 45, 135, 225 and 315 degrees; a counterclockwise contour with R = 2; the graph fits 1/(1 + x^4) with rms 0.56 px (113); p_1 and p_2 are the two poles inside (114).
- 14:
  - Bars doubling at 2, 6, 13, 27 and 55 px (119).
  - Sum of z^(2^n): the zero at -0.656 against the series' -0.6586, and 2 zeros inside 0.9 (120).
  - Gamma poles at 0, -1, -2, -3 and -4, each winding -1, with alternating real signs (126).
- 15:
  - A multiplication by 2 + i: 26.6 degrees, about 2.2x (127).
  - Zeros - poles = 2 - 1 in panel 3 (127).
  - Poles at +-i at 256.5 px from the axis and the dot at 3.00 units; the hidden answer is sqrt 10 = 3.16 (128).

The quiz answers are hidden on the quiz stills checked (094, 118 and 128).

## Files

All files are in the session scratch folder; nothing in the repository was changed. No build, render, synthesis, Beads or production edit was made.

- This report: `native995-cc3-review.md`.
- Progress: the 32 `native995-cc3-progress-*.json` files above.
- Notes: the matching `native995-cc3-notes-*.json` files.
- Crops: `native995-cc3-crops/`.
- Tools: `native995-progress.py`, which writes a progress file and asserts each PNG hash, and `native995-diff.py`, which compares two stills.

Report, notes and progress files are made read-only after writing.
