# complex-frederick-remaining-cues-review01

WhiteSwan, for root Mail 4481. A read-only pre-synthesis audit of the 50 complex cues with no Frederick downloads. Frozen once sealed.

**What was not done.**
- No audio was generated, heard, decoded or transcribed.
- No job ran and nothing was submitted.
- No script, request, units, map, chapter source, Bead or TODO file was changed.
- Nothing here is a claim about pronunciation or delivery.

**What was read.**
- **The stage05 crate:** script `12442480`, units `e776c860`, map `70888624`, chapter sources `a8e92ca2` and `facts.rs`.
- **Root's oracle:** `oracle.py`, run once read-only, as `facts-check` runs it.
- **The requests:** the 50 `request01.json` files, and 09_5's two receipts.
- **For earlier content questions:** my sealed timing review (`42f34a75`).

## Verdict

- **The 50 requests are prepared exactly.** Each carries the stage05 script's text, which equals the units text. Each has the right text and script hashes, Frederick Surrey on Eleven v4 at 0.5/0.75/0 with WAV 48 kHz, and `submitted: false`. All 105 requests in the folder name Frederick; none names Oliver.
- **47 of the 50 cues need nothing.** Their words, their mathematics, their hooks and their holds agree with the source.
- **Three cues have wording I would fix before they are generated:**
  - 05_8 and 15_2 (Medium): each states something the film itself contradicts in the next-door words or chapter.
  - 10_3 (Low).
  - The proposals keep every hook phrase once and keep each cue's sentence count.
- **Next production action:**
  - Submit the 47 unaffected cue pairs as prepared.
  - Decide 05_8, 15_2 and 10_3 first, because any wording change moves the script hash (see "Cost").

## Findings

