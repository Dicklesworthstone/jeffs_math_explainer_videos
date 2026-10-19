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
| `R-F14.estimate_mean` | 4 | 03_4 "around four", 03_2, 03_7 "above four"; the gold line; the 03_7 card "allow negatives: 2 × yes − 10 averages 4" | the distribution's mean estimate, checked equal to the copiers |
| `R-F14.clamped_mean`, `R-F14.clamped_mean_2dp` | 584948368349/137438953472, 4.26 | 03_7 "push the average above four"; the note and card "4.26" | the mean of max(estimate, 0) |
| `R-F14.answer_variance` | 3/16 | 03_8 "Every answer wobbles the same"; TeX 3/16 × 20 | p(1 − p) at 3/4 and at 1/4, checked equal |
| `R-F7.var_yes`, `R-F7.var_estimate`, `R-F7.sd_estimate`, `R-F14.sd_rounded` | 15/4, 15, 3.87, 4 | TeX 15/4, 15/4 × 4 = 15, √15 ≈ 3.87; the bracket; 03_8 "miss by about four people" | the distribution's variance, checked to be 20 answers of 3/16, times 4 |
| `R-F7.coin_variance_per_person` | 3/4 | TeX 3/(4n) | the share's coin term |
| `R-F8.cost`, `R-F8.cost_2dp`, `R-F14.cost_rounded` | 91/16, 5.69, 6 | 03_10 "almost six times as many people"; TeX 91/16 ≈ 5.69; the card (from 03_9 "Sample twenty") | independent draws at p = 1/5, direct answers assumed truthful; checked just under 6 |
| `R-F1.copied_share` | 1/5 | 03_10 "If one in five copied"; the card's "p = 1/5" (from 03_9) | as before |

## Chapter 03: claims

| claim | where | evidence |
|---|---|---|
| The estimate is right on average for any class, and only if negatives stay | 03_4, 03_7 and the card | `the_estimate_is_right_on_average_only_if_negatives_stay`: for every number of copiers 0 to 20 the mean is the truth and the clamped mean is above it |
| A fixed class's scatter is the coins' alone, the same whoever copied | 03_8 | the variance is 3n/4 for every number of copiers (`the_estimate_is_unbiased_with_variance_three_quarters_per_person`); each answer's 3/16 for both truths |
| A sample adds who gets picked | 03_9, 03_10 | `sampling_variance` = `direct_variance` + 3/(4n) for independent draws (`the_coins_add_three_quarters_over_n_to_the_sampling_variance`); without replacement from the whole class that term is 0 |
| The eight classes are real, not chosen | 03_1, 03_2 and the card | seed 3, the first eight draws; the oracle replays them with NumPy |

## Chapter 04: numbers

