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
| M-F1.block.cols / .rows / .disease_rows | 45 / 13 / 2 | gcd(90, 495) = 45; 585/45; 90/45; so 2/13 is visible as rows | 01_9, 01_10 ("2 rows of 13") |
| M-F2.prevalence.exact | 1/10 | clinic B | 02 |
| M-F2.with / .without | 1000 / 9000 | counted | 02 |
| M-F2.true_pos / .false_pos / .positives | 900 / 450 / 1350 | counted; positives pinned to the brief | 02 |
| M-F2.ppv.exact / _percent_2dp | 2/3 / 66.67 | counted = Bayes; pinned to the brief | 02 |

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
| M-T1.pairs | 20 | the study | 03 |
| M-T1.observed | 15 | the synthetic data (pairs 3, 7, 11, 16, 19 went the other way) | 03 |
| M-T1.assignments | 1048576 | 2^20, the world listed | 03, 04 |
| M-T1.null.counts | 1,20,190,…,20,1 | assignments with k favouring pairs, k = 0…20 | 03 histogram |
| M-T1.tail.at_least | 21700 | K ≥ 15 counted; pinned to the brief | 04 |
| M-T1.p.exact / _10dp | 5425/262144 / 0.0206947327 | 21,700 / 1,048,576; pinned to the brief | 04 |
| M-T1.p_percent_2dp / _1dp | 2.07 / 2.1 | | 04 ("about two percent") |
| M-T2.observed / .tail.at_least / .p_10dp | 14 / 60460 / 0.0576591492 | K ≥ 14; pinned to the brief | 05 |

## M-X · the wrong readings (each a different number, each tested)

| Key | Value | What it is, and why it is not the answer |
|---|---|---|
| M-F1.tempting_percent | 95 | the share of negatives among people without the disease; not the share of the disease among positives (2/13) |
| M-F1.reverse.exact | 11/13 | P(no disease \| positive): the complement of the PPV; the false-alarm rate 1/20 is not 1 − PPV |
| M-X1.exactly / _percent_2dp | 15504 / 1.48 | K = 15 only: leaves out the more extreme outcomes |
| M-X1.more_than / _percent_2dp | 6196 / 0.59 | K > 15: leaves out the observation itself |
| M-X1.at_most | 1042380 | K ≤ 15: the other direction (everything but the more extreme) |
| M-X1.two_sided / _percent_2dp | 43400 / 4.14 | K ≥ 15 or K ≤ 5: a different, two-sided question, not this predeclared one-sided test |

Tests that hold these apart: `model.rs` `swapping_the_condition_changes_the_number`,
`the_p_value_counts_the_whole_tail_and_the_wrong_tails_differ`; `oracle.py`
cross-checks of the same names (24 in all).

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

## Terms (first use)

| Term | Line | As spoken or shown |
|---|---|---|
| p-value | 01_1 | named only (the survey's word); defined in act II |
| false alarm | 01_4 | "a false alarm to five percent of people without the disease" |
| share | 01_11 | what a percent is: a part of a group |
| group | 01_11 (shown) | the people a share is taken of; outlined in ink |

## Always / never

Chapter 01 says no always or never. The model's own edges are tested: a rate
that would split a person is refused; a p-value in the study is never zero
(the most favourable K gives 1/2^20) and the least favourable gives 1; the
two-sided p at the middle is 1.
