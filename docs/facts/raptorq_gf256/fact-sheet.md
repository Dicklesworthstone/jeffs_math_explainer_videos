# Video B fact sheet: The Arithmetic Where 3 × 7 = 9

This sheet covers bead `mx-e-b-qyy.2`. It checks every row B-F1 … B-F28 of the facts ledger in
`docs/planning/COMPREHENSIVE_PLAN_FOR_JEFFS_MATH_EXPLAINER_VIDEOS.md` §10.5 against
`oracle.py` in this directory. It also records how each of the bead's open items (to verify) was resolved.

**Run (from the repo root):** `python3 -I docs/facts/raptorq_gf256/oracle.py`

The oracle needs only the standard library and numpy (2.4.1 was used). It prints one `KEY<TAB>VALUE` line per fact
and writes a summary line to stderr. Final run (2026-10-10, CPython 3.14.6, this Mac): **297 keys, real 3.54 s** (`/usr/bin/time -p`). Four runs
took 3.5–4.6 s. The output is byte-identical from run to run.

## How to read this sheet

- **MATCH** means the oracle's value agrees with the ledger at the precision the ledger states.
  **MISMATCH** would mean it does not. **No ledger row is a MISMATCH.** Some plan prose outside the
  ledger does not agree with the oracle; see [Discrepancies outside the ledger](#discrepancies-outside-the-ledger).
- **RECORDED** values are Monte Carlo counts that the oracle does not recompute. Each one is pinned in `oracle.py`
  with the output file it came from. On every run the oracle re-reads that file and prints a
  `….provenance_check` key: `match`, `MISMATCH:…` or `file-missing`. All of them currently print `match`.
- **SOURCED** values are quoted from a standard. They cannot be computed.
- All other values are computed. Integers are exact. Probabilities come from exact `Fraction` products and sums, or from
  50-digit `Decimal` where noted. Each truncated infinite product or sum has its error bounded in a comment; every
  bound is below 1e−30.
- `oracle.py` copies and adapts its functions from `docs/research/raptorq_gf256/scripts/` instead of importing them,
  so it has no import side effects and never writes `__pycache__` outside this directory. The RFC 6330 tables (Table 1,
  Table 2, OCT_EXP, OCT_LOG) are transcribed into `oracle.py` from `https://www.rfc-editor.org/rfc/rfc6330.txt`
  (sha256 `87f6da89…21ace`, fetched 2026-10-10). The parser that extracted them was written for this oracle and is not
  `rfc6330_data.py`.
- Keys ending in `_ok` are internal checks. Most compare two independent routes to the same value. Some are
  structural, for example primality of S and W, monotonicity, or the transcription hash. `1` means the check passed.
  All 22 of them print `1`.

## Ledger rows

| id | quantity | oracle value | plan value (§10.5) | match | how verified |
|---|---|---|---|---|---|
| B-F1 | sticker album, K = 10,000 | K·H_K = **97,876.06036** (`B-F1.K_times_H_K`); H_K = 9.7876 | K·H_K = 97,876.06 on average (9.79×) | MATCH | `bf1`: exact `Fraction` sum Σ1/i, i ≤ 10⁴. Agrees with `misc_facts.py` (rerun: 97876.1) and `integration_check.py` §6 (97876.06). Memo §2.5. |
| B-F2 | misses per packet at 1% loss, 10⁶ phones | expected **10,000** exactly (SD 99.5); ln P(packet reaches all) = −10,050.34 | ≈10,000 | MATCH | `bf2`: n·p exact; binomial SD. `misc_facts.py` (e^−10050.34). Memo §2.6. |
| B-F3 | XOR example | A⊕C = **2**, B⊕C⊕D = **6**, A⊕B = **6**, A⊕B⊕C⊕D = **3**; GF(2) rank **4**; B⊕C = **4** = card 1 ⊕ card 3; rank stays 4 with it as a fifth card; D = 6 ⊕ 4 = 2 | 2, 6, 6, 3; rank 4; B⊕C = 4 redundant | MATCH | `bf3`, `rank_gf2` (own elimination). `toy_examples.py` (rerun identical). Memo §3 Ch2. |
| B-F4 | GF(2) success, K×K | K = 1, 2, 3: **1/2, 3/8, 21/64 = 0.328125** (= 7/8·6/8·4/8); limit **0.2887880951** | 0.5, 0.375, 0.328 (K=3: 7/8·6/8·4/8) → 0.288788 | MATCH | `bf4`, `p_full_finite`, `p_full_limit`: exact ∏(1−2⁻ⁱ). `rank_probs.py`. Memo §2.3. |
| B-F5 | GF(2) failure at K, +1, +2, +3 | **0.711212, 0.422424, 0.229898, 0.119884** (K → ∞); at K = 10, h = 0: 0.710930 | 71.12%, 42.24%, 22.99%, 11.99% | MATCH | `bf5`: 1 − ∏_{i>h}(1−2⁻ⁱ). These are K → ∞ limits, as in `rank_probs.py`. Memo §2.3. |
| B-F6 | average spares | Σ1/(2ⁱ−1) = **1.6066951524**; Σ1/(256ⁱ−1) = **0.0039368875** | 1.6067 (GF(2)); 0.00394 (GF(256)) | MATCH | `bf6`: three routes agree to < 1e−15: Σ1/(qⁱ−1); Σ_h P(fail at K+h); Erdős's Lambert form Σd(r)/qʳ. `rank_probs.py`, `lt_peel_sim.py`. Memo §2.3. |
| B-F7 | GF(256) success (model) | K = 1: 0.99609375; limit **0.9960784912**; the sequence decreases strictly in K, so the limit is the minimum | ≥ 0.996078 for every K | MATCH | `bf7`. `rank_probs.py`. Memo §2.3. |
| B-F8 | GF(256) failure (model) at K, +1, +2 | **3.921509×10⁻³; 1.531863×10⁻⁵; 5.983839×10⁻⁸** (K → ∞) | 3.92×10⁻³; 1.53×10⁻⁵; 5.98×10⁻⁸ | MATCH | `bf8`. `rank_probs.py`. Memo §2.3. |
| B-F9 | pentagon | rank **4** over GF(2), and 4 over GF(256) for the same 0/1 matrix; with E + 2A, rank **5**; cards 6, 4, 5, 14, 6; S = 9; y₅ ⊕ S = **15 = 3A** (3 = 1 ⊕ 2); A = 15/3 = **5** | rank 4 over GF(2); with E + 2A rank 5; 3A = 15 ⇒ A = 5 | MATCH | `bf9`: rank by two implementations, pure-Python `rank_gf256_py` and numpy `rank256_np` (adapted from `gf256np.py`); they agree. `toy_examples.py`, `e_gf.py`. Memo §2.5. |
| B-F10 | Z/256 facts | 2·128 ≡ **0**; 2·131 ≡ **6** (= 2·3); 128 units, **127** nonzero non-units; for all 256 values of a, row a has 256/gcd(a,256) values (row 2: 128; row 128: 2) | 2·128 ≡ 0; 2·131 ≡ 6; 127 nonzero non-units; row a has 256/gcd(a,256) values | MATCH | `bf10`, exhaustive. `gf256_facts.py`, `integration_check.py` §7. Memo §2.4. |
| B-F11 | GF(256) products (0x11D) | 3·7 = **9**; 2·128 = **29**; 128·3 = **157**; 128·4 = **58**; 150·3 = **167**; 7·7 = **21**; 0x57·0x83 = **0x31** (49), under AES's 0x11B **0xC1** (193); 2⁻¹ = **142**; 3⁻¹ = **244** | same list | MATCH | `bf11`: shift-and-add plus reduction. For all 65,536 pairs this agrees with the RFC's OCT_EXP/OCT_LOG tables and with the plan's Rust `mul`/`xtime` code panel (05). `gf256_facts.py`. Memo §2.4. |
| B-F12 | polynomial counts | **30** irreducible, **16** primitive; 0x11D primitive; 0x11B irreducible but not primitive; under 0x11B, ord 2 = **51** and ord 3 = **255** | 30 irreducible, 16 primitive; 0x11D primitive; 0x11B not (ord 2 = 51, ord 3 = 255) | MATCH | `bf12`: trial division over all 256 monic degree-8 polynomials. Agrees with (2⁸−2⁴)/8 and φ(255)/8. Element orders 1, 3, 5, 15, 17, 51, 85, 255 occur 1, 2, 4, …, 128 times. `gf256_facts.py`. Memo §2.4. |
| B-F13 | logs (base 2, 0x11D) | log 3 = **25**, log 29 = **8**, log 128 = **7**, log 150 = **180**; 180 + 25 = 205; α²⁰⁵ = **167**; also 7 + 2 = 9 → 58 | log 3 = 25, log 29 = 8, log 128 = 7, log 150 = 180; 180 + 25 = 205 → 167 | MATCH | `bf13`. The log-ordered table is Hankel (all 255² entries checked). `gf256_facts.py`. Memo §2.4. |
| B-F14 | RFC tables | OCT_EXP[0..12] = **1, 2, 4, 8, 16, 32, 64, 128, 29, 58, 116, 232, 205**. All 510 OCT_EXP and all 255 OCT_LOG entries equal the powers and logs of x mod 0x11D | OCT_EXP 1, 2, 4, …, 128, 29, 58, 116, 232, 205; = powers of x mod 0x11D | MATCH | `bf14`, against the tables transcribed from RFC 6330 §5.7.3–5.7.4. `gf256_facts.py --rfc` (rerun: True). Memo §2.1. |
| B-F15 | dense elimination | 50,000³ = 1.25×10¹⁴; ÷3 = **4.1667×10¹³** | ≈K³/3 = 4.2×10¹³ for K = 50,000 | MATCH | `bf15`, exact. `misc_facts.py`, `e_gf.py`. |
| B-F16 | RFC degree table (Table 1) | P(1) = 5243/2²⁰ = **0.50001%**; P(2) = **1/2** exactly; for d = 2…29, every mass is within 0.81 units of 2⁻²⁰ of 1/(d(d−1)) (max relative deviation 5.0×10⁻⁴); P(30) = 2.948% (the tail); mean degree **4.816642**; **7.152** symbols per repair | P(2) = 1/2, P(1) ≈ 0.5%, ideal soliton for d = 2…29, mean 4.817; ≈7.15 symbols per repair | MATCH | `bf16` on RFC §5.3.5.2 Table 1. The 7.15 is the mean degree plus E[d1] = 2.3358 (RFC §5.3.5.4: d1 = 2 + Rand[X,3,2] if d < 4). It treats Rand[X,3,2] as uniform on {0,1}. `misc_facts.py`. Memo §2.1. |
| B-F17 | untouched pieces, plain sparse code, N = K | e^(−4.816642) = **0.8094%**; finite K = 10,000: (1−4.8166/K)^K = 0.8085% | e^(−4.82) ≈ 0.81% | MATCH | `bf17`. Memo §2.5. |
| B-F18 | peeling (pure LT, RFC degrees, K = 10,000, 5 seeded runs) | N = K: 0.0924, 0.421, 0.2249, 0.1735, 0.0825 (**8.25–42.1%**). +2%: 0.9824, 0.9871, 0.9853, 0.9884, **0.1578** (1 collapsed; others **98.24–98.84%**). +5%: **98.65–99.03%** | N = K collapses (8–42% recovered); 2–5% extra → ≈98–99%; 1 of 5 runs collapsed at 2% | MATCH | `bf18` re-runs the seeded simulation of `lt_peel_sim.py` (`random.Random(42)`, same call order). Its output is identical to `results/rerun_lt_peel_sim.txt`. Memo §2.5. |
| B-F19 | RFC parameters (Table 2) | **477** rows; H ∈ **[10, 16]**; K′ = 10,017: S **241**, H **11**, W **10,111**, L **10,269**, P **158**; K′_max **56,403** (H 16, L 57,326); K′ = 10: L = 27 | H ∈ [10, 16] over 477 K′; K′ = 10,017: S 241, H 11, W 10,111, L 10,269, P 158; K′_max 56,403 | MATCH | `bf19` on Table 2, transcribed from RFC 6330 §5.6. As the RFC states, S and W are prime in every row. Agrees with the separate parse in `results/rerun_misc_facts.txt` on all 15 rows printed there. |
| B-F20 | inactivation toy | peel order **C, A, B, D, E** (ripple 1, 1, 1, 2, 2); without y1 nothing peels; c = 15 ⊕ 14 ⊕ 6 = **7**, the only one of 256 values consistent with y6; then A, B, D, E = 5, 3, 2, 12 | peel C, A, B, D, E; c = 15 ⊕ 14 ⊕ 6 = 7 | MATCH | `bf20`. `toy_examples.py`. Memo §2.5. |
| B-F21 | ablation (same harness), K′ = 10 and 101 | **RECORDED**: RFC rows 646 / 100,000 = **0.646%**; GF(2) rows 7,066 / 10,000 = **70.66%** (K′ = 10) and 702 / 1,000 = **70.2%** (K′ = 101). Also recorded: K′ = 10, GF(2) at +1/+2: 42.69% / 22.93%; K′ = 101: 41.6% / 24.4%; random GF(256) rows 137 / 20,000 = 0.685% | RFC rows 646 / 10⁵ = 0.646% (K′ = 10); GF(2) rows 7,066 / 10⁴ = 70.7% (K′ = 10), 702 / 10³ = 70.2% (K′ = 101) | MATCH (recorded) | `bf21`, `RECORDED_ABLATION`. Sources: `results/original_rq_ablate_k10_gf256_0_1e5trials.txt`, `…_k10_gf2random_{0,1,2}.txt`, `…_k101_gf2random_{0,1,2}.txt`, `…_k10_gf256random_0.txt` (all under `docs/research/raptorq_gf256/scripts/`). Provenance check: match. Not re-run: `rq_ablate.py` needs the RFC text; the recorded files report 84–143 s each (the 10⁵-trial file reports no time). Memo §2.2. |
| B-F22 | measured RaptorQ, K = 10, 10⁶ trials, cberner decoder | **RECORDED**: **6,316 / 24 / 1** failures at K, K+1, K+2. Exact 95% intervals: 0.616–0.647%; 1.54–3.57×10⁻⁵; **2.53×10⁻⁸–5.57×10⁻⁶**. Also recorded: K = 101: 1,262 / 2×10⁵ (0.631%), 4 / 2×10⁵; K = 1,002: 108 / 2×10⁴ (0.54%), 0 / 2×10⁴ | 6,316 / 24 / 1 failures at K, K+1, K+2 | MATCH (recorded) | `bf22`, `RECORDED_MC`. Sources: `results/original_rq_mc_K10_1e6trials.txt`, `…_K101_2e5trials_partial.txt`, `…_K1002_2e4trials_partial.txt`. Provenance check: match. Not re-run: `rq_mc.py` needs PyPI `raptorq==2.0.0`, and the K = 10 run took 869 s. The Clopper–Pearson intervals are computed by `clopper_pearson` and match the plan's 10_x note (2.5×10⁻⁸–5.6×10⁻⁶). Memo §2.2. |
| B-F23 | RFC 6330 §5.8 requirement | **SOURCED**: at most **1/100, 1/10⁴, 1/10⁶** at K′, K′+1, K′+2 | ≤ 1/100, ≤ 1/10⁴, ≤ 1/10⁶ at K′, K′+1, K′+2 (average over random ESIs) | MATCH (sourced) | RFC 6330 §5.8, pp. 65–66. Quoted under [Primary-source quotes](#primary-source-quotes). `bf23` also reports that the recorded K = 10 point estimates are within the requirement (1, 1, 1). The 95% upper bounds are not (1, 1, **0**): one failure in 10⁶ cannot confirm the 10⁻⁶ requirement. |
| B-F24 | carousel, 1% loss, K = 10,000 | mean **2.642** loops per phone; all 10⁶ phones done by 5 loops: **0.3679**; by 6: **0.9900**. Per phone: done by 2 loops 0.368, by 3 loops 0.990 | mean 2.64 loops; all 10⁶ phones done: 0.37 by 5 loops, 0.99 by 6 | MATCH | `bf24`: exact formula P(done by m) = (1 − 0.01^m)^(K·phones) and E[T] = Σ_m P(T > m), at 50-digit `Decimal`. Model: each phone loses each packet independently with probability 1%. `integration_check.py` §6. |
| B-F25 | Z/256 lottery | MC (K = 8, 40,000 trials): **11,413 / 40,000 = 0.2853** vs ∏(1−2⁻ⁱ) = **0.2899**. Certificate on the first 4,000 samples: exact integer-determinant parity equals mod-2 invertibility in 4,000/4,000. For all 1,168 odd-determinant samples, an explicit inverse mod 256 was built and checked | invertible ⇔ det odd ⇔ invertible mod 2 ⇒ success = ∏(1−2⁻ⁱ); MC K = 8: 0.285 vs 0.290 | MATCH | `bf25` repeats `integration_check.py` §7 (seed 1, same draw order) and reproduces its 0.2853. The theorem is standard: an integer matrix is invertible mod 2⁸ iff its determinant is a unit, i.e. odd. `verify_claims.py` (B1). |
| B-F26 | rotation fold | 3·255 mod (x⁸+1) = **0**: the carry-less product is 0x101 = x⁸+1 = (x+1)⁸. Doubling is still a bijection (256 values) | 3·255 = 0 mod x⁸+1 | MATCH | `bf26`. `verify_claims.py` (B5). |
| B-F27 | union bound 256^−h/255 vs exact | agree to **4, 8, 9** significant digits at h = 0, 1, 2 (relative gaps 1.5×10⁻⁵, 6.0×10⁻⁸, 2.3×10⁻¹⁰) | matches exact to 3 digits | MATCH (the ledger understates it) | `bf27`. `verify_claims.py` (B3). Memo §2.3. |
| B-F28 | GF(2) spares for 10⁻⁶ | the smallest h with failure ≤ 10⁻⁶ is **20**; failure(20) = **9.5367×10⁻⁷**, failure(19) = 1.907×10⁻⁶ | ≈20 (failure ≈ 2^−h; h = 20: 9.5×10⁻⁷) | MATCH | `bf28`. `integration_check.py` §6 (9.54e−07). |

### Extra rows for the open items (not in the ledger)

| id | quantity | oracle value | plan claim | match | how verified |
|---|---|---|---|---|---|
| B-X1 | RAID-6 weights | 2ⁱ for i = 0…254 gives **255** distinct weights under 0x11D and **51** under 0x11B (2⁵¹ = 1 there). {02}⁸ = **0x1D**. The Horner loop of Linux `int.uc` (re-implemented byte by byte, not the kernel's SWAR code) equals Σ2ⁱ·Dᵢ on 200 random stripes | "Q = Σ 2ⁱ·Dᵢ in this exact field; with generator 2 under AES's polynomial, only 51 distinct weights exist" (§10.4 06) | MATCH (wording caveat below) | `bx1`. Anvin's paper and the Linux source are quoted below. |
| B-X2 | QR-code field polynomial | 100011101₂ = **285** = 0x11D | "QR codes … same polynomial (285)" (§10.4 10) | MATCH | `bx2`. ISO/IEC 18004:2006 §6.5.2 is quoted below. |

## Open items from the bead

### 1. What Erdős proved in 1948: VERIFIED

The plan (03, Ruth note) says Σ1/(2ⁱ−1) = 1.6067 is "the Erdős–Borwein constant (irrational, Erdős 1948)".

- **Primary source (scan of the paper):** P. Erdős, "On arithmetical properties of Lambert series", printed pp. 63–66,
  "[Received 8 July, 1948.]", at https://static.renyi.hu/~p_erdos/1948-04.pdf. The paper defines
  f(x) = Σ xⁿ/(1−xⁿ) and states: *"THEOREM. Let |t| > 1 be any integer. Then both f(1/t) and g(1/t) are irrational."*
  For t = 2, f(1/2) = Σ 1/(2ⁿ−1), which is the average-spares constant of B-F6. The proof writes f(1/t) as
  Σ d(r)/tʳ and shows that its base-t expansion has arbitrarily long runs of zeros. `bf6` checks this identity
  numerically.
- **Journal citation:** from Wikipedia, https://en.wikipedia.org/wiki/Erdős–Borwein_constant. It cites
  *"On arithmetical properties of Lambert series" (PDF), J. Indian Math. Soc., New Series, 12: 63–66* and states:
  *"In 1948, Erdős showed that the constant E is an irrational number."* The scanned pages do not print the journal
  name. The journal and volume therefore come from this secondary source; the page numbers 63–66 can be read on the scan.
- **A bonus fact from the same theorem (t = 256):** the GF(256) average spares, Σ1/(256ⁿ−1) = 0.003937, is also irrational.
- **Narration-safe wording:** "Erdős proved in 1948 that this number is irrational."

### 2. RAID-6 (Q = Σ 2ⁱ·Dᵢ in 0x11D; 51 weights under 0x11B): VERIFIED, with a wording caveat

- **Anvin, "The mathematics of RAID-6"** (https://www.kernel.org/pub/linux/kernel/people/hpa/raid6.latex, also
  `raid6.pdf`; "First version 20 January 2004, Last updated 20 December 2011"):
  - *"Hardware engineers will recognize this as a linear feedback shift register (LFSR), and matematicians as boolean
    polynomial multiplication modulo the irreducible polynomial x^8 + x^4 + x^3 + x^2 + 1."* ("matematicians" is
    Anvin's spelling.)
  - *"Q = g^0 · D_0 + g^1 · D_1 + g^2 · D_2 + ... + g^{n-1} · D_{n-1} … where g is any generator of the field (we use
    g = {02}.)"* (LaTeX rendered as text)
  - *"(Note, however: {02}^8 = {1d}.)"*
  - Footnote: *"{02} is not a generator of the AES representation of GF(2^8), and as such it is not suitable."*
- **The Linux kernel source agrees:** https://github.com/torvalds/linux/blob/master/lib/raid/raid6/mktables.c, line 22,
  `a = (a << 1) ^ (a & 0x80 ? 0x1d : 0);`. `lib/raid/raid6/int.uc` builds Q in the syndrome loop with
  `w2$$ &= NBYTES(0x1d);`, which is Horner's rule for Σ2ᶻ·D_z. The latest commit touching `mktables.c` was
  `30bf04bd13a5`, 2026-05-29. The directory is `lib/raid/raid6/`; older trees used `lib/raid6/`.
- **The 51 is computed, not quoted.** Under 0x11B the powers 2⁰…2²⁵⁴ take only 51 distinct values
  (`B-X1.raid6_distinct_weights_2^i_i0_254_under_0x11B` = 51; `B-F12.ord2_under_0x11B` = 51).
- **Wording caveat:** the plan says "with generator 2 under AES's polynomial". Under 0x11B, 2 is *not* a generator,
  which is exactly Anvin's footnote. A correct wording: "with 2 as the multiplier under AES's polynomial, only 51
  distinct weights exist."

### 3. QR codes' polynomial (285 = 0x11D): VERIFIED against the standard's own text (2006 edition)

- **ISO/IEC 18004:2006** (second edition, "QR Code 2005 bar code symbology specification"), clause 6.5.2 "Generating the
  error correction codewords". The clause starts on printed p. 44; the sentence is at the top of p. 45:
  *"The polynomial arithmetic for QR Code 2005 shall be calculated using bit-wise modulo 2 arithmetic and byte-wise
  modulo 100011101 arithmetic. This is a Galois field of 2^8 with 100011101 representing the field's prime modulus
  polynomial x^8+x^4+x^3+x^2+1."*
- **Copy used:** https://law.resource.org/pub/us/cfr/ibr/inc/iec/iec.18004.2006.pdf, PDF page 53. It is
  Public.Resource.Org's copy of the standard as incorporated by reference in the US CFR, stamped "Edict of government".
  This is the standard's text, not a secondary source.
- `bx2`: 100011101₂ = 285 = 0x11D.
- **Limit:** only the 2006 edition was checked. The current edition, the fourth (2024), is paywalled and was not checked.

### 4. RaptorQ vs Raptor in 3GPP TS 26.346 (MBMS): VERIFIED. RaptorQ is not in TS 26.346; keep "older sibling"

- **TS 26.346 V19.3.0** (ETSI TS 126 346 V19.3.0, 2026-02), at
  https://www.etsi.org/deliver/etsi_ts/126300_126399/126346/19.03.00_60/ts_126346v190300p.pdf. The full text layer
  (110 pages) contains **no** occurrence of "6330" or "RaptorQ". Clause 7.2.12.1 says: *"This clause defines an FEC
  encoding scheme for the MBMS forward error correction code defined in RFC 5053 [91] for the download delivery
  method."* Clause 7.2.2 says: *"A UE that supports MBMS User Services shall support a decoder for the Raptor FEC scheme."*
- **TR 26.947 V19.0.0** (ETSI TR 126 947, 2025-11), "Selection and characterisation of application layer FEC", clause 10
  Conclusions, at https://www.etsi.org/deliver/etsi_tr/126900_126999/126947/19.00.00_60/tr_126947v190000p.pdf:
  *"6330 and RS+LDPC each received significant support as MBMS Application Layer FEC (AL-FEC). As a final conclusion
  of the work, no new AL-FEC code was adopted."* "6330" is the code of RFC 6330, i.e. RaptorQ.
- **The 2005 choice of Raptor:** SP-050246, TSG SA Meeting #28, Quebec, 6–9 June 2005, at
  https://www.3gpp.org/FTP/tsg_sa/TSG_SA/TSGS_28/Docs/PDF/SP-050246.pdf: *"Based on the above, SA4 has agreed to
  select Raptor codes as the FEC code for MBMS file delivery and streaming in Release 6."*
- **Conclusion:** the script's "Its older sibling was chosen for mobile broadcast in 2005" is correct. Do not say RaptorQ
  is in MBMS.
- **Not checked:** 5G broadcast specifications other than TS 26.346.

### 5. Intel GFNI's polynomial: NOT USED by the plan's script (verified anyway, from a secondary source)

The plan's Video B script (§10.4) never mentions GFNI. It appears only as an optional suggestion in
`docs/reviews/plan_r1/dev_28.md` (B-18: "Optional 'huh' (verify first): Intel's GFNI multiply instruction is
hard-wired to AES's polynomial").

For the record, https://www.felixcloutier.com/x86/gf2p8mulb says of GF2P8MULB: *"The field GF(2^8) is represented in
polynomial representation with the reduction polynomial x^8 + x^4 + x^3 + x + 1."* That is 0x11B. The site describes
itself as an unofficial extract of the Intel SDM, so the official manual was not checked.

## Primary-source quotes

**RFC 6330 §5.8** (https://www.rfc-editor.org/rfc/rfc6330.txt, pp. 65–66), backing B-F23:

> A RaptorQ-compliant decoder SHALL have the following recovery properties for source blocks with K' source symbols
> for all values of K' in Table 2 of Section 5.6.
> 1. If the decoder receives K' encoding symbols generated according to the encoder specification in Section 5.3 with
> corresponding ESIs chosen independently and uniformly at random from the range of possible ESIs, then on average
> the decoder will fail to recover the entire source block at most 1 out of 100 times.
> 2. If the decoder receives K'+1 encoding symbols … at most 1 out of 10,000 times.
> 3. If the decoder receives K'+2 encoding symbols … at most 1 out of 1,000,000 times.

**RFC 6330 §5.6**, backing the B-F19 primality check: *"For each value of K', the corresponding values of S(K') and
W(K') are prime numbers."*

## Recorded values (not recomputed)

| ledger row | pinned constant(s) | file under `docs/research/raptorq_gf256/scripts/` | produced by |
|---|---|---|---|
| B-F21 | 646 / 100,000 (K′ = 10, RFC HDPC rows, h = 0) | `results/original_rq_ablate_k10_gf256_0_1e5trials.txt` | `rq_ablate.py` |
| B-F21 | 7,066 / 4,269 / 2,293 of 10,000 (K′ = 10, GF(2) rows, h = 0, 1, 2) | `results/original_rq_ablate_k10_gf2random_{0,1,2}.txt` | `rq_ablate.py` |
| B-F21 | 702 / 416 / 244 of 1,000 (K′ = 101, GF(2) rows, h = 0, 1, 2) | `results/original_rq_ablate_k101_gf2random_{0,1,2}.txt` | `rq_ablate.py` |
| B-F21 | 137 / 20,000 (K′ = 10, random GF(256) rows) | `results/original_rq_ablate_k10_gf256random_0.txt` | `rq_ablate.py` |
| B-F22 | 6,316 / 24 / 1 of 10⁶ (K = 10) | `results/original_rq_mc_K10_1e6trials.txt` | `rq_mc.py` + PyPI `raptorq==2.0.0` |
| B-F22 | 1,262 / 4 of 2×10⁵ (K = 101) | `results/original_rq_mc_K101_2e5trials_partial.txt` | `rq_mc.py` |
| B-F22 | 108 / 0 of 2×10⁴ (K = 1,002) | `results/original_rq_mc_K1002_2e4trials_partial.txt` | `rq_mc.py` |

### Caveats on B-F22

- `rq_mc.py` draws K+h packets from one fixed pool returned by `get_encoded_packets(20000)`. The memo §2.2 says the
  draws are "drawn mostly from repair symbols". RFC 6330 §5.8 instead specifies ESIs drawn uniformly from the whole
  ESI range. The two set-ups are close but not identical.
- `rq_ablate.py` does draw ESIs uniformly from 2²⁴.

## Discrepancies outside the ledger

None of these is a ledger row. The ledger values are all correct. These are statements in the plan's chapter text
that the oracle's computations contradict or qualify.

1. **§10.4 06, Ruth note (plan line 1763):** "element orders are all powers of 2 because 255 = 3·5·17". This is
   **false as written.** The orders are 1, 3, 5, 15, 17, 51, 85, 255, all odd. What is a power of 2 is the
   *number of elements* of each order: 1, 2, 4, 8, 16, 32, 64, 128 (`B-F12.element_order_counts_0x11D`), because each
   φ(d) is a power of 2. The memo (§2.4) states it correctly.
2. **§10.4 07, "Precisely" lower-third (plan line 1782):** "exact values 0.39216% / 1.5319×10⁻⁵ / 5.984×10⁻⁸".
   The exact h = 0 failure is 0.0039215088 = **0.39215%**. 0.39216% is the union bound 1/255. The other two figures
   are correct exact values. Suggested fix: "0.39215%", or "0.392%".
3. **§10.4 06, "For programmers" (plan line 1768):** "with generator 2 under AES's polynomial". 2 is not a generator
   under 0x11B; see open item 2. The count 51 is right.
4. **§10.4 04_1 (was 03_7 before the G2 split):** "Two spares still fail one time in four … a quarter of a million stuck". The exact
   value is 22.99%, i.e. 229,898 of 10⁶ (`B-F28.phones_stuck_of_1e6_at_h2`). That is about 1 in 4.35. Acceptable as
   rounding; "about one in four" or "nearly a quarter" would be more exact. Applied at the G2 pass: "nearly one time in four", "almost a quarter of a million".
5. **§10.4 11_3 (was 10_3):** "for every block size". The RFC says "for all values of K′ in Table 2". The Ruth note under 11
   already covers this (the requirement applies at the padded K′).

## Reproducibility notes for the Rust `facts` binary (gate G1)

- **B-F18 and B-F25 (Monte Carlo part) depend on CPython's RNG.** They use `random.Random(42)` and
  `random.Random(1)`: Mersenne Twister plus CPython's `randrange`/`sample` algorithms. The output was verified identical
  to the research reruns on CPython 3.14.6. A Rust binary cannot match these key by key unless it re-implements
  CPython's generator and sampling. Either pin them as constants, as with the recorded rows, or exclude them from the
  exact comparison.
- **B-F14, B-F16 and B-F19 depend on the RFC tables transcribed into `oracle.py`.** The transcription is guarded in
  three ways:
  - a sha256 of Table 2 (`B-F19.table2_transcription_hash_ok`);
  - the RFC's own primality statement for S and W;
  - exact agreement of OCT_EXP/OCT_LOG with the computed field and of Table 1 with `misc_facts.py`.
- **The `.source` and `.provenance_check` keys are strings.** They describe provenance and are not numbers to compare.

## Not done or not verified

- **The recorded runs were not repeated** (B-F21, B-F22). Repeating them needs `raptorq==2.0.0` from PyPI and 15+ minutes.
- **ATSC 3.0 (A/331) was not re-checked.** It underlies "It's in the standard for next-generation TV broadcasts". It was
  not among this bead's open items; the memo §2.6 quotes A/331:2021.
- **Only the 2006 edition of ISO/IEC 18004 was read.** The 2024 edition was not checked.
- **The official Intel SDM was not opened for GFNI.** The plan does not use GFNI.
