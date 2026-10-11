# How to Count Fish You've Never Seen: draft video description

<!-- Draft YouTube description (R2, 2026-10-11). Every number here is an oracle key
(oracle.py) or a fact-sheet row (fact-sheet.md); every source is a fact-sheet row.
Chapter timestamps come from the narrated cut's cue logs (release bead mx-e-a-8y5.16). -->

Mark a thousand fish, let them go, and catch a thousand more: the hundred caught twice tell you
roughly how many fish no net ever touched. Starting from William Feller's 1950 textbook puzzle,
this video builds the four-cell table, the exact best guess, an honest 95% range, the ways the
method fails, and its uses from Laplace's France to software bugs and the war dead of Kosovo.

## Chapters

(timestamps after the narrated cut)

1. The fish nobody caught
2. The marks measure the net
3. Why the marks spread
4. Every possible netful
5. Turning it around
6. Same mountains, two questions
7. Is the score still rising?
8. The table balances
9. Ten thousand lakes
10. Dividing by zero
11. Turning it around again
12. A range that keeps its promise
13. One rule for every failure
14. France and the bugs
15. Kosovo, and the coda

Names to search: capture–recapture (mark–recapture), the hypergeometric distribution, maximum
likelihood, Chapman's estimator, confidence intervals by test inversion, multiple systems
estimation.

## Precisely

The narration simplifies in places. Here is the exact version of each simplification, by the
beat where it is said (chapter_beat). Notation: N lake size, M marked (the first catch), n second
catch, k caught twice; the table's cells are a = M − k, b = n − k and the dark cell
u = N − M − n + k.

**The lake in chapters 1 and 3 is a simulation.** 10,000 fish drawn independently from a habitat
with a weed bed in the north-west and a deep hole in the south-east (density proportional to
s(x, y) = 1 + 40·exp(−|p − (0.25, 0.75)|²/(2·0.08²)) + 20·exp(−|p − (0.75, 0.25)|²/(2·0.12²)) on
the unit square), by rejection sampling with SplitMix64 seeded 20261009. The model, the seed and
the three dip squares were fixed before the first run, and nothing was re-run to reach a number.

- **01_7.** One-square dips give 88,000 (weeds), 2,800 (open water) and 45,100 (deep end), 31.4×
  apart. That spread is this draw's; the model's expected spread at the same squares is 28.5×.
- **01_8.** Averaging many random dips does work: the 100 squares average exactly 10,000. But only
  22 of the 100 single dips land within a factor of 2 of it, and the median dip says 3,700. One
  net can't tell what share of the fish it reached; the marks measure that.
- **02_6.** E[K] = nM/N exactly, for every catch size (linearity of expectation). The soup guess
  Mn/k sets the observed k equal to that expectation.
- **03_4.** The fish move by dX = −D∇U dt + c·J∇U dt + √(2D) dW with U = −log s, D = 0.01,
  c = 0.02 (J a quarter turn), time step 0.01, reflecting shores; the 1,000 fish nearest the dock
  are marked. The share of marked fish across the nine regions goes from 0–65.9% at the start to
  9.3–10.7% at t = 400, while the regions' fish counts stay about 12.8 times apart. The tint is the
  simulated density, which drifts up to 14% from s itself because the circulation crosses the
  shore; the marks spread evenly whatever the density settles to.
- **03_5.** The nets are real catches from the t = 400 frame, each the size of the story's second
  catch: the 1,000 fish nearest each of the nine region centres hold 90 to 112 marked. Nets of 10 or 20 fish hold 0–3 and 0–5: small
  catches wobble (chapter 9).
- **03_6.** The two corners are the most and the least crowded of the nine regions, picked by fish
  count alone: 2,732 fish with 276 marked, and 214 with 23.
- **03_8.** After mixing, being marked is independent of position. That every fish is equally easy
  to catch is a separate assumption (chapter 13).
- **04_8.** The five chances exactly: 91/323, 455/969, 70/323, 10/323, 1/969.
- **04_9.** P_N(K = k) = C(M, k)·C(N − M, n − k) / C(N, n), the hypergeometric distribution. The
  piles add up by Vandermonde's identity: Σₖ C(5, k)·C(15, 4 − k) = C(20, 4) = 4,845.
