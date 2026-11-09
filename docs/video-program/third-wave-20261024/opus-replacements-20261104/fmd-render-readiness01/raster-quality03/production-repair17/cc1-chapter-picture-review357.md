# cc1 chapter picture review 357: 11_counting, 12_residues, 14_continuation

Reviewer: cc1 (BronzeJaguar), independent of root and of CC2. Written 2026-11-08T21:39:06Z. Companion record: `cc1-chapter-picture-review357.json` (same directory).

This is a still-picture review. I opened every existing contact sheet listed below as an image and read every tile on it: the preview274-review305 sheets for the three chapters, the coverage grids (coverage-grid305b for 11 and 14; coverage-grid305/12_residues for 12, because coverage-grid305b has no 12_residues folder), and the four transition sheet sets. In total: 163 sheets, 1,491 pictures. For the faint spots listed under Method I also amplified crops (read-only) of the extracted PNG pictures or of the sheets.

It is not a motion, listening, fine-sync, 4K, final-master or AV review. The six acceptance flags in the JSON are false. Nothing here says the chapters are approved for release.

## Sources, bound by sha256

| chapter | movie (Q-relative) and sha256 | frames | timeline sha256 | preview274 sampling.json sha256 |
|---|---|---|---|---|
| 11_counting | `preview274-review305/preview/11_counting.mp4`<br>`0b6244a005f64ff9b9b381a8ec8a58ffb7c174d085aceb452871d7f5dd0f811e` | 11932 | `29a3afd345fa3a376e5c163d5bf5e01267a866e2305bdd27c6413a5bab46807d` | `eb67bf33a9ad2f7c91c5601026816d7ebd5d1fc3dafb5a68ae3c5004e8307846` |
| 12_residues | `preview274-review305/preview/12_residues.mp4`<br>`a78c20024c257eaa4783db80f9b99c23442fe67ac8f646e354e633074f842199` | 3943 | `d25e8b958cd32f2a233b38c293183f3d39b2d1502137c389c69ba49ff0d0dca7` | `df4ac4872fe729b2cb11b91b44041f143f9cfca707a4872c3056503cda59272e` |
| 14_continuation | `preview274-review305/preview/14_continuation.mp4`<br>`a0d34489e9757ff3db96e3d8b7a438845a90036453b2d4d7c9426efd8dc7c432` | 11756 | `2b3a5bff177889f00fb06871a8910d98babb4ac3c4636f50758c897cb11fbc99` | `5faa729974a76a216bfcc5d2a36975df67fabd0bd79a07a1912fc784db5feb2b` |

All three movies are the 540p60 preview274 renders. Each preview274 sampling.json names its movie by this hash and its timeline by the hash shown. The two grid samplings and the four transition manifests name the same movie hashes (checked by the builder, see JSON `checks`).

## Sets viewed

| set | chapter | binding file and sha256 | sheets viewed/existing | pictures | frames |
|---|---|---|---|---|---|
| `preview274-review305/11_counting` | 11_counting | `preview274-review305/11_counting/sampling.json`<br>`eb67bf33a9ad2f7c91c5601026816d7ebd5d1fc3dafb5a68ae3c5004e8307846` | 25/25 | 193 | 0…11931 (193 samples; exact list in appendix) |
| `preview274-review305/12_residues` | 12_residues | `preview274-review305/12_residues/sampling.json`<br>`df4ac4872fe729b2cb11b91b44041f143f9cfca707a4872c3056503cda59272e` | 13/13 | 101 | 0…3942 (101 samples; exact list in appendix) |
| `preview274-review305/14_continuation` | 14_continuation | `preview274-review305/14_continuation/sampling.json`<br>`5faa729974a76a216bfcc5d2a36975df67fabd0bd79a07a1912fc784db5feb2b` | 10/10 | 80 | 0…11755 (80 samples; exact list in appendix) |
| `coverage-grid305b/11_counting` | 11_counting | `coverage-grid305b/sampling.json`<br>`163f404d7d9e0e7e57b02fb9d02a2764115546dd06976bf76369ec1365670f65` | 19/19 | 146 | 180…11880 (146 samples; exact list in appendix) |
| `coverage-grid305b/14_continuation` | 14_continuation | `coverage-grid305b/sampling.json`<br>`163f404d7d9e0e7e57b02fb9d02a2764115546dd06976bf76369ec1365670f65` | 22/22 | 176 | 180…11700 (176 samples; exact list in appendix) |
| `coverage-grid305/12_residues` | 12_residues | `coverage-grid305/12_residues/sampling.json`<br>`bd9bd9969c332a18b141947030a37aba1b472191960f0b9844d5f6825ece6c00` | 7/7 | 53 | 180…3900 (53 samples; exact list in appendix) |
| `counting-opening-frames305` | 11_counting | `counting-opening-frames305/manifest.json`<br>`9b889e043ce97bfbb2ce8b72f8306c3dbd3718a29c52b97f4adefa2b702bea8a` | 5/5 | 54 | 34-87 |
| `counting-transition-frames305` | 11_counting | `counting-transition-frames305/manifest.json`<br>`9e74486edaaac78d1a4a868f83507a69378f1be9d64ab0456ddd453a34fc64d6` | 11/11 | 81 | 7464-7493, 8294-8344 |
| `residue-transition-frames305` | 12_residues | `residue-transition-frames305/manifest.json`<br>`c6cca51f4290b32b7e4f2587900cdf1adefde66bf48d032b32f322edd684eea2` | 24/24 | 288 | 2475-2516, 2536-2577, 2763-2804, 3025-3154, 3911-3942 |
| `continuation-transition-frames305` | 14_continuation | `continuation-transition-frames305/manifest.json`<br>`a787e39d74c54651b8bbb3a97f0c75e7c863be4ab946c3004bb5aa153e34649b` | 27/27 | 319 | 34-98, 1223-1257, 8876-8940, 10179-10300, 11724-11755 |

Per-sheet hashes and the frames on each sheet are in the appendix and in the JSON.

## Result

**Concrete picture defects observed: none** in the 1,491 pictures viewed.

Low and informational observations (no repair proposed; none blocks anything by itself):

