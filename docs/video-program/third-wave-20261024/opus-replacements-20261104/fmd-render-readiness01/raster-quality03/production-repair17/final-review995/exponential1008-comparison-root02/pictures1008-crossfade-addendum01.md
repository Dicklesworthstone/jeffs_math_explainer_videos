# pictures1008 vs 985, chapter 02 frame 11020, chapter 05 successor (addendum01)

CC1 scratch, session b0a72b9b, 2026-11-11 UTC. Read-only. I launched no job, render, decode or upload, and did no listening. This addendum follows `exponential-publication-review01.md` (229b32ec, SUMS02) and does not edit it.

Authorship limits:
- I wrote `pictures1008.py`, `compare1008-985.py`, `dump1008-records.py` and the small scripts named here.
- The independent picture review of the same images is BlackThrush's 985 review.
- My own viewing below is not independent of that tooling.

## 1. Byte comparison of actual 1008 output with 985 (final: dump `records1008-06.json`, 00:25:21 UTC)

`compare1008-985.py` (9a7838cb; self-test PASS) was run against verification 6c273f16974dd3ac0f0363722be9ef308ed4f58a3d5809837acb3aaaf47f7909. Logs: `compare1008-985-run02.log` to `run06.log`.

The 985 review columns:
- Each 985 review (root BlueSalmon and independent BlackThrush) covered all frames and sheets of its chapter with 0 concrete defects. Each binds the `completed-pictures985-NN.json` seed sha that verify1005 binds (`review985-coverage01.log`).
- Root's files carry `safe_to_master` false because their scope is the chapter only and whole-master acceptance was still pending. BlackThrush's carry true, scoped to that chapter's sampled-picture gate.

| Chapter | 1008 pictures = 985 bytes | Sheets | Local frame lists = 985 | Sheet bytes = 985 | 985 root / independent coverage |
|---|---|---|---|---|---|
| 01_rate_law | 64/64 | 8 | 8/8 | 0/8 | 64/64, 8/8 each |
| 02_factorials | 172/172 | 22 | 22/22 | 0/22 | 172/172, 22/22 each |
| 03_product_grid | 150/150 | 19 | 19/19 | 0/19 | 150/150, 19/19 each |
| 04_tiny_multipliers | 68/68 | 9 | 9/9 | 0/9 | 68/68, 9/9 each |
| 05_whole_sum | 246/246 | 31 | 31/31 | 0/31 | 246/246, 31/31 each |
| 06_only_solution | 236/236 | 30 | 30/30 | 0/30 | 236/236, 30/30 each |
| 07_calculating_ruler | 130/130 | 17 | 17/17 | 0/17 | 130/130, 17/17 each |
| 08_decay_law | 114/114 | 15 | 15/15 | 0/15 | 114/114, 15/15 each |
| 09_charging_gap | 473/473 | 60 | 60/60 | 0/60 | 473/473, 60/60 each |
| 10_changing_rates | 154/154 | 20 | 20/20 | 0/20 | 154/154, 20/20 each |
| 11_money_limits | 120/120 | 15 | 15/15 | 0/15 | 120/120, 15/15 each |
| 12_chance_and_limits | 94/94 | 12 | 12/12 | 0/12 | 94/94, 12/12 each |
| 13_turning_rate | 546/546 | 69 | 69/69 | 0/69 | 546/546, 69/69 each |
| 14_euler_identity | 559/559 | 70 | 70/70 | 0/70 | 559/559, 70/70 each |
| 15_logarithm_bridge | 37/37 | 5 | 5/5 | 0/5 | 37/37, 5/5 each |
| **All 15** | **3163/3163** | **402/402** | 402/402 | 0/402 | 3163/3163 frames, 402/402 sheets, each review |

**Final (dump `records1008-06.json` at 00:25:21 UTC, `compare1008-985-run06.log`): compare exit 0, hard problems 0, pictures differing from 985: 0.**
- All 15 chapter receipts are stable successes with matching stream, extract source, offsets, frames, reasons, canonical paths and sheet frames.
- The recheck receipt agrees field for field with root's preserved `exponential1008-recheck-terminal-root01/resource-receipt.json` (c2f3c749…):
  - admitted;
  - finished 2026-11-11T00:24:29.184077Z;
  - exit 0, no resource stop, no error;
  - inputs and final inputs 4fa1894bd6cf1d28487a5bbac9cd194e89933e414c2f773cca4caa5a34dabcc2.
- The recheck record sha in the dump, cc00e6005f278ac1187f76b00bc4d4fc238446cbead49c82e4281cc261625034, equals root's preserved `recheck1008.json`. It records:
  - samples 3163;
  - records_exact true;
  - 15 chapters;
  - upstream_rows 151719, unchanged;
  - verification 6c273f16…, verify inputs 5d45eac1….

Intermediate dumps are kept unchanged:

