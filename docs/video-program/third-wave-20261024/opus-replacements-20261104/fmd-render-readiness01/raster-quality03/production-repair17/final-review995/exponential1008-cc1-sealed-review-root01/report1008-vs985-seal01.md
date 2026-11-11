# Exponential pictures1008 vs 985: full byte comparison, review provenance, ch02 crossfade, publication 1010 check (seal01)

BronzeJaguar (CC1), session b0a72b9b, sealed 2026-11-11 about 00:45 UTC. Scratch only.

**What I did not do:**
- No job, render, decode, upload or listening.
- No continuous-motion or audiovisual acceptance is claimed.
- No edits to production, active files, Beads or TODO.

**Authorship:**
- I wrote `pictures1008.py` (70dfce04; root pinned it as 0d9d48e1) and every comparison or check script here.
- The picture judgements rest on root's and BlackThrush's 985 reviews, which looked at these exact bytes.

`P808` = `narration_lab/third-wave-20261024/root/elevenlabs-exponential-20261101/expanded-production808`. Every artifact is copied unchanged into `artifacts/`, and `SHA256SUMS.txt` covers them.

## 1. Result: actual 1008 pictures are the 985 bytes

The final dump is `artifacts/records1008-06.json` (00:25:21 UTC, after the recheck finished). Compared with `compare1008-985.py` (9a7838cb, self-test PASS) against verification 6c273f16974dd3ac0f0363722be9ef308ed4f58a3d5809837acb3aaaf47f7909, it gives `compare1008-985-run06.log`: **exit 0, hard problems 0, pictures differing 0**. Root reran the same comparison on dump06 and got exit 0.

| Chapter | Pictures (1008 bytes = 985 bytes) | Sheets (same local frames) |
|---|---|---|
| 01 | 64/64 | 8/8 |
| 02 | 172/172 | 22/22 |
| 03 | 150/150 | 19/19 |
| 04 | 68/68 | 9/9 |
| 05 | 246/246 | 31/31 |
| 06 | 236/236 | 30/30 |
| 07 | 130/130 | 17/17 |
| 08 | 114/114 | 15/15 |
| 09 | 473/473 | 60/60 |
| 10 | 154/154 | 20/20 |
| 11 | 120/120 | 15/15 |
| 12 | 94/94 | 12/12 |
| 13 | 546/546 | 69/69 |
| 14 | 559/559 | 70/70 |
| 15 | 37/37 | 5/5 |
| **Total** | **3163/3163** | **402/402** |

- All 15 chapter receipts are stable successes. In every chapter these match: stream, extract source (chapter sha = 985 movie sha), offsets, frame counts, selection reasons, canonical paths and sheet frames.
- **Recheck:**
  - Receipt: admitted; finished 2026-11-11T00:24:29.184077Z; exit 0; no stop or error; inputs = final inputs = 4fa1894bd6cf1d28487a5bbac9cd194e89933e414c2f773cca4caa5a34dabcc2.
  - Record sha cc00e6005f278ac1187f76b00bc4d4fc238446cbead49c82e4281cc261625034: samples 3163, records_exact true, upstream_rows 151719 unchanged.
  - Both equal root's preserved `final-review995/exponential1008-recheck-terminal-root01/` (`recheck1008.json` cc00e600…, `resource-receipt.json` c2f3c749…).
- **Sheets:** 0/402 sheet files are byte-equal to 985's. That is by design: the 1008 labels carry master frame numbers (`pictures1008.py` line 28). Each sheet's eight tiles are byte-identical pictures that both reviewers already viewed. **No further identical-pixel review is needed.**
- Intermediate dumps and logs are kept: `records1008-01..05.json` and `compare1008-985-run01..05.log`.

## 2. Exact transfer of prior 985 image-review provenance (`artifacts/review985-provenance01.md`)

- The table lists, for each chapter:
  - the seed `P808/completed-pictures985-NN.json` and its sha;
  - `P808/root-final985-NN-review.json` (BlueSalmon / root);
  - `P808/independent-final985-NN-review.json` (Independent Codex / BlackThrush).
- For each file it gives the path, sha256, reviewer, UTC, whether it binds the seed sha, safe_to_master, defect count, and frames and sheets viewed.
- What the table shows:
  - All 30 review files bind their seed's sha.
  - Each viewed all of its chapter's frames (3163 in total) and all sheets (402).
  - Each records 0 concrete defects.
  - BlackThrush's 15 files have `safe_to_master` true; root's 15 are false only because of their chapter-only scope.
