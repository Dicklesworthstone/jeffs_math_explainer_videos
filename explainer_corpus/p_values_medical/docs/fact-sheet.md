# Fact sheet — Why P-Values Confuse Even Doctors

Every number the video shows or says, with its key (`pvalues facts`,
`oracle.py`), how it is obtained and where it appears; then the sources, what
was actually read, the competing explanations of the survey, the exact
versions of the simplifications and the terms. Chapter 01 is built; the rows
for later chapters are computed and oracle-checked now so the continuation
starts from checked numbers.

## M-C · cited (the survey)

| Key | Value | Source | Where |
|---|---|---|---|
| M-C1.survey.published_year | 2007 | S1 (JAMA 2007;298(9), published 5 September 2007) | 01_1 said ("published in two thousand seven") and on the tag |
| M-C1.survey.published | 2007-09-05 | S1's publication date | not shown |
| M-C1.survey.collected_year / .collected_months | 2006 / February to July | S1 methods: data collected February–July 2006 | 01_1 tag ("data collected February to July 2006"); never spoken as the survey's year |
| M-C1.survey.respondents | 277 | S1 abstract: "completed by 277 of 367 residents (75.5%)" | 01_1 said; 277 lit dots |
| M-C1.survey.residents | 367 | S1 abstract | 01_1: 367 dots |
| M-C1.survey.response_percent | 75.5 | computed 277/367, pinned to S1's printed 75.5% | 01_1 under the dots |
| M-C1.survey.programs | 11 | S1 abstract: "in 11 residency programs" | 01_1 tag |
| M-C1.survey.confident_percent | 88 | S2: "88% of the residents expressed fair to complete confidence in understanding p-values" (reporting S1) | 01_1 said and the first bar |
| M-C1.survey.keyed_choice | c | S2: "58.8% of the residents selected choice c which was designated by the authors as the correct answer" | 01_2 bar label and quote card |
| M-C1.survey.keyed_percent | 58.8 | S2, same sentence | 01_2 second bar |
| M-C1.survey.keyed_percent_spoken | 59 | 58.8 rounded half away from zero | 01_2 said ("about fifty-nine percent") |
| M-C2.gao.year | 2020 | S2 | 01_2 quote card and the dispute chip |

Connecticut and the dates: S1's methods, in JAMA's indexed publisher text
(jamanetwork.com/journals/jama/fullarticle/208638), say the data were
collected February to July 2006 in 11 Connecticut programs; the article was
published on 5 September 2007. The orchestrator (SageBasin) read that indexed
text on 2026-10-17 through a web search restricted to jamanetwork.com; a
direct fetch of the page timed out. BrownThrush read only S1's abstract
(Europe PMC), which names neither the state nor the dates. The first draft of
01_1 said "In two thousand seven, researchers surveyed", which put the
publication year on the fieldwork; it now says "published in two thousand
seven".

The quote on the 01_2 card is choice c **verbatim as S2 reprints it**, the
keyed one of "four potential answers for a correct interpretation of p >
0.05". The original question's clinical stem was not retrieved, so the video
shows no stem and never presents the card as a facsimile of the
questionnaire.

## M-F · the clinics (synthetic, labelled "model")

Clinic A, chapter 01: 10,000 people, one in a hundred with the disease, a
test that catches nine in ten of them and falsely alarms one in twenty
people without it. Clinic B (chapter 02): the same test where one in ten has
the disease. Each count is a whole number of people or the model refuses the
clinic (`model::Clinic::counts`, tested). Every share is counted over an
explicit list of people in both `model.rs` (the icon array) and `oracle.py`
(built independently, one tuple per person); Bayes' rule is checked against
the count in both.

