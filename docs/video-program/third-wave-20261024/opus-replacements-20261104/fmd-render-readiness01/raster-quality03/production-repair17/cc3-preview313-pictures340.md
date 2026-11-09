# cc3 preview313 picture review 340

cc3, 2026-11-08, written at about 20:45 UTC. This is an independent review of the actual preview313 pictures of chapter `09_boundary`, read from the guarded local copy `Q/preview313-review333/`. Here Q is `docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17`, and D is the job root on the build host.

Root's 341 and 343 reports reached me while I was writing. I did not read those files, and nothing below relies on them.

**What this is not.** These are 540p stills, plus a few frames I extracted from the 540p preview movie myself. I make no motion, continuous-viewing, listening, fine-sync, pronunciation, final-AV or 4K acceptance. No audio was heard or checked.

## 1. Answer

- **The repair works in every picture I viewed.** Repair310 is meant to make the two gray zoom-link lines leave with the hole name "g is undefined at a":
  - the lines draw in at 28.5 s and hold outside the proof column;
  - they dim with the hole name from frame 2088, and nothing measurable is left of them at frame 2099 (34.98 s);
  - they are absent when the bound arrives (2326, 38.77 s), at every old crossing (69.67, 90.28, 104 and 115–117 s), throughout the 39–133 s sweep, and across the proof_gone gap (7980–8333).
- **Concrete visual defects remaining: none in the repair's scope.**
- **Two Low layout contacts outside the scope.** They sit in the same scenes; the repair did not touch them and the lint did not flag them (L1, L2 in §5). I did not check whether they predate 313.
- **Bounded disposition.** At 540p and at the frames listed here, preview313 gives no picture reason to hold the 4K render for the zoom-link repair. That is not acceptance of motion between samples, of the 4K pixels, or of audio (§6).

## 2. Inputs and records

**Copy.** `Q/preview313-copy-inventory333.json` lists 368 files: 5 at the top level, 44 in `09_boundary/`, 313 in `09_boundary/pictures/`, 5 in `preview/`, and 1 soundtrack. I hashed the local tree:

- 368 of 368 files match the inventory;
- the tree holds no other files;
- the inventory records `terminal_success_verified` true.

| File (under `Q/preview313-review333/`) | sha256 | Equals my host read |
|---|---|---|
| `resource-receipt.json` | `8c88c62aba2e934a98ed4df19733eedf44942955fded9b7c81ebcc9883a2bb45` | yes |
| `summary.json` | `d3552f32d2cc35fedfb18ee10ea1d7205c9d96454527b1f894ec1fbae18b8c8a` | yes |
| `09_boundary/zoom-probe.json` | `e2d05d81ebdbbeabe03db8afd470b7ee22648c9be5fa819de90c1d22977c2e99` | yes |
| `09_boundary/sampling.json` | `4e7ab1d8d2e96b61e2dda0ef051964fe4455085fe587183d3d1f12575c82c0a5` | yes |
| `preview/09_boundary.mp4` | `5e50811b77c075ae638c3d539775ccb29134504152a60f5e327ca28940cb3a1c` | yes; also equals `movie_sha256` in sampling and the probe |

**Receipt** (`D/jobs/csd-review-repair-preview313`):

- started 19:37:37 and finished 20:25:04 UTC;
- admitted, exit 0, phase native, no error;
- inputs = final inputs = `3ca1a2949bd4…`;
- 267 resource samples, minimum free space 98.46 GiB.

The inputs manifest `Q/review-repair-preview313-inputs.json` has 12,903 rows, all present and unchanged when I read them on the host.

**Summary:**

- binary `1a82f72c53db…`;
- 16,504 frames, 313 samples, 40 sheets;
- 2 plate plays, both exact; not_localized 0; flags none;
- `zoom_probe_exit_frame` 2094, `zoom_probe_fails` none.

**Zoom probe.** This is pixel arithmetic on 21 points outside the column, read on every frame from 1500 to 8400. It is not viewing.

- In the control movie (`dba964d2…`), the lines are present from 1707 to 7988.
- The exit window is 2086–2100. In the new movie:
  - frames up to 2094 have 12 + 9 points lit;
  - 2095 has 8 + 5;
  - every frame from 2096 has 0 + 0.
- Seven anchors (2326, 4180, 5417, 6240, 6900, 6960, 7020) all read new 0 + 0 against control 12 + 9.
- Nothing is flagged.
- After 2115, at most 1 point is lit, in 401 frames, all from frame 8000 on, where the control has the same point.

**Sampling:**

- `fade_flags` is empty.
- E1 (`boundary.rs:474`): starts at frame 36, 60 frames, exact.
- E2 (`boundary.rs:1402`): starts at frame 16474, 30 frames, exact.
- The timeline, cue and cuelog tables equal timeline313's, and equal timeline269's after path mapping.

**Layout lint.** Two lines, the same two as preview274's:

