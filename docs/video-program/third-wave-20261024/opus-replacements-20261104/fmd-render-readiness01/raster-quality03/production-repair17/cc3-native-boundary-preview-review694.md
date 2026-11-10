# Native 09_boundary preview571: selected pictures read as images (task 694)

cc3 (WhiteSwan), 2026-11-09. A picture review of the native FrankenManim preview571 of chapter `09_boundary`, rendered on mac-mini-max at 960×540 and 60 fps.

**Scope in one line: thumbnails of a 960×540 preview, 313 selected frames out of 16,504.** Each picture was seen at 640×360 on a contact sheet. This review does not accept final 4K (final586 is still rendering), motion, audio, listening, fine sync or independent AV. It addresses the native-pixel gap in my review676 R4 only at this preview-thumbnail scale.

## Result

- **Viewed: 40 of 40 sheets, which is 313 of 313 selected pictures.** Each sheet was opened with an image-capable Read and looked at. A scan of this session's transcript finds 48 Read calls on the 40 sheets and 48 image results: all 40 at least once, and sheets 002, 017, 019, 021, 024, 025, 027 and 030 twice.
- **Defects: High 0, Medium 0, Low 6.** No clipped text, no wrong number and no wrong formula in anything I read. None of the six looks to me like a reason to hold final586, and each would need a source change and a re-render; that decision is root's.
- **09_3 panel:** the "opposite samples share a value" backing is opaque. At 4× I see no axis show-through, which settles the question my review340 O3 left open, at this scale. No sample can lie under it.
- **09_7 edge at 223.2 s:** the axis "x" is whole and unclipped in all 39 selected pictures that show it, 223.2 to 273.2 s. It stays 0.18 units from the right edge, inside the frame and outside the lint's 0.25 title-safe margin.
- **Disagreement with root's viewing record:** root's record lists no concrete defects and "no persistent concrete overlap" in sheets 1-30. I see one persistent contact there (F1, 28 selected pictures). I agree with root's two focus observations.

## What was viewed, and how

Evidence, all under `/tmp/complex-boundary572-review604/`:

| File | sha256 (computed by me) | Note |
|---|---|---|
| `pictures572.json` | `3ad07ddc85e4d1d3d9ff714bcdd3b9129586e339639ce42172bfd27e18f7ab9a` | Equals root's manifest hash. 84,594 bytes. |

The manifest records movie sha256 `5b1b5a5e3ad1e892c2c02c9df7cb141e661c148a32920969c80b04a05bbaef7b`, 16,504 frames and a full decode. The movie is not in the cache, so I did not hash or decode it. The manifest's own scope line says "Technical checks and extraction only."

Sheet layout: 1280×1536, 8 tiles of 640×360 at JPEG quality 95, downscaled from 960×540, each captioned with picture number, frame and time. Sheet N holds pictures 8(N−1)+1 to 8N. Sheet 040 holds only picture 313, as expected. All 40 sheet hashes below match the manifest rows, 0 mismatches. A matching hash shows which file I opened; the seeing was done by reading each image.