| Key | Value | Obtained | Where |
|---|---|---|---|
| M-F1.people | 10000 | the clinic | 01_6 said; the array's label |
| M-F1.prevalence.exact / .one_in | 1/100 / 100 | the clinic | 01_6 said "one in a hundred"; ledger |
| M-F1.sensitivity.exact / _percent | 9/10 / 90 | the clinic (assumed the same in every clinic) | 01_7 "catches ninety" |
| M-F1.false_alarm.exact / _percent | 1/20 / 5 | the clinic (assumed the same in every clinic) | 01_4, 01_8, 01_11 said; the lab's card |
| M-F1.with / .without | 100 / 9900 | counted | 01_7, 01_8 said; ledger |
| M-F1.true_pos / .false_neg | 90 / 10 | counted | 01_7, 01_10 said; 90 tiles |
| M-F1.false_pos / .true_neg | 495 / 9405 | counted | 01_8, 01_10 said; 495 tiles |
| M-F1.positives | 585 | counted; pinned to the brief | 01_10 the fraction's bottom |
| M-F1.ppv.exact | 2/13 | P(disease \| positive), counted; = Bayes | 01_11 lower-third |
| M-F1.ppv_percent / _2dp | 15 / 15.38 | 2/13; the 2-place value pinned to the brief | 01_10 said "about fifteen percent", shown "≈ 15%"; lower-third 15.38% |
| M-F1.reverse.exact / _percent_2dp | 11/13 / 84.62 | P(no disease \| positive), the direct reversal of the false-alarm rate; pinned | not shown in 01 (the negative check) |
| M-F1.tempting_percent | 95 | P(negative \| no disease) = 1 − false-alarm rate | 01_5 said and shown "95%?", struck in 01_10 |
| M-F1.grid.cols / .rows | 100 / 100 | layout of the array, checked to fit the counts | 01_6 |
| M-F1.grid.disease_rows | 1 | 100 people with the disease fill one row | 01_7 said "one row" |
| M-F1.grid.false_pos_per_row | 5 | 495 false alarms over 99 rows | 01_8 the stripe |
| M-F1.block.cols / .rows / .disease_rows | 45 / 13 / 2 | rows of 45 (= gcd(90, 495)); 585/45; 90/45; so 2/13 is visible as rows | 01_9, 01_10 ("2 rows of 13"); 02 clinic A's block |
| M-F2.people / .prevalence.exact / .one_in | 10000 / 1/10 / 10 | clinic B: the same 10,000, the disease ten times as common | 02_1 said "one person in ten"; header |
| M-F2.sensitivity.exact / _percent | 9/10 / 90 | the same test (assumed) | 02_2 said "catches nine in ten"; the test's card |
| M-F2.false_alarm.exact / _percent | 1/20 / 5 | the same test (assumed) | 02_2, 02_5 said; the card; 450 / 9,000 = 5% |
| M-F2.with / .without | 1000 / 9000 | counted | 02_4, 02_5 said; ledger |
| M-F2.true_pos / .false_neg | 900 / 100 | counted | 02_4, 02_6 said; 900 tiles; the tenth gold row stays dark |
| M-F2.false_pos / .true_neg | 450 / 8550 | counted | 02_5 said; 450 tiles |
| M-F2.positives | 1350 | counted; pinned to the brief | 02_6 said "thirteen hundred fifty"; the fraction's bottom |
| M-F2.ppv.exact / _percent / _2dp | 2/3 / 67 / 66.67 | counted = Bayes; the 2-place value pinned | 02_6 said "two in three", 02_7 "about sixty-seven percent", shown "≈ 67%" |
| M-F2.reverse.exact / _percent_2dp | 1/3 / 33.33 | P(no disease \| positive) in clinic B | not shown (the negative check) |
| M-F2.tempting_percent | 95 | 1 − the false-alarm rate, the same as clinic A's | not shown |
| M-F2.grid.disease_rows / .false_pos_per_row | 10 / 5 | 1,000 people fill ten rows; 450 false alarms over 90 rows | 02_1 ten rows turn gold; 02_5 the stripe |
| M-F2.block.cols / .rows / .disease_rows | 45 / 30 / 20 | rows of 45 like clinic A's; 1,350/45; 900/45; so 2/3 is visible as rows | 02_6 ("20 rows of 30") |
| M-F2.prevalence_vs_a | 10 | (1/10) / (1/100) | the two headers |
| M-F2.caught_vs_a | 10 | 900 / 90 | 02_8 said "ten times as many"; the "× 10" chip |

Chapter 02's assumption, on screen as "(assumed)": the test's two rates are
the same in both clinics. Real tests can perform differently in different
populations; the chapter compares the same model test, and `facts::compare`
refuses to build if the two clinics' rates differ.

## M-T · the study's null world (synthetic)

