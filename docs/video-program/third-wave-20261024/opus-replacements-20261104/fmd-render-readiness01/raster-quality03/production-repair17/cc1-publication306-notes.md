# CC1 publication draft 306: changes and scope

`publication-draft306/` is a new copy of draft 292 with the review304 corrections applied and the
chapter times bound to the actual timeline269 tables. Draft 292 is untouched: all six of its files,
including `upload-assets298.json`, have the same hashes before and after.

This is publication text only. Nothing was uploaded, synthesized, rendered or launched. No TODO,
Beads or shared files were edited. There is no video, audio, picture, motion or listening approval.
Final-master matching is still pending.

## Files

| File | SHA-256 | Against 292 |
|---|---|---|
| `title.txt` | `48a242e3e4209b446850be61f1868d8f6280881d00d34d2bc387e529c0edc920` | unchanged: "The Hidden Geometry of Complex Functions", no em dash |
| `tags.txt` | `8fab018e2221b8d64646310553b5f58af5496d1df01052b23073c16ee5e497f5` | unchanged |
| `script-reviewed.tsv` | `234f725781bde0bdf35afbc21fd2d6ec0f73f99a95723294259cc1b75a813d3c` | unchanged (equals the 268 workspace `script.tsv`) |
| `description.txt` | `c3443cd5023f0e4bf2be2de2930ed0db0520d2f929ac089a1ac2f56a66fa2afb` | five exact edits (below) |
| `chapter-times.json` | `217abdaae5e945816371ffd6f50d51c91fb519b58f224b6ca92e7399c115e3e9` | rebound to timeline269; two labels |
| `thumbnail.json` | `7d1e911d4fb70e956ae144a8dfc2041ebd08df8fb65bf217f5e87735dfa7ef54` | new: records the unchanged thumbnail208 |
| `SHA256SUMS` | n/a | hashes of the six files above |

## Description: the exact edits

Each edit replaced a string that occurs exactly once. Nothing else changed; `diff` shows only these lines.

1. **Proof scope (304 item 1).** "We build the proof of Cauchy's theorem from shrinking squares and cancelling edges, then use it…" becomes "We prove Cauchy's theorem for squares and triangles, from shrinking squares and cancelling edges, then use it…". This matches cue 08_5, which proves squares and then triangles, and 09_1, which uses those triangles.
2. **12:40 label (304 item 5).** "One derivative is not enough" becomes "Critical points, and a derivative at one point". This matches cues 07_1–07_2 (critical points) and 07_3–07_6 (a derivative at a single point; the holomorphic condition).
3. **14:28 label (304 item 1).** "Contour integrals and a proof of Cauchy's theorem" becomes "Contour integrals and Cauchy's theorem for squares and triangles".
4. **Sources (304 item 4).** Added "NIST DLMF, Calculus of a Complex Variable: https://dlmf.nist.gov/1.9" above §1.10. DLMF §1.9 covers the Cauchy–Riemann equations, Cauchy's theorem, the integral formula, Liouville's theorem and winding numbers.
5. **Credits (304 items 2 and 3).**
   - "Narration: ElevenLabs, Frederick Surrey." becomes "Narration: ElevenLabs Frederick Surrey, a synthetic voice."
   - New line: "Music: procedural score generated in code." Every final render passes score21's stems, which `score21.py` generates. The wording follows the published sister video.

The description is now 2,318 characters (under YouTube's 5,000-character limit), with no em dash and no angle brackets. The one en dash, in "Cauchy–Riemann", is kept. "Animation: FrankenManim." and the sister-video link are unchanged.

## Chapter times: the binding

- **Source job.** csd `csd-review-repair-timeline269`.
  - Receipt `3a21c37e…`: exit 0, finished 2026-11-08T07:10:23Z, inputs unchanged (`2c4e86e5…`).
  - `comparison.json` `d2bf8e9b…`: all 45 tables marked equal after the one narration-path mapping (assembly19 → rebind269).
- **What I checked myself, read-only on csd:**
  - every one of the 45 tables equals timeline21 after that mapping;
  - each hash equals the comparison's `new_sha256`;
  - each frames header gives the expected chapter at 60 fps;
  - every narration line names rebind269;
  - the cue tables hold all 105 cues in exact `script-reviewed.tsv` order.
- **Result.**
  - There are 15 chapters and 133,662 frames, which is 2,227.7 s (37:07.7).
  - Start frames and floor timestamps are identical to 292's.
  - The description's 15 chapter lines equal the file's timestamp and label pairs.
- **What each chapter row now carries.**
  - `timeline_sha256`, `cues_sha256` and `cuelog_sha256` from timeline269.
  - The old timeline21 hash as `timeline21_sha256`.
  - New top-level fields: the timeline269 job, receipt, inputs and comparison hashes, `final_master_matched: false` and `uploaded: false`.

## Kept as is

- **Thumbnail.** `publication19-thumbnail208/thumbnail.png` (`e5dfc2bd04aa3af148ee66ca8107594acf3015a237f5f36a08f1c3c72c4c2eab`) was re-hashed now and equals the reviewed `thumbnail24-reviewed` PNG. Its headline and "Color" spelling are still root's call (304).
- **Script header.** `script-reviewed.tsv` keeps its internal line 1. It must be dropped if this file ever feeds public captions or a transcript (304 item 7).
- **Upload manifest.** No upload manifest was made for 306. `upload-assets298.json` still names draft 292's files.

## Still pending before any upload

- **Final master.** The combined-268 final master and its technical verification, then matching these offsets to that master's actual chapter boundaries.
- **Reviews.** The actual final picture and motion review, continuous listening and fine sync, and the independent final AV review.
- **Captions.** The uploaded SRT binding.
- **Uploading.** A root upload decision, with a manifest naming draft 306.