| Sheet | Pictures | sha256 (computed by me) |
|---|---|---|
| 001 | 1-8 | `c365fc91d5f46253ca7c39a42461025aea88c1e9f191c636b98f4855c04544fe` |
| 002 | 9-16 | `8cc605bd20b1f00a3b8b56ce2f26941cb0ea2a0fbc5527cfc1d9c8060b978c0d` |
| 003 | 17-24 | `cb9c42f37bff74404b84580abb83765311d5ba51a560901e5b0db7f10c07c91a` |
| 004 | 25-32 | `bb2718684eee24ad86f06fd9b22f340a72189345721177b70f0661e232dc411c` |
| 005 | 33-40 | `eba2823232ebd3a341c2682bae58f1e369f7c137654b84c9087ba35ee6b9ad82` |
| 006 | 41-48 | `4bf671ca4f21c7f491f91b671ebc75db4ce5ad5f4f57915b1db623ee78e0d86c` |
| 007 | 49-56 | `9b6369bb5bbfe312408046e219f38afb4cdb24d4145c2e6202eb903dbaba2700` |
| 008 | 57-64 | `b7c709324571535085b0f77607b953c14abb9002664bdd6f0c88e7bb5c433a29` |
| 009 | 65-72 | `ce0eee723c79685490b637a35b797e62e27a0ecb17a033a7f1fdffb675bc988f` |
| 010 | 73-80 | `b024aec6cda1a96567d3e6545d5fbcc410757e8c748c5179dc98ab2a00765095` |
| 011 | 81-88 | `62cf537c2a8953511afd27703fa58fd1ba843204a0d10a1239d97fd2aeceee53` |
| 012 | 89-96 | `d18d6ef9785135e119a276327bc887c2a31112d49b499894ca8131fb7c7205dc` |
| 013 | 97-104 | `2762d073852a7261b7a8f08f5fb983a09e477985d6d59f46c116771b06f684f5` |
| 014 | 105-112 | `37473ac814c0094df16a2d57c834e995442e380eb18898a2fd36023b54fe101b` |
| 015 | 113-120 | `7a98ede3b69c1bea42f5492ac86b6f139bcde4e7248cc2487252b7f1f91b0974` |
| 016 | 121-128 | `8aa4c15791efe1b0d351914c536501240a70cec4f62a5e9ff5615bc20578efbe` |
| 017 | 129-136 | `287f823a31de5648493e275c92191fca904a65cda80e23857bebc9b4f923acbc` |
| 018 | 137-144 | `dad417be51942b11ef8a29248fc0969130c4df791f78c058135e1d542260b041` |
| 019 | 145-152 | `3a4d54886cecd2bf7692a892e130134d0504d8d9554b83b6342f0f56be5d0ad1` |
| 020 | 153-160 | `807a942eb1f98941a8b387bfebeea5d2faf11b2eb043ee3c0338284b736faf2e` |
| 021 | 161-168 | `6637ba50cac6c77c1f0c03f4c98b118a44734a02fe02d065fff8a1f899ad41f5` |
| 022 | 169-176 | `4f638880e7592a0ca5af8babaf7397471d21ffa3f1e96d31ae72b27fa53cb9e7` |
| 023 | 177-184 | `43bfddb891a0b0605273376a3425ee87fee303e68a5b77c4eb694dddae1fe2c0` |
| 024 | 185-192 | `8c095c56c680e52627cc682087c2d9bb824a2883fa285575c1c7266af03aecb6` |
| 025 | 193-200 | `daddeb8854be0d5eb15732f8d0c103fb64567e55093d7cac5dbabc5d9a582922` |
| 026 | 201-208 | `cdcb4dbef2d5fed37cc9b4818dec323b224c49b72739da0d089d356924afd46f` |
| 027 | 209-216 | `c7baabfe56a0ce6fb1fc1898a09a8e5ea4d98c8492a616345a1145186c4d9371` |
| 028 | 217-224 | `c1c53f6a73cee625be52a8a3dc8505a0a61e3c410f545bb3e0e01a6e8cd5a2b4` |
| 029 | 225-232 | `bf52879605693b62c3f499baeab116990dc4de1e737198364f8141980f7d3934` |
| 030 | 233-240 | `e4b01b264d195d979baa05cd049f27e9cc7bcbec6931bc7c13bd32c9a2a04d94` |
| 031 | 241-248 | `acedca2c6ce2d4c514270e7b3b613e844f339dba5f35fd612c0211f2377cc3a7` |
| 032 | 249-256 | `d630ba9a9f3950bd26f242519014c50122907eb9008d632e04344df709a54a91` |
| 033 | 257-264 | `e7b1392a9a383a3773bd26e6ce8e889a7230210d8356440599b419cd1cf0b0b5` |
| 034 | 265-272 | `a62480aeb803993e7cc28a6eec1791cc9b3f9090fdb5cf7dc19480a6981172e7` |
| 035 | 273-280 | `30952d4b2adb58321a4d5f3d47e7dee302c6738eb202ff77d5ab84b6cb74702e` |
| 036 | 281-288 | `c81c3a3465e678dbf9c65b76725cdd58b4bb1610bcce498e203442f6b481b91b` |
| 037 | 289-296 | `897608c687b624878c610bff818e5c155980ae67494c81388aed64655a3f8ba2` |
| 038 | 297-304 | `9fe497ce41b78f3e24496ce477e9282ad0ddda65339a69e3ea3747f8dc698187` |
| 039 | 305-312 | `ebbf29cfab4fd8807efdb93c226eb720148f6700ade92a064f351cd20a6dff8a` |
| 040 | 313 | `e257cf67ca33e3c1804ee1370478bc0f504ea2c15f3dc24562ac69bd2c9fcee8` |

**Viewing aids.** I enlarged regions of the same sheets (nearest neighbour, 2× to 8×) and viewed 12 of those crops: pictures 149, 166, 191, 193, 233, 241, 254, 256, 269, 271, 273 and 307. A thirteenth crop was made and not viewed. They magnify a downscaled JPEG, so they add no detail beyond the thumbnails. They stay in my scratch area and are not committed.