| key | value | where | how it is obtained |
|---|---|---|---|
| `R-F9.asks`, `R-F9.outcomes` | 3, 64 | 04_1 "three times"; 04_2 "sixty-four"; TeX 4 × 4 × 4 = 64; the 04_2 card | 4^asks: every outcome of every ask's two coins |
| `R-F9.by_yeses.copied`, `R-F9.by_yeses.didnt` | 1,9,27,27; 27,27,9,1 | the two grids (04_4, 04_5, 04_8) | `model::repeated_answers` runs the rule on every outcome, and the scene tallies its drawn squares against it; the oracle uses `itertools.product` over the written-out rule |
| `R-F9.ratio.all_yes`, `R-F9.all_yes.copied`, `R-F9.all_yes.didnt` | 27; 27, 1 | 04_4 "Twenty-seven", "in twenty-seven outcomes, anyone else in one"; "3 yes: 27 of 64", "3 yes: 1 of 64"; TeX 27/1 = 27; 04_10 "fresh coins: 27" | the all-yes pattern's counts and their ratio |
| `R-F9.all_no.didnt`, `R-F9.all_no.copied` | 27, 1 | 04_5 "Three noes do the same the other way"; "3 no: 1 of 64", "3 no: 27 of 64" | checked to be the all-yes counts reversed |
| `R-F5.worst_ratio`, `R-F9.ratios`, `R-F9.worst_ratio_by_asks` | 3; 1/27,1/3,3,27; 3,9,27,81,243 | 04_5 "multiply it by three"; TeX 27 = 3 × 3 × 3; the 04_5 card | each pattern's ratio by its number of yeses; the largest over all patterns after 1 to 5 fresh asks |
| `R-F1.copied_share`, `R-F1.copied`, `R-F11.others` | 1/5, 4, 16 | 04_6 "one in five copied", "Those four", "the sixteen others"; the grid names "4 copied", "16 didn't copy"; the 04_7 card "4 of 20" | as before |
| `R-F15.all_yes.copied`, `R-F15.all_yes.didnt`, `R-F15.all_yes` | 108, 16, 124 | 04_6 "a hundred and eight", "others sixteen"; TeX 4 × 27 = 108, 16 × 1 = 16, 108/(108 + 16) | every student with every outcome: the class's three-yes outcomes (the oracle counts seat by seat) |
| `R-F9.posterior.all_yes`, `R-F9.percent.all_yes` | 27/31, 87 | 04_7 "twenty-seven in thirty-one"; TeX = 27/31; the 04_7 card | 108 of 124, checked against the closed form `model::posterior` |
| `R-F11.posterior.yes` | 3/7 | 04_7 "After one, three in seven"; "after one yes: 3/7" | chapter 02's count |
| `R-F15.posterior.all_yes_at_half`, `R-F15.posterior.all_yes_none_copied` | 27/28, 0 | the 04_7 card | the same three yeses from an even prior, and in a class where nobody copied |
| `R-F9.majority_right`, `R-F9.majority.outcomes`, `R-F9.majority_right.didnt`, `R-F9.percent.majority_right` | 27/32, 54, 27/32, 84 | 04_8 "twenty-seven times in thirty-two, whichever the truth"; "majority right: 54 of 64" under each grid; TeX 54/64 = 27/32 | at least two yeses for someone who copied, at least two noes for anyone else, counted for each truth and checked equal |
| `R-F10.memoized.outcomes`, `R-F10.memoized.all_yes.copied`, `R-F10.memoized.all_yes.didnt`, `R-F10.memoized.ratio` | 4; 3, 1; 3 | 04_9 the columns; 04_10 "three times as likely"; "3 yes: 3 of 4", "3 yes: 1 of 4"; TeX 3/1 = 3 | one flip's four outcomes, its answer repeated; only all-yes and all-no happen (both sides check) |
| `R-F5.ratio.yes` | 3 | 04_10 "the same as one" | as before |
| `R-F10.memoized.posterior` | 3/7 | not shown | the class's memoized three yeses leave one yes's chance (checked) |

## Chapter 04: claims

| claim | where | evidence |
|---|---|---|
| Fresh answers about one unchanged secret multiply the evidence: three yeses are 27 times as likely from someone who copied | 04_4, 04_5 | `R-F9`; the oracle checks that every pattern's ratio is 3^(2y − k) whatever the order, and that each fresh ask multiplies the worst ratio by three |
| The chance after three yeses is the class's | 04_6, 04_7 and the card | counted over the class (108 of 124) and by the closed form; 27/28 from an even prior, 0 if nobody copied |
| A majority of three answers is right 27 times in 32 for either truth | 04_8 | counted for each truth, 54 of 64 each |
| Repeating one memoized answer adds no evidence | 04_9, 04_10 | `R-F10`: only all-yes and all-no happen, and the all-yes ratio is 3 for every ask count from 1 to 10 |
| Scope: one secret that never changes, under this rule | 04_11 and the 04_9 card | no claim about repeated surveys in general, other questions, or RAPPOR's full design. RAPPOR §1 states its own conditions for memoization (a true value that does not change, or changes in an uncorrelated way); this video does not test them |

## Chapter 05: numbers