- The seeds are the ones verify1005's inputs bind (`review985-coverage01.log`, `vm-chapter-rows01.txt`).
- Together with section 1 (3163/3163 identical bytes, records_exact recheck), this is the explicit transfer that `upload-preparation1010.json` open gate 2 allows, as an alternative to reviewing the new sheet labels.
- The transfer carries over the reviews' own limits:
  - downscaled sheets;
  - no continuous-motion acceptance;
  - no listening or fine-sync.

## 3. Chapter 02 frame 11020 (183.667 s): disposition

Details are in `pictures1008-crossfade-addendum01.md` §2.

**The picture.**
- Local frame 5100, picture 75/172, sheet 010 slot 3.
- It comes from the 5 s overview grid, not from a choice to show the transition.
- sha 123fd656…971b, the same in 985 and 1008.
- The 985 sheet 010 (288ec274…) is in BlackThrush's `viewed_sheet_hashes`.

**What happens on screen.**
- At 02_4 sentence 3, `factorials.rs:596–608` (ca48d460, bound by stage954) fades out "A finite polynomial loses its highest power" while fading in "The infinite tail needs a proof" at the same caption position.
- It also pulses both "+⋯".
- All of this is authored to last 0.6 s (36 frames).

**Measured duration.**
- verify1005's framehash (a26ca744) shows decoded change over local 5085–5138: 54 frames, 0.900 s, master 183.417–184.317 s.
- The picture is 0.250 s into that run, and the picture settles 0.65 s after it.
- The extra 18 frames are unexplained.

**Neighbouring selected pictures.**
- 4800, 5.0 s earlier: the old caption alone.
- 5308, 3.47 s later: the new caption, settled.

**Prior dispositions.**
- BlackThrush: "Finite-polynomial highest-power loss and the two infinite-series proof obligations are visually separated." and "… selected transitional text crossfades settle cleanly, with no persistent concrete collision."
- Root: "Temporary text crossfades settle … no persistent concrete unintended overlap or clipping."
- Both recorded 0 defects.

**Disposition.**
- This is an authored temporary crossfade, caught mid-transition.
- It is not a defect under the criterion both reviews applied, and it is not a 1008 difference.
- **No continuous-motion verdict:** nobody has judged whether it reads well in motion, and stills cannot. Changing it would be a source edit plus re-render, and nothing is proposed.

## 4. Narrow check of root's five publication976 *1010 files

| File | sha256 |
|---|---|
| `title-reviewed1010.txt` | a9efabc71427359f664653b8da86649afa05ba70f635eef117f8ec4ea5395225 |
| `description-reviewed1010.txt` | e280d5588ea779144d49318bd3cab4e77184381c305017cb301d490e7609313c |
| `tags-reviewed1010.txt` | 9a00b2749a875c7a7563617bde26c999a477a45870b15d17e4dfbde25522a794 |
| `chapter-times1010.json` | a1854bfe258a97bbff93c8c5a73a3bd38e4a2f4943f9d5c1e4e8610373fdeba4 |
| `upload-preparation1010.json` | bbd279db07a961d2e97af0adbf313e3dfe123a609fbaaebb7a61ea127072520c |

**Bindings: no mismatch.** `check-publication1010-01.py` gives `check-publication1010-01.log`: 36 PASS, 0 FAIL.
- Every asset sha in upload-preparation equals its file: title, description, tags, SRT 127f3d1a, VTT 0d2698d0, thumbnail 0d54f69e.
- chapter-times1010 and upload-preparation bind the same description, e280d558.
- Master fe33b9e4 is the same in upload-preparation, chapter-times1010 and verification1005's movie. The verification path hashes to 6c273f16.
- The captions are verify1005's delivery pins. caption_count 751 equals verification. Frames are 164420 everywhere, and duration 2740.333 s = frames/60.
- Each of the 15 chapters' start frame and frame count equals the actual pictures1008 record's verify1005-bound offset and frames. Display = floor(start/60) as m:ss, and the frames sum to 164420.
- The description's chapter block equals chapter-times1010's display plus label, line for line.
- **Lengths:**
  - title 28 characters;
  - description 2024 characters;
  - 16 tags, about 255 counted characters (limit 500), no duplicates;
  - no angle brackets anywhere.
