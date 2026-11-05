# Review: the L1 mark-interval fix and the plan-binding delta (87do.63)

BronzeJaguar, 2026-11-05. This answers root's Mail 4079, and Mail 4094 for the delta. `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`.

## Verdict

**Integrate.** The combined staged helper `08a26927…` is the L1 fix `2f080ab7…` plus the binding delta. It implements root's rule (Mail 4018) the way the native kit and films read it. I found no High or Medium defect.
- **F1 (Low, latent).** Nothing needs to change for stage09.
- **N1 (nit).** One wording slip in the delta's line count.
- **Integration.** Both staged helpers still set `HERE` to the sealed stage09 folder. Root must point it at the new helper's folder, as Lavender's integration note 2 says (R11).

## What I reviewed

| Item | Pins |
|---|---|
| L1 fix, sealed `60f5f63a…` | patch `1265a1be…`; staged `2f080ab7…`; base `c6c854ce…` from stage09 `2bb8e280…` |
| Binding delta, sealed `5d2523af…` | patch `637788fb…`; staged `08a26927…` |
| Native sources | `narration.rs` `8072720d…`; `coherent_narration.py` `95615cfa…`; the six candidate chapter files that place a byte in its sentence |
| Real data | `script.tsv` `559afa36…` (80 cues); `phrase-anchors.json` `557d74a0…` (33 anchors); the stage09 report's 162 recordings; my timing inspection `8c18a243…` |

I wrote neither fix. My checks load the helpers in process and compare them with a Python mirror of the native rule. Each mirrored Rust or Python line must still be present in its pinned source, or the check stops with exit 3.

## The native rule, as checked

The kit has no interval check of its own; the interval is root's rule. Everything it is built from is native:

- **Sentence split.** The kit's `sentences()` splits on bytes, after `.`, `?` or `!` followed by a space or the end.
- **Which sentence holds a byte.** The films (`phrase_seconds` in five chapters, `sentence_offsets` in `doing_nothing.rs`) put byte b in the last sentence whose opening is ≤ b. The openings are 0, then i + 2 after each terminator-plus-space.
- **Onsets.** `report_starts` gives each onset as the running sum of `out_samples + gap_after_samples`, divided by `meta.rate`.
- **The end.** The loader refuses a mark at or past the line's WAV length.

So a mark at byte b belongs to sentence j, and must satisfy start(j) ≤ mark < start(j + 1). For the last sentence the upper bound is the length. All values are seconds, computed as samples / 24000.

## L1 results

`checks/review_checks.py`: 11 pass, 0 fail, 1 finding. The two runs are byte-equal.

