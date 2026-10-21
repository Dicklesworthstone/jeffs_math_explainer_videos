# Check a Giant Matrix Product Without Doing the Product Again

A narrated explainer of Freivalds' check (catalog identity `freivalds`).
Someone hands you C and says it is A times B. Redoing the product row by
column costs n³ multiplications. Instead, toss a coin for each column, add up
the heads columns of B and multiply by A, add up the heads columns of C, and
compare. A right answer always passes. A wrong answer passes at most half the
tosses, and fresh tosses multiply that down.

## Status (2026-10-20)

The complete episode is written as source: eleven chapters, 87 lines, 1,812
spoken words (the editorial candidate root accepted for source
implementation on 2026-10-20, with its corrections). **Nothing of it has been
compiled, rendered, narrated or listened to.** The lane that writes the code
never compiles, renders or narrates it; root's serial lane does.

| Chapter | Scene (`src/chapters/`) | What it shows |
|---|---|---|
| `01_handed_an_answer` | `handed_an_answer.rs` | cold open, title, row times column, a billion at a thousand rows |
| `02_cheap_checks` | `cheap_checks.rs` | spot checks at three by three and at a million entries; the sum of every column |
| `03_coins_choose` | `check_with_coins.rs` | a cancelling pair fools the sum; coins choose; one toss proves C wrong |
| `04_eight_tosses` | `check_with_coins.rs` | all eight tosses; the mistake; fair and unfair coins |
| `05_shapes_of_mistakes` | `never_more_than_half.rs` | other shapes; 19,682 nonzero small mistakes checked |
| `06_never_more_than_half` | `never_more_than_half.rs` | the pairing proof on one pair, with a nonzero entry of the mistake |
| `07_toss_again` | `toss_again.rs` | two fresh tosses, reuse, the ladder, what the chance means, the name |
| `08_seen_first` | `toss_again.rs` | a toss seen first can be beaten; what a programmer must do |
| `09_what_it_costs` | `exact_and_rounded.rs` | the cost in multiplications; the break-even; the loop |
| `10_exact_and_rounded` | `exact_and_rounded.rs` | doubles break both promises; a tolerance |
| `11_the_promise` | `exact_and_rounded.rs` | the exact promise and its assumptions; thirty checks for a billion; names to search; receipts |

Words and pictures: `docs/storyboard.md` (the generated `script.tsv`), and
`narration_units.json`, an unqualified grouping proposal of 47 units.

### History

The earlier four-chapter cut (288.5 s; chapters 01 to 04 accepted by root at
`94988fd`, `ca4f677`, `d139ca3` and `0846fc1`, with the review repairs of
`96cbe8e`) is preserved with its receipts in the narration lab. Its scenes are
reused here: its chapter 01 became chapters 03 and 04, its 02 became 05 and
06, its 03 became 07 and 08, and its 04 became 09 to 11. The accepted
evidence binds that source, not this one.

## What the video claims, exactly

- For n × n matrices of whole numbers (the proof works in any ring), let r
  be a column of n independent fair coins, 1 for heads and 0 for tails. The
  check passes when A(Br) = Cr. B·r and C·r only add columns; A(Br) takes n²
  multiplications.
- If C = AB, every toss passes: A(Br) = (AB)r exactly.
- If C ≠ AB, at most half of the 2^n tosses pass. Proof: pick an entry where
  the mistake D = C − AB is not zero, say row i, column j, and pair each toss
  with the one that differs only in coin j. Between the two, row i of D·r
  changes by exactly D_ij, which is not zero, so at most one of each pair
  passes. A "wrong entry" of C could itself be 0; the proof needs the nonzero
  entry of the mistake (06_1, 06_6). The count is a chance because every toss
  is equally likely: fair coins, independent of each other, and tossed after
  C is fixed. With coins heads one time in ten, the slip passes 9/10
  (F-F12.slip_passes); with only the slip's column's coin fair, still 1/2
  (F-F12.only_slip_coin_fair), so the video says the guarantee *uses* fair
  coins, not that fairness is necessary for every construction.
- k fresh tosses all pass a wrong C with chance at most 2^−k
  (`facts::theorem_bound`); thirty is the fewest with 2^−k under one in a
  billion (F-F13.target). That is a chance over the coins for a fixed C, not
  the chance that C is wrong after passing (07_7). A caught toss proves C
  wrong, and its disagreeing row is a nonzero row of the mistake (F-X6, over
  all 19,682 small mistakes and every toss).
