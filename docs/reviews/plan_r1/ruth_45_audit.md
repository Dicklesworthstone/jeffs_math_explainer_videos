Plan review round 1 (2026-10-09), mathematician audit (Ruth, 45), against plan revision 1. Findings are being dispositioned in the plan's revision 2.

# Mathematician's audit of §9–§11 (Ruth Adler)

**Scope.** I applied §5.3 to every draft-words line and every facts-ledger row in §9–§11, plus the parts of §0, §5.1 and Appendix A that repeat those claims. I compared every number with the three memos and their saved outputs, and recomputed the cheap ones myself with `python3 -I` in my scratchpad; no repo files were touched.

**Bottom line.** Almost all the numbers are right. Three ledger rows misstate the memo or contradict each other: A-F16, A-F21 and B-F21. One Ruth note has a wrong rounding (B-F1, ⌈K·H_K⌉). The serious problems are in the draft words:
- **One transcription error:** the slate order in C 03_5.
- **Two proof gaps:** Fact one and Fact two in C 07, as narrated.
- **One misattribution:** Dijkstra and starvation freedom.
- **Several probability-language slips** in Video A.
- **A wrong holonomy description** in B 09_6.

I also proved the likelihood-area identity that the plan calls unproved. The proof is in §2 below.

**Numbers I recomputed and that match** (no finding needed):
- **Toy lake:** C(20,4) = 4,845 and the split 1,365 / 2,275 / 1,050 / 150 / 5.
- **Toy ratios:** 529/528 and 576/578; numerator minus denominator is +1 and −2, equal to Mn − kN.
- **Feller's ratios:** 80,982,001/80,981,901, then 81,000,000/81,000,000, then 81,018,001/81,018,101 (differences ±100).
- **Likelihood values:** L(33) = 0.3178 and L_max = 0.04429; log₁₀ q(1900) = −429.27; relative likelihood at 10⁶ is 3.8×10⁻¹⁶².
- **Intervals:** CI endpoints 8,400 and 12,073 (tails 0.02503 / 0.02503); toy CI [19, 138]; toy Chapman 29.25.
- **Small study:** P(k = 0) = 7.198%, which is 1 in 13.9, so "about one study in fourteen" is accurate.
- **History arithmetic:** Laplace 28,352,844.86; the two-reviewer example 60 and 18.
- **Rank products:** ∏(1−2⁻ⁱ) gives 0.5, 0.375, 0.328 and tends to 0.288788. Failures are 71.12 / 42.24 / 22.99 / 11.99%. ∏(1−256⁻ⁱ) = 0.996078 for every K ≥ 1. Expected excess is 1.60670 and 0.003937.
- **Field products:** 0x57·0x83 = 0x31 mod 0x11D and 0xC1 mod 0x11B. Also 2·128 = 29, 128·3 = 157, 150·3 = 167, 7·7 = 21, 2⁻¹ = 142, 3⁻¹ = 244.
- **Logs and polynomials:** log 150 = 180, log 3 = 25, α²⁰⁵ = 167; OCT_EXP begins 1…128, 29, 58, 116, 232, 205. There are 30 irreducible and 16 primitive polynomials; ord(2) = 51 under 0x11B.
- **Other RaptorQ figures:** e^(−4.8166) = 0.81%; 0.99^(10⁶) = e^(−10,050.3); 50,000³ = 1.25×10¹⁴.
- **Bakery:** 31,557,600,000 ms per Julian year is below 2³⁵ = 34,359,738,368; 2⁶⁴ at 10⁹ per second lasts 584.5 years. I reproduced C(4,2) = 6 with 4 lost updates, and the naive flag lock's 14 schedules with 8 bad (counting one step per read, write and successful wait).

## 1. Findings

