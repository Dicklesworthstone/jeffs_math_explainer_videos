# P-values: a medical opening and a visible explanation

Research/editorial proposal, 2026-10-16. Consumer: Jeff and the next script author. This is source research and checked mathematics, not a finished script, listening result, or audience test. The user's medical focus supersedes optional stopping as this video's central story; keep optional stopping for a short closing pointer or a sequel.

## Recommendation

Open with the doctors' survey, turn immediately to the question a patient actually needs answered, and make the audience see a conditional probability reverse. Then explicitly change from **a diagnostic test on a person** to **a statistical test in a treatment study**. The visual connection is the direction of conditioning; the two numbers are different quantities. Finish with what the study establishes about treatment benefit and what remains uncertain.

Working title: **Why P-Values Confuse Even Doctors**. The audience promise is practical understanding without memorizing a definition. A thumbnail can put **“5% of WHAT?”** beside two different highlighted denominators. Avoid an unsupported percentage of doctors who “fail,” or imagery implying medical incompetence.

## Primary evidence and access boundaries

**S1 — Original survey.** Windish, Huot, and Green, *Medicine Residents' Understanding of the Biostatistics and Results in the Medical Literature*, JAMA 2007;298:1010–1022, DOI 10.1001/jama.298.9.1010. Publisher-indexed main text and the authors' abstract were accessible; direct full-page retrieval failed. The study surveyed 277 internal-medicine residents across 11 Connecticut programs; 277 of 367 residents available for the survey responded. It reports 88% expressing fair-to-complete confidence about p-values. This regional training sample is not a nationally representative survey of all US doctors. [JAMA](https://jamanetwork.com/journals/jama/fullarticle/208638), [original abstract on PubMed](https://pubmed.ncbi.nlm.nih.gov/17785646/).

**S2 — Published criticism of the question.** Gao's 2020 commentary reproduces the four choices for interpreting p > .05 and reports 58.8% chose the keyed answer, c. Gao disputes every choice. This is evidence of a published critique, not evidence that nobody understood p-values. [Gao, *P-values – a chronic conundrum*, Background](https://link.springer.com/article/10.1186/s12874-020-01051-6).

Choices, paraphrased except c:

- a: a greater-than-5% chance of reproducing a difference.
- b: less than 5% for a difference this large from chance.
- c: “The probability is greater than 1 in 20 that a difference this large could occur by chance alone.”
- d: 95% probability that the study is correct.

The original appendix's exact clinical stem was **not independently retrieved**. Do not present a reconstructed full questionnaire as a facsimile. The selected opening below needs no such reconstruction. We have no option-by-option response distribution supporting a claim that most chose d. Exclude Gao's universal 28.9% posterior assertion; it cannot follow from a p-value alone without additional assumptions.

**S3 — Statistical interpretation.** The ASA's 2016 statement treats a p-value as a model-dependent tail probability. It does not assign probability to the hypothesis, certify an effect's size, or justify decisions by a threshold alone. The statement also requires context and disclosure of how analyses were selected. Use it as the interpretation reference, while deriving the concrete examples below explicitly. [Wasserstein and Lazar, DOI 10.1080/00031305.2016.1154108, institutional copy, printed pp. 131–132](https://www.stat.berkeley.edu/~aldous/Real_World/ASA_statement.pdf); [ASA's official release](https://www.amstat.org/asa/files/pdfs/p-valuestatement.pdf). The journal's full-text URL was inaccessible; the institutional copy contains the original article and statement.

**S4 — Evidence for the teaching format.** Hoffrage and Gigerenzer tested 48 physicians in Munich and Düsseldorf on four diagnostic problems. Correct positive-predictive-value judgments occurred in 10% of probability-form cases versus 46% with natural frequencies. This supports trying count-based explanations; it is neither a p-value experiment nor proof that this particular animation will teach successfully. [*Using natural frequencies to improve diagnostic inferences*, Academic Medicine 1998;73:538–540, original abstract](https://pubmed.ncbi.nlm.nih.gov/9609869/).

All links inspected or searched on 2026-10-16. Original survey facts above were verified in indexed publisher text; S2's options were read directly. The study's national comparisons do not convert its sampling design into a national probability sample. No clinical recommendation is inferred from any synthetic example below.

## Proposed opening, approximately 45 seconds

> In 2007, researchers surveyed 277 medical residents in Connecticut. Eighty-eight percent expressed at least fair confidence in their understanding of p-values. But only about 59 percent selected the answer marked correct—and a later paper challenged even that answer.
>
> Why is this so confusing? Imagine your medical test comes back positive. You're told it falsely alarms in five percent of people without the disease. Is there a ninety-five percent chance you have it?
>
> Before answering, we need to find the people that five percent refers to. That one move will also help us understand what a p-value is actually telling us.

The first paragraph draws on S1/S2. The second and third are an original hypothetical. Keep a visible “2007 · Connecticut medical residents” qualifier during the survey card. Use restrained delivery: curiosity, a short pause before the question, and no mocking or alarmist music. The narrator should say “the answer marked correct,” not “understood correctly.”

## Why a smart person can answer the wrong question

These are proposed explanatory mechanisms for the script, **not causal findings of S1**:

1. **The wanted answer differs from the supplied number.** A patient wants a probability of illness after a result; a lab may supply a probability of that result among people without illness. Both sentences contain the same nouns. Animate the change in which people form the denominator.
2. **A percentage can hide its reference group.** “Five percent” sounds complete. It is not complete until the population or hypothetical world is specified. Keep that qualifier attached to every numeric label.
3. **Ordinary language compresses different questions.** “It happened by chance” can mean an event was unusual under a model, a causal explanation is probably true, or a conclusion is probably mistaken. Ask which of those has actually been calculated.
4. **A crisp answer feels more useful than qualified evidence.** A green/red threshold can obscure magnitude, uncertainty, and competing explanations. Use the threshold only after the model and data are visible.

S4 provides bounded evidence that representation matters for diagnosis. Whether these particular beats improve p-value comprehension is an unanswered production question. Test transfer, not viewers' ability to repeat the narrator's wording.

## Central medical scene: two clinics, the same test

Use synthetic cohorts of 10,000. All counts below are an illustrative population constructed to satisfy the stated rates, not a prediction that every real sample produces exact counts. Disease status is binary in this toy model. The stated prevalence is the pre-test probability for a uniformly selected person in that cohort; no additional patient-specific information is being used. Assume sensitivity and false-positive rate stay fixed when changing the cohort.

| Quantity | Clinic A | Clinic B |
|---|---:|---:|
| Prevalence | 1% | 10% |
| People with condition | 100 | 1,000 |
| People without condition | 9,900 | 9,000 |
| Sensitivity | 90% | 90% |
| True positives | 90 | 900 |
| False negatives | 10 | 100 |
| False-positive rate among people without condition | 5% | 5% |
| False positives | 495 | 450 |
| True negatives | 9,405 | 8,550 |
| All positive results | 585 | 1,350 |
| Condition among positive results, PPV | 90 / 585 = **15.38%** | 900 / 1,350 = **66.67%** |

Animation sequence: first partition by condition, then reveal test outcomes inside each branch, then physically regroup only the positive results. Preserve disease-color identity during regrouping; add labels/patterns so color is not the sole cue. The audience should watch the denominator change from all people without the condition to all people with a positive result. Zoom individual tiles only when needed; do not expect 10,000 tiny marks to communicate the answer alone.

For technical notes, with prevalence q, sensitivity s, and false-positive rate f:

`P(condition | positive) = s*q / (s*q + f*(1-q))`.

Keep the complement explicit: the direct reversal of `P(positive | no condition)` is `P(no condition | positive)`. In Clinic A, the latter is `495/585 = 84.62%`; its complement is the 15.38% PPV. The diagnostic false-positive rate is not `1 - PPV`.

Narration: “The test has not changed. The mix of people being tested has. Among the positive results, the balance between true cases and false alarms is different.” State that the constant test-performance assumption is part of this illustration; real settings need appropriate performance estimates and patient context.

**Mandatory bridge card:** “Diagnostic false-positive rate ≠ positive predictive value. Neither number is a p-value.” Then: “Keep the direction of the question in mind. Now we are asking a new question about a treatment study.” Do not relabel the 5% diagnostic error rate as p = .05, or use the 15.38% result as a posterior probability that a treatment works.

## Treatment scene: construct the hypothetical world before calculating

Original synthetic example: 40 patients arranged into 20 pairs before treatment. Independently within each pair, one is randomly assigned a treatment and one placebo. The outcome is recovery time; lower is better. All 40 outcomes are observed, none of the within-pair comparisons is tied in this example. The direction, statistic, sample size, and analysis are chosen in advance. This is a teaching model, not a proposed clinical protocol.

The **sharp null** says treatment changes no patient's outcome, and there is no interference between patients. Under this hypothesis, the two recovery times within a pair remain fixed when assignment labels are exchanged. Each independent assignment therefore has probability one-half of putting the treatment label on the faster recovery. The number K of pairs favoring treatment has distribution Binomial(20, 1/2). This derivation does not require normal recovery times. It tests this sharp null, not merely a zero average effect in every possible heterogeneous-treatment model.

Suppose 15 pairs favor treatment. Highlight **15, 16, 17, 18, 19, and 20** in the null distribution:

`p = P_null(K >= 15) = [C(20,15) + ... + C(20,20)] / 2^20`

`= 21,700 / 1,048,576 = 0.0206947327 ≈ 2.07%`.

Narration: “If treatment changed nobody's recovery, only about two percent of these random assignments would favor treatment in fifteen or more pairs.” Maintain the hypothetical qualifier in frame. It is not the probability that treatment does nothing, not the chance this paper is wrong, and not the exact probability of obtaining K = 15 alone.

The displayed p-value is **one-sided**, because the direction was specified before outcomes were seen. A two-sided analysis asks a different question and must not silently inherit this number. Explain the tail visually before naming it. A complete enumeration has 2^20 assignments; any animation showing a subset must say it is an illustration, while the printed calculation is exact.

One nearby result, 14 favorable pairs, has tail probability `60,460 / 1,048,576 = 0.0576591492`. Use it to challenge a “works/doesn't work” stamp, not to claim these two datasets are identical. They contain different evidence; crossing .05 still does not calculate the probability a drug works. For the familiar .049/.051 comparison, say explicitly it is a separate illustrative comparison, not another output of this 20-pair calculation.

The validity of this calculation depends on its assignment scheme, outcome handling, and predeclared analysis. Do not suggest that a p-value automatically corrects bias, missing outcomes, confounding, outcome switching, or repeated unaccounted looks. The detailed stopping-rule demonstration belongs in a sequel.

## Return to the clinical question: size, uncertainty, and decisions

First reveal the sizes of the pairwise recovery-time differences. Multiplying every difference by ten keeps all signs and this sign-based p-value unchanged. Thus the headline p-value alone cannot tell the viewer how many days of recovery are gained. Show the actual outcome scale before discussing practical benefit.

For a compact uncertainty panel, use a **separately labeled synthetic normal-estimator model**, not invented confidence intervals for the 20-pair data. Let an unbiased recovery-time-benefit estimator have an exactly normal sampling distribution with known standard error. Compare:

| Hypothetical estimate | Known SE | Two-sided p against zero | 95% confidence interval, using 1.96 |
|---|---:|---:|---|
| 0.2 days faster | 0.1 day | 0.0455003 | [0.004, 0.396] days |
| 2.0 days faster | 1.0 day | 0.0455003 | [0.04, 3.96] days |

Both have z = 2. The shared p-value does not identify the effect magnitude or the width of uncertainty. The intervals are rounded normal-model intervals, not Bayesian 95% probabilities assigned to these particular ranges. Their confidence level refers to coverage of the interval-producing procedure across repetitions under the model. Do not call every effect inside an interval equally likely. Any treatment decision additionally needs harms, costs, patient values, and relevant prior evidence; none is specified by these synthetic numbers.

If this inset overloads the film, keep its arithmetic in the companion notes and build the screen around one estimate, its interval, and a clearly labeled example of a meaningful benefit chosen for the hypothetical scenario. Do not omit uncertainty altogether.

## Suggested story beats, approximately 10–12 minutes

| Beat | Purpose and visual payoff |
|---|---|
| 0:00–0:45 · Doctors' survey | Establish an intellectually honest puzzle; confidence, keyed answer, later dispute. |
| 0:45–3:10 · The positive test | Let viewers commit to an answer, then regroup the synthetic patients. |
| 3:10–4:00 · Change the clinic | Same test performance, different positive-result composition. Keep both denominators visible. |
| 4:00–4:30 · Change the question | Explicit diagnostic/statistical distinction; a person-level diagnosis becomes a study-level inference. |
| 4:30–7:15 · Treatment and random assignment | Derive the null distribution, count the entire tail, reveal the p-value only after its meaning is visible. |
| 7:15–9:15 · Benefit and uncertainty | Reveal magnitude and interval; replace a green/red stamp with questions the viewer can answer. |
| 9:15–10:15 · Other studies | Transfer to a classroom randomized intervention and a production-process test. Identify the null model, statistic, and tail in each; do not import the clinical probabilities. |
| 10:15–11:00 · Return to the opening | Rewrite the vague question with explicit assumptions; ask what evidence is still needed. Optional one-sentence stopping-rule sequel pointer. |

For the nonmedical examples, introduce no real-world empirical claims without a new source check. A randomized classroom experiment can use the same label-assignment logic; a process test needs its own sampling model. Neither a small p-value nor an association alone establishes causation.

## Concrete comprehension checks before locking the script

- A viewer correctly names the reference group for the diagnostic false-positive rate and PPV, then explains why Clinic B changes the answer without changing the assumed test performance.
- When shown the trial tail, the viewer includes outcomes more extreme than the observation and states the no-effect assumption. Merely reciting “probability under the null” is insufficient.
- The viewer rejects both “p is the probability the treatment does nothing” and “p is the chance a positive diagnosis is false,” while explaining why.
- With one new set of counts, the viewer can reconstruct the denominator rather than recall 15.38%. With a new trial statistic, they can identify what must be specified before its tail can be calculated.
- The viewer asks about effect magnitude, uncertainty, study design, and harms before declaring a treatment useful. Do not score a useful uncertainty question as failure merely because it does not repeat a slogan.

These are proposed review checks, not measured learning outcomes. No arbitrary listener quota or accuracy target is asserted. A real misunderstanding discovered in review should change the relevant scene.
