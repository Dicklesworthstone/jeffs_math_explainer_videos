# Audit scripts: mathematician audit (Ruth, 45), plan review round 1

These are the independent recomputations behind `../ruth_45_audit.md`. Every
script uses only the Python standard library, reads and writes no files, uses
no hard-coded paths and imports nothing local. The saved output of each script
is in `outputs/<script>_out.txt`.

**Running them:** from any directory, `python3 -I -B <path>/<script>.py`. Each one
finishes in a few seconds.

| Script | What it checks | Audit items |
|---|---|---|
| `a_fish.py` | **Toy lake.** C(20,4) = 4,845; the split 1,365 / 2,275 / 1,050 / 150 / 5; probabilities at 4 and 3 decimals (the 3-decimal sum is 1.001).<br>**Ratios.** 529/528 and 576/578; Feller's 80,982,001/80,981,901 → 1 → 81,018,001/81,018,101 (numerator − denominator = Mn − kN); L(33) and the toy MLE.<br>**MLE edge case.** k = M or k = n gives a unique maximum, not a tie.<br>**Feller's lake.** log₁₀ q(1900) and q(10⁶); exact mean and SD of Mn/K (10,082.5; 924); delta-method SE with and without the finite-lake factor (900; 949).<br>**CV.** Binomial vs symmetric CV vs exact CV for the M ↔ n splits.<br>**Small study** (N = 1,000, M = n = 50): P(k = 0); exact Chapman mean 935.2 and the z-score of the simulated 920.<br>**Chapman.** The bias identity on 4 new cases. | A2, A5, A7, A8, A12; P2, P3 |
| `b_area.py` | Partial sums of Σ_N L(N), using exact fractions up to N = 4000, against Mn/(k(k−1)). Also Σ_N N·L(N) against M(M−1)n(n−1)/(k(k−1)(k−2)), the flat-prior posterior-mean corollary.<br>The last two columns test a guess, Σ_N L(N)/N = 1/k. It is **false**, and that result is not used in the audit. | A1; P1 |
| `d_tele.py` | Exploratory, negative result. It looks for a simple telescoping closed form for the tail of Σ_N L(N). The ratio of tail to term is not a simple rational function of N, so P1 uses the Beta-integral proof instead. | P1 (background) |
| `e_gf.py` | **GF(256).** Products mod 0x11D and mod 0x11B (0x57·0x83 = 0x31 vs 0xC1; 3·7, 2·128, 128·3, 150·3, 7·7, 2·142, 3·244, 3·5); OCT_EXP prefix; logs (log 150 = 180, α²⁰⁵ = 167); ord(2) and ord(3) under 0x11B; 30 irreducible and 16 primitive polynomials.<br>**Z/256.** Row facts.<br>**Rank products.** ∏(1−q⁻ⁱ) and failures at K+h for q = 2 and q = 256; the Erdős–Borwein sum.<br>**Other numbers.** K·H_K for K = 10,000 (97,876.06; its ceiling is 97,877); 0.99^(10⁶); e^(−4.8166); 50,000³ and K³/3.<br>**Pentagon.** Values, 1 ⊕ 2 = 3, 3·5 = 15, and the holonomy factor 2⁻¹ = 142; 1⊕1⊕1 = 1. | B1, B2, B5, B10, B12, B15; P4 |
| `f_bakery.py` | Milliseconds per Julian year against 2³⁵; 2⁶⁴ and 2⁶³ at 10⁹ per second in years; C(4,2).<br>Brute-force enumeration of the lost-update schedules (6, of which 4 lose an update) and of the naive flag lock (14, of which 8 put both inside). It counts each read, each write and each successful wait as one step. | C14, C15 |
| `g_ci.py` | Toy exact 95% CI [19, 138] with exact fractions; Chapman 117/4 = 29.25.<br>Feller's tails 0.0347 / 0.0289; the boundary probabilities of the CI endpoints 8,400 and 12,073; P(K = 100 \| N = 10,000).<br>Laplace's 28,352,844.86. | A2, A10 |