| key | value | where | how it is obtained |
|---|---|---|---|
| `R-F16.ours.truth`, `R-F16.ours.truth_slices`, `R-F16.ours.yes.copied`, `R-F16.ours.yes.didnt`, `R-F16.ours.ratio`, `R-F16.ours.coin_variance` | 1/2; 2; 6, 2; 3; 3/4 | 05_1 "Ours sits at one half"; the spinner's two green slices; "yes: 6 of 8", "yes: 2 of 8"; the table's 1/2 row | `Channel` at 1/2, which the facts check is `Channel::two_coins` |
| `R-F16.slices`, `R-F16.outcomes` | 4, 8 | 05_2 "a spinner of four slices"; 05_4 "the eight outcomes"; the 05_2 card | a slice and the second coin's face, 4 × 2 |
| `R-F16.up.truth`, `R-F16.up.truth_slices` | 3/4, 3 | 05_2 "three quarters", "three saying tell the truth"; 05_3; the dial and the table's 3/4 row | `Channel::new(3/4)` |
| `R-F16.up.yes.copied`, `R-F16.up.yes.didnt`, `R-F16.up.ratio` | 7, 1, 7 | 05_4 "Seven", "yes in seven, anyone else in one"; "yes: 7 of 8", "yes: 1 of 8"; the table; the 05_4 card | `Channel::p_said` × 8 and `Channel::ratio`, both answers checked equal and equal to (1 + t)/(1 − t); the scene's squares are `model::answer` with the slice as the first coin; the oracle counts the spinner's 8 outcomes with its own inline rule (a truthful slice gives the truth, any other the second coin's face, heads for yes), equivalent to the rule table but not read from it |
| `R-F16.up.coin_variance` | 7/36 | the table's 3/4 row | `Channel::coin_variance`, checked against (1 − t²)/(4t²); the oracle takes the variance of one person's share, (answer − (1 − t)/2)/t, over the 8 outcomes for each truth |
| `R-F1.class`, `R-F16.up.class_variance`, `R-F16.up.miss_2dp`, `R-F16.up.miss_rounded`, `R-F16.ours.miss_2dp`, `R-F16.ours.miss_rounded` | 20; 35/9, 1.97, 2; 3.87, 4 | 05_6 "Our class of twenty would miss by about two people, not four"; TeX √(20 × 7/36) ≈ 1.97 and √(20 × 3/4) ≈ 3.87 | the fixed class's count variance, 20 times one person's; the miss is its square root (to two places from the float root) and the nearest whole number to it in exact arithmetic (Rust: the largest m with (m − 1/2)² ≤ v; the oracle: an integer square root of 4pq). At 1/2 checked equal to chapter 03's `R-F7.sd_estimate` 3.87 and `R-F14.sd_rounded` 4 |
| `R-F16.down.truth`, `R-F16.down.truth_slices`, `R-F16.down.yes.copied`, `R-F16.down.yes.didnt`, `R-F16.down.ratio`, `R-F16.down.coin_variance` | 1/4; 1; 5, 3; 5/3; 15/4 | 05_7 "at a quarter"; "yes: 5 of 8", "yes: 3 of 8"; the table's 1/4 row | as above |
| `R-F16.down.ratio_under`, `R-F16.down.class_variance`, `R-F16.down.miss_2dp`, `R-F16.down.miss_rounded` | 2; 75, 8.66, 9 | 05_7 "under twice as likely", "the class would miss by about nine"; TeX √(20 × 15/4) ≈ 8.66 | the whole number just above 5/3; the miss as above |
| `R-F16.direct.class_variance`, `.miss_2dp`, `.miss_rounded`; `R-F16.pure.*` the same | 0, 0.00, 0; none | not shown (the table's 0 and "no estimate" stand for them) | as above |
| `R-F16.up.wobble_vs_ours`, `R-F16.up.wobble_vs_ours_nearest_quarter`, `R-F16.down.wobble_vs_ours` | 7/27, 1/4, 5 | not shown or said (they compare per-person variances, not misses; the voice speaks misses, Mail 4097) | kept as exact checks: 7/36 over 3/4, its nearest quarter, 15/4 over 3/4 |
| `R-F16.direct.truth`, `.truth_slices`, `.yes.copied`, `.yes.didnt`, `.ratio`, `.coin_variance` | 1; 4; 8, 0; unbounded; 0 | 05_8 "the direct question, no wobble and no limit"; "yes: 8 of 8", "yes: 0 of 8"; the table's 1 row "no bound", 0; the 05_8 card | `Channel::new(1)`: a yes never comes from someone who didn't copy, so `ratio` is `None` |
| `R-F16.pure.truth`, `.truth_slices`, `.yes.copied`, `.yes.didnt`, `.ratio`, `.coin_variance` | 0; 0; 4, 4; 1; none | 05_9 "the second coin answers for everyone. Nothing is revealed, and nothing can be counted"; "yes: 4 of 8" twice; the table's 0 row, 1 and "no estimate"; the 05_8 card | `Channel::new(0)`: every answer as likely from everyone, and `coin_variance` and `estimate` are `None` (nothing to divide by; `R-X1.pure_coin.*`) |
| `R-F16.class.count_variance`, `R-F16.class.share_variance` | 15, 3/80 | the 05_6 card ("count variance 20 × it", "share variance it/20") and the 1/2 miss line, √(20 × 3/4) | n times one person's 3/4 for a fixed class's count (checked equal to chapter 03's `R-F7.var_estimate`), 1/n of it for its share |
| `R-F2.coins` | 2 | 05_9 "the second coin" | as before |

## Chapter 05: claims

| claim | where | evidence |
|---|---|---|
| A yes and a no carry the same ratio at every setting, (1 + t)/(1 − t) | 05_4, 05_5 and the card | the facts refuse to build otherwise; `the_formulas_on_screen_are_the_channels` checks every twelfth of the dial; the oracle counts both answers at every setting |
| Turning the dial up makes every answer say more and the count sharper | 05_5, 05_6, 05_7 | the ratio rises and the variance falls at every step of twelfths between the edges (the same test); the class's misses, about 9, 4 and 2 at 1/4, 1/2 and 3/4 (`R-F16.*.miss_*`) |
| The misses the voice speaks are 03_8's, the fixed class's; the table's variance is one person's; the count's variance is n times it, the share's 1/n of it, and a sample adds p(1 − p)/n | 05_6, 05_7 and the card | `R-F16.class.*` and the 1/2 miss checked against chapter 03; chapter 03's `sampling_variance`. No ratio of per-person variances is spoken as a miss (TanElm, Mail 4095). None of these is a chance that anyone copied |
| At 1 nothing bounds a yes; at 0 nothing can be counted | 05_8, 05_9 and the card | `R-F16.direct.*`, `R-F16.pure.*`, `R-X1.*`; both edges are counted on screen (0 of 8, 4 of 8 and 4 of 8) |
| The promise holds for one answer each, everyone following the rule with fair, private coins | 05_10 and the card | the brief's Assumption 1 (compliance) stated on the card; chapter 04's `R-F9` (fresh coins multiply the ratio) and `R-F10` (a memoized answer does not); no claim about real surveys' honesty or RAPPOR's guarantees |

## Planned chapters (computed now)

| keys | values | for |
|---|---|---|
| `R-F6.*` | among the 7 yeses on average 3 copied (3/7, 43%); among the 13 noes 1 (1/13, 8%); the class's share 1/5 (20%) | chapter 02 (above) counts them over outcomes; these are the same shares from chapter 01's average tally, checked against Bayes |
| `R-F7.*` | 4²⁰ = 1,099,511,627,776 coin outcomes; yes count mean 7, variance 15/4; estimate variance 15 (3/4 a person), standard deviation 3.87; exactly 4 with chance 14028881103/68719476736 (20%); below zero with chance 25556466249/274877906944 (9%) | 03, the price of the coin |
| `R-F8.*` | independent draws (with replacement, or from a large population) at a share of 1/5: one person's variance 4/25 asked directly, 91/100 with the coins, 91/16 ≈ 5.69 times in total; the coins' added 3/4 alone is 75/16 times (`R-F8.added_cost`); that term is Warner's P(1 − P)/(2P − 1)² at P = 3/4. Without replacement from N = n the sampling part is 0 (the fixed class) | 03, sampling from a population |
| `R-F9.*`, `R-F10.*` | asked 3 times, 64 outcomes; by yeses 1, 9, 27, 27 (copied) against 27, 27, 9, 1; ratios 1/27, 1/3, 3, 27; three yeses leave 27/31 (87%); a majority is right 27/32 (84%); the worst ratio after 1 to 5 fresh asks 3, 9, 27, 81, 243; a memoized answer stays at 3 | 04, asking again |
| `R-X3.rappor.no_after_75_of_100` | 1.39e-24 | RAPPOR §1's example, **pinned** |