| id | location | severity | issue | correct statement / fix | how verified |
|---|---|---|---|---|---|
| A1 | §9.4 05 Ruth note; A-F7 | imprecise | Σ_N L(N) = Mn/(k(k−1)) is called "verified numerically, **not proved**". | It is a theorem for k ≥ 2; for k ≤ 1 the sum is infinite (proof P1). State it as a fact: 100/6 for the toy, 10⁶/9,900 = 101.01 for Feller. | Proof; exact partial sums for 5 new parameter sets converge to the closed form, and so does the first-moment corollary. |
| A2 | 07_2 | **error** | "The chance of that…" attaches a probability to N = 1,900. "Almost the whole lake" is also wrong: at N = 1,900 the two catches are *exactly* the whole lake (Feller: "exhaust the entire population"). | "If the lake held only 1,900, our two catches would be every fish in it, and the chance of a catch like ours would be about 5 in 10⁴³⁰." Show ≈5×10⁻⁴³⁰, not "10⁻⁴²⁹". | Recomputed 10^−429.27 = 5.35×10⁻⁴³⁰. |
| A3 | 06_8 | misleading | "9,999 and 10,000 are equally likely" reads as a probability of N. | "…make our catch exactly equally probable: they tie for the highest likelihood." | Ratio is exactly 1. |
| A4 | 02_6 | misleading | "And they're right." 10,000 is an estimate; the truth is unknown (CI 8,400–12,073). | "And that is exactly what careful statistics says too." | — |
| A5 | 06 "exact version"; 06 Ruth | **error** (edge case) | "If Mn/k is an integer, both Mn/k − 1 and Mn/k." This fails when k = M or k = n: then Mn/k = M + n − k and L(Mn/k − 1) = 0. | MLE: none if k = 0; ⌊Mn/k⌋ otherwise; tie with Mn/k − 1 iff Mn/k ∈ ℤ **and** k < min(M, n). | Exhaustive argmax for (M,n,k) = (10,4,4), (4,10,4), (10,10,10): each has a unique maximum. |
| A6 | 08_6 | **error** | Chapman is "exactly unbiased when the two catches together cover the lake". If the catches literally covered the lake, N = M + n − k would be known exactly. The condition in the identity is on sizes, M + n ≥ N, which no real study meets. | "Adding one means it never divides by zero, and its average is almost exactly right once you expect a few recaptures." Lower-third: "exactly unbiased iff M + n ≥ N; bias −1.1% at E[k] ≈ 4." | Identity proved (P2); checked on 4 new cases with exact fractions. |
| A7 | A-F21; 08_6 picture | **error** | "Chapman mean 920" is the seeded simulation. The exact mean is **935.2** (−6.5%), and the run sits 2.75 SE low (SD 545). The "naive mean" next to it is undefined because P(k = 0) = 7.2%. | Show exact 935 (−6.5%). Label the naive figure "among studies with ≥1 recapture: 1,241". | Exact sum; verify_out §7. |
| A8 | A-F16; 07 exact versions; 07_7 screen | **error** (internal) | SE ≈ 949 and CV ≈ √(1/k − 1/n) are the *binomial* delta method, the approximation A-F15 rejects. With the hypergeometric variance, CV² ≈ (1/k − 1/n)(1 − k/M), giving SE 900. The exact SD of Mn/K (given K > 0) is **924**. The binomial CV is not symmetric in M ↔ n, but the problem is (Feller eq. 6.2). So the A-F28 "same k, different split" check would show a spurious gap: 9.75% for (500, 2000) vs 8.94% for (2000, 500), where both are exactly 8.944%. | CV ≈ √(1/k − 1/M − 1/n + k/(Mn)) (P3). Ledger: "SE ≈ 920 (exact SD 924; the textbook 949 ignores the finite lake)". In 07_7 say "the *relative* error shrinks like 1/√k". | Recomputed exact pushforward SDs. |
| A9 | 07_7 words | imprecise | "Not catching a big share of the lake." But k ≈ Mn/N is set by the shares caught. | "What sets precision is how many marked fish you catch *again*, not directly how much of the lake you net." | — |
| A10 | 07_6 | imprecise (quantifier) | The coverage guarantee holds at every N. | Add "whatever the true size is". | Each one-sided test has size ≤ 2.5% at every N. |
| A11 | A-F19; 08 Facts | imprecise | "Seeded sim = exact pushforward": the quantiles agree, but the means are 10,080.0 (sim) and 10,082.5 (exact). | Test means at Monte Carlo tolerance; voice "about ten thousand and eighty". | appendix_out |
| A12 | 04_6 | imprecise | Probabilities shown to 3 decimals sum to 1.001. | Show the ledger's 4 decimals, or mark them "rounded". | Recomputed. |
| A13 | 05_7; §6.4 | suggestion | P(k \| N) conditions on a non-random N. | Use P_N(K = k) or Feller's q_k(N). | — |
| A14 | 05_5 | imprecise (history) | "Introduced by Ronald Fisher." | "Developed and named by Fisher (1912–1922); Daniel Bernoulli, Lambert and Gauss used versions earlier" (Stigler, *Stat. Sci.* 22, 2007). Or attribute it: "as Feller says". | Literature (not in the memo; P1 should cite). |
| A15 | 10_2; §0 | **error** (history) | "Thirty sample districts": the 2,037,615 people lived in sample communes spread over 30 départements (30 whole départements would hold millions). "France registered about a million births": Laplace *assumed* 1,000,000. §0 calls it a "census", which it explicitly was not. | "Officials counted the people in selected towns across thirty départements… Taking France's births as about a million a year…"; in §0, "Laplace's estimate of France's population". | Memo §3; arithmetic. Confirm the commune wording against the *Essai*. |
| A16 | 02_1–02_2 | imprecise (citation) | The narration says 1950, but the quote card is 3rd-ed. (1968) text; the memo records only 3rd-ed. wording. | Check the 1950 text (archive.org) or caption the card "(3rd ed., 1968)". | Memo §1.1 heading. |
| A17 | 07_3; §9.1; §5.1 rule 2; 01_4 | suggestion | "Just as absurd": 1,000,000 is about 10²⁶⁶ times more plausible than 1,900. The takeaway says the catch "carries the same tagged fraction" (it does on average). The hook spread is given four ways: 90×, ≥30×, "tens", and 250× in the draft. | "Also absurd"; "on average, and usually close"; one P1 number for the hook, used everywhere. | — |
| B1 | 02_2 | **error** | "Each column is one if exactly one of them is one" is false for the 3- and 4-term packets in 02_4 (1⊕1⊕1 = 1). | "One if the two bits differ" (two operands), or "one if an odd number are one". | — |
| B2 | 03_5 | imprecise | 29% is the K → ∞ limit; the K = 3 cube on screen gives 32.8%. | "With many pieces, about twenty-nine percent." | rank_probs |
| B3 | 03_3 | imprecise | "The second lights up a line's worth": after two independent packets the span is 4 points, a plane through the origin. | "The second lights two more: itself, and the ex-or of the two." | — |
| B4 | 01_3, 01_5 | imprecise (quantifier) | "About ten thousand" assumes 1% loss, which is never said. 97,876 is a mean. | "If each phone loses one packet in a hundred…"; "on average". | — |
| B5 | 01 Ruth note | **error** | "97,876 = ⌈K·H_K⌉" is wrong: K·H_K = 97,876.06, whose ceiling is 97,877. | "K·H_K = 97,876.06 ≈ 97,876." | Exact rational H₁₀₀₀₀. |
| B6 | 04_6 | **error** (edge) | "Every row is a reshuffle" (Z/7): row 0 is all zeros. | "Every row except zero's." | — |
| B7 | 05_6 | imprecise | "Can't be factored" needs "with zero-one coefficients". The step from "no zero divisors" to "every byte has a partner" uses finiteness. | Add the pigeonhole step (D4). | — |
| B8 | 05_7 code | **error** | `if a & 0x80 { … }` does not compile in Rust (a `u8` is not a `bool`). | `(a << 1) ^ (if a & 0x80 != 0 { 0x1D } else { 0 })` | Rust typing rules. |
| B9 | 07_4 | misleading | "Every extra packet makes failure about 256 times less likely" holds for an extra *uniformly random GF(256) equation* in the model. RaptorQ's extra packets are XOR rows; the RFC promises only 100× per symbol; we measured ≈263× from K to K+1 at K = 10, and K+1 → K+2 cannot be estimated from one failure. | "In this model, each extra random byte equation…" | rank_probs; rq_mc |
| B10 | 07_3 | imprecise | 99.6% is the dense uniform GF(256) model. It holds for **every** K (the product falls from 0.99609 to 0.99608); real RaptorQ measured 99.37%. | Say "with random byte coefficients, in this model." | Recomputed. |
| B11 | §10.1 takeaway | misleading | "Random equations whose coefficients come from a 256-element field": RaptorQ's packets are XOR equations, and only 10–16 HDPC rows use GF(256). "That field exists only because…" is also wrong: the field exists regardless; irreducibility is what lets us *build* it this way. | "Mostly-XOR equations plus a few byte-field checks…"; "we can build that field because…" | Memo §2.1 |
| B12 | 08_1 | imprecise | Dense elimination costs about K³/3 multiply-adds, 4.2×10¹³ at K = 50,000, not "more than a hundred trillion". | "Tens of trillions", or a lower-third. | Arithmetic. |
| B13 | 08_5 | imprecise | e^(−4.82) untouched is pure LT over K pieces. In RaptorQ the LT rows hit W intermediate symbols, and the precode touches all of them. | "With a plain sparse code using the standard's degree table…" | misc_facts |
| B14 | 08_6 | misleading | The line follows "…when you've received exactly enough", but at N = K peeling **collapses** (8–42% recovered). 98–99% needs 2–5% extra packets. | "Even with a few percent extra, peeling alone stalls around ninety-eight or ninety-nine percent; with exactly enough, much earlier." | rerun_lt_peel_sim |
| B15 | 09_6 | **error** | "You come back multiplied by three" mixes up the holonomy (∏aᵢ/bᵢ = 2^±1) with the closing coefficient 1 ⊕ 2 = 3. | "With ex-or, A meets itself and A plus A is zero, so the loop says nothing. With the two, A plus two A is three A in byte arithmetic, and that pins A." See P4. Footnote: 3·5 = 15 only because there are no carries (contrast 3·7 = 9). | Recomputed; proof P4. |
| B16 | 09_7; B-F21 | imprecise | The screen's 0.65% is the ablation's own rank-based baseline (646 failures in 10⁵); the ledger's 0.632% is the cberner decoder (10⁶ trials). "Two thirds of one percent" is 0.667%. | Pair 70.7% with 0.646% (same harness); say "about six in a thousand". | results/*.txt |
| B17 | 01_6; 10_1 | imprecise (K vs K′) | Measured rates are for K = K′ = 10 only (0.631% at 101, 0.54% at 1002). The hook's K = 10,000 pads to K′ = 10,017, and the RFC states its requirement at K′. | "Measured on small blocks; the standard requires it at every size." Mention padding, or use 10,017. | Memo §2.1–2.2 |
| B18 | 10_2 | **error** | "Two extra, about one in a million" is presented as measured, from one failure (exact 95% CI 2.5×10⁻⁸ to 5.6×10⁻⁶). It also contradicts the plan's own Ruth note. | "The standard demands one in a million. In a million trials we saw exactly one failure." | Poisson CI |
| B19 | §0; §5.1 rule 2 | **error** | "Random XOR equations are redundant 71% of the time." | "A random set of exactly K XOR equations fails to determine the K pieces 71% of the time." | rank_probs |
| B20 | 03/07 Ruth notes; 03 exact version | suggestion | Lower-case k for the number of unknowns, while the narration and ledger use K. | Use K throughout. | — |
| C1 | 03_5 | **error** (transcription) | Slates "three, zero, zero, four, two" are served P5, P1, P4, not "P5, P2, P4", and they contradict 03_4's reads (0, 3, 0, 2). The ch. 03 example also uses P5, but §11.3 defines only four process colors. | "Zero, three, zero, four, two" (memo: [0,3,0,0,2] before P4 writes 4). Add a fifth hue or drop P5. | Memo §5 Ch3 |
| C2 | 02_4 | **error** (history) | All four rules are attributed to Dijkstra in 1965. "Everyone who wants in eventually gets in" is Knuth's (1966); Dijkstra's solution was not starvation-free. | "Dijkstra posed it in 1965. A year later Donald Knuth added: everyone who wants in eventually gets in." | Memo §2.3 (Lamport 1986: "first stated and satisfied by Knuth") |
| C3 | 02_6 | misleading (history) | "Lamport asked something harder." Lamport: "I didn't try to devise an algorithm with this property." It also clashes with 08_6. | "Lamport's answer turned out to survive something harder." | Memo §2.2 (verbatim) |
| C4 | 07_2 | **error** | Fact one leaves out "and you're still in the bakery". Counterexample: you finish choosing (5), go in, leave (slate 0); I then choose and take 1. | "If you finished choosing before I started, and you're still waiting or inside when I read your slate, my ticket is bigger." | Paper's Assertion 1; memo §4 |
| C5 | 07_3 → 07_5 | **error** (proof gap) | Fact two is stated for "I'm *waiting*", but 07_5 applies it to two customers who are both *inside*. | "If you're inside, everyone else holding a ticket, waiting or inside, has a bigger pair." Define "in the bakery" at 05_7. | Paper's Assertion 2 |
| C6 | 09_1 | misleading | "Counted from the doorway, not from when you started waiting." Waiting starts exactly when the doorway ends; the real contrast is with arrival. Overlapping doorways carry no order. | "…not from when you walked up. If our doorways overlap, either of us may go first." | Lamport 1986 definition |
| C7 | 09_2 | imprecise | "So nobody starves": FCFS alone does not give that. | "Since someone always gets in, and everyone inside eventually leaves, nobody starves." | Lamport 1986, Thm 1 |
| C8 | 09_4 vs 08 | misleading | The 2³⁵ bound assumes no read returns a value larger than any written. That is exactly what ch. 08's safe memory allows. | Lower-third with Lamport's assumption (§3). | Paper quote, memo §2.1 |
| C9 | 09_5; Dev note | misleading | In "twice the theoretical minimum", N of the 2N cells are **unbounded** integers, and the Burns–Lynch ≥ N bound is for deadlock-free mutual exclusion. An uncontended entry reads 3(N−1) cells (max, L2, L3), not "~2(N−1)". | "2N shared variables, half of them unbounded counters; any read/write lock needs at least N." Reads: 3(N−1). | Counted from the published code. |
| C10 | 10_1; 10_3 | misleading | "It fails" should be "it can fail". "Two per customer is enough" holds for **x86-TSO** (model-checked, N ≤ 3, bounded; Abdulla et al. 2012). Arm, including Apple-silicon laptops, is weaker, and compilers reorder too. | "On x86, two fences per customer are enough. Arm needs its own barriers, and in C, C++ or Rust you need atomics." | Memo §2.3, §3.6 |
| C11 | 07_6 | imprecise | "Every schedule with up to four customers": N = 4 was checked only with tickets ≤ 4 and atomic registers; safe registers only up to N = 3. | "…up to four customers, with small tickets." | sweep_big.txt |
| C12 | 08_7 | imprecise | The double-fetch failure needs non-atomic reads (no violation in the atomic model at N = 3). | Add "when reads can return garbage". | Memo §3.5 |
| C13 | 08_3 | imprecise (quote) | "In his words," followed by a paraphrase. | Verbatim: "The read may return any arbitrary value!" | Memo §2.1 |
| C14 | 01_4–01_5 | misleading | "Four of six" is a count of orders, not a probability (the scheduler is not uniform); "rare" then contradicts it. | "In a real machine the bad orders are rare, but at billions of steps a second, rare means constantly." | — |
| C15 | 02_3; 05_6; 02_5 | suggestion | The schedule counts depend on how steps are counted. "Indivisible" is used both for the read-modify-write instruction and for a single read or write. | Lower-third: "counting each read, write, and successful wait as one step". Say "an instruction that reads *and* writes in one indivisible step". | Reproduced 6/4 and 14/8. |
| C16 | 10_5; 10_6 | suggestion | "That same pair *became* Lamport clocks" overstates lineage. "Of everything he's done" is Levin's framing. | "The same trick reappears…" (CHM p. 9 supports the counter link). Quote "perhaps that's why I'm proudest of it". | Memo §2.2 |

