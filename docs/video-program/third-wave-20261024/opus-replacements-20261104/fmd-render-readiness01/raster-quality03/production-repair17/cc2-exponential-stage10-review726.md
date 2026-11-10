# CC2 independent review 726: exponential expanded-stage10 (sealed)

Reviewer: CC2. Author of the reviewed work: CC1. Date: 2026-11-10 (UTC).
Scope: the sealed 726 handoff and its stage10 tree, by reading source, recomputing numbers and running CC1's
read-only checker. Source and desk evidence only.

Abbreviations: E = `narration_lab/third-wave-20261024/root/elevenlabs-exponential-20261101`,
S = `E/expanded-stage10`, Q = this folder.

## Inputs

All hashes were checked against CC1's seal at the start and again just before this file was written. The bytes
did not change during the review.

| Input | sha256 |
|---|---|
| `S/handoff726.md` | befc7e90e101d79abfb2037a66efd2b8362070f09530808e76584680e287893b |
| `S` tree, 55 files, CC1's digest method | 7c9f396a374a84b19981128dfc80fa63d5701e64979013fcf56287a5a2ea1ef4 |
| `S/native-crate/script.tsv` (94 cues) | fb4a889e9c48e02e319ae18a3df259a3a41ae27e88c2b558754ca15a8660b419 |
| `S/oracle10.tsv` | 123eef033530a2330951c6c5f1852c6d5199cec1d6021896f3e8c14749c35630 |
| `S/exact-diff726.diff` | 6a42ecf6f6751e88147bd111df7282058d871b548e139140a45ded69e1d08da3 |
| `S/direction-holds726.json` | ef7f648a00349f027923afe0fe3d67d60d1f6104b835b4c86fa24f91b14ac242 |
| `S/check726b.py` | fa2c1b52633b7375a9dd4fdfb0fe259156e56d40ba60b6d9a8009870be081d32 |
| `Q/cc2-exponential-ending-review721.md` (the review being answered) | 9a433ce8f3748cb6886e9bea233cd2ec539aa711f0ad31d1e419e6d3faf9ded3 |
| `E/pronunciation-review728/pronunciation-inventory728.md` | 55d36b3ce5bd436d3e231eddbe20d7f361986bee55429d098ab9c4f31027f628 |

## Verdict

I found no blocker in the source. Every disposition I could check in source matches what the handoff says.
The spoken and printed numbers recompute. In each of the 23 changed cues, the sentence and phrase hooks land on
the sentence that says what appears. `check726b.py` passes 277 of 277 on the sealed bytes. The 14 rows of
`direction-holds726.json` match their source lines.

Two narration wordings should be settled before takes are made: N1 (the letter "a" in 14_6) and N2 ("E of two pi
times i" in 15_3). The other findings are Low or unmeasured. Nothing here was compiled, rendered, synthesized
or heard, so every native gate is still open (see "Not established").

## What was checked, and how

1. **Seal.** I recomputed the tree digest and the file hashes above, before and after the review.
2. **Cues.** I read the full text of all 23 changed cues: 03_3, 06_2, 09_4, 10_1, 10_2, 10_3, 10_5, 11_2, 12_2,
   12_4, 12_5, 13_1, 13_3, 13_4, 13_5, 14_1, 14_2, 14_4, 14_5, 14_6, 15_1, 15_2 and 15_3.
3. **Source.** I read every production hunk of `exact-diff726.diff`. That covers `changing_rates.rs`,
   `money_limits.rs`, `turning_rate.rs`, `euler_identity.rs`, `log_bridge.rs`, `changing_rate_model.rs`,
   `turning_model.rs`, `applications_checks.rs` and `oracle.py`. I also read the 26 new tests.
4. **Hooks.** I counted each changed cue's sentences by the kit rule (`narration.rs:623`: split after `.`, `?` or
   `!` followed by a space or the end; indices start at 0). I then checked that each `until_sentence` index and
   each `until_phrase` needle reaches the words that name what is drawn. Two examples:
   - 10_5: 4 cells at s1, left-edge bars at s2 ("Every cell read its rate at its left edge"), 16 at s3, 64 and
     1024 at s4, fade at s5, peak and "not an exponential" at s6, clock line at s7, "no half-life" at s10.
   - 14_2: the phrases "sixty-four" and "one thousand twenty-four" both occur in the cue, so `until_phrase` will
     not error on them.