- **O1** (11_counting, frame 4348; preview274 s0003; low): The "|f| =" readout is written without its value at 4348; "|f| = 0.7" is present at 4408. Looks like a transient write-in; motion not checked.
- **O2** (11_counting, frame 7448; preview274 s0005; low): The bottom root marker touches the top edge of the chip "one root inside, at distance 0.335". Both stay readable.
- **O3** (11_counting, frames 8121-8303; preview274 s0012-s0013; counting-transition; low): "0 ÷ 2i = 0, and nothing else lands on 0" stays up after the formula has become "p(z) = zⁿ + ⋯". The sentence is still true but its context has gone. A wording/context note, not a picture fault.
- **O4** (11_counting, frames 9081-9096; preview274 s0020; informational): The counter goes +3.3 then +3.5 between 15-frame samples (+3.4 falls between them). This is a sampling effect. The geometry agrees (walker ≈ 293° at 9186 gives +4.1).
- **O5** (11_counting, frames 9731-11880; preview274 s0024; grid s14-s19; low): The "for programmers" code type is small at 540p. Legibility has to be judged at 4K.
- **O6** (12_residues, frames 2339-3000; preview274 s0002; residue-transition; low): The right text block's right edge is about 4% of the frame width from the edge. It stays inside the frame.
- **O7** (12_residues, frames 2532-3002; preview274 s0004; residue-transition; low): The "residue 1" baseline is about 4% of the frame height from the bottom edge, so a player's captions could cover it.
- **O8** (12_residues, frame 2766; preview274 s0006; residue-transition s008; informational): A faint plate-3 square is already visible at 2766, one frame before the localized H1c start (2767). This is a localization note, not a picture fault.
- **O9** (14_continuation, frames 1080-1200; preview274 s0002; grid s02; low): The top edge of the footer chip "names to search: analytic continuation" meets the plate's bottom edge. The text does not collide.
- **O10** (14_continuation, frames 8876-8877; preview274 s0006 (zoom); continuation-transition (PNG zoom); informational): The lower half of the z-panel's right edge leaves a vertical line at ≤ 2 code values above background for 2 frames. It is gone at 8878 (the blank-frame hash). Invisible at normal contrast.
- **O11** (14_continuation, frames 10260-10278; preview274 s0009-s0010 (zoom); continuation-transition; low): While the Γ box outline is drawn in, its bottom edge carries a faint warm or multicolour tint along its length. It reads uniform grey by about 10290-10301. The cause cannot be resolved from 540p stills: a 1-px plate leak or chroma fringe.
- **O12** (11_counting, frames 1860-2160; grid s04; low): The top edge of the footer "names to search: argument principle · winding number" meets the plate's bottom edge. The text does not collide.
- **O13** (11_counting, frames 3840-4260; grid s08-s09; low): The inset chip "+1 − 1 = 0" overlaps the top-right corner of the output panel border. The text does not collide.
- **O14** (14_continuation, frames 2160-2340; grid s04-s05; informational): "1/(1 − 1.2) =" is held with an empty right side for at least 3 s before "−5" appears. Reads as a deliberate reveal; pacing not judged.
- **O15** (12_residues, frames 3911-3916; residue-transition (PNG zoom); informational): A blocky remnant of the three right-hand text lines stays after their fade: ≤ 3 code values at 3911-3912 and ≤ 2 through at least 3916. Invisible at normal contrast; looks like codec residue.
- **O16** (14_continuation, frames 1253-1254; continuation-transition (PNG zoom); informational): After the I2 plate leave, a horizontal line at ≤ 2 code values stays at the old plate's bottom edge. It is gone at 1255 (the blank-frame hash). Invisible at normal contrast.
- **O17** (11_counting, frame 2280; grid s05; low): Three condition chips stack edge to edge, and the third sits on the output panel's horizontal axis. Still legible.
- **O18** (11_counting, frames 8304-8344; counting-transition s005-s011; informational): During the G3 fade the output panel frame, axes and "0" stay at full brightness while everything else fades. They are reused straight away (counter at 8344), so this reads as a deliberate carry-over.

The maths shown on screen checks out in every place I looked. Examples:
- 11: the turn counts (1, 2, −1, +1 − 2 = −1, 0); |f| = 30 at 0.03 from the pole; Rouché |z⁵ + 1| ≤ 2 < 3 with one root at 0.335; 3⁵ = 243 against a leash of 10; five turns; L × step = 8 × 2π/8 = 6.28 (refuse) and 8 × 2π/64 = 0.79 (pass).
- 12: residue = a₋₁; the residues of f′/f are 1, 2 and −1; Res_{z=i} 1/(1 + z²) = 1/(2i) = −0.5i.
- 14: 1.999023 and 32.15 after 11 terms; 1/(1 − 1.2) = −5; 2047 and −1; ζ(−1) = −1/12 ≈ −0.083333; √z turns at half the angle and reaches −1 after one loop; Γ has poles at 0, −1, −2, …

The quiz answers stay hidden through the think time in 11 (from the predict mark at 5671 to the 11_6 start at 6046) and in 14 (from 5389 to 5704) in the pictures viewed.

## Fades seen frame by frame (transition sets)

**11_counting**
- G1 (36+49): 34-36 blank; plate faint at 37, about full by 70; output axes start at 85-87.
- G2 window (7464-7493): steady quintic scene with no visible plate change; the title starts fading in at 7492.
- G3 (flagged as not localized in the sampling): seen frame by frame in counting-transition. The full scene holds to about 8300. Dimming is first visible at 8301-8304 (the preview274 sample at 8301 is already slightly dimmer; at sheet scale the every-frame window shows it clearly from 8304). The scene is very faint at 8311-8312 and the plate is gone at 8313. A new unit circle fades in from 8314 and the counter appears at 8344.

**12_residues**
- H1a (2474+36), H1b (2535+36), H1c (2767+36): each plate fades in smoothly and its labels follow; earlier plates and labels hold. H1c's first trace is at 2766 (O8).
- H2 (3027+25 ×3): three sequential plate leaves. Plate 1 is gone from 3050, plate 2 from 3075, plate 3 by 3100. 3100-3102 are empty. No plate returns.
- H3 (3102+49): the bump plate fades in from 3103 and is full by about 3140.
- H4 (3913+30): smooth dimming, faint blocky residue at 3941, blank at 3942 (the last frame).

**14_continuation**
- I1 (36+60): 34-36 blank (identical hash); smooth fade-in, about full by 70.
- I2 (1225+30): smooth leave, faint blocky residue at 1253, empty 1254-1255 (O16 is sub-visible); the series formula fades in from 1256.
- I3 (8878+60): 8876-8878 empty at normal contrast (O10 is sub-visible); the lacunary disk fades in from 8879.
- I4 (10206+25): text, red boundary and frame leave 10190-10202, the disk dims 10206-10228, empty at 10229.
- I5 (10231+7): 10230-10239 empty (10238 and 10239 carry the blank-frame hash), so the Γ painter adds nothing visible. The Γ strip fades in from 10240 and the outline draws 10246-10300 (O11).
- I6 (11726+30): full Γ plate with five poles shown as plate singularities; smooth leave; blank at 11755.


## Coverage and gaps

- **11_counting**: 456 distinct frames viewed of 11932 (frames 0-11931). The longest unviewed run is 181-239 (59 frames). No unviewed run reaches 60 frames; the grid's 59-frame steps are the usual gap.
- **12_residues**: 368 distinct frames viewed of 3943 (frames 0-3942). The longest unviewed run is 97-155 (59 frames). No unviewed run reaches 60 frames; the grid's 59-frame steps are the usual gap.
- **14_continuation**: 527 distinct frames viewed of 11756 (frames 0-11755). The longest unviewed run is 181-239 (59 frames). No unviewed run reaches 60 frames; the grid's 59-frame steps are the usual gap.
- Short events that fall wholly inside an unviewed run were not seen. Examples from the samples: the label fade-outs 3003-3025 in 12, the marker and label leave within 11701-11724 in 14, and the scene clear between 7440 and 7464 in 11.

