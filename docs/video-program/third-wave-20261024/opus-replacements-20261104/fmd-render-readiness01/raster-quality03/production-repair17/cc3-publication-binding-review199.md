# cc3 publication binding review 199: complex analysis draft03

Dispatch-3 (cc3), 2026-11-08 03:32 UTC. Source review only.

This file is the only file I wrote. I made no edits to code, shared files, Beads, packaging or earlier outputs, and I ran no render, synthesis or publication. `publication19-draft03/` and `cc3-publication-review197.md` are unchanged.

I did no listening. Every narration statement below rests on receipts, hashes and one fallible recognizer comparison.

Paths are relative to this directory (`production-repair17/`) unless they start with `narration_lab/` or `explainer_corpus/`, which are relative to the repository root. Job names are root's guard job names.

## Answers

1. **Authoritative script.** It is the production-repair19 workspace's `explainer_corpus/complex_analysis_visual/script.tsv`:
   - sha256 `234f725781bde0bdf35afbc21fd2d6ec0f73f99a95723294259cc1b75a813d3c`, 29,103 bytes, 105 cues.
   - Byte-identical local copies:
     - `script.tsv`
     - `hz2-review07-84/script-production19-review34.tsv`, plus the four other copies of that file (in `hz2-review10-85/`, `hz2-review12-59/`, `hz2-review15-53/` and `unchanged-preview16-review30/`)
   - Its 08_5 (502 words) and 09_1 (366 words) are the full proofs.
   - Review197 quoted exactly this file, by the same hash.
2. **The difference root noticed comes from the committed corpus script, which is out of date.**
   - The repository's `explainer_corpus/complex_analysis_visual/script.tsv` (sha256 `2968b0fd…929e`) was last committed in 2fb1c0c7 on 2026-11-04.
   - Its header reads "Opus replacement candidate … narrator ElevenLabs Oliver, unsynthesized".
   - It differs from the bound script in 9 cues: 05_8, 08_5, 09_1, 09_5, 10_3, 11_7, 14_1, 14_3 and 15_2.
   - Its 08_5 and 09_1 are short sketches of 71 and 64 words.
   - No master chapter, permit or narration record binds it.
3. **Draft03 is faithful** to the bound script and to the retained narration records. Each claim is matched to its cue in the table below.
4. **One substantive issue remains, and it lies outside the packaging.** The committed source does not contain the script the film speaks. The copy issues that remain are minor and optional (listed below).

## Evidence chain: master-plan183 to the script and takes