5. **Numbers.** I recomputed them with my own Python, separate from `oracle.py`:

   | Quantity | Value |
   |---|---|
   | Four left-edge cells, 1.5 × 1.25 × 1 × 0.75 | 1.40625 |
   | Four midpoint cells | 0.845947265625 (fact `change.midpoint.4`, 0.84595) |
   | 16 cells | 1.08704 ("near one point zero nine") |
   | 1024 cells | 1.00130 ("within about a tenth of a percent"; see N8) |
   | Five heads more, 2.5^5 | 97.65625 ("about a hundred", card "≈ 97.66") |
   | One swap of a tail for a head | 2.50 |
   | 50 ln 0.9 | -5.26803 (alternating run, "ends at E(-5.268)") |
   | 1.05^100 | 131.50 (W2) |
   | ln 1.2, ln 0.9, ln 1.08 | 0.18232, -0.10536, 0.07696 (M6) |
   | ln 10 | 2.30259 (H3) |
   | i(1.5 + i) | -1 + 1.5i (H1) |
   | P(≥ 56 heads in 100) | 13.56% ("about fourteen in a hundred") |
   | 1024-cell miss in 14_2 | 0.48% ("within half a percent") |

   The Rust fact strings and the `oracle.py` rows agree in format by reading: `point_label` prints
   `five(re) five(im)`, as the oracle does. The seven new oracle rows are accounted for in the
   `1104 + 454 + 206 + 7` assertion.
6. **Checker.** Before running `check726b.py`, I read it and confirmed it only reads files. Its subprocesses are
   the lint, `oracle.py`, `diff` and `rustfmt --check`. Result: 277 passed, 0 failed. The stage10 folder was
   unchanged afterwards.
7. **Holds.** For each of the 14 rows in `direction-holds726.json`, I checked five things:
   - the named file's sha256;
   - that the hold, `mark(.., "predict")` and `music_breath` lines sit at the stated line numbers;
   - that `next_cue` directly follows `cue` in the script;
   - that the last `say()` before the hold names the cue.

   Result: 14 of 14. The six new think times are all inside the ranges review 721 proposed:

   | Hold | Seconds | 721 range |
   |---|---|---|
   | 10_2 | 4.0 | 3 to 4 |
   | 11_2 | 4.0 | 3 to 4 |
   | 12_1 | 3.5 | 3 to 4 |
   | 12_3 | 2.5 | 2 to 3 |
   | 13_2 | 3.0 | 2 to 3 |
   | 14_1 | 2.5 | 2 to 3 |

## Dispositions checked in source

"Verified" means the source and script do what the handoff row says. It does not mean the change looks or
sounds right on screen.