## Still-only limits

- I viewed JPEG contact sheets of decoded 540p60 preview frames. Tiles are downscaled from the extracted pictures, so a defect one or two pixels wide at 4K may not show.
- Sampled coverage. Outside the every-frame transition windows, the frames between samples were not seen. The gaps are listed above and in the JSON.
- No playback, so no judgement on motion smoothness, timing feel, flicker between unsampled frames, or pacing.
- No audio. No listening, no narration or caption sync, no loudness.
- This is the preview274 render, not the 4K final or master. No pixel equivalence to any 4K file is claimed.
- The amplified crops show residues of 1–3 code values that are invisible at normal contrast. I report them for completeness, not as defects.

## Method

- Each sheet was opened as an image with the Read tool and read tile by tile. Batch notes were written as I went (file times 2026-11-08 21:14:28Z to 21:33:55Z); this record is built from those notes and from fresh sha256 of every file named.
- Frame numbers per sheet come from the binding sampling or manifest, chunked by the sheet layout (preview274 and grids 8 tiles; counting-transition 8; counting-opening, residue-transition and continuation-transition 12). The tile labels on the sheets agreed wherever I read them.
- Amplified crops: (v − background) × gain on PNG pictures from the residue-transition and continuation-transition sets: 3911–3916 (right text area), 8876–8878, 1253–1255. Plus earlier amplified crops of preview274 sheets: 12 sheets 0010, 0011, 0013; 14 sheets 0006, 0009, 0010.
- I did not consult the existing `*-transition-review305.json` files or any other review of these chapters, so these readings are my own.
- No renders, no source edits, no TODO or Beads edits, no launch.

## Appendix: sheets viewed (Q-relative path, sha256, frames)

**preview274-review305/11_counting** (binding `eb67bf33a9ad2f7c91c5601026816d7ebd5d1fc3dafb5a68ae3c5004e8307846`)

| sheet | sha256 | frames |
|---|---|---|
| `preview274-review305/11_counting/sheet-0001.jpg` | `ab03d33b4d46633dd85dea3c3c471d6f8d494b906ccddc3a048896ca37a500db` | 0, 35, 36, 60, 66, 84, 85, 96 |
| `preview274-review305/11_counting/sheet-0002.jpg` | `781c1267347aca9cdd313677b8615d55ecc988ee97b3cb323f29f1904aceaa9c` | 115, 156, 1177, 1207, 1237, 1297, 2538, 2568 |
| `preview274-review305/11_counting/sheet-0003.jpg` | `f17b263e8bfafecaab5eaca3e0c1539a15decea5f98c9c5ac95326092c605ca0` | 2598, 2658, 4288, 4318, 4348, 4408, 5078, 5108 |
| `preview274-review305/11_counting/sheet-0004.jpg` | `52dc741209355865936ae0d40828caab8133f280793dc41c9dff5ca519f17bf3` | 5138, 5198, 5671, 5701, 5731, 5791, 6046, 6076 |
| `preview274-review305/11_counting/sheet-0005.jpg` | `3a9e7d51d7c88b5e65973b63d86e0657bbbff29923002d00405e4038fa6947aa` | 6106, 6166, 7448, 7465, 7466, 7478, 7488, 7489 |
| `preview274-review305/11_counting/sheet-0006.jpg` | `5f125e944d13ea49c1690a6bc2bc4908a3e803d04066e02530bb91fa6ff6eb9a` | 7490, 7491, 7492, 7493, 7494, 7506, 7508, 7521 |
| `preview274-review305/11_counting/sheet-0007.jpg` | `a6a48d02dcb4caf9e24c56b01cb5fc256b9b25aa5bfd767a7e2d0b82615fe5c8` | 7536, 7551, 7566, 7568, 7581, 7596, 7611, 7626 |
| `preview274-review305/11_counting/sheet-0008.jpg` | `344a737f18d21661fe690e54f3f508bc1a63422120b9abeda97db4e77f64248b` | 7641, 7656, 7671, 7686, 7701, 7716, 7731, 7746 |
| `preview274-review305/11_counting/sheet-0009.jpg` | `42b470245fe9a203b6bf881f0c9ad020220a740adf3e8ca123dff8985600a116` | 7761, 7776, 7791, 7806, 7821, 7836, 7851, 7866 |
| `preview274-review305/11_counting/sheet-0010.jpg` | `f7eacb089ffbf4c0a1d62b7cbe885928b17f9e992df7a7666dfa0f5c319d2292` | 7881, 7896, 7911, 7926, 7941, 7956, 7971, 7986 |
| `preview274-review305/11_counting/sheet-0011.jpg` | `283c3ce83dc173f64550f798b07575cf926ab4a5fa41d1b75d9cb3841f442aa9` | 8001, 8016, 8031, 8046, 8061, 8076, 8091, 8106 |
| `preview274-review305/11_counting/sheet-0012.jpg` | `e4d7f92df693cf478737bc746abc6fae19dd7d62fdeb3192172b972c52be3c8c` | 8121, 8136, 8151, 8166, 8181, 8196, 8211, 8226 |
| `preview274-review305/11_counting/sheet-0013.jpg` | `2e8bd9cc54fc4b786a270a0afb092b0c9a191b5e26601488ebac7eb578703448` | 8241, 8256, 8271, 8286, 8301, 8316, 8331, 8346 |
| `preview274-review305/11_counting/sheet-0014.jpg` | `b5c6b17ba38e5d94e25aab1712ca0276e830127d891fb482103f1823bcdcd66d` | 8361, 8376, 8391, 8406, 8421, 8436, 8451, 8466 |
| `preview274-review305/11_counting/sheet-0015.jpg` | `0acd25439d9952025b2572e61a045ac718f7250792e6fe3d72eed90b6d665423` | 8481, 8496, 8511, 8526, 8541, 8556, 8571, 8586 |
| `preview274-review305/11_counting/sheet-0016.jpg` | `1ed6e5cd263d50143eb255310cf8389a3cc29c4c458428af70000db969796854` | 8601, 8616, 8631, 8646, 8661, 8676, 8691, 8706 |
| `preview274-review305/11_counting/sheet-0017.jpg` | `3145f9a2883289bac1bd8293e5f47d1feee04d651bf7accd3f977f99e9d5c04b` | 8721, 8736, 8751, 8766, 8781, 8796, 8811, 8826 |
| `preview274-review305/11_counting/sheet-0018.jpg` | `7b18aa6f572994648ad50ff7250e4e0ef629d1267d7c851278e99957789d3003` | 8841, 8856, 8871, 8886, 8901, 8916, 8931, 8946 |
| `preview274-review305/11_counting/sheet-0019.jpg` | `3595a41342a649b36beff612636cc528b40e8fc49312bb3b91ef24d444b9cf11` | 8961, 8976, 8991, 9006, 9021, 9036, 9051, 9066 |
| `preview274-review305/11_counting/sheet-0020.jpg` | `b60abb0dd5e34ed7196f714d99e4f09a9bbf958229bff68dd1ffcafe8170707e` | 9081, 9096, 9111, 9126, 9141, 9156, 9171, 9186 |
| `preview274-review305/11_counting/sheet-0021.jpg` | `76ebecbc83f788fc507daa17861b6dcc3a24034491336c113ff93b746503c099` | 9201, 9216, 9231, 9246, 9261, 9276, 9291, 9306 |
| `preview274-review305/11_counting/sheet-0022.jpg` | `ef19f195ae4897a8a768e4acd393cf56e875ae841dfd0812f78f6cb296856ad1` | 9321, 9336, 9351, 9366, 9381, 9396, 9411, 9426 |
| `preview274-review305/11_counting/sheet-0023.jpg` | `9645fb307877e713af4224e99d62ae47d49f94aff8b23b9ef8b4d576a8e0aa9e` | 9441, 9456, 9471, 9486, 9501, 9516, 9531, 9546 |
| `preview274-review305/11_counting/sheet-0024.jpg` | `e187c0fd942e36c6de709c26bd33780a1e8c0b725ef57bc35a6b09029c32b8b0` | 9561, 9576, 9591, 9606, 9611, 9641, 9671, 9731 |
| `preview274-review305/11_counting/sheet-0025.jpg` | `b54ce6a0e2486764fdc564ab08ded8dae2e7806cc7db25825d9f7ff1f1cffa00` | 11931 |

