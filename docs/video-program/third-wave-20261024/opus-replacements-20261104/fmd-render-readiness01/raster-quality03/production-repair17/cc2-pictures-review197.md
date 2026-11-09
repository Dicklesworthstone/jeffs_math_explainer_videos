# Final-picture plan review 197

RubyRaven, for root, under `mx-video-program-program-87do.39`. Independent of the plan's author.

This is source and technical evidence only. I read the plan, the helper, its waiter, the selected master plan, the 08 and 09 permits and their evidence. I ran read-only Python checks on local copies.

I viewed no picture, ran no extraction, master, build or job, and changed no file except this one. Nothing here accepts any picture, listening, fine sync or final audiovisual result.

## Files reviewed (sha256)

| File | sha256 |
|---|---|
| `final-picture-plan182.json` | `20a36f91989e0f1e0ca5149a74483ec22fe3728be44b872fc31a0859747b962a` |
| `trj-final-pictures182.py` | `0f9b2de7db9a48c8bc0cc1f5fef4762fe2cd13348e8e9cfdbea28658d8efd072` |
| `trj-pictures-after-verify182.py` | `86e8224bf738fb913ecb8e06e2c7dbb9f7daa75e88efb9c66d04bf3b9c043334` |
| `master-plan183/chapter-plan183.json` | `4b12d7698c798df80bc02f338b14bf285fc00b24c13cd6af7c93dc3877f479ba` |
| `repaired08-review181/final-render-permit181.json` | `dae3e84f2150aac7d1b76c69f13cd7369424ff82adec9919fa3327e7015184b8` |
| `repaired09-review158/final-render-permit158.json` | `5e2c0ab2a5665999a1f8a79ceb8f4b5a18f0fcea699926b2c7aecb86ff28f8b2` |
| `final-chapter-pictures142.py` (the per-chapter standard, for comparison) | `a0430c030b48e9defdbaa50b22494cb7f96a763ce9ec1419233dff752edd8dc5` |

## Findings

**F1. Blocks publication, not the master. Nothing in the 182/183/184 chain requires a native-4K final picture review of 08, 09, 13 or 14.**
- **What the chain checks.** `trj-final-master184.py` and `trj-master-after-chapters184.py` check each chapter's render permit. For these four chapters that is permit181, permit158, permit96 and permit111. All four come from preview reviews and authorize rendering only. The master's summary already writes `final_picture_review=False`.
- **What plan182 gives these chapters.**
  - The overview is scaled to 960×540 during extraction, and no native copy is kept.
  - Only 18 native 4K PNGs cover these chapters: 08 at 120, 124.65, 153, 160 and 163.5 s; 09 at 58, 64, 74, 101, 137.5, 157, 243 and 261 s; 13 at 88.7 and 90 s; 14 at 90, 98 and 150 s.
  - In time, 13 and 14 average one overview frame per 1.25 and 1.57 s, with gaps of up to 4.8 and 5.0 s.
- **What the other eleven chapters got.** `final-chapter-pictures142.py` extracts a frame every second at 3840×2160, plus four frames per cue or mark and persistence frames for each layout anchor. The 04 review198 used it on the exact selected chapter (146 pictures, movie `6ba1ebd0…`).
- **Risk.** If plan182's sheets are taken as the final-4K review of 08, 09, 13 or 14, anything visible only at 4K could pass. That covers thin strokes, raster seams and small 18–20 pt captions such as 14_7's dense-roots line. The 09 permit itself says "final4K pictures remain mandatory".
- **Fix (root's call).** Once each final finishes, run the existing `final-chapter-pictures142.py` on 08, 09, 13 and 14, as for 04. Then have the publication gate require those four review receipts, each bound to the chapter hash the master joined. Root's TODO already plans these reviews, but nothing enforces them.

**F2. Don't reuse the old 08 review.** The only per-chapter final review of 08 on disk is `final08-terminal64/pictures-review71.json`. Its inputs are the held 08: movie `ea2673439b1c663e…` and timeline `78700af1fb68ced6…`. That is not the selected `csd-repaired-final-08-loop-sums170`, whose permit timeline is `338f16a8…`. Plan183 rightly leaves the held 08 out, and review71 must not be counted for 08.

**F3. Optional unless F1 is skipped. Plan182's 09 additions come from a different preview than the one the 09 permit reviewed.**
- Its 320 "Actual preview21/09 review sample51" frames come from `complex-preview09-pictures51/sampling.json`. That is movie `b417a99a…`, a mid-job export of an earlier 09.
- Permit158 cites `pictures-reviewed146` instead: movie `a277716f…`, 383 samples, plus `playback147`. None of permit158's evidence is among the plan's 68 sources. Permit181's three files are there.
- The times still hold, because root's `timeline69-comparison.json` records 09's tables as equal. The plan also carries 92 prior rho/p collision samples, and each of permit158's eight motion frames has a sample within 3 frames.
- Not covered natively: review146's native detail checks at 50 s (magnification leaders) and 145 s (Cauchy equality). They are not among plan182's 09 native details. A 1-second native review under F1 covers both. Without F1, add them.

**F4. Optional. The helper checks each final timeline only by its `frames` row.** The anchors come from the plan's timeline21 tables.
- For 08, I compared the plan's table (`fed5faea…`) with the permit's table (`338f16a8…`, local copy in `preview166-review180`): 10 of 10 rows are equal once narration paths are reduced to file names. For 09, root's comparison records `ae7a13f4…` → `4cbb007a…` as equal; I did not redo it.
- For every chapter, cue-time equality is enforced upstream by master184's `verify_tables`, not by this helper. A cue shift with an unchanged frame count would move the anchors silently if that upstream check ever changed.

**F5. Optional. Not all samples are coverage.** 157 of 08's 1,499 samples and 423 of 09's 1,867 are "production16 selected timestamp; historical" frames. Their times come from the chapters before expansion: `continuous-timeline16` has 08 at 5,911 frames and 09 at 9,587. They fall on unrelated moments in the current 14,959- and 16,504-frame chapters. They do no harm, but they shouldn't be counted.

**F6. Hygiene.** The plan contains 1,501 absolute home-path strings, in every `local_snapshot_path` and in 473+ of the 09 sample reasons. If this folder is committed, that breaks the repository's public-path rule.

## What holds (source and technical only)
- **Chapters:** 15, contiguous, 133,662 frames in total, in the same order as plan183. 08 has 14,959 frames and 09 has 16,504, matching permits 181 and 158.
- **Sources:** all 68 local snapshots match their bound SHA-256. 22 of the bound `layout.tsv` rows are the empty file (`e3b0c442…`), meaning a clean lint at that time.
- **The helper fails closed on:** the admitted guard receipt, the bound inputs (re-hashed at the start and the end), the master and verify receipts, the plan frames, chapter order against the master summary, per-chapter frames against each final timeline, and the master hash. Its layout regex accepts every row of the four non-empty bound layout files I checked.
- **Overview density:** no gap longer than 5.0 s in any chapter. 08_5 gets about 80 frames per 10 s, 09_1 about 95, and the rest of 09 28–49. Every motion frame in permit181 has a sample within 5 frames, and every one in permit158 within 3. Permit181's detail seconds 120, 153 and 160 are all native PNGs.

## Not done
No picture was viewed, extracted or judged. As far as these files show, the master and the 08, 09, 13 and 14 finals have not finished. Continuous listening, fine sync, captions and independent final audiovisual acceptance remain open.