- **05_7.** Summed over every lake size, the scores (likelihoods) give Σ_N L(N) = Mn/(k(k − 1))
  when k ≥ 2: 50/3 for the mystery lake, 10,000/99 ≈ 101.01 for Feller's catch. For k ≤ 1 the sum
  is infinite. So the scores are not probabilities over lake sizes. Feller (1950, p. 37): "It is,
  therefore, meaningless to ask for the probability that n is greater than, say, 6000." (His n is
  our lake size N.)
- **06_4.** Each fixed-size row of the landscape peaks at ⌊(n + 1)(M + 1)/(N + 2)⌋, tied with the
  count one below when that quotient is whole (checked on every lake of up to 40 fish). Each
  fixed-count curve (k ≥ 1) rises up to ⌊Mn/k⌋ and falls after it, with the tie of 08_5 when
  Mn/k is whole.
- **07_1–07_2.** L(N)/L(N − 1) = (N − M)(N − n) / (N(N − M − n + k)), for N > M + n − k. In the
  mystery lake: 529/528 = (23·23)/(33·16) going from 32 to 33, and 576/578 = (24·24)/(34·17) going
  from 33 to 34. Code walks a score curve the same way, one multiplication per step, in logarithms
  for big lakes (11_2).
- **07_4–07_5.** The top minus the bottom is Mn − kN, and Mn − kN = ab − k·u exactly (a polynomial
  identity).
- **08_4.** k·u = a·b says the table's odds ratio is 1: the 2×2 picture of independence. A real
  catch's table wobbles around it; at Feller's lake k has standard deviation 9.
- **08_5.** The best lake size is ⌊Mn/k⌋, tied with Mn/k − 1 when Mn/k is whole and
  k < min(M, n); with k = 0 there is no best size. Feller's 9,999 and 10,000 tie exactly; the
  neighbouring ratios are 80,982,001/80,981,901 and 81,018,001/81,018,101. Floating point may
  break the tie either way: a log-likelihood through `lgamma` prefers 10,000 by 7.3 × 10⁻¹²,
  while a float ratio scan ties exactly. Feller (1950, p. 38) gives 10,000.
- **09_3.** The exact standard deviation of k is 9.00 (variance 90,000/1,111); the binomial
  approximation, which ignores that the lake is finite, gives 9.49. The panel's 10,000 runs give
  8.96.
- **09_5.** Exactly, Mn/K has quantiles 8,474.58 / 10,000 / 12,048.19 (k = 118 / 100 / 83), mean
  10,082.50 given K > 0, and standard deviation 924.36. The panel's runs average 10,088.6 (standard
  error 9.2).
- **10_2.** P(K = 0) = 0.0720, about 1 in 13.9. In numpy, `M * n / k` at k = 0 gives `inf` with a
  RuntimeWarning.
- **10_3.** Chapman's estimator (M + 1)(n + 1)/(k + 1) − 1 has mean
  N − (N + 1)·C(N − M, n + 1)/C(N + 1, n + 1): exactly N when M + n ≥ N, and never low by more than
  N·e^(−Mn/N), which is under 2% (1.83%) once you expect 4 caught twice. In this small study (a lake
  of 1,000, marking 50 and catching 50) you expect 2.5, Chapman's mean is 935.22 (−6.48%), and the
  plain guess averages 1,241.24 (+24.12%) over the experiments that catch any. With none caught
  twice, Chapman still answers 2,600, a sign the experiment was too small. The +1s are two
  absorption identities, (M + 1)/(k + 1)·C(M, k) = C(M + 1, k + 1) and
  (N + 1)/(n + 1)·C(N, n) = C(N + 1, n + 1). The identity and its "exactly when M + n ≥ N" are
  Wittes (1972), as reported by Sadinle (2008) and Dettloff (2023); the oracle checks them exactly.
- **Planning an experiment (10–12).** The typical error (one standard deviation) is about
  1 ÷ √(caught twice): about ±32% at 10 caught twice, ±10% at 100. Catching a big share of the
  lake trims it by √((1 − M/N)(1 − n/N)). An exact 95% range is roughly twice as wide, and
  lopsided upward when few are caught twice: −31% / +52% at 25, −16% / +21% at 100, −7.5% / +8.5%
  at 400 (lake of 10,000, M = n).
- **11_2.** P_1900(K = 100) = (1000!)²/(100!·1900!) = 5.35 × 10⁻⁴³⁰. That is below float64's
  smallest positive number, 4.94 × 10⁻³²⁴ (its smallest normal number is 2.2 × 10⁻³⁰⁸), so a
  float gives 0.0; code works in logarithms.