| Link | What I checked | Result |
|---|---|---|
| Workspace to script | `trj-production19-transfer-manifest01.json` | `work/production-repair19/workspace/explainer_corpus/complex_analysis_visual/script.tsv` is `234f7257…3d3c` |
| Repaired chapter 09 source | `repaired09-review158/source-comparison158.json` | `script.tsv` is `234f7257…` in both the old and new workspaces. Only `narration_units.json` and `src/chapters/loop_sums.rs` changed |
| Permits to script | The 15 permits named in `master-plan183/chapter-plan183.json`, found locally under root's review folders | All 15 found. Five also bind `script-production19-review34.tsv` as `234f7257…`: chapters 06, 07, 10, 12 and 15. The others bind binary and timeline only |
| Permits to timelines | Same 15 permits | 13 bind exactly the `timeline_sha256` that draft03's `review.json` lists (see the two rows below for 08 and 09) |
| Repaired 08 timeline | `timeline165-terminal176/comparison.json` (root's comparison) | The permit's `338f16a8…` is the rebind of draft03's `fed5faea…`, "equal after named narration path mapping". Its cue table hash is unchanged, and its cue log is equal after the same mapping |
| Repaired 09 timeline | `repaired09-review158/timeline69-comparison.json` and `two-label-terminal156/comparison.json` (root's comparisons) | The permit's `4cbb007a…` is the rebind of draft03's `ae7a13f4…`, with the same "equal after mapping" result |
| Timelines to narration | All 15 `timeline21-terminal01/*.timeline.tsv` and `*.cuelog.tsv` (recomputed here) | Every timeline hashes to draft03's `timeline_sha256`, and every frame and cue count equals draft03's. The 105 lines follow the bound script's cue order. Every cue-log WAV hash equals the assembly19 narration hash in the transfer manifest |
| Path-only differences in finals | `final04-terminal198` and `final08-terminal64` timelines against timeline21 (recomputed here) | Equal after replacing the narration folder. The chapter 04 final's cue log carries the same WAV hashes |
| Assembly19 to the takes | `prepare-assembly19.py`, `audio17/` | The helper refuses unless the text differs from the prior assembly in exactly {08_5, 09_1}, and unless the selected takes' text equals the script. Both local requests (`audio17/request08_5.json`, `audio17/request09_1.json`) have text equal to the bound cue, a `script_fragment_sha256` equal to `08_5-script.txt` / `09_1-script.txt`, voice "Frederick Surrey - Smooth and Velvety", and model Eleven v4. The assembled 08_5 and 09_1 sizes match the takes' sample counts (4,287,360 and 3,304,320 at 24 kHz) |
| The 103 retained cues | `assembly-inputs16/` (the prior assembly) and the transfer manifest | The 103 assembly19 outputs are byte-identical to the prior outputs. The prior unit texts equal the bound script except at 08_5/09_1. Every unit WAV hash equals its decode record |
| Takes to submitted text | `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/production-audio-stage07-01/decoded01/decode-receipt.json` | 105 records: 54 requests from `frederick-stage05-01`, 50 from `frederick-stage06-01`, and one (09_5) from `remote-ts1-01/cauchy-cue-repair01`. Every request file matches its recorded hash, and its submitted text equals the record. For all 103 retained cues, that text equals the bound script exactly. The old 08_5/09_1 requests carry the short text of `before-script.tsv` (`946a9c07…`), and assembly19 replaced exactly those two |

The receipts record the voice setting in the ElevenLabs UI, not a voice anyone heard. The credit "ElevenLabs Frederick Surrey, a synthetic voice" claims no more than that.

**Recognizer comparison (fallible, not listening).** The primary ASR of the two new takes (`audio17/08_5-primary-asr.json`, `audio17/09_1-primary-asr.json`) matches 491 of 502 and 356 of 366 script words. No unmatched run is longer than three words. The differences are:
- digits ("4^n" for "four to the n", "2pi" for "two pi");
- one "too" heard as "two" in 08_5;
- one "of a" missing from the 09_1 transcript.

Only listening can settle whether that "of a" was spoken.

Not checked:
- the remote final timelines and binaries themselves (root's comparisons are quoted above);
- the final master, which is not built;
- the pictures;
- the thumbnail;
- any audio by ear.

## Draft03 claims against the bound script

| Draft03 text | Bound cue(s) | Verdict |
|---|---|---|
| A smooth curve's Taylor series stops working; the curve has no visible trouble | 01_1, 01_3, 01_4 | Faithful |
| We leave the real line for the complex plane | 01_5, 01_6 | Faithful |
| Grids, circles and colors through polynomials, reciprocals, exponentials and trigonometric functions | 03_2–03_6, 04_1–04_5, 05_1–05_5 | Faithful. Gamma and zeta (05_6–05_8) are left out, but nothing is misstated |
| A complex derivative has to agree from every direction | 06_2 | Faithful but loose (see L1) |
| Being holomorphic throughout a region is a strong condition | 07_5, 07_6 | Faithful. The script says "open region"; "region" carries that meaning |
| Shared edges cancel, nested squares shrink | 08_5 | Faithful to the 502-word bound cue, not to the 71-word corpus cue |
| Loop integrals lead to Cauchy's integral formula | 09_1, 09_2 | Faithful to the 366-word bound cue |
| Count zeros and poles | 11_1–11_8 | Faithful |
| Residues evaluate two real integrals | 12_4, 13_2, 13_7–13_9 | Faithful (π and π/√2) |
| Continuation beyond a power series' disk | 14_1–14_3 | Faithful |
| Identity theorem, Liouville's theorem and argument principle stated and illustrated, not proved | 10_1, 10_6, 11_2 | Faithful. Each is stated, then shown on examples |
| Assumes derivatives, integrals and power series | 01_2, 06_1, 13_1 | Fair. 13_1 says "You may know it from calculus" |
| DLMF §1.9 "Calculus of a Complex Variable", §1.10 "Functions of a Complex Variable" | none (sources) | Titles match the DLMF headings fetched on 2026-11-08 |
| The complex logarithm has its own film (lgpTDROHgQY) | 14_6 ("That story has its own video.") | Faithful. YouTube oEmbed returned it at 03:32 UTC as "Logarithms Made Visual: From Slide Rules to Matrices", on the same channel |
| Narration credit | All 105 requests | Faithful (Frederick Surrey, Eleven v4, on every request) |
| Music: procedural score generated in code | `score22.py` runs the film binary's `score all` | Faithful |
| Chapter labels 07, 08, 13 | 07_1–07_6; 08_1–08_5; 13_2, 13_9 | Faithful. Chapter 08 no longer says the loop integrals cancel |
| 15 timestamps | draft03 `review.json` start frames | Each start, at 60 fps, gives exactly the printed time (recomputed here). The starts are contiguous, 133,662 frames in total. They are identical to draft02 |

Measured from the draft03 files:
- The title is 62 characters.
- The description is 1,977 characters.
- The tags come to 242 by YouTube's count, against a limit of 500.
- There are no em dashes or angle brackets.
- All three files match the hashes in `publication19-draft03/review.json`.

## Remaining issues

**S1, substantive, outside the packaging: the committed source does not contain the script the film speaks.**
- The corpus script in this checkout, `2968b0fd…`, is the 2026-11-04 Oliver-era candidate.
- The bound script, `234f7257…`, exists only in production staging (`script.tsv` here) and in the remote workspace.
- AGENTS.md says a finished video's published cut stays reproducible from its tagged commit. Today, anyone working from the committed corpus would read and rebuild the old 71- and 64-word 08_5 and 09_1, and seven older cues besides.
- Before publication, root should decide how the production-repair19 source is preserved and tagged: at least the script, `narration_units.json`, the chapter sources and the direction map. This is root's call. It does not change draft03.

The following are low priority and optional; none makes draft03 unfaithful.

- **L1.** "See why a complex derivative has to agree from every direction" is loose, since the derivative is one number. 06_2's own sense reads more precisely as: "See why the ratio defining a complex derivative must give the same answer from every direction".
- **L2.** The tags are unchanged from draft02.
  - The narration never says "Goursat theorem", and the usual search term is "Cauchy-Goursat theorem".
  - With 258 characters to spare, terms the film covers could be added: Cauchy-Riemann equations, contour integral, Riemann zeta function, Riemann hypothesis, fundamental theorem of algebra, Liouville's theorem.
- **L3.** The opening paragraph is accurate but generic. Review197 suggested a concrete version from 01_3: at x = 1.1, the series up to x⁴⁰ gives about 25, while the curve is below 1/2.

## Still open, as draft03's own `review.json` records

- Re-read the final master times once the master is joined. The 13 unchanged timelines and the path-only 08/09 rebinds keep the frame counts that the timestamps come from, so no change is expected.
- Review the thumbnail.
- Final audiovisual review. Nothing here is listening, picture or sync acceptance.
