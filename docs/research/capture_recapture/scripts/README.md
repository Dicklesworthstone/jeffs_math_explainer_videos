# Capture–recapture verification scripts

These scripts back every computed number in `../MEMO.md`. Timings below are wall-clock with CPython 3.14.6 on macOS: first alone, then (in brackets) when the three slow scripts ran at the same time during the re-run from this folder.

**Running them:** use `python3 -I -B <path>/<script>.py` from any working directory. No script reads or writes files or uses hard-coded paths, and none imports another.

**Dependencies:**
- Six scripts need only the standard library.
- `verify2.py` and `appendix.py` need **numpy** (tested with 2.5.3), e.g. `pip install numpy` in a virtualenv.

**Saved runs:** each script's output is in `outputs/<script>_out.txt`.

**Relationship to the memo:** the memo's Appendix is a lightly condensed copy of `appendix.py`; the files here are authoritative. The seeded simulations (`numpy.random.default_rng(20261009)`) give the same numbers on every run.

## Scripts

- **`verify.py`** — stdlib only; ≈ 1 min alone [219 s concurrent]. Exact Fractions throughout.
  - Ratio identity L(N)/L(N−1).
  - Feller's tie q(9999) = q(10000) = 0.04429; 10^−429.27; exact tails 0.03468 / 0.02887 and their normal approximations.
  - Likelihood-shape table, half-max range 9,019–11,148, LR interval 8,445–12,021, exact test-inversion interval 8,400–12,073.
  - Lincoln–Petersen / Chapman estimates and SEs (949 / 883); Chapman exact-bias identity.
  - Exact sampling distribution of Mn/k for Feller's lake (8,475 / 10,000 / 12,048; mean +0.82% given k > 0) and for N = 1,000, M = n = 50 (+24%); toy-lake tables. The N = 1,000, M = n = 100 case (625 / 1,000 / 2,000; +10.2%) is in `verify2.py` and `appendix.py`.
  - Reproduces memo §1.2, §2.1–§2.5, §2.6 (exact part).
- **`verify2.py`** — numpy; ≈ 13 s alone [47 s concurrent].
  - Area under the likelihood, 101.0101.
  - Bayesian posteriors: flat prior 8,536–12,186; 1/N prior 8,475–12,073.
  - Sensitivity to k; toy 10/10/3 (MLE 33, exact interval 19–138, LR 19–104, Chapman 29.25).
  - Seeded Monte Carlo table.
  - Assumption-violation simulations (7,752 / 7,143 / 14,493 / 12,500 / 12,122 / 10,000 / 6,250 / 25,000).
  - History arithmetic: Laplace 28,352,845; Graunt 384,000; Lincoln 41.7 M; Rodli tarn 338; tank 74; two-reviewer 60.
  - Reproduces memo §2.2, §2.5, §2.6, §2.7, §3, §4.
- **`appendix.py`** — numpy; ≈ 2 min alone [171 s concurrent]. The consolidated check printed in the memo Appendix. It re-derives the headline numbers of §0, §1.2 and §2.
- **`area.py`** — stdlib; ≈ 6 s. Numerically checks Σ_N L(N) = Mn/(k(k−1)) for five parameter sets (memo §2.2).
- **`rr.py`** — stdlib; < 1 s. Exact Chapman bias versus expected recaptures: −1.1% / −1.6% at E[k] ≈ 4, −6.5% at 2.5 (memo §2.4).
- **`sadinle.py`** — stdlib; < 1 s. Sadinle transformed-logit interval: 8,392–11,927 for Feller, 20–91 for the toy (memo §2.5).
- **`normal_approx_check.py`** — stdlib; < 1 s. Binomial–normal tails 0.042 / 0.046 and 0.028 / 0.032 that explain Feller's "0.04" and "0.03", plus 93.6% coverage (memo §0, §1.2).
- **`ratio_numbers.py`** — stdlib; < 1 s. On-screen seesaw numbers: 80,982,001 / 80,981,901 → 81,000,000 / 81,000,000 → 81,018,001 / 81,018,101; toy 529/528 and 576/578 (memo §2.2).

Downloaded third-party material (Feller scans and OCR text, papers, reports, web pages) is deliberately **not** kept here.