| Dump | Time (UTC) | Chapters finished | Pictures same / total | Sheets |
|---|---|---|---|---|
| `records1008-02.json` | 00:03:49 | 01–05 | 700/700 | 89 |
| `records1008-03.json` | 00:06:38 | 01–06 | 936/936 | 119 |
| `records1008-04.json` | 00:11:42 | 01–09 | 1653/1653 | 211 |
| `records1008-05.json` | 00:18:09 | 01–12 | 2021/2021 | 258 |

The sheets are not byte-equal to 985's. That is by design: `pictures1008.py` line 28 has the sheet labels carry the joined-master frame number. Every 1008 sheet holds the same eight local frames as its 985 sheet (402/402), so its tiles are cut from byte-identical pictures. But the 1008 sheet files themselves are new images that no 985 review saw. Root's view of every 1008 sheet is the review of those files; root reports chapters 01–08 viewed so far (1180 pictures, 151 sheets). The byte-identity statement covers the pictures only.

## 2. Chapter 02, master frame 11020 (183.667 s), sheet 010

### Identity
- Master frame 11020 is chapter 02 local frame 5100 (offset 5920): 85.000 s into the chapter, 183.667 s into the master.
- It is picture 75 of 172, `frame-000075.jpg`, slot 3 of sheet 010.
- Sheet 010 frames:
  - master 10420, 10720, 11020, 11228, 11258, 11288, 11318, 11320;
  - local 4500, 4800, 5100, 5308, 5338, 5368, 5398, 5400.
- Selection reason: `chapter overview every5s`. This is the fixed 5-second grid sample, not a frame chosen for this transition.
- Picture sha 123fd656e320e6df539294f600e64483ce035044389cbbb2c138e62106ae971b in both 985 and 1008. My read-only copy in `frame11020-01/` matches.
- Sheet 010:
  - 1008: 71df8362187afb99f87ecd68a98bf3325c0278025bcd39b6010f1d4a5dc6e00e.
  - 985: 288ec274f703c22cdf8e6b22e5dc9bb43acb71b33ef113b3194528cea8afa27e. This is exactly the sheet hash in BlackThrush's `viewed_sheet_hashes`, with frames [4500, 4800, 5100, 5308, …].
- Neighbouring selected pictures:

  | Picture | Local frame | Master frame | Master time | Distance from 5100 | sha |
  |---|---|---|---|---|---|
  | 74 | 4800 | 10720 | 178.667 s | 300 frames (5.000 s) before | 7149ac96… |
  | 76 | 5308 | 11228 | 187.133 s | 208 frames (3.467 s) after | dfd521bc…; reason `02_4 provisional phrase and that differentiating` |

  No other selected picture falls inside the transition below.

### Measured duration in the decoded master (verify1005 framehash)
- Source: `framehash1005-02_factorials.framehash`, sha a26ca744bf4d41d29c0a7d13cda21141907baf5aa341cc2bc1ddf55ff63ac412. It equals verification1005's framehash_sha256 for 02. 8964 frames; tool `frame-runs02.py`; log `frame-runs02-5100.log`.
- An unbroken run of decoded-picture change covers local 5085–5138: **54 frames = 0.900 s**. That is master 11005–11058, 183.417 s to 184.317 s.
- It has holds on both sides: local 5081–5084 before and 5139 after.
- Frame 5100 is the 16th changed frame of the run, 0.250 s after its first frame. 38 changed frames (0.633 s) follow it.
- Correction to my tool's output: its "full change run around it" summary line in that log is off by one (it says 5084–5138, 55 frames). The run table, 5085–5138 / 54 frames, is correct.
- Outside the run, static stretches show single-frame hash changes in a 12-frame cycle (gaps of 3 and 9 frames), with no matching authored motion. They are probably from encoding, not from scene content; this is unverified.
- A hash change shows only that pixels changed, not how much or how the motion looks.

### Authored duration
- Source: `explainer_corpus/exponential_magic/src/chapters/factorials.rs:595–609`, sha ca48d460151a418ab13ca8b9995ac22f068ea0b975021f81a2ed93e580cdc4b0. `assembly954/stage954.json` (fb47992c) binds it at line 63.
- At sentence index 3 of 02_4 ("But dots are not a proof."), one `play_prepared` call runs four animations together:
  - `fade_out` of LOST_POWER, "A finite polynomial loses its highest power";
  - `fade_in` of NEEDS_PROOF, "The infinite tail needs a proof", at the same position `v(0.0, CAPTION_Y)`;
  - an `indicate` (scale 1.6) on each "+⋯".
- All four are `rt(0.6)`, i.e. **0.6 s = 36 frames** at 60 fps. The two captions sharing one line is therefore authored, not a layout fault.
- The measured run is 18 frames (0.300 s) longer than the authored 36.
  - I found no pace factor.
  - The sentence onset comes from the runtime cue time plus the `report.json` sentence start (`explainer_kit` `sentence_at`). I did not reconstruct it, so I cannot place the authored start inside the run.
  - Lossy-encoder settling after motion is possible but unverified. Settling would need per-frame difference sizes, which means a decode, and that is out of scope.
  - The extra 18 frames remain **unexplained**.