- `09_3 +6.8s @156.8s behind`: a stroke of an unlabeled shape (x 2.80..2.82) passes under "opposite samples share a value".
- `09_7 +0.5s @223.2s tight`: "x" stands 0.18 from the right edge, where a text wants 0.25.

`native.stderr` is 141 bytes ("LAYOUT 09_boundary: 2 findings"); `command.log` is empty.

## 3. What I viewed

- **All 40 sheets, read as images in order.** They hold all 313 pictures: 39 sheets of 8 and 1 of 1. A sheet caption shows only a picture's first reason, so I mapped every slot to its frame, time and full reason list from `sampling.json`.
- **14 pictures at full size (960×540), read one by one:** frames 2092, 2094, 2095, 2096, 2098, 2100, 2326, 4180, 5400, 6240, 6900, 7020, 7980 and 9408.
- **Enlarged crops I made from the pictures:**
  - the hole name at 2093–2098, at 3× size, plain and at 4× brightness;
  - the line region at 2094–2101, at 3× brightness;
  - the label and card at 5400, at 5× size;
  - the end of the bound formula at 6240, at 5× size.
- **Frames I extracted from the local movie myself.** I decoded them to PNG with ffmpeg; they stay in my scratch area and are not committed. Of these, I viewed:
  - a 10-frame montage of 7980, 7988, 7996, 8004, 8012, 8020, 8028, 8040, 8100 and 8333;
  - the hole name at 2094, 2096, 2097 and 2098, at 4× brightness;
  - a montage of 14808, 14810 and 14812–14815.
- **Checking the extracted frames line up with the pictures.** During a fade, line brightness differs from frame to frame, so I compared it between my PNGs and the pictures:

  | Frame | My PNG | Same-numbered picture |
  |---|---|---|
  | 2092 | 51 | 50 |
  | 2094 | 28 | 28 |
  | 2096 | 6 | 6 |

  The values are the 90th percentile defined below. The neighbouring pictures read 39 (2093) and 17 (2095), so each PNG matches its own frame, not a neighbour.
- **My own line-path measurement (separate from the probe).**
  - It uses 78 points along the two drawn lines, 39 per line. The upper line runs from (447, 225) to (641, 199) at 540p, and the lower line from (447, 384) to (641, 467).
  - Each point's value is its luma minus the darker of the two pixels 6 px above and below it.
  - I measured all 216 pictures from frame 1680 to 8393, on 0–255 luma.
  - "Brightness" below is the 90th percentile of the 78 values.

## 4. Results at the assigned checks

| Check | Pictures | What the pixels show |
|---|---|---|
| Lines draw in | 1680–1770 (28.0–29.5 s) | Absent at 28.0 and 28.25 s. Drawing at 28.5 s (brightness 42). Full from 28.75 s (76–79). The hole name and close-up are held from 29 s. |
| Hold | 1785–2087, including the 33.48 s picks 1997, 2009, 2021, 2069 | Both lines run from the plate edge to the close-up and stay above and below the column's content. Brightness 76–79 throughout. |
| Exit | every frame 2080–2109; full size 2092–2100 | The lines and the hole name dim together from 2088. Brightness: 76 at 2088, 50 at 2092, 28 at 2094, 17 at 2095, 6 at 2096, 2 at 2098, and ≤ 2 (median 0) from 2099. The hole name is fully legible to 2094, faint at 2095, barely visible at 2096–2097, and gone at 2098. "z → a: g(z) → f′(a)" begins fading in at 2100. |
| Bound arrival | every frame 2316–2336, full size 2326 | "\|g\| ≤ M" and "inside the dashes" fade in from 2327, with no line in any frame. Brightness stays ≤ 1; the only bright points are where the new text crosses a line path. |
| 69.67 s | 4168, 4180, 4192, 4240, 4300; full size 4180 | Column clear: "\|g\| ≤ M", "inside the dashes", "inside edges cancel: walked both ways". |
| 90.28 s | 5405, 5417, 5429, 5477, 5537; full size 5400 | Clear: "\|g\| ≤ M", "inside the dashes" and the "same endpoints" card (see L1). |
| 104 s | 6228, 6240, 6252, 6300, 6360; full size 6240 | Clear: the bound formula (see L2), "w: how much g varies", "L: arc + chord lengths". |
| 115–117 s | 6888, 6900, 6912, 6948, 6960, 6972, 7008, 7020, 7032; full size 6900, 7020 | Clear: "for every small ϱ" and the arrowed circles. |
| Sweep 39–133 s | one picture per second (largest gap 60 frames) through 7980 | No line in any picture. In all 166 pictures from 2099 to 8393, brightness is ≤ 2 and the median is 0. |
| proof_gone gap | pictures 7980 and 8333; my PNGs every 4th frame 7980–8040, plus 8100, 8200, 8333 | The proof is at full opacity at 7980, fading at 7988, and gone at 7996. "The second piece: its loop integral is 0" and "∮ f(z)/(z − a) dz = 2πi f(a)" fade in from 8004. No line reappears. From 8000 one of my 78 points is lit by the new content while the 90th percentile stays 0, which is consistent with the probe's single shared point. |
| Rest of the chapter | sheets 28–40 | The lint picks, line and mark starts, the 09_6–09_9 scenes, the E2 fade and the last frame show no zoom-line content. At E2, frame 16473 shows the plate alone, and 16474–16503 ramp it down to black at 16503, the last frame. |

