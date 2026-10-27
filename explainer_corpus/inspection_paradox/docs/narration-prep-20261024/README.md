# Inspection paradox: script-first narration plan (2026-10-24)

This is a plan only. Nothing here was compiled, synthesized, measured,
transcribed or listened to. It does not claim natural pacing, Michael's
identity or English-accent acceptance. It also does not claim that any
unit size or word count gives good speech: the skill's bounded experiment
still comes first (step 4).

## What it binds

The script is the eventual one: the live reviewed 01–09 (67 cues), plus
the two delivered patches, applied in dependency order to an isolated copy:

1. the 10/11 patch, which is `docs/integration-10-11.md` lines 617–1029
   (sha256 `c42705490f51d03cefe3a79b950b759118e885d68ff0932c5f33cdba5f102305`);
2. then the 12 patch, which is `docs/integration-12.md` lines 545–907
   (sha256 `7d84592dbe539f5fc73c960c6810ee99a31d8c8c2c437fd87f734f482f287cb9`).

The script was then written with `tools/storyboard.py`'s own `parse`,
`check` (no problems) and `script_text`. The result is
`projected-script.tsv`: 92 cues over 12 chapters, 13,703 bytes, sha256
`8a65ec15cfafd499006ac2590013562744eb81d478fd50e1b67ea8968920ad50`.
The live `script.tsv` is a byte prefix of it.

