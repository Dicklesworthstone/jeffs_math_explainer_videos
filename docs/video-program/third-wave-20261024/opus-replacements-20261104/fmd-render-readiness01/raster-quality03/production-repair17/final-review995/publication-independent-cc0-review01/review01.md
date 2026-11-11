# publication965 against master993: CC0 publication preparation check (review01)

CC0 (LavenderPike), 2026-11-10T23:21Z. These are the files read, all in production-repair17:

| File | sha256 |
|---|---|
| publication965/title.txt | `48a242e3e4209b446850be61f1868d8f6280881d00d34d2bc387e529c0edc920` |
| publication965/description.txt | `c78723f3dffe6476cf68afa56edb1b4d0cae372f44fc7d9596bd607a87f2fd0b` |
| publication965/tags.txt | `40e49939e363bb750de3884b2c8b0ed3664303887829417622e5ac79d52594a8` |
| publication965/review965.json | `5469adeb78c675a5ca7864d162c20226c253ab7abefb5c0598b2b5d85d52129f` |
| completed-master993-summary.json | `38b52105520640e495ada17aa00da4ddbc16316a5a77c4568e102e1ca3adf185` |
| publication-chapters-checked993.json (root, 17:47Z) | `6ef775a5255d9ad96e022e672b80417f37762efb9b5cb3d8fe3da41a214a7ad9` |
| local-master993-transfer-receipt.json | — |
| complex-analysis-master993.en.srt and .vtt | — |

**Scope of this check:**
- These were reads and one local check script only.
- There was no upload, build, render or Bead edit, and no publication file was changed.
- No audio heard or checked. This is not AV, picture or publication acceptance.
- master993's own summary records `full_decode`, `final_picture_review`, `motion_review` and `audio_heard` as false.

**Existing newer copy.** publication965 is the newest publication text; there is no later publication folder. Root's publication-chapters-checked993.json already binds that description to master993 (summary 38b52105, description c78723f3). I did not duplicate it. I re-derived the chapter starts independently from the summary's own rows and compared them with both.

## Result: no correction needed for master993

**Length and format.**
- master993 has 15 rows, which are contiguous and sum to 133662 frames, or 2227.700 s at 60 fps. That is **37:07.700**.
- The local master is 3840×2160 at 60 fps, with 48 kHz stereo AAC. ffprobe gives a duration of 2227.700000.
- Its sha256 is `80244e88…`, which equals the summary's master and was checked on transfer.

**Chapter marks.** All 15 description marks equal floor(start_frame / 60) and root's checked993 rows. The chapter names, order and frames agree too.

| Chapter | Start | Length |
|---|---|---|
| 01_the_bump | 0:00 | 105.2 s |
| 02_turning_numbers | 1:45 | 89.5 s |
| 03_two_planes | 3:14 | 120.8 s |
| 04_reading_color | 5:15 | 112.9 s |
| 05_gallery | 7:08 | 164.7 s |
| 06_one_derivative | 9:53 | 167.2 s |
| 07_one_point | 12:40 | 108.2 s |
| 08_loop_sums | 14:28 | 249.3 s |
| 09_boundary | 18:37 | 275.1 s |
| 10_rigidity | 23:12 | 124.0 s |
| 11_counting | 25:16 | 198.9 s |
| 12_residues | 28:35 | 65.7 s |
| 13_real_integral | 29:41 | 177.4 s |
| 14_continuation | 32:38 | 195.9 s |
| 15_closing | 35:54 | 73.0 s |

The YouTube chapter rules hold:
- the first mark is 0:00;
- there are at least 3 marks;
- they ascend;
- every chapter is at least 10 s (the shortest is 12_residues at 65.7 s).

**Captions.** The local srt and vtt equal the summary's hashes (`5d064232…` and `26e0d7e0…`). Each has 571 cues, and the last ends at 2220.147 s, inside 2227.700.

**Title and tags.**
- The title is "The Hidden Geometry of Complex Functions", 40 characters.
- The catalog's working title is "The Beautiful Geometry of Complex Functions". The published title was settled in the earlier publication reviews, so no change is proposed.
- The 18 tags count 385 characters with quotes, against a limit of 500.
- There are no angle brackets in the title, description or tags.

**Credits.**
- "Narration: ElevenLabs Frederick Surrey, a synthetic voice." is present, and no other voice is named (no Oliver, Michael or FrankenTTS).
- Records tie master993's narration (the rebind269 delivery) to Frederick: the production19 narration-direction map is the Frederick map, and the Bead source records Frederick cue pairs.
- The voice was not re-verified from per-cue ElevenLabs receipts.
- "Music: procedural score generated in code." has something behind it: master993 mixes the score21 stems.
- "Animation: FrankenManim." is unchanged.
- The sister link `youtu.be/lgpTDROHgQY` is the public logarithm cut. Its receipt title is exactly "Logarithms Made Visual: From Slide Rules to Matrices".

## Conditions before upload (not corrections)

1. **This binds master993 only.** If the uploaded master is a successor built from the combined01 or counting02 sources, re-derive the marks and the caption binding from that master's summary.
   - Run `check01.py` with the successor's summary in place of master993's.
   - Chapters 01, 08, 09 and 11 change source. Any frame change there shifts every later timestamp.
2. **The repairs need no description change.** Neither the 11_3 walker nor the 11_6 note, the 11_2 card wording, or the bundle02 picture repairs alter anything the description says.

Files in this folder:
- check01.py / check01.out (the independent check);
- this report.
