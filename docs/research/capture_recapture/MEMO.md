> 2026-10-09. All computed numbers below come from the scripts in `scripts/` (see `scripts/README.md`).

# Research memo: "How many fish are in the lake?" Capture–recapture, the hypergeometric distribution, and Feller's maximum-likelihood argument

Every claim carries one of three tags:
- **[V]** verified, either from a cited source or by my own exact computation.
- **[F]** Feller's own text, checked against page scans.
- **[P]** my creative proposal.

Numbers come from exact rational arithmetic (`math.comb`, `fractions.Fraction`) unless marked *float* or *sim*. Scripts are in `docs/research/capture_recapture/scripts/` (`verify.py`, `verify2.py`, `appendix.py`). A condensed version is in the Appendix.

---

## 0. Headline findings and corrections to the brief

1. **Where it is [F].** 3rd ed. (1968), Vol. I, Ch. II, §6 "The Hypergeometric Distribution", **Example (c)**, "Estimation of the size of an animal population from recapture data", **pp. 45–46**. It is example (c), not (b); (b) is the senators example. The ratio is eq. (6.3) and there is a footnote 13. In the **1st ed. (1950)** the same example is §II.5, **Example (e)**, "A Sampling Problem", pp. 37–38.
2. **Your numbers are right [F].** n₁ = r = 1000, k = 100, n̂ = 10,000.
3. **Feller's example is the tie case [V].** Because n₁r/k = 10,000 is an integer, q₁₀₀(9,999) = q₁₀₀(10,000) *exactly*. Feller writes that the maximum is at "the largest integer short of n₁r/k", which is 9,999, "so that n̂ equals about n₁r/k". He then reports n̂ = 10,000. Both values are maximum-likelihood estimates. A mathematician watching will notice if the video glosses over this.
4. **Feller's interval [V].** His "8500 to 12,000" uses normal-approximation tails of 0.04 and 0.03. The exact tails are **0.0347** and **0.0289**, so his range is about a 93.6% interval. The exact 95% interval (by inverting the hypergeometric test) is **[8,400, 12,073]**.
5. **No exercise to borrow [V].** I OCR'd the whole 3rd edition. Chapter II has no capture–recapture exercise, though several exercises are closely related (§1.3).
6. **History corrections [V].**
   - Petersen (1896) **never actually computed** N̂ = Mn/k (Goudie & Goudie 2007).
   - Laplace's 1802 figure is **28,352,845**.
   - Mills' error seeding is from **1972**, not 1970.
   - Graunt's method is ratio estimation, not capture–recapture.
7. **The best "aha" for the script [V].** Feller's ratio test simplifies exactly to comparing the lake's tag fraction M/N with the catch's tag fraction k/n. The rigorous argument and the naive proportion guess agree, up to rounding down to an integer.

---

## 1. Feller's text

### 1.1 Key passages, 3rd ed. pp. 44–46 [F]

**Formulas.**
- (6.1): q_k = C(n₁,k)·C(n−n₁, r−k) / C(n,r).
- (6.2), the symmetric form: q_k = C(r,k)·C(n−r, n₁−k) / C(n,n₁).

**The setup:**
> "Suppose that 1000 fish caught in a lake are marked by red spots and released. After a while a new catch of 1000 fish is made, and it is found that 100 among them have red spots. What conclusions can be drawn concerning the number of fish in the lake? This is a typical problem of *statistical estimation*."

**The assumptions:**
> "We assume naturally that the two catches may be considered as random samples from the population of all fish in the lake. (In practice this assumption excludes situations where the two catches are made at one locality and within a short time.) We also suppose that the number of fish in the lake does not change between the two catches."

**Notation.**
- n = the unknown number of fish in the lake.
- n₁ = the first catch ("They play the role of red balls").
- r = the second catch.
- k = the number of red fish in the second catch.
- q_k(n) = the probability that the second catch contains exactly k red fish.

**N is fixed, not random.** "Notice that we consider n as an unknown fixed number which in no way depends on chance." The 1950 edition went further: "It is, therefore, meaningless to ask for the probability that n is greater than, say, 6000." That sentence was dropped by the 3rd edition.

**The only certainty.** n ≥ n₁ + r − k = 1900. If n = 1900, the two catches "between them exhaust the entire population", with probability (1000!)²/(100!·1900!), "of the order of magnitude 10⁻⁴³⁰." Feller concludes that "common sense bids us to reject" this, and similarly rejects n = 1,000,000.

**Naming the method.** The maximizing n "is called the *maximum likelihood estimate* … This notion was introduced by R. A. Fisher."

**The ratio (6.3):**

q_k(n)/q_k(n−1) = (n−n₁)(n−r) / ((n−n₁−r+k)·n)

