# Fact sheet — How to Learn a Secret About a Crowd Without Learning Each Person's Answer

Every number chapter 01 shows or says, and every claim it makes, with the fact
key `survey facts` prints, its value, and where it is checked. `src/facts.rs`
computes each from `src/model.rs`; `oracle.py` recomputes each by its own
method; `./x randomized_response facts-check` compares them key by key.
Pinned rows also match a literal from the named primary source.

## Chapter 01: numbers

| key | value | where | how it is obtained |
|---|---|---|---|
| `R-F1.class` | 20 | 01_1 "a class of twenty"; the grid | the invented class (`model::CLASS_SIZE`) |
| `R-F1.rows`, `R-F1.columns` | 4, 5 | the grid | `CLASS_COLUMNS`; rows by division |
| `R-F1.copied` | 4 | 01_2 "Four of them did", 01_5 "our four"; the rings | `model::COPIERS` (seats 2, 6, 13, 17) |
| `R-F2.coins` | 2 | 01_4 "a second coin" | two fair coins: log₂ of the 4 outcomes |
| `R-F2.coin_outcomes` | 4 | 01_5 card; 01_11 grid columns | `COIN_PAIRS` |
| `R-F2.yes_outcomes.copied`, `.didnt` | 3, 1 | 01_11 "3 of 4", "1 of 4" | the rule run on each coin outcome |
| `R-F2.p_yes.copied`, `.didnt` | 3/4, 1/4 | 01_11 card | counted; **pinned**: Dwork and Roth, claim 3.5 |
| `R-F3.truthful`, `R-F3.truthful_share` | 10, 1/2 | 01_5 "half the class"; 01_10 "the half who told the truth" | the class on average (`Class::average_coins`), tallied |
| `R-F3.truthful.yes`, `.no` | 2, 8 | 01_5 "Two of our four say yes"; the upper places' counts | tallied |
| `R-F3.forced`, `.forced.yes`, `.forced.no` | 10, 5, 5 | 01_6 "The other ten", "Five yes, five no"; the lower places' counts | tallied |
| `R-F3.forced.yes.copied`, `.forced.no.copied` | 1, 1 | the rings that return at 01_10 | tallied |
| `R-F3.yes`, `R-F3.no` | 7, 13 | 01_7 "Seven yes, thirteen no"; the totals | tallied; equal to 20 × `R-F2.yes_share` (7/20, **pinned** to Dwork and Roth §2.3's 1/4 + p/2) |
| `R-F3.counts_seen` | 2 | 01_8 "those two numbers" | one count per answer |
| `R-F4.forced_yes`, `R-F4.forced_yes_share` | 5, 1/4 | 01_9 "that's five, a quarter of the class"; TeX 20/4 = 5 | `model::undo`: n (1 − t)/2 at t = 1/2 |
| `R-F4.left` | 2 | 01_9 "Two are left", 01_10 "Those two"; TeX 7 − 5 = 2 | `undo` |
| `R-F4.factor` | 2 | 01_10 "double them"; "× 2" | 1/t |
| `R-F4.estimate` | 4 | 01_8 "get back to four", 01_10 "Four."; TeX 2 × 2 = 4 | `undo`, checked equal to the dial's 2 · yes − n/2 |
| `R-F4.matches_truth` | true | the rings returning at 01_10 | the estimate equals the copiers on the average counts |
| `R-X2.estimate.no_yes`, `.all_yes` | −10, 30 | 01_10 code card | `undo(0, 20)`, `undo(20, 20)` |
| `R-F5.ratio.yes`, `R-F5.ratio.no` | 3, 3 | 01_11 "three times as often"; "× 3"; the card | P(said \| its truth) / P(said \| the other); **pinned**: claim 3.5 |
| `R-F5.worst_ratio`, `R-F5.epsilon` | 3, 1.0986 | 01_11 card "never more" | the largest ratio; ln 3 **pinned** to claim 3.5's (ln 3, 0) |

## Chapter 01: claims

| claim | where | evidence |
|---|---|---|
| The rule: heads, the truth; tails, the second coin answers, heads yes, tails no | 01_3, 01_4 | `model::route`, `model::answer`; Dwork and Roth §3.2 with the first coin's faces swapped |
| "On average" means every coin outcome equally often, and a real class strays | 01_5 card, 01_9 | `Class::average_coins` refuses any class whose groups are not multiples of 4 (`R-X4`); the scatter is `R-F7` |
| Taking away a quarter of the class and doubling gives the count on average, for any class | 01_9, 01_10 | `the_estimate_is_unbiased_with_variance_three_quarters_per_person` over every number of copiers 0 to 20; the oracle's DP repeats it |
| A yes is three times as likely from someone who copied, a no three times as likely from someone who didn't, never more | 01_11 and its card | `every_answer_is_at_most_three_times_likelier_and_the_dial_has_two_edges`: the edges are the direct question (no bound, `R-X1.direct.worst_ratio` unbounded) and a pure coin (no information, no estimate, `R-X1.pure_coin.*`) |
| No answer is a confession | 01_11 | both answers happen under both truths (every ratio finite and positive); this is not "no information": the same line says three times |

## Planned chapters (computed now)

| keys | values | for |
|---|---|---|
| `R-F6.*` | among the 7 yeses on average 3 copied (3/7, 43%); among the 13 noes 1 (1/13, 8%); the class's share 1/5 (20%) | 02, what a yes says; counted and checked against Bayes (`model::posterior`) |
| `R-F7.*` | 4²⁰ = 1,099,511,627,776 coin outcomes; yes count mean 7, variance 15/4; estimate variance 15 (3/4 a person), standard deviation 3.87; exactly 4 with chance 14028881103/68719476736 (20%); below zero with chance 25556466249/274877906944 (9%) | 03, the price of the coin |
| `R-F8.*` | independent draws (with replacement, or from a large population) at a share of 1/5: one person's variance 4/25 asked directly, 91/100 with the coins, 91/16 ≈ 5.69 times in total; the coins' added 3/4 alone is 75/16 times (`R-F8.added_cost`); that term is Warner's P(1 − P)/(2P − 1)² at P = 3/4. Without replacement from N = n the sampling part is 0 (the fixed class) | 03, sampling from a population |
| `R-F9.*`, `R-F10.*` | asked 3 times, 64 outcomes; by yeses 1, 9, 27, 27 (copied) against 27, 27, 9, 1; ratios 1/27, 1/3, 3, 27; three yeses leave 27/31 (87%); a majority is right 27/32 (84%); the worst ratio after 1 to 5 fresh asks 3, 9, 27, 81, 243; a memoized answer stays at 3 | 04, asking again |
| `R-X3.rappor.no_after_75_of_100` | 1.39e-24 | RAPPOR §1's example, **pinned** |