## 2. Proofs

**P1. The likelihood area: Σ_N L(N) = Mn/(k(k−1)) for k ≥ 2, and the sum is infinite for k ≤ 1.**

Here L(N) = C(M,k)·C(N−M, n−k)/C(N, n), summed over N ≥ M+n−k. Put a = n−k and N = M+j with j ≥ a; then N ≥ n automatically. For N ≥ n ≥ 1, the Beta integral gives

  1/C(N,n) = n·B(n, N−n+1) = n ∫₀¹ t^(n−1) (1−t)^(N−n) dt.

All terms are nonnegative, so Tonelli lets us swap sum and integral. Using Σ_{j≥a} C(j,a) x^j = x^a/(1−x)^(a+1) with x = 1−t:

  Σ_N C(N−M, a)/C(N, n) = n ∫₀¹ t^(n−1)(1−t)^(M−n) · (1−t)^a / t^(a+1) dt = n ∫₀¹ t^(k−2)(1−t)^(M−k) dt.

This converges iff k ≥ 2, and then equals n·B(k−1, M−k+1) = n·(k−2)!(M−k)!/(M−1)!. Multiplying by C(M,k) gives **Mn/(k(k−1))**. ∎

For the toy this is 100/6 = 16.67; for Feller, 10⁶/9,900 = 101.01.

