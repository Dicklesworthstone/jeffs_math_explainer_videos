# cc1 caption runtime 331: native captions from caption-preflight327

Task 331 reads the receipt and the complete SRT and VTT from root's guarded job `csd-caption-preflight327` (the recipe is in 327). It compares cue text and timing coverage with the script, the rebind269 report and the timeline269 schedules. It inspects fast, long and short captions with the existing pinned caption screen, run only in a local scratch output outside the repository.

This is a reading and comparison of caption files. It is not listening, sync, accessibility or final AV acceptance, and no ASR was used. No caption, source, Bead or TODO was edited. No job, export or write ran on any host: the CSD reads were `sha256sum`, `ls`, `stat` and one read-only Python listing. Root will compare these captions with 318.

## 1. The job, as its receipt shows it

`D` is the CSD project root `/data/projects/jmev-stroke-probe-20261106`.

| Field | Value |
|---|---|
| Receipt | `D/jobs/csd-caption-preflight327/resource-receipt.json`, sha256 `47277c23dad18e2c911da53c3d563dde271015a6af3abb8b855c1b51ab5ae06e` |
| Started / admitted / finished (UTC) | 2026-11-08 20:25:04.597 / 20:25:04.630 / 20:30:02.740 |
| Exit, error, resource stop | `exit 0`, error `null`, `stopped_for_resource_limit null` |
| Inputs | 701 rows; `inputs_sha256 = final_inputs_sha256 = de8245060e716839c6e0e8b18bdfba4575c90e97604e4e9560e335f2687401b0` |
| Admission | free 105,725,366,272 B (sbh `check` exit 0, status ok); policy 10/8/4/56/20 GiB, 10 s samples |
| Command | `/usr/bin/timeout 3600 /bin/sh D/qualification01/production-repair17/caption-preflight327-run.sh` (run.sh sha256 `7609dfbb64716c13762b150d21f72908552974d7f2c0b229ec1ab6b9e0a975c8`) |
| `command.log` (419 B, `8101ababa6084a2247c3112fd0967d8debd14695a297f4b833d7887f5a325e93`) | contract `consumer captions`, `verified_cues 105`, `selected_cues 105`, `plan_mode external-ui`, `directed_caption_segments []`, `listening_acceptance false`, `promotion false`; then `captions: 461 sentences as 571 captions` |
| Job folder | `captions/`, `command.log`, `resource-receipt.json` only |
| `captions/` | 15 symlinks, one to each timeline269 `KEY.timeline.tsv`, plus the two outputs below |
| SRT | `complex_analysis_visual.en.srt`, 48,287 B, `5d06423248f088a34774f361721bf05e1c4950ac834086e3666661e1adfe5e1b` |
| VTT | `complex_analysis_visual.en.vtt`, 46,119 B, `26e0d7e0eec26eb6f9136bb56f795a6ff8ab9cead26f29fc76a75e9abc4bb2e3` |

These were fetched and re-hashed locally, and every copy equals the CSD listing:
- the 15 timeline tables;
- `report.json` (`b4a4c91e8e8fd26dbad621d118b762660e60532a2bbcf535ad88b87a45406bdd`);
- the SRT, VTT, receipt, inputs file, run.sh and log.

The 15 symlinks name exactly the 15 fetched tables. Both caption files are ASCII.

**Negative observable: no writes to the existing timeline and audio folders.**
- **timeline269** (`jobs/csd-review-repair-timeline269/timeline-output`, 46 entries): the folder's mtime and ctime, and the newest entry ctime and mtime, are all 2026-11-08 07:10:19.658Z. That is before the job started.
- **rebind269 narration** (109 files): the newest ctime and mtime are 05:36:36.160Z, also before the job started.
- The workspace `script.tsv` is still `234f7257…` and `report.json` is still `b4a4c91e…`.

## 2. What was compared and how

Every check below ran locally on the fetched copies, with these tools:

- **`verify_stage09_final.py` `check_captions`.** Pinned at `f266a231…`, local copy at `docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-final-verification02/`. It ran against `script.tsv` `234f725781bd…` (build268's source copy) and against placements built from the 15 timeline269 tables. The tables are offset in registry order `01_the_bump` … `15_closing`, and each cue lasts `sum(out_samples + gap_after_samples) / rate` from the report. The placements total 133,662 frames at 60 fps (2,227.700 s), and the cue order equals the script.
- **`caption-interval-audit01.py` `screen`.** Pinned at `3cc26436…`, local copy at `narration_lab/third-wave-20261024/root/six-machine-screened-production-20261103/`. Its scope is more than 20 chars/s and lines longer than 42 characters.
- **A per-caption table** of cue, times, chars/s, line lengths and offsets from cue start and audio end. I read every row of it: captions 1–571.
- **A Python mirror of the kit's `wrap` / `number_word` / `balanced_captions`.** It mirrors `crates/explainer_kit/src/captions.rs` `28e1aa51…`. That file is byte-equal to the kit binary268 links against: build268's pinned `Cargo.toml` (`7644838e…`) takes `explainer_kit` by path from `D/work/tree/repo-view/crates/explainer_kit`, whose `src/captions.rs` is `28e1aa51…` with mtime 2026-11-06 02:02Z, before build268. Fed the report's 461 sentences, it reproduces the line groups of all 571 SRT captions exactly. That positive control locates the defects below in the source.

| Check | Result |
|---|---|
| SRT = VTT | equal, 571 blocks each |
| Order, inside the video | yes; first caption 0.600 s, last ends 2,220.147 s of 2,227.700 s |
| Text = script | whole and per cue (all 105 cues); the report's `sentence` equals its `tts` in every row |
| Screen: more than 20 chars/s | 0; fastest is caption 459, "You may know it from calculus.", 19.23 chars/s over 1.56 s |
| Screen: lines longer than 42 characters | 0 (longest is 42) |
| Sentence starts | all 461 start exactly at the report's recorded sentence start (0 ms difference) |
| Cue coverage | every cue's first caption starts at its timeline269 placement (0 ms); no gap inside any cue; covered fraction ≥ 0.9999 (the 3 at 0.9999 are under 1 ms of millisecond stamp rounding) |
| Cue ends | 92 cues end exactly at placement + report duration. The other 13 hold their question caption to the next cue's placement (5.85–6.86 s past the audio), one for each of the 13 `predict` marks in timeline269. No answer caption appears before its cue |
| Gaps between cues | 104 gaps: 13 are zero (the holds), the other 91 are 0.600–11.204 s (the longest is after caption 440, 11_8 → 12_1). These caption-free spans are the timeline's silences between cues |
| Shape | 177 one-line and 394 two-line captions; durations 0.660–11.783 s; 13 over 7 s (all the prediction holds); 14 under 1.5 s |

On this evidence, the native export from binary268 matches the script, rebind269 and timeline269 completely in text and timing coverage. The defects below are in **how sentences are split** into captions and lines.

## 3. Readability and content defects

Severity is caption readability, not audio. "Mid-sentence change" means the 110 caption changes that fall inside a sentence (571 − 461).

### F1 (Medium): kit fallback splits a spoken number across captions despite its own rule

- **Captions 478/479** (13_5, 30:51.827, 5.10 s then 2.94 s):
  - 478 reads "They sit on the unit circle at forty five, / one hundred thirty five, two hundred".
  - 479 reads "twenty five and three hundred fifteen / degrees."
- For 5.1 s the screen reads "two hundred" as the third angle, which is 225°.
- **The rule this breaks.** `captions.rs:372–374` and `:440` exist so that "'two hundred' must not disappear before 'seventy-seven' replaces it."
- **Source reason.**
  - `balanced_captions` fixes the caption count at `greedy.len().div_ceil(MAX_LINES)` (`:428`), which is 2 for this 125-character sentence.
  - The number run "forty five, … two hundred twenty five" leaves no 2-caption split that both keeps the number together and fits each caption in 2 lines, so `best[2][n]` is `None`.
  - The fallback at `:464–466` then returns the greedy wrap's lines two at a time, ignoring the number rule. Its comment ("not reachable") is wrong when `:440` rules out every split.
- A 3-caption split would satisfy both limits: "They sit on the unit circle at" | "forty five, … two hundred twenty five" (2 lines) | "and three hundred fifteen degrees.". The search never tries a third caption.
- Across all 461 sentences the mirror reaches this fallback once, and this is the only caption change that falls between two number words.

### F2 (Medium): mid-sentence caption changes that read as a different value or expression

`number_word` (`:375–418`) protects only adjacent pairs of listed number words. "point", "minus", "over", "pi", "root", "half", "squared" and "percent" are not protected, and `balanced_captions` weighs only character balance (`:420–477`). These captions show a misleading value or expression until the next one replaces them:

| # | Cue, start | Shown on screen | Then | Misread |
|---|---|---|---|---|
| 406 | 11_3 25:59.150 (3.12 s) | "Here is a function with a zero at zero / point" | "four five and a pole at one point three / five." | a zero **at zero** (it is 0.45) |
| 470 | 13_3 30:25.783 (3.36 s) | "So that part of the loop is at most pi R / over R" | "squared minus one, …" | bound reads πR/R = π, not πR/(R²−1) |
| 216 | 07_5 13:52.063 (3.37 s) | "This function has a complex derivative at / exactly one" | "point in the whole plane, …" | "a derivative at exactly one", just after the voice showed it fails at the point one |
| 137 | 05_7 08:51.067 (3.71 s) | "The zeta function starts as one, plus one / over two" | "to the s, plus one over three to the s, …" | 1 + 1/2, not 1 + 1/2^s |
| 335 | 09_3 21:12.477 (3.00 s) | "Eight sample points already average to / within half" | "a percent of one, …" | within a half, not 0.5 % |
| 350 | 09_6 22:09.740 (3.21 s) | "Turn back by twice the angle and divide by / the radius" | "squared, and you get the next coefficient …" | repeats the slope rule of 348; radius squared arrives later |
| 326 | 09_1 20:38.010 (4.26 s) | "The magnitude of its integral is at most / its circumference, two" | "pi times its radius, …" | circumference "two" |
| 45 | 02_6 02:53.433 (3.03 s) | "A quarter turn, with the stretch of the / square" | "root of two applied twice, …" | "the stretch of the square" |

These captions end on a dangling operator or a partial number, without reading as a wrong value:
- 362, 09_9 23:00.367: "…works from about minus" | "point four one …"
- 134, 05_6 08:38.983: "…pole at zero, at minus" | "one, at minus two, …"
- 483, 13_7 31:14.133: "…its residue is / one over" | "the derivative of the bottom, …"
- 496, 13_10 31:58.597: "…one over R / to the fourth minus" | "one, …"
- 532, 14_5 34:31.970: "…the famous claim that / one plus" | "two plus three, …"

Every one of these resolves within one caption change, and the full text equals the script.

### F3 (Low): mid-sentence caption changes after a function word

43 of the 110 mid-sentence changes leave a caption ending on a function word, and only 11 of the 110 fall at punctuation. Examples:
- 20/21 "only works inside the" | "largest circle around zero"
- 50/51 "Every complex derivative in this video / will" | "be one of these …"
- 64/65 "bends / into" | "a parabola"
- 118/119 "a / pole of" | "order two"
- 204/205 "is a" | "critical point"
- 261/262 "because / the" | "steps around a closed loop"
- 298/299 "hole in the" | "middle"
- 314/315 "give / the" | "polygons"
- 516/517 "has at" | "most one holomorphic extension"

The cause is the same as F2: the split has no cost for breaking a phrase.

### F4 (Low): greedy line wrap leaves short second lines inside a caption

`wrap` (`:350–364`) fills the first line to 42 characters. As a result, 110 of the 394 two-line captions have a second line of 8 characters or fewer.

- **Lone function words:** "the" (20, 261, 314), "will" (50), "into" (64), "real" (67), "and" (144), "for" (306), "it" (563), "point" (406).
- **A symbol cut off from its expression:** 330 "…two pi i times f of / a.", 445 "…except one over z minus / p.", 462 "…from minus R to / R,".
- **A number broken across lines in one caption:** 363 "two point four / one.", 407 "one point three / five.", 478 "forty five, / one hundred". Both lines show together here, so these are less serious than F1/F2.

### F5 (Low): three prediction answers flash for under 5/6 s

- 63 "No." (03_4, 03:56.900, 0.700 s)
- 385 "No." (10_5, 24:32.700, 0.817 s)
- 420 "One." (11_6, 26:57.617, 0.660 s)

Each is the reveal after a held question. Each is replaced, with no gap, by a two-line caption, and each lasts less than the 5/6 s minimum used by common timed-text style guides. Their length is the sentence's own recorded span in rebind269's report (the sentence start matches to 0 ms), so the kit timed them faithfully. A fix would merge or extend the caption, not change the audio. The other 11 captions under 1.5 s ("Hold on to that.", "Here is why.", "Nowhere.", "It does.", …) are 1.06–1.48 s and short.

### F6 (Low, script-level; captions copy the script verbatim)

- **No hyphens.** The caption text contains no hyphen at all (0 in 571 captions), so these appear unhyphenated:
  - 182 "These are the Cauchy Riemann equations" (the standard name is Cauchy–Riemann, two people);
  - 498 "less than / one three hundredth" (one three-hundredth);
  - 386/388 "pile up point";
  - 91 "yellow green";
  - 286 "upside down one";
  - compound numbers ("twenty five", "forty five").
- **Lowercase start.** 112 (05_1, 07:10.873) starts a sentence lowercase: "z cubed minus one has three zeros, …".
- **Mixed spelling.** British "colour" (15 occurrences) and "centre/centred/Recentre" (10) appear beside "counterclockwise" (6) and "clockwise" (1); the British form would be "anticlockwise". This is root's call, not a caption-pipeline defect.

### Reading speed (screened, not a defect here)

- 0 captions above 20 chars/s.
- 19 captions above 17 chars/s, clustered in the dense proof passages: 08_5 (276, 281, 282, 284) and 09_5–09_7 (345, 346, 352, 355).
- 178 of 571 (31 %) are above 15 chars/s.

The caption pace is the narration's own pace, shared by character count. The doctrine forbids speeding narration up or slowing it down, so the only levers are wording and caption splitting.

## 4. Not established

- **Caption-to-voice sync of the 110 mid-sentence changes.** `sentence_captions` (`:479–501`) shares each sentence's recorded span in proportion to characters. These change times are estimates, so a caption can change before or after the word is spoken; I did not measure this. Sentence starts are bound to recorded boundaries (§2). The caption-free lead-in silence at each cue start was not measured either.
- **Listening, accessibility, legibility, and fine sync on the final 4K master.** Not done. The screen's own method note says its rates are screening flags, not acceptance.
- **The 318 comparison.** Root will compare these captions with 318. If 318's `captions all` uses this same binary268 with these same timeline269 tables and rebind269 report, I expect its SRT/VTT to be byte-identical to `5d064232…` / `26e0d7e0…`. That is a prediction only; I have not read 318's captions.
- **Repairs.** Nothing here is repaired. F1–F4 are kit split and wrap behaviour (`captions.rs` `28e1aa51`); F5 is a kit duration-floor choice; F6 is script wording. Fixing any of them needs a new caption export, and that decision belongs to root.
