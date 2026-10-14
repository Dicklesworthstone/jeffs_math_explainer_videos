# Video A fact sheet: How to Count Fish You've Never Seen (capture_recapture)

Bead `mx-e-a-8y5.2` (P1); the claim walk's keys were added for gate G3 on 2026-10-11, and the
R2 review round's keys the same day (see "R2 review round" below). Every value below was printed
by the independent oracle `explainer_corpus/capture_recapture/oracle.py`; the full precision value
is on the named key.

- **Run** from `explainer_corpus/capture_recapture/`, as `./x capture_recapture facts-check`
  does: `uv run --quiet --no-project --with numpy python3 -I -B oracle.py` (standard library
  plus numpy; last run with Python 3.14.0 and numpy 2.5.4 under uv).
- **Measured (R2, 2026-10-11, Apple M4 Pro):** **547 keys**, one `KEY<TAB>VALUE` per line on
  stdout (runtime and key count go to stderr); wall clock 49.2 s and 50.6 s on the last two runs,
  whose stdout is byte-identical. Against the claim walk's 451 keys the R2 run adds 96 keys and
  changes or drops none (checked key by key). Every ledger row A-F1 … A-F32 has at least one key;
  10 keys are `.recorded.`; 105 are `A-CH03.*`; 65 are the claim walk's `A-CH11.*`, `A-CH12.*`
  and `A-CH14.*` (see "Claim walk (gate G3)" below).
- **Seeds:** numpy `default_rng(20261009)` replays `verify2.py`'s exact call order, so the
  simulated numbers reproduce `docs/research/capture_recapture/scripts/outputs/verify2_out.txt`
  digit for digit. The A-F27 lake uses SplitMix64 seeded 20261009; chapter 03 uses
  `default_rng([20261009, 3])`; the two-reviewer runs (A-CH14) use `default_rng([20261009, 14])`.
- **Notation:** N lake size, M marked, n second catch, k caught twice; cells a = M − k,
  b = n − k, u = N − M − n + k.

**Status legend.**
- **MATCH:** the oracle reproduces the plan's value at the plan's rounding.
- **MISMATCH:** the values differ, and the reason is given.
- **NOTE:** the values match, but the script or storyboard needs a caveat.
- **RECORDED:** an external figure copied from its source, not computed. Keys containing
  `.recorded.` are recorded figures, and the source is quoted under "Non-numeric claims".
- **NEW:** the plan had a placeholder, and this sheet sets the value.

## What needs attention (uncomfortable findings first)

1. **A-F27, the hook's dip estimates (NEW; the draft example is a MISMATCH).**
   - **The dips.** The documented lake gives **88,000 / 2,800 / 45,100** (weed bed / open
     water / deep end), a **31.4×** spread.
   - **The draft example does not occur.** The draft's "four hundred fish, times a hundred,
     is forty thousand" happens at none of the three fixed spots. The nearest is the deep end:
     451 fish, so 45,100.
   - **The ≥ 30× holds for this draw, not in expectation.** At the same spots the model's
     *expected* spread is 28.5×; the weed cell drew 880 fish against 833 expected, and the
     open-water cell 28 against 29.2.
   - **I did not change the lake.** By hand estimate before the first run, the expected
     spread was about 29×; I left the parameters alone.
   - **Use this one number everywhere:** "about thirty times" (31.4×).
2. **Chapter 03's dynamics: the claim holds in shape, not exactly.**
   - **What holds.** The marked fraction flattens (0–66% across the 3 × 3 regions at t = 0;
     9.3–10.7% at t = 400). The density stays clumpy: the region max/min is 12.8 at t = 0 and
     12.8 at t = 400, against 11.4 in the model.
   - **What doesn't.** With the contour circulation and reflecting shores, the simulated
     density is **not** e^(−U): time-averaged regions sit up to **14%** off the model.
   - **The cause.** A diagnostic run without circulation stays within **2.2%**, so the cause
     is the circulation crossing the shore near the deep hole, not the time step.
   - **Consequence.** The plan's "stationary density ∝ e^(−U)" needs the shore handled (see
     the chapter 03 section).
3. **Fisher attribution (MISMATCH in wording).**
   - Stigler (2007) does **not** credit Lambert with a version of maximum likelihood. He
     credits Lagrange, Daniel Bernoulli, Laplace and Gauss, and says Lambert "got what was
     arguably the wrong answer".
   - The "1912–1922" framing is Aldrich (1997)'s title.
   - Stigler shows the name "method of maximum likelihood" already in Fisher's Nature
     abstract of November 1921.
4. **Mills's year.**
   - The IBM report FSC-72-6015 is dated 1972 (NBS SP 500-75 reference list).
   - Mills's own collected papers date the article **1970**.
   - So the memo's "correction" ("1972, not 1970") is only half right. Say "Mills (IBM
     report, 1972)", or give both dates.
5. **Laplace's wording.** What was "spread equally over France" is the **thirty
   départements**. Within them, communes were *chosen because they could furnish the most
   precise information*. Also, the count was made by the government, at Laplace's request.
   So "he counted everyone in selected towns" (10_2) needs "officials" and "communes".
6. **Briand et al. (2000) and Goudie & Goudie (2007) are only partly supported by their
   abstracts.**
   - Briand: underestimation "may be substantial" *when the number of inspectors is too
     small*. That is not a blanket "tends to underestimate".
   - Goudie: the abstract says they "examine the claim" that Petersen did not use the
     estimator. I did not read the paper's conclusion.
7. **Feller, 3rd edition (1968): UNVERIFIED by me.** This covers footnote 13's wording, "an
   unknown fixed number", Example (c), pp. 45–46 and eq. (6.3). No legitimate copy was
   readable: archive.org lends it only. Everything about the 1st edition was verified.
8. **A-F20, NOTE.** The ledger's "mean 1,103" is the simulation's mean (1,103.3). The
   *exact* mean is **1,102.0**, yet the row is labelled "sim + exact".
9. **A-F3, NOTE.** The five on-screen percents (28.2 / 47.0 / 21.7 / 3.1 / 0.1) sum to
   **100.1%**.
10. **07_7 lower-third, NOTE (not a ledger row).** The "−1.1% at E[k] ≈ 4" figure depends on
    the setup.
    - It is −1.10% for N = 1,000, M = n = 64 (E[k] = 4.096).
    - It is −1.62% for N = 10,000, M = n = 200 (E[k] = 4 exactly).
    - The memo's "−1.1% to −1.6%" is the honest range.
11. **Dangling ledger reference.** The plan cites **A-F35** (02 exact version; the
    non-numeric list), but the ledger ends at A-F32.
12. **The G1 facts-check cannot compare the numpy-seeded keys at 1e-9.**
    - **Which keys:** `A-F19.sim_*`, `A-F20.sim_*`, `A-F22.*` and `A-CH03.*`.
    - **Why:** a Rust `facts` binary would have to port numpy's PCG64, SeedSequence and its
      hypergeometric, choice and normal samplers to reproduce them.
    - **What to do:** prefer the `exact_*` keys on screen, and pin the sim values as
      labelled-model numbers or compare them at a Monte Carlo tolerance.
    - **Exception:** A-F27 is portable by design (SplitMix64 plus rejection; see below).
    - **Also numpy-bound (claim walk):** `A-CH12.ring_toss.*` (A-F19's stream), `A-CH14.*`, and
      the 03_6 insets and 03_5 trays (`A-CH03.inset.*`, `A-CH03.trays.*`), which are read from the
      numpy chapter-03 run's t = 400 frame. A Rust chapter-03 scene with its own generator will
      have different fish there; it must load these positions or re-pick its insets by the same
      rule and pin its own counts.
    - **Also numpy-bound (R2):** the 09_6 panel's runs (`A-CH09.panel.*`, and the ring toss on
      them, `A-CH12.ring_toss_panel.*`), which depend on numpy's `Generator.choice` stream;
      `A-CH03.corners.*` and `A-CH03.trays.net1000.*` (the same t = 400 frame); and the range
      checks `A-CH14.reviewers.*.ci95_*` (the A-CH14 runs). The chapter-09 and chapter-12 scenes
      must load the panel's 10,000 caught-twice counts, not draw their own.
