<!-- Claim walk (gate G3) of script.tsv at 49ef758, 2026-10-11: the walker's final answer, verbatim (scratch paths shortened to <scratch>/). Fixes are applied in the storyboard; see the commit that lands this file. -->

# Claim walk — Video A (script at HEAD)

Script at HEAD `49ef758`, 89 spoken lines. I edited no repo files and `git status` is still clean; scratch work is in `<scratch>/`.

## Summary

There are 148 claims: 108 OK, 7 MISMATCH, 12 UNSOURCED, 21 EDGE (17 hold, 3 fail, 1 not testable), 0 unchecked.

**No spoken number is wrong.** Every number the voice says matches its oracle key or cited source at the precision it is spoken, and screen numbers round the same way as the voice. All 7 mismatches are in the picture column or lower-thirds.

**MISMATCH (picture or lower-third):**
1. **11_2:** the marker at the 308th zero says "a float64 gives up here". 2.2×10⁻³⁰⁸ is the smallest *normal* float64; a float64 only reaches 0 below about 5×10⁻³²⁴, which is the 324th zero. The beat's own lower-third already says 5×10⁻³²⁴.
2. **12_5:** "the range in percent: ±9% at 100". ±9% is one standard deviation (A-F16). The 95% range the voice is talking about is −16% / +21% ([8,400, 12,073]).
3. **09_6:** the code panel's exact line fails with a NameError when run as shown, because the setup has no `from math import comb`.
4. **09_3:** "SD 9.0" is the exact value, but the histogram of the seeded runs has SD 8.91. Under the live-readout rule it would show 8.9.
5. **Stale cross-references:**
   - 03_8 lower-third says "(chapter 9)"; it should be 13.
   - 12_3 labels the range "07's range"; it should be 09's.
   - 02_8 lists act cards at "03_1, 07_1, 09_1"; they are at 03_1, 09_1, 13_1.

**EDGE claims that fail:**
- **13_1** "Every failure breaks one rule (equally likely to be caught)" is not literally true for lost marks (13_5). Those fish are just as catchable; they are not counted as marked.
- **14_6** "read sixty as a floor": it is a direction (probably too low), not a floor. The exact range starts at 47.
- **15_3** receipts card "Every number … computed by code or copied from a cited source" is not true yet, because of the UNSOURCED items below.

**UNSOURCED (no oracle key or fact-sheet row):**
- "One week later" (01_1) clashes with the 03_4 caption "≈ two weeks in 12 s".
- "Biologists had been doing it for decades" (02_1): the only verified row is Schnabel 1938, a mathematician writing 12 years before 1950.
- The trays in 03_5 and the patch counts in 03_6 (50/5 and 10/1). I confirmed 03_6's counts occur in the real t = 400 frame.
- 08_5's floating-point tie-break, 11_3's "about one marked fish", 12_5's "halves the range", "Douglas" (Chapman's first name, which I did not verify), and 14_6's "reviewers of different skill are fine".
- 12_2's ring-toss counts (3 and 447) and 95.45%.

Where these can be computed I recomputed them, and all agree with the storyboard. They still need keys or rows.

**Voice rules:** all 89 lines in `script.tsv` match the storyboard's words column. None contains a digit, symbol, colon, semicolon, dash, parenthesis or abbreviation. One style point: "five twenty-nine" (07_2) can be heard as a time.

**Not covered:**
- `facts.rs` pins 32 values and all match the oracle, but most spoken numbers are not pinned there yet (the A-F27 dips, A-F3/A-F5 percents, A-F13, A-F19, A-F21–A-F25, A-F30). I did not run the Rust facts-check, because building would write into the repo.
- The fact sheet's header is out of date: it says 338 keys and gives the path `docs/facts/…`, but the oracle now prints 349 keys from `explainer_corpus/…`.

## What I ran

- **The oracle:** the command you gave. It printed 349 keys in 118 s, saved to `<scratch>/oracle.tsv`.
- **`checks1.py` / `checks3.py`:** exact Fraction arithmetic for every spoken number, plus edge tests:
  - E[K] = nM/N, the ratio formula, the sign rule and the green-share rule, with 0 failures in 47,640 cases;
  - MLE = ⌊Mn/k⌋ by brute force for M, n ≤ 15;
  - the area formula on 5 cases;
  - Chapman's identity and its bound on 22,140 exact cases and 200,000 random ones.
