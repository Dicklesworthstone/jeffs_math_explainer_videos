# Independent final995 picture review, sheets 0262–0522 (CC1)

Reviewer: CC1 (BronzeJaguar). This review is independent of root's; root reviews separately.
Date: 2026-11-10.

## What was reviewed

- **Master:** complex993, sha256 `80244e88ca5970d0da07a5d96d6d825ae1851bae703d75bd200f8810b04e7b90`.
- **Sheets:** `final-review995/overview/sheet-0262.jpg` to `sheet-0522.jpg`, with `sampling.json` sha256 `c211e7d571a9ddd67f655c9b5070ff568555723b4e30f92891ee224b4b64626f`.
- **Coverage:** **all 261 sheets of the assigned range**, 0262 to 0522, with none missing, which is 2088 sampled stills. 0 sheets remain.
- **Method:**
  - I opened each sheet as an image with the Read tool and read it slot by slot against the frame, chapter and reasons for that slot in sampling.json.
  - Every sheet's sha256 equals its sampling.json entry. merge-progress.py asserts this for every viewed sheet.
  - Where a note says so, I cropped and enlarged a tile (`crops/`) or measured its maximum deviation from background (tile995-maxdev.py).
- **Evidence:**
  - Final progress file: `final995-cc1/progress-016.json`, sha256 `0db18a32b0458ed33bdee724a5cf1b0b3b63b5e2e9ca554cafad082b3054f918`. It lists each viewed sheet with its sha256, its 8 master frames, its chapter(s) and my note, plus every finding.
  - Earlier versions are kept: progress-001 to progress-015, batch-001 to batch-016, and batch-013-draft-a.json.bak.
- **Chapters in range:** the end of 07_one_point (07_5/07_6), all of 08_loop_sums from about L3700 to its last frame L14958, and 09_boundary L0–L587 (the 09_1 opening).

## Limits (read these first)

These are 960x540 stills of the 993 master. This review is **not**:
- listening, or any audio check;
- motion review;
- fine sync, or any check of picture against narration;
- native 4K detail;
- whole-film AV acceptance or publication acceptance.

Durations come from sampled frame numbers at 60 fps. They are bounds, not every-frame measurements, except where a note says every frame was read (for example 08_5 L10200–10380 on sheets 0415–0437). Whenever a finding says the narration might cover something, I can't tell from stills whether it does.

## Findings, most important first

All frames are master frame numbers. L = chapter-local frame: 08 starts at master frame 52106 and 09 starts at 67065.

### Construction and labelling of the square proof (08_5)

**L12, construction fidelity. Root viewed 0434 and 0445 and confirms it. I would fix this first.**
- Where: sheets 0444–0453, frames 62900–62927 (L10794–10821) onward.
- Under "Two halves make one quarter", the next square is drawn concentric with Q_n, centred on c, at half the side (`crops/s0445-slot1-inner-square.png`).
- The bisection the proof uses produces four quarters, each sharing a corner of Q_n. None of them is concentric with Q_n.
- The sizes are right (half the diagonal and half the perimeter, so D_n P_n drops by a factor of 4).
- But the picture shows a square the construction never makes. The `|I_{n+1}| ≥ |I_n|/4` line added at L11141 refers to a chosen quarter that is never drawn.
- A repair proposal is in `l12-repair/`.

**L11, label placement.**
- Where: sheets 0411–0418 and 0432–0437, frames 62445–62459 (L10339–10353).
- The green z–c segment, with z at the bottom-left corner, covers the whole lower half of the teal diagonal. The visible teal therefore runs only from c to the top-right corner, and "D_n" sits beside that half (`crops/s0434-slot1-Dn-Pn-labels.png`).
- The picture reads as D_n = |z − c| = half the diagonal. The bound |z − c| ≤ D_n stays true, but the picture suggests equality and a halved D_n.
- "P_n" is centred under the bottom side, where it reads as that side's length.
- **Resolved:** on none of the sheets 0434–0522 does text define P_n (or D_n) in words. The column shows only `P_n = P_0/2^n` and `|I_n| ≤ εD_nP_n`.
- The triangle version (0498) puts "D_n" on the longest side, which is correct for a triangle's diameter. P_n is still not named there.

**L5, labelling.**
- Where: sheet 0357 onward, frames 59102–59166 (L6996–7060).
- The green "c" sits about 64 tile px from its dot, outside the level-2 and level-3 squares and across two of their edges, with no leader line (`crops/s0357-slot4-c-label.png`).

**L8, legibility.**
- Where: sheets 0382–0389, frames 60572–60612 (L8466–8506).
- In the green "exact changes" chain, the third step runs within about 4 px of the long return step. In the held state the four-step loop reads as three strokes (`crops/s0383-slot1-green-chain.png`).

### Pacing and composition

**L2, pacing and pedagogy.**
- Where: sheets 0299–0310, frames 55890–56497 (L3784–4391).
- "A closed square, with a little room beyond" stands for 10.1 s before any square is drawn. The left two-thirds of the frame measures empty in all 85 stills in between.

**L7, fade continuity.**
- Where: sheet 0380, frames 60412–60437 (L8306–8331).
- The caption "Exact changes, added tip to tail" does not change, yet it fades out and back in across the picture swap. Its maximum deviation from background goes 132 → 13 → 130 over about 0.5 s (`crops/s0380-slot4-caption-dip.png`).