20 pairs matched before treatment; in each pair a fair coin gives one patient
the treatment; recovery time, lower is better, no ties; direction, statistic
and analysis fixed in advance. The sharp null: the treatment changes no
patient's recovery and there is no interference, so every recovery time is
fixed and only the labels move: each of the 2^20 assignments is equally
likely. The statistic K counts pairs whose faster patient carries the
treatment label. `model::Study::null_world` lists every assignment (bit
masks); `oracle.py` lists them again with `itertools.product` over the pairs
and compares with `math.comb`. Neither uses a normal approximation.

| Key | Value | Obtained | Where |
|---|---|---|---|
| M-T1.pairs | 20 | the study | 02_10 tag; 03_1, 03_4, 03_7 said "twenty" |
| M-T1.patients | 40 | two patients a pair | 03_8 card ("the same 40 patients in 20 pairs") |
| M-T1.observed | 15 | the synthetic data (pairs 3, 7, 11, 16, 19 went the other way) | 03_4 said and lit, after 03_3's null in words; 03_9 said; the study's dot on the histogram |
| M-T1.assignments | 1048576 | 2^20, the world listed | 03_7 said "just over a million", shown 1,048,576 |
| M-T1.null.counts | 1,20,190,…,20,1 | assignments with k favouring pairs, k = 0…20 | 03_7 histogram |
| M-T1.null.max / .mode | 184756 / 10 | the tallest bar and where it stands | 03_7, above the tallest bar |
| M-T3.retoss.seed | 2006 | fixed before any toss was looked at | not shown |
| M-T3.retoss.assignments | 216459, 666002, 803510, 64675 | four whole assignments: the low 20 bits of four draws of `Data::new(2006).raw()` (NumPy `PCG64DXSM(2006).random_raw()`), one fair bit a pair | 03_5, 03_6: the rings jump to these |
| M-T3.retoss.counts | 12, 12, 11, 12 | each re-toss's pairs favouring the treatment | 03_5, 03_6 the live count and the row of tosses; 03_7 dots on their bars |