Corollary for the Bayesian aside. N/C(N,n) = n/C(N−1, n−1), so the same calculation gives Σ_N N·L(N) = M(M−1)n(n−1)/(k(k−1)(k−2)) for k ≥ 3. With a flat prior, then:
- the posterior is proper iff k ≥ 2;
- its mean is (M−1)(n−1)/(k−2), which is 999²/98 = **10,183.7** for Feller, matching the memo's numerical 10,184.

**P2. Chapman's identity.** Use (n+1)/(k+1)·C(M,k) = C(M+1, k+1) and Vandermonde:

  E[(M+1)(n+1)/(K+1)] = ((n+1)/C(N,n)) · Σ_{j≥1} C(M+1, j)·C(N−M, n+1−j) = ((n+1)/C(N,n)) · [C(N+1, n+1) − C(N−M, n+1)].

Since (n+1)·C(N+1, n+1)/C(N, n) = N+1, subtracting 1 gives E[N̂_C] = N − (N+1)·C(N−M, n+1)/C(N+1, n+1). The correction vanishes iff N−M < n+1, that is, iff M + n ≥ N. ∎

Ledger A-F18 can say "proved", not "verified on 3 cases".

**P3. The symmetric CV.** Var K = n(M/N)(1−M/N)(N−n)/(N−1). By the delta method, CV²(Mn/K) ≈ Var K/(EK)² ≈ (1−M/N)(1−n/N)/(EK). Substituting M/N ≈ k/n and n/N ≈ k/M gives (1/k − 1/n)(1 − k/M) = 1/k − 1/M − 1/n + k/(Mn). This is symmetric in M and n, as it must be, because q_k is symmetric in (M, n) (Feller eq. 6.2).