**preview274-review305/12_residues** (binding `df4ac4872fe729b2cb11b91b44041f143f9cfca707a4872c3056503cda59272e`)

| sheet | sha256 | frames |
|---|---|---|
| `preview274-review305/12_residues/sheet-0001.jpg` | `97251560012a406cc93a7f765a1aed3656a92cbbd806f65c0db40764c5fa17b8` | 0, 36, 66, 96, 156, 1220, 1250, 1280 |
| `preview274-review305/12_residues/sheet-0002.jpg` | `42c3b9347b9f7033b1429ab7f4529cacf3ded49f4484c1c11a820e623affd5fc` | 1340, 2279, 2309, 2339, 2399, 2471, 2472, 2473 |
| `preview274-review305/12_residues/sheet-0003.jpg` | `296012cc688b26583a396ba3a644229fbdb7790e83275083be9d9fff4429b535` | 2474, 2475, 2476, 2491, 2507, 2508, 2509, 2510 |
| `preview274-review305/12_residues/sheet-0004.jpg` | `452a0a8ea46b038b82e244a9931a7fc7f2796f87d9d0bf99de153d21e5a515df` | 2511, 2512, 2513, 2532, 2533, 2534, 2535, 2536 |
| `preview274-review305/12_residues/sheet-0005.jpg` | `73e83513f7391f6fef9da550eac01e54691870155406b7ecd1697bdfadd776c9` | 2537, 2540, 2552, 2568, 2569, 2570, 2571, 2572 |
| `preview274-review305/12_residues/sheet-0006.jpg` | `d21ee1963ecc65243e80b689b9e2e48d87cf39361e6910544c6648839cdf717a` | 2573, 2574, 2601, 2764, 2765, 2766, 2767, 2768 |
| `preview274-review305/12_residues/sheet-0007.jpg` | `33cb0df00636cfe08455907648e43afd2d311db7eac4276e22147e4e3671f8c6` | 2769, 2784, 2800, 2801, 2802, 2803, 2804, 2805 |
| `preview274-review305/12_residues/sheet-0008.jpg` | `3b6a2d8008dc1d99df9f3d788b90b88866d4888928c6b06470959775c049e09a` | 2806, 2833, 3002, 3026, 3027, 3032, 3039, 3049 |
| `preview274-review305/12_residues/sheet-0009.jpg` | `9458245f25d30df5dcaed299ba53723664b7e5d0d94b49834bfc28e2cc7cc469` | 3050, 3051, 3052, 3053, 3054, 3055, 3062, 3064 |
| `preview274-review305/12_residues/sheet-0010.jpg` | `ad3e6dfda7385c6baddd7197729ee257eca091594b28626936ecd0fcb7383d29` | 3074, 3075, 3076, 3077, 3078, 3079, 3080, 3082 |
| `preview274-review305/12_residues/sheet-0011.jpg` | `fcac8060691fa68c570de68e72b6f7223bbe45bc669d7b23e602788696f9c45c` | 3089, 3099, 3100, 3101, 3102, 3103, 3104, 3105 |
| `preview274-review305/12_residues/sheet-0012.jpg` | `5dc9bae9be24d38a4b65b8db89ca857e1edc235ad6d8b5e4727ce968034b2e61` | 3107, 3122, 3126, 3132, 3150, 3151, 3181, 3912 |
| `preview274-review305/12_residues/sheet-0013.jpg` | `ea187a62faadd7ca8e07955a47354c872d17e4858439fcaae3e654aeb3cbd183` | 3913, 3927, 3940, 3941, 3942 |

**preview274-review305/14_continuation** (binding `5faa729974a76a216bfcc5d2a36975df67fabd0bd79a07a1912fc784db5feb2b`)

| sheet | sha256 | frames |
|---|---|---|
| `preview274-review305/14_continuation/sheet-0001.jpg` | `4d4c5c508cf15172255559b0d3348f601521ade33cfc1b6da56dbc701f4058b1` | 0, 35, 36, 65, 66, 95, 96, 126 |
| `preview274-review305/14_continuation/sheet-0002.jpg` | `bcd50a45edaa9e0c15ef757bb3d252965a2873ba265002e7cf6e92a54b834668` | 156, 1200, 1224, 1225, 1230, 1239, 1252, 1253 |
| `preview274-review305/14_continuation/sheet-0003.jpg` | `c5f76b5184029b744148f91b895be0d2f746dae09be2a103205c445ceee250ff` | 1254, 1255, 1256, 1257, 1258, 1260, 1285, 1320 |
| `preview274-review305/14_continuation/sheet-0004.jpg` | `64b5fe7050835c70c4fd61945dd9c46d80794e4b51f17bcc2c325a3ca6db044f` | 2523, 2553, 2583, 2643, 4997, 5027, 5057, 5117 |
| `preview274-review305/14_continuation/sheet-0005.jpg` | `40b850844d2a20033bf8db4ba55635ae452a3da4bf8520a989adbb2575728f29` | 5389, 5419, 5449, 5509, 5704, 5734, 5764, 5824 |
| `preview274-review305/14_continuation/sheet-0006.jpg` | `e67cdaa74b558ed1b22364abb16b0182e2550e0d83885f27285cbd2d274fd133` | 7641, 7671, 7701, 7761, 8853, 8877, 8878, 8883 |
| `preview274-review305/14_continuation/sheet-0007.jpg` | `fb2d9c1dd1a087b45e0b4ee168b65bd7a7e10a619db71713ae78fbd2042bf811` | 8907, 8913, 8937, 8938, 8968, 8973, 10181, 10205 |
| `preview274-review305/14_continuation/sheet-0008.jpg` | `99879c5c411418f34da393d3e42d4eb9ea0ffc6531d0cb8464ed48f589e839c8` | 10206, 10211, 10218, 10226, 10227, 10228, 10229, 10230 |
| `preview274-review305/14_continuation/sheet-0009.jpg` | `e0523e68b88e748aa154ed6b09074f99b8d9a0b9be6e4f1fca50dfc60278fdbe` | 10231, 10232, 10233, 10234, 10237, 10238, 10241, 10261 |
| `preview274-review305/14_continuation/sheet-0010.jpg` | `0bd630c87169dd254ff0544bd39496b291f463eeb59552283214dbb8f978053b` | 10268, 10301, 11725, 11726, 11740, 11753, 11754, 11755 |