**L10, notation.**
- Where: sheets 0392–0405, frames 61075–61153 (L8969–9047). The same pattern recurs on 0472–0482: "for every epsilon > 0" under `|I_0| ≤ εD_0P_0`.
- The text says "Choose any epsilon > 0", while the formula right below it uses ε.

**L3, legibility.**
- Where: sheets 0310–0316, frame 56533 (L4427).
- The square's arrowheads overlap its outline in the same yellow, so the heads fuse with the edge (`crops/s0311-slot1-right-edge-arrow.png`).
- Same class: sheet 0497 (L13859 onward). The gold outer arrows on the triangle touch the midpoint dots (`crops/s0497-slot8-outer-arrows.png`).

### Typography (one glyph-face class)

- **L4.** Sheets 0337–0341, frames 58134–58135. The ≥ in `|I_{n+1}| ≥ |I_n|/4` is heavy and looks sans-serif next to thin Computer Modern (`crops/s0337-slot8-geq-formula.png`).
  - **L4-ext:** the ≤ on sheets 0342–0347 (frame 58426) has the same look.
  - Recurs at 0439 (≤), 0450 (≥) and 0455 (≤).
- **L6.** Sheets 0366–0373, frame 59666. The minus in `(z − c)` is a short, heavy dash that looks like a hyphen (`crops/s0367-slot3-taylor-formula.png`).
- **L9.** Sheets 0390–0405, frame 61158. The ∂ in `∮_{∂Q_n}` is upright and heavy, and so is the ≤ in `|r(z)| ≤ ε|z − c|` (`crops/s0393-slot1-In-epsilon.png`, `s0395-slot1-r-bound.png`).
  - Recurs in `∮_{∂T}` at 0504 (`crops/s0505-slot1-goursat-statement.png`).
- **L1.** Sheets 0289–0290, frame 51545. The card sets U upright while the figure uses italic U.
  - Recurs at 0504 (upright f and T in "f holomorphic on a neighborhood of T") and at 0507 (f′ on the "precisely:" card).
- In every case the meaning stays clear.

### Observations (not graded)

**O1.**
- Where: sheet 0288, frames 50968–50972, between 07_5 and 07_6.
- The screen is fully dark for at least 1.95 s.
- Root may want to check this against the cue timeline.

**O2.**
- Where: sheets 0310–0325, L4392–5296.
- No "room beyond" margin is drawn around the square.

**O3.**
- Where: sheet 0334, frames 57917–57921.
- A bare green `|I_0| > 0` replaces `∮ f dz = 0`, and no word on screen marks it as a supposition.

**O4.**
- Where: sheets 0366–0374, L7552–7985.
- The right two-thirds of the frame is empty for about 7.2 s.

**O5.**
- Where: sheets 0395–0407, L9156–9768.
- A static hold of about 10.2 s.

**O6.**
- Where: sheets 0408–0421.
- The δ-circle is not drawn in the magnified view, while `|z − c| < δ` stays in the text.

**O7.**
- Where: sheets 0455–0469, frames 63517–64388 (L11411–12282).
- `|I_0| ≤ 4^n|I_n|` stands alone for about 6.9 s (L11418–11831).
- The picture is then static again for about 7.1 s (L11856–12282).

## What checked out (sheets 0470–0522)

- **The square proof's conclusion is correct throughout:**
  - the green slashes cancel the two 4^n's, and both stay legible (0470);
  - `|I_0| ≤ εD_0P_0`, then "for every epsilon > 0", then gold `I_0 = 0` (L12756–12770);
  - the cross-fade to "The same proof, now on a triangle" leaves no residue (L13064–13083).
- **The triangle subdivision is correct:**
  - the midpoint dots sit on the true edge midpoints, measured (0485);
  - the four sub-walks run counter-clockwise, the arrows on each interior edge point opposite ways, and the outer walk runs counter-clockwise (0492);
  - "The middle walk is also counterclockwise" agrees with the arrows (0493–0494).
- **The diameter is marked correctly.** "D_n" goes on the longest side (298 vs 273/262 displayed px, 0498).
- **The statements are correct:**
  - `∮_{∂T} f(z) dz = 0` with its hypothesis (0504);
  - the "precisely:" card (0507).
- **The 08→09 seam is a normal chapter break.** It is near-black from frame 67062 to at least 67101, so at least 0.67 s and at most 2.5 s (0508–0509).
- **The 09_1 opening is correct:**
  - the plate is uniform near a and has two ringed singular points;
  - the yellow circle and the grey "a little beyond" circle both clear the singular markers by about 17 px (`crops/s0513-slot8-a-label-circles.png`);
  - f(z)/(z − a) fades in at L414–428, clean.
  - Note: the plate and card hold alone for about 3.9 s (L180–414) with the right half of the frame empty. This is not a finding.
- **Stale layout flags show nothing.** The current-layout and previous-layout persistence samples in sampling.json cite cancel strokes recorded at 205.6 s and an old stroke through "a" at 09_1. Every such sample in 0470–0522 shows a clean frame.

## Files

- Report (this file): `independent-final995-cc1-pictures.md`.
- Progress: `final995-cc1/progress-016.json`, the final version; versions 001–015 are kept.
- Batch notes: `final995-cc1/batch-001.json` to `batch-016.json`.
- Crops: `final995-cc1/crops/`.
- Read-only aids: `sheets995-list.py`, `crop995.py`, `tile995-maxdev.py`, `final995-cc1/merge-progress.py`, `reason-grep.py`, `dump-findings.py`.