| ID | Check | Result |
|---|---|---|
| H1 | 13_1 worked case, cue s15 to s17: "1.5+i", then "-1+1.5i" with `i\,(1.5+i)=-1+1.5i`, then the general rule. Facts `turn.example` and `turn.example_turned`. | Verified |
| H2 | 10_2 draws the last step to 1.40625 at s7 ("Four cells finish near one point four"), then asks at s11 and holds 4.0 s. 10_5 brings back the four left-edge bars at s2 with the reason. 10_5 no longer contradicts 10_2. | Verified |
| H3 | 15_2 s2 adds "slide rule: the same marks, a unit 2.30259 times as long". The direction is right: one base-ten unit spans ln 10 of E's units. Row 4 says "unit of E" on both sides. 12_2 labels the ruler in E's units. | Verified |
| M1 | 12_2 middle run, spread ("five heads more: ×2.5^5 ≈ 97.66"), rare runs; numbers above. | Verified |
| M2 | 12_5 table: six rows at s1, s2, s3, s5, s6 and s7. Each lands on its sentence; the two chance rows land on "For the average over many runs" and "For a typical run". | Verified |
| M3 | 13_1 quarter turn: still arrows with right-angle marks at s12 to s14; no circle, tracker or live walk. | Verified |
| M4 | 13_1 spring at s2, "r·r = r², never negative" at s3, i² = -1 at s4. | Verified |
| M5 | 10_1 pond at s3, more algae at s4, the cooling card and algae falling from 7 to 5 at s5, law swap at s6, numbers at s9. 5 is still above the starting 4, so the pond does not give away the 10_2 answer. All algae sit inside the pond: largest offset about 1.05, radius 1.75. | Verified |
| M6 | 11_2 inverse cards and "zero" for the digit; ln values above. | Verified |
| M7 | The landing ring and its name return as a pair and fade together at 14_5. | Verified |
| M8 | `this_rate` fades with the proof at the start of 10_5, before the first row lands at (3.6, 1.05). | Verified |
| M9 | Drops at ±0.12; "90" moved to (-0.45, -0.12); "108" fades at 11_3 s1. | Source verified; clearance unmeasured (N7) |
| M10 | Six named think-time constants, each between hold, `mark("predict")` and `music_breath`. The test needle string is present for all six. | Verified (see holds above) |
| M11 | 15_1 sentence plus `E'(0)=1` in row 3. | Verified |
| L1 | 12_4 card "noisy early data can hardly tell them apart". | Verified |
| L2 | Time ticks at 0.5, 1, 2, 3 and 4. `unlimited_reaches(level) = ln(level / 0.08)` inverts `unlimited(t) = 0.08·e^t`. | Verified |
| L3 | One alternating run, +0.405 and -0.511 in turn, ending at -5.268. Deterministic. | Verified |
| L4 | The 64 and 1024 rows wait for their spoken words. | Verified |
| L5 | `INWARD_OFFSET` 0.1 instead of the 0.06 I proposed, because the shafts are about 0.05 wide. The reasoning is sound. | Source verified; gap unmeasured (N7) |
| W1 to W8 | Script wordings are in place. The constants `OUTWARD`, `QUESTION` and `OPENING` and the two name cards land after the sentences that earn them (13_4 s8, 14_4 s7). | Verified |

## Findings

**N1. 14_6, the letter "a".** Severity: Low to Medium. Not a source blocker; it is a gate for audio-review.

The cue says "a real part called a, plus i times a real coefficient called w" and "E evaluated at a times t".
The risk is that the voice reads the letter as the article "uh". If so, the listener hears "called uh" and "at uh
times t", and the second can blur into "at". Inventory 728 row 2 rated this High. Stage 726 kept the letter and
changed the words around it, so row 2 is only partly closed.

Recommended order:
1. Audition 14_6 with the Frederick Surrey voice and judge both instances.
2. If either is heard as the article, change the letter in voice and screen together: `rate=a+iw`, the
   both-parts card and the kinds table. Use a letter root chooses that cannot be read as a word. If the new letter
   is Greek, rerun the glyph probe.

**N2. 15_3, "E of two pi times i".** Severity: Low to Medium. Not a source blocker.

"E of zero and E of two pi times i are the same point" can be parsed as E(2π) times i. That is the misreading the
film avoids elsewhere: 14_6 uses "E evaluated at". The screen shows `E(0)=E(2\pi i)=1`, which settles the meaning
for a viewer who is looking at it, so this is a listening-clarity risk, not an error.

Recommended: "E evaluated at two pi times i", matching 14_6. I share part of the responsibility here: my 728
row 5 asked CC1 to protect the dropped "i", and "times" is how CC1 applied it.

