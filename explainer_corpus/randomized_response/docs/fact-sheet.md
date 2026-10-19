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

## Chapter 02: numbers

| key | value | where | how it is obtained |
|---|---|---|---|
| `R-F2.coin_outcomes`, `R-F1.class` | 4, 20 | 02_1 "all four coin outcomes", "Twenty students" | as in chapter 01 |
| `R-F11.class_outcomes` | 80 | 02_1 "eighty outcomes"; TeX 20 × 4 = 80 | every student with every coin outcome (`model::class_outcomes`), counted |
| `R-F2.yes_outcomes.copied`, `.didnt` | 3, 1 | 02_2 "three of their four", "in one"; "yes: 3 of 4", "yes: 1 of 4"; 02_7 "3 each", "1 each" | the rule on each outcome |
| `R-F5.ratio.yes`, `R-F5.ratio.no` | 3, 3 | 02_2 "Three times as likely", 02_3 "three times again", 02_7; "× 3" | as in chapter 01; **pinned**: claim 3.5 |
| `R-F11.yes.copied`, `.yes.didnt`, `.yes` | 12, 16, 28 | 02_5 "twelve", "sixteen"; 02_6 "Twelve of twenty-eight"; TeX 12/28 | counted over the class's outcomes, checked against the drawn squares |
| `R-F11.others`, `R-F11.outnumber` | 16, 4 | 02_5 "the sixteen others"; 02_7 "four to one"; "16 didn't copy" | the class |
| `R-F11.posterior.yes` | 3/7 | 02_6 "Three in seven", 02_9; TeX = 3/7; the chance line | 12/28, checked equal to the average tally (`R-F6`) and to Bayes (`model::posterior`) |
| `R-F2.p_yes.copied` | 3/4 | 02_6 "not three in four"; "3/4 is a copier's chance of saying yes" | the tempting answer, a likelihood, not a chance of having copied |
| `R-F11.average.yes`, `.average.yes.copied` | 7, 3 | 02_6 "three of the seven yeses"; the note beside 3/7 | a quarter of 28 and 12: chapter 01's class on average, checked against its tally |
| `R-F11.no.copied`, `.no.didnt`, `.no`, `.posterior.no` | 4, 48, 52, 1/13 | 02_8 "Four of the fifty-two", "one in thirteen"; TeX 4/52 = 1/13; the chance line | counted, checked as for a yes |
| `R-F11.average.no`, `.average.no.copied` | 13, 1 | the storyboard's notes (not on screen) | a quarter of 52 and 4 |
| `R-F1.copied_share` | 1/5 | 02_9 "one in five"; the gold mark, "the class: 4 of 20" | 4 of 20 |
| `R-F12.odds.prior`, `.odds.yes`, `.odds.no` | 1/4, 3/4, 1/12 | 02_10 card "1 : 4 becomes 3 : 4 or 1 : 12" (their chances are the line's marks) | odds of the chances, checked to be the prior's times 3 and over 3 |
| `R-F12.half_class`, `R-F12.posterior.yes_at_half` | 10, 3/4 | 02_6 card "It would be 3/4 only if half the class, 10, had copied" | a class of 20 with 10 who copied, counted |
| `R-X5.posterior.yes_none_copied`, `.no_all_copied` | 0, 1 | 02_10 card "If nobody copied, a yes would mean 0" | the edges of "depends on the class": classes where nobody, or everybody, copied |
| `R-F5.worst_ratio` | 3 | 02_10 "The coin fixes the three times" | as in chapter 01 |

## Chapter 02: claims

| claim | where | evidence |
|---|---|---|
| The ratio of three belongs to the coin, the chance to the class | 02_10 and its card | `counting_every_students_outcomes_gives_the_posteriors`: for every class of 20 (0 to 20 copiers) the counted share equals the closed form at that prior; the ratio stays 3 throughout |
| 3/7, not 3/4 | 02_6 and its card | 3/4 = P(yes \| copied) is read off one copier's four outcomes; 3/7 = P(copied \| yes) needs the whole class's outcomes; the two are equal only at a prior of 1/2 (`R-F12.posterior.yes_at_half`) |
| Deniable, but not meaningless | 02_11 | a yes leaves 4/7 that the giver did not copy, yet moves the chance from 1/5 to 3/7 |
| Privacy and accuracy are separate questions | 02_11 | the factor of three is per answer (`R-F5`); the count's scatter is `R-F7`, `R-F8` (planned chapter 03) |

## Chapter 03: numbers

| key | value | where | how it is obtained |
|---|---|---|---|
| `R-F13.seed`, `R-F13.runs` | 3, 8 | 03_1 "eight times"; the 03_2 card "seed 3" | `facts::SEED`, `facts::RUNS`: the chapter's number, fixed before any draw was seen |
| `R-F13.yes`, `R-F13.estimates` | 3,5,12,10,9,7,4,8; −4,0,14,10,8,4,−2,6 | the eight rows and their columns | one raw draw a class (`Data::new(3).raw()`), two bits a student (`model::coins_from_bits`), tallied; the oracle replays NumPy's `PCG64DXSM(3).random_raw()` |
| `R-F13.lowest`, `.highest`, `.on_four`, `.below_zero` | −4, 14, 1, 2 | 03_2 "from minus four to fourteen", "Only one lands on four" | the eight estimates |
| `R-F7.outcomes`, `R-F14.outcomes_trillions` | 1,099,511,627,776; 1 | 03_3 "Four to the twentieth, over a trillion"; TeX 4²⁰ = …; the 03_6 card | 4²⁰ |
| `R-F7.p_exactly_right`, `R-F7.percent.exactly_right`, `R-F14.one_in.exactly` | 14028881103/68719476736, 20, 5 | 03_6 "About one class in five"; "exactly 4: about 1 class in 5 (20%)" | the exact distribution at 7 yeses; one in round(1/p) |
| `R-F7.p_negative`, `R-F7.percent.negative`, `R-F14.one_in.negative` | 25556466249/274877906944, 9, 11 | 03_6 "about one in eleven goes below zero"; "below 0 (under 5 yeses): about 1 in 11 (9%)" | under 5 yeses |
| `R-F4.forced_yes` | 5 | the 03_6 note "under 5 yeses"; 03_7 "fewer yeses than the second coin alone gives on average" | the second coin's yeses on average |
| `R-F14.estimate_mean` | 4 | 03_4 "around four", 03_2, 03_7 "above four"; the gold line | the distribution's mean estimate, checked equal to the copiers |
| `R-F14.clamped_mean`, `R-F14.clamped_mean_2dp` | 584948368349/137438953472, 4.26 | 03_7 "push the average above four"; the note and card "4.26" | the mean of max(estimate, 0) |
| `R-F14.answer_variance` | 3/16 | 03_8 "Every answer wobbles the same"; TeX 3/16 × 20 | p(1 − p) at 3/4 and at 1/4, checked equal |
| `R-F7.var_yes`, `R-F7.var_estimate`, `R-F7.sd_estimate`, `R-F14.sd_rounded` | 15/4, 15, 3.87, 4 | TeX 15/4, 15/4 × 4 = 15, √15 ≈ 3.87; the bracket; 03_8 "miss by about four people" | the distribution's variance, checked to be 20 answers of 3/16, times 4 |
| `R-F7.coin_variance_per_person` | 3/4 | TeX 3/(4n) | the share's coin term |
| `R-F8.cost`, `R-F8.cost_2dp`, `R-F14.cost_rounded` | 91/16, 5.69, 6 | 03_10 "almost six times as many people"; TeX 91/16 ≈ 5.69; the card | independent draws at p = 1/5, direct answers assumed truthful; checked just under 6 |
| `R-F1.copied_share` | 1/5 | 03_10 "If one in five copied"; the card's "p = 1/5" | as before |

## Chapter 03: claims

| claim | where | evidence |
|---|---|---|
| The estimate is right on average for any class, and only if negatives stay | 03_4, 03_7 and the card | `the_estimate_is_right_on_average_only_if_negatives_stay`: for every number of copiers 0 to 20 the mean is the truth and the clamped mean is above it |
| A fixed class's scatter is the coins' alone, the same whoever copied | 03_8 | the variance is 3n/4 for every number of copiers (`the_estimate_is_unbiased_with_variance_three_quarters_per_person`); each answer's 3/16 for both truths |
| A sample adds who gets picked | 03_9, 03_10 | `sampling_variance` = `direct_variance` + 3/(4n) for independent draws (`the_coins_add_three_quarters_over_n_to_the_sampling_variance`); without replacement from the whole class that term is 0 |
| The eight classes are real, not chosen | 03_1, 03_2 and the card | seed 3, the first eight draws; the oracle replays them with NumPy |

## Planned chapters (computed now)

| keys | values | for |
|---|---|---|
| `R-F6.*` | among the 7 yeses on average 3 copied (3/7, 43%); among the 13 noes 1 (1/13, 8%); the class's share 1/5 (20%) | chapter 02 (above) counts them over outcomes; these are the same shares from chapter 01's average tally, checked against Bayes |
| `R-F7.*` | 4²⁰ = 1,099,511,627,776 coin outcomes; yes count mean 7, variance 15/4; estimate variance 15 (3/4 a person), standard deviation 3.87; exactly 4 with chance 14028881103/68719476736 (20%); below zero with chance 25556466249/274877906944 (9%) | 03, the price of the coin |
| `R-F8.*` | independent draws (with replacement, or from a large population) at a share of 1/5: one person's variance 4/25 asked directly, 91/100 with the coins, 91/16 ≈ 5.69 times in total; the coins' added 3/4 alone is 75/16 times (`R-F8.added_cost`); that term is Warner's P(1 − P)/(2P − 1)² at P = 3/4. Without replacement from N = n the sampling part is 0 (the fixed class) | 03, sampling from a population |
| `R-F9.*`, `R-F10.*` | asked 3 times, 64 outcomes; by yeses 1, 9, 27, 27 (copied) against 27, 27, 9, 1; ratios 1/27, 1/3, 3, 27; three yeses leave 27/31 (87%); a majority is right 27/32 (84%); the worst ratio after 1 to 5 fresh asks 3, 9, 27, 81, 243; a memoized answer stays at 3 | 04, asking again |
| `R-X3.rappor.no_after_75_of_100` | 1.39e-24 | RAPPOR §1's example, **pinned** |