- Two cheap checks fail honestly (chapter 02):
  - k different entries picked at random catch one wrong entry with chance
    k/n²; at a thousand rows half a million entries are the fewest for one
    half, 500,000,000 multiplications, half the product's (F-F11).
  - The sum of every column (the all-heads toss) costs nine multiplications
    at three by three and catches the slip in row 2 (15, 16, 15 against 15,
    15, 15; F-F10), but the slip with the entry beside it one too small (5,
    2, 8 in row 2) passes it (F-F10.cancelling).
  - With three columns every fixed choice of columns has a wrong C it misses
    (F-X3.known_toss.fooled 8 of 8); from two by two up as well (F-X3.n2); a
    single column with heads checks the product exactly (F-X3.n1).
- Costs are multiplications counted row by column, never times: one check is
  a thousandth of the product at a thousand rows, twenty a fiftieth, and at
  three by three three checks cost the product's 27 (F-F13). Additions and
  memory traffic are also work (09_3); a selected-column check need not read
  the tails columns of B and C. No claim is made about every known method of
  multiplying.
- In IEEE doubles (chapter 10, F-X4): N + 1 = 10^16 + 1 rounds to N, a right
  C fails toss 11, a wrong C by one passes all four tosses, and a tolerance of
  2 (the right C's largest gap in doubles) lets the right C through and the
  wrong C through every toss in exact sums: a different test, with no half
  bound.
- **Not claimed:** anything about floating point beyond those examples;
  modular or larger-alphabet checks; wall-clock speed; derandomizing the
  check; anything about every known multiplication method.

The scenes refuse to render when a spoken claim stops being true (the
`Err(fail(..))` guards beside each picture), and `src/main.rs`' SPOKEN table
binds every number word of the script to the fact it speaks for.

## Sources

- R. Freivalds, "Probabilistic Machines Can Use Less Running Time",
  *Information Processing 77* (IFIP Congress 77), North-Holland, pp. 839–842,
  as Wikipedia cites it. **The paper itself has not been read**, so the video
  says no date and no biography; it only names the algorithm.
- Wikipedia, "Freivalds' algorithm" (read 2026-10-19 and 2026-10-20): named
  after Rūsiņš Mārtiņš Freivalds; "This is called one-sided error"; the
  citation above. "Monte Carlo" does not appear there, so the names card does
  not use it.
- UIUC CS 473, Fall 2019, lecture 12, "Freivalds Trick and Schwartz-Zippel
  Lemma" (read 2026-10-19; text re-read 2026-10-20): r uniform in {0,1}^n,
  each coordinate independent and uniform; YES with probability 1 when AB =
  C, at most 1/2 otherwise; k repetitions, error ≤ 1/2^k in O(kn²); the
  pairing proof; and, in its outline, the generalization to polynomial
  identity testing via the Schwartz–Zippel lemma, the only basis for chapter
  11's names to search.
- Wikipedia, "Computational complexity of matrix multiplication": the
  row-by-column n³ count. The video quotes no exponent and no claim about the
  fastest methods.

## Closest competing treatments (inspected, not watched)

- Wikipedia's 2 × 2 worked example (A = (2 3 / 3 4), B = (1 0 / 1 2),
  C = (6 5 / 8 7)).
- "Freivalds' Algorithm: a randomized algorithm to verify matrix
  multiplication || Monte Carlo", Cyber Enlightener (YouTube yh73ChFj5ik,
  40 min 14 s) and "Power of randomized algorithm: Freivalds' algorithm",
  Math Geeks (LdJzuNSkHCQ, 8 min 59 s). Only titles, channels, dates, lengths
  and descriptions were read. **Neither has been watched.**

## Pronunciation before any synthesis

- "Freivalds" (Latvian; Rūsiņš Mārtiņš Freivalds), said once, in 07_9: look
  it up, respell it for the voice, probe it against a control and a decoy,
  and lock it into `lexicon.json` with its sources. The owner's pronunciation
  overrides every source. Root performs the bounded route probe; no
  respelling is entered here.
- The letter names A, B and C as matrix names, in their sentences.
- 10_6 opens with the article "A" ("A tolerance would …"), kept verbatim from
  the accepted line: probe that it is read as the article, not the matrix.

## Checks

Cheap, no compilation (from this folder, then from the repo root):

```bash
python3 -I -B oracle.py                                   # the facts, and 29 cross-checks
python3 -I tools/storyboard.py --check freivalds          # G2, script.tsv current
python3 -I tools/lint_script.py explainer_corpus/freivalds/script.tsv --chapters 01_handed_an_answer,02_cheap_checks,03_coins_choose,04_eight_tosses,05_shapes_of_mistakes,06_never_more_than_half,07_toss_again,08_seen_first,09_what_it_costs,10_exact_and_rounded,11_the_promise
```

Native, root's batch: `./x freivalds facts-check`, the crate's fmt, clippy
and tests, the glyph probe, and a 540p scratch-voice draft with the layout
lint and contact and cue sheets read.