**N3. Still open from 728 row 3 and row 18.** Severity: Low; for audio-review.

"E of i t" in 13_2 and 13_4 can run together as "E of it", and "E of k t" in 12_5 is a milder case. The handoff
leaves these open, and I agree they belong to the take check, not to the source.

**N4. 15_1 shows "unit of E" a cue early.** Severity: Low.

Row 4 now reads "unit of E: value at x=E(x)" while 15_1 says only "A ruler that multiplies by sliding". The unit
is explained in 15_2 s1 and s2. A viewer reads a term before it is defined. It is acceptable as it stands.

Option: show the old plain row in 15_1 and swap to the unit row at 15_2 s1. Today the test
`the_rows_pin_the_exponential_and_name_the_ruler_unit` requires "unit of E" in `ROWS[4]`, so this option would
need that test changed.

**N5. 13_5 voice and card differ.** Severity: trivial; optional.

The voice says "the weight on the spring we began with" and 13_1 now says "a weight on a spring". The card
still says "ideal mass on a spring, stiffness over mass equal to one". This is an existing card, not a stage10
change, and the meaning matches.

**N6. 13_1 is 205 words in 18 sentences.** Severity: Low; for synthesis planning.

This is now one long cue. Whether it can be made as one stable take, or needs splitting for synthesis, is a
take-planning question. Splitting the cue id would change hooks and tests.

**N7. Layout clearances I could not measure.** These are for the draft frames.

| Item | Estimate |
|---|---|
| "90" against the 11_2 and 11_3 paths (M9) | not estimated |
| Inward arrow beside the position arrow (L5) | about 0.05 |
| 14_4 "Euler's identity" card at -1.62, between the 58 pt line at -1.05 and the half-length line at -2.15 | gaps of about 0.2 and 0.3, from glyph-height estimates |
| 10_2 "1.40625" label, 0.55 right of the four-cell end | not estimated |
| 15_2 slide-rule line at -2.05 against the table at -2.75 | not estimated |

None of these is a finding until frames show a contact.

**N8. 10_5, "within about a tenth of a percent".** Severity: Low. This is existing wording, unchanged in 726.

1024 cells end at 1.00130, which is 0.13% from one. "About" covers it, so no change is needed. "Within about
two tenths of a percent" would be stricter.

## Independence limit

Many of the wordings repaired here are my own proposals from review 721: H1, M1, M2, M4, M5, M6, M11, L1, L2, L3
and W1 to W8. The 728 rows that CC1 applied are also mine. My judgment that those wordings read well is not
independent, and someone other than CC2 should judge them on the draft and the takes.

The following are independent of CC1's own checker:
- the hook walk;
- the recomputed numbers;
- the holds check;
- the check that each edit matches its handoff row.

## Evidence kinds

- **Desk evidence:** source and script reading; my own float recomputations; CC1's `check726b.py` run on sealed
  bytes. That checker runs the lint, `oracle.py`, `diff` and `rustfmt --check`. It does not compile anything.
- **Native evidence:** none.

## Not established

- Compilation, strict Clippy and the 217 authored tests.
- The native facts compared against `oracle10.tsv` at runtime. The rows agree by reading only.
- The glyph probe for the new strings: the G1 rerun, and O1.
- The layout lint, draft frames and contact sheets.
- Motion, takes, pronunciation acceptance, listening, timing or sync, and AV.
- The think times against Frederick Surrey takes.

This review accepts no pronunciation and makes no release claim.

## Recommended next steps for root

1. Treat stage10 as source-ready for the native build, strict Clippy and the 217 tests. Its const-only asserts are
   written as `const { assert!(..) };`. I have not confirmed that strict Clippy accepts that form on this toolchain.
2. Before takes:
   - decide N2, a one-phrase script change;
   - plan the N1 audition, with the letter swap as the fallback;
   - plan N3 in the same take session.
3. On the first draft, read frames at the N7 items and at 10_1 s3 to s9 (the pond).