> "greater than or smaller than unity, according as nk < n₁r or nk > n₁r … first increases and then decreases; it reaches its maximum when n is the largest integer short of n₁r/k, so that n̂ equals about n₁r/k. In our particular example … n̂ = 10,000."

**The interval:**
> "let us test the hypothesis that n is smaller than 8500 … using the normal approximation of chapter VII, we find easily that x ≈ 0.04. Similarly, if n = 12,000, the probability that the second sample contains 100 or more red fish is about 0.03. These figures would justify a bet that the true number n of fish lies somewhere between 8500 and 12,000."

**Footnote 13**, a nice detail for the video:
> "This example was used in the first edition without knowledge that the method is widely used in practice."

It cites Bailey (1951, *Biometrika* 38:293–306) and Chapman (1951, *Univ. Calif. Publ. Statist.* 1:131–160). So Feller came up with the fish example on his own.

### 1.2 Feller's numbers, recomputed [V]

| Feller's claim | Exact value | Notes |
|---|---|---|
| q₁₀₀(1900) ~ 10⁻⁴³⁰ | 10^−429.27 ≈ 5.4×10⁻⁴³⁰ | ✓ |
| n̂ = 10,000 | {9,999, 10,000} tie; q = 0.0442892 at both | ratio at N = 10,000 is exactly 81,000,000/81,000,000 = 1 |
| P(K ≤ 100 \| n = 8500) ≈ 0.04 | **0.03468** | Normal approx to the hypergeometric gives 0.0366 (with continuity correction). Normal approx to the binomial, which ignores the finite-population factor, gives 0.042 without / 0.046 with correction. Feller's 0.04 matches the binomial–normal version. |
| P(K ≥ 100 \| n = 12,000) ≈ 0.03 | **0.02887** | binomial–normal: 0.028 / 0.032 |
| "8500–12,000" | 1 − 0.0347 − 0.0289 = 0.936 | roughly a 94% interval |

### 1.3 Related passages and exercises in the 3rd ed. [V]

- II.6 example (a), Quality inspection: the same inverse problem, "beyond the scope of the present book".
- II.10 problem 20, p. 56: the probability that a sample contains none of N prescribed elements. This is P(K = 0), the "no tagged fish" case.
- II.11 problem 2, p. 59: the hypergeometric distribution tends to the binomial in large populations.
- II.12 problem 8, p. 64: prove the Vandermonde identity (6.4) by induction ("the terms … should add to unity").
- VI.10 problems 35–36, p. 172: binomial and Poisson approximations to the hypergeometric.
- VII.7 problem 10, p. 194: the normal approximation to the hypergeometric, the tool behind his 0.04 and 0.03.
- IX.5 example (d), pp. 232–233: mean and variance via indicator variables. (5.14): E(S_r) = rb/(b+g), Var(S_r) = rbg/(b+g)² · (1 − (r−1)/(b+g−1)). This is the cleanest proof of E[K] = nM/N.

**Not confirmed.** I did not see the wording of the 2nd edition (1957). The 1st edition (open full text) and 3rd edition (page scans) were both read directly.

---

## 2. The mathematics

