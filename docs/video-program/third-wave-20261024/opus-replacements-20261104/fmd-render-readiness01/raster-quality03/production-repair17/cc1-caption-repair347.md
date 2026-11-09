# cc1 caption repair 347: caption-only candidate from the native 327 captions

**Status: candidate, frozen for independent review.** It has not been accepted, published or muxed. Root decides whether to adopt it.

**Scope.** The repair changes caption files only. No audio, narration, source, timeline, Rust, engine, build, render, synthesis, Bead, TODO or canonical file was edited or run. The 327 SRT/VTT on CSD (`D/jobs/csd-caption-preflight327/captions/`) are untouched; this work read local copies, verified byte-equal by sha256 (331 §1).

**Freeze and lease.**
- The 331 review is unchanged at sha256 `dcb70cfdf7e08f2cb8e47673d40878a343cf0e0a762bc78a09c217f5b3f93bf2`: 15,184 bytes, 15,070 characters. Root's note says 15,182; please compare against the hash.
- The writes ran under Agent Mail lease 6250, held by BronzeJaguar, exclusive, on `Q/cc1-caption-repair347*`, expiring 2026-11-08 22:46:09Z. The broader pattern `*347*` overlapped BlueSalmon's leases `caption-preflight327-*` and `preview313-copy*333*`, so I took only the narrow prefix. Every file below is new.

## Files

| File | sha256 | Bytes |
|---|---|---|
| `cc1-caption-repair347.py` (tool) | `61fee2e97b1febf23927926a5e9886032d6d004414e938e014ba8f97c258cdb8` | 23,973 |
| `cc1-caption-repair347-overrides.json` (163 hand-authored entries + 1 exception) | `4b8330c430f1392192a965db6bdcd6c534f04addecd8a8d9ec56bbf7b29b20cf` | 52,729 |
| `cc1-caption-repair347-candidate.en.srt` | `0ea9277e6d598540e5e9057a84ee84daf7f88420b076434659fc3d3ad0ff3230` | 48,567 |
| `cc1-caption-repair347-candidate.en.vtt` | `6a383cb7f207be81254e9a820ecb701ca77d00a26557194c675dc1717ef6979b` | 46,367 |
| `cc1-caption-repair347-changes.tsv` (327 and 347 rows of every changed unit) | `2ad1a40b845bca205bb5075aeb16823d4ac8a94c17b546415b4031d7f0839b63` | 75,259 |
| `cc1-caption-repair347-checks.json` | `8b3ef4f40eaf35adfbea2df8f8faa3cf4711c897b5cfb70027bb9265bd9ed458` | 5,585 |

To rerun from the repository root (it refuses existing outputs, so use a new prefix):

```
python3 -I -B Q/cc1-caption-repair347.py INPUTS SCRIPT Q/cc1-caption-repair347-overrides.json NEWPREFIX
```

- `INPUTS` holds the 327 `complex_analysis_visual.en.srt`/`.vtt`, the rebind269 `report.json` and the 15 timeline269 `KEY.timeline.tsv`.
- `SCRIPT` is build268's `script.tsv`.
- The tool pins all of these by full sha256, plus the two existing checkers: `verify_stage09_final.py` `f266a231…` and `caption-interval-audit01.py` `3cc26436…`.

A second run into a new scratch prefix produced byte-identical SRT, VTT, TSV and JSON.

## Method

**Rules.** Each 327 sentence's caption groups were tested against these rules:

| Rule | Where | Fails when | 331 class |
|---|---|---|---|
| C1 | caption change inside a sentence | the earlier caption ends on a function word (*the, a, of, and, so, that, though, will, into, at, …*) not followed by punctuation | F3; F2 when the word is an operator (*minus, plus, over, point, times*) |
| C2 | caption change | it splits an expression, with no punctuation between: two adjacent number, variable or operator words (*zero \| point*, *R \| squared*, *two \| pi*, *half \| a*), a following *squared/cubed/prime*, an exponent *X \| to the*, or *square \| root* | F1 for two number words, otherwise F2 |
| L1 | inside a two-line caption | a line is under 9 characters | F4 |
| L2 | line break | it splits an expression as in C2 | F4 |
| L3 | line break | line 1 ends on an article (*the, a, an*) | F4 |

**What failed.**
- 163 of the 461 sentences fail at least one rule in 327: F1 1, F2 23, F3 39, F4 205 failures.
- 331's F3 count of 43 used a word list that counted *minus, point, plus, over, times* as F3; those now fall under F2.
- `L3` adds 31 sentences that 331 did not list. They are the same kind of line/phrase defect. Examples:
  - "plus a / linear term";
  - "the / upside down | one in the middle";
  - "integral's | size".

**Overrides.**
- `overrides.json` gives, for each of the 163 units, the exact report sentence text(s) and the hand-authored caption groups, line by line.
- The tool refuses any entry whose words, in order, differ from the report sentence.
- Every other sentence keeps its 327 captions, byte-identical.
- There is no general splitter. The rules only check; the splits are chosen by hand.

**Timing.**
- Timing is explainer_kit `captions.rs` `28e1aa51` exactly:
  - recorded sentence spans from the report's sample counts;
  - each span shared between a sentence's captions in proportion to their characters;
  - cue offsets from timeline269;
  - prediction holds from its 13 `predict` marks.
- **Positive control:** fed the 327 caption groups, the tool reproduces all 571 327 stamps to the millisecond. It refuses to run otherwise.
- So every sentence keeps its 327 start and end. Only the split times inside a sentence move, and they remain character-share estimates.