**coverage-grid305b/11_counting** (binding `163f404d7d9e0e7e57b02fb9d02a2764115546dd06976bf76369ec1365670f65`)

| sheet | sha256 | frames |
|---|---|---|
| `coverage-grid305b/11_counting/sheet-0001.jpg` | `90de6f24c312d720d7278e3a49901db748cdb752986f921fa81d9a0521f77c2d` | 180, 240, 300, 360, 420, 480, 540, 600 |
| `coverage-grid305b/11_counting/sheet-0002.jpg` | `421e5f2e1e1a3be38c3528866b84b72028b15d47be5ee361f84c8a1ea36202c1` | 660, 720, 780, 840, 900, 960, 1020, 1080 |
| `coverage-grid305b/11_counting/sheet-0003.jpg` | `34965b2d4b6be2f78093d284ae7608c0e1307a67b2debaa04a92b5b43da2bd4e` | 1140, 1320, 1380, 1440, 1500, 1560, 1620, 1680 |
| `coverage-grid305b/11_counting/sheet-0004.jpg` | `fabf1954c026331593bd588fef453d28017f245ff0c28e0eed8fb502240ddcae` | 1740, 1800, 1860, 1920, 1980, 2040, 2100, 2160 |
| `coverage-grid305b/11_counting/sheet-0005.jpg` | `7fa2e781efb51f66811b57bfbe2dc7de6fe1a206174d7c7d861b4b98caf34cff` | 2220, 2280, 2340, 2400, 2460, 2520, 2700, 2760 |
| `coverage-grid305b/11_counting/sheet-0006.jpg` | `ac1eb9414d6138647254439ac907322e2b6e7d2530bb06e0a06526036f752eb8` | 2820, 2880, 2940, 3000, 3060, 3120, 3180, 3240 |
| `coverage-grid305b/11_counting/sheet-0007.jpg` | `9480e1de76a4d1d33a714fb070b56e79cc2fa4e1a50494d0cfcaacd23281fc22` | 3300, 3360, 3420, 3480, 3540, 3600, 3660, 3720 |
| `coverage-grid305b/11_counting/sheet-0008.jpg` | `e4c34792c342da45d4a2743225f1809eafb541f2662c35714eb269f48601feea` | 3780, 3840, 3900, 3960, 4020, 4080, 4140, 4200 |
| `coverage-grid305b/11_counting/sheet-0009.jpg` | `86142b7d96ab923779dbe9e27497583384cb577d04a5eecac5912c703f5bfbf7` | 4260, 4440, 4500, 4560, 4620, 4680, 4740, 4800 |
| `coverage-grid305b/11_counting/sheet-0010.jpg` | `86622369b5b28e1acf8085614903c1efad7edea15af7e691896caa4696bba423` | 4860, 4920, 4980, 5040, 5220, 5280, 5340, 5400 |
| `coverage-grid305b/11_counting/sheet-0011.jpg` | `b8649a79b512129d835207f4e4fa9a59a83e1caa1ecdf3e9448946c062d6853e` | 5460, 5520, 5580, 5640, 5820, 5880, 5940, 6000 |
| `coverage-grid305b/11_counting/sheet-0012.jpg` | `e762bce90ef307439a193fbc15754b416ac300aba94338fb49d48c7632928d48` | 6180, 6240, 6300, 6360, 6420, 6480, 6540, 6600 |
| `coverage-grid305b/11_counting/sheet-0013.jpg` | `64f2c70b2cdb77b48d194da979c093898d39e15691c95df3662ac35a6e06bb20` | 6660, 6720, 6780, 6840, 6900, 6960, 7020, 7080 |
| `coverage-grid305b/11_counting/sheet-0014.jpg` | `7f724f8753661f7a17842ad5b8652e773f867e8d0140c94f5c79f88f1b7ede46` | 7140, 7200, 7260, 7320, 7380, 7440, 9780, 9840 |
| `coverage-grid305b/11_counting/sheet-0015.jpg` | `5ec3ea5d1026533ef8200870cdcb798685c70c3836acad8ae331e9e5d908307c` | 9900, 9960, 10020, 10080, 10140, 10200, 10260, 10320 |
| `coverage-grid305b/11_counting/sheet-0016.jpg` | `92c28b7f2fef1b343a450338390f0af11304963861ac8c8b49231bbec9f0380a` | 10380, 10440, 10500, 10560, 10620, 10680, 10740, 10800 |
| `coverage-grid305b/11_counting/sheet-0017.jpg` | `10d993c180d157285bb298055a1d07c9e005e033bb0513546e42d9ad2fc78362` | 10860, 10920, 10980, 11040, 11100, 11160, 11220, 11280 |
| `coverage-grid305b/11_counting/sheet-0018.jpg` | `d81ab30cf31342f54e95d4c6e9b4a6df96618c814c6e80566e3b6193bfb8325e` | 11340, 11400, 11460, 11520, 11580, 11640, 11700, 11760 |
| `coverage-grid305b/11_counting/sheet-0019.jpg` | `39981c72fa0e6a72381c5ac632f5e9358c8d7b2dd8f227151980d5160b9537be` | 11820, 11880 |

**coverage-grid305b/14_continuation** (binding `163f404d7d9e0e7e57b02fb9d02a2764115546dd06976bf76369ec1365670f65`)