### What I looked at (full-resolution crops of the 1008 bytes; still pictures only)
- Files: `frame11020-01/views01/*-whole1920.png` and `frame11020-01/views02/*-crop-1240-900-2600-1060.png`.
- **4800:**
  - The y row runs to x⁴/4!.
  - The y′ row runs to x³/3! with the gold box.
  - LOST_POWER stands alone on the caption line.
- **5100:**
  - Both rows are extended to x⁶/6! and x⁵/5!.
  - Both "+⋯" are enlarged and yellow (the indicate pulse).
  - LOST_POWER (brighter) and NEEDS_PROOF (dimmer) are drawn on top of each other, centred on the same line. Neither reads cleanly where they cross.
- **5308:**
  - NEEDS_PROOF stands alone.
  - "Does the sum exist?" sits below it.
  - The "+⋯" are back to normal size.
  - The crossfade has settled.

### Previous dispositions (verbatim)
- **Independent, BlackThrush** (`independent-final985-02-review.json`):
  - File sha 1f157a65a500474ac50d0a183d3e33079303d3956abd08f82b32588ec6e48a75; reviewed 2026-11-10T15:04:08Z.
  - Extraction bd98d208…, which is `completed-pictures985-02.json`; movie 605751de….
  - Observations: "Finite-polynomial highest-power loss and the two infinite-series proof obligations are visually separated." and "… selected transitional text crossfades settle cleanly, with no persistent concrete collision."
  - `concrete_defects` [].
  - Limitations include "172 selected frames are not all 8964 frames; no continuous-motion acceptance." and downscaled sheets.
- **Root, BlueSalmon** (`root-final985-02-review.json`):
  - File sha fea538aec50b340d358b1e49176e6b9e017b0c8c37cb6ae9153ad5947d832635; 2026-11-10T15:08:30Z; viewed all 22 sheets.
  - Observation: "Temporary text crossfades settle. Tail bound, derivative shift, uniform convergence and E-summary remain legible with no persistent concrete unintended overlap or clipping."
  - Limitation: "Selected final4K contact sheets displayed at reduced resolution; not exhaustive continuous motion review."
- Neither review names frame 5100 or 11020. Both viewed this exact picture (the same bytes) and treated temporary crossfades that settle as non-defects.

### Disposition
- Frame 11020 catches the authored simultaneous fade-out/fade-in about a quarter-second into its run. It is the same bytes both 985 reviews saw. The decoded picture stops changing at local 5139 (0.65 s after this frame), and the next selected picture shows the settled caption.
- By the criterion both reviews used (temporary, settles, no persistent collision), it is not a new defect and not a 1008 difference.
- I make **no continuous-motion verdict**. Whether a 0.6 s same-line crossfade reads acceptably in motion was never judged by anyone, and still pictures cannot judge it.
- If root or the owner want it changed, it is a source change at `factorials.rs:602–608` (for example, a fade-out before the fade-in), followed by a re-render. Nothing is proposed or staged here.

## 3. Chapter 05 copy correction: root's successor files

- Root wrote `publication976/description-reviewed1009.txt`, sha 41e5a79d8b141717692f9050fa77cf0b35a38a6925b339243184fcb36a12b051.
  - It is **byte-identical** (`cmp`) to my proposal `ch05-label-proposal01/description-timeline982-ch05-proposed01.txt`.
  - Its only change from 982 is line 15: `9:31 Why the whole series converges` → `9:31 Why the stepped approximations reach E`.
- Root wrote `publication976/chapter-times1009.json`, sha 4de735c0a4cf322af36ef07588a3128341d519ac86a1a8805e500795d6378da0.
  - Against my proposal (62e93304…), it differs only in:
    - `utc`;
    - `final_master_verified` false → true;
    - the new `scope` text;
    - two added fields, `master_sha256` fe33b9e48bcb456b3cad41f7651bd8e5f8e3bb1b7c6deba657e6b4d9e5fa2b80 and `verification_sha256` 6c273f16….
  - Times, frames and labels equal the 982 values with the chapter 05 label changed. I checked those 982 times earlier against verify1005's decoded framehash counts. Root reports checking all 15 offsets and counts (164420 frames).
- The originals are unchanged: `description-timeline982.txt` 4098b326…, `chapter-times982.json` 6cd4be35….
- I use the 1009 files as the publication successors. The P2 caption requirement (delivery-caption1004 127f3d1a / 0d2698d0) and P3–P7 are unchanged by this.

## Checks not run
- No listening; no continuous playback or motion review.
- No decode. Framehash runs come from verify1005's existing framehash.
- No pixel-difference magnitudes.
- I have viewed only 10 sheets, all from chapters 01–02, plus the three full pictures above. The 402/402 result is a byte comparison, not my viewing.