**P4. Cycles and holonomy.** Take the equations aᵢxᵢ + bᵢxᵢ₊₁ = yᵢ (indices mod m) over a field of characteristic 2. The determinant is ∏aᵢ + ∏bᵢ, so the system is singular iff ∏(bᵢ/aᵢ) = 1. Walking around the loop gives x₁ ↦ h·x₁ + c with h = ∏(aᵢ/bᵢ), and there is a unique fixed point iff h ≠ 1.

In the twisted pentagon (closing row E + 2A), the walk gives E = S + A, so (1 ⊕ 2)A = S ⊕ y₅, i.e. 3A = 15 and A = 5. The holonomy is 2^±1 ≠ 1. The 3 is 1 ⊕ 2, not a "return multiplier". If one coefficient is uniform on the nonzero bytes, P(h = 1) = 1/255.

## 3. Where to add "precisely: …" lower-thirds

1. **A 03_6:** "precisely: E[K] = nM/N for every catch size, by linearity; N̂ = Mn/k sets k equal to its expectation."
2. **A 05_6:** "precisely: Σ_N L(N) = Mn/(k(k−1)) = 100/6; infinite when k ≤ 1 (proved)."
3. **A 06_8:** "precisely: MLE = ⌊Mn/k⌋; tie with Mn/k − 1 iff Mn/k is an integer and k < min(M,n); none if k = 0."
4. **A 07_2:** "precisely: P_N(K = 100) at N = 1,900 is (1000!)²/(100!·1900!) ≈ 5.4 × 10⁻⁴³⁰."
5. **A 07_5:** "precisely: exact test inversion, [8,400, 12,073]; coverage ≥ 95% at every N." Voice "about twelve thousand one hundred", so the contrast with Feller's 12,000 is not lost.
6. **A 07_7:** "precisely: CV ≈ √(1/k − 1/M − 1/n + k/(Mn)) ≈ 9.0%; exact SD of Mn/K is 924."
7. **A 08_6:** "precisely: E[N̂_C] = N − (N+1)C(N−M, n+1)/C(N+1, n+1); zero iff M + n ≥ N; here −6.5%."
8. **B 03_5:** "precisely: P(K random 0/1 rows independent) = ∏ᵢ₌₁ᴷ(1−2⁻ⁱ), which is 0.328 at K = 3 and tends to 0.2888."
9. **B 04_8:** "precisely: a commutative ring in which every nonzero element has an inverse; finite fields exist exactly for prime-power sizes."
10. **B 07_3–07_4:** "model: dense uniform GF(256) coefficients; ∏(1−256⁻ⁱ) ≥ 0.99608 for all K; RaptorQ measured 99.37% (K = 10)."
11. **B 10_2:** "RFC 6330 §5.8: failure ≤ 10⁻², 10⁻⁴, 10⁻⁶ at K′, K′+1, K′+2, averaged over uniformly random ESIs; our 10⁶ trials at K = 10 gave 6,316 / 24 / 1."
12. **C 07_2–07_3:** the Lemma 1 and Lemma 2 statements from memo §4 (with "in the bakery" defined).
13. **C 09_1:** "precisely: if i finishes its doorway before k starts its doorway, i enters first."
14. **C 09_4:** "assuming a read never returns a value larger than one that was written (Lamport 1974)."
15. **C 10_3:** "x86-TSO, model-checked for N ≤ 3 (bounded); Abdulla et al. 2012. Arm needs DMB barriers."

