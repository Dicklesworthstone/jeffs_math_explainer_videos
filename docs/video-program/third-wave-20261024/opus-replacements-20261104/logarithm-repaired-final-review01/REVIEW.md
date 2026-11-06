# Repaired logarithm delivery: final-artifact review

BronzeJaguar, 2026-11-06 UTC, for BlueSalmon (Mail 5675).

Subject: master `8534ec21…` from `logarithm-remote-render-work01`, verified by `logarithm-remote-verify01`, not the old stage09 master.

## Verdict

I found no release-blocking defect in what I examined. There is one Low caption finding, L1.

This is not acceptance. I read 48 of the 367 sheets as stills and did not listen or watch motion, so full independent AV review stays open.

## Finding

**L1 (Low): some caption blocks appear after their words are spoken.**

- **Evidence:** I measured 96 blocks that start mid-sentence, where the recognized word runs straight on from the word before it. That filter removes the known early-onset bias in pauses.

| Lag behind the recognized word | Blocks |
|---|---|
| Median | 0.35 s |
| Over 0.5 s | 28 |
| Over 0.75 s | 11 |
| Over 1 s | 2 |

- **Worst two:**
  - Block 409 at 27:10, "both routes: first A and then X…", is 1.27 s late.
  - Block 145 at 9:15, "a times b is the logarithm of a plus…", is 1.16 s late.
  - The next nine are in `checks/outputs/run_checks.txt`.
- **Cause:** `crates/explainer_kit/src/captions.rs:366-368` shares a sentence's time among its blocks by character count. That share doesn't follow the actual pace or pauses inside a sentence. None of the six repaired sources touches this code, so the repair did not introduce it.
- **Effect:** the words are right and only the timing is off. These are recognizer estimates, so I can't say how noticeable this is. That needs the fine-sync listener.
- **Fix, if you want one:** the captions are sidecars, so the master would not change. `tools/retime_captions_from_phrases.py` moves only boundaries inside a sentence, which fits this case. It needs its own bound timing inputs, and I did not check whether they exist for this delivery. I did not run it.

## Bindings

`checks/run_checks.py` recomputes these from the delivered files: 58 checks passed, 0 failed. Transcript: `checks/outputs/run_checks.txt`.

- **Assets:** master, SRT, VTT, thumbnail and draft06 all equal `delivery-package01.json`.
- **Terminal state:**
  - The master and verify receipts both show exit 0, and their inputs did not change during the run.
  - The strict log says VERIFY PASS, with an A/V delta of 0.001 s and 0 decode errors on both decoders.
- **Sources:** all six repaired files on disk equal both `verification.json` and `integration.json`.
- **Sheets:** all 367 equal their recorded hashes.
- **Container:** `moov` sits before `mdat`, and the tags hold no paths (checked earlier this session, not in `run_checks.py`).
- **Captions:**
  - There are 475 blocks, and the SRT equals the VTT.
  - Blocks don't overlap, and none is over 2 lines of 42 characters.
  - The fastest block is 16.9 characters per second.
  - All 80 cues equal the script text.
  - Neither reading hold (10_4, 11_20) has a caption over it.
- **Chapters:** all 13 timestamps match the master's frames, and the frames sum to 116,665. The 29:33 recap label matches the caption "Look back along the way." at that time.

I also read all 475 captions and worked the numbers and claims by hand, including 440√2 ≈ 622, 3^12 against 2^19, ln 2/.05, ln 1000/.05, the 2×2 Culver statement, the winding and product-rule arguments, and the cube. I found no mathematical defect. That is my judgment, not an automated result.

## Repairs and closing, viewed as stills

| Repair | Author | Sheets | Result |
|---|---|---|---|
| 04_7 reveal | LavenderPike | 116–119 | Fraction only through 618.70 s, blank at 618.90 s, answer at 619.10 s. Nothing lingers. |
| Pause line, 13 sites | BronzeJaguar | see `viewed-sheets.tsv` | The line shows at every site, with no answer visible and no collision. |
| `ln 2 ≈ 0.693` | BronzeJaguar | 152 | Reads as approximate. |
| Bar moves, 05_5 and 06_4 | BronzeJaguar | 145, 146, 171 | Bars clear the axes. |
| 11_1 and 11_2 gloss and footer | RubyRaven | 282, 284, 289 | Inside the frame. The gloss clears at 11_2. |
| 11_5 sample card | RubyRaven | 304, 305, 309 | My hand-computed routes, (−1.1, 1.15) and (−2, 0.4), agree with sheet pixels (−1.09, 1.12) and (−1.96, 0.40). |
| 11_8 formula | RubyRaven | 331, 333, 334 | Clear of the frame edges and the footer. |
| Closing hold and fade | RubyRaven | 363, 365–367 | Last word 1936.82 s, hold to 1942.90 s, fade to 1944.40 s. |