**F5 merges (3).** A merge is allowed only when all of these hold:
- the answer opens line 1 of the following sentence's first caption;
- that caption starts at the answer's own recorded start, which is the cue placement and the end of the held question;
- the following sentence's internal shares are computed over its own span;
- the unit ends at the 327 end.

## Results (`checks.json`, `all_passed: true`)

| Check | Candidate |
|---|---|
| Pinned `check_captions` (`f266a231`) | SRT = VTT, 579 blocks, text equals the script **whole and per cue**; last caption ends 2,220.147 s of 2,227.700 s |
| Pinned screen (`3cc26436`) | 0 captions over 20 chars/s, 0 lines over 42; fastest 19.231 chars/s ("You may know it from calculus.", unchanged) |
| Structure (kit `validate`) | 1–2 lines, longest line 42, start < end, no overlap |
| Exact text | word stream equal to 327 for the whole video; per cue equal to the report sentences |
| Sentence/unit boundaries | all 458 units start and end at their 327 times (461 sentences; the 3 merges each make one unit of 2) |
| Cue coverage | all 105 cues keep their 327 first start and last end, contiguous with no gaps |
| Prediction holds | all 13 end unchanged at the next cue's placement, each on its question's last words |
| Unchanged units | all 295 byte-identical to 327 |
| Durations | minimum 1.060 s, down from 0.660 s; none under 1 s; changed captions at least 1.391 s; maximum chars/s unchanged at 19.231 |
| Rules | 0 failures except the one listed exception |
| Caption count | 571 → 579; the 163 changed units go from 266 captions to 274 |

163 units changed: 160 re-split and 3 merged.
- **67** keep their 327 stamps exactly; only the line breaks or the line text moved.
- **96** have new split times inside the sentence, or a different caption count.

| Code | Units |
|---|---|
| F4 | 146 |
| F3 | 39 |
| F2 | 17 |
| F5 | 3 |
| F1 | 1 |

A unit can carry several codes.

## The 331 defects, before and after

The table uses 327 and 347 caption numbers.

| 331 | 327 | 347 |
|---|---|---|
| F1 478/479 | "…one hundred thirty five, two hundred" \| "twenty five and three hundred fifteen / degrees." | 485 "They sit on the unit circle / at forty five, one hundred thirty five," \| 486 "two hundred twenty five / and three hundred fifteen degrees." |
| F2 406 | "…with a zero at zero / point" \| "four five …" | 411 "Here is a function with a zero / at zero point four five" \| 412 "and a pole at one point three five." |
| F2 470 | "…at most pi R / over R" \| "squared minus one, …" | 477 "So that part of the loop is at most / pi R over R squared minus one," |
| F2 216 | "…derivative at / exactly one" \| "point in …" | 217 "This function has a complex derivative / at exactly one point in the whole plane," |
| F2 137 | "…one, plus one / over two" \| "to the s, …" | 137 "The zeta function starts as one, / plus one over two to the s," |
| F2 335 | "…average to / within half" \| "a percent …" | 339 "Eight sample points already average / to within half a percent of one," |
| F2 350 | "…divide by / the radius" \| "squared, …" | 354 "Turn back by twice the angle / and divide by the radius squared," |
| F2 326 | "…its circumference, two" \| "pi times …" | 330/331 "… / is at most its circumference," \| "two pi times its radius, times that bound, / and that goes to zero." |
| F2 45 | "…stretch of the / square" \| "root of two …" | 46 "A quarter turn, with the stretch / of the square root of two" |
| F2 dangling (362, 134, 483, 496, 532) | "…about minus" \| "point four one …" etc. | 366 "So the new recipe works / from about minus point four one"; the other four likewise end on complete terms |
| F5 63 "No." 0.700 s | "No." then "Every vertical line … bends / into" | 64 "No. Every vertical line off the axis / bends into a parabola," (236.900–241.324 s) |
| F5 385 "No." 0.817 s | "No." then "The pile up point … is not / in the" | 390 "No. The pile up point is zero, / and zero is not in the region," (1,472.700–1,477.310 s) |
| F5 420 "One." 0.660 s | "One." then "Walk the unit circle, …" | 426 "One. Walk the unit circle, and the output / turns around zero exactly once." (1,617.617–1,623.617 s) |

**F5 details.** In each merge the answer appears exactly at its recorded start, which is also the end of the held question; no caption appears before the hold ends. The following sentence's first words therefore appear 0.700, 0.817 and 0.660 s before their recorded sentence start, inside the same caption as the answer. Its last caption still ends at its 327 time.

## Exceptions and residuals

- **14_4 (534), kept as the 327 caption:** "So what does one plus two plus four plus / eight, and so on forever, add up to?". This is the one listed `L2` exception. The question is held for 5.86 s, and at 76 characters no two-line break avoids splitting the sum. Two captions would hide the sum's start during the hold.
- **04_6 (106/107, unchanged; passes the rules):** this held question spans two captions, so during the hold only "and how many times will the colours turn / around each one?" is visible. Root may want the whole question on screen; that is outside F1–F5.
- **F6 spelling and style are unchanged, as instructed.** This includes "Cauchy Riemann", "one three hundredth", the lowercase "z cubed …" and colour/counterclockwise.
- **Internal split times** (96 retimed units, plus the 3 merges) are the kit's character-share estimates. They are not word timing and not fine-sync acceptance.

## Limits

These are caption files and their checks only. There was no listening, ASR, sync, accessibility or final AV review. Nothing has been muxed into a master, uploaded or published, and the files have not been compared with 318. An independent review is to follow; this candidate is not accepted.