## 4. Delights worth adding (all checked or sourced)

- **D1 (05_6).** Feller's 1950 edition said: "It is, therefore, meaningless to ask for the probability that n is greater than, say, 6000." The sentence was dropped by 1968 (memo §1.1). It pairs naturally with P1, which shows the likelihood's area is 101, not 1.
- **D2 (07).** Marking 2,000 and catching 500 is statistically the same experiment as marking 500 and catching 2,000 (Feller eq. 6.2). The exact CVs are equal (8.944%), and this symmetry is what exposes the asymmetric CV formula in A8.
- **D3 (08).** Chapman's unbiasedness has the two-line proof P2. "Add one" is the absorption identity (n+1)/(k+1)·C(M,k) = C(M+1,k+1), so the trick is combinatorial, not ad hoc. That is a stronger version of 08_7.
- **D4 (05_6).** Pigeonhole: if ab = ac with a ≠ 0, then a(b⊕c) = 0, so b = c. Row a is therefore one-to-one, and with 256 values it is a permutation that must contain a 1. "No zero divisors" plus finiteness is exactly why every byte has a partner, and it ties to the "reshuffle" picture in 04_6.
- **D5 (06).** Every element order in GF(256)* is a power of 2 (1, 3, 5, 15, 17, 51, 85, 255 occur 1, 2, 4, …, 128 times), because 255 = 3·5·17 is a product of Fermat primes (memo §2.4).
- **D6 (10_4).** QR codes use the same polynomial, 285 = 0x11D (ISO/IEC 18004). That is a stronger "you've used this today" than RAID-6, but verify it against the standard before scripting.
- **D7 (03_6).** Erdős proved in 1948 that Σ1/(2ⁿ−1) = 1.6067…, the average number of spare packets, is irrational. Worth checking the citation (J. Indian Math. Soc.) in P1.
- **D8 (02_6 / 08).** The 1974 paper's abstract sells *fault tolerance*, and Lamport's Turing lecture says that in 1973 the problem was "considered impossible… Even in 1990, experts still thought it was impossible" (memo §2.1–2.2). That is a more accurate "huh" than C3's "he asked something harder".
- **D9 (A 10).** Laplace attached odds to his figure: "almost 300,000 against 1" that the error is under half a million. That is an 1802 uncertainty statement, a natural bridge to chapter 07 and the Bayesian aside (memo §3).

My scratch scripts are in `docs/reviews/plan_r1/audit_scripts/`. The audited plan is `docs/planning/COMPREHENSIVE_PLAN_FOR_JEFFS_MATH_EXPLAINER_VIDEOS.md`.