**Also read, in Q:**

| File | sha256 (computed by me) | Use |
|---|---|---|
| `root-boundary572-pictures.py` | `0e2afe838633274bd26980864f38420f424c3ec8c94b30acf9c657d270951034` | How the sheets were made; pins at lines 42 and 53 |
| `root-boundary558-reference/09_boundary.layout.tsv` | `f91bb8a1bb6b7ea895414e739250922aa86eb02f9ecab57f851e95ee06b59a7e` | The two lint rows; equals the pin at line 42 |
| `root-boundary558-reference/09_boundary.cues.tsv` | `eed5a50ad5797fa258ee3fcf9319e29946f403a86b3909090e73a3a15dc74249` | Cue times; equals the manifest's `cues.tsv` hash |
| `root-boundary558-reference/select.filter` | `af3bf5a06eb73f44f7a7c83a0889e92849bfa5d9b02ac48658b4c49964a90d9c` | Equals the pin at line 53 |
| `root-boundary572-pictures-viewed604.json` | `e6b203d70ea7774a45a960d06976465c51c86769d6d18dc0fcf4ca9f259b7a88` | Root's viewing record, opened only after all 40 sheets were viewed |
| `cc3-preview313-pictures340.md` | `7a0c1f9fd38a62c437b66cf1d356ef3d479008070c9ff9af06a51e1d3b6b9c14` | My earlier review of preview313 |

**Independence.** I read root's record after my own first pass over all 40 sheets. I am not independent of the lint rows or of my own review340 on preview313, and the two 340 findings carried in F1 and F3 were known to me before I looked.

## Focus results

Cue starts from the table: 09_3 at 149.967 s, 09_7 at 222.717 s.

### 09_3 panel: the lint row at 156.8 s

Lint row: `09_3 +6.8s @156.8s behind`, a stroke at x 2.80..2.82, y −1.02..1.62, painted under "opposite samples share a value" (still there at 223 later looks).

- **What the stroke is.** Its x and y span match the vertical axis of the "the samples' values" plot, at tile x about 446 and running border to border. It is that plot's axis.
- **On screen in 25 selected pictures, 232-256 (frames 9408-12069, 156.8 to 201.15 s), fading in at 232, gone by picture 257 (frame 12099, 201.65 s).** The cue table puts 09_6 at 201.15 s, consistent with that exit.
- **Backing is opaque at this scale.** In 4× crops of pictures 233, 241 and 256 (frames 9420, 10038 and 12069) the axis stops at the top edge of the pill and nothing shows through it. Root's record says the same.
- **No data can sit under it.** The plotted values lie on a circle with |Im f| ≤ ρ/|1−ρ²| for |z²| = ρ, which is 4/15 ≈ 0.267 at both ρ = 1/4 and ρ = 4. On the plot's scale the pill's top is about 0.33 below the real axis (read off the crop at about 107 tile px per unit). The lowest dots I see sit clear of it in every picture from 233 to 256.
- **Text is readable but dim.** It is the weakest text in that column, and the time on screen is about 44 s.
- **Cosmetic only.** The pill's lower edge lies along the plot's bottom border and replaces the green line across most of the plot's width, about 59% in the crops. It reads as a pill docked on the frame. I do not list this as a defect.

### 09_7 edge at 223.2 s and persistence

Lint row: `09_7 +0.5s @223.2s tight`, "x" stands 0.18 from the right edge; a text wants 0.25 (still there at 255 later looks).

- **Which pictures.** Frame 13393 (223.217 s) is cue start + 0.5 s. Pictures 269 and 270 (frames 13392 and 13393, 223.2 and 223.217 s) show it at low opacity, and picture 271 (frame 13404, 223.4 s) at full.
- **Measured against the thumbnail.** The glyph's right edge is at about tile x 632 of 640 in the 6× crops, about 0.18 units from the edge, in agreement with the lint.
- **Whole glyph, no clipping,** in crops of pictures 269, 271, 273 and 307, and by eye in every selected picture between them.
- **Persistence.** All 39 selected pictures 269-307 (frames 13392-16392, 223.2 to 273.2 s) show the "x". It is gone at picture 308 (frame 16473, 274.55 s). Those 39 are a subset of the lint's 255 later looks.
- **Cue change at the same moment.** Picture 268 (frame 13380, 223.0 s) has the old content at about a quarter opacity. Pictures 269 and 270 have none of it left, and the new graph at low opacity. Picture 271 is at full. I see no double exposure in these four samples.

The "x" is outside the title-safe margin by 0.07 units, about 5 px at 960×540. Whether that matters is a 4K question; nothing about it is broken here. Root's record ("no clipped text observed") agrees.