## 5. Findings

**High 0, Medium 0, Low 2. None is in the repair's scope.**

### Low

**L1. The "inside the dashes" caption sits on the top border of the "same endpoints" card.**

- **Where.** The gray caption under "|g| ≤ M" has ink in rows 242–250 at 540p. The card's top border is at about row 252, so the gap is 0–1 px (5× crop of 5400).
- **When.** Measured in the pictures at 5280, 5400, 5537 and 6240 (88–104 s), with the same rows in each. The card is gone by 6900.
- **Status.**
  - The lint does not flag it.
  - It does not involve the zoom-link lines.
  - I did not view the control movie or preview274 at these frames, so whether it predates 313 is not established.
  - Moving the caption or the card would need a source change and a re-render. None has been requested.

**L2. The long bound formula ends close to the right edge.**

- **Where.** "|∮circle g dz − ∮polygon g dz| ≤ max w · total L → 0" has its rightmost ink at x = 942 of 960 at 540p. That leaves 17 px, about 0.25 frame units, which is right at the lint's title-safe threshold, so it is not flagged.
- **When.** On screen at about 103–108 s (pictures 6228–6360).
- **Status.** Cosmetic. Whether it predates 313 is not established, for the same reason as L1.

### Observations (not defects)

**O1. The hole name's fade tail shows block noise.**

- At frames 2096–2097, 4× brightness shows block noise in the faint hole name.
- My PNG decode of the movie shows the same noise as the JPEG pictures, so it comes from the 540p preview encode.
- At normal brightness it is a faint smudge for two frames, about 33 ms.
- The 4K encode was not examined.

**O2. The lines outlast the hole name by about one frame.**

- At 2098 the line paths still measure 2 while the hole name is no longer visible.
- This is not visible in the stills, and the probe reads 0 + 0 from 2096.

**O3. Both lint lines appear in the pictures, unchanged from preview274.**

- 09_3: pictures 9396–9468.
- 09_7: pictures 13380–13392, where the axis "x" sits near the right edge.
- I did not resolve at 540p whether the axis shows through the 09_3 caption's backing.

## 6. Disposition: preview to 4K (bounded)

**(a) Concrete visual defects remaining, seen in pixels.**

- None in the zoom-link repair's scope.
- L1 and L2 are Low layout contacts outside that scope.
- Holding the 4K for them, or fixing them, is root's call. Either would need a new source change and a re-render, which nobody has requested.

**(b) Established for the repair, at 540p, in the frames listed in §3–§4:**

- the lines are present and outside the column from 28.75 s until the exit;
- they fade with the hole name over 2088–2098;
- they are absent in every viewed picture and extracted frame from 2099 to 8393, including every old crossing and the 7980–8333 gap.

**(c) Not observed. These are gaps in scope, not defects.**

- **Motion between samples.** 313 stills cover 275 s. Between 2109 and 7980 the pictures are mostly one per second.
  - The probe reads every frame on 21 points and found at most one lit point after 2115, which the control shares.
  - That bounds a reappearance on those points, but it is arithmetic, not viewing.
- **Continuous playback.** Root's 343 QuickTime screenshots, as root describes them, are also sequential stills.
- **The 4K render's own pixels.** The 4K is a different scale and encode, so 540p cannot stand in for it.
- **Audio, sync and final AV.**

**(d) What should happen before the 4K is accepted.** Its own pictures should be viewed at:

- the exit, frames 2088–2100;
- 2326;
- the old crossings 4180, 5417, 6240 and 6900–7020;
- the 7980–8040 gap.

Listening, fine-sync and final AV remain open.

## 7. Not established

- No motion, continuous-viewing, listening, fine-sync, pronunciation, final-AV or 4K acceptance.
- Picture judgements cover only the frames named above, at 540p.
- Only chapter 09 is in this preview; the other 14 chapters were not re-run under binary 313.
- I did not view the control movie or preview274's pictures, so the pre-existing status of L1 and L2 is unverified.

## 8. Method and hygiene

- I worked read-only, with `python3 -I -B` and PIL on the local copy, plus an ffmpeg decode of the local preview movie into my scratch area.
- My scripts, crops and extracted frames stay in my session scratch area and are not committed.
- This file is my only write in Q.
- No job, launch, transfer, permit, source, candidate, TODO, Beads, render or memory change.
- 308, 315, 316, 320 and 324 were left as they were.
