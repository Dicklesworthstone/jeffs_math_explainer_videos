# How to Learn a Secret About a Crowd Without Learning Each Person's Answer

Ask a class of twenty whether they ever copied homework and a yes is a
confession. Give everyone two private coins instead: heads on the first, tell
the truth; tails, let the second coin answer, heads for yes, tails for no.
On average the survey hears seven yes and thirteen no from a class in which
four copied, and gets back to four by taking away the second coin's yeses (a
quarter of the class) and doubling what is left. Yet no answer is harmless:
a yes is three times as likely from someone who copied, and asking the same
person again with fresh coins multiplies that.

**Status (2026-10-19, revision `v1-michael-20261019`, Beads
`mx-video-program-program-87do.8.1`):** chapter 01 (`01_the_coin`) is written
as source: facts, model, the independent oracle, the scene, its storyboard,
script and narration units. Nothing in this crate has been compiled, tested,
rendered, narrated or listened to by its lane; the orchestrator's combined
native batch and frame review are pending. Chapters 02 to 05 are planned (the
storyboard's arc); their numbers are already computed and oracle-checked.
The owner rejected the first wave's narration on 2026-10-19 (AGENTS.md,
"Owner rejection and withdrawal"): `narration_units.json` and the pause plan
are text inputs only, and nothing here is to be synthesized with the
rejected method.

| File | What it holds |
|---|---|
| `src/model.rs` | the mathematics: exact fractions, the coin rule, the class on average, the yes count's exact distribution, sampling variance, repeated and memoized answers |
| `src/facts.rs` | every number the video shows or says (83 keys), three pinned to their sources |
| `oracle.py` | the same numbers by independent methods (standard library only), 21 cross-checks |
| `src/chapters/the_coin.rs` | chapter 01's scene |
| `src/chapters/draw.rs` | the video's shapes, playback, line timing and think time |
| `docs/storyboard.md` | the words and pictures (the source of `script.tsv`), with chapter 01's semantic pause plan |
| `narration_units.json` | chapter 01's lines in three coherent takes for the central synthesis |
| `docs/fact-sheet.md`, `docs/brief.md`, `docs/protected.tsv` | each number's source; the brief; the words every revision keeps |

```bash
python3 -I -B oracle.py                                  # every number, and 21 cross-checks on stderr
python3 -I ../../tools/storyboard.py --check randomized_response
python3 -I ../../tools/lint_script.py script.tsv
./x randomized_response facts-check                      # the Rust facts against the oracle (builds)
```

## Sources

- **Dwork and Roth**, *The Algorithmic Foundations of Differential Privacy*,
  Foundations and Trends in Theoretical Computer Science 9 (2014). Read
  2026-10-19 from the authors' PDF (cis.upenn.edu/~aaroth/Papers/privacybook.pdf).
  §2.3 and §3.2 give this exact two-coin rule (their *tails* tells the truth;
  for a fair coin the swap changes nothing), the expected yes share
  (1/4)(1 − p) + (3/4)p = 1/4 + p/2 and the estimate "twice the fraction
  answering Yes minus 1/2". Claim 3.5: the rule is (ln 3, 0)-differentially
  private, Pr[Yes | Yes] = 3/4, Pr[Yes | No] = 1/4, and both answers' ratios
  are 3. Theorem 3.16: under composition "the epsilons and the deltas add up".
  §3.7: "Randomized Response is due to Warner".
- **Warner**, "Randomized response: a survey technique for eliminating evasive
  answer bias", *JASA* 60(309):63–69 (1965), as Dwork and Roth cite it.
  **Not read**: the paper is paywalled and its PubMed record did not load.
  Warner's own design is a spinner choosing between "I am in group A" and "I
  am not", with estimator [λ̂ − (1 − P)]/(2P − 1) and variance
  π(1 − π)/n + P(1 − P)/[n(2P − 1)²], **from secondary sources only** (the
  QuantEcon lecture "Randomized Response Surveys", read 2026-10-19, and a
  search summary of later papers). The two-coin rule gives exactly the answer
  chances of that spinner at P = 3/4, so its extra variance is 3/(4n)
  (`Channel::warner_extra_variance`); the video never calls the coin rule
  Warner's.
- **Erlingsson, Pihur and Korolova**, "RAPPOR: Randomized Aggregatable
  Privacy-Preserving Ordinal Response", CCS 2014 (arXiv 1407.6981). Read
  2026-10-19, §1: the one-coin and two-coin rules; "assuming absolute
  compliance with the randomization protocol (an assumption that may not hold
  for human subjects ...)"; memoization keeps ln(3)-differential privacy
  "even with an infinite number of responses" when the true value does not
  change or changes in an uncorrelated way; without it, 75 yes in 100 answers
  leave "No" a 1.39 × 10⁻²⁴ fraction of cases (`R-X3`, recomputed exactly as
  1/(3⁵⁰ + 1)).

## Closest treatments

None of these was watched as video; what each covers comes from text read on
2026-10-19.

- *Randomized Response Explained*, Security and Privacy Academy (YouTube
  `SD7EzSkBXug`): title and channel from YouTube's oEmbed metadata only. **Not
  watched; no transcript read; its content is unknown.**
- Pearce and Jiang, *How randomized response can help collect sensitive
  information responsibly* (Google PAIR explorable, 2020), through a text
  fetch: a plagiarism question, one coin (heads reports plagiarism), the
  estimate by doubling the non-plagiarism rate, simulated error spreads and
  the privacy/accuracy trade-off. No repeated answers or privacy budget. Its
  interactive pictures were not seen.
- University of Washington CSE 163, *Randomized Response* lesson: the same
  two-coin rule with heads telling the truth and a cheating question; a worked
  1/3 → 5/12 example, inverted in words; no variance, no repeated questions.
- QuantEcon, *Randomized Response Surveys*: Warner's spinner, the estimator
  and its two-part variance, an MSE comparison with untruthful direct
  answers; no repeated answers or composition.

What this video adds, if its later chapters are built: the rule's chances
counted from four coin outcomes on screen, both answers' exact ratios, the
exact scatter of one class of twenty, the variance split into sampling and
coins, and repeated-answer leakage counted over all 64 outcomes beside a
memoized answer. The question overlaps the common one (cheating, plagiarism);
no vacant niche is claimed.

## Scope limits

- The arithmetic assumes everyone follows the rule with fair, private coins.
  Whether real people answer more honestly under it is an empirical question
  this video does not answer and no participation or honesty effect is
  claimed.
- Chapter 01's counts are average counts (every coin outcome equally often),
  said and carded as such; a real class of twenty scatters, standard
  deviation √15 ≈ 3.87 people (`R-F7`).
- The class is fixed and every member answers: its scatter is the coins'
  alone, a variance of 3n/4 for the estimated count (15 for the class) or
  3/(4n) for the estimated share. Drawing n people independently, with
  replacement or from a population so large that it makes no difference,
  adds the ordinary sampling term: n·p(1 − p) for the count, p(1 − p)/n for
  the share (`R-F8`). Drawing all N = n people without replacement adds
  nothing, which is the fixed class again. The two stay apart.
- "Three times, never more" is about what one answer says about its own
  giver under this rule. Fresh coins on each repeat multiply the worst ratio
  (3, 9, 27, …); a memoized answer does not, while the truth stays put.
- The class, seats and question are invented; no real survey data is used.