## Findings, most important first

All six are Low. All are thumbnail-scale readings. I re-opened the sheets named under each finding before citing it, except where marked first pass.

### F1. Low: "inside the dashes" sits on the top border of the "same endpoints" card

- **Where and when.** Pictures 164-191, frames 5280-6480, 88.0 to 108.0 s, 28 selected pictures. Picture 192 (frame 6540, 109 s) has no card. Sheets 021 and 024 re-read; 022 and 023 first pass.
- **Evidence.** In the 8× crop of picture 166 (frame 5400) the feet of the grey caption run into the card's top border with no visible gap. The caption is also the lowest-contrast text in that column.
- **History.** This is L1 of my review340 on preview313, with the same contact now seen in native preview571. Review340 measured a 0-1 px gap at 540p; I cannot measure that here.
- **Lint.** Neither layout row reports it.
- **Root action.** None required by this review. If the chapter is ever re-rendered, separate the caption and the card. Please do not read `concrete_picture_defects_observed: []` as excluding this contact.

### F2. Low: labels for the inner curves stand outside the outer curve

- **ϱ.** From frame 2580 (43.0 s) until it has moved inside by frame 7140 (119.0 s) (first pass on sheets 015 and 026; sheets 019, 021, 025 and 027 re-read), a green "ϱ" stands at the lower right, outside the yellow circle. Its colour ties it to the small green circle. At frames 6600-6900 (110 to 115 s) the formula reads ∮ over |z−a|=r equals ∮ over |z−a|=ϱ, and no "r" is drawn in the inset, so the only named radius sits beside the circle it does not name.
- **p.** From frame 4320 (72.0 s, picture 148) and gone by frame 5100 (85.0 s, picture 161), the lower-case "p" that names the inner polygon stands outside the outer polygon at the lower left. It is pale, like "P", so it has no colour tie (5× crop of picture 149, frame 4380).
- **Fixed by the film.** By frame 7200 (120.0 s, picture 209) "ϱ" sits beside the shrunken circle.
- **Root action.** None required. If re-rendered, move both labels next to the curves they name.

### F3. Low: the long bound formula ends close to the right edge

- **Where and when.** Pictures 185-191, frames 6228-6480, 103.8 to 108.0 s, 7 selected pictures. Sheet 024 re-read.
- **Evidence.** The rightmost ink is at about tile x 629 of 640, about 943 of 960 scaled, in agreement with the 942 that review340 L2 measured at 540p. That is about 0.25 units from the edge, at the lint's threshold, and the lint does not flag it.
- **Root action.** None required. Check the same formula in the 4K review.

### F4. Low: the "r = 1/2" pill touches the yellow circle

- **Where and when.** Pictures 233-237 (frames 9420-9918, 157.0 to 165.3 s) and 261-267 (frames 12189-13363, 203.15 to 222.717 s). Sheets 030, 033 and 034.
- **Evidence.** At the sheet's scale, the right edge of the pill meets the left edge of the circle with no gap. No text is touched. By contrast the "a = 0" pill keeps a clear gap on the other side.
- **Root action.** None required.

### F5. Low: the small "a" and the shrinking green circle run into each other

- **Where and when.** Pictures 209-216, frames 7200-7620, 120.0 to 127.0 s, clearest from picture 212 (frame 7380, 123.0 s). Sheet 027 re-read.
- **Evidence.** As the green circle shrinks, its top stroke reaches the small "a" above the open red dot. By 124 to 127 s the label and the stroke overlap at this scale.
- **Root action.** None required. The open red dot still marks the point. Check 124 to 127 s in the 4K review.

### F6. Low: the two leader lines end on a frame that is not drawn

- **Where and when.** From picture 012 (frame 1710, 28.5 s) until they fade out around frame 2096 (34.9 s). Sheet 002 re-read; sheets 003-008 first pass.
- **Evidence.** The upper and lower lines stop at about tile x 428, about 40 px short of the inset circle's left edge and above and below its top and bottom. They end on the corners of a box with no outline, so they point at empty space. Review340 measured the same endpoints on preview313 and treated the zoom lines as repaired, without considering this point.
- **Root action.** None required. An outline on the close-up, or ending the lines on the circle, would resolve it if the chapter is re-rendered.

## Observations, not defects