| # | Severity | Cue | The words now | The problem | Proposed (one exact span replaced) |
|---|---|---|---|---|---|
| F1 | Medium | 05_8 s0–s1 | "No zero found so far sits off that line. Whether all of them lie on it is the Riemann hypothesis" | 05_7, the cue before, ends: "Zeros at minus two, minus four, and on. And more zeros climbing the line…". The zeros at −2 and −4 have been found, and they sit off the line Re s = 1/2 (ζ(−2) = ζ(−4) = 0; checked from the Bernoulli numbers). So the first sentence is false as said, and "all of them" includes zeros the hypothesis is not about. The picture rings only the four climbing zeros (`gallery.rs`: `let rings: Vec<Mob> = zeta_zeros()`). | "Apart from those at minus two, minus four, and on, no zero found so far sits off that line. Whether all the rest lie on it is the Riemann hypothesis" (+10 words) |
| F2 | Medium | 15_2 s2 | "On an open region, that makes every small piece of the map a stretch and a turn." | 07_1 and 07_2 teach the exception: "Where the derivative is zero, the picture changes… The angle has doubled", and "A point where the derivative is zero is a critical point, and there angles get multiplied." The recap drops it, so near a critical point it says what chapter 07 showed false. The panel caption is "on an open region: / stretch and turn" (`closing.rs` `CAPTIONS[0]`). | "On an open region, away from critical points, that makes every small piece of the map a stretch and a turn." (+4 words; "critical point" is 07_2's own term). The caption is root's call. |
| F3 | Low | 10_3 s3 | "so its power series misses the curve completely." | The series is 0, so it matches the curve on the whole negative half line, which the cue says itself one sentence earlier ("It agrees with zero on a whole half line"). The picture draws the zero recipe only from x = 0 to the right, with gap bars at x = 0.5 … 3.5 (`rigidity.rs`). | "so its power series misses the whole rise." (0 words; "rises" is the cue's own word) |
| F4 | Low | 09_8, 14_3 | "Recentre the recipe" and "Re-centre the series" | The same word has two spellings in the script. On screen, 14_3 shows "re-centre" and 09_8 shows "recipe centred at 1". This matters only where the script becomes captions. | Use one spelling if the script is reopened anyway. |
| F5 | Low | 14_1 s3 | "Of course not. We know it everywhere." | The same cue's card says "the bump: defined everywhere but ±i", and 09_5 and 09_7 have said the bump is not holomorphic at ±i. | Optional, only if the script is reopened: "We know it everywhere except at i and minus i." |

## Cost of any wording change

The script hash `12442480` is pinned by:
- all 105 requests (`script_sha256`);
- `narration_units.json`;
- both copies of the direction map. 05_8 also has a reading-window override whose `after` text is the cue's exact words, so F1 must change that too;
- recognition plans 01–10 (`frozen-inputs.json`) and the targeted small-model jobs' inputs;
- the intake and bridge pins, and the repository checkers built on them.

So adopting F1–F3 is a rebinding stage, not a text edit. No proposal changes any other cue's text. The check proves each proposal replaces one exact span, keeps every anchor phrase exactly once and keeps the sentence count, so no hook moves to another sentence (f2).

## Checked and clean

- **Registry (q1).** The 50 cues are 05 (2), 06 (2), 09 (5), 10 (6), 11 (8), 12 (4), 13 (11), 14 (8) and 15 (4). Each has exactly one `say()` block, in its chapter's registered module. None of the bridge's 93 anchors has a hook in the seven chapter files `mod.rs` does not declare.
- **Requests (q2).** As in the verdict. Stage05 differs from the frozen canonical script only in 11_7 and 15_2, and the requests carry stage05's words.
- **Hooks and holds (k1).**
  - 40 anchor phrases in 22 cues. The check's own scan of each scene block finds the same 40, and each occurs once in its cue.
  - One helper site sits below 11_8's `say()` but is 11_4's words ("There the count"); the bridge already binds it to 11_4.
  - 14 holds, each placed where its block calls `predict()` or `reading_window()` and each matching the map and `mod.rs`:
    - prediction: 06_8 5.0 s, 09_8 6.0, 10_4 5.0, 11_5 6.0, 13_5 5.0, 14_4 5.0, 15_3 6.0;
    - reading: 05_8 3.5, 09_5 3.0, 10_5 8.5, 11_6 7.0, 11_8 7.5, 13_11 5.0, 14_3 5.5.
- **Mathematics (m1).** 62 spoken numbers and claims in 39 cues were recomputed with Python's `cmath`, `math` and `fractions`, sharing no code with root's numpy and scipy oracle. Every one holds, and the words checked are each in their cue. Among them:
  - the mean value 0 and the −½ pole shares (09_5);
  - coefficients from turned averages (09_6);
  - radius 1, √2, and the interval (−0.41, 2.41), with the 60-term check giving 0.200000 and 212 (09_9);
  - sin²(3i) and cos²(3i) of −100.36 and 101.36, summing to 1 (10_2);
  - the turn counts 1, 2, −1; −1; 1 and 0 (11_1–11_3);
  - one root of z⁵ − 3z + 1 inside the unit circle, and |z⁵ + 1| ≤ 2 < 3 (11_6);
  - 5 turns on |z| = 3, with 243 > 10 (11_7);
  - the z⁸ check: 8 steps read 0 turns with reach 6.28, and 64 steps give a certified 8 (11_8);
  - the residues (12_1–12_4);
  - π, and the arc bounds 2.094, 0.654 and 0.317 (13_1–13_3);
  - the 13_4 card's antiderivative, which differentiates back to 1/(1+x⁴);
  - −i√2/4 and π/√2 ≈ 2.22, with 0.0031 < 1/300 and the numerical integral 2.2214 (13_5–13_11);
  - the geometric and doubling sums, ζ(−1) = −1/12, √z carried around giving −1 and the logarithm gaining 2πi, the lacunary series growing toward roots of unity, and Γ(z) = Γ(z+1)/z with its poles (14_x);
  - √10 ≈ 3.16 (15_4).

  40 of these values carry `facts.rs` keys (48 keys in all). Root's oracle prints the same value for every one, and these chapters' scenes read 47 of the 48 keys; the exception is `integral.quartic.value`, whose spoken rounding `value2` (2.22) is the key the scene shows.

  The other 11 cues make no number-bearing claim and were read against their scenes: 05_8 (F1), 05_9, 06_8, 09_8, 10_1, 10_5, 14_1 (F5), 14_4, 15_1, 15_2 (F2) and 15_3.

## Spoken notation

No pronunciation claims here; these notes are about the words as written.

- **Characters.** No digit or symbol appears in the 50 texts. The only character beyond letters and `. , ? '` is the hyphen in "Re-centre" (F4).
- **13_10 "less than one three hundredth".** This is the only fraction in the 50 texts whose denominator is a multi-word ordinal; the screen shows "< 1/300". It is for listening, not a wording change.
- **Compound spoken fractions.** "one over R squared minus one", "pi R over R squared minus one" (13_3), "one over four p cubed" (13_7) and "pi R over R to the fourth minus one" (13_10) do not mark their grouping in words. In each case the cue's own scene block places the formula: 13_3 at `real_integral.rs:444` and `:458`, 13_7 at `:656–662`, 13_10 at `:749–781`. No change proposed.
- **Expected content questions.** These are forms that drew bridge content questions in the 94 reviewed takes, counted in the 50 texts:
  - same sound: "sine" 6× (10_2 ×3, 10_4, 10_5, 10_6); "roots" 6× (11_5, 11_7 ×3, 11_8, 13_5);
  - open by hearing: "turn" 11× (06_9, 09_6 ×2, 11_1 ×2, 11_3 ×2, 11_8 ×3, 15_2);
  - notation:
    - "to the fourth/fifth/eighth/n" 10× (11_5, 11_7 ×3, 13_4, 13_7, 13_10 ×2, 14_7 ×2), like "x^40" before;
    - "minus" followed by a single-letter symbol 16× (06_8, 09_5, 09_7, 12_1 ×2, 12_3, 13_2, 13_7 ×2, 13_8, 13_11, 14_2 ×2, 14_3, 15_1, 15_4), like "-i" before;
    - "plus" in an expression 6× (06_8, 11_5, 12_4, 14_2 ×2, 14_8);
    - "one half" 2× (09_5, 14_2);
    - "nonconstant" (11_7), like "non-zero" before.
- **Function notation.** Root's targeted small-model jobs since my timing review found turbo dropping the words "of z" and "of a" in function notation where small.en keeps them: small02 on 08_2 (Mail 4527) and small04 on 09_1 (Mail 4550). The same form recurs in "gamma of z equals gamma of z plus one" (14_8), and as "sine of one over z" (10_4, 10_5).
- **Names.** "Liouville's" (10_6) is the one name no processed cue has said. Riemann, Cauchy, gamma and zeta all occur in cues already processed.

## Status noted, not acted on

- **09_5:** `submission01` (06:43:10Z) says Generate was clicked once. `submission-observation01` (06:43:23Z) supersedes it: "not dispatched; no completed generation claimed". It says to inspect the provider history before any retry.
- **Oliver in provenance text:** the script's header line and the units `scope` still say "ElevenLabs Oliver", as does the map's filename, which root has called provenance only. No request names Oliver.

## Replay

```
R=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-remaining-cues-review01
(cd $R && shasum -a 256 -c SHA256SUMS)
python3 -I -B $R/checks/remaining_cues_review.py
```

- **Where and what it writes:** run it from the repository root. It writes nothing; it runs root's `oracle.py` once and reads its output.
- **Exit codes:** 0 PASS, 1 FAIL, 3 if an input moved.
- **What it replays:** `checks/outputs/remaining_cues_review.txt` is its transcript, and a fresh run reproduces it byte for byte while the take folder is unchanged.
- **Once root has downloads:** after takes exist for some of these cues, line q1 adds a NOTE and nothing else changes.

## Limits

- **Source reading is not viewing.** Hooks, holds and on-screen text are as coded at `a8e92ca2`, not as rendered.
- **The mathematics check covers what the words claim,** not how they are spoken.
- **The content-question counts are counts of forms,** not predictions of what the recognizer will write.