- **11_3–11_5.** At a million fish E[K] = 1. At the best size the chance of exactly 100 caught twice
  is 4.43%; exactly 50 heads in 100 fair flips has chance 7.96%, and 50 is the most likely count.
- **12_1–12_2.** The careful range is an exact test inversion: its lower end is the smallest N
  with P_N(K ≤ 100) > 2.5%, its upper end the largest N with P_N(K ≥ 100) > 2.5%, giving
  [8,400, 12,073]. Each one-sided test rejects a true lake size with chance at most 2.5%, so the
  procedure covers the true size at least 95% of the time, for every true size. Checked on 27,901
  lake sizes from 1,000 to 1,000,000 (with M = n = 1,000): the lowest coverage is 95.007%, at
  10,967; at 10,000 it is 95.45%. In the panel's 10,000 runs, 464 ranges missed 10,000, against
  454.7 expected.
- **12_3.** Feller (1950, p. 38): "These figures would justify a bet that the true number n of fish
  lies somewhere between 8500 and 12,000." Exactly, the tails at 8,500 and 12,000 are 3.47% and
  2.89%, so his range is a 93.6% range. A probability for the lake size itself needs a starting
  belief: with a flat prior on N ≥ 1,900 (cut off at 400,000), the middle 95% of the posterior is
  8,536 to 12,186, median 10,121. Almost the same numbers, a different question.
- **12_5.** The ±9% is the symmetric approximation √((1 − M/N)(1 − n/N)/k). The exact standard
  deviation of Mn/K (given K > 0) is 924, 9.2% of the lake. Marking 2,000 and catching 500 is the
  same experiment as marking 500 and catching 2,000: the law of K is identical term by term, and
  the standard deviation is 894 both ways. With the whole lake caught twice the range is
  [10,000, 10,000], since a lake of 10,001 gives that catch with chance 1/10,001.
- **13_1.** Mn/k = M ÷ (marked share of the second catch). It aims at a lake of L fish
  (Mn/E[k] = L) exactly when that share is, on average, M/L. Even then it runs a little high
  (09_5). Ideal lake: 10,000. Deaths: 10,000, the lake on marking day. Newcomers: 12,000, the lake
  at the second catch. Lost marks: 12,500.
- **13_3–13_9.** The failure models: a true lake of 10,000, M = n = 1,000, 400 seeded runs each
  (medians shown). Bold fish: 3 in 10 fish 3× as catchable in both catches (sampling without
  replacement in proportion to weight): 7,752. Fish that love the bait: marked fish 1.5× as
  catchable in the second catch: 7,143; that avoid it, ⅔×: 14,493. Lost marks: each mark lost
  with chance 1 in 5: 12,500. Newcomers: 2,000 unmarked fish join: 12,122. Deaths: each fish dies
  with chance 3 in 10, about 7,000 left: 10,000. Unmixed marks: all marks in one half of the lake,
  the second catch 800 fish from one half and 200 from the other: 6,250 fishing the marked half,
  25,000 the other. Feller (1950, p. 37) excludes "situations where the two catches are made at one
  locality and within a short time."
- **14_2–14_3.** Laplace's figures (Essai philosophique; tr. Truscott & Emory 1902, pp. 66–67):
  in thirty départements spread over France, officials counted, at his request, communes chosen
  because they "would be able to furnish the most exact information": 2,037,615 people on
  23 September 1802, and 110,312 boys and 105,287 girls born in 1800–1802, so 71,866⅓ births a
  year and 28.352845 people per birth. Taking France's births as an assumed 1,000,000 a year gives
  28,352,845. His odds, "a bet of almost 300000 against 1" that the error is under half a million,
  come from his own calculation, in which the chosen communes stand in for all of France.
- **14_4.** H. D. Mills seeded bugs on purpose: the planted bugs are the marked fish (IBM report
  FSC-72-6015, 1972; his collected papers date the article 1970). p. 73: "In our application, of
  course, 'lake' is synonymous with 'system,' 'unmarked fish' is synonymous with 'indigenous
  errors,' 'marked fish' with 'calibration errors,' and 'catching fish' with 'testing process.'"
- **14_5.** The 30 / 24 / 12 example is made up. Its exact 95% range is 47 to 94 bugs (5 to 52
  still hiding); Chapman gives 58.6 ± 8.4.