Chapter 03's assumptions, on screen: pairs matched before treatment, one fair
coin per pair, no ties (the model's order bits cannot tie), and the sharp
null: no patient's recovery would change treated or untreated, and no patient
affects another. A zero average effect alone does not make every assignment
equally likely to produce these exact counts; the 03_3 lower-third says so.
The study's bits are an order and a label (`Study::faster_a`, `treated_a`):
the video draws who recovered first and who was treated, never a recovery
time, because the model has none.

Chapter 04 counts the whole tail in assignments before it names anything.
What was fixed before the data is the statistic K and its favourable
direction (more pairs favouring is better news); the boundary 15 is the
observed result, not a cutoff chosen in advance, so the tail is "this result
or more favourable". The share is the one-sided p-value under the sharp null
and the fair coins, never a probability that the treatment does nothing, an
efficacy or an effect size. One predicate, `Tail::holds`, decides every lit
or outlined set and every count (`tail_members`, `tail_count`).

| Key | Value | Obtained | Where |
|---|---|---|---|
| M-T4.tail_counts | 15504,4845,1140,190,20,1 | the listed world's bars 15 to 20, in assignments; `math.comb` in the oracle | 04_3 lavender overlays; 04_6 ledger |
| M-T4.tail_bars | 6 | the counts 15 to 20, both ends included | 04_3 |
| M-T4.count_20 / .count_19 | 1 / 20 | enumerated: assignments with all 20, or exactly 19, pairs favouring | 04_4 "only one way", 04_5 "Twenty ways"; the small pairs show `favouring_all_but(&[])`, then three of the twenty `favouring_all_but(&[j])` |
| M-T4.k_one_against | 19 | the count after one pair of the all-favouring assignment is turned over | 04_5 "For nineteen" |
| M-T1.tail.at_least | 21700 | K ≥ 15 counted; pinned to the brief | 04_6 said, the ledger's total; 04_7 numerator |
| M-T1.p.exact / _10dp | 5425/262144 / 0.0206947327 | 21,700 / 1,048,576; pinned to the brief | 04_7 |
| M-T1.p_percent_0dp / _4dp | 2 / 0.0207 | | 04_7 "about two percent", ≈ 2% / the lower-third |
| M-T1.p_percent_2dp / _1dp | 2.07 / 2.1 | | not said in 04 (it says the whole percent) |
| M-T2.observed / .tail.at_least / .p_10dp | 14 / 60460 / 0.0576591492 | K ≥ 14; pinned to the brief | 05 |

## M-X · the wrong readings (each a different number, each tested)

| Key | Value | What it is, and why it is not the answer |
|---|---|---|
| M-F1.tempting_percent | 95 | the share of negatives among people without the disease; not the share of the disease among positives (2/13) |
| M-F1.reverse.exact | 11/13 | P(no disease \| positive): the complement of the PPV; the false-alarm rate 1/20 is not 1 − PPV |
| M-F2.reverse.exact | 1/3 | clinic B's reversal: the false-alarm rate stays 1/20 from A to B while its reversal falls from 11/13 to 1/3 |
| M-X1.exactly / _percent_2dp | 15504 / 1.48 | K = 15 only: leaves out the more extreme outcomes |
| M-X1.more_than / _percent_2dp | 6196 / 0.59 | K > 15: leaves out the observation itself |
| M-X1.exactly_percent_1dp / .more_than_percent_1dp | 1.5 / 0.6 | 04_8 says both, each beside a red outline round its own bars: other subsets, other numbers, neither the test |
| M-X1.at_most | 1042380 | K ≤ 15: the other direction (everything but the more extreme) |
| M-X1.two_sided / _percent_2dp | 43400 / 4.14 | K ≥ 15 or K ≤ 5: a different two-sided question, which would have to be chosen in advance itself; not this one-sided test, and never "the one-sided p doubled" as a rule (not shown in 04) |
| M-X3.a_treated | 9 | how many A patients the coins treated: a planted wrong statistic, not the count of pairs favouring the treatment (15); its world is the same binomial, so only the statistic tells them apart |

Tests that hold these apart: `model.rs` `swapping_the_condition_changes_the_number`,
`swapping_the_condition_in_the_second_clinic_changes_the_number_too`,
`the_same_test_keeps_its_rates_while_the_ppv_moves_with_the_clinic` (a
counted sweep over seven prevalences: the two rates never move, the PPV only
rises), `the_second_clinic_counts_whole_people_and_near_misses_are_refused`
(10,001 people at 1 in 10, 1,000 at 1 in 100), `both_clinics_regroup_into_rows_of_forty_five`
(a 1-in-20 clinic's 450 and 475 positives are refused),
`the_p_value_counts_the_whole_tail_and_the_wrong_tails_differ`,
`the_tail_is_counted_in_assignments_with_its_endpoint_included` (inclusive
endpoints, all six counts, the extremes 0 and 20, the mirror, and the
off-by-one and wrong-direction mutations refused), `each_highlighted_set_is_its_inequality`,
`twenty_has_one_way_and_nineteen_has_one_per_pair` (enumerated), and chapter
04's own `the_wrong_tails_are_other_subsets_of_the_tail` and
`the_words_say_what_was_fixed_and_what_was_set`; `oracle.py` cross-checks of
the same claims, computed its own way (53 in all).

## Sources

| | Source | Read in this session | Used for |
|---|---|---|---|
| S1 | Windish DM, Huot SJ, Green ML. Medicine residents' understanding of the biostatistics and results in the medical literature. JAMA 2007;298(9):1010–1022. doi:10.1001/jama.298.9.1010 | abstract, via Europe PMC (2026-10-17); the methods (Connecticut, February–July 2006) read by the orchestrator in JAMA's indexed publisher text, direct fetch timed out | 277 of 367, 75.5%, 11 programs, internal medicine, Connecticut; collected 2006, published 2007 |
| S2 | Gao J. P-values – a chronic conundrum. BMC Medical Research Methodology 2020;20:167. doi:10.1186/s12874-020-01051-6 | full text, Europe PMC PMC7315482 (2026-10-17) | 88%, 58.8%, choice c verbatim, the dispute ("none of the four choices are correct") |
| S3 | Wasserstein RL, Lazar NA. The ASA's statement on p-values: context, process, and purpose. The American Statistician 2016;70(2):129–133. doi:10.1080/00031305.2016.1154108 | not re-read (the brief read it) | act II: what a p-value is and is not |
| S4 | Hoffrage U, Gigerenzer G. Using natural frequencies to improve diagnostic inferences. Academic Medicine 1998;73(5):538–540 | not re-read (the brief read the abstract) | why the chapter counts people (not quoted in the video) |

Downloaded texts stay outside the repository (public-repository hygiene).

## The survey: what the video says, and what it does not

- Said: a survey published in 2007 (run in 2006) of 277 internal-medicine
  residents in Connecticut;
  88% at least fairly confident; about 59% picked the keyed answer; a later
  paper argued the keyed answer is wrong too.
- Not said: anything about all doctors or a national sample (S1 is 277 of
  367 residents in 11 programs of one state); that most residents chose a
  wrong option (most chose the keyed one; there is no option-by-option
  distribution for the others); S2's "100% of them had the p-value
  interpretation wrong" (that follows only if the key is wrong, which is
  S2's argument, presented as an argument); S2's "at least 28.9%" posterior
  (it needs assumptions a p-value does not carry).
- Competing explanations of the gap between confidence and the keyed
  answer, none asserted as the cause: residents misunderstand p-values; the
  item and its key were themselves flawed (S2); a single multiple-choice item
  measures little; confidence was self-reported on a scale. The video's own
  explanation (the question changes which group a percent counts) is
  offered as a way to see the trap, not as S1's finding.

## Simplification → exact

| Line | Simplified | Exact (where it appears) |
|---|---|---|
| 01_5 | "a ninety-five percent chance you have it" | the chance for a person chosen at random from this model clinic's positives (01_11 lower-third) |
| 01_10 | "about fifteen percent" | 90/585 = 2/13 ≈ 15.38% (01_11 lower-third) |
| 01_11 | "a share of people without the disease", "a share of people who tested positive" | P(positive \| no disease) = 1/20, P(disease \| positive) = 2/13; neither is a p-value (lower-third) |
| 01_7 | "the test catches ninety" | sensitivity 9/10, assumed, and assumed the same in every clinic (storyboard 02) |
| 02_2 | "The test hasn't changed" | the model test's sensitivity and false-alarm rate are assumed the same in both clinics (the card says "assumed") |
| 02_7 | "a positive result means about sixty-seven percent" | P(disease \| positive) = 900/1,350 = 2/3 ≈ 66.67% for a person chosen at random from clinic B's positives (02_9's card) |
| 02_8 | "the false alarms barely moved" | 495 → 450 false alarms, the rate 5% in both (the outlined ledgers and fractions) |
| 02_9 | "the chance a positive result is right" | the positive predictive value, P(disease \| positive): 2/13 in clinic A, 2/3 in clinic B (the card) |
| 03_3 | "Each patient would have recovered at the same time either way" | the sharp null: every patient's recovery the same treated or untreated, no interference; a zero average effect alone would not give this world (the lower-third) |
| 03_7 | "just over a million ways, all equally likely" | 2^20 = 1,048,576 assignments, each with probability 1/2^20 because each pair's coin is fair and independent (the title) |
| 03_8 | "Same patients, coins tossed again" | the randomization distribution: the same 40 patients, the treatment labels re-assigned; not a sample of new patients, and not the probability that the treatment does nothing (the card) |

## Terms (first use)

| Term | Line | As spoken or shown |
|---|---|---|
| p-value | 01_1 | named only (the survey's word); defined in act II |
| false alarm | 01_4 | "a false alarm to five percent of people without the disease" |
| share | 01_11 | what a percent is: a part of a group |
| group | 01_11 (shown) | the people a share is taken of; outlined in ink |
| false-alarm rate | 02_5 (shown), 02_9 said | the share of people without the disease who test positive |
| positive predictive value | 02_9 (shown once, on the card) | "the chance a positive result is right" |
| favours the treatment | 03_2 | "a pair favours the treatment if its treated patient recovered first", decided before looking |
| sharp null | 03_3 (shown once, on the lower-third) | "the treatment changed nobody's recovery" |

## Always / never

Chapter 03's "who recovered first stays fixed" is the sharp null itself, not
a claim about the data; the tests hold the model to it (re-tosses move only
labels). Chapters 01 and 02 say no always or never. 02_8's "barely moved" is
qualitative and shown exactly (495 and 450). The model's own edges are tested: a rate
that would split a person is refused; a p-value in the study is never zero
(the most favourable K gives 1/2^20) and the least favourable gives 1; the
two-sided p at the middle is 1.