| sheet | sha256 | frames |
|---|---|---|
| `coverage-grid305b/14_continuation/sheet-0001.jpg` | `e5ecf89c599a473a10921ea09100457f8343eb7a9851feb1f4bf08c3846734f5` | 180, 240, 300, 360, 420, 480, 540, 600 |
| `coverage-grid305b/14_continuation/sheet-0002.jpg` | `3bed727757b1f31b570c76685ed4b5df65b817cba1fc1671a75906784bbb6ef5` | 660, 720, 780, 840, 900, 960, 1020, 1080 |
| `coverage-grid305b/14_continuation/sheet-0003.jpg` | `d3217afd4abf9fc2653b691b9fc32a216688ec11aa08df2347089fb53c0d19d6` | 1140, 1380, 1440, 1500, 1560, 1620, 1680, 1740 |
| `coverage-grid305b/14_continuation/sheet-0004.jpg` | `cf11675ff24b0cdef6c5a30bb24a1856319965fe3ddeb361adc24ef718e13007` | 1800, 1860, 1920, 1980, 2040, 2100, 2160, 2220 |
| `coverage-grid305b/14_continuation/sheet-0005.jpg` | `02ea5683de0143998087eec5637d3e4dbb758cdb481871cfdf9b8c57f62a2747` | 2280, 2340, 2400, 2460, 2520, 2700, 2760, 2820 |
| `coverage-grid305b/14_continuation/sheet-0006.jpg` | `8099c5f132c5fb30235837b5d1c2c983d91a9919c661b3467b422f400ac049e9` | 2880, 2940, 3000, 3060, 3120, 3180, 3240, 3300 |
| `coverage-grid305b/14_continuation/sheet-0007.jpg` | `af039c4c94e351f0409d234c427799b9f1511043fdc076c9d41585ee7cecd741` | 3360, 3420, 3480, 3540, 3600, 3660, 3720, 3780 |
| `coverage-grid305b/14_continuation/sheet-0008.jpg` | `e5fb8cd731545bb2ba816dcbb137cb71cb14cab2a58e3e093c60acd5f9302880` | 3840, 3900, 3960, 4020, 4080, 4140, 4200, 4260 |
| `coverage-grid305b/14_continuation/sheet-0009.jpg` | `2ef2af5734737e342daedca7d96ca27debee00aeef973b6c239ce1618fd85d0b` | 4320, 4380, 4440, 4500, 4560, 4620, 4680, 4740 |
| `coverage-grid305b/14_continuation/sheet-0010.jpg` | `31dd9c9993a294e008e73ccabdc18abe70800f11e0c07c915e227bf829e37e42` | 4800, 4860, 4920, 4980, 5160, 5220, 5280, 5340 |
| `coverage-grid305b/14_continuation/sheet-0011.jpg` | `28527f23de3a62a06893f5868796c19408e7ea9019e0e8ff49f71bb0c569bfac` | 5520, 5580, 5640, 5700, 5880, 5940, 6000, 6060 |
| `coverage-grid305b/14_continuation/sheet-0012.jpg` | `d92df56606ad1d5e77d0a8c5373fa61b5181bce5191b5233ef0acd2d13b4084e` | 6120, 6180, 6240, 6300, 6360, 6420, 6480, 6540 |
| `coverage-grid305b/14_continuation/sheet-0013.jpg` | `78d6ce250f8bdf60c9322e0bd3283e352180adccec1e632fbed27699c3444f30` | 6600, 6660, 6720, 6780, 6840, 6900, 6960, 7020 |
| `coverage-grid305b/14_continuation/sheet-0014.jpg` | `a95a2f6c33bc8e33aeac195b39595be788cb9aea57456aef52d4d3a186b8ab72` | 7080, 7140, 7200, 7260, 7320, 7380, 7440, 7500 |
| `coverage-grid305b/14_continuation/sheet-0015.jpg` | `59669e8472c10d9be4490730be67d0710b2dfb8edb52e27397810641982e6607` | 7560, 7620, 7800, 7860, 7920, 7980, 8040, 8100 |
| `coverage-grid305b/14_continuation/sheet-0016.jpg` | `7d7d5540946261a43f2110ee90e1150036cc2d4d5748845bd1719522844dc21d` | 8160, 8220, 8280, 8340, 8400, 8460, 8520, 8580 |
| `coverage-grid305b/14_continuation/sheet-0017.jpg` | `7f90becb3a8687996214dce829726ff745f4e96663f75c86fb3215fa14d9cf31` | 8640, 8700, 8760, 8820, 9000, 9060, 9120, 9180 |
| `coverage-grid305b/14_continuation/sheet-0018.jpg` | `e7689fcd43b76e387eb50994a11f3c5887bae29634dd74b1227ce37a1b69aadb` | 9240, 9300, 9360, 9420, 9480, 9540, 9600, 9660 |
| `coverage-grid305b/14_continuation/sheet-0019.jpg` | `492139faf698873bd23e0239242090514cd387833802dc5abb379d63da7a1d90` | 9720, 9780, 9840, 9900, 9960, 10020, 10080, 10140 |
| `coverage-grid305b/14_continuation/sheet-0020.jpg` | `e661ac4c899c806d718026b057b02a4bd93416850f074e07d3aa4e60a3afebe5` | 10320, 10380, 10440, 10500, 10560, 10620, 10680, 10740 |
| `coverage-grid305b/14_continuation/sheet-0021.jpg` | `79da5dbc1cbbd238ac776d7959116dc871ac7097ba8789173a8a86c48e10a4d2` | 10800, 10860, 10920, 10980, 11040, 11100, 11160, 11220 |
| `coverage-grid305b/14_continuation/sheet-0022.jpg` | `462fb100710f85e27d3621aac9e75a3142b9c85f4a838ca458a763f80a798c5f` | 11280, 11340, 11400, 11460, 11520, 11580, 11640, 11700 |

**coverage-grid305/12_residues** (binding `bd9bd9969c332a18b141947030a37aba1b472191960f0b9844d5f6825ece6c00`)

| sheet | sha256 | frames |
|---|---|---|
| `coverage-grid305/12_residues/sheet-0001.jpg` | `0690a3e1332a21a9724dcd08a840ad038e5dd3ad45efb706f209142bc1867719` | 180, 240, 300, 360, 420, 480, 540, 600 |
| `coverage-grid305/12_residues/sheet-0002.jpg` | `070a6da9548df5f2a19c08bca1d6711d90f3f45f119cb1357042b7e83e9393eb` | 660, 720, 780, 840, 900, 960, 1020, 1080 |
| `coverage-grid305/12_residues/sheet-0003.jpg` | `0ce5f2565b2e5ebb6890045a3ef8ac742a8875f07cc2d36d51a605c5395ebb75` | 1140, 1200, 1380, 1440, 1500, 1560, 1620, 1680 |
| `coverage-grid305/12_residues/sheet-0004.jpg` | `234e33d3201a46ecb02aa6e641a779e7c6b82b078e1e7e0960b445c975fd1e3d` | 1740, 1800, 1860, 1920, 1980, 2040, 2100, 2160 |
| `coverage-grid305/12_residues/sheet-0005.jpg` | `63e9e3daba5a331965449f58e805e465ea5be1ea5004a96a2ce5e3974ff0121c` | 2220, 2400, 2460, 2640, 2700, 2760, 2880, 2940 |
| `coverage-grid305/12_residues/sheet-0006.jpg` | `9ebb9a69f4691ab816b9d04928e08070d169eb1ed680db99b3fb45814fbdfa2d` | 3000, 3240, 3300, 3360, 3420, 3480, 3540, 3600 |
| `coverage-grid305/12_residues/sheet-0007.jpg` | `d1cf62bb208bf1b4072d1c578403bdd4bbe20d2a2abd575d0bb408013a91958a` | 3660, 3720, 3780, 3840, 3900 |

