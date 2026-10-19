# Medical p-values proposal: factual, mathematical, and pedagogical review

2026-10-16. Scope: the new user-directed medical opening and proposed examples in `evidence/pvalues-medical-opening.md`. Sources were read or inspected through publisher indexing; arithmetic was evaluated independently with Python's standard-library `math.comb` and `math.erfc`. This is the same agent's self-audit of its editorial proposal, not an independent audience study or a new independent-review round. No main plans, Beads, source code, audio, or video were changed.

## Result

The proposal is suitable for an evidence-bounded script brief. The diagnostic-to-trial bridge is mathematically sound only with the explicit distinction already written into the proposal. Optional stopping should cease being the main storyline for this topic; it can be a later extension. The most useful change is the visible reference-group transformation followed by a separately constructed null world.

No listening, rendered-visual, learning-effect, or publication-readiness claim is made. Competition and differentiation review still belongs to the existing topic-selection process; this note does not assert that the medical angle has little existing coverage.

## Source controls

- The survey opening uses a bounded historical sample and the phrase **keyed answer**. It makes no national prevalence claim about doctors' understanding. It does not confuse an overall test score with one p-value item.
- The criticism is attributed as criticism, and not treated as a validated replacement measure of comprehension. The full original clinical stem remains unverified. If the film uses a facsimile or an exact full-question quotation, obtain the original appendix first. The proposed paraphrased opening does not require that step.
- No response distribution is invented for the distracting wrong answer. “None of the choices is precise” cannot establish every participant's mental model.
- The source section excludes the commentary's unsupported universal posterior conversion. A posterior calculation would require an explicit prior and likelihood/model structure, not a p-value alone.
- The natural-frequency experiment supports the choice of representation for diagnosis. It is not direct evidence that this film will teach p-values, or that representation explains every survey mistake.
- The manuscript should cite the [original survey](https://jamanetwork.com/journals/jama/fullarticle/208638), the [published critique](https://link.springer.com/article/10.1186/s12874-020-01051-6), the [ASA statement](https://www.stat.berkeley.edu/~aldous/Real_World/ASA_statement.pdf), and the [diagnostic-format experiment](https://pubmed.ncbi.nlm.nih.gov/9609869/) at their relevant claims. No secondary commentary is needed to support the mathematical examples.

## Arithmetic and model checks

| Check | Recomputed result / condition |
|---|---|
| Clinic A positive results | 90 true + 495 false = 585; 90/585 = 0.1538461538 |
| Clinic A negative results | 10 false + 9,405 true = 9,415; total cohort = 10,000 |
| Clinic B positive results | 900 true + 450 false = 1,350; 900/1,350 = 2/3 |
| Clinic B negative results | 100 false + 8,550 true = 8,650; total cohort = 10,000 |
| Assignment tail, K >= 15 | sum of binomial coefficients = 21,700; divide by 1,048,576 = 0.0206947327 |
| Assignment tail, K >= 14 | numerator 60,460; p = 0.0576591492 |
| Separate normal illustration | erfc(2/sqrt(2)) = 0.0455002639; both stated intervals equal estimate ± 1.96 SE |

The trial calculation is exact for independent randomized assignment within each of 20 fixed pairs, complete outcomes, no within-pair ties, no interference, and the sharp no-individual-effect null. Matching alone does not create the coin-flip distribution; random assignment under that null does. The one-sided direction and analysis must be predeclared. Do not collapse this sharp null into the weaker assertion of zero mean effect. Real ties/dropout/selection require an explicitly appropriate procedure; silently deleting inconvenient outcomes would break the example.

The normal illustration has its own stated model and is not inferred from the sign-test dataset. Its intervals use rounded 1.96 critical values. Scaling differences preserves the sign test, but clinical importance does not scale with a p-value. Cohort prevalence is applicable only under the toy population assumptions; the examples do not estimate any individual viewer's medical risk.

The direct conditional reversal of the false-positive rate has **absence** of disease as its outcome. PPV is the complement of that reversed probability. The proposal now writes this distinction explicitly, preventing a quiet disease/no-disease substitution at the central bridge.

## Production hazards to catch at script and animatic review

1. **Denominator confusion introduced by animation.** Recoloring tiles when regrouping would obscure what changed. Retain identity, show subgroup totals, and use labels/patterns alongside colors.
2. **Letting the diagnostic scene consume the entire promise.** It prepares conditional reasoning; the trial scene must still derive an actual p-value. Keep the explicit bridge card and obtain a fresh comprehension check on that transition.
3. **Tail versus point probability.** The highlighted region must include the observed result and every more extreme result in the chosen direction. A single highlighted bar would teach the wrong quantity.
4. **Threshold magic.** The neighboring trial outcome contains different evidence; the script should reject certainty labels without saying the data are equivalent.
5. **Null-model failure versus a proved treatment story.** A small p-value can challenge the modeled world. It does not, by itself, establish effect size, exclude all other failures, or select a clinically worthwhile treatment.
6. **Confidence-interval inversion.** Do not narrate the particular frequentist interval as a 95% posterior belief. Avoid implying that values inside it are equally plausible or that an interval spanning zero proves no benefit.
7. **Historical survey used as present-day indictment.** Keep the year and sample visible, retain the uncertainty of the question, and make the intellectual puzzle the hook.
8. **Overclaiming the learning design.** The proposed checks test transfer and explanation. A count graphic or an elegant script is not evidence of comprehension until someone actually reviews it.

## Remaining bounded work

The script author can use the recommended opening now. Retrieve the original appendix only if exact-question reproduction is retained. Keep the existing competitor-viewing task, then review the completed script/animatic for the hazards above and listen to the rendered narration for numerical phrases and qualifiers. These are production prerequisites, not work performed by this research note. No new bureaucracy or full-video render is needed to accept this revised brief.
