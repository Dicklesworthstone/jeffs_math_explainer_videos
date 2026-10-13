# Fact sheet: Video E, "How to Make a Fair Coin Out of an Unfair One"

Bead `mx-e-a-e-e-k23.2` (gate G1). One row per number the video may show or
say, and one row per claim that rests on a source. Every value in the table
at the end was copied from the oracle's output by a script, not typed.

```
./x fair_coin facts-check        # coin facts == oracle.py, key by key
```

- **Result, 2026-10-12** (the build host, Apple M5 Ultra; CPython 3.14.0,
  NumPy 2.5.4): `facts-check fair_coin: 237 video facts against 237 oracle
  keys: PASS`. The oracle ran twice (10.0 s and 9.6 s) with byte-identical
  stdout: 237 keys, 42 of 42 cross-checks passed, exit 0.
- **Rust side:** `./x fair_coin test`: 20 tests pass, clippy `-D warnings`
  and `fmt --check` clean.
- **Rows:** E-F computed chances and costs; E-X edges of always/never
  sentences that are counts; E-S seeded runs; E-C cited literals.

## How the two sides get their numbers

| What | `src/model.rs` (the video) | `oracle.py` (independent) |
|---|---|---|
| chances of two tosses, costs, dice | its own exact fraction type (`Q`, i128, rounded in integers) | `fractions.Fraction`, plus a full listing of the outcomes wherever one exists |
| the coin with memory: the plain rule, Samuelson's rule | the absorbing chain's equations solved exactly by elimination | the same chains set up separately and solved with Fractions |
| Blum's algorithms A and B | every history followed forward in floating point until 1e-15 of the chance is left | an exact linear solve over every reachable memory state (18 states for A, 45 for B); a second forward pass for the first four results |
| "no fixed-length rule is fair" | every list of how many winning patterns have each count of heads, tested at n + 1 chances | the same lists, and for n ≤ 4 every rule as an actual set of patterns (65,536 at n = 4) in integer arithmetic |
| rates | Peres's recursion and Elias's sum in f64 | the same formulas written separately; the entropy checked as a fixed point of Peres's recursion |
| the procedures on real tosses | recursive Peres; Elias by a combinatorial rank | Peres with a work list; Elias by listing each class |
| seeded runs | `explainer_kit::stats::Data` (franken_manim's PCG64DXSM) | NumPy's own `Generator(PCG64DXSM(seed))`, the same draws in the same order |

Seeds: run *i* uses seed 195100 + *i* (1951 is the year von Neumann's note
was issued; chosen before any run was looked at, and no run was re-seeded).
A toss is heads when the draw is below the chance; a die's face is the first
whose running total of twentieths exceeds the draw.

## Findings (uncomfortable first)

1. **Blum's Theorem A, part 1, reads the other way round from his own
   algorithm.** Part 1 as printed in the FOCS 1984 text says algorithm A
   outputs "nearly k/2 Hs (nearly k/3 Hs) before outputting its first T".
   With the algorithm exactly as printed on the same pages (exits 1 then 0
   give H, exits 0 then 1 give T), an H needs a return to a state, so the
   first results are Ts: from a fresh state the leaving exit completes a
   0-then-1 pair with chance (1 − ε)/(2 − ε), which is 1/2 as ε → 0 and 1/3
   at ε = 1/2. Those are his k/2 and k/3 with the letters interchanged. Our
   exact numbers for k = 2: the first result is T with chance 7/10 at
   ε = 1/5, 5/8 at ε = 1/2, 0.7475 at ε = 1/100. Part 2 (after HH, TT almost
   never follows: 0.0074 at ε = 1/100 where 1/4 is fair; the quadruple HHTT
   has chance 0.0009 where 1/16 is fair) and Theorem B (every quadruple
   exactly 1/16, for k = 2, 4 and 6) reproduce. The Combinatorica 1986
   version is behind a paywall and was not seen, so whether it corrects the
   wording is unknown. Nothing in the video depends on the direction; the
   script says only that announcing early is wrong and announcing on return
   is right.
2. **"A spun penny lands tails 80% of the time" has no data behind it that
   we could find.** The only source is a journalist's paraphrase of Diaconis
   (Science News, 2004). Measured spins say less: 55% tails over 20,422
   spins, and 66% tails for 1960s pennies over 5,520 spins (classroom
   records in Chance News 11.02). The video's 80% coin is a model and says
   so; the 80% penny is not said.
3. **The trick does not need independence so much as symmetry, and
   independence alone is not enough.** What makes heads-tails and tails-heads
   tie is that swapping the two tosses leaves the chance unchanged. Two
   independent tosses of *different* coins break it (0.40 against 0.10);
   a coin with memory, taken from a long run, keeps the first pair fair
   (`E-F6.sticky.long_run_first_pair`) and breaks only what follows a
   thrown-away pair. The script's wording is "the same chance on both
   tosses, and neither toss affects the other".
4. **Samuelson's rule is exactly fair only when played to the end.** Under a
   deadline ("stop after N tosses") the two orders of a pair complete at
   different times, so the truncated game leans: for the sticky coin, of the
   games decided within 8 tosses heads-first wins 64.7%, within 16 tosses
   53.9%, within 32 tosses 50.5% (exact, a scratch computation outside the
   oracle). Zhou and Bruck make the same point about exit sequences of a
   fixed-length input. The video never sets a deadline.
5. **A die from four tosses is not cheap.** With the 80% coin it works 15.4%
   of the time, 26.0 tosses a die on average; three tosses that are not all
   alike (a fair choice among three) plus one coin trick cost 12.5. The
   script says "this is the simplest way, not the cheapest".
6. **Which pair counts as heads differs between sources.** Von Neumann's
   text takes the first toss of the differing pair (heads-tails → heads);
   Peres, Hoeffding and Simons, Intel and the NIST draft take the second;
   Elias, Vadhan and RFC 4086 write 01 → 0. The video follows von Neumann.
7. **NIST's final SP 800-90B (2018) does not mention von Neumann's method;**
   its 2016 second draft listed it as an approved post-processing function.
   Never "NIST-approved".
8. **Samuelson's own two pages were not seen** (paywall). His rule is taken
   from Elias 1972 (p. 866), Stout and Warren 1984 (p. 213), the abstract in
   OpenAlex's metadata, and Blum's sentence quoted below. Every statement
   about it in the video is also computed here (`E-F6.*.after_heads_only`).
9. **Gelman and Nolan's text and figure caption disagree** on their flipped
   checker (47 tails in 100 in the text, "54 heads, 46 tails" in the caption
   of Figure 2). We quote neither number.
10. **A wrong citation circulates for von Neumann's note:** the scan on
    LANL's site carries an overlaid header "J. Res. Nat. Bur. Stand. Appl.
    Math. Series 3" and its BibTeX says 1955. The title page says Applied
    Mathematics Series 12, issued June 11, 1951.

## Sources

Downloaded copies and text extractions live outside the repository and are
never committed. "Seen" means read on the page (or its image) by this lane
or by its research agent, and re-checked by a search for the quoted words.

| id | Source | Seen | Used for |
|---|---|---|---|
| S1 | John von Neumann, "Various Techniques Used in Connection With Random Digits" (summary written by George E. Forsythe), in *Monte Carlo Method*, ed. A. S. Householder with G. E. Forsythe and H. H. Germond, National Bureau of Standards Applied Mathematics Series 12, U.S. Government Printing Office, issued June 11, 1951, pp. 36–38. Proceedings of a symposium held June 29, 30 and July 1, 1949, on the UCLA campus | title page, p. II, preface, pp. 36–38 (page images) and the Collected Works reprint, vol. 5, pp. 768–770 | the trick, its efficiency, the dates |
| S2 | Yuval Peres, "Iterating von Neumann's Procedure for Extracting Random Bits", *Annals of Statistics* 20(1), 1992, 590–597 | whole paper | the iteration, Proposition 2 (the recursion), Proposition 3 (the limit) |
| S3 | Peter Elias, "The Efficient Construction of an Unbiased Random Sequence", *Annals of Mathematical Statistics* 43(3), 1972, 865–870 | whole paper | blocks sorted by count; Theorem 2 |
| S4 | Paul A. Samuelson, "Constructing an Unbiased Random Sequence", *JASA* 63(324), 1968, 1526–1527 | **not seen**; described in S3 p. 866, S8 p. 213, S5 | tosses that follow a common outcome |
| S5 | Manuel Blum, "Independent Unbiased Coin Flips From a Correlated Biased Source: A Finite State Markov Chain", 25th FOCS, 1984, 425–433; journal version *Combinatorica* 6(2), 1986, 97–108 | FOCS version, whole paper (an archived copy); the journal's abstract only | algorithms A and B, Theorems A and B |
| S6 | Wassily Hoeffding and Gordon Simons, "Unbiased Coin Tossing with a Biased Coin", *Annals of Mathematical Statistics* 41(2), 1970, 341–352 | pp. 341–342 | the cost 1/(pq); "rediscovered by others" |
| S7 | Edsger W. Dijkstra, "Making a fair roulette from a possibly biased coin", EWD 1071 (15 November 1989); *Information Processing Letters* 36(4), 1990, 193 | the handwritten manuscript | 37 equally likely outcomes from 37 tosses |
| S8 | Quentin F. Stout and Bette Warren, "Tree Algorithms for Unbiased Coin Tossing with a Biased Coin", *Annals of Probability* 12(1), 1984, 212–222 | pp. 212–213 | priority; no optimal algorithm |
| S9 | Ari Juels, Markus Jakobsson, Elizabeth Shriver, Bruce K. Hillyer, "How to turn loaded dice into fair coins", *IEEE Trans. Inform. Theory* 46(3), 2000, 911–921 | the authors' 1998 preprint, p. 1 | dice as sources |
| S10 | Andrew Gelman and Deborah Nolan, "You Can Load a Die, But You Can't Bias a Coin", *The American Statistician* 56(4), 2002, 308–311 | whole paper | what can and cannot be biased |
| S11 | Persi Diaconis, Susan Holmes, Richard Montgomery, "Dynamical Bias in the Coin Toss", *SIAM Review* 49(2), 2007, 211–235 | abstract, p. 218 | same side about .51; spun pennies |
| S12 | František Bartoš et al., "Fair Coins Tend to Land on the Same Side They Started: Evidence from 350,757 Flips", *JASA* 120(552), 2025, 2118–2127 | arXiv 2310.04153 v4 (the publisher's page was not reachable; venue from Crossref) | 0.508 |
| S13 | *Chance News* 11.02 (2002), "Euro coin" discussion, archived copy | whole item | counts of spun pennies |
| S14 | Benjamin Jun and Paul Kocher, "The Intel Random Number Generator", Cryptography Research white paper for Intel, April 22, 1999 | §4.3 | the von Neumann corrector in hardware |
| S15 | Salil Vadhan, *Pseudorandomness*, Foundations and Trends in TCS 7(1–3), 2012 | author's August 2012 draft, §6.1 and §6.5 | "goes back to von Neumann" |
| S16 | Noam Nisan and David Zuckerman, "Randomness is Linear in Space", *JCSS* 52(1), 1996, 43–52 | Zuckerman's posted copy, p. 2 | the word "extractor" |
| S17 | NIST SP 800-90B, second draft (January 2016) and final (January 2018) | both, searched in full | finding 7 |
| S18 | Hongchao Zhou and Jehoshua Bruck, "Efficient Generation of Random Bits from Finite State Markov Chains", arXiv 1012.5339 | pp. 1–8 | finding 4; a second description of S5's algorithm B |

### Passages quoted or leaned on (verbatim)

- **S1, p. 36** (the whole passage): "To cite a human example, for
  simplicity, in tossing a coin it is probably easier to make two consecutive
  tosses independent than to toss heads with probability exactly one-half. If
  independence of successive tosses is assumed, we can reconstruct a 50-50
  chance out of even a badly biased coin by tossing twice. If we get
  heads-heads or tails-tails, we reject the tosses and try again. If we get
  heads-tails (or tails-heads), we accept the result as heads (or tails). The
  resulting process is rigorously unbiased, although the amended process is at
  most 25 percent as efficient as ordinary coin-tossing."
- **S1, p. 36:** "Any one who considers arithmetical methods of producing
  random digits is, of course, in a state of sin." ("Any one" is two words
  in the original.)
- **S2, abstract:** "Given a sequence of independent, identically distributed
  random biased bits, von Neumann's simple procedure extracts independent
  unbiased bits. In this note we show that the number of unbiased bits
  produced by iterating this procedure is arbitrarily close to the entropy
  bound."
- **S2, Proposition 2** (p. 593): r_ν(p) = pq + ½ r_{ν−1}(p² + q²) + ½ (p² +
  q²) r_{ν−1}(p²/(p² + q²)) for ν ≥ 2, with r_1(p) = pq. **Proposition 3**
  (p. 594): lim r_ν(p) = h(p) uniformly in p ∈ (0, 1).
- **S3, p. 868:** "Given N, divide the set of 2^N possible input sequences
  into the N + 1 composition classes S_k … Since all of the input sequences in
  S_k have the same probability for any fixed input p, …". **Abstract:**
  "…gaining a factor of 3 or 4 in efficiency."
- **S5 (FOCS), p. 426:** "It is easy to see that it is possible in principle
  to generate independent unbiased bits from a MC: use algorithm vN on the
  symbols produced by the MC just when it exits some particular state,
  throwing out all symbols produced by the MC when it exits other states."
  And: "Algorithm A outputs the H (or T) as soon as 10 (or 01) is produced by
  a state. Algorithm B, however, is patient: it waits and outputs the H (or
  T) later, when the state that produced 10 (or 01) is reentered. The
  difference (between A and B) is crucial. A is bad while B is good."
- **S6, p. 341:** "This procedure is described by von Neumann (1951) and has
  been rediscovered by others."
- **S8, p. 212:** "Von Neumann ([7]) seems to have been the first to propose
  such an algorithm and Hoeffding and Simons ([4]) the first to analyze the
  problem in depth."
- **S10, abstract:** "…it is not possible to bias a coin flip—that is, one
  cannot, for example, weight a coin so that it is substantially more likely
  to land "heads" than "tails" when flipped and caught in the hand in the
  usual manner. Coin tosses can be biased only if the coin is allowed to
  bounce or be spun rather than simply flipped in the air."
- **S11, abstract:** "We show that vigorously flipped coins tend to come up
  the same way they started. … For natural flips, the chance of coming up as
  started is about .51." **p. 218:** "The spun coins show a pronounced
  proportion of tails; several students had coins that came up fewer than 10%
  heads."
- **S12, abstract (arXiv v4):** "…the coins landed on the same side more
  often than not, Pr(same side) = 0.508, 95% credible interval (CI) [0.506,
  0.509]…"
- **S13:** "Using 1960's pennies, my students have 1878/5520 = 34.0% heads
  over the past four semesters." (Doug Andrews's classes; the same item
  gives Robin Lock's 9,197 heads in 20,422 spins.)
- **S14, §4.3:** "The initial random measurements are processed by a hardware
  corrector based on a concept proposed by John von Neumann to produce a
  balanced distribution of "0" and "1" bits." And: "The corrector generates
  an average of one bit for every 6 raw binary samples."
- **S15, §6.5:** "The study of randomness extraction goes back to von Neumann
  [von], who gave the deterministic extractor described in the text for
  IID-bit sources."
- **S16, p. 2:** "Our main technical tool is a construction of the following
  kind of function which we call an extractor."

## Edge tests for always/never sentences (plan §5.3)

Each sentence the storyboard may voice, the test that tries it at its edges,
and what the edge is. Tests are in `src/model.rs` unless noted; the oracle
repeats each as a cross-check.

| Sentence | Test | The edge |
|---|---|---|
| "Heads-tails and tails-heads are equally likely, whatever the coin's chance of heads" | `edge_heads_tails_equals_tails_heads_for_every_chance` | true at every chance, 0 and 1 included (both areas are then zero); but at 0 and 1 no pair ever decides, so the *game* needs a coin that can land both ways |
| "At least half the pairs are thrown away" | `edge_at_least_half_the_pairs_are_thrown_away` | exactly half only for a fair coin |
| "It never costs fewer than four tosses on average" | `edge_expected_tosses_bottom_out_at_four` | four only for a fair coin; no upper limit near 0 or 1 (over 1,000 at 99.9%) |
| "With two different coins it is fair only if they match" | `edge_two_coins_are_fair_only_when_they_match` | every pair of 29 chances (0, 1, 1/1000, 999/1000, the twentieths and the sevenths); a fair second coin returns the first coin's chance exactly |
| "No rule that stops after a fixed number of tosses is fair for every coin" | `edge_no_fixed_length_rule_is_fair_for_every_coin`, `E-X1.*` | every rule on 1 to 5 tosses; the check does accept rules that are fair for one coin |
| "A coin with memory breaks the rule after a thrown-away pair, but the first pair of a long run is still fair" | `edge_memory_breaks_the_plain_rule_but_not_the_first_long_run_pair` | every pair of the same 29 chances as a memory; fair from both starts exactly when there is no memory |
| "Pairing only the tosses that follow a head is fair for any coin that remembers one toss" | `edge_after_only_rule_is_fair_for_every_memory` | fails exactly when heads can never be followed by both faces, or heads never recurs |
| "Announce at once and it is wrong; announce on return and it is right" (Blum) | `blum_at_once_is_bad_and_on_return_is_good` | even when every exit is a fair toss (ε = 1/2), announcing at once leans |
| "Patterns with the same counts have the same chance" | `edge_same_counts_same_chance` | every pattern of 1 to 6 tosses at the 29 chances |
| "The n! orders of n different rolls are equally likely" | `edge_every_order_of_different_rolls_is_equally_likely` | three dice (loaded, fair, 1:2:3:4:5:6), n = 2 and 3 by full listing (the oracle adds n = 4) |
| "No rule beats the entropy; Peres's and Elias's climb toward it" | `edge_rates_climb_to_the_entropy_and_never_pass_it` | six chances, depths 1–10, blocks 2–64: always below, always rising |
| "The output is fair tosses" (all three procedures) | `the_procedures_give_fair_output_on_every_short_run` | every run of 8 tosses of the 80% coin: outputs of equal length are exactly equally likely |

## Simplification → exact (plan §5.1 rule 11)

| The video says | Exactly |
|---|---|
| "the coin's chance of heads" stays the same | the two tosses of a pair are independent and identically distributed; more generally, it is enough that swapping the two tosses leaves the pair's chance unchanged (exchangeability) |
| "every coin" | every chance of heads strictly between 0 and 1 |
| "sooner or later a pair decides" | with probability one; the number of pairs is geometric with success chance 2p(1 − p), unbounded |
| "6.25 tosses on average" | the expectation 1/(p(1 − p)) at p = 4/5 |
| "a coin that remembers its last toss" | a stationary two-state Markov chain with P(H after H) = P(T after T) = 4/5 |
| "fair tosses per hundred unfair ones" | output bits per input bit, times 100; in the limit of long runs for Peres and long blocks for Elias |
| "at most 72" | the binary entropy h(4/5) = 0.7219… bits per toss, an upper bound on any such rule's long-run rate (Elias 1972, Theorem 1) |
| "the order carries no trace of the coin's chance" | given the counts, the arrangement is uniform over the arrangements with those counts |

## The table

<!-- generated from oracle.py's output; do not edit by hand -->

**E-F1.** The unit square for the 80% coin: the chance of each pair of tosses (the cell's area).

| key | value |
|---|---|
| `E-F1.square.hh` | 0.64 |
| `E-F1.square.hh.exact` | 16/25 |
| `E-F1.square.ht` | 0.16 |
| `E-F1.square.ht.exact` | 4/25 |
| `E-F1.square.th` | 0.16 |
| `E-F1.square.th.exact` | 4/25 |
| `E-F1.square.tt` | 0.04 |
| `E-F1.square.tt.exact` | 1/25 |
| `E-F1.heads_percent` | 80 |
| `E-F1.tails_percent` | 20 |

**E-F2.** One pair: how often it decides, how often it is thrown away, and who wins when it decides.

| key | value |
|---|---|
| `E-F2.decides` | 0.32 |
| `E-F2.decides_percent` | 32 |
| `E-F2.thrown_away` | 0.68 |
| `E-F2.thrown_away_percent` | 68 |
| `E-F2.heads_first_share.exact` | 1/2 |
| `E-F2.heads_first_share_percent` | 50 |

**E-F3.** The cost at 80%: expected pairs and tosses, and the chance that the first 1, 2, 3, 5, 10 pairs are all thrown away.

| key | value |
|---|---|
| `E-F3.expected_pairs` | 3.125 |
| `E-F3.expected_tosses` | 6.25 |
| `E-F3.expected_tosses.exact` | 25/4 |
| `E-F3.undecided_after.1_pairs_percent` | 68.0 |
| `E-F3.undecided_after.2_pairs_percent` | 46.2 |
| `E-F3.undecided_after.3_pairs_percent` | 31.4 |
| `E-F3.undecided_after.5_pairs_percent` | 14.5 |
| `E-F3.undecided_after.10_pairs_percent` | 2.1 |

**E-F4.** The slider's stops (chance of heads 50, 60, 70, 80, 90, 99 percent): the area of heads-tails (equal to tails-heads), how often a pair decides, expected tosses.

| key | value |
|---|---|
| `E-F4.p50.ht` | 0.2500 |
| `E-F4.p50.decides_percent` | 50.0 |
| `E-F4.p50.expected_tosses` | 4.00 |
| `E-F4.p60.ht` | 0.2400 |
| `E-F4.p60.decides_percent` | 48.0 |
| `E-F4.p60.expected_tosses` | 4.17 |
| `E-F4.p70.ht` | 0.2100 |
| `E-F4.p70.decides_percent` | 42.0 |
| `E-F4.p70.expected_tosses` | 4.76 |
| `E-F4.p80.ht` | 0.1600 |
| `E-F4.p80.decides_percent` | 32.0 |
| `E-F4.p80.expected_tosses` | 6.25 |
| `E-F4.p90.ht` | 0.0900 |
| `E-F4.p90.decides_percent` | 18.0 |
| `E-F4.p90.expected_tosses` | 11.11 |
| `E-F4.p99.ht` | 0.0099 |
| `E-F4.p99.decides_percent` | 2.0 |
| `E-F4.p99.expected_tosses` | 101.01 |
| `E-F4.p99.expected_tosses_rounded` | 101 |
| `E-F4.p50.expected_tosses.exact` | 4 |

**E-F5.** Two different coins: 80% then a fair one.

| key | value |
|---|---|
| `E-F5.two_coins.second_percent` | 50 |
| `E-F5.two_coins.ht` | 0.40 |
| `E-F5.two_coins.th` | 0.10 |
| `E-F5.two_coins.heads_first_percent` | 80 |
| `E-F5.two_coins.ratio` | 4 |

**E-F6.** A coin that repeats its last face 80% of the time ("sticky"), and a lopsided memory (heads after a head 9/10, after a tail 1/2): the pair after a head or a tail, the plain rule played to the end, the first pair of a long run, and Samuelson's rule (only the tosses that follow a head).

| key | value |
|---|---|
| `E-F6.sticky.repeat_percent` | 80 |
| `E-F6.sticky.after_heads.ht` | 0.16 |
| `E-F6.sticky.after_heads.th` | 0.04 |
| `E-F6.sticky.after_heads.ratio` | 4 |
| `E-F6.sticky.after_heads.heads_first_percent` | 80 |
| `E-F6.sticky.after_tails.heads_first_percent` | 20 |
| `E-F6.sticky.long_run_first_pair.heads_first.exact` | 1/2 |
| `E-F6.sticky.plain_rule.after_heads_percent` | 61.5 |
| `E-F6.sticky.plain_rule.after_tails_percent` | 38.5 |
| `E-F6.sticky.plain_rule.long_run.exact` | 1/2 |
| `E-F6.sticky.after_heads_only.from_heads.exact` | 1/2 |
| `E-F6.sticky.after_heads_only.from_tails.exact` | 1/2 |
| `E-F6.lopsided.plain_rule.long_run.exact` | 5/9 |
| `E-F6.lopsided.after_heads_only.exact` | 1/2 |

**E-F7.** Blum's algorithms on his two-state chain at ε = 1/5 (the sticky coin): the chances of the first two announced results.

| key | value |
|---|---|
| `E-F7.blum.at_once.hh` | 0.1500 |
| `E-F7.blum.on_return.hh` | 0.2500 |
| `E-F7.blum.at_once.ht` | 0.1500 |
| `E-F7.blum.on_return.ht` | 0.2500 |
| `E-F7.blum.at_once.th` | 0.3500 |
| `E-F7.blum.on_return.th` | 0.2500 |
| `E-F7.blum.at_once.tt` | 0.3500 |
| `E-F7.blum.on_return.tt` | 0.2500 |

**E-F8.** Same counts, same chance: Pascal's rows; four tosses with two heads; row 4's groups at 80%; three tosses.

| key | value |
|---|---|
| `E-F8.pascal.row2` | 1,2,1 |
| `E-F8.pascal.row3` | 1,3,3,1 |
| `E-F8.pascal.row4` | 1,4,6,4,1 |
| `E-F8.four_two.patterns` | 6 |
| `E-F8.four_two.list` | HHTT,HTHT,HTTH,THHT,THTH,TTHH |
| `E-F8.four_two.each` | 0.0256 |
| `E-F8.four_two.each.exact` | 16/625 |
| `E-F8.four_two.works_percent` | 15.4 |
| `E-F8.four_two.expected_tosses` | 26.0 |
| `E-F8.four_two.fair_coin.works_percent` | 37.5 |
| `E-F8.four_two.fair_coin.expected_tosses` | 10.7 |
| `E-F8.row4.heads0_percent` | 0.16 |
| `E-F8.row4.heads1_percent` | 2.56 |
| `E-F8.row4.heads2_percent` | 15.36 |
| `E-F8.row4.heads3_percent` | 40.96 |
| `E-F8.row4.heads4_percent` | 40.96 |
| `E-F8.three.works_percent` | 48 |
| `E-F8.three_plus_coin.expected_tosses` | 12.5 |

**E-F9.** The loaded die (5, 5, 10, 10, 20, 50 percent): two rolls, three rolls, the brick of faces four, five and six, and n! for n = 2 to 6.

| key | value |
|---|---|
| `E-F9.loaded.percents` | 5,5,10,10,20,50 |
| `E-F9.loaded.six_percent` | 50 |
| `E-F9.two_rolls.tie_percent` | 31.5 |
| `E-F9.two_rolls.first_bigger_percent` | 34.25 |
| `E-F9.two_rolls.decides_percent` | 68.5 |
| `E-F9.two_rolls.expected_rolls` | 2.9 |
| `E-F9.two_rolls.fair_die.expected_rolls` | 2.4 |
| `E-F9.three_rolls.orders` | 6 |
| `E-F9.three_rolls.each_order_percent` | 5.43 |
| `E-F9.three_rolls.works_percent` | 32.6 |
| `E-F9.three_rolls.expected_rolls` | 9.2 |
| `E-F9.three_rolls.fair_die.works.exact` | 5/9 |
| `E-F9.three_rolls.fair_die.expected_rolls` | 5.4 |
| `E-F9.three_rolls.trios` | 20 |
| `E-F9.three_rolls.boxes_all_different` | 120 |
| `E-F9.three_rolls.boxes` | 216 |
| `E-F9.brick.sides_percent` | 10,20,50 |
| `E-F9.brick.volume_percent` | 1.0 |
| `E-F9.orders.2` | 2 |
| `E-F9.orders.3` | 6 |
| `E-F9.orders.4` | 24 |
| `E-F9.orders.5` | 120 |
| `E-F9.orders.6` | 720 |

**E-F10.** Fair tosses out per unfair toss in, at 80% (and for a fair coin): von Neumann's rule, the entropy, Peres's procedure by depth, Elias's rule by block size.

| key | value |
|---|---|
| `E-F10.von_neumann.rate` | 0.16 |
| `E-F10.von_neumann.per_hundred` | 16 |
| `E-F10.entropy` | 0.7219 |
| `E-F10.entropy.per_hundred` | 72 |
| `E-F10.von_neumann.share_of_entropy_percent` | 22 |
| `E-F10.fair_coin.von_neumann.rate` | 0.25 |
| `E-F10.fair_coin.von_neumann.percent` | 25 |
| `E-F10.fair_coin.entropy` | 1 |
| `E-F10.peres.depth1.rate` | 0.1600 |
| `E-F10.peres.depth2.rate` | 0.2876 |
| `E-F10.peres.depth3.rate` | 0.3874 |
| `E-F10.peres.depth4.rate` | 0.4655 |
| `E-F10.peres.depth6.rate` | 0.5721 |
| `E-F10.peres.depth8.rate` | 0.6351 |
| `E-F10.elias.block2.rate` | 0.1600 |
| `E-F10.elias.block4.rate` | 0.2816 |
| `E-F10.elias.block8.rate` | 0.3708 |
| `E-F10.elias.block16.rate` | 0.4940 |
| `E-F10.elias.block32.rate` | 0.5844 |
| `E-F10.elias.block64.rate` | 0.6462 |

**E-X1.** Edge: rules that stop after exactly n tosses, as lists of how many winning patterns have each count of heads, and how many of them are fair for every coin.

| key | value |
|---|---|
| `E-X1.fixed_rules.n1.count_lists` | 4 |
| `E-X1.fixed_rules.n1.fair_for_every_coin` | 0 |
| `E-X1.fixed_rules.n2.count_lists` | 12 |
| `E-X1.fixed_rules.n2.fair_for_every_coin` | 0 |
| `E-X1.fixed_rules.n3.count_lists` | 64 |
| `E-X1.fixed_rules.n3.fair_for_every_coin` | 0 |
| `E-X1.fixed_rules.n4.count_lists` | 700 |
| `E-X1.fixed_rules.n4.fair_for_every_coin` | 0 |
| `E-X1.fixed_rules.n5.count_lists` | 17424 |
| `E-X1.fixed_rules.n5.fair_for_every_coin` | 0 |

**E-S0.** The seed.

| key | value |
|---|---|
| `E-S0.seed` | 1951 |

**E-S1.** Run 1: 1,000 tosses of the 80% coin, just calling it.

| key | value |
|---|---|
| `E-S1.call_it.tosses` | 1000 |
| `E-S1.call_it.heads` | 791 |
| `E-S1.call_it.tails` | 209 |

**E-S2.** Run 2: 1,000 games of the trick at 80%.

| key | value |
|---|---|
| `E-S2.trick.games` | 1000 |
| `E-S2.trick.heads_first` | 494 |
| `E-S2.trick.tails_first` | 506 |
| `E-S2.trick.tosses` | 6256 |
| `E-S2.trick.mean_tosses` | 6.26 |
| `E-S2.trick.longest_pairs` | 22 |
| `E-S2.trick.first_games` | HHHHHHTH HHHHHT HHHHHT HHHHHT HHHHHHHHHHHT TH HHHHHHTTHHHHHHHHHHHHTH HT |

**E-S3.** Runs 10 to 15: 1,000 games at each stop of the slider.

| key | value |
|---|---|
| `E-S3.p50.heads_first` | 493 |
| `E-S3.p50.mean_tosses` | 3.96 |
| `E-S3.p60.heads_first` | 502 |
| `E-S3.p60.mean_tosses` | 4.12 |
| `E-S3.p70.heads_first` | 490 |
| `E-S3.p70.mean_tosses` | 4.93 |
| `E-S3.p80.heads_first` | 509 |
| `E-S3.p80.mean_tosses` | 6.35 |
| `E-S3.p90.heads_first` | 472 |
| `E-S3.p90.mean_tosses` | 10.92 |
| `E-S3.p99.heads_first` | 513 |
| `E-S3.p99.mean_tosses` | 100.32 |

**E-S4.** Run 4: 10,000 games at 80%: mean tosses, the share decided by the first pair, how many games needed 1 to 8 pairs, 9 or more, and the longest.

| key | value |
|---|---|
| `E-S4.price.games` | 10000 |
| `E-S4.price.mean_tosses` | 6.21 |
| `E-S4.price.first_pair_percent` | 32.1 |
| `E-S4.price.pairs_1_to_8` | 3209,2170,1562,926,701,461,320,205 |
| `E-S4.price.pairs_9_plus` | 446 |
| `E-S4.price.longest_pairs` | 30 |

**E-S5.** Run 5: 1,000 games with the 80% coin then a fair coin.

| key | value |
|---|---|
| `E-S5.two_coins.games` | 1000 |
| `E-S5.two_coins.heads_first` | 807 |

**E-S6.** Run 6: 200,000 tosses of the sticky coin: the deciding pairs that follow a pair of heads or of tails, and Samuelson's rule on the same tosses.

| key | value |
|---|---|
| `E-S6.sticky.tosses` | 200000 |
| `E-S6.sticky.heads_percent` | 50.0 |
| `E-S6.sticky.after_hh.ht` | 6306 |
| `E-S6.sticky.after_hh.th` | 1615 |
| `E-S6.sticky.after_hh.heads_first_percent` | 79.6 |
| `E-S6.sticky.after_tt.ht` | 1681 |
| `E-S6.sticky.after_tt.th` | 6442 |
| `E-S6.sticky.after_heads_only.heads_first` | 7847 |
| `E-S6.sticky.after_heads_only.tails_first` | 8043 |
| `E-S6.sticky.after_heads_only.heads_first_percent` | 49.4 |

**E-S7.** Run 7: 24,000 tosses of the 80% coin in blocks of four: the dice made (two heads), by pattern HHTT, HTHT, HTTH, THHT, THTH, TTHH.

| key | value |
|---|---|
| `E-S7.coin_die.tosses` | 24000 |
| `E-S7.coin_die.dice` | 949 |
| `E-S7.coin_die.faces` | 166,161,136,160,167,159 |
| `E-S7.coin_die.tosses_per_die` | 25.3 |

**E-S8.** Run 8: 6,000 rolls of the loaded die: its faces; in threes, the dice made, by order; in twos, which roll was bigger.

| key | value |
|---|---|
| `E-S8.loaded.rolls` | 6000 |
| `E-S8.loaded.faces` | 318,283,579,607,1237,2976 |
| `E-S8.three_rolls.dice` | 650 |
| `E-S8.three_rolls.orders` | 119,108,103,105,112,103 |
| `E-S8.three_rolls.rolls_per_die` | 9.2 |
| `E-S8.two_rolls.first_bigger` | 1021 |
| `E-S8.two_rolls.second_bigger` | 1020 |
| `E-S8.two_rolls.ties` | 959 |

**E-S9.** Run 9: 100,000 tosses of the 80% coin: the fair tosses each procedure gets out, and how many of them are heads.

| key | value |
|---|---|
| `E-S9.squeeze.tosses` | 100000 |
| `E-S9.squeeze.heads` | 80042 |
| `E-S9.squeeze.von_neumann.fair_tosses` | 16114 |
| `E-S9.squeeze.von_neumann.heads` | 8047 |
| `E-S9.squeeze.von_neumann.per_hundred` | 16.1 |
| `E-S9.squeeze.peres2.fair_tosses` | 28734 |
| `E-S9.squeeze.peres2.heads` | 14330 |
| `E-S9.squeeze.peres2.per_hundred` | 28.7 |
| `E-S9.squeeze.peres3.fair_tosses` | 38754 |
| `E-S9.squeeze.peres3.heads` | 19332 |
| `E-S9.squeeze.peres3.per_hundred` | 38.8 |
| `E-S9.squeeze.peres_all.fair_tosses` | 69624 |
| `E-S9.squeeze.peres_all.heads` | 34762 |
| `E-S9.squeeze.peres_all.per_hundred` | 69.6 |
| `E-S9.squeeze.elias16.fair_tosses` | 49439 |
| `E-S9.squeeze.elias16.heads` | 24799 |
| `E-S9.squeeze.elias16.per_hundred` | 49.4 |

**E-C1.** Cited: von Neumann's note (S1).

| key | value |
|---|---|
| `E-C1.von_neumann.talk_year` | 1949 |
| `E-C1.von_neumann.printed_year` | 1951 |
| `E-C1.von_neumann.pages` | 36-38 |
| `E-C1.von_neumann.series_number` | 12 |
| `E-C1.von_neumann.efficiency_percent` | 25 |

**E-C2.** Cited: the later papers (S2 to S9), and the size of each group of rotations in Dijkstra's roulette, (2^37 − 2)/37.

| key | value |
|---|---|
| `E-C2.samuelson.year` | 1968 |
| `E-C2.hoeffding_simons.year` | 1970 |
| `E-C2.elias.year` | 1972 |
| `E-C2.blum.year` | 1986 |
| `E-C2.dijkstra.year` | 1990 |
| `E-C2.dijkstra.outcomes` | 37 |
| `E-C2.peres.year` | 1992 |
| `E-C2.juels.year` | 2000 |
| `E-C2.dijkstra.patterns_per_outcome` | 3714566310 |

**E-C3.** Cited: real coins (S10 to S13).

| key | value |
|---|---|
| `E-C3.gelman_nolan.year` | 2002 |
| `E-C3.diaconis.year` | 2007 |
| `E-C3.diaconis.same_side_percent` | 51 |
| `E-C3.bartos.year` | 2025 |
| `E-C3.bartos.flips` | 350757 |
| `E-C3.bartos.same_side` | 0.508 |
| `E-C3.pennies.spins` | 5520 |
| `E-C3.pennies.heads` | 1878 |
| `E-C3.pennies.heads_percent` | 34 |

**E-C4.** Cited: the Intel generator (S14).

| key | value |
|---|---|
| `E-C4.intel.year` | 1999 |
| `E-C4.intel.raw_samples_per_bit` | 6 |