| Check | Result |
|---|---|
| R01 | All three seals verify with no unlisted file. 7 pins hold, and 25 mirrored lines are found in 8 source files |
| R02 | `difflib(base, staged)` equals `prepare_audio.patch` byte for byte (+41/−4). The hunks are in `sentence_opens`/`mark_outside_sentence`/`resolved_points`, `cmd_prepare` and `cmd_timings` only |
| R03 | 80 cues, 364 sentences. The film openings, the staged `sentence_opens` and the base `kit_sentences` (converted to bytes) are equal, and the kit, film and assembler sentence counts agree. No cue has a non-ASCII character |
| R04 | All 33 anchors sit at their phrase, and the staged sentence equals the native sentence for each. None is on a sentence's first word |
| R05 | **All 162 real recordings, in three views:** recognizer values (570 onsets, 66 marks); the inspection's 181 `leave_24k` sample answers; and the same with 08_5 dl02 s2 = 167040. The oracle also re-checks every mark. No refusal, and the staged output equals the base's. Recognizer view: the smallest lead over the onset is 1.759999 s (11_7 b174, download02) and the smallest room before the next bound is 1.560001 s (05_20 b276, download02). With the answers, the smallest lead is 1.59 s (12_21 b192, download01). These agree with Lavender's M04/M10 |
| R06 | **20,000 seeded random texts**, with multi-byte words, `?` and `!`, `e.g.` and `3.14`, a double space, tab or no-break space after a split, and an unterminated last sentence. Marks sit at each edge, one sample either side of it, at the length, or at random. The staged helper disagrees with the native rule in 0. In 1,556 of these texts, comparing the byte with character openings would pick another sentence |
| R07 | The R06 differential catches four mutants: character openings (1,030 disagreements), upper edge included (3,210), lower edge excluded (2,233), last sentence unbounded (1,915) |
| R08 | **Real 08_5 download01**, both marks in sentence 4 of 0–5:<br>• s4 moved 0.1 s after b267 → refuses b267 only; omitting b267 then passes<br>• s4 moved after both marks → refuses b267+b306 in one message<br>• b306 at s5's onset → refused; one sample before → accepted<br>• b267 at s4's onset → accepted; one sample before → refused<br>• a `use_candidate` mark inside sentence 5 → refused<br>• an entry one sentence short of the text → both marks refused |
| R09 | **Synthetic text `ππππ x. Ab cd. Last words here.`** The mark at byte 9 (character 5) stays in sentence 0, where a character rule would put it in sentence 1. In the last sentence, the last sample is accepted and the length is refused. In sentence 0, 0.0 s is accepted and sentence 1's onset is refused |
| R10 | A first-word mark at its onset's recognizer time 14.72 or 14.720001 s is accepted. At 14.719999 s it is refused: this is F1 |
| R11 | Static check of `cmd_timings`. Its onsets follow `report_starts`, its last bound is the WAV length (the kit's `lengths`), and it checks every plan mark. `prepare` writes all-zero gaps, so both steps see the same onsets. `HERE` still names the base folder |

## Findings

**F1 (Low, latent).** A mark on the word that opens its sentence can be refused by 1 µs.
- The onset is compared as its rounded sample: 14.719999 s becomes sample 353280, which is 14.72 s. The mark keeps its unrounded recognizer seconds.
- The kit itself never compares the two.
- **Exposure:** 31 of 570 real onsets and 17 of 66 real marks have a sample after their recognizer time. No real anchor is on a first word (R04), so stage09 cannot hit this.
- **If it happens:** the refusal names the mark, and a sample answer at the onset resolves it.
- **Recommendation:** no change now. If a later stage binds an anchor on a first word, root can answer it, or compare marks at sample resolution.

**N1 (nit).** The delta HANDOFF line 21 says +7/−3. The patch is +8/−3, and its own P02 row says +1/−1 and +7/−2. This affects only the wording. My first delta run failed B02 only because I had copied +7/−3 as the expectation; I corrected the expectation, not the helper.

## Binding delta results

`checks/binding_delta_checks.py`: 5 pass, 0 fail. The two runs into fresh scratch folders are byte-equal.

| Check | Result |
|---|---|
| B01 | Both seals verify with no unlisted file; 6 pins hold |
| B02 | `diff(L1 staged, combined)` equals the delta patch, with hunks in `cmd_prepare` and `cmd_timings` only. `kit_sentences`, `sentence_opens`, `mark_outside_sentence`, `resolved_points` and `kit_mirror` are byte-identical, so R03–R10 hold for the combined helper |
| B03 | On all 162 recordings, in the recognizer and `leave_24k` views, the combined `resolved_points` equals L1's in 324 of 324 |
| B04 | `prepare` writes `phrase-marks-plan.json` before the receipt, and the receipt hashes that file. Timings output still pins the receipt by `prepare_receipt_sha256` |
| B05 | **`cmd_timings` in process on fixture folders.** I ran each fixture through both helpers and compared the first refusal (table below). Neither helper wrote or changed a file in any fixture |

B05, first refusal in each fixture:

| Fixture | L1 helper | Combined helper |
|---|---|---|
| A good plan | the workspace-copy check | the same: it passed the plan check |
| The plan missing | `FileNotFoundError` | "missing" |
| A receipt without the plan key | proceeds | refused |
| A plan mark moved 0.05 s within its sentence | proceeds | refused |
| A wrong binding with the plan missing | `FileNotFoundError` | the binding refusal, which still comes first |
| A changed spans file with the plan missing | `FileNotFoundError` | the spans refusal, which still comes first |

**Remaining limit, as Lavender notes:** a receipt and plan rewritten together pass the binding. L1 still refuses any mark that leaves its sentence. That is enough to catch an accidental change, but not a deliberate one.

## For root when integrating

1. Build the new helper from `08a26927…`, or from the base plus both patches, in a new folder, and set `HERE` to that folder. Otherwise the printed `then` timings command runs the sealed base, which has neither check.
2. Run prepare and timings from the same new helper. A receipt from an older helper is refused (B05).
3. The checks agree because `prepare` writes zero gaps and no intra-sentence pauses (R11). If a later helper adds authored gaps, the marks after each gap must move by it. The timings check would catch only an unmoved mark that leaves its sentence.

## Limits

- No Rust compiled; the mirrored source lines are asserted present instead.
- No production decode, assembly, timing, selection or recognition, and nothing heard.
- The fixture answers are labelled "review fixture" and are not answers. The `leave_24k` values are my own energy estimates (`8c18a243…`), not heard boundaries.
- The B05 fixtures hold no audio and stop at the workspace-copy check, so `verify` and the writing of output are not exercised there. Lavender's P03 and M09 run those steps.

## Files and replay

| File | What it is |
|---|---|
| `REVIEW.md` | This note |
| `checks/review_checks.py` | R01–R11; read-only, writes nothing |
| `checks/binding_delta_checks.py` | B01–B05; read-only on the repository, writes fixtures only under a new scratch folder |
| `checks/outputs/review_checks-01.txt`, `checks/outputs/binding_delta_checks-01.txt` | Transcripts at seal |

From the repository root:

```
python3 -I -B P/logarithm-stage09-mark-interval-review01/checks/review_checks.py
python3 -I -B P/logarithm-stage09-mark-interval-review01/checks/binding_delta_checks.py NEW_SCRATCH
```