- **14_6.** Reviewers who read independently, each finding every bug with their own fixed chance,
  keep the formula right on average: E[a]·E[b]/E[k] = N. In a simulation (60 bugs; chances 0.5 and
  0.4) the median estimate is 60 and the careful range holds 60 in 97% of runs. Bugs that are
  easier for everyone break it: with 30 easy bugs (0.8 and 0.7) and 30 hard ones (0.2 and 0.1),
  the median estimate is 41.25 and the range lies wholly below 60 in 77% of runs, never above.
  Briand et al. (2000), on software inspections: "When the number of inspectors is too small, no
  model is sufficiently accurate and underestimation may be substantial."
- **15_1.** Ball et al. (2002) drew on four sources (interviews by ABA/CEELI, Human Rights Watch and
  the OSCE, and exhumation records for the ICTY). About 10,000 reports by name gave 4,400 unique
  victims, a count rather than an estimate. Multiple systems estimation with log-linear models on
  the four lists gave 10,356 Kosovar Albanians killed between 20 March and 22 June 1999, with an
  interval of 9,002 to 12,122. Spagat (2014, p. 9) compared it with the Kosovo Memory Book's 9,790
  violent deaths for the period (which he gives as 20 March to 20 June), "right in the middle of
  the BBSDA range", and noted that false reports in the lists could make the estimate "a bit too
  high".

## Code

The panel from chapter 9, runnable as shown (Python 3 with numpy):

```python
import numpy as np
from math import comb
rng = np.random.default_rng(20261009)
M = n = 1_000; N = 10_000
marked = np.arange(N) < M
k = np.array([marked[rng.choice(N, n, replace=False)].sum() for _ in range(10_000)])
est = M * n / k
p = [comb(M, j) * comb(N - M, n - j) / comb(N, n) for j in range(n + 1)]
print(est.mean(), sum(M * n / j * p[j] for j in range(1, n + 1)))
```

It prints `10088.586031204479 10082.498763370464`: the simulated average and the exact one,
0.66 standard errors apart. Those digits were printed with numpy 2.5.4; a different numpy may
draw a different random stream and print different digits. Every number in the video is computed
by `oracle.py` in the repository (repository link).

## Sources

- W. Feller, *An Introduction to Probability Theory and Its Applications*, Vol. I, 1st ed. (1950),
  ch. 2 §5, Example (e) "A Sampling Problem", pp. 37–38. https://archive.org/details/dli.ernet.5666
- Schnabel, "The Estimation of Total Fish Population of a Lake", *The American Mathematical
  Monthly* 45(6), 1938, from p. 348. doi:10.2307/2304025. People were estimating fish this way
  before Feller.
- Goudie and Goudie, on how the modern estimator grew from the "fishing coefficient",
  *J. R. Statist. Soc. A* 170(3):825–839, 2007. doi:10.1111/j.1467-985X.2007.00479.x (abstract read).
- Chapman, "Some properties of the hypergeometric distribution with applications to
  zoological sample censuses", *Univ. California Publ. Statist.* 1(7), 1951 (citation checked by
  a research agent only).
- Sadinle, CERAC Working Paper 13 (2008), p. 4, and Dettloff, *Fisheries Research* 265:106756
  (2023), §2.2.2, both reporting Wittes (1972) on Chapman's estimator.
- Stigler, "The Epic Story of Maximum Likelihood", *Statistical Science* 22(4):598–620,
  2007. doi:10.1214/07-STS249 (for the history behind "score": Feller, p. 38, credits R. A. Fisher).
- Laplace, *Essai philosophique sur les probabilités* (1840 ed., pp. 82–83); English
  translation by Truscott and Emory (1902), pp. 66–67.
  https://www.gutenberg.org/ebooks/58881
- H. D. Mills, "On the Statistical Validation of Computer Programs" (1970), in *Software
  Productivity*, Article 9, p. 73; IBM Report FSC-72-6015 (1972), as listed in NBS Special
  Publication 500-75 (1981).
- Briand et al., *IEEE Transactions on Software Engineering* 26(6):518–540, 2000.
  doi:10.1109/32.852741 (abstract read).
- Ball et al., *Killings and Refugee Flow in
  Kosovo, March–June 1999: A Report to the International Criminal Tribunal for the Former
  Yugoslavia* (3 January 2002), pp. 3–6 and 41.
  https://www.icty.org/x/file/About/OTP/War_Demographics/en/s_milosevic_kosovo_020103.pdf
- Spagat, *A Triumph of Remembering: Kosovo Memory Book* (10 December 2014), pp. 8–9.
  https://www.hlc-rdc.org/wp-content/uploads/2015/03/Michael-_Spagat_Evaluation_of_the_Database_KMB_December_10_2014.pdf
