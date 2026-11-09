# CC1 publication review 304: complex analysis draft 292

This is an independent editorial check of the publication text against the script and the actual
15-chapter schedule. It approves nothing. The final master, final pictures and motion, listening,
fine sync, the independent final AV review and the public checks all remain pending. No video or
audio claim is made here.

## Inputs read in full

| File | SHA-256 |
|---|---|
| `publication-draft292/title.txt` | `48a242e3e4209b446850be61f1868d8f6280881d00d34d2bc387e529c0edc920` |
| `publication-draft292/description.txt` | `2bccf7a19c042987dc62a1d33037f68878ed5234789f48c786dbea0e6092dee1` |
| `publication-draft292/tags.txt` | `8fab018e2221b8d64646310553b5f58af5496d1df01052b23073c16ee5e497f5` |
| `publication-draft292/script-reviewed.tsv` | `234f725781bde0bdf35afbc21fd2d6ec0f73f99a95723294259cc1b75a813d3c` |
| `publication-draft292/chapter-times.json` | `10ee70317c58255e9ebb52c6801cc9e4315743eab0c1ab844ebe95fb85734ff5` |
| `upload-assets298.json` | `51591d59475595d42c205b6bd37669a450e8b28a24bec1a9a5753d8da0594f74` |

**What I compared them against:**
- **On csd, read-only:**
  - build268's chapter order;
  - all 15 timeline21 tables, their frames and cue tables;
  - the timeline269 tables written so far (01–08);
  - the three `script.tsv` copies;
  - score21's command.
- **Source pages:** DLMF §1.9, §1.10, §5.2 and §25.2.
- **Local records:**
  - the thumbnail records;
  - the logarithm sister's catalog entry and its final description draft.

## Corrections

1. **Medium: Cauchy's theorem is overclaimed.**
   - **The claim.** The description says "We build the proof of Cauchy's theorem from shrinking squares…" and the 14:28 label says "Contour integrals and a proof of Cauchy's theorem".
   - **What the film does.** Cue 08_5 states the general theorem and then says "We will prove the pieces we need, starting with a square". It proves only squares and triangles ("So around every triangle, the integral is zero too"), and 09_1 uses those triangles.
   - **Minimal fix.**
     - Description: "We prove Cauchy's theorem for squares and triangles, from shrinking squares and cancelling edges, then use it to…".
     - Label: "Contour integrals and Cauchy's theorem for triangles", or similar.
2. **Medium: the narration credit can read as a human narrator.**
   - **Current line:** "Narration: ElevenLabs, Frederick Surrey."
   - **Precedent.** The published sister video's final draft (`logarithm-publication-draft06`) says "ElevenLabs Oliver, a synthetic voice", and `complex-publication-draft02` said "ElevenLabs Frederick Surrey, a synthetic voice".
   - **Fix:** "Narration: ElevenLabs Frederick Surrey, a synthetic voice." The voice name is correct.
3. **Low: the music credit is missing.**
   - Every final render passes `--score` with the score21 stems. Score21 is `score21.py`, a generated score whose register follows the narration's measured pitch.
   - The sister video credits "Music: procedural score generated in code."
   - **Fix:** add that line.
4. **Low: the sources miss the section that covers half the film.**
   - DLMF §1.10 covers Taylor series, continuation, residues and the argument principle.
   - The Cauchy–Riemann equations, Cauchy's theorem, Cauchy's integral formula, Liouville's theorem and winding numbers are in §1.9, which I checked.
   - **Fix:** add "NIST DLMF, Calculus of a Complex Variable: https://dlmf.nist.gov/1.9".
   - **Left as is.** The §5.2 ("Definitions", Gamma Function) and §25.2 ("Definition and Expansions", Zeta) labels are fair.
5. **Low: the 12:40 label "One derivative is not enough" is ambiguous.** It can read as "a first derivative is not enough". The chapter covers critical points (07_1–07_2), then a derivative at a single point and the holomorphic condition (07_3–07_6). Suggested label: "Critical points, and a derivative at one point".
6. **Low, binding: `chapter-times.json` still binds timeline21 hashes.**
   - For example, 06 is `dbc82d84…` and 08 is `fed5faea…`.
   - The combined final is scheduled by timeline269. Its tables differ in bytes because of the narration path, but for 01–08 their frame counts equal timeline21's.
   - Before upload, rebind the hashes to timeline269 or the final master and re-derive the offsets from the master, as the file's own scope says.
7. **Low, conditional: the script file starts with an internal note.** Line 1 of `script-reviewed.tsv` is a production comment ("stage06 root candidate; … five unrecorded cue corrections…"). If this file feeds a public transcript or captions, drop that line.

## Checked with no change needed

- **Chapters.**
  - All 15 rows match the actual chapter order, frames at 60 fps, cumulative start frames, floor timestamps, timeline21 hashes and the description lines.
  - The total is 133,662 frames, 37:07.7.
  - YouTube's rules hold: the list starts at 0:00, rises, and every chapter is at least 10 s (the shortest, 12, is 65.7 s).
  - The cue tables hold all 105 cues in exact script order.
- **Script.** `script-reviewed.tsv` is byte-equal to the `script.tsv` in the review-repair268 workspace, the production19 workspace and csd's Q (`234f7257…`).
- **Promises against the 105-cue film.** Every topic the description names appears in the script:
  - the bump and the poles at ±i;
  - grids, colour, zeros and poles;
  - the gallery, including tangent, gamma and zeta;
  - the Cauchy–Riemann equations, critical points, holomorphic functions;
  - the integral formula, rigidity, the argument principle, residues;
  - the two-pole integral of 1/(1+x⁴);
  - continuation and its three limits.

  The identity theorem (10_1) and the natural boundary (14_7) are shown but not named aloud, which is acceptable.
- **Dashes and length.**
  - No em dash anywhere. The one en dash, in "Cauchy–Riemann", is correct.
  - No angle brackets.
  - The title is 40 characters, the description 2,141 characters, the tags 278 characters.
- **Sister link.** `lgpTDROHgQY` is public in the catalog. "Logarithms Made Visual" shortens its published title "…: From Slide Rules to Matrices", and 14_6 ("That story has its own video") matches it.
- **Thumbnail.**
  - The selected `publication19-thumbnail208/thumbnail.png` is byte-identical to the reviewed native `thumbnail24-reviewed/complex_analysis_visual_thumbnail.png` (`e5dfc2bd04aa3af148ee66ca8107594acf3015a237f5f36a08f1c3c72c4c2eab`, 3840×2160).
  - Upload-assets298, `review.json` and `picture-review.json` all name that hash.
  - I viewed it: a smooth two-turn z² field with "The Beauty of Complex Analysis / A Plane in Color".
  - **Root's call, not a correction:** its headline differs from the title, and "Color" differs from the description's "colour".

**Not done:** no rewrite, no competitor search, no edits to the draft, source, Beads or TODO.