**counting-opening-frames305** (binding `9b889e043ce97bfbb2ce8b72f8306c3dbd3718a29c52b97f4adefa2b702bea8a`)

| sheet | sha256 | frames |
|---|---|---|
| `counting-opening-frames305/sheet-001.jpg` | `3152e94c2eda060ef62510b3e7eaa16974fbc5db5a3e4e53a9525c4b0034a930` | 34-45 |
| `counting-opening-frames305/sheet-002.jpg` | `bae5e39418bf282342182caa36d33d04d8787aa78161181b0061b9bc3b5aa0da` | 46-57 |
| `counting-opening-frames305/sheet-003.jpg` | `2522b9c9724d46e728dd536f0b73951b822cf9c92bbf03097355882923cf8e11` | 58-69 |
| `counting-opening-frames305/sheet-004.jpg` | `413b7a5412850e22fc1113b248575bf11164261266f6b48807d2a65249d39cd7` | 70-81 |
| `counting-opening-frames305/sheet-005.jpg` | `f156b93bd0a96b5923163a755ae10f55a153d19115d647307b23aaecc5715710` | 82-87 |

**counting-transition-frames305** (binding `9e74486edaaac78d1a4a868f83507a69378f1be9d64ab0456ddd453a34fc64d6`)

| sheet | sha256 | frames |
|---|---|---|
| `counting-transition-frames305/sheet-001.jpg` | `92243e680963ef6f70cc94cf67e513c66b55bd26daa62a395d201e766d224e98` | 7464-7471 |
| `counting-transition-frames305/sheet-002.jpg` | `efa35f2cd272dd89fe8d6cfdf1e4acec62ce468024eb712b69b2c7d5f1caf8ec` | 7472-7479 |
| `counting-transition-frames305/sheet-003.jpg` | `34e318181d48e684663e60a72324219efb6052ea3937a87d1c611e9cfa97d518` | 7480-7487 |
| `counting-transition-frames305/sheet-004.jpg` | `04653e90bd40043ff176d9f063bcb77f727d91c59170c5931b7bbb3f39bfd5cb` | 7488-7493, 8294-8295 |
| `counting-transition-frames305/sheet-005.jpg` | `9fbc4bd142a1f2dc1f7826dadeac3d3144e29a7527b584c89904d41ab3f71b58` | 8296-8303 |
| `counting-transition-frames305/sheet-006.jpg` | `c7dbe8067c1d498bd7e144704b7eccf6dc5076429bb94c4f49c0f418176c5d2f` | 8304-8311 |
| `counting-transition-frames305/sheet-007.jpg` | `6dfd101c93392deb9566282ab1562ae8b15027322bdc223ce42a86f3f4eed232` | 8312-8319 |
| `counting-transition-frames305/sheet-008.jpg` | `a619f31246a337f3020d910be95cbabe736120847bfabef538f3445af77d290f` | 8320-8327 |
| `counting-transition-frames305/sheet-009.jpg` | `a46853c970f416cbc0c07195bf5700fd0e06e6382f28ec269944f9044a2dbce6` | 8328-8335 |
| `counting-transition-frames305/sheet-010.jpg` | `5979f012b7eb7cf6d6becb208f8ef4436b3215b3ae6f1c7878402393e4301fbf` | 8336-8343 |
| `counting-transition-frames305/sheet-011.jpg` | `22b2f6c6ac6bd5057eb3db3bcd75516b6cf03b9c61ef73c6f3a76fa9a511a627` | 8344 |

**residue-transition-frames305** (binding `c6cca51f4290b32b7e4f2587900cdf1adefde66bf48d032b32f322edd684eea2`)

| sheet | sha256 | frames |
|---|---|---|
| `residue-transition-frames305/sheet-001.jpg` | `99127088f40d6f0d9ffd4a38c4538ae11b221655ec8eed68bb86141df724bb11` | 2475-2486 |
| `residue-transition-frames305/sheet-002.jpg` | `8f174a6dcfb8a15eb486c63053910927dec4df5c6fdb1c159d41d65e672e5491` | 2487-2498 |
| `residue-transition-frames305/sheet-003.jpg` | `327a28624517a6360e3f5cf0a7ed67f78679ac3647986420c4b5718b1befbfc0` | 2499-2510 |
| `residue-transition-frames305/sheet-004.jpg` | `96cfd54cef0021ffef8f423845b5a78e2a1b22d5dc9cd65814e5f28acb19bd0a` | 2511-2516, 2536-2541 |
| `residue-transition-frames305/sheet-005.jpg` | `ad4075a2f2f5c947c1130b1201db4c3c4ae301004cb7155c5d83c6df01eb4619` | 2542-2553 |
| `residue-transition-frames305/sheet-006.jpg` | `d0caae71fd5c633e1f256ccfeccdc777a10d8ad8cdb5a0ff708e8d0218a20316` | 2554-2565 |
| `residue-transition-frames305/sheet-007.jpg` | `0212c3e0e9aede7e1c607f8227e2386cb1f7514f6abcdcc4b77531029c26cc0e` | 2566-2577 |
| `residue-transition-frames305/sheet-008.jpg` | `aff02d0dac3f03b83b5f8a67eeeb5e73ac4314fb91a823b4b00c076b1ca44fd7` | 2763-2774 |
| `residue-transition-frames305/sheet-009.jpg` | `36c4281eda5de9ec00ac8e75d5b591912552f16786eb483a393c436c8200e162` | 2775-2786 |
| `residue-transition-frames305/sheet-010.jpg` | `4f59ce2b1c42be0d3e5f2263d7c48f5cfbe667b6c441e078ce7f0da74aa6e875` | 2787-2798 |
| `residue-transition-frames305/sheet-011.jpg` | `b97c55819aab43eb4b2c370e29a300eb777b758a75942651f3b2b1a8cf49deae` | 2799-2804, 3025-3030 |
| `residue-transition-frames305/sheet-012.jpg` | `366143f2cbf4981ccd822881fb3d4c61831b05fd1e9e5f0bc5456fa9ff9078d7` | 3031-3042 |
| `residue-transition-frames305/sheet-013.jpg` | `03821425f37ba3851231c65be81e30d280d0f208acff4e84174c4c21c72ca934` | 3043-3054 |
| `residue-transition-frames305/sheet-014.jpg` | `3c1479a844d9b1e152e6881093c3c4125edbbddbb5c9ee240851f3b38c157680` | 3055-3066 |
| `residue-transition-frames305/sheet-015.jpg` | `13072da560b75de79c00c1c8c32dcefa5f549e655d1263d99f8f82f48ddbcd11` | 3067-3078 |
| `residue-transition-frames305/sheet-016.jpg` | `6ca4cc05587a32f99dfe33f9a4b7f8ffb5c321d9421dc693b2ad03384721b2ee` | 3079-3090 |
| `residue-transition-frames305/sheet-017.jpg` | `e9a75b979becab797a56e364c2b22b46d46cf3957c9c85b241b5dc166ad4f46a` | 3091-3102 |
| `residue-transition-frames305/sheet-018.jpg` | `c4ecb781e093adf55874db48abd9f2fc7cc4b8462a65a1a5ca92118c0c8a99cb` | 3103-3114 |
| `residue-transition-frames305/sheet-019.jpg` | `5da562eca4b178594f61b3e0d331c30c4a3b7b2c1248b9d390baec1e85525d8a` | 3115-3126 |
| `residue-transition-frames305/sheet-020.jpg` | `ad828e1dc181e561b93fdede0313517167856d91728bf3cb5c22ac3c75d62e86` | 3127-3138 |
| `residue-transition-frames305/sheet-021.jpg` | `64f690126189816fd36975edea18a05719dc20813d4df8fe10d370a5b6f684df` | 3139-3150 |
| `residue-transition-frames305/sheet-022.jpg` | `012a9400c4df680c164269eb360aaeed0cd345990e5893137a0871a23c76d3e3` | 3151-3154, 3911-3918 |
| `residue-transition-frames305/sheet-023.jpg` | `6a5eea9c50d39b042b17a2b79554716456b82bc26935651a94e5203090f5451f` | 3919-3930 |
| `residue-transition-frames305/sheet-024.jpg` | `184227c29487fc41cc542f960b2b50f5251ff7a12831366e2659ec8c71be7920` | 3931-3942 |