For the closing I also pulled native frames from the actual master. The last 1.2 s fades cleanly and the last frame is flat, with a maximum deviation of 0.

The thumbnail is readable with no clipped or overlapping text. Chapter labels in draft06 fit the on-screen chapter titles.

## Citations

I read Gallier (arXiv 0805.0245) and Higham's Handbook chapter (`eprints.maths.manchester.ac.uk/310/`) as extracted text. I did not read Culver's paper.

| Where | Citation | Checked against | Result |
|---|---|---|---|
| 10_4 footer | Higham; Gallier Thm 3.11 | Higham §4 Fact 1; Gallier Thm 3.11(a) | Matches |
| 11_20 card | Culver as stated by Higham and Gallier Thm 3.4 | Higham §4 Fact 3; Gallier Thm 3.4 | Matches |
| 11_8 footer | Gallier Thm 3.3, via the Jordan form | Gallier Thm 3.3 | Matches |
| 12_21 footer | Higham §4 Fact 1 | Higham §4 Fact 1 | Matches |
| Culver details in the description | Proc. AMS 17, 1146–1151, 1966 | Crossref record for the DOI | Matches |

Description links:

- `/310/` is the Handbook chapter.
- `/1067/` is the 2008 book.
- arXiv and DLMF 4.2 retrieved and match.
- `doi.org` itself was not fetched.

The book's "sections 1.5–1.7 and chapter 11" is unchecked, because the table of contents refused the connection.

## Checked and dropped

- **Chapter joins:** all 12 cut from a full picture to an empty frame, so this is the film's consistent style. Whether it feels right is a motion judgment.
- **Words up to 8.64 s outside their caption:** these are recognizer artifacts, with one word stretched across a hold (for example "Use", 178.98–183.70 s), plus one false match in my alignment.
- **−20.1 LUFS:** the same as the prior master, because the −1.5 dBTP ceiling limits it.
- **`publication-master-times01.json` pins draft05's chapter file:** draft06 changes only the last title and status fields, so its frames are unchanged. The delivery package pins draft06.
- **`docs/sources.md`:** its Handbook record link points at the book record, and it says the DOI is unresolved. This is a crate note, not a release asset, and the description's links are correct.
- **"The companion video builds it from scratch" (7:29):** the description says that film is in production.
- **"Closing 1.00 looks bright":** native frames show it fading in step with the title.

## Not done

- No listening, motion review or fine-sync review.
- 319 sheets not read in this review. Root viewed all 367.
- Native 4K frames for the last 1.2 s only, so compression at 570 kbps video was not inspected elsewhere.
- Caption overlay on low footers, which Lavender is sampling separately.
- Player seeks, and any post-upload check.
- 02_3 "turned" against the recognizer's "turn": it stays with a listener.
- `remote-binding01.json` isn't local, so the binding hash is recorded but not re-derived.

## Independence

This review is independent of RubyRaven's chapter 11 and 12 repair and LavenderPike's 04_7 repair.

It is not independent of my own work:

- the middle repair (pause line, ≈, two bar moves)
- the 05–07 repair
- the pane 168 review of the prior master
- the verifier that produced these sheets
- the ASR02 bundle whose word times L1 uses

Someone else should judge those parts in the full AV review.

## Reproduce

```bash
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-repaired-final-review01/checks/run_checks.py
```

It is read-only and needs numpy and PIL. The citation checks were by hand and are not in the script.

`checks/__pycache__/run_checks.cpython-315.pyc` appeared even though every run used `-B`, and I didn't find what wrote it. It holds a local path, so it is outside the seal and listed in `ANCILLARY.json`. Git ignores it and nothing reads it. I left it in place, since deletion isn't authorized.
