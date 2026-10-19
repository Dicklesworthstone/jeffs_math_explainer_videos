<!--
Draft delivery description for p_values_medical (2026-10-18). Every number
below is a fact of docs/fact-sheet.md, computed in src/facts.rs and recomputed
by oracle.py, or a cited source's printed value; the bracketed tags name them,
and `python3 -I tools/description_numbers.py p_values_medical` checks each one.
The chapter list carries no timestamps: they come from the final narrated cut,
which does not exist yet. The six chapters are accepted as silent drafts only;
the narration voice awaits the owner's listening. The release package adds the
repository link and trims this to what the upload page accepts, linking this
file for the rest.
-->

# Why P-Values Confuse Even Doctors

A survey published in 2007 asked internal-medicine residents in Connecticut
about p-values [M-C1.survey.published_year]. Of the 277 who answered, 88% felt
fairly to completely confident [M-C1.survey.respondents,
M-C1.survey.confident_percent], and 58.8% picked the answer marked correct
[M-C1.survey.keyed_percent]; a later paper argued that even that answer is
wrong (Gao, 2020) [M-C2.gao.year]. This video makes the trap visible by
counting. Before trusting a percent, find the group it is a share of: first in
a model clinic, then in the imaginary world behind a p-value.

## Chapters

1. Five percent of what?
2. The same test in another clinic
3. The imaginary world
4. Count the whole tail
5. What two percent does not say
6. The question again

## What the video shows

**A rate is a share of a group.** In a model clinic of 10,000 people
[M-F1.people] where one in 100 has the disease [M-F1.prevalence.one_in], the
test catches 90 of the 100 with it [M-F1.true_pos, M-F1.with] and falsely
alarms 5% of the 9,900 without it [M-F1.false_alarm_percent, M-F1.without]:
495 false alarms [M-F1.false_pos]. Of the 585 positives [M-F1.positives], 90
have the disease, 2/13 or about 15% [M-F1.ppv.exact, M-F1.ppv_percent], not
the tempting 95% [M-F1.tempting_percent].

**The same test, another group.** Where one person in 10 has the disease
[M-F2.prevalence.one_in], 900 of the same test's 1,350 positives have it
[M-F2.true_pos, M-F2.positives]: 2/3, about 67% [M-F2.ppv.exact,
M-F2.ppv_percent]. The test did not change; the group did. Neither number is a
p-value.

**A p-value counts a tail in an assumed world.** A synthetic study has 20
pairs of patients [M-T1.pairs], a fair coin in each pair choosing who is
treated, and in 15 pairs the treated patient recovered first [M-T1.observed].
If the treatment changed no patient's recovery (the sharp null), only the
coins' labels can move, so all 1,048,576 assignments are equally likely
[M-T1.assignments]. Of them, 21,700 give 15 or more favouring pairs
[M-T1.tail.at_least]: the bars from 15 up hold 15,504, 4,845, 1,140, 190, 20
and 1 assignments [M-T4.tail_counts]. That share, 21,700 / 1,048,576 ≈ 0.0207
[M-T1.p_4dp], is this study's one-sided p-value. What was fixed in advance is
the statistic and its direction; the boundary 15 is the observed result.

**The whole tail, not a piece of it.** Counting exactly 15 (15,504, about 1.5%)
or only more than 15 (6,196, about 0.6%) gives other numbers [M-T1.observed,
M-X1.exactly, M-X1.exactly_percent_1dp, M-X1.more_than,
M-X1.more_than_percent_1dp].

**What it is not.** It is not the chance that the treatment did nothing, nor
the chance that the study is wrong: the share was counted inside the world it
assumes. One more pair the other way, 14 [M-T2.observed, M-T2.moved], would
give 60,460 / 1,048,576 ≈ 5.77% [M-T2.tail.at_least, M-T1.assignments,
M-T2.p_percent_2dp]. A 5% line between 2.07% and 5.77% [M-T5.line_percent,
M-T1.p_percent_2dp] is a convention for deciding, not a switch in the truth.

**What it never measured.** Two made-up versions of the study's recovery days
[M-D.versions] keep its order in every pair, so both have the same 15
favouring pairs [M-D1.favouring, M-D2.favouring] and the same 2.07%
[M-T1.p_percent_2dp]. Among those 15 pairs, the median treated patient
recovered 1 day sooner in one version and 7 days sooner in the other
[M-D1.median_sooner_days, M-D2.median_sooner_days]. The count never used how
much sooner. Those bars are observed differences in constructed data, not a
treatment effect.

**The survey's question, asked precisely.** The keyed answer, as Gao reprints
it, was written for p > 0.05 [M-C1.survey.keyed_for] and speaks of a
difference arising "by chance alone". For this study the precise question is:
if the treatment changed no patient's recovery and the fair coins were tossed
again, how often would 15 or more pairs favour the treatment? In 21,700 of
1,048,576 assignments, about 2.07% [M-T1.observed, M-T1.tail.at_least,
M-T1.assignments, M-T1.p_percent_2dp]. That answers one question about one
assumed world. By itself it says nothing about how large a difference is,
whether it matters to a patient, or how uncertain it is.

## What is surveyed and what is synthetic

- **Surveyed.** Windish, Huot and Green's survey was published in JAMA in
  2007, with data collected February to July 2006
  [M-C1.survey.published_year, M-C1.survey.collected_year,
  M-C1.survey.collected_months]: 277 of 367 residents (75.5%) in 11
  internal-medicine programs in Connecticut [M-C1.survey.respondents,
  M-C1.survey.residents, M-C1.survey.response_percent, M-C1.survey.programs].
  The 88% and 58.8% are as Gao reports them [M-C1.survey.confident_percent,
  M-C1.survey.keyed_percent]. Choice c is quoted as Gao reprints it; the
  original question's clinical stem was not retrieved, so the video shows
  none. This is a regional sample of residents, not doctors in general. Most
  of those who answered picked the keyed answer, and the key itself is
  disputed.
- **Synthetic, labelled "model" on screen.** Both clinics, the study and its
  15 [M-T1.observed], every assignment, and the two versions of the recovery
  days. No real patient is in them, and no treatment effect is estimated.

## Further reading

- Windish DM, Huot SJ, Green ML. Medicine residents' understanding of the
  biostatistics and results in the medical literature. JAMA
  2007;298(9):1010–1022. https://doi.org/10.1001/jama.298.9.1010
- Gao J. P-values – a chronic conundrum. BMC Medical Research Methodology
  2020;20:167. https://doi.org/10.1186/s12874-020-01051-6
- Wasserstein RL, Lazar NA. The ASA's statement on p-values: context,
  process, and purpose. The American Statistician 2016;70(2):129–133.
  https://doi.org/10.1080/00031305.2016.1154108
- Hoffrage U, Gigerenzer G. Using natural frequencies to improve diagnostic
  inferences. Academic Medicine 1998;73(5):538–540.
- Names to search: randomization test, permutation test, sign test, Fisher's
  sharp null hypothesis, positive predictive value, base-rate fallacy, effect
  size, confidence interval.