- **`checks2.py`:**
  - ran the 09_6 panel code verbatim;
  - recomputed the ring-toss misses and exact coverage at N = 10,000;
  - redid the intervals by an independent float test inversion;
  - computed coverage across N and the 12_5 slider widths.
- **`checks4.py`:**
  - re-ran the chapter-13 failure modes with 5 other seeds;
  - simulated two reviewers with different skill, and bugs with different difficulty;
  - replayed the chapter-03 simulation and searched it for the 03_6 patches.
- **Sources:** I downloaded each into its own directory under `<scratch>/` and grepped it:
  - Feller's 1st-edition OCR (archive.org);
  - Laplace in translation (Gutenberg 58881);
  - Ball et al. 2002 (p. 6) and Spagat 2014 (p. 9);
  - Mills's reprint (p. 73).

## Table

| line | claim | source | status | note / fix |
|---|---|---|---|---|
| 01_1 | "A thousand fish are caught, marked" (M = 1,000) | Feller 1950 p. 37, read in the archive.org OCR; facts.rs `FELLER` | OK | |
| 01_1 | "One week later" | none (Feller: "After a while") | UNSOURCED | Clashes with the 03_4 caption "≈ two weeks in 12 s", which shows the marks still mixing after one week. Say "Later", or pin the sim-time-to-weeks scale in a key. |
| 01_1 | "a second net catches a thousand" (n = 1,000) | Feller p. 37; facts.rs | OK | |
| 01_1 | "a hundred are marked" (k = 100) | Feller p. 37; facts.rs | OK | |
| 01_2 | picture: cells 100 / 900 / 900 / ? | A-F29.feller_a = feller_b = 900 | OK | |
| 01_3 | "about eight thousand one hundred" | A-F29.feller_u_hat 8100; facts.rs pin | OK | Screen "≈ 8,100" matches. |
| 01_3 | "we can say how sure we are" | A-F13 (delivered at 12_3) | OK | |
| 01_5 | "covers one hundredth of the lake" | A-F27.cell_share_of_lake 0.01 | OK | |
| 01_5 | "Four hundred fifty-one fish" | A-F27.spot.deep.fish_in_net 451 | OK | Picture risk: the lake draws 1 fish per 10, so a live count of the drawn fish in that cell reads about 45. The 451 must be counted from the 10,000-fish lake. |
| 01_5 | "times a hundred, is forty-five thousand one hundred" | A-F27.spot.deep.estimate 45100 | OK | |
| 01_7 | "In the weeds, eighty-eight thousand" | A-F27.spot.weeds.estimate 88000 | OK | |
| 01_7 | "In open water, two thousand eight hundred" | A-F27.spot.open.estimate 2800 | OK | |
| 01_7 | "Not even close"; screen "31× apart" | A-F27.spread_max_over_min 31.43 | OK | The fact sheet wants "about thirty times" wherever it is said; "31×" on screen is consistent with that. |
| 01_8 | the net can't know its share of the fish | A-F27: 451 of 10,000 fish (4.5%) on 1% of the water | OK | |
| 02_1 | "In nineteen fifty, the probabilist William Feller put this puzzle in his famous textbook" | fact sheet non-numeric 1; OCR title page "Copyright, 1950", "WILLIAM FELLER", Example (e), p. 37 | OK | The copy is the 2nd printing (May 1951) of the 1950 edition. |
| 02_1 | "Biologists had been doing it for decades" | no fact-sheet row. MEMO: Petersen 1896, Dahl about 1917 (via Le Cren 1965), Lincoln 1930. Fact sheet verifies only Schnabel 1938 and the Goudie abstract | UNSOURCED | Schnabel was a mathematician writing 12 years before 1950, so on its own that source does not support "biologists … for decades". Needed: a verified row for Lincoln (1930, USDA Circ. 118) and/or Dahl. |
| 02_1 | "His book never mentions them" | fact sheet non-numeric 2; my grep of the 1st-ed. OCR | EDGE (holds for 1950) | 0 hits for Petersen, Lincoln, Schnabel, Bailey, capture, recapture or tagg. "Chapman" appears only as Chapman & Hall and Chapman–Kolmogorov. The claim fails for the 3rd ed. if footnote 13 exists (it cites Bailey and Chapman, UNVERIFIED), but the line is anchored to 1950. |
| 02_1 | quote card "Suppose that 1000 fish caught in a lake are marked by red spots…" (p. 37) | OCR, page head "2.5] THE HYPERGEOMETRIC DISTRIBUTION 37" | OK | Verbatim. |
| 02_2 | "Feller marked his fish with red spots" | OCR p. 37 | OK | |
| 02_3 | "the second net caught a hundred. One in ten" | 100/1,000 | OK | |
| 02_5 | "Ten thousand … about a tenth of the fish"; picture 10,000 − 1,900 = 8,100 | A-F29.feller_N_hat; A-F11.min_lake 1900 | OK | |
| 02_6 | "one in ten in the second catch is marked, so about one in ten in the lake is" | 100/1,000 | OK | |
| 02_6 | lower-third "E[K] = nM/N for every catch size" | exact test (mine) | EDGE (holds) | Exact equality for every N ≤ 25 and all M, n. |
| 02_7 | "A thousand times a thousand, over a hundred. Ten thousand" | A-F29.feller_N_hat 10000 | OK | |
| 02_8 | picture: question cards "return as act titles (03_1, 07_1, 09_1)" | R1 A-18: act cards are at 03_1, 09_1, 13_1 | MISMATCH | Stale beat ids: use 03_1, 09_1, 13_1. |
| 03_2 | "marked the first thousand near the dock, and let them go there" | A-CH03 design (1,000 nearest the dock, radius 0.271) | OK | |
| 03_4 | "The fish stay crowded where they like to be" | A-CH03.habitat.t0/t400.count_max_over_min 12.78 / 12.77 | OK | |
| 03_4 | "the marks spread out evenly"; picture "66% … about 10% everywhere" | A-CH03.habitat.t0.marked_fraction_max 0.659; t400 0.0925–0.1075 | OK | My replay of the habitat run gives the same t400 fractions to 4 dp. |
| 03_4 | picture "≈ two weeks in 12 s" | none: the sim has no time scale in weeks | UNSOURCED | See 01_1: either pin it or drop "two weeks". |
| 03_5 | "Wherever the second net goes, about one fish in ten it catches is marked" | A-CH03 t400 regions 9.3–10.7% | EDGE (holds, "about") | Holds for regions. A 10-fish net shows exactly 1 marked only 39% of the time; otherwise 0% or 20%. |
| 03_5 | picture: nets of 10 and 20, "each tray's marked share sits near 10%" | no key | UNSOURCED | Pin the trays from the sim, or tag them "illustration". |
| 03_6 | "fifty fish, five of them marked … ten, with one marked" | no key (the storyboard's own "Facts to pin" asks for one) | UNSOURCED | Achievable: in the t = 400 frame, squares with exactly 50/5 (crowded) and 10/1 (sparse) exist at 17 window sizes. Example: side 0.04 at (0.21, 0.63) and (0.02, 0.82). Pin one as A-CH03.inset.*. |
| 03_7 | "Mixing makes being marked unrelated to where a fish swims" | A-CH03 (flat to within binomial SD 0.020); fact sheet "Honest reading" | OK | |
| 03_8 | lower-third "equal catchability is a separate assumption (chapter 9)" | chapter list | MISMATCH | Failures are chapter 13 now. |
| 04_1 | "Twenty fish, five of them marked. Our net holds four" | A-F1–A-F4 setup | OK | |
| 04_2 | "Random means every group of four is equally likely" | definition (Feller: "random samples") | EDGE (holds) | Holds by definition; no edge case. |
| 04_3 | "Twenty … nineteen … eighteen … seventeen" (screen 116,280) | A-F1.falling_product 116280; facts.rs pin | OK | |
| 04_3 | "twenty-four orders, four times three times two times one" | A-F1.orders_of_4_fish 24 | OK | |
| 04_3 | "Four thousand eight hundred forty-five" | A-F1.C20_4 4845 | OK | |
| 04_5 | picture: bins 1,365 / 2,275 / 1,050 / 150 / 5 | A-F2.k0–k4 | OK | |
| 04_6 | biggest pile is "One marked fish. One in four, same as the lake" | A-F2.k1 2275 (max); 5/20 | OK | |
| 04_7 | "Ten ways to pick two of the five marked" | A-F4.C5_2 10 | OK | |
| 04_7 | "a hundred and five ways to pick two of the fifteen unmarked" | A-F4.C15_2 105 | OK | |
| 04_7 | "One thousand fifty" | A-F4.product 1050 | OK | |
| 04_8 | "about twenty-eight percent, forty-seven, twenty-two, three, and under one" | A-F3.screen_whole_pct0–4 28/47/22/3/<1; A-F3.p*.exact | OK | Bars and the fraction lower-third match (91/323, 455/969, 70/323, 10/323, 1/969). |
| 04_9 | "Mathematicians call it the hypergeometric distribution" | Feller §2.5 title (OCR) | OK | |
| 04_9 | "If we know the lake, we know the chances of every catch" | the pmf is fixed by N, M, n | EDGE (holds) | Holds under random sampling. |
| 05_1 | "We marked ten, caught ten, and three were marked" | A-F5 (TOY) | OK | |
| 05_3 | "About thirty-three" | A-F5.soup 33.33 | OK | |
| 05_3 | "seventeen different fish" | A-F5.min_lake 17 | OK | |
| 05_4 | "A lake of twenty, about eight percent" | A-F5.still_N20_pct 7.8 | OK | The printed heights (1 dp) round the same way as the voice. |
| 05_4 | "Thirty-three, about thirty-two percent" | A-F5.still_N33_pct 31.8 | OK | |
| 05_4 | "Sixty, about sixteen" | A-F5.still_N60_pct 15.9 | OK | |
| 05_6 | "The curve peaks at thirty-three"; screen "L ≈ 0.318" | A-F5.argmax 33; A-F5.L33 0.31783 | OK | My exact scan from 17 to 5,000 gives only 33. |
| 05_7 | "add up every height … about sixteen and two thirds, not one" | A-F7.toy_area_closed_form 50/3; partial 16.6666656 | OK | My float sum to N = 10⁶ is 16.6666666. |
| 05_7 | picture "17 to 60: 9.9 … all sizes: 16.67" | A-F5.area_17_to_60 9.89 | OK | The live running sum at N = 2,000 is 16.656, which displays as 16.66. Keep "all sizes: 16.67" as a separate closed-form label. |
| 05_7 | card "It is, therefore, meaningless to ask for the probability that n … is greater than, say, 6000." | OCR p. 37 | OK | Verbatim; the bracketed gloss is marked as such. |
| 05_7 | lower-third Σ L(N) = Mn/(k(k−1)) for k ≥ 2, infinite for k ≤ 1 | A-F7; my sums | EDGE (holds) | Sums to N = 2×10⁶ match the formula for (10,10,3), (7,4,3), (6,6,4), (5,5,2) and (8,3,2). The k = 2 cases converge slowly (12.4999 vs 12.5). k = 1 diverges per the A-F7.k1 keys. |
| 06_1 | picture "44 sizes × 11 counts" | N = 17…60 is 44 sizes; k = 0…10 | OK | |
| 06_3 | "They add to one" | exact (mine) | EDGE (holds) | Σ_k P_N(K = k) = 1 exactly for every N from 17 to 60. |
| 06_4 | "Its heights add to sixteen and two thirds" | A-F7 | OK | The slice on screen (17–60) sums to 9.89, so any Σ label on the landscape must say "all sizes". |
| 07_1 | "Feller found the top score without drawing anything. Compare each lake size with the one just below it" | Feller eq. (5.5), p. 38 (OCR) | OK | |
| 07_1 | lower-third L(N)/L(N−1) = (N−M)(N−n)/(N(N−M−n+k)), N > M+n−k | exact test (mine) | EDGE (holds) | 0 failures in 47,640 cases (M, n ≤ 12, all k, 60 values of N each). |
| 07_2 | "five twenty-nine against five twenty-eight … the score goes up" | A-F6.N33 529/528; facts.rs pins | OK | Style: "five twenty-nine" can be heard as a time. Consider "five hundred twenty-nine". |
| 07_2 | "five seventy-six against five seventy-eight, so it goes down" | A-F6.N34 576/578 | OK | |
| 07_4 | "Marked times second catch, minus caught twice times lake size" survives | my expansion; A-F6 numerator − denominator keys | OK | The struck pairs N², N·n, N·M are correct. |
| 07_4 | "While that's positive, the score is still rising" | exact test (mine) | EDGE (holds) | sign(L(N) − L(N−1)) = sign(Mn − kN) in all 47,640 cases. |
| 07_5 | Mn − kN = ab − k·u | A-F29.identity_proved_on_grid; facts.rs test; my algebra | OK | |
| 07_6 | "seven times seven, minus three times sixteen. One, same as before" | A-F29.toy; A-F6.N33.Mn_minus_kN 1 | OK | |
| 08_2 | "While the unmarked side's green share is bigger, the score keeps climbing" | exact test (mine) | EDGE (holds) | (n−k)/(N−M) > k/M ⇔ L(N) > L(N−1): 0 failures. |
| 08_3 | "A hundred times eight thousand one hundred is eight hundred ten thousand" | A-F29.feller_k_times_u_hat; facts.rs pin | OK | |
| 08_3 | "So is nine hundred times nine hundred" | A-F29.feller_ab 810000 | OK | |
| 08_4 | "A straight line is exactly what two independent catches look like" | algebra (k·u = a·b ⇔ kN = Mn) | EDGE (holds) | Exact for the 2×2 table (odds ratio 1). |
| 08_4 | "the same answer, rounded down to a whole fish" | brute force (mine); facts.rs test | EDGE (holds for k ≥ 1) | argmax (largest of any tie) = ⌊Mn/k⌋ for all M, n ≤ 15 and 1 ≤ k ≤ min(M, n). All 538 ties sit where Mn/k is whole and k < min(M, n). With k = 0 there is no top (L rises toward 1); 10_2 covers that case. |
| 08_5 | "nine thousand nine hundred ninety-nine and ten thousand tie" | A-F9.argmax, A-F9.tie_exact; my Fractions | OK | |
| 08_5 | "Feller just says ten thousand" | OCR p. 38 "…the maximum likelihood estimate of the number of fish is n = 10,000" | OK | |
| 08_5 | lower-third "floating point picks 10,000 by rounding" | no key (A-F9.lgamma_argmax_in_tie says only "in the tie") | UNSOURCED | Recomputed: math.lgamma gives logL(9,999) − logL(10,000) = −7.3e−12, so 10,000. Which one wins depends on the library; add a key. |
| 09_1 | "a lake of exactly ten thousand fish … ten thousand times" | A-F19.sim_experiments 10000 | OK | |
| 09_3 | "In a bell around a hundred" | E[K] = 100 exactly; A-F19.sim_k_mean 100.01 | OK | |
| 09_3 | "Most land within about nine of it" | A-F15.sd_k_exact 9.0005 | OK | P(91 ≤ K ≤ 109) = 70.9% (mine). |
| 09_3 | picture "Histogram of k (mean 100, SD 9.0)" | A-F15 exact 9.00 against A-F19.sim_k_sd 8.91 | MISMATCH | The histogram is of the seeded runs, whose SD reads 8.9 under the live-readout rule. Label "exact SD 9.0" or show 8.9. |
| 09_4 | "Ninety caught twice gives about eleven thousand one hundred" | A-F31.Mn_over_90 11,111.1 | OK | |
| 09_4 | "A hundred and ten gives about nine thousand one hundred" | A-F31.Mn_over_110 9,090.9 | OK | |
| 09_4 | "Ten too few pushes harder than ten too many" | A-F31 excess 1,111 against shortfall 909 | OK | Follows from 1/k being convex. |
| 09_5 | "Ninety-five percent … from about eighty-five hundred to just over twelve thousand" | A-F19.exact_q025 8,474.58 / q975 12,048.19 (sim identical) | OK | Exact P(83 ≤ K ≤ 118) = 95.45% (mine). Screen 8,475 / 12,048 rounds the same way. |
| 09_5 | "Their average is about ten thousand and eighty, a little high" | A-F19.sim_mean 10,080.002 (exact conditional 10,082.5) | OK | |
| 09_6 | "Two lines of code" | I ran the panel lines | OK | Mean 10,080.002 and p2.5 / p97.5 8,474.58 / 12,048.19, identical to the A-F19.sim keys. |
| 09_6 | panel's exact line `p = [comb(…)…]` | I ran it as shown | MISMATCH | NameError: `comb` is not defined, because the greyed setup has only `import numpy as np`. Add `from math import comb`. With it, Σp = 1.0 and p[100] = 0.044289. |
| 09_6 | "Always check a simulation against something you can compute exactly" | advice | EDGE (n/a) | Normative and not testable; the panel follows it. |
| 10_2 | "About one time in fourteen"; screen 7.2% | A-F21.P_k0 0.07198 (1 in 13.89) | OK | |
| 10_3 | "Douglas Chapman's fix" | Chapman 1951 (fact sheet 5, agent-verified only); the first name is in no row | UNSOURCED | "D. G. Chapman" = Douglas George Chapman as far as I recall. Not checked against a source; add it to the row. |
| 10_3 | "add one to each count, and subtract one at the end" | A-F17, A-F18: (M+1)(n+1)/(k+1) − 1 | OK | |
| 10_3 | "They're what makes the average come out right" | A-F18.zero_bias_iff_M_plus_n_ge_N; my exact test | EDGE (holds with qualifier) | Exactly right only when M + n ≥ N; otherwise slightly low (0 failures for N ≤ 40, all M, n). The next sentence carries the qualifier. |
| 10_3 | "It never divides by zero" | k + 1 ≥ 1 | EDGE (holds) | At k = 0 Chapman gives 2,600 here (mine; the lower-third value has no key). |
| 10_3 | "once you expect four or more caught twice, it's nearly exact on average" | A-F18.bias_pct_N10000_M200_n200 −1.62% | EDGE (holds) | 200,000 random triples (N up to 2×10⁵) never break the bound N·e^(−Mn/N), and the worst case with E[k] ≥ 4 is −1.46%. "Nearly exact" = within 2%. |
| 10_3 | picture: plain +24%, Chapman −6.5% | A-F21 +24.12% / −6.48% | OK | |
| 11_2 | "Only nineteen hundred fish?" | A-F11.min_lake 1900 | OK | |
| 11_2 | "all nine hundred unmarked … a hundred of the thousand marked" | 1,900 − 1,000; k = 100 | OK | |
| 11_2 | "about five in a one followed by four hundred thirty zeros" | A-F11.P1900 5.3515e−430; Feller p. 37 "10^−430" | OK | |
| 11_2 | picture: marker at the 308th zero, "a float64 gives up here" | A-F32: smallest normal 2.2e−308, smallest positive 4.9e−324 | MISMATCH | float64 reaches 0 only below 5×10⁻³²⁴ (the 324th zero), as the beat's own lower-third says. Move the marker to 324, or relabel 308 "float64 starts losing digits". |
| 11_3 | "we'd expect to catch about one marked fish" | no key (A-F12 is only the likelihood ratio) | UNSOURCED | Recomputed: Mn/N = 10⁶/10⁶ = 1. Trivial; add a key or leave as derivation. |
| 11_4 | "about a four percent chance"; screen 4.43% | A-F9.L_max 0.04429 | OK | |
| 11_5 | "Exactly fifty heads … about eight percent" | A-F30 7.96% | OK | |
| 11_5 | "it's the most likely count" | A-F30.50_is_mode | OK | |
| 12_1 | construction: keep sizes where 100 lands in the middle 95% | A-F13 / exact_ci definition (both tails > 2.5%) | OK | |
| 12_2 | "rings the true size at least ninety-five times in a hundred, whatever the true size is" | exact coverage (mine) | EDGE (holds) | For M = n = 1,000, coverage is 95.34–97.68% across N = 1,900–100,000. The minimum over all M, n ≤ 12 and N ≤ 60 is 95.02%. |
| 12_2 | picture: "97 clack on, 3 miss … all 10,000: 447 missed" | no key (storyboard asks for one) | UNSOURCED | Recomputed on the 09_6 seeded stream: 3 of the first 100 and 447 of 10,000 (454.7 expected). Matches; pin the keys. |
| 12_2 | lower-third "95.45% at this one" | no key | UNSOURCED | Recomputed exactly: 0.954528. Matches. |
| 12_3 | "about eighty-four hundred to about twelve thousand one hundred" | A-F13.ci95 [8,400, 12,073] | OK | My float test inversion agrees. Screen matches. |
| 12_3 | "Close to the range we watched when the lake was known" | A-F19 8,475–12,048 | OK | |
| 12_3 | picture label "07's range (8,475–12,048)" | chapter list | MISMATCH | Stale: it is chapter 09's range. |
| 12_3 | lower-third: Feller's 8,500–12,000 = 3.5% / 2.9%, ≈ 93.6% | A-F14 0.0347 / 0.0289 / 0.9364 | OK | Feller's own normal approximations (≈0.04 and ≈0.03) checked on p. 38. |
| 12_5 | "Four times as many halves the range" | no key | UNSOURCED | Recomputed in the slider model (N = 10,000, M = n = √(10,000k)): width 3,673 → 1,597 (×0.43) for k 100 → 400, and 8,274 → 3,673 (×0.44) for k 25 → 100. The 1/√k part halves the width and the growing share does the rest, so the claim holds with the next sentence. |
| 12_5 | "Catching a big share of the lake helps a little" | A-F16 factor (1−M/N)(1−n/N); my widths | OK | Worth about 11% here. |
| 12_5 | "catching all of it leaves no doubt" | my test inversion | EDGE (holds) | M = n = k = 10,000 gives [10,000, 10,000]. If only the second catch is the whole lake (M = 1,000, n = 10,000, k = 1,000), the range is [10,000, 10,035]. |
| 12_5 | picture "the range in percent: ±9% at 100" | A-F16.cv_symmetric_approx 0.09 is one SD; A-F13 range is −16% / +20.7% | MISMATCH | Show "−16% / +21%", or label "typical error (1 SD) ±9%". |
| 13_1 | "Every failure breaks one rule … marked and unmarked fish must be equally likely to be caught" | A-F22 models; R1 R-01 | EDGE (fails literally) | Holds for bold, trap-happy/shy and unmixed marks; deaths break nothing; newcomers change the target. Lost marks (13_5) do not break equal catchability: a marked fish is as catchable but is not counted as marked. The lower-third's share wording covers that case; the voice does not. Fix: "…equally likely to be caught, and seen as marked" or tie 13_5 to the rule. |
| 13_3 | "Too small … three fish in ten are three times as easy to catch … around seventy-seven fifty" | A-F22.median.bold_fish 7,751.94; model het = 3 with p = 0.3 | OK | Seeds 1–5 give 7,752–7,812, so the value is robust. |
| 13_4 | "Fish that learn to love the bait do the same" (screen ≈ 7,140) | A-F22.median.trap_happy 7,142.86 | OK | Other seeds: 7,092–7,194. |
| 13_4 | "Fish that learn to avoid it push the estimate up" (screen ≈ 14,490) | A-F22.median.trap_shy 14,492.75 | OK | Other seeds: 14,286–14,493. The screen value is the seeded one. |
| 13_5 | "Marks that wash off … pushes it up too" (screen ≈ 12,500) | A-F22.median.marks_lost_20pct 12,500 | OK | |
| 13_6 | "Two thousand newcomers … the lake as it is at the second catch" (≈ 12,120 against 12,000) | A-F22.median.newcomers_2000 12,121.66 | OK | |
| 13_8 | "The estimate doesn't move" (30% deaths) | A-F22.median.deaths_30pct 10,000; E[k] = 1,000·700/7,000 = 100 (mine) | OK | |
| 13_9 | "near the release site … sixty-two fifty" | A-F22.median.release_site 6,250 | OK | |
| 13_9 | "the far side … twenty-five thousand" | A-F22.median.far_side 25,000 | OK | |
| 13_9 | silent card: Feller's caveat "catches made in one spot within a short time" | OCR p. 37: "(In practice this assumption excludes situations where the two catches are made at one locality and within a short time.)" | OK | The card paraphrases. If it is shown in quote marks, use the verbatim sentence. |
| 14_1 | "Around eighteen hundred, the French mathematician Laplace used the same table to count France" | fact sheet 10; Gutenberg 58881 text (count of 23 Sep 1802; births 1800–1802) | OK | |
| 14_2 | "At his request, officials counted everyone in chosen towns across France" | Truscott & Emory text: "decided at my request to order its execution. In thirty districts … communities have been chosen" | OK | |
| 14_2 | "About two million people" | A-F23.recorded.inhabitants 2,037,615 (in the text) | OK | |
| 14_2 | "about seventy-two thousand births a year" | A-F23.births_per_year 71,866⅓ | OK | |
| 14_2 | "one birth for every twenty-eight and a bit people" | A-F23.ratio 28.3528 (text: "28 + 352845/1000000") | OK | |
| 14_2 | "Taking France's births as a million a year … twenty-eight point four million" | A-F23.population_estimate 28,352,845 (text: "supposed to be 1000000 … 28352845") | OK | |
| 14_3 | card "a bet of almost 300000 against 1" | Gutenberg 58881 text | OK | Verbatim. Page 67 not checked (plain text has no page numbers). |
| 14_4 | "One finds thirty bugs, the other twenty-four, and twelve are the same" | A-F24 inputs (hypothetical, "Say") | OK | |
| 14_4 | lower-third Mills, 1970: "'lake' is synonymous with 'system'…" (p. 73) | Software Productivity reprint, read in the PDF: on p. 73; the TOC dates the article "(1970)" | OK | Verbatim. The fact sheet asks for "Mills (IBM report, 1972)" or both dates; "1970" alone is the reprint's date. |
| 14_5 | "Thirty times twenty-four, over twelve, is sixty" | A-F24.estimate 60; facts.rs pin | OK | |
| 14_5 | "They found forty-two, so about eighteen" | A-F24.seen 42, hiding 18; facts.rs pins | OK | |
| 14_5 | "forty-seven to ninety-four bugs in all" | A-F24.ci95 [47, 94] | OK | My independent inversion agrees. |
| 14_6 | "Reviewers of different skill are fine" | no fact-sheet row (storyboard Dev note, M_t) | UNSOURCED | Simulated: 60 alike bugs, skills 0.5 / 0.4, 20,000 runs. Median estimate 60.0 (truth 60). Holds if the reviewers are independent; add a row (e.g. Otis et al. 1978 or Briand 2000). |
| 14_6 | "Bugs of different difficulty … pull the estimate down, like the bold fish" | Briand et al. 2000 (fact sheet 9, PARTIAL); A-F22 bold | OK | My sim (half easy, half hard bugs, same expected 30 / 24) gives a median of 41 against a truth of 60. |
| 14_6 | "So read sixty as a floor" | A-F24.ci95_lo 47; my difficulty sim | EDGE (fails as a bound) | It is a direction, not a floor. With alike bugs the truth can be as low as 47 (A-F24.ci95_lo), and reviewers who specialise in different bugs bias it up. Say "read sixty as probably too low". |
| 15_1 | "In the war in Kosovo, in nineteen ninety-nine" | Ball et al. 2002 PDF: period 20 Mar–22 Jun 1999 | OK | |
| 15_1 | "several lists named about four thousand four hundred different victims between them" | A-F25.recorded 4,400; Ball p. 6 text "four sources … 4 400 unique individuals" | OK | |
| 15_1 | "estimated about ten thousand four hundred killed" | A-F25.recorded.estimate 10,356 | OK | Scope: Ball says "10 356 Kosovar Albanians"; Spagat compares it with all violent deaths in KMBD. The script follows Spagat. |
| 15_1 | "Years later, a name-by-name count found nine thousand seven hundred ninety" | A-F25.recorded.later_count_kmb; Spagat 2014 PDF text | OK | |
| 15_1 | "inside the estimate's range" (screen 9,002–12,122) | A-F25.later_count_inside_interval true | OK | |
| 15_3 | "Each is a table with one cell nobody can see" | four lists give 2⁴ = 16 cells, one unseen; France and the bugs are 2×2 | OK | |
| 15_3 | receipts card "Every number in this video was computed by code or copied from a cited source" | my audit of the keys against this table | EDGE (fails at HEAD) | Not yet true. These have no key: 03_6, 03_5, the "two weeks", 12_2's 3 / 447 / 95.45%, 08_5's floating-point pick, 12_5's halving, 11_3, "2,600", "Douglas", "for decades", and "reviewers of different skill". |

Questions and pure analogies (02_8's spoken questions, 03_1, 04_4, 06_2, 08_1, 15_2) make no claim and are left out. The table is also saved as `<scratch>/table.md`, and the check scripts are `<scratch>/checks1.py` to `checks4.py`.