13. **Claim walk (G3) findings.** Details are in "Claim walk (gate G3)" below.
    - **03_5 trays:** real nets of 10 or 20 fish do not sit near 10%. At the nine region centres
      they hold 0–3 of 10 and 0–5 of 20 marked. The storyboard now tags the trays "illustration".
    - **03_6 insets:** these are real squares, but they were chosen to match the spoken counts.
    - **08_5 "floating point picks 10,000":** this holds for an lgamma log-likelihood, but a
      float ratio scan ties exactly. The lower-third now names lgamma.
    - **11_2 marker:** 5.35 × 10⁻ᶻ is still a nonzero float64 at Z = 324 and rounds to 0 from
      Z = 325. The label now reads "rounds to zero past here".
14. **R2 review round (2026-10-11).** Details are in "R2 review round" below.
    - **The 13_1 rule was wrong** (Ruth R1, the R2 blocker): the share rule makes Mn/k aim at the
      lake *on marking day*; deaths leave the needle at 10,000 while about 7,000 fish are left.
      `A-CH13.*` pins both targets.
    - **03_5 and 03_6 no longer use selected or drawn numbers.** The R1 insets (`A-CH03.inset.*`)
      were picked to match the voice and the trays were drawn. Both are replaced by numbers read
      from the t = 400 frame by rules that never look at the marks (`A-CH03.corners.*`,
      `A-CH03.trays.net1000.*`). The old keys stay as the record.
    - **The 09_6 panel nets fish.** The R1 panel sampled `rng.hypergeometric`, so its check could
      only test numpy's sampler (Dev dev-01). The new panel's runs (`A-CH09.panel.*`) are what
      chapter 09 and the 12_2 ring toss now show; A-F19's replay stream keeps its keys but is no
      longer on screen.

## Facts ledger A-F1 … A-F32