- **O1. Closing absolute-value bars are tight against "dz"** in the bound formulas at 95 s and later, for example pictures 185-191 (frames 6228-6480) and 210-216 (frames 7260-7620). In the 6× crop of picture 191 (frame 6480) the bar nearly touches the "z". TeX sets `|` without added space, so this may be normal typesetting; I could not separate glyphs from JPEG blur at this scale.
- **O2. The "precisely" band** (pictures 254-256, frames 11508-12069, 191.8 to 201.15 s) covers about the bottom 10 sheet px of the plate. It is opaque, its text reads, and the r = 2 ring stays clear of it (2× crop of picture 254).
- **O3. The triangle arrows at 58 to 68 s** form a dense starburst in an inset about 100 px wide. I cannot judge their direction at this scale (pictures 129-136, frames 3480-3900, sheet 017 re-read). The single red shared edge at 66 to 68 s is visible (first pass).

## Checked, no finding

- **On-screen numbers**, read from the sheets and recomputed from the on-screen formula alone in a scratch script, without film code:

  | On screen | Computed |
  |---|---|
  | average of 8 at r = 1/2: 1.0039 | 1.003922 |
  | average of 16 at r = 1/2: 1.000015 | 1.0000153 |
  | average of 16 at r = 2: −0.000015 | −0.0000153 |
  | limit 0, and 1 − 0.5 − 0.5 = 0 | residues of f(z)/z: 1, −1/2, −1/2 |
  | c₀ to c₄: 1, 0, −1, 0, 1 | 1, 0, −1, 0, 1 |
  | works on (−0.41, 2.41), √2 ≈ 1.414 | 1 ± √2 = −0.4142, 2.4142 |
  | 60 terms at 2: 0.200000 | 0.2000000002 |
  | 60 terms at 2.6: 212 | 212.18 |

- **Formulas as displayed**, by hand: the split of f(z)/(z−a), the arc and chord bound, the circle and polygon bound, the 2πϱ·M bound, the Cauchy formula, c_n as a turned average, and the "precisely" wording, which is stated as sufficient ("holds if").
- **Plot geometry.** Dot positions at r = 1/2 for n = 1 land at ±0.8 and ±1.333 on the scale shown, and the n = 2 average mark at −0.25.
- **Plate.** At thumbnail scale: poles at ±i on the 50 px per unit scale, red along the real axis and between the poles, teal beyond the poles on the imaginary axis. All consistent with 1/(1+z²).
- **Quiz order, in pictures only.** "average = 1?" and its countdown precede the answer: pill from 171.8 s, ring 174.3 to 176.8 s, result text at 180.55 s. "works from ? to ?" likewise: pill from 251.8 s, ring 253.4 to 258.2 s, radius line and √2 from 260.1 s, interval at 266.8 s.
- **Circle of radius √2 at centre 1** stays inside the plate (pictures 303-307).
- **Ending.** At frame 16473 (274.55 s) only the plate remains. It is dimmed at frame 16488 (274.8 s) and nearly black at frame 16502 (275.033 s). Frame 16503 (275.05 s), the last, is blank (sheet 040).

## Exact steps for root

1. Nothing in this review needs action before final586 finishes.
2. When the native 4K exists, look at these times in the 4K frames: 90 s and 104 s (F1, F3), 121 to 127 s (F5), 58 to 68 s (O3 arrows), 157 and 201 s (09_3 pill backing), 223.4 and 273.2 s (09_7 "x").
3. Look at the unsampled stretches in the 4K or at motion scale, since the 313 frames say nothing about them: 2.6 to 28.0 s, 133.0 to 138.883 s, 140.883 to 149.967 s, and the 5 s steps through 09_5 and 09_6.
4. Decide F1 to F6 as owner's calls. Each needs a source change and a re-render.
5. For the R4 record: a second reader has now looked at the selected native pictures, at thumbnail scale. The native final 4K has not had a second reader.

## Not established here

- Final586, or any 4K picture. Nothing at 960×540 full size either: only the sheets are in the cache.
- Motion between samples. 313 of 16,504 frames is 1.9%. Cue coverage is uneven: 220 selected pictures in 09_1, 4 in 09_2, 9 in 09_3, 6 in 09_4, 8 in 09_5, 11 in 09_6, 16 in 09_7, 5 in 09_8, 7 in 09_9. The longest gaps are 25.4 s (2.6 to 28.0 s), 9.083 s and 5.883 s.
- Audio, listening, narration match, fine sync or independent AV. The quiz and label timings above are picture order only.
- Exact pixel gaps. Distances here are read from downscaled JPEG thumbnails, and sizes in units are estimates, except where a lint row or review340 supplied the number.
- Colour accuracy on any display, and legibility on a phone.
- Whether the F1 and F3 contacts are identical in pixels to preview313's: I compared what I saw, and did not diff frames.
- Chapter 10 and every other chapter.