| file | what it is |
|---|---|
| `narration_units.json` | 66 coherent units in `tools/coherent_narration.py`'s schema v1, bound to the projected script |
| `director-map.json` | `tools/narration_director.py` map v1: the default policy, 73 overrides (the patched `docs/pause-overrides.json`'s 36 unchanged, plus 37 new), and 47 visual holds bound to source lines |
| `seams.json` | all 80 within-chapter seams: the decision, the source events with their lines, the tail derivations and slack, and root's preview gaps, with every input hashed |
| `projected-script.tsv` | the exact projected script bytes |
| `check_plan.py` | the read-only checker |
| `checks.txt` | its output, plus the run that failed |

The full sha256 of each file is in the handoff mail.

## Integration (root's, in order)

1. **Patches and script.** Integrate the 10/11 patch, then the 12 patch,
   as their notes say. Then run
   `python3 -I tools/storyboard.py inspection_paradox`. The new
   `script.tsv` must have sha256 `8a65ec15…20ad50`. If it does not, this
   plan does not bind: re-derive it, and do not edit the units by hand.
2. **Checker.** From the repo root, run
   `python3 -I -B explainer_corpus/inspection_paradox/docs/narration-prep-20261024/check_plan.py --script explainer_corpus/inspection_paradox/script.tsv`.
   Expect 242 checks with 0 failures and all 14 controls refused.
3. **Install.** Copy `narration_units.json` byte for byte to
   `explainer_corpus/inspection_paradox/narration_units.json`, the topics'
   convention. Then run
   `python3 -I tools/coherent_narration.py check --script … --units …`,
   which prints `{"cues": 92, "units": 66}`. Use `director-map.json` as
   the `--map` for `narration_director.py audit`. The topic's
   `docs/pause-overrides.json` stays the source of its 36 overrides.
4. **Bounded experiment, before all 66 units.** Use the existing accepted
   route, `coherent_narration.py generate` with root's recorded `--ftts`,
   `--voice`, `--model` and `--lexicon`. Its fixed options are balanced,
   in-process, verbatim, no voice cache, seed 2. Keep the prior approved
   Michael conditioning unchanged: Accepted B anchor
   `narration_lab/michael-baseline-20261017/baseline.json` (sha256
   `56318a06c4c1a0ef2119e2d5b0ff768da89a1ac082100667ae3df394e68edf60`,
   reference B `d7ac3b8b…`, transcript `0d121a2d…`). Nothing here makes a
   new voice, reference or detector. Suggested first `--unit` set, four
   requests:
   - `01u1`, the longest at 80 words, for the opening identity;
   - `12u2`, the half-of-squares and fractions;
   - `10u6`, the profiler and the millisecond grouping;
   - `10u4`, 4 words, as a separate `--following-context` call (context
     10_5, 29 words).

   Keep every output. Screen the whole text, endings and number words, and
   stop expanding a failing condition. Offer the owner one short
   comparison. Only then generate the rest.
5. **Assemble and render.** Run `coherent_narration.py assemble`. Its
   `report.json` gives the kit its contiguous cues. Then run the narrated
   native render. The kit fails closed at any contiguous seam with
   sounding edges that would need inserted samples. Watch the five
   marginal seams under Risks first.
6. **Measure, then add pauses.** Measure with FW, then run
   `narration_director.py audit --map director-map.json`. Every `range_s`
   is the total quiet heard, not an amount to add. The map's `accounting`
   block gives how much to add:
   - **inside a take:** `max(0, range_s[0] − measured quiet)` as an
     exact-sample edit before assembly, never inside a `protected_phrase`;
   - **between units:** `X = T − 0.10 − (tail_A + lead_B)`, because the kit
     re-subtracts inserted cue-end silence (`class_gap`, `MIN_GAP_S`);
   - **at a visual hold:** nothing.

   Never time-stretch.

## The units

There are 66 units of 4–83 words (median 31), at most 3 cues each.
26 seams sit inside units. Every one of them is an animation-only seam
whose picture finishes at least 0.5 s before the words, by the estimate
in `seams.json`.

Boundaries come from the source (file lines in `seams.json` and the map):

- **12 predictions**, `think`: the question before its answer;
- **13 reading holds**, `kit.hold` plus `wait_until` or `pause`;
- **8 authored windows**, `kit.music_breath`;
- **3 deliberate picture overruns:**
  - 01_4|01_5, the unlucky countdown, 4.6 s of picture after "Some
    mornings you've";
  - 05_2|05_3, the ride down the long triangle, 5.0 s;
  - **10_3|10_4**, every student asked in turn, 7.6 s against about 3.7 s
    of words;
- **11 chapter ends.**

They also come from the estimate:

- **4 overruns:** 06_2|06_3, 07_1|07_2, 10_2|10_3, 11_2|11_3;
- **9 tight seams:** 02_3|02_4, 02_5|02_6, 04_6|04_7, 05_3|05_4,
  06_1|06_2, 08_6|08_7, 10_1|10_2, 10_7|10_8, 11_6|11_7.

Five eligible seams stay boundaries by thought or size: 01_3|01_4 (the
four cues would be 111 words), 03_2|03_3 (118), 04_2|04_3, 05_4|05_5 and
12_5|12_6.

As a cross-check, root's native scratch-voice previews show every
source-proven hold as a gap over 1.05 s. No coherent-eligible seam held
or overran in them. Only the 08 and 09 previews were rendered from today's chapter
and script bytes; 04–07 match the chapter bytes only; 01–03 are unbound.

## The pauses

The 37 new overrides were authored from the words and the scene source.
Root's preview gaps set no range. They cover:

- **questions before answers.** Every question that ends a cue ends at a
  prediction ring, which is a visual hold. Three questions sit mid-cue:
  11_6 and 12_6 already had overrides, and 09_1 "What if there's no
  timetable at all?", answered at once, is new.
- **calculations.** The expression is whole before its value. This covers
  04_7, 05_5 ×2, 08_5, 09_6, 11_3, and 12_3 ×2.
- **new concepts, turns and payoffs.** 02_2 tags, 05_8 the square law,
  06_6, 08_2, 10_1, 11_2 "You sample a moment", 12_9.
- **contrasts and limits.** 02_6, 04_8, 07_2, 10_7, 12_8.
- **16 protected phrases**, with no inserted break:
  - "half of four | squared", "half of eight | squared" and "half its
    length | squared";
  - the operator kept with its denominator: "sixteen over | twice | six"
    (two boundaries), "thirty-six over | twelve", "square minutes over |
    twelve", "Forty over | twelve";
  - "three | and a third";
  - durations: "eight minutes | forty", "four minutes | twenty" ×3, and
    bare "four | twenty" ×2, "eight | forty".

No override sits at a cue end. Those boundaries belong to the 47 visual
holds, each with a kind, a reason and evidence. The evidence is the
chapter file, its sha256 and the line span of the call, so the audit
counts recorded quiet and scene delay once and flags nothing there.

## Demanding passages

Every intended reading below is unprobed and unheard. The topic lexicon
is unchanged; it stays root's or the topic's to edit.

| cue | passage | intended reading | source / open need |
|---|---|---|---|
| 10_6 | Scott Feld, Bernard Grofman | FELD; GROF-mən | **Open:** no pronunciation source found (Wikipedia and UCI pages give none). Look one up, then probe. |
| 10_9 | Torahiko Terada | toh-rah-hee-koh teh-rah-dah, even syllables, no English stress | The reading テラダ, トラヒコ is from the NDL authority record [00083550](https://id.ndl.go.jp/auth/ndlna/00083550). IPA [toɾaçiko] is from [Wikipedia: Torahiko](https://en.wikipedia.org/wiki/Torahiko). The English approximation is a hypothesis: probe, then the owner. |
| 01_2 | routes | root /ruːt/ for the English accent | **Owner choice open**, as the lexicon's note says. |
| all | minute(s) | MIN-it, the time noun | Lexicon note; listen in 12_4's "square minutes by minutes leaves minutes". |
| 05_5, 08_1, 11_3, 12_3, 12_4 | square minutes | unit, not "square" the verb | Listen. |
| 08_1, 12_3 | half its length squared; half of four/eight squared | no break before "squared" (protected) | The row on screen shows ½ × (4 min)². Listen. |
| 08_5, 09_6, 11_3, 12_4 | sixteen over twice six; thirty-six over twelve; … over twelve minutes; Forty over twelve | "over" as division, joined to its denominator | Listen. |
| 10_7 | one ten millisecond call; five two millisecond calls | "one ten-millisecond call", "five two-millisecond calls" | Risk: "one-ten", "fifty-two". **Probe** in context. ASR digits are not proof. |
| 06_7, 09_8 | four twenty; eight forty (bare) | four minutes twenty; eight minutes forty | Lexicon note. Listen for clock-time readings. |
| 02_6, 06_4, 10_7; 04_7; 10_5; 12_4 | five sixths, a sixth; four and a third; eight and two thirds; three and a third | fractions | Lexicon note. |
| 01_6, 04_7, 06_4, 11_3, 12_4 | … minutes and twenty/forty seconds | durations, "and" unstressed | Listen. |
| 06_5; 10_7 | a second way; many times a second | ordinal vs unit | Lexicon note. |
| 10_6, 10_9 | nineteen seventy-seven, nineteen twenty-two | years | Listen. |
| 06_6, 10_6 | the inspection paradox; the class size paradox | new names | A definition pause precedes 06_6's name. |

## Risks

- **Estimates.** Unit boundaries at animation seams rest on hand-read run
  times and an assumed 2.68 words/s (3.0 for a faster read). These are
  not measurements.
- **Marginal in-unit seams**, slack under 1.0 s at 2.68 or 0.7 s at 3.0:
  - 04_1|04_2, 0.74 s (0.59 s fast);
  - 06_4|06_5, 0.80 s (0.66 s);
  - 07_6|07_7, 0.85 s (0.72 s);
  - 08_1|08_2, 0.95 s (0.66 s);
  - 11_1|11_2, 0.72 s (0.55 s).

  If a real read is fast, the kit's guard will refuse a sounding seam.
  Split that unit at the seam rather than trimming picture or speech.
- **Fragmentation.** 44 of the 66 units are one cue, because the holds
  demand it. More requests mean more joins where identity or accent could
  drift. Nothing here shows that longer units would fix that.
- **Short units:** 10u4 (4 words), 06u3, 09u3, 10u3 (10 words each) and
  08u6 (12). Use following context for 10u4 at least. `--following-context`
  cannot be used for 01u1 or 02u1: with the next cue each request exceeds
  100 words.
- **Long silent pictures**, scene time not voice:
  - 08_4|08_5, 19.1 s in root's bound preview;
  - 09_4|09_5, 10.6 s;
  - 12_9's names and receipts, about 23 s in my 12 emulation.
- **Picture overruns that are not authored holds.** The audit will report
  06_2|06_3, 07_1|07_2, 10_2|10_3 and 11_2|11_3 as scene waits. They have
  no visual-hold entry, so root judges them.
- **Unregistered sources.** The projection depends on unapplied,
  uncompiled 10–12 sources. Any change to them, to 01–09's words, or to
  storyboard.py's output breaks the binding (the checker's step A).

## Controls that failed (all kept)

1. **Patch order.** Applying the 12 patch before 10/11 is refused on five
   files. This is the expected dependency-order control.
2. **Scanner.** My scanner's first pass knew no `kit.music_breath` and
   took only literal `wait_for` anchors. It would have read 03_4|03_5 and
   09_6|09_7 as coherent seams, and 06_2's anchor too early. I fixed it
   before any unit was drawn. The corrected scan agrees with every preview
   gap.
3. **Hold kinds.** Hold kinds were first named by the first call, so
   05_7, 08_2 and others read as reading holds although a music window
   follows. I relabelled them.
4. **Checker run 1** failed twice. 10_9|11_1 and 11_7|12_1 lack `stand(`
   because chapters 10 and 11 end on `kit.hold` + `wait_until`. I fixed
   the check and the hold's reason; the output is preserved in
   `checks.txt`.

## Gates not run

None of these has run:

- integrating 10/11 and 12, Cargo and the crate's tests;
- regenerating `script.tsv`;
- synthesis and FW/ASR;
- the kit's seam guard in a narrated render;
- the director audit on real audio;
- pronunciation lookups and probes;
- owner listening for identity, accent and cadence;
- the final render.

Root owns all of these, along with Beads and the catalog.