| id | quantity | oracle value | plan value | match | how verified |
|---|---|---|---|---|---|
| A-F1 | C(20,4) | 4,845; 20·19·18·17 = 116,280; ÷ 24 = 4,845 | 4,845 = 20·19·18·17/24 | MATCH | `af1_choose_20_4` (math.comb); memo §2.1; `a_fish.py` |
| A-F2 | netfuls by k = 0…4 (N = 20, M = 5, n = 4) | 1,365 / 2,275 / 1,050 / 150 / 5; sum 4,845 (Vandermonde) | 1,365 / 2,275 / 1,050 / 150 / 5 | MATCH | `af2_netful_counts`, `num()`; `verify.py` §8 |
| A-F3 | their probabilities | 91/323 = .2817, 455/969 = .4696, 70/323 = .2167, 10/323 = .0310, 1/969 = .0010; screen 28.2 / 47.0 / 21.7 / 3.1 / 0.1% | same | MATCH; NOTE the screen percents sum to 100.1% | `af3_netful_probabilities` (exact Fractions) |
| A-F4 | k = 2 decomposition | C(5,2)·C(15,2) = 10·105 = 1,050 (= A-F2.k2) | 10·105 = 1,050 | MATCH | `af4_k2_decomposition` |
| A-F5 | toy best size (M = n = 10, k = 3) | argmax {33} over N = 17…2,000 (exact); L(33) = 7429/23374 = 0.31783; soup 100/3 = 33.33 | 33; 0.3178; 33.3 | MATCH | `af5_toy_best_size`; `verify2.py` §D |
| A-F6 | toy comparisons | N = 33: 100 − 99 = +1, ratio 529/528; N = 34: 100 − 102 = −2, ratio 576/578 (each ratio checked against L(N)/L(N−1) exactly) | same | MATCH | `af6_toy_comparisons`; `ratio_numbers.py` |
| A-F7 | likelihood area | closed form 50/3 = 16.667 (toy), 10000/99 = 101.0101 (Feller). Partial sums 16.6666656 (N ≤ 200,000) and 101.0101010098 (N ≤ 400,000). k = 1 toy: 413.8 / 643.3 / 873.5 at N = 10⁴ / 10⁵ / 10⁶, growing 230.19 per decade (≈ Mn·ln 10 = 230.26), so it diverges | 16.667; 101.01; infinite for k ≤ 1 | MATCH | `af7_likelihood_area`; proof: `docs/reviews/plan_r1/ruth_45_audit.md` P1 (Beta integral; not re-derived here); `area.py`, `b_area.py` |
| A-F8 | toy exact 95% CI; Chapman | [19, 138] (bisection on exact integers; an independent linear scan agrees); Chapman 117/4 = 29.25 | [19, 138]; 29.25 | MATCH | `af8_toy_interval`, `exact_ci`; `verify2.py` §D, `g_ci.py` |
| A-F9 | Feller best size | {9,999, 10,000}: sign(Mn − kN) is + for every N < 10,000, 0 at 10,000, − after (N ≤ 40,000); L(9,999) = L(10,000) exactly; k = 100 < min(M,n); L = 0.0442892 | {9,999, 10,000} tie; 0.04429 | MATCH | `af9_feller_best_size`; `verify.py` §2 |
| A-F10 | Feller comparisons | Mn − kN = +100 / 0 / −100; ratios 80,982,001/80,981,901; 81,000,000/81,000,000; 81,018,001/81,018,101 | +100 / 0 / −100 | MATCH | `af10_feller_comparisons`; `ratio_numbers.py` |
| A-F11 | minimum lake | 1,900; P = (1000!)²/(100!·1900!) (the exact Fraction equals L(1900)) = 5.3515 × 10⁻⁴³⁰; lgamma log₁₀ −429.2715 | 1,900; ≈ 5.4 × 10⁻⁴³⁰ | MATCH | `af11_minimum_lake` (exact + lgamma); `verify.py` §2 |
| A-F12 | relative likelihoods | 8,500: 0.170675; 12,000: 0.151834; 1,000,000: 3.8376 × 10⁻¹⁶² | 0.171; 0.152; 3.8 × 10⁻¹⁶² | MATCH | `af12_relative_likelihoods` (lgamma); `verify.py` §3 |
| A-F13 | exact 95% CI (test inversion) | [8,400, 12,073]. Exact boundaries: P(K ≤ 100) = 0.0249417 at 8,399 and 0.0250263 at 8,400; P(K ≥ 100) = 0.0250310 at 12,073 and 0.0249817 at 12,074. A float linear scan agrees | [8,400, 12,073] | MATCH | `af13_feller_interval`, `exact_ci`; `verify.py` §4 |
| A-F14 | Feller's 8,500–12,000 tails | 0.0346771 / 0.0288747; coverage 0.936448 | 0.0347 / 0.0289 (≈ 93.6%) | MATCH | `af14_feller_tails` (exact); `normal_approx_check.py` |
| A-F15 | SD of k at N = 10,000 | Var = 90000/1111 = 81.0081, SD 9.00045; binomial 9.48683 | 9.000 (binomial 9.487) | MATCH | `af15_sd_of_k` |
| A-F16 | spread of the estimate | exact SD of Mn/K given K > 0: 924.36 (mean 10,082.50); CV 9.24% of N, 9.17% of the mean; symmetric approximation 9.00% | 924 (CV 9.2%); 9.0% | MATCH (the 9.2% holds for either denominator) | `af16_spread_of_estimate`, `plain_estimate_law` (exact to the final square root); `integration_check.py` §3 |
| A-F17 | Chapman (Feller's data) | 1001900/101 = 9,919.80; SE 883.19 | 9,919.8; 883 | MATCH | `af17_chapman_feller`; `verify.py` §5 |
| A-F18 | Chapman bias identity | holds exactly on 1,503 cases (every 1 ≤ M, n ≤ N ≤ 16, plus 7 named cases up to N = 1,000); bias is 0 exactly iff M + n ≥ N; Feller-scale bias −3.56 × 10⁻⁴⁵ | identity proved + exact fractions | MATCH | `af18_chapman_identity`, `chapman_mean_exact`, `chapman_bias_closed`; `verify.py` §6; proof idea memo §2.4 / Ruth P2 |
| A-F19 | simulated quantiles (Feller's lake) | exact 8,474.58 / 10,000 / 12,048.19 (k = 118 / 100 / 83); sim (10,000 runs) p2.5 / median / p97.5 identical; sim mean 10,080.002; exact mean 10,082.499 | 8,475 / 10,000 / 12,048 (k = 118 / 100 / 83); 10,080.0 (exact 10,082.5) | MATCH | `af19_feller_simulation`, `verify2_replay`, `plain_estimate_law`; `verify2.py` §E, `appendix.py` |
| A-F20 | small study N = 1,000, M = n = 100 | exact 625 / 1,000 / 2,000; **exact mean 1,102.04**; sim mean 1,103.34 | 625 / 1,000 / 2,000; mean 1,103 ("sim + exact") | MATCH for the quantiles; NOTE 1,103 is the sim mean, the exact mean is 1,102.0 | `af20_small_study_100`; `verify2.py` §E |
| A-F21 | small study N = 1,000, M = n = 50 | P(k = 0) = 0.0719770 (1 in 13.89); Chapman exact mean 935.22 (−6.48%); plain formula given k > 0: 1,241.24 (+24.12%) | 7.20% (1 in 13.9); 935.2 (−6.5%); 1,241 (+24%) | MATCH | `af21_small_study_50`; `integration_check.py` §2 |
| A-F22 | failure-mode medians (400 runs each) | bold 7,751.94; trap-happy 7,142.86; trap-shy 14,492.75; marks lost 12,500; +2,000 newcomers 12,121.66; 30% deaths 10,000; release site 6,250; far side 25,000 (ideal 10,000) | 7,752; 7,143; 14,493; 12,500; 12,122; 10,000; 6,250; 25,000 | MATCH | `af22_failure_modes`, `verify2_replay` (same numpy stream; matches `verify2_out.txt` §F to every digit) |
| A-F23 | Laplace 1802 | RECORDED 2,037,615 people and 110,312 + 105,287 births (1800–1802). Computed: 215,599 births; 215599/3 = 71,866⅓ a year; ratio 6112845/215599 = 28.3528449, i.e. 28.352845 to 6 dp. × 1,000,000 (RECORDED assumption) = 28,352,844.86, i.e. 28,352,845. Odds RECORDED: "almost 300000 against 1" | same | MATCH (inputs and odds RECORDED from the *Essai*, quoted below) | `af23_laplace`; `verify2.py` §G |
| A-F24 | two reviewers | 30, 24, 12 → 42 seen; estimate 60; 18 hiding; exact 95% [47, 94] → 5–52 hiding; Chapman 762/13 = 58.62 ± 8.41 | same | MATCH | `af24_two_reviewers`, `exact_ci`; `integration_check.py` §4 |
| A-F25 | Kosovo | RECORDED: 4,400 unique named victims (an exact count, not an estimate); estimate 10,356, 95% 9,002–12,122; later count 9,790 (Kosovo Memory Book, 20 Mar – 20 Jun 1999, violent deaths). Computed: 9,790 is inside the interval; 10,356/4,400 = 2.35 | ≈ 4,400; 10,356 (9,002–12,122); 9,790 | MATCH (RECORDED; both sources quoted below) | `af25_kosovo`; Ball et al. 2002 p. 6; Spagat 2014 p. 9 |
| A-F26 | Bayesian aside (flat prior) | 8,536 – 12,186 (median 10,121), N ≤ 400,000; mean numeric 10,183.68; closed form (M−1)(n−1)/(k−2) = 998001/98 = 10,183.68 | 8,536–12,186; 10,183.7 | MATCH | `af26_bayesian`; `verify2.py` §B; `b_area.py` (first-moment identity) |
| A-F27 | dip estimates (chapter 01) | weeds 880 fish → **88,000**; open water 28 → **2,800**; deep end 451 → **45,100**; spread 220/7 = **31.4×**. Model expectation at the same spots: 83,327 / 2,919 / 45,736 (28.5×) | "computed in P1"; draft example "400 × 100 = 40,000"; three spots spanning ≥ 30× | NEW; MISMATCH with the draft example; ≥ 30× met by this draw, not in expectation | `af27_clustered_lake`, `draw_lake`, `LAKE`, `SPOTS`; design below |
| A-F28 | split symmetry | the pmf of K is identical term by term (exact, Feller 6.2); exact SD 894.44 both ways (variances equal exactly) | 894 both | MATCH | `af28_split_symmetry`; `integration_check.py` §3 |
| A-F29 | table identity | Mn − kN = ab − k·u, proved: the difference has degree ≤ 2 in each variable and vanishes on {0,…,3}⁴. Feller: û = 810000/100 = 8,100 (whole, hence the tie). Toy: û = ⌊49/3⌋ = 16, so N̂ = 7 + 7 + 3 + 16 = 33 | ab − k·u; 8,100; 16 | MATCH | `af29_table_identity`; `integration_check.py` §1 |
| A-F30 | coin analogy | C(100,50)/2¹⁰⁰ = 0.0795892 (7.96%); 50 is the mode | 7.96% | MATCH | `af30_coin` |
| A-F31 | bending | 10⁶/90 = 11,111.1, i.e. 11,111; 10⁶/110 = 9,090.9, i.e. 9,091 (+1,111 against −909) | 11,111; 9,091 | MATCH | `af31_bending` |
| A-F32 | float64 | 5.35 × 10⁻⁴³⁰ < 2.2250738585072014 × 10⁻³⁰⁸ (smallest normal), and also < 4.9 × 10⁻³²⁴ (smallest subnormal); `float()` of the exact value and `exp(lgamma …)` both give 0.0 | 5.4 × 10⁻⁴³⁰ < 2.2 × 10⁻³⁰⁸ | MATCH | `af32_float64` |

## A-F27: the clustered lake and its three dips

The lake model, the seed and the spots were fixed before anything was run (docstring above `LAKE`
in `oracle.py`). Nothing was re-run to reach a target.

- **Lake.** The lake is the unit square (x east, y north), so each cell of the 10 × 10 grid is
  exactly 1/100 of it. It holds **N = 10,000** fish, Feller's lake, which is the "real answer"
  behind 01's question mark.
- **Habitat.** s(x, y) = 1 + 40·exp(−|p − (0.25, 0.75)|²/(2·0.08²)) + 20·exp(−|p − (0.75,
  0.25)|²/(2·0.12²)).
  - The first bump is a weed bed in the north-west; the second is a deep hole in the
    south-east. Open water scores 1.
  - Fish density ∝ s, i.e. U = −log s, the same U as chapter 03.
  - The amplitudes 40 and 20 are round "tens of times" contrasts for the narration's "fish
    crowd into weeds and the deep end, and avoid open water". They were not tuned.
- **Draw.**
  - The fish are i.i.d. from density s, by rejection with **SplitMix64 (seed 20261009)**.
  - Each candidate consumes three uniforms (x, y, a), each (u64 >> 11)·2⁻⁵³; the candidate is
    accepted if a·61 < s(x, y).
  - The draw took 139,559 candidates. Check keys: `A-F27.check_sum_x` = 5065.9449974785075 and
    `A-F27.check_sum_y` = 4924.850531364975. The Rust scene can draw the identical lake.
- **Spots, named by habitat rather than by count.**
  - **Weeds:** the cell containing the weed-bed centre, cell (2,7).
  - **Open water:** the cell containing the lake centre (0.5, 0.5) under half-open cells, cell
    (5,5).
  - **Deep end:** the cell containing the deep-hole centre, cell (7,2).
  - The estimate at each spot is 100 × the fish in the cell.

| spot | cell (ix, iy) | fish in the net | estimate | model expectation |
|---|---|---|---|---|
| weeds | 2, 7 | 880 | 88,000 | 833.3 fish (83,327) |
| open water | 5, 5 | 28 | 2,800 | 29.2 fish (2,919) |
| deep end | 7, 2 | 451 | 45,100 | 457.4 fish (45,736) |

**Spread.** The three estimates span 88,000 / 2,800 = **31.4×**; the model's expectation at
these spots is 28.5×.

**Over all 100 cells.**
- The fewest fish in a cell is 12 and the most is 880.
- The median single-dip estimate is 3,700: most of the lake is open water.
- The mean of all 100 dips is exactly 10,000, by construction.
- Only 22% of the cells give an estimate within 2× of the truth.

This supports Ruth's note: many random dips averaged do work, but one dip cannot tell how much of
the lake it searched.

Cell counts (the heat map; north at the top, west at the left; row iy = 9 … 0, columns ix = 0 … 9):

```
iy9   36  50  63  41  31  23  31  24  32  21
iy8   65 237 402 219  47  23  12  21  21  26
iy7   79 433 880 402  49  21  21  21  19  28
iy6   61 225 410 227  41  30  25  27  21  26
iy5   25  42  85  47  35  28  34  38  33  27
iy4   23  22  25  29  32  52 103 137 107  58
iy3   26  15  15  24  44 110 259 340 263 110
iy2   26  29  14  28  34 149 351 451 361 123
iy1   23  23  20  27  51 100 242 335 250 114
iy0   21  20  22  27  49  48  96 139 105  63
```

**Suggested words (01_5–01_6).** "This net covers one hundredth of the lake. Four hundred fifty-one
fish, times a hundred, is forty-five thousand one hundred." Then the weed bed gives 88,000 and
open water 2,800: "about thirty times apart". The storyboard chooses the order; the numbers do not
change.

## Chapter 03: habitat-potential dynamics (keys `A-CH03.*`, not a ledger row)

**Model.**
- dX = −D∇U dt + c·J∇U dt + √(2D) dW, with U = −log s (the A-F27 habitat) and J the +90° rotation.
- Parameters: D = 0.01, c = 0.02; Euler–Maruyama with dt = 0.01; reflecting shores;
  `default_rng([20261009, 3])`.
- Start from the A-F27 lake, which is already a draw from e^(−U).
- The first catch is the **1,000 fish nearest the dock** at (0.5, 0) on the south shore (radius
  0.271). They are marked and released where they were caught.
- Regions are 3 × 3 thirds, listed SW, S, SE, W, C, E, NW, N, NE. Snapshots are at t = 0, 25,
  100 and 400; time averages are over t = 200–400.

| t | marked fraction across regions (min–max) | region count max/min, habitat run | region count max/min, contrast run (no −∇U drift) | habitat run: largest region deviation from the model |
|---|---|---|---|---|
| 0 | 0.000 – 0.659 (S region 65.9%; six regions 0%) | 12.78 | 12.78 | 9.3% (the lake draw's own sampling noise) |
| 25 | 0.046 – 0.155 | 12.54 | 1.78 | 13.6% |
| 100 | 0.087 – 0.116 | 13.91 | 1.78 | 20.6% |
| 400 | 0.093 – 0.108 | 12.77 | 1.72 | 16.3% |
| avg 200–400 | — | 12.62 | 1.74 | **14.1%** (diagnostic run with c = 0: **2.2%**) |

The model's max/min is 11.43. The binomial SD of a region's marked fraction near 10% is about
0.020 in the smallest region, so the t = 400 spread of 0.093–0.108 is flat within noise.

**Honest reading.**
- **Shown:** the marked fraction flattens to ≈ 10% everywhere ("starts hot at the dock,
  flattens to ≈ 10%"), while the fish stay crowded. The habitat run keeps its 12–14× contrast.
- **Shown:** divergence-free circulation plus noise *without* the −∇U drift flattens the density
  (12.8× → 1.7× by t = 25). That confirms the plan's warning about curl noise.
- **Not shown exactly:** "the stationary density ∝ e^(−U)". With the circulation on and
  reflecting shores, region densities drift up to 14% from e^(−U) on average. The southern
  regions gain (SW +6%, S +9%, SE +10%); the northern and western ones lose (NE −14%, NW −7%,
  N −5%, W −5%).
- **The diagnosis:**
  - Without the circulation, the density stays within 2.2% of the model at the same dt. With
    the circulation, dt = 0.0025 gives the same drift, measured in a scratch run outside the
    oracle.
  - So the cause is the circulation c·J∇U having a component normal to the shore wherever
    U's contours meet it (the deep hole sits only 2.1 sd from the south and east shores).
    Reflection then breaks the invariance that holds in the open plane.
  - The same effect keeps the contrast run from being exactly uniform (1.74×, not 1.0×).
- **Fix options for the scene:**
  - (a) keep habitat features ≥ ~4 sd from the shore, so U is flat along it;
  - (b) make the shoreline a contour of U;
  - (c) show the simulated density map, not e^(−U), and do not label it as e^(−U).
- The flattening of the marked fraction does not depend on this fix. Marked and unmarked fish
  follow the same dynamics, so marks become independent of position whatever the stationary
  law is. That is Ruth's statement in 03's persona notes.

## Claim walk (gate G3): keys for the storyboard's remaining claims

The claim walk (`docs/claims.md`) found numbers that the storyboard shows or says but that no key
covered. This table adds those keys, computed by the oracle, and checks them against the
storyboard as it stands after the walk's fixes. The rules and seeds are in the comments above
each function in `oracle.py`. The non-numeric claims are rows 13 and 14 of the next section.

| beat | storyboard claim | keys | oracle value | status / how computed |
|---|---|---|---|---|
| 03_5 | trays: "each tray's marked share sits near 10%" | `A-CH03.trays.*` | Nets of the 10 fish nearest each of the nine region centres (t = 400) hold 1, 0, 3, 0, 0, 2, 2, 0, 0 marked. Nets of 20 hold 2, 0, 5, 0, 2, 5, 2, 1, 0. A random net of 10 holds exactly 1 marked with chance 0.388; a random net of 20 holds exactly 2 with chance 0.285 | NOTE: real nets this small do not sit near 10%, so pinning them would contradict the picture. The picture now tags the trays "illustration". |
| 03_6 | "A crowded patch has fifty fish, five of them marked. A sparse one has ten, with one marked." | `A-CH03.inset.*` | Side 0.04; the lake-average square holds 16 fish. **Crowded:** x 0.32–0.36, y 0.76–0.80, 50 fish, 5 marked (3.125× average density). **Sparse:** x 0.48–0.52, y 0.46–0.50, 10 fish, 1 marked (0.625×). Of the 9,409 squares, 25 have 50 fish (8 of them with exactly 5 marked) and 185 have 10 fish (62 with exactly 1). A direct recount agrees | MATCH. These are real squares of the habitat run's t = 400 frame, **selected** to match the spoken counts. The rule (nearest the weed-bed centre and nearest the lake centre) is in the oracle. |
| 08_5 | lower-third "exact fractions tie, … picks 10,000 by rounding" | `A-F9.lgamma_logL9999_minus_logL10000`, `.lgamma_prefers`, `.lgamma_scan_argmax`, `.float_ratio_L10000_over_L9999` | logL(9,999) − logL(10,000) = −7.276 × 10⁻¹² with `math.lgamma`, so it prefers 10,000, and the scan's argmax is 10,000. The float ratio 81,000,000/81,000,000 is exactly 1.0 | NOTE: which size wins in floats depends on the method. The lower-third now says "an lgamma log-likelihood picks 10,000 by rounding". |
| 09_3 | "exact SD 9.0 (the runs' own SD is 8.9)" | `A-F15.sd_k_exact`, `A-F19.sim_k_sd` (existing) | 9.00045; 8.91199 (numpy `std`, ddof 0, of the 10,000 seeded k) | MATCH; no new key needed |
| 10_3 notes | "At k = 0 Chapman still answers (2,600 here)" | `A-F21.chapman_at_k0` | (50 + 1)(50 + 1)/(0 + 1) − 1 = 2,600 | MATCH (exact) |
| 11_2 | marker at the 324th zero; "smallest positive ≈ 4.9 × 10⁻³²⁴" | `A-F32.smallest_subnormal_*`, `A-F32.zeros.*` | The smallest subnormal is exactly 2⁻¹⁰⁷⁴ = 4.9407 × 10⁻³²⁴, so its first nonzero digit is at decimal place 324. P1900's mantissa 5.3515 × 10⁻ᶻ, converted to float64 with correct rounding, is nonzero up to Z = 324 and 0.0 for every Z from 325 to 430 | MATCH after the label fix: "a float64 rounds to zero **past** here" (at Z = 324 itself the value survives as 4.9 × 10⁻³²⁴). |
| 11_3 | "we'd expect to catch about one marked fish" | `A-CH11.*` | E[K] = nM/N = 10⁶/10⁶ = 1. The formula is checked against the exact pmf for every 1 ≤ M, n ≤ N ≤ 25 | MATCH (exact) |
| 12_2 | "97 clack on, 3 miss"; "all 10,000: 447 missed"; lower-third "95.45% at this one" | `A-CH12.ring_toss.*` | At N = 10,000 the rings catch the peg exactly for k = 83 … 118. Of the first 100 experiments, 97 ring and 3 miss (runs 40, 61 and 80). Of all 10,000, 447 miss, against 454.7 expected. Exact coverage is 0.9545281 (95.45%) | MATCH. The experiments are A-F19's seeded stream, which is the 09_6 panel's own k. The test is in exact integers, and `exact_ci` run at k = 82, 83, 118 and 119 agrees. |
| 12_5 | "Four times as many halves the range"; picture "−16% / +21% at 100"; "catching all of it leaves no doubt" | `A-CH12.slider.*` | k = 25 / 100 / 400 (M = n = 500 / 1,000 / 2,000): ranges [6,936, 15,210] / [8,400, 12,073] / [9,254, 10,851], widths 8,274 / 3,673 / 1,597. Each fourfold step multiplies the width by 0.444, then 0.435. At 100 the ends are −16.00% and +20.73%. With the whole lake caught twice the range is [10,000, 10,000], since P(K ≥ 10,000) at N = 10,001 is 1/10,001 | MATCH. "Halves" is the 1/√k part; the growing share M/N narrows the range a little more, which is the next sentence's point. |
| 14_6 | "Reviewers of different skill are fine. Bugs of different difficulty are not." | `A-CH14.reviewers.*` | **Skill** (60 alike bugs, chances 0.5 and 0.4): E = 30 / 24 / 12, and 30·24/12 = 60 exactly. Over 20,000 runs the median estimate is 60.0 (mean 61.80; no run with k = 0). **Difficulty** (30 easy bugs at 0.8 / 0.7, 30 hard at 0.2 / 0.1): E = 30 / 24, but 17.4 caught twice, so 1200/29 = 41.38. The median is 41.25, and 99.9% of runs fall below 60 | MATCH as a MODEL number (seeded, `default_rng([20261009, 14])`). The model's assumption is in row 14 below. |

The 03_5, 03_6 and 12_2 rows above record the claim walk; R2 replaced what those beats show (next
section), and the old keys keep their values.

## R2 review round (2026-10-11): keys for the R2 fixes

The R2 reviews (`docs/reviews/capture_recapture/R2-*.md`) and their dispositions (`R2.md`) changed
the words and pictures below. Each new number has a key; the rule or model is in the comment above
each function in `oracle.py`. The draft video description, `docs/description.md`, states only
what this sheet and the oracle back.

| beat | storyboard claim (after R2) | keys | oracle value | status / how computed |
|---|---|---|---|---|
| 01_8 | lower-third: "all 100 squares here average exactly 10,000 … only 22 of the 100 single dips land within 2×" | `A-F27.all_cells.mean_estimate`, `.share_of_dips_within_2x_of_truth` (existing) | 10,000.0; 0.22 | MATCH; no new key |
| 03_5 | nets of a thousand at the weeds, open water and the deep end hold 105, 94, 103 marked; all nine region centres 90–105 | `A-CH03.portable.nets1000.*` | 90, 93, 103, 100, 94, 100, 105, 100, 99 (SW … NE); 9.0%–10.5% | MATCH. The 1,000 fish nearest each region centre (squared distance, ties to the lower index) at t = 400 of the portable run, the one the scene draws |
| 03_6 | "The fullest corner has twenty-eight hundred eleven fish, two hundred eighty-two marked, and the emptiest has two hundred four, twenty-five marked. Both are near one in ten, and the small one wobbles more." | `A-CH03.portable.corners.*`, `.t400.count` | SE 2,811 / 282 (10.0%); NE 204 / 25 (12.3%); both extremes unique | MATCH. Most and least crowded regions by fish count. 12.3% is 1.1 SD above 10% for a 204-fish region (SD √(0.09/204) = 2.1 points): the "wobbles more" |
| 03_7 | "In both columns, the second net takes the same one in ten" | `A-F29.green_share_*_N10000`, `A-F29.unmarked_column_N10000` | 100/1,000 = 900/9,000 = 0.1 | MATCH (exact) |
| 05_7 | screen "16.67 = 1,667%" | `A-F7.toy_area_pct*` | 5000/3 % = 1,666.67%, shown 1,667% | MATCH (exact) |
| 06_4 | description: each fixed-size row peaks at ⌊(n+1)(M+1)/(N+2)⌋, tied with one less when that is whole | `A-CH06.row_mode_rule.*` | 0 failures in 23,820 cases (every N ≤ 40, every M, n) | MATCH (exact brute force) |
| 07_2 | "twenty-three times twenty-three, over thirty-three times sixteen"; screen 576 = 24 × 24 over 578 = 34 × 17 | `A-F6.N33.*`, `A-F6.N34.*` | 23, 23, 16; 24, 24, 17; factors equal the exact L(N)/L(N−1) | MATCH (exact) |
| 09_3, 09_5, 09_6 | panel nets fish; "most land within about nine"; "their average is about ten thousand and ninety"; "they agree" | `A-CH09.panel.*` | SD of k 8.960 (exact 9.000); mean 10,088.586 (rounded to 10 = 10,090), standard error 9.21; exact mean on the panel's last line 10,082.499; gap 0.66 standard errors; quantiles 8,474.58 / 10,000 / 12,048.19; no k = 0; Σp = 1.0; p[100] = 0.0442892; printed line `10088.586031204479 10082.498763370464` | MATCH (seeded, `default_rng(20261009)`, `Generator.choice`). Printed with numpy 2.5.4; Dev's R2 review reports the same mean, misses (runs 13, 71) and 464 on numpy 2.4.1 (reviewer-reported) |
| 10 notes | numpy: the soup guess at k = 0 | `A-CH09.numpy_*` | `inf`, with a RuntimeWarning | MATCH |
| 10_3 | "Here we expect only two and a half caught twice … about six and a half percent low … within two percent" | `A-F21.expected_k`, `A-F21.chapman_bias_pct` (existing), `A-F18.bound_pct_at_expected_k4`, `A-F18.bound_holds_N_le_40` | 5/2; −6.48%; 100·e⁻⁴ = 1.83%; the bound \|bias\| ≤ N·e^(−Mn/N) holds on all 22,140 cases with N ≤ 40 | MATCH. Proof (Ruth, R1): \|bias\| = (N+1)·Π_{j=0..n} (N−M−j)/(N+1−j), each factor ≤ 1 − M/N, and (N+1)(1 − M/N)^(n+1) ≤ N·e^(−Mn/N) |
| 10 notes | planning card: 1/√(caught twice) at 10 and 100 | `A-CH12.planning.*` | 31.6%; 10% | MATCH (the R1 card's "±30%" now reads "about ±32%") |
| 12_2 | 98 ring, 2 miss (runs 13, 71); "all 10,000: 464 missed (455 expected)"; "in the long run … whatever that size is" | `A-CH12.ring_toss_panel.*`, `A-CH12.coverage.*` | 98 / 2 / 13,71 / 464; 454.7 expected. Coverage over 27,901 lake sizes from 1,000 to 1,000,000: minimum 95.007% at N = 10,967 (exact integers agree), never below 95%; 95.45% at 10,000 | MATCH. Coverage ≥ 95% holds for every N by construction (each one-sided test rejects with chance ≤ 2.5%, since P(F(K) ≤ a) ≤ a for a discrete cdf F); the grid confirms it. Ruth's own grid found 95.014% at 1,857, which this grid also contains |
| 12_5 | caption "typical error ±9% … rule of thumb 1 ÷ √(caught twice) = 10%" (19 vs 20 at 25; 4 vs 5 at 400) | `A-CH12.slider.k*.typical_error_pct`, `.one_over_sqrt_k_pct` | 19 / 9 / 4%; 20 / 10 / 5% | MATCH (symmetric approximation √((1 − M/N)(1 − n/N)/k)) |
| 13_1, 13_6, 13_8 | Mn/k aims at the lake on marking day; deaths leave 7,000 at the second catch; newcomers make it 12,000 | `A-CH13.*` | target Mn/E[k]: ideal 10,000; deaths 10,000; newcomers 12,000; marks lost 12,500. Expected marked share of the second catch 0.1 / 0.1 / 0.0833 / 0.08. Lake at the second catch: 7,000 (deaths, expected) and 12,000 (newcomers) | MATCH (exact, A-F22's models; deaths by symmetry: each fish of the second catch is marked with chance M/N) |
| 14_5, 14_6 | "If every bug is equally easy to find, the careful range runs from forty-seven to ninety-four"; "expect sixty, and the range, to run low" | `A-CH14.reviewers.*.ci95_*` | skill: the range holds 60 in 97.2% of runs (1.5% wholly below, 1.2% above); difficulty: holds 60 in 22.9%, wholly below in 77.1%, never above | MATCH as a MODEL number (the A-CH14 runs; `exact_ci` on each run's own counts) |

## Non-numeric claims and their sources

Verbatim quotes below were read in the fetched source. "VERIFIED (re-checked)" means I fetched the
source myself and found the sentence. "agent-verified" means only a research sub-agent saw it.

1. **Feller's example, by edition.**
   - **1st edition, VERIFIED (re-checked).** In the 1st edition (1950; copy read: "Copyright,
     1950 … SECOND PRINTING, MAY, 1951") the example is chapter 2, §5 "The Hypergeometric
     Distribution", **Example (e) "A Sampling Problem", pp. 37–38**. The running head is
     "2.5] *THE HYPERGEOMETRIC DISTRIBUTION 37"; the ratio is eq. (5.5).
     - Source: https://archive.org/details/dli.ernet.5666 (full-text OCR
       `…/5666-An%20Introduction%20To%20Probability%20Theory%20And%20Its%20Applications_djvu.txt`).
     - Quote: "(e) A Sampling Problem. Suppose that 1000 fish caught in a lake are marked by
       red spots and released. After a while a new catch of 1000 fish is made, and it is found
       that 100 among them have red spots."
     - Small correction to the plan: the 1st edition numbers chapters in Arabic ("2.5"), not
       "§II.5".
   - **3rd edition (1968), UNVERIFIED.** The memo places it at Ch. II §6, **Example (c)**
     "Estimation of the size of an animal population from recapture data", pp. 45–46,
     eq. (6.3). The only legitimate copies (archive.org `introductiontopr0001fell`,
     `introductiontopr0001unse`) are lending-only. The memo's reading came from a third-party
     scan, which I did not use.
2. **Footnote 13 (3rd ed.), UNVERIFIED (same reason).** The memo's text: "This example was used
   in the first edition without knowledge that the method is widely used in practice", citing
   Bailey (1951) and Chapman (1951).
   - Consistent evidence (re-checked): the 1st-edition text contains no mention of Bailey,
     Petersen, Lincoln, Schnabel or "capture". "Chapman" occurs only in "Chapman-Kolmogorov".
3. **The quotes.**
   - **"An unknown fixed number" (3rd ed.), UNVERIFIED.** The memo's text: "Notice that we
     consider n as an unknown fixed number which in no way depends on chance." The **1st
     edition differs**, p. 37 (VERIFIED, re-checked): "Note, incidentally, that n is a fixed
     number which in no way depends on chance."
   - **The 1950 "meaningless" sentence, VERIFIED (re-checked), 1st ed. p. 37:** "It is,
     therefore, meaningless to ask for the probability that n is greater than, say, 6000." That
     the 3rd edition dropped it is UNVERIFIED by me.
   - **Fisher in Feller, VERIFIED (re-checked), 1st ed. p. 38:** "…is denoted by h and is called
     the maximum likelihood estimate of n. This notion was introduced by R. A. Fisher." The OCR
     reads "h" where the print has the hatted n.
   - **Feller's estimate and bet, VERIFIED (re-checked), 1st ed. p. 38:** "In our particular
     example the maximum likelihood estimate of the number of fish is n = 10,000." (OCR; the hat
     is presumably lost.) And: "These figures would justify a bet that the true number n of fish
     lies somewhere between 8500 and 12,000."
   - **The "same locality" caveat (09_8), VERIFIED (re-checked), 1st ed. p. 37:** "(In practice
     this assumption excludes situations where the two catches are made at one locality and
     within a short time.)"
4. **Maximum likelihood "developed and named by Fisher (1912–1922), with earlier versions by
   Bernoulli, Lambert and Gauss" (Stigler 2007): MISMATCH in wording.**
   - **Source:** Stigler, "The Epic Story of Maximum Likelihood", *Statist. Sci.* 22(4):598–620,
     doi:10.1214/07-STS249, https://arxiv.org/abs/0804.2996. Page numbers below are arXiv
     pages. Re-checked in the arXiv PDF:
     - "There were early intelligent comments related to this problem already in the 1750s by
       Thomases Simpson and Bayes and by Johann Heinrich Lambert in 1760, but the first serious
       assault related to our topic was by Joseph Louis Lagrange in 1769" (§2).
     - Daniel Bernoulli "sought as the true value for the observed quantity, that value that
       made the product a maximum (Bernoulli, 1769, 1778; …; Laplace, 1774)".
     - "The philosophically most cogent of these early treatments was that of Gauss, in his
       first publication on least squares in 1809".
     - "When Lambert (1760) in a sketchy presentation gave only one example, he got what was
       arguably the wrong answer there" (§13).
     - Fisher "even published in 1912 a short piece commending the virtues of the Gaussian
       approach to estimation" (§4).
     - The monumental paper "that he read to the Royal Society in November 1921 and published in
       1922".
     - His Nature abstract (24 Nov 1921) stated "Statistics obtained by the method of maximum
       likelihood are always sufficient statistics."
   - **"1912–1922" (agent-verified, not re-checked by me):** this is the title of Aldrich (1997),
     "R.A. Fisher and the making of maximum likelihood 1912–1922", *Statist. Sci.* 12(3). Its
     abstract begins "In 1922 R. A. Fisher introduced the method of maximum likelihood. He first
     presented the numerical procedure in 1912."
   - **Suggested:** "developed by Fisher between 1912 and 1922 (Aldrich 1997; Stigler 2007), with
     earlier versions by Lagrange, Daniel Bernoulli, Laplace and Gauss (Stigler 2007)". Drop
     Lambert, or call his "early comments". Or keep Feller's own line, "This notion was
     introduced by R. A. Fisher" (as Feller says).
5. **Chapman (1951) and Bailey (1951).**
   - **Bailey, VERIFIED (re-checked, Crossref):** "On estimating the size of mobile populations
     from recapture data", *Biometrika* 38(3–4):293–306, doi:10.1093/biomet/38.3-4.293.
   - **Chapman, agent-verified only:** "Some properties of the hypergeometric distribution with
     applications to zoological sample censuses", *Univ. California Publ. Statist.* 1(7). The
     agent reports HathiTrust's catalog gives "131-159 p.", while the memo and citing papers give
     131–160. I could not reach HathiTrust (Cloudflare), so the end page is unresolved.
   - **That Feller's footnote cites both: UNVERIFIED (3rd ed.).**
6. **Chapman's unbiasedness identity.** Cite the identity (proved; checked exactly here, A-F18)
   plus Wittes (1972) *as reported by* secondary sources. Wittes, *Biometrics* 28(2):592–597, and
   Seber (1982) were not accessible: UNVERIFIED as primary sources.
   - **Sadinle (2008), CERAC Working Paper 13, p. 4** (re-checked),
     https://www.cerac.org.co/assets/pdf/Other%20publications/CERAC_WP_13_Sadinle_PerformanceDSE.pdf:
     "From this Wittes [1972] demonstrates that the Chapman estimator is unbiased if n1 + n2 ≥ N,
     and that when n1 + n2 ≤ N its bias is given by …". Its formula (4) equals A-F18's closed
     form.
   - **Dettloff (2023), *Fisheries Research* 265:106756, §2.2.2** (re-checked),
     https://repository.library.noaa.gov/view/noaa/52288/noaa_52288_DS1.pdf: "The Chapman
     estimator has been shown to be exactly unbiased when the sum of the sample sizes is at least
     as large as the population size, or M + n > N (Robson and Regier, 1964; Wittes 1972)." The
     symbol in the extracted text reads ">"; the words say ≥.
7. **Schnabel 1938 in the *Monthly*, VERIFIED (re-checked, Crossref).** "The Estimation of Total
   Fish Population of a Lake", *The American Mathematical Monthly* 45(6), first page 348, June
   1938, doi:10.2307/2304025. The end page 352 comes from the memo and a citing reference list;
   I did not re-check it.
8. **Petersen (1896) did not compute Mn/k (Goudie & Goudie 2007): PARTIAL.**
   - **Abstract (re-checked, Crossref):** "We examine the claim that the well-known Petersen
     estimator which is used in population size estimation was not in fact used by the scientist
     after whom it is named. We show how, in the early years of the last century, the modern use
     of the Petersen estimator grew from that of the fishing coefficient."
   - **Reference:** *J. R. Statist. Soc. A* 170(3):825–839, doi:10.1111/j.1467-985X.2007.00479.x.
   - The abstract does not state the conclusion in so many words; the body was not read. Never
     saying "Petersen's formula" is safe either way.
9. **Briand et al. (2000) found estimates tend to underestimate: PARTIAL.**
   - **Citation (re-checked, Crossref):** *IEEE TSE* 26(6):518–540, doi:10.1109/32.852741.
   - **Abstract (re-checked; reconstructed from OpenAlex's word index because the publisher page
     was blocked):** "Our results show that models are strongly affected by the number of
     inspectors, and therefore one must consider this factor before using capture-recapture
     models. When the number of inspectors is too small, no model is sufficiently accurate and
     underestimation may be substantial."
   - **Their recommendation:** "we recommend using a model taking into account that defects have
     different probabilities of being detected and the corresponding Jackknife Estimator". That
     answers Dev's question: M_h with the jackknife.
   - **Suggested:** "with few reviewers, these estimates can badly undercount (Briand et al.
     2000)".
10. **Laplace's sample: VERIFIED (re-checked), with a wording correction.**
    - **French source:** *Essai philosophique* (1840 ed., pp. 82–83),
      https://fr.wikisource.org/wiki/Page:Laplace_-_Essai_philosophique_sur_les_probabilités.djvu/90
      and /91. The wiki number markup is rendered and apostrophes are normalised:
      - "Le gouvernement, convaincu de l'utilité d'un semblable dénombrement, a bien voulu en
        ordonner l'exécution, à ma prière. Dans trente départemens répandus également sur toute
        la France, on a fait choix des communes qui pouvaient fournir les renseignemens les plus
        précis. Leurs dénombremens ont donné 2037615 individus pour la somme totale de leurs
        habitans au 23 septembre 1802."
      - "…j'ai trouvé que le nombre des naissances annuelles en France, étant supposé d'un
        million, ce qui porte sa population à 28352845 habitans, il y a près de trois cent mille à
        parier contre un que l'erreur de ce résultat n'est pas d'un demi-million."
    - **English source:** Truscott & Emory (1902), pp. 66–67, https://www.gutenberg.org/ebooks/58881
      (`pg58881.txt`):
      - "In thirty districts spread out equally over the whole of France, communities have been
        chosen which would be able to furnish the most exact information. Their enumerations have
        given 2037615 individuals as the total number of their inhabitants on the 23d of
        September, 1802."
      - Births "110312 boys / 105287 girls" in "the years 1800, 1801, and 1802".
      - "the number of annual births in France being supposed to be 1000000, which brings the
        population to 28352845 inhabitants, it is a bet of almost 300000 against 1 that the error
        of this result is not half a million."
    - **So:**
      - "spread" describes the **départements**;
      - the **communes were chosen for their reliable records** (not at random);
      - the government counted, at Laplace's request;
      - 1,000,000 births was **assumed**;
      - 215,599 is our sum of the printed boys and girls;
      - the ratio is printed as 28 352845/1000000.
    - **Suggested for 10_2:** "In thirty départements spread across France, officials counted
      everyone in chosen communes."
11. **Mills (1972) error seeding: VERIFIED (re-checked), with a date caveat.**
    - **Reference:** NBS SP 500-75 (1981), reference list, "[MILL72] H. D. Mills, "On
      statistical validation of computer programs," IBM Report FSC72-6015, Federal Systems
      Division, IBM, Gaithersburg, Md., 1972." It describes the method: "The technique is to
      insert known errors in the code in some way that is statistically similar to the actual
      errors." (https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nbsspecialpublication500-75.pdf)
    - **Mills's own reprint:** *Software Productivity* (1983/1988), Article 9, titled "On the
      Statistical Validation of Computer Programs (1970)"
      (https://trace.tennessee.edu/bitstreams/3cb7e71c-572e-40a5-8cab-49c60f73efb8/download).
      - Abstract: "Techniques of statistical inference are introduced into the question of
        program correctness by the intentional, but randomized, introduction of programming
        errors into a program before a testing process on it."
      - p. 73, a gift for the video: "In our application, of course, "lake" is synonymous with
        "system," "unmarked fish" is synonymous with "indigenous errors," "marked fish" with
        'calibration errors," and "catching fish" with "testing process.""
      - p. 74: "Feller also shows that the maximum likelihood estimator for the original number
        of indigenous errors … is the integer part of the expression …". Mills got the idea from
        Feller's fish.
    - **Wording:** "Mills (IBM report, 1972)" is accurate. The memo's "1972, not 1970" ignores
      Mills's own 1970 date.
12. **Kosovo: VERIFIED (re-checked).**
    - **Ball et al., report to the ICTY (3 Jan 2002), p. 6:** "From approximately 10 000 victims
      reported by name, this process identified 4 400 unique individuals. The number 4 400 is not
      an estimate; it is the actual count of uniquely reported victims." and "the overall
      estimate of 10 356 killed … We estimate this interval to be 9 002 to 12 122."
      (https://www.icty.org/x/file/About/OTP/War_Demographics/en/s_milosevic_kosovo_020103.pdf)
    - **Spagat (10 Dec 2014), p. 9:** "The BBSDA overall estimate is 10,356 with a 95% confidence
      interval of 9,002 to 12,122. The KMBD number for this period is 9,790, placing it right in
      the middle of the BBSDA range."
      (https://www.hlc-rdc.org/wp-content/uploads/2015/03/Michael-_Spagat_Evaluation_of_the_Database_KMB_December_10_2014.pdf)
    - **Caveat:** 9,790 is inside the interval but below 10,356. "Right in the middle" is
      Spagat's phrase, not ours.
    - **Scope and method, re-read in both PDFs for R2 (2026-10-11; Ruth R16, Dev dev-10).**
      - Ball et al., *Killings and Refugee Flow in Kosovo, March–June 1999: A Report to the
        International Criminal Tribunal for the Former Yugoslavia* (3 January 2002), printed
        page numbers. p. 3: "The data on killings were drawn from four sources: interviews
        conducted by the American Bar Association Central and East European Law Initiative
        (ABA/CEELI) and its partners; interviews conducted by Human Rights Watch (HRW);
        interviews conducted by the Organization for Security and Cooperation in Europe (OSCE);
        and records of exhumations conducted by international teams on behalf of the
        International Criminal Tribunal for the Former Yugoslavia (ICTY)";
        "An estimated 10 356 Kosovar Albanians were killed". p. 4, footnote 5: "the analysis of
        killings covers the period 20 March - 22 June". p. 6: the estimate "was generated by
        means of a widely-used demographic statistical technique known as 'multiple systems
        estimation'". p. 41 (appendix): "The hierarchical loglinear approach extends naturally to
        allow for the modeling of more intricate dependencies among i lists", and "the 4-way
        multiple systems estimation models fitted in this document".
      - Spagat, *A Triumph of Remembering: Kosovo Memory Book* (10 December 2014). p. 8: BBSDA
        "used a completely different method known as 'multiple systems estimation (MSE).'"
        p. 9: "The BBSDA estimates cover the period from March 20, 1999 through June 20, 1999";
        "For the sake of these comparisons we restrict KMBD to violent deaths"; and, on false
        reports in the lists, "it would not be surprising if the BBSDA estimate was a bit too
        high." (BBSDA is Spagat's name for Ball et al.; Spagat's end date, 20 June, differs from
        Ball's own 22 June.)
      - **So:** the estimate is of Kosovar Albanians killed, 20 March to 22 June 1999, from four
        sources by log-linear multiple systems estimation; the later count is the Kosovo Memory
        Book's violent deaths for the period as Spagat states it, and the comparison is
        Spagat's.
13. **02_1 "Others had been using it for years" (claim walk, G3).** The line replaced
    "Biologists had been doing it for decades", which no row supported. It rests on two rows
    above, which were checked by this sheet's author; I did not re-read them for the walk.
    - **Row 7, Schnabel (1938):** "The Estimation of Total Fish Population of a Lake", in the
      *Monthly*, 12 years before Feller's 1950 edition. The row verifies the citation only; the
      plan and storyboard describe it as a multi-census marking estimator.
    - **Row 8, Goudie & Goudie (2007), abstract only:** "in the early years of the last century,
      the modern use of the Petersen estimator grew from that of the fishing coefficient".
    - **What they support:** "others" and "for years". "Biologists" and "decades" would need a
      verified row for Lincoln (1930) or Dahl, and the sheet has none.
    - **Row 2** covers "his book never mentions them" for the 1950 edition.
14. **14_6 "Reviewers of different skill are fine": the model assumption (claim walk, G3).**
    - **When it holds:** the reviewers read independently, without comparing notes, and each bug
      is as findable as any other for a given reviewer. The reviewers' own chances may differ.
      This is the M_t model in the storyboard's Dev note.
    - **Derivation:** with chances p_A and p_B per bug, E[a] = N·p_A and E[b] = N·p_B, and
      independence gives E[k] = N·p_A·p_B. So E[a]·E[b]/E[k] = N exactly, whatever p_A and p_B
      are. `A-CH14.reviewers.skill` confirms it (median 60 of 60).
    - **When it fails:** reviewers who compare notes break the independence. Bugs that are easier
      for both reviewers swell k and bias the estimate low: `A-CH14.reviewers.difficulty`, and row
      9 (Briand et al. 2000, PARTIAL) for software inspections.
    - **Source:** none. No literature source was read for this row; it rests on the derivation
      and the seeded simulation.

### The portable chapter-03 run (2026-10-11)

The scene runs chapter 03's mixing itself (`model.rs`), so the run it shows is
the oracle's `ch03_portable`: the habitat model and steps of the numpy runs
above, with SplitMix64 noise (counter stream, seed SEED + 3) and Irwin–Hall(4)·√3
normals, so both languages compute the same bits. Pin points for the port:
`A-CH03.portable.normal_bits_step0_fish{0,1}_x_y`,
`A-CH03.portable.position_bits_step1_fish0_x_y` and
`A-CH03.portable.t400.position_bits_xor_x`. Region counts and marked counts at
t = 0, 25, 100, 400 (`A-CH03.portable.t*.count`, `.marked`,
`.marked_percent_1dp`): the dock region (S) starts at 65.9% marked; at t = 400
every region is 8.6–12.3%. The numpy runs (`A-CH03.habitat.*`,
`A-CH03.contrast_no_drift.*`, `A-CH03.diag_no_circulation.*`) stay as the
research record; the retired R1 insets and the R2 `A-CH03.corners.*` /
`A-CH03.trays.net1000.*` keys came from the numpy habitat run and are kept as
the record of that run.
