# logarithm-narration-clarity-review01: review of WhiteSwan's clarity repair (04_4, 08_5) against stage08

LavenderPike, 2026-11-05, for root (BlueSalmon) under `mx-video-program-program-87do.63`. Copied to WhiteSwan.

**Reviewed:** `logarithm-narration-clarity-repair01`. The bundle is sealed with SHA256SUMS `e9fea0ed…` (7 files, verified). Its patch is `2463334a…`, against `EL/combined-repair-stage08/candidate`. I also read WhiteSwan's Mail 3710, which corrects the bundle's voice prose and is not inside the seal.

**Short answer:**
- Both rewordings are mathematically correct.
- Both are less ambiguous than the current words, and natural for the retained British Oliver voice.
- Both keep every native sentence hook and phrase hook.
- The patch is minimal and applies cleanly to stage08.

The defects are around the patch, not in it. Two need action before root installs stage09:
- the voice premise in the sealed prose;
- an unnamed binding, the phrase-timings template, which can fail silently.

**What this review did:**
- It read source text, hashes and root's machine-recognition records.
- It compiled, rendered, synthesized and measured nothing, and decoded no audio. The four Oliver takes were only hashed.
- No one has heard any take for this review.
- Recognition output is machine text and is not listening.
- The edit to the Rust test is uncompiled.