- **Against the earlier versions:**
  - The description differs from 1009 only in lines 3 and 7.
  - chapter-times1010 differs from 1009 only in `utc`, `description_sha256` and `scope`. It has no trailing newline, which is harmless.

**Copy facts against `script.tsv` (d5a945b2): no factual error found.**
- Line 3 (P3) now splits the two laws correctly:
  - the addition law explains the slide rule (07);
  - the rate law explains the models (06_8, 08_3, 08_7).
- Line 5 uses the film's three pictures from 14_4, "The walk, the polygons and the spiral":
  - the walk is 13_4's "walk around the unit circle";
  - the polygons are 14_1–14_2;
  - the series spiral is 14_3.
- Line 7 names the companion correctly. The title "Logarithms Made Visual: From Slide Rules to Matrices" (lgpTDROHgQY) was confirmed by oEmbed on 2026-11-10. "From real numbers through the complex plane to matrices" matches the order in `logarithm_mysteries/script.tsv`: complex lines first appear at 00_6, 03_4 and 05_1, and matrices from 09_1.
- Tags:
  - "Euler's formula" is named in 13_4 (script line 84: "It is called Euler's formula").
  - "RC circuit" fits 09_1, a capacitor charged through a resistor.
  - half-life, decay, compound interest, slide rule, logarithms and complex numbers all appear in the film.
- The title "The Magic of the Exponential" now shares the thumbnail's "THE MAGIC OF e^x" and drops "Everywhere" (P4).

**One optional nit (Low):**
- Line 5's "shrinking multiplication steps" stands for the film's polygons.
- Their step size π/n does shrink as n grows. But 14_2 says each step multiplies the arrow's length by "a number just above one", so a reader could take "shrinking" to mean the arrows shrink.
- "ever-finer multiplication steps" or "polygons of tiny multiplication steps" avoids that reading. It is not a factual error.

**One note (not a mismatch):**
- upload-preparation's master entry names only the CSS path. The byte-identical local copy is `P808/exponential-master987.mp4`: I re-hashed it at about 00:30 UTC on 2026-11-11 to fe33b9e4, and it has 216578415 bytes and transfer receipt af97018066ba7b3e807b17fe630f437c5ed6249ee7bb8ef682548c224bd6273f.
- If the upload runs from this Mac, that local path is the file to bind.

## 5. Remaining exponential publication blockers, from existing receipts only

**Closed by receipts:**
- **Master technical verify.**
  - verify1005 PASS (6c273f16).
  - fe33b9e4: 164420 frames, full A/V decode, all chapter frame identities.
  - bt709, −18.8 LUFS with `nominal_loudness_exception` true at −1.5 dBTP.
- **Local master copy.** See the note above.
- **Captions.**
  - delivery-caption1004 SRT 127f3d1a and VTT 0d2698d0.
  - 751 captions, caption_words_exact true, srt_vtt_equal true, fast_captions 0.
- **Chapter times and labels.** chapter-times1010.
- **Upload package and settings.** upload-preparation1010 binds:
  - title, description, tags, captions, thumbnail and master;
  - English, Education, not for kids, no paid promotion, synthetic disclosure on, automatic chapters off, Private first.

  The old `draft-status.json` (c868069f, 12:36Z) is superseded and preserved.
- **Pictures.** Covered by section 1 plus the provenance transfer in section 2. This satisfies open gate 2's "explicitly transfer exact prior image-review provenance".
- **Copy.** P1, P3, P4, P5 and P6 are dispositioned in 1010, and I found no factual error (section 4).

**Still open (`upload-preparation1010.json` `open_gates`; I add none):**
1. **Continuous audiovisual review, including listening and fine sync.**
   - verify1005's scope excludes it, no receipt records it, and I do not claim it.
   - It is the one remaining human acceptance before publication. The chapter 02 crossfade's look in motion falls under it.
2. **Live check of the Smithsonian link.**
   - My direct fetch got HTTP 403 at 00:32:26 UTC. Root got 500 direct and 404 on an alternate.
   - Either a person opens it in a browser, or lines 30–31 come out (DLMF 4.2 is confirmed). Publishing a link no one has opened is the risk.
3. **After upload:** processing and checks completing, 2160p60 availability, and an English watch check (root's existing gate 4).

`publication_accepted` stays false, and the thumbnail reuse receipt (`thumbnail-reuse986.json` 524c1418) still says `publication_acceptance` false. Accepting both is the owner's or root's decision, not something a receipt shows.