**continuation-transition-frames305** (binding `a787e39d74c54651b8bbb3a97f0c75e7c863be4ab946c3004bb5aa153e34649b`)

| sheet | sha256 | frames |
|---|---|---|
| `continuation-transition-frames305/sheet-001.jpg` | `59e5bf527bbd4687638eacd8bb63678b9b6ceb7cf39e17254dfa39e367a814f7` | 34-45 |
| `continuation-transition-frames305/sheet-002.jpg` | `5cf8300dd78a2a930e39704bf99b52334787bca76a943453bf38774c80fb3409` | 46-57 |
| `continuation-transition-frames305/sheet-003.jpg` | `b08ed0bc0540981a98a948fd285c83e19e2848222db6af0681b3cdd91b56ec9b` | 58-69 |
| `continuation-transition-frames305/sheet-004.jpg` | `c132919eb88d45f4d3bd38abffc7c71070a1d7773b046b968a2dc1760f5b1d5e` | 70-81 |
| `continuation-transition-frames305/sheet-005.jpg` | `b294649e621dabfbe0f939e04f4bbf205f294337b3a454d0ab9bb3b7efbe3298` | 82-93 |
| `continuation-transition-frames305/sheet-006.jpg` | `b0574a43d5cf5cd5a0713a1ec88e03a8850c9393cda721956696fa1b5f8c3ace` | 94-98, 1223-1229 |
| `continuation-transition-frames305/sheet-007.jpg` | `45b2ec81672638cecb7a2779f499089429621d416b006ce5fd2c5641d216ba26` | 1230-1241 |
| `continuation-transition-frames305/sheet-008.jpg` | `43ad85ea865ef4cc21fe00885990446c910c8bce9f1846011a9e4d4fb4dfa8a8` | 1242-1253 |
| `continuation-transition-frames305/sheet-009.jpg` | `e38d621008bc53598fb3396d59cdeb0b5c3591c485305d917bc5ef1ceaea7d1e` | 1254-1257, 8876-8883 |
| `continuation-transition-frames305/sheet-010.jpg` | `98627635b9b2a4eda1902d19c80e762dabcbeb82af3090f56d47456accaf7cd6` | 8884-8895 |
| `continuation-transition-frames305/sheet-011.jpg` | `d5a2bed2fd125943275531e4a142bf2b74a34fe01f07f57d8c572d32e3087ef0` | 8896-8907 |
| `continuation-transition-frames305/sheet-012.jpg` | `2c8d543ce466a8acca9dddb7c0b1ba277194ad1336b95a12f81ac5b05d673455` | 8908-8919 |
| `continuation-transition-frames305/sheet-013.jpg` | `189a86085adb1863b5ffed15dc1473df9d5ef331d3081796b6c2a803ddec6aec` | 8920-8931 |
| `continuation-transition-frames305/sheet-014.jpg` | `5cc74ef42153b8e5c86dacd33ac2a93adb6db71a306b2c34cf966be84faf0511` | 8932-8940, 10179-10181 |
| `continuation-transition-frames305/sheet-015.jpg` | `8381243e7ce38113a1d272b0678a4108a248054e6e37a9ec1cdded8afe3a7811` | 10182-10193 |
| `continuation-transition-frames305/sheet-016.jpg` | `a54523844e00acd6bc1b29d940e3c67bf431361a86ff9284ae1b076af8ff3d6d` | 10194-10205 |
| `continuation-transition-frames305/sheet-017.jpg` | `b8d0da5c64b1deb889025c6ce6079b07da25c8d4ecdf30d9aa65be061152d7cd` | 10206-10217 |
| `continuation-transition-frames305/sheet-018.jpg` | `6d0a06c2701f7e4e62a925285c0e5a395a6015dddfcc8343b401dc50591e6764` | 10218-10229 |
| `continuation-transition-frames305/sheet-019.jpg` | `a50fb1e63f6f849dcd2425b7ffc5bf27f2b8f5d3f83fc12f06bf339088659ae2` | 10230-10241 |
| `continuation-transition-frames305/sheet-020.jpg` | `1e284a33ee8fd873867528bd79123df5d9aaf687c6bc7241cce39748b0095677` | 10242-10253 |
| `continuation-transition-frames305/sheet-021.jpg` | `b76ed53244669e0f946f9ac9d5246c2edc21a21e3b97e4f8b3defddc5816d330` | 10254-10265 |
| `continuation-transition-frames305/sheet-022.jpg` | `14cf308d43ef439add19fbff5679ecbbc06001919c6734ae432ed2adda56ec35` | 10266-10277 |
| `continuation-transition-frames305/sheet-023.jpg` | `460f483ff21f9e4fbddc2589d6197d1ac219636e862b33293f0900c8a5695872` | 10278-10289 |
| `continuation-transition-frames305/sheet-024.jpg` | `9dffa34c4a116af656b8c01ca0ef3b8170bdface9d346c1a9aee96d539fed54c` | 10290-10300, 11724 |
| `continuation-transition-frames305/sheet-025.jpg` | `d2e633f8ff74d8357fa0f3a97b7e5761451e4b35f19100769c8d5882d78e177a` | 11725-11736 |
| `continuation-transition-frames305/sheet-026.jpg` | `060878e50189ad331d096fe3183b87eb220399ed3306a107168b178a03955f52` | 11737-11748 |
| `continuation-transition-frames305/sheet-027.jpg` | `3e557b26ef70b9591e24da24b057a9942c1796ef3cf7b804c77f8f9bad0f266f` | 11749-11755 |

