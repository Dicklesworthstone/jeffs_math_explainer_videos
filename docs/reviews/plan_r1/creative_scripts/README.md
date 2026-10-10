# Creative-direction review: verification scripts

These scripts back the new mathematical claims in `../creative_director.md` (plan review round 1). Both use only the Python 3 standard library. Run them from this folder:

```
python3 -I verify_claims.py   > verify_claims.out.txt    # about 1–2 minutes; the likelihood-area sums dominate
python3 -I progress_paths.py  > progress_paths.out.txt   # instant
```

The `.out.txt` files hold the outputs as recorded. The Monte Carlo uses a fixed seed (`random.seed(1)`), so the outputs are reproducible.

## `verify_claims.py`

The memo's claims and the output lines that back them:

| Memo claim | Output line(s) |
|---|---|
| A9: Σ_N L(N) = Mn/(k(k−1)) | `area …`: partial sums over N < 200,000 against the closed form. The (5,4,2) row reads 9.9994 instead of 10 because its tail converges slowly. The proof is analytic and appears in the memo. |
| A3: Mn − kN = (M−k)(n−k) − k·u | `9999 / 10000 / 10001 …`: both sides are +100, 0 and −100, with u = 8,099, 8,100 and 8,101. `toy u*` gives 49/3 = 16.33, so the toy's MLE is 33. |
| B1: random mod-256 matrices are invertible with the GF(2) probability | `Z/256 invertible frac K=8` (Monte Carlo, 20,000 trials, determinant parity on exact integer determinants) against `GF2 prod`. The theorem itself is analytic: a determinant is a unit in Z/256 iff it is odd. The helper `inv_mod256` is defined but never called. |
| B1: 2·131 ≡ 6 (mod 256) | `2*131 mod 256` |
| B3: union bound q^−h/(q−1) is tight for q = 256 | `q … h … exact fail … bound` |
| B4/B8: GF(256) products (0x11D) | `3*7 9 3*5 15 2*3 6 2*128 29` |
| B5: under rotation (mod x⁸+1), 3·255 = 0 while doubling is a bijection | `3*255 mod x^8+1 = 0 … 256` |
| B2: for q = 2, the risk before the last 10 slots is ≤ 2⁻¹⁰ | `risk before last 10 steps` |
| B2: failure barely depends on K | `K … fail GF2 … GF256` |
| A7: symmetric CV ≈ √((1−M/N)(1−n/N)/k) = 9.0%, consistent with exact SD(k) = 9.000; the 2,000/500 split gives 8.7% | `SD k …`, `split 2000/500 …` |

## `progress_paths.py`

This backs C3: of the 6 interleavings of two `read x; write x+1` programs, 4 lose an update, and those 4 are exactly the staircase paths that pass through the progress-square point (1,1), where both have read and neither has written.

The memo says this was checked "by enumeration". When the memo was written, the count had only been worked out by hand. This script was added when the scripts were copied here, so that the claim is now backed by code.

## Not covered by these scripts

These items are analytic arguments or sourced claims, not computations:
- the stationary-density argument in A6;
- the fixed-N mode formula in A4;
- the Lemma-2 case analysis in C4, which is taken from the bakery research memo §4;
- every historical attribution and quotation (the memo flags these as "verify").