`EL` = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104`, and `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`.

## Voice

The owner refined the voice direction at 2026-11-05T03:22:06Z. It is line 1 of `docs/video-program/NARRATION-CONSISTENCY.md`:
- **Logarithm keeps ElevenLabs Oliver.** 04_4 and 08_5 get targeted Oliver replacements once the reviewed patch is integrated.
- The whole film is not regenerated in Frederick. Every old take is preserved.

This review judges the wording for Oliver. Every receipt in `ui-takes01` names him: "Oliver" or "Oliver - Clean, British and Steady", with Eleven v4, stability 0.5, similarity 0.75 and clone strength 0.

## Verdict on the two sentences

| Cue | Sentence | Stage08 | Proposed | Verdict |
|---|---|---|---|---|
| 04_4 | s5 | …which is why **this base** is called natural. | …which is why **base e** is called natural. | Accept |
| 08_5 | s4 | **Halving** the lost lap leaves a half turn, and a half turn is a factor of minus one. | **Taking half of** the lost lap leaves a half turn, and a half turn is a factor of minus one. | Accept |

### Mathematics

All of the following hold. The float checks are in `checks/clarity_review.py`, section "meaning".

**04_4**
- d/dx ln x = 1/x, which is 1 at x = 1.
- ln 1.01 = 0.00995033…, within 0.5% of 0.01.
- So a 1% step in x is about a 0.01 step in ln x.
- The patch does not change the claim "which is why … is called natural". It only names the base the claim was already about.
- "Base" as a word for logarithms is introduced in 01_6 ("the logarithm base two … a different base"). 04_5 follows at once with "Other bases are the same ruler stretched." So "base e" uses a term the film has already set up.

**08_5**
- √z = e^{½ Log z}, and Log(−1) = πi, so each principal root of −1 is i and their product is e^{πi} = −1.
- Log 1 = 0, so the root of the product is 1.
- The lost lap is (πi + πi) − 0 = 2πi. Half of it is πi, and e^{πi} = −1, the factor between the two sides.
- "Taking half of the lost lap leaves a half turn" can also be heard as "after removing half". That reading still gives π, so neither reading is false.
- The picture fixes the intended reading. At `until_sentence(stage, "08_5", 4)` (`product_rule.rs:431`), the purple ½(2πi) = πi fades in (`HALVED_LAP`, lines 433–435) before either phrase.

### Natural wording for Oliver

**04_4: "why base e is called natural".**
- "This base" runs the /s/ of "this" into /b/, and that is nearly "this space". The new word before "base" is "why", which ends in a vowel.
- The voice now says the words of the card on screen at that sentence: "that is why base e is called natural" (`undoing_growth.rs:551–557`, shown at `until_sentence(stage, "04_4", 5)`, line 550). Before, the voice said "this base" while the card said "base e".
- A lone spoken "e" already occurs in 13 cues, including 04_1, 04_3 ("The natural logarithm of e is one", the same "e is" sequence), 04_5 and 08_5.
- In 08_5's existing Oliver takes, both recognizers wrote "e to the power" as "e".
- No recognition transcript of 04_4 by large-v3-turbo exists in the lane. `middle-recognition01` lists the takes but holds no transcripts.

**08_5: "Taking half of".**
- The phrase uses the cue's own "half a logarithm" and the on-screen ½.
- "Half" ends in /f/, which "have" lacks.
- The sentence has three "half"s ("Taking half of … a half turn … a half turn"). That is repetitive on the page, but it is the idea being taught, and it reads naturally aloud.
- I found no better wording that keeps both `HALF_TURN_PHRASE` ("leaves a half turn") and `FACTOR_PHRASE` ("a factor of minus one") verbatim.

**After synthesis.** Root should check the new takes' transcripts at "base e is called" and "Taking half of", and review the joins listed under Timing below.

### Hooks

These are recomputed in `checks/clarity_review.py` from stage08 and from the patched copy.

| Cue | Span (say → next say or finish) | Hooks | After the patch |
|---|---|---|---|
| 04_4 | `undoing_growth.rs` 458 → 562 | `until_sentence` 1–5 at 522, 531, 534, 540, 550; no phrase hooks, no anchor | 6 kit sentences; only s5 changes; the card stays the same |
| 08_5 | `product_rule.rs` 349 → `kit.finish` 468 | `until_sentence` 1–5 at 352, 362, 408, 431, 458; `until_phrase` HALF_TURN at 438, FACTOR at 456 | 6 kit sentences = 6 `phrase_seconds` opens; only s4 changes; both phrases once each, starting a word, in s4, in order, before "The root of a product" |

More detail:
- **Rust test.** I mirrored `the_half_turn_and_its_sign_wait_for_their_words_and_the_whole_line_stays`, with the opener read from each copy's own test source. It passes on old text with the old opener and on new text with the new opener. Crossed, it panics, so the edit to the test is required.
- **Scene order.** The test's 11 `found(...)` calls, read from its source, are present and in order in both copies' scene source.
- **Changed lines.** `product_rule.rs` changes only lines 432 (a comment inside 08_5's span), 722 and 723 (the test module starts at 611). No line moves.
- **Hold intervals.** The 08_1, 08_4 and 04_6 intervals, recomputed with binder03's formula, are unchanged.
- **Gap classes.** The inputs to `classify_gap`, namely the end mark of the previous line and the first word of the next, are unchanged at 04_3→04_4, 04_4→04_5, 08_4→08_5 and 08_5→09_1.
- **Anchor bytes.** `HALF_TURN_PHRASE` moves 260 → 267 (word 51 → 53). `FACTOR_PHRASE` moves 299 → 306 (word 60 → 62).
- **Estimates until marks exist.** Until marks are measured, the character-share estimates move: HALF_TURN 0.256 → 0.315 of s4, and FACTOR 0.732 → 0.753. Measured marks at 267 and 306 replace both.

## Findings

### R1, medium: the sealed bundle's voice premise is superseded for logarithm

**What the sealed bundle says:**
- `HANDOFF.md` says "Every cue of this film needs a Frederick take anyway, so this rewording adds no synthesis."
- `binding-inventory.json` has `voice`: "ElevenLabs Frederick Surrey for new production".
- `map_rebind.naming` says the oliver names are "root's call".

**What holds now:**
- The patch costs exactly **two targeted Oliver takes**, made with the retained settings.
- The **78 other Oliver pairs remain production input**.
- The oliver names are accurate again.

**What still stands:**
- WhiteSwan's Mail 3710 corrects all three points without touching the seal, and the patch, the hashes and the bindings are unaffected.
- I checked independently that 78 of the 80 `ui-takes01` request receipts still match the patched text byte for byte. Only 04_4 and 08_5 drop.
- No receipt has a context field: none has previous or next text, request stitching or a prior request.
- No other cue's receipt carries 04_4's or 08_5's changed words.
- So no neighbouring take is invalidated.

**Action:** root should treat Mail 3710 as part of the handoff, or ask for `…-repair02` with a byte-identical patch.

### R2, medium: the phrase-timings template is not in the inventory, and a stale byte fails silently

**The binding.** `EL/combined-repair-stage07/bound-plan03/phrase-timings-unmeasured-template.json` binds 04_4 and 08_5 to the old text hashes, and puts 08_5's marks at bytes **260 and 299**. The repair inventory never names this file.

**How the kit loads it** (`narration.rs:921–945`):
- It rejects a stale `text_sha256`.
- It accepts any mark byte that is in range, on a character boundary, and increasing.

**The silent failure.** In the new text, bytes 260 and 299 fall at "st lap leaves " and "urn is a facto".
- A template refreshed by hash but not by byte therefore loads cleanly.
- `phrase_seconds` (`product_rule.rs:556–560`) then looks up `text.find(phrase)` = 267 and 306, finds no mark, and falls back to the character-share estimate.
- Nothing reports this.

**Action.** Root's Mail 3724 already asks WhiteSwan to rebind the template to 267/306. The filled `phrase-timings.json` for the new 08_5 take must use the same bytes. `checks/rebind_acceptance.py --template` rejects 260/299; selftest case t1 shows this.

### R3, medium-low: the rebind route and the request plan are unnamed

**binder03 cannot rebind the patched bytes.** Its `STAGE` is stage07 (`bind_final80-stage07-r3.py:61–62`), and it pins script `b89eab60` (line 70) and `product_rule.rs` `f2c8684a` (line 255). It exits 3 when an input moves. The bundle's "rebind items 3–7" does not say how; root's Mail 3724 asks for a new helper.

**`cue-delta.json` is unnamed.** `bound-plan03/cue-delta.json`, the bound request plan, still maps 04_4 and 08_5 to `narration-request-final80-01/{04_4,08_5}.txt` with the old hashes. The bundle names only the request list. The targeted Oliver requests need new text files at new paths; the old files are preserved.

**Records of the past bind.** `stage07-delta.json`, `increment-delta.json` and `inputs.sha256` in bound-plan03 also hold the old hashes. They record the stage06→stage07 bind and need no rebind.

### R4, low: the repair's word_index formula is not binder03's

- WhiteSwan's `source_checks.py` computes `word_index` as a whitespace count.
- binder03 computes it as `len(coherent.direction_words(...))`, using `tools/coherent_narration.py:52`. That rule counts "sixty-six" as two words.
- The two rules agree for 08_5, so the inventory's 53 and 62 are right.
- They disagree for 11_20, which is unaffected by the patch: `BLOCKS` 15 vs 16, and `MATCHING` 22 vs 23.
- A helper that recomputes all 33 anchors with the whitespace rule would write two wrong indices.
- 10_3's two anchors have `const: null`, so anchors must be keyed by (cue, phrase).
- I sent both points early, as Mail 3744. `rebind_acceptance.py` uses binder03's rule, and selftest case t11 rejects the whitespace indices.

### R5, low: the recognition evidence is stronger for 08_5 than the bundle cites, and context-dependent for "halving"

**08_5 is stronger than cited.** root's `remaining-recognition01` (whisper large-v3-turbo) also wrote "Having" for "Halving" in both 08_5 takes. That makes **4 of 4 transcripts across two models**.

**But "halving" depends on context.** 01_2's sentence-initial "Halving must undo a doubling" was written "Halving" by large-v3-turbo in both retained Oliver takes (`opening-recognition01`). Two related words also came through correctly: 07_5's "halves" and 11_3's "Halve". So the bundle's line "both confusions sit in the words, not in one voice" is too strong for "halving".

**This does not weaken the 08_5 change.** I propose nothing for 01_2. Its take is retained production input, and its machine transcripts show no sign of trouble.

**04_4's evidence is small.en only** (2 of 2, "this space").

### R6, low: cosmetic leftovers, as the bundle states

- `docs/storyboard.md:141` still describes "halving the lost lap leaves a factor of −1".
- The test's `expect("08_5 has the halving")` message is unchanged.

Both are descriptions, not narration, and neither is bound. Aligning them is optional.

## Timing and joins under Oliver retention

**Fresh measurements.** Fresh sentence onsets are needed for 04_4 and 08_5 only, plus 08_5's two marks at 267 and 306. The kit refuses a stale `report.json` loudly: `narration.rs:833–856` checks the sentence count and text against `script.tsv`.

**Chapter timing.**
- 04_4 has a `say_breath` boundary. If the new take's length differs, the start times of 04_5–04_7 within chapter 04 shift by that difference. Their takes are untouched.
- 08_5 ends chapter 08 (`chapter_end`, `kit.finish` at line 468), so nothing after it in that chapter moves.

**Joins.** The joins 04_3→04_4→04_5 and 08_4→08_5→09_1 put a new take beside retained ones. Their transcripts and their sound are root's to review. I claim nothing about them.

## Runnable checks

All of these run from the repo root and are read only, except for writing to a fresh scratch folder.

```
python3 -I -B P/logarithm-narration-clarity-review01/checks/clarity_review.py FRESH_SCRATCH
python3 -I -B P/logarithm-narration-clarity-review01/checks/rebind_acceptance.py CRATE [--anchors F] [--template F] [--hold-audit F] [--receipt F]...
python3 -I -B P/logarithm-narration-clarity-review01/checks/rebind_acceptance_selftest.py FRESH_SCRATCH
python3 -I -B P/logarithm-narration-clarity-repair01/checks/source_checks.py EL FRESH_SCRATCH
```

**1. `clarity_review.py`** gives RESULT PASS (`checks/outputs/clarity_review.txt`). Exit 3 means one of its 28 pinned inputs has moved. It checks:
- the baseline bindings above;
- the replay with `git apply`;
- the candidate's `docs/check_cues.py` on both copies;
- the meaning, hook, interval and mathematics checks;
- the Oliver-retention receipt checks;
- the recognition tally;
- the inventory scan that names R2's and R3's files.

**2. `rebind_acceptance.py`** is for WhiteSwan's `logarithm-clarity-integration01` candidate and for root's stage09. It needs no pin change after installation. It passes only when all of the following hold:
- **Files:** the crate is stage08 except `script.tsv` `559afa36`, `product_rule.rs` `bc9fbe1d`, the units, and the map the units name (which may be new).
- **Script and units:** the 78 other rows and unit texts are stage08's bytes.
- **Map:** it equals oliver15 except the four `product_rule.rs` hash fields, with every hold's file hash current and its interval recomputed. Changes to `basis`/`reason` wording are reported, not failed.
- **Anchors:** all 33 are recomputed with binder03's rules.
- **Template:** one entry per cue, with marks exactly at the anchor bytes.
- **Hold audit:** the words come to 4594.
- **Receipts:** the new texts, with Oliver / Eleven v4 / 0.5 / 0.75 / 0 / 48 kHz. The four original takes are intact.

**3. `rebind_acceptance_selftest.py`** gives 12/12 (`checks/outputs/rebind_acceptance_selftest.txt`). It builds a plain positive control: stage08 plus the patch plus field substitutions. That control is a test fixture, not a proposed integration. The control passes, and each of these planted defects fails:

| Case | Planted defect |
|---|---|
| t1 | template marks at 260/299 |
| t2 | map keeps `f2c8684a` |
| t3 | 04_5 reworded |
| t4 | units keep the old 08_5 text |
| t5 | anchor at 260 |
| t6 | unpatched crate |
| t7 | patched but not rebound |
| t8 | stray `.orig` |
| t9 | receipt voice "Frederick Surrey" |
| t10 | hold audit keeps the old source hash |
| t11 | 11_20 whitespace `word_index` |

**4. WhiteSwan's `source_checks.py`**, rerun by me: exit 0 and RESULT PASS. The output is byte-identical to the shipped `source-checks.out` (`checks/outputs/repair01-source-checks-rerun.txt`). That run includes their rustfmt check (nightly-2026-08-31, width 120) on both copies.

## Integration list for root and WhiteSwan

1. Apply `2463334a` to a new stage. Then rebind:
   - **units:** 2 texts, both script hashes, and the contract map;
   - **map:** `scripts` and the four `product_rule.rs` fields; naming the new map is root's call;
   - **anchors:** the script hash, and the two 08_5 rows' bytes, word indices and text and source hashes;
   - **template:** the script hash, the 04_4 and 08_5 text hashes, and 08_5's marks at 267/306;
   - **hold audit:** the script hash, the 08_1 and 08_4 source hashes, and words 4594 (08_5: 83);
   - **cue-delta and request plan:** new request texts at new paths.
2. Run `rebind_acceptance.py` on the result with all four optional files.
3. **Synthesis is root's.** Make one targeted Oliver request each for 04_4 and 08_5, with the retained settings, saved at new paths. Keep the four originals (`ui-takes01/04_4-download0{1,2}.wav`, `08_5-download0{1,2}.wav`). Re-run `rebind_acceptance.py` with `--receipt`.
4. Measure fresh onsets for both cues and the 08_5 marks at 267/306. Run recognition and review the transcripts at the changed words and at the joins. Listening and fine-sync acceptance stay separate and are root's.
5. In the guarded native run, the edited test must pass.

## Independence and limits

**My relation to these bindings.** I wrote binder02, which BronzeJaguar revised as binder03. binder03's plan files (anchors, hold audit, template, map) are the bindings checked here. I did not write the patch or the wording, and I edited nothing of WhiteSwan's.

**What my checks share with WhiteSwan's.** My checks are written separately, but they must use the same kit rules, so a shared misreading of those rules would not be caught:
- `sentences()`;
- the `phrase_seconds` opens;
- binder03's interval and word rules.

**What was not done.** No build, render, synthesis, measurement or audio decode. No Beads, TODO or commit. Nothing outside this folder was written, except scratch and Agent Mail.