**Notation used below:** N = lake size, M = tagged fish, n = second catch, k = tagged fish in the second catch (Feller's n, n₁, r, k).

### 2.1 The hypergeometric pmf [V]

P(K = k) = C(M,k)·C(N−M, n−k) / C(N,n).

"Random catch" means every one of the C(N,n) possible netfuls is equally likely. The numerator counts the netfuls with exactly k tagged fish: choose which k of the M tagged fish, and which n−k of the N−M untagged fish.

**Toy lake** (N = 20, M = 5, n = 4; C(20,4) = 4,845 possible netfuls):

| k | 0 | 1 | 2 | 3 | 4 |
|---|---|---|---|---|---|
| count | 1365 | 2275 | 1050 | 150 | 5 |
| probability | .2817 | .4696 | .2167 | .0310 | .0010 |

**Moments** (Feller IX.5):
- E[K] = nM/N.
- Var K = n(M/N)(1−M/N)(N−n)/(N−1).
- At N = 10,000 with Feller's catches: E[K] = 100, Var = 81.008, SD = **9.000**. The binomial approximation would give 9.487.

### 2.2 Likelihood and the ratio trick [V]

L(N) = P(K = k | N), seen as a function of N with the data held fixed. The identity L(N)/L(N−1) = [(N−M)/(N−M−n+k)]·[(N−n)/N] was checked exactly in all test cases. The key step is a cancellation:

(N−M)(N−n) − N(N−M−n+k) = **Mn − kN**.

The N² and (M+n)N terms cancel. Therefore:
- **L(N) > L(N−1) ⇔ N < Mn/k ⇔ M/N > k/n.**
- If Mn/k is not an integer, the MLE is unique: ⌊Mn/k⌋.
- If Mn/k is an integer, Mn/k − 1 and Mn/k tie.
- If k = 0, L increases forever and there is no MLE.
- The MLE is always feasible: Mn/k ≥ M+n−k ⇔ (M−k)(n−k) ≥ 0.

**On-screen ratio numbers** (numerator − denominator = Mn − kN, exactly):

| N | numerator / denominator | ratio |
|---|---|---|
| 9,999 | 80,982,001 / 80,981,901 | > 1 (difference +100) |
| 10,000 | 81,000,000 / 81,000,000 | = 1 |
| 10,001 | 81,018,001 / 81,018,101 | < 1 (difference −100) |
| toy N = 33 (M = 10, n = 10, k = 3) | 529/528 | > 1 |
| toy N = 34 | 576/578 | < 1 |

So the toy's peak is at 33, since 100/3 = 33.3.

**Shape of Feller's likelihood** (L(N)/L_max):

| N | 5,000 | 7,000 | 8,000 | 8,500 | 9,000 | 9,500 | 10,000 | 10,500 | 11,000 | 12,000 | 13,000 | 15,000 | 20,000 | 1,000,000 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| L/L_max | 6.0e−20 | 7.7e−5 | .032 | .171 | .486 | .847 | 1 | .866 | .585 | .152 | .023 | 2.1e−4 | 2.6e−10 | 3.8e−162 |

- The half-maximum range is 9,019–11,148.
- The curve is skewed right: 9,000 scores 0.49 while 11,000 scores 0.58.
- L_max = q₁₀₀(10,000) = **0.04429**. Even under the best-fitting N, the exact observed count had under a 5% chance. Also P(95 ≤ K ≤ 105 | N = 10,000) = 0.459.

**Likelihood is not a probability distribution over N.**
- Σ_N q₁₀₀(N) = **101.0101…**, which equals Mn/(k(k−1)) = 10⁶/9,900.
- The toy gives 16.667 = 100/6.
- I verified this identity numerically for 7 parameter sets. I have not proved it or found a reference, so treat it as trivia.

### 2.3 Lincoln–Petersen, method of moments, and a precision rule [V]

**The estimator.** N̂ = Mn/k. It differs from Feller's MLE by less than 1.

**Why it works:** the catch is a miniature lake. If mixing is good, k/n ≈ M/N.

**Method of moments.** Each fish in the net is tagged with probability M/N, whatever dependence there is between fish. By linearity of expectation, E[K] = nM/N. Setting k = E[K] gives the same estimator.

**Sensitivity.** One recaptured fish moves the estimate by about 100: k = 99 → 10,101; k = 101 → 9,900.

**Precision rule.** The delta-method coefficient of variation is CV ≈ √(1/k − 1/n). For Feller's numbers that is 9.5%, matching SE ≈ 949. In words: **precision is governed mainly by the number of recaptures k, not by what fraction of the lake you catch.**

### 2.4 Bias, the k = 0 problem, and Chapman [V]

**k = 0 problem.** Whenever n ≤ N−M, P(K = 0) > 0, so E[Mn/K] is infinite or undefined. For Feller's numbers P(K = 0) = 4.4×10⁻⁴⁹, so this is a technicality. In small studies it is not.

**Upward bias given k > 0.** Because 1/k is convex (Jensen's inequality), the estimator overshoots on average:

| Lake | Mean of Mn/k given k > 0 | Bias |
|---|---|---|
| Feller (N = 10,000) | 10,082.5 | +0.82% |
| N = 1,000, M = n = 100 | 1,102.0 | +10.2% |
| N = 1,000, M = n = 50 | 1,241 | +24%, with P(K = 0) = 7.2% |

**Chapman's estimator:** N̂_C = (M+1)(n+1)/(k+1) − 1.
- Exact identity, which I derived and checked with exact fractions: **E[N̂_C] = N − (N+1)·C(N−M, n+1)/C(N+1, n+1)**.
- The correction term is zero iff M + n ≥ N, so the estimator is then exactly unbiased.
- Proof idea: (M+1)/(k+1)·C(M,k) = C(M+1,k+1) and (n+1)/C(N,n) = (N+1)/C(N+1,n+1). The sum then becomes (N+1)·P(K′ ≥ 1) for a hypergeometric K′ with parameters N+1, M+1, n+1.
- Attribution: the history agent found the proof credited to Wittes (1972), and I believe Seber (1982) states it; I have not seen either source, so cite the identity, not a page.
- Bias in practice:

| Expected recaptures E[k] | Example | Chapman bias |
|---|---|---|
| 100 | Feller | −3.6×10⁻⁴⁵ |
| 10 | N = 1,000, M = n = 100 | −0.012 fish |
| ≈ 4 | | −1.1% to −1.6% |
| 2.5 | N = 1,000, M = n = 50 | −6.5% |

- These numbers support the Robson & Regier (1964) planning rule that Mn should exceed 4N.

**Bailey's estimator** (binomial model): M(n+1)/(k+1).

**Variances for Feller's data:**
- Lincoln–Petersen, approximate M²n(n−k)/k³ = 900,000, SE **949**.
- Chapman/Seber, (M+1)(n+1)(M−k)(n−k)/((k+1)²(k+2)) = 780,028, SE **883**.
- Chapman point estimate = 1,001,900/101 = **9,919.8**.

### 2.5 Ninety-five-percent intervals for Feller's data [V]

| Method | Interval |
|---|---|
| Feller, normal approx (≈ 93.6%) | 8,500 – 12,000 |
| **Exact test inversion** | **8,400 – 12,073** |
| Likelihood ratio (relative likelihood ≥ e^(−1.92) = 0.1465) | 8,445 – 12,021 |
| Sadinle (2009) transformed logit (from Wikipedia formula) | 8,392 – 11,927 |
| Wald ± 1.96 SE around Chapman (symmetric, *misleading*) | 8,189 – 11,651 |
| Bayesian, flat prior on N | 8,536 – 12,186 (median 10,121, mean 10,184) |
| Bayesian, prior ∝ 1/N | 8,475 – 12,073 (mode 9,919, median 10,039) |

**How the exact interval was computed.**
- Lower limit: the smallest N with P(K ≤ 100 | N) > 0.025. At 8,399 that probability is 0.02494; at 8,400 it is 0.02503.
- Upper limit: the largest N with P(K ≥ 100 | N) > 0.025. At 12,073 it is 0.02503; at 12,074 it is 0.02498.
- Both limits were checked with exact fractions.

**Toy lake (M = 10, n = 10, k = 3).** The MLE is 33, the exact 95% interval is **[19, 138]**, the LR interval is [19, 104], and Chapman gives 29.25. Three recaptures tell you very little.

### 2.6 Simulation plan and verified results [V]

The estimate is a deterministic function of k. So the exact sampling distribution is just the hypergeometric pmf pushed through the map k ↦ Mn/k. Use it to check the Monte Carlo, and to show where the skew comes from: the hyperbola stretches small-k outcomes far to the right.

**Exact quantiles of Mn/k (2.5% / 50% / 97.5%):**
- Feller's lake: **8,475 / 10,000 / 12,048**. These correspond to k = 118, 100, 83.
- N = 1,000, M = n = 100: **625 / 1,000 / 2,000**. That is 375 below the truth and 1,000 above it.

**Seeded simulation** (numpy, seed 20261009, 10,000 experiments each):

| Lake | k = 0 | Mean | Median | 2.5–97.5% | Skew | Chapman mean |
|---|---|---|---|---|---|---|
| N = 10,000, M = n = 1,000 | 0 | 10,080 | 10,000 | 8,475–12,048 | 0.53 | 9,997.6 |
| N = 1,000, M = n = 100 | 0 | 1,103 | 1,000 | 625–2,000 | 4.7 | 1,000.3 |
| N = 1,000, M = n = 50 | 663 | 1,229 | 833 | 417–2,500 | — | 920 |

### 2.7 When the assumptions fail: direction of bias [V by simulation; intuition is mine]

Each row below is a simulated median estimate for a true lake of 10,000 with M = n = 1,000; "ideal" gives 10,000.

| Violation | Effect on k | Direction | Sim median |
|---|---|---|---|
| **Heterogeneous catchability** (30% of fish 3× easier to catch in both catches) | The same easy fish get caught twice, so k is inflated | **Underestimate**. This is the main real-world bias. | 7,752 |
| Trap-happy (tagged fish 1.5× more catchable) | k up | Underestimate | 7,143 |
| Trap-shy (tagged fish 0.67×) | k down | Overestimate | 14,493 |
| Tag loss, 20% | Tagged fish look untagged | Overestimate, about N/(1−loss) | 12,500 |
| Recruitment (+2,000 untagged fish) | Tags diluted | Estimates the population at the *second* catch | 12,122 |
| 30% random deaths, no births | Tag fraction unchanged | Still valid for the population at the *first* catch | 10,000 |
| No mixing; second catch at the release site | k up | Underestimate. This is Feller's "same locality" warning. | 6,250 |
| No mixing; second catch on the far side | k down | Overestimate | 25,000 |

---

## 3. History [V via sub-agent, with my spot-checks of primary text]

- **Graunt (1662).** He estimated London at 8 × 48,000 = **384,000**: 12,000 christenings → 24,000 "teeming women" → 48,000 families × 8 people. This is a forerunner of ratio estimation and sampling, **not** capture–recapture. Source: Wikisource, ch. XI.
- **Laplace, 1783 memoir (published 1786).** He proposed sampling parishes to get the population-to-births ratio. Using ratio ≈ 26 × 973,054½ births gives **25,299,417**. He asked for a sample of 771,469 people to get odds of 1000:1 that the error is under half a million.
- **Laplace, 1802 survey** (*Essai philosophique*):
  - 30 departments, **2,037,615** inhabitants.
  - **215,599** births over 3 years, i.e. 71,866⅓ per year.
  - Ratio **28.352845**. Taking France's births as 1,000,000 gives "**28352845** inhabitants, it is a bet of almost 300000 against 1 that the error of this result is not half a million."
  - The often-quoted 1161:1 odds belong to a 1.5-million-births scenario in the *Théorie analytique*, not to France.
  - The USGS handbook's 28,328,612 comes from rounding the ratio.
  - Mapping to capture–recapture: registered births are the "tags" (M), the sampled communes' population is the "second catch" (n), and births inside those communes are the recaptures (k).
- **Petersen (1896),** "The yearly immigration of young plaice into the Limfjord from the German Sea", *Rep. Danish Biol. Station* 6.
  - He used tag-return rates to measure *fishing intensity*. He did **not** compute N.
  - Goudie & Goudie (2007, *JRSS A* 170:825–839) examine "the claim that the well-known Petersen estimator … was not in fact used by the scientist after whom it is named."
  - Knut Dahl's trout work (about 1917, per Le Cren 1965) is an early true use. A nice data point: Rodli tarn, 109 marked, 177 caught, 57 recaptured → N̂ ≈ 338.
- **Lincoln (1930),** USDA Circular 118. His 1920–26 bandings returned 11.94% (2,083 of 17,449 ducks). His illustration is hypothetical: about 5,000,000 ducks killed with a 12% return rate implies about **42,000,000** waterfowl (5,000 × 5,000,000/600 = 41.7 M).
- **Schnabel (1938),** "The Estimation of Total Fish Population of a Lake", *American Mathematical Monthly* 45(6):348–352. It extends the method to many censuses: N̂ = Σ(C·M_t)/ΣR. It is a nice touch for the mathematician that it appeared in the *Monthly*.
- **Chapman (1951)** and **Bailey (1951)**: confirmed, and cited by Feller's footnote.

---

## 4. Modern applications [V]

**Recommended for the video:**

1. **Kosovo, 1999 (HRDAG; Ball et al. 2002, report to the ICTY).**
   - About 10,000 named reports collapsed to **4,400** unique victims.
   - Multiple-systems estimation with four lists gave **10,356** (95% CI **9,002–12,122**).
   - Years later an exhaustive database counted **9,790** for the same period (Spagat 2014), inside the original interval.
   - Be clear on screen that this uses log-linear models on four lists, not a two-catch formula.
2. **Software bugs.**
   - Two independent reviewers: N̂ = n₁n₂/overlap. A hypothetical example: 30 and 24 bugs found, 12 in common → 60 total, so about 18 found by neither reviewer.
   - Mills (1972, IBM FSC-72-6015) seeded deliberate bugs. Eick et al. (ICSE 1992) applied the method to inspections.
   - Briand et al. (2000) found these estimators **tend to underestimate**. That is the heterogeneity bias from §2.7 in a form developers will recognise: easy bugs get found by everyone.
3. **Laplace's France.** The history hook, and the "tags" don't have to be physical.
4. **German tank problem, as a contrasting cousin.**
   - Estimator N̂ = m + m/k − 1 (largest serial seen m, k tanks observed).
   - June 1940: statistical estimate **169**, intelligence estimate **1,000**, German records **122**. August 1942: 327 vs 1,550 vs 342 (Ruggles & Brodie 1947, via secondary sources).
   - It estimates from *order* (serial numbers), not *overlap*.

**Alternates:**
- **Peru truth commission (2003):** 69,280 deaths (CI 61,007–77,551 or 77,552 depending on source). Shining Path accounted for about 46% in the estimate.
- **Grizzly bears (Kendall et al. 2009):** 563 unique bears identified, estimate 765 (715–831).
- **UK modern slavery (Silverman 2014):** 10,000–13,000 victims against 2,744 known; the report itself explains the method with a fish example (100, 100, 20 → 500).
- **Size of the web (Lawrence & Giles 1998):** at least 320 million pages, a *lower bound* because search engines do not sample independently.

**Avoid presenting as the same problem:** Efron & Thisted's Shakespeare vocabulary estimate (31,534 distinct words, at least 35,000 more unknown). That is unseen-species estimation, a different problem.

---

## 5. Explainer design [P]

**Notation.** Use N (lake), M (marked), n (net), k (marked in the net). Show Feller's page once, mapping n₁→M, r→n, n→N.

**Jargon to define:**
- "n choose k" = the number of different netfuls.
- "Likelihood" = how well a hypothesis about N predicts what we actually saw.
- "Estimate" (a number) vs "estimator" (a recipe).
- "Confidence interval" = a procedure that brackets the true N in 95% of repeated studies.
- Name "hypergeometric" once, then call it "sampling without putting fish back".

### 5.1 Narrative arc (about 12.5 min, 10 chapters)

**1. "You can't count what you can't see" (60 s)**
- Core idea: draining the lake is absurd; catching every fish fails because you never know when you have the last one.
- Present Feller's data and invite a guess. Most people say 10,000. Promise: *why* that is right, *how* right, and *when* it fails.
- Picture: a murky lake with fish fading in and out of view.

**2. The fingerprint (75 s)**
- Core idea: the lake's tagged fraction, M/N, is a fingerprint. A well-mixed random catch is a miniature lake that carries the same fingerprint, so k/n ≈ M/N.
- Picture: a lake color-bar (10% glowing) beside a catch color-bar that jitters around it over repeated catches.
- Analogy: tasting one spoonful of a stirred soup.
- Misconception: "you must catch most of the lake." In fact precision depends mainly on k (CV ≈ √(1/k − 1/n)).
- Persona needs:
  - 14-year-old: the proportion argument.
  - Developer: the 1/√k scaling.
  - Mathematician: a promise that this will be justified, not assumed.

**3. What "random" buys you (85 s)**
- Toy lake N = 20, M = 5, n = 4: all 4,845 netfuls, sorted into counts 1365 / 2275 / 1050 / 150 / 5.
- Name the hypergeometric distribution. The sum check is Vandermonde's identity (Feller 6.4).
- Point out that randomness comes from the *fish mixing*, not from the net.

**4. Turning the question around (85 s)**
- Fix the data (M = 10, n = 10, k = 3) and ask, for each possible N, how probable *this exact* outcome is.
- Picture: slide N from 17 to 60 while the pmf for that N morphs below. The bar at k = 3 traces L(N) above. The peak is at 33 (0.3178).
- Then the 3D surface over (N, k): a slice at fixed N is a probability distribution summing to 1; a slice at fixed k is the likelihood, with area 16.67, not 1.

**5. The seesaw (85 s)**
- Feller's ratio, the cancellation down to Mn − kN, and the reading "make the lake bigger while its tag fraction exceeds the catch's".
- Show 529/528 tipping to 576/578.
- Then Feller's scale: 80,982,001/80,981,901 → exactly 1 → 81,018,001/81,018,101. The flat top: 9,999 and 10,000 tie.
- Payoff: the rigorous argument recovers the fingerprint intuition.

**6. How sure? (85 s)**
- Feller's 1,900-fish argument (10⁻⁴³⁰) and the million-fish argument.
- Even at the best N, the observed count had only a 4.4% chance.
- Relative likelihood: 0.17 at 8,500 and 0.15 at 12,000.
- Feller's bet versus the exact interval [8,400, 12,073]. The interval is asymmetric.
- Optional 15-second Bayesian aside (see 5.4).

**7. Ten thousand lakes (85 s)**
- Monte Carlo rain on a lake whose size we know.
- Picture: the symmetric bell of k on one axis, mapped through the hyperbola N = Mn/k into a right-skewed histogram of estimates (8,475 to 12,048; mean 10,080).
- Small study: 625 to 2,000. The k = 0 → ∞ cliff. Chapman's +1 fix, exactly unbiased when M + n ≥ N.

**8. When the fish lie (80 s)**
- Run through §2.7 with an estimate gauge swinging for each case.
- Lead with heterogeneity, the bias that matters most in practice.
- Include Feller's own "same locality, short time" caveat.

**9. Same idea everywhere (80 s)**
- Laplace's births as tags (28,352,845).
- Kosovo: 4,400 named → 10,356 estimated → 9,790 later counted.
- Two code reviewers.
- The German tank problem as a contrast.

**10. Coda (40 s)**
- Fingerprint → likelihood → honest uncertainty.
- Reasoning backward from what you saw to what must be true.

### 5.2 Set pieces that show off the engine

1. **Living lake.** Hundreds of fish advected along a curl-noise (divergence-free) flow field. Released tagged fish glow and visibly diffuse outward from the release point, which is mixing made visible.
2. **Net sweep.** Caught fish lift into a sorted tray that becomes a proportion bar.
3. **All 4,845 netfuls.** Thumbnails in a grid that fly into histogram bins.
4. **Likelihood tracer.** A slider on N, the pmf morphing below, the curve traced above.
5. **3D P(k | N) surface.** Ridge along k = Mn/N; animated slicing in both directions.
6. **Ratio seesaw or needle.** The algebra terms fly off as they cancel.
7. **Monte Carlo rain.** A 100 × 100 grid of tiny lakes feeding a histogram, with the exact pmf outline overlaid.
8. **Assumption montage.** Trap-happy fish, fading tags, newborn fry, clumped tags.

### 5.3 Numbers the video might show (all verified above)

| Quantity | Value |
|---|---|
| Lake size from Feller's data | 10,000 (tie with 9,999) |
| Minimum possible lake | 1,900 |
| P(lake = 1,900 data) | 10⁻⁴²⁹·²⁷ |
| L_max | 0.04429 |
| Exact Feller tails | 0.0347 / 0.0289 |
| Exact 95% interval | 8,400–12,073 |
| SD of k at N = 10,000 | 9.0 |
| SE of the estimate | ≈ 949 (9.5%) |
| Chapman estimate | 9,919.8 |
| Simulated 95% range | 8,475–12,048 |
| Simulated mean | 10,080 (+0.8%) |
| Area under the likelihood | 101.01 |
| Toy pmf counts | 1365/2275/1050/150/5 of 4,845 |
| Toy MLE | 33 (L = 0.3178) |
| Toy ratios | 529/528, 576/578 |
| Toy exact interval | 19–138 |
| Laplace | 2,037,615 people; 71,866⅓ births/yr; 28,352,845 |
| Kosovo | 4,400 / 10,356 / 9,002–12,122 / 9,790 |
| Tank, June 1940 | 169 vs 1,000 vs 122 |

### 5.4 Pitfalls a statistician would flag, with correct phrasing

- **Avoid** "the probability that N = 10,000." **Say** "the value of N that makes our observation most probable." Feller himself insists N is fixed.
- **Avoid** "the MLE is unbiased." **Say** "Mn/k has no finite mean, because k = 0 is possible; given k > 0 it overshoots slightly (+0.8% here). Chapman's version fixes this."
- **Avoid** "the estimate is Mn/k." **Say** "the MLE is the integer ⌊Mn/k⌋, or a tie when Mn/k is a whole number."
- **Avoid** "95% chance N lies in [8,400, 12,073]." **Say** "this procedure captures the true N in at least 95% of repeated studies." A Bayesian credible interval *does* license the probability statement, but only given a prior.
- **Do not** use ± symmetric intervals, and do not use the binomial in place of the hypergeometric without saying so (SD 9.49 vs 9.00).
- **Always state** equal catchability and mixing as assumptions. Never present multiple-systems estimates as simple two-list Lincoln–Petersen.
- **Bayesian view: include it as a one-sentence aside, not a chapter.** With a uniform prior the likelihood normalizes to a posterior (proper when k ≥ 2), giving 8,536–12,186; a 1/N prior gives 8,475–12,073. Mentioning it shows respect for the mathematician; dwelling on it would compete with the core story.

---

## 6. Sources

- **Feller, 1st ed. (1950)**, open full text: https://archive.org/details/dli.ernet.5666
- **Feller, 3rd ed. (1968)**, borrowable: https://archive.org/details/introductiontopr0001fell. I read pp. 43–46, 56, 59, 64, 172, 194, 232–233 from a third-party scan: https://bitcoinwords.github.io/assets/papers/an-introduction-to-probability-theory-and-its-applications.pdf
- **Laplace:**
  - *Essai* (Gutenberg #58881): https://www.gutenberg.org/ebooks/58881
  - 1783 memoir, translated by Pulskamp: https://gtfp.cs.rhul.ac.uk/pulskamp/Laplace/naissances.pdf
- **Amstrup et al. 2005**, ch. 1: https://pubs.usgs.gov/publication/70186644
- **Graunt**, ch. XI: https://en.wikisource.org/wiki/Natural_and_Political_Observations_Made_upon_the_Bills_of_Mortality/Chapter_11
- **Lincoln 1930:** https://archive.org/details/calculatingwater118linc
- **Goudie & Goudie 2007:** doi:10.1111/j.1467-985X.2007.00479.x
- **Schnabel 1938:** doi:10.2307/2304025
- **Mark and recapture** (Wikipedia; Chapman formula and Sadinle interval): https://en.wikipedia.org/wiki/Mark_and_recapture
- **Robson & Regier 1964:** https://scinapse.io/papers/2163942609
- **Kosovo:**
  - ICTY report: https://www.icty.org/x/file/About/OTP/War_Demographics/en/s_milosevic_kosovo_020103.pdf
  - Spagat 2014: https://www.hlc-rdc.org/wp-content/uploads/2015/03/Michael-_Spagat_Evaluation_of_the_Database_KMB_December_10_2014.pdf
- **Peru:**
  - https://arxiv.org/abs/1906.04763
  - https://cverdad.org.pe/ingles/ifinal/conclusiones.php
- **Software (Briand et al.):** https://www.ehealthinformation.ca/web/default/files/wp-files/2001-A-Comprehensive-Evaluation.pdf
- **Modern slavery:** https://assets.publishing.service.gov.uk/government/uploads/system/uploads/attachment_data/file/386841/Modern_Slavery_an_application_of_MSE_revised.pdf
- **Lawrence & Giles:** https://clgiles.ist.psu.edu/pubs/Science-98.pdf
- **German tank problem:** https://en.wikipedia.org/wiki/German_tank_problem
- **Grizzlies:** https://pubs.usgs.gov/publication/70032746

---

## Appendix: verification code

This is the consolidated script; it runs in about 2 minutes and its outputs match everything above. The full scripts are in `docs/research/capture_recapture/scripts/`.

```python
from math import comb, lgamma, log, exp, sqrt
from fractions import Fraction as F
import numpy as np

def L(N, M, n, k):                       # exact P(K=k | N,M,n)
    if n - k > N - M or k > M or N < n: return F(0)
    return F(comb(M, k) * comb(N - M, n - k), comb(N, n))
def logL(N, M, n, k):
    lc = lambda a, b: lgamma(a+1) - lgamma(b+1) - lgamma(a-b+1)
    return lc(M, k) + lc(N-M, n-k) - lc(N, n) if n - k <= N - M else float('-inf')

assert all(L(N,M,n,k)/L(N-1,M,n,k) == F((N-M)*(N-n), N*(N-M-n+k))
           for (M,n,k) in [(5,4,1),(10,10,3),(1000,1000,100)]
           for N in range(M+n-k+1, M+n-k+200))
print([comb(5,j)*comb(15,4-j) for j in range(5)], comb(20,4))     # [1365,2275,1050,150,5] 4845
M, n, k = 1000, 1000, 100
print(L(9999,M,n,k) == L(10000,M,n,k), float(L(10000,M,n,k)))     # True 0.04428917
print((2*lgamma(1001) - lgamma(101) - lgamma(1901)) / log(10))     # -429.27
le = lambda N, kk: sum(L(N,M,n,j) for j in range(kk+1))
print(float(le(8500,100)), float(1 - le(12000,99)))                 # 0.03468 0.02887
lmax = logL(10000,M,n,k); rel = lambda N: exp(logL(N,M,n,k) - lmax)
lr = [N for N in range(1900, 30000) if rel(N) >= exp(-3.841458820694124/2)]
print("LR 95%:", lr[0], lr[-1])                                      # 8445 12021
P_le = lambda N: sum(exp(logL(N,M,n,j)) for j in range(k+1))
P_ge = lambda N: sum(exp(logL(N,M,n,j)) for j in range(k, n+1))
lo = next(N for N in range(1900, 20000) if P_le(N) > 0.025)
hi = next(N for N in range(20000, 1900, -1) if P_ge(N) > 0.025)
print("exact 95%:", lo, hi)                                          # 8400 12073
print((M+1)*(n+1)/(k+1) - 1, sqrt(M*M*n*(n-k)/k**3),
      sqrt((M+1)*(n+1)*(M-k)*(n-k)/((k+1)**2*(k+2))))               # 9919.80 948.68 883.19
for (N_, M_, n_) in [(20,5,4),(20,10,10),(1000,50,50)]:             # Chapman exact bias identity
    E = sum(L(N_,M_,n_,j) * (F((M_+1)*(n_+1), j+1) - 1) for j in range(min(M_,n_)+1))
    assert E - N_ == -F((N_+1)*comb(N_-M_, n_+1), comb(N_+1, n_+1))
def lp_summary(N_, M_, n_):                                         # exact law of Mn/k
    rows = [(j, exp(logL(N_,M_,n_,j))) for j in range(min(M_,n_)+1)]
    Z = 1 - rows[0][1]; c = 0; q = {}
    for j, p in sorted(rows[1:], key=lambda t: -t[0]):
        c += p / Z
        for a in (.025, .5, .975):
            if a not in q and c >= a: q[a] = M_*n_/j
    return rows[0][1], sum(p*M_*n_/j for j, p in rows[1:]) / Z, q
print(lp_summary(10000,1000,1000), lp_summary(1000,100,100))
Ns = np.arange(1900, 400001); w = np.exp([logL(int(x),M,n,k) for x in Ns])
print(w.sum(), M*n/(k*(k-1)))                                       # 101.0101 both
for lab, ww in (("flat", w), ("1/N", w/Ns)):
    c = np.cumsum(ww/ww.sum()); qq = lambda a: int(Ns[np.searchsorted(c, a)])
    print(lab, qq(.025), qq(.5), qq(.975))   # flat 8536 10121 12186 ; 1/N 8475 10039 12073
rng = np.random.default_rng(20261009)
est = M*n / rng.hypergeometric(M, 10000-M, n, size=10000)
print(est.mean(), np.median(est), np.percentile(est, [2.5, 97.5]))  # ~10080 10000 [8475 12048]
print(2037615 / (215599/3) * 1_000_000)                             # Laplace: 28,352,844.86
```
