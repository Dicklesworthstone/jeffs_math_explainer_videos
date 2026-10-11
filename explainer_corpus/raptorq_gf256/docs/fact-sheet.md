# Video B fact sheet: The Arithmetic Where 3 × 7 = 9

This sheet covers bead `mx-e-b-qyy.2` and the claim walk of gate G3 (`claims.md` beside this file). It checks every
row B-F1 … B-F28 of the facts ledger in `docs/planning/COMPREHENSIVE_PLAN_FOR_JEFFS_MATH_EXPLAINER_VIDEOS.md` §10.5
against `explainer_corpus/raptorq_gf256/oracle.py` (the crate root, one level up from this file). It also records how
each of the bead's open items (to verify) was resolved, and the keys and source rows the claim walk asked for
([Rows added at the claim walk](#rows-added-at-the-claim-walk-gate-g3)).

**Run (from `explainer_corpus/raptorq_gf256/`):** `uv run --quiet --no-project --with numpy python3 -I -B oracle.py`,
which is what `./x raptorq_gf256 facts-check` runs.

The oracle needs only the standard library and numpy. It prints one `KEY<TAB>VALUE` line per fact and writes a summary
line to stderr. Latest run (2026-10-11, after R2; CPython 3.14.0 and numpy 2.5.4 from uv, this Mac, under
`nice -n 10`): **447 keys, real 5.2 s** (`/usr/bin/time -p`). R2 added 50 keys
([Rows added at R2](#rows-added-at-r2-2026-10-11)); a line diff against the claim walk's 397-key output shows only
additions, and no key is printed twice. The claim walk's run printed 397 keys in 11.0 s; the run of 2026-10-10
(CPython 3.14.6, numpy 2.4.1) printed 297 keys in 3.5–4.6 s; the claim walk added 100 keys and changed none of the 297.

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
  All 48 of them print `1` (37 before R2).

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
| B-F22 (stream, cost) | the hook's stream and one big decode, raptorq 2.0.0 crate (R3 Ruth R3-09, Dev D7, D2) | **RECORDED**: originals first (ESI 0…K−1, then repair packets in order), each packet lost i.i.d. with probability 1/100, a phone keeps the first K + h it receives: at K = 1,002 (no padding), **21,186 / 98 / 3** failures in **5,000,000** phones at h = 0, 1, 2 (the same phones for each h; ChaCha12 seed 20261011). At K = 10,000: 757 / 2 / 0 in 180,000. One decode of a 50,000-piece block (K′ = 50,511) from K + 2 packets, one core of an Apple M5 Ultra, medians of 5 runs: 96–127 ms with 8-byte pieces, 225–315 ms with 1,280-byte pieces (1% of the originals lost or repair packets only, two sets of runs) | 17_4 lower-third; 14_1 lower-third | MATCH (recorded) | `recorded_stream_and_cost` re-reads `results/rq_rust_stream_K1002_R200_5e6.txt` and `results/rq_rust_cost_K50000_T*.txt`; the harness is `scripts/rq_rust` (`rq_stream.rs`, `rq_cost.rs`; `rq_pool.rs` reproduces `rq_mc.py`'s pool draw: 105 / 20,000 at K = 1,002, h = 0, against the recorded 108). The stream's h = 0 rate (0.42%) is below the pool's (0.53%). Times are machine-dependent; the dense count (14_1) is not a time on this machine. |
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

## Rows added at the claim walk (gate G3)

The claim walk (`claims.md`, 2026-10-11, written against the script at `49ef758`) listed what the video says or shows
without an oracle key or a cited row. The rows below close those items against the current storyboard; each names the
beat it backs. Every new key is computed in `oracle.py`; a literal appears only for a sourced constant, and its key
contains `sourced`. The sources were downloaded on 2026-10-11 into a scratch folder and read there; none is committed.

### Keys added under existing rows

| row | beat | keys and values | how computed |
|---|---|---|---|
| B-F1 | 01_8 "The last few take forever"; the 01_8 screen's units | the last piece alone takes **10,000** draws on average and the last ten **29,289.7**, which is **29.9%** of the 97,876; the first half of the pieces takes **6,931**. At 1% loss the 97,876 pieces caught cost **98,864.7** packets sent | with j pieces missing, the next new one takes K/j draws on average; exact `Fraction` sums. Sent = caught / 0.99 (Wald's identity) |
| B-F6 | 13_6 screen "exact: 1.606 / 0.0039" (ten pieces) | Σ_{i ≤ 10} 1/(2ⁱ − 1) = **1.6057183**; Σ_{i ≤ 10} 1/(256ⁱ − 1) = **0.0039369**; a byte race needs any spare with probability **0.39215%** (`B-F6.mean_spares_gf*_K10`, `B-F6.p_any_spare_needed_gf256_K10`) | exact sums; a second route sums P(K + h cards leave a piece unpinned) over h (`…_K10_two_routes_ok`). 1.6067 is the many-piece limit (B-F6 above) |
| B-F6 | 13_6 race "with this run's own counts" (D15, RB-23) | `random.Random(1306)`, 1,000 races per side: ex-or spares 0, 1, 2, … occur **297, 303, 171, 94, 65, 39, 17, 4, 7, 0, 2, 0, 0, 0, 0, 1** times (mean **1.603**); byte spares 0, 1 occur **995, 5** times (mean **0.005**). Both means are within 4 standard errors of the exact ones | uniform random cards (the zero card included, as in the model) until rank 10; the rank is tracked as a GF(2) basis and as GF(256) rows reduced at their pivots |
| B-F9 | 16_5 lower-third "a loop of two-piece cards is stuck exactly when its amounts multiply to 1 going around" (R2: that wording is false; the lower-third now states the ratio rule this key tests, see [R2](#rows-added-at-r2-2026-10-11)); 13_7 "the other two fifty-four values" | in 3,000 seeded random 5-cycles with nonzero amounts the rank is below 5 exactly when ∏ bᵢ/aᵢ = 1 (**12** stuck cycles, all on the rule); 256 − 2 = **254** | `rank_gf256_py`, `random.Random(1605)` |
| B-F10 | 07_4 screen "2·131 = 262 → 6"; 07_6 lower-third "3 × 171 = 513 = 2·256 + 1" | **262**; 3⁻¹ mod 256 = **171**; 3·171 = **513** = 2·256 + 1 | exact |
| B-F11 | 10_7 code panel; 10_5 "x to the eighth"; 12_4 lower-third "(x²+x+1)² = x⁴+x²+1 = 21" | under 0x11B: mul(2, 141) = **1** and mul(2, 142) = **7**, so the panel's comment must flip with the fold. The panel's own code with `0x1B` gives mul(2, 141) = 1 and mul(0x57, 0x83) = **193** (0xC1), and equals multiplication mod 0x11B for all 65,536 pairs. x·x⁷ = **256** = x⁸ before the fold; 7·7 carry-less = **21** (no fold) | `rust_mul(…, fold=0x1B)` against `gmul(…, 0x11B)`; `clmul` |
| B-F12 | 12_4 lower-third "255 = (2+1)(2²+1)(2⁴+1); together they make 11111111₂ = 255" | (2+1)(2²+1)(2⁴+1) = **255**; the counts 1, 2, 4, …, 128 sum to **255** = 11111111₂ | exact |
| B-F13 | 12_4 lower-third "7 × 7: 198 + 198 = 396 → 141 clicks → 21"; 11_7 picture (127 mirror pairs); 11_5 "two cubed times two to the fourth" | log 7 = **198**; 198 + 198 = **396**, mod 255 **141**; α¹⁴¹ = **21**. log a⁻¹ = 255 − log a for all 255 nonzero bytes: **127** mirror pairs, and **1** is its own partner. 2³·2⁴ = **128**, as integers and as bytes | exact |
| B-F15 | 14_1 "about forty-two trillion steps. At a billion a second, half a day"; screen "≈ 11.6 h at 10⁹/s" | K³/3 = **41.67** trillion; at 10⁹ steps per second **41,666.7 s = 11.574 h = 0.482 days** | exact `Fraction` |
| B-F18 | 14_6 "over a hundred pieces are missing" | the runs that did not collapse leave **116–176** pieces missing at 2% extra and **97–135** at 5% extra | from the existing seeded runs; no new draws |
| B-F24 | 01_6 "Most phones need two or three loops"; picture "after loop 3 a few red stragglers" | P(a phone finishes in loop 2 or 3) = **0.99005**; after loop 3 **0.995%** of phones still wait: **9,950** of 10⁶, or **9.95** of the 1,000 glyphs if each glyph shows its share | the B-F24 formula |
| B-F24 | 01_8 screen "an average carousel phone ≈ 23,500" (packets caught, like the other two counts); "two extra packets 10,002" | expected packets **sent** until a phone is done: **23,729.95**; **caught**: **23,492.65**. Whole loops × K would give 26,421.9: a phone finishes partway through its last loop. The fountain phone: **10,002** caught, **10,103.0** sent on average | exact: P(N ≤ mK + j) = (1 − p^(m+1))^j (1 − p^m)^(K−j), summed in closed form at 120 digits; a second route sums every term in floating point; a third, independent of the formula, simulates 4,000 phones (`random.Random(18)`): 23,738.6, within 4 standard errors. Caught = 0.99 × sent (Wald) |
| B-F26 | 10_1 "The simplest fold drops the fallen bit back in at the bottom. A rotation." | ×2 mod x⁸ + 1 is an 8-bit rotate-left for all 256 bytes; 2·128 → **1** | exhaustive |

### New extra rows

| id | beat | keys and values | source and check |
|---|---|---|---|
| B-X3 | 14_8 lower-third "RaptorQ: up to 2²⁴ ≈ 16.8 million packets per block"; 03_8 "endless" | **SOURCED**: the ESI is 24 bits wide. Computed: 2²⁴ = **16,777,216** ESIs per source block = **16.78 million** | RFC 6330 §3.2 (quoted below). There is one ESI per encoding symbol, and a packet may carry several symbols (§4.4.2), so "packets" assumes one symbol per packet, as the video does |
| B-X4 | 02_1–02_6, 06_3, 08_1–08_3, 09_2–09_5 (small arithmetic said aloud or shown) | place values 1, 2, 4, …, 128; 5 = 101, 7 = 111; 2⁸ − 1 = 255; 5 ⊕ 7 = 2 (010) and 2 ⊕ 7 = 5; (a ⊕ b) ⊕ b = a for all 65,536 byte pairs; 2¹⁰ = 1,024, 2²⁰ = 1,048,576; 3⁻¹ mod 7 = 5, rows 1–6 of the seven-hour table are reshuffles and 0 has no partner; 2·128 = 256 needs 9 bits; the primes either side of 256 are 251 and 257; 3 = x + 1, 7 = x² + x + 1, 128 = x⁷, 9 = x³ + 1 | exact |
| B-X5 | 10_7 lower-third "CRC-8/SAE-J1850 uses this very 0x1D" | **SOURCED**: poly 0x1D (29), the low byte of 0x11D. Computed: the catalogue's 7 attested codewords from SAE J1850 Table 1 reproduce **7 of 7** with init 0xFF and xorout 0xFF, where each bit step is the code panel's xtime | reveng CRC catalogue (quoted below) |
| B-X6 | 10_7 "A famous encryption standard picked a different prime polynomial" | **SOURCED**: m(x) = x⁸ + x⁴ + x³ + x + 1 = 0x11B (283). Computed: FIPS 197's worked example, {57}•{02}, {04}, … , {80} = {ae}, {47}, {8e}, {07}, {0e}, {1c}, {38}, and {57}•{13} = **{fe}** | FIPS 197-upd1 §4.2 (quoted below); 0x11B is irreducible (B-F12) |

### Source rows (claims with no number to compute)

| id | beat | claim | source (read 2026-10-11; quotes below) | status |
|---|---|---|---|---|
| B-S1 | 03_9 "The fountain code we'll open up is called RaptorQ, an internet standard since twenty eleven." (RFC Editor record: `"pub_date": "August 2011"`, read 2026-10-11; the wording avoids "the standard", see the caveat); 17_6 "It's in the standard for next-generation TV broadcasts." | RaptorQ is a current IETF standard fountain code, and ATSC 3.0 uses it | RFC 6330 header "Category: Standards Track" and §1; the RFC Editor's record `https://www.rfc-editor.org/rfc/rfc6330.json` gives `"status": "PROPOSED STANDARD"` and `"obsoleted_by": []`. ATSC A/331:2021 §1, normative reference [28], Annex A.3.5.2, A.4.2.4 and Figure A.1.2 | SOURCED. Caveat for "the standard": Raptor (RFC 5053) is also a Proposed Standard with `"obsoleted_by": []` (`rfc5053.json`), and 3GPP MBMS still specifies Raptor (open item 4). RFC 6330 calls RaptorQ "a next generation of the Raptor code", so "the standard fountain code today" holds as the current generation. Only A/331:2021 was read; later editions were not |
| B-S2 | 11_7 "That's how the standard multiplies. Look up two click counts, add them, and look up the answer." | the standard defines multiplication through log and antilog tables | RFC 6330 §5.7.2; the click counts are OCT_LOG (B-F14) | SOURCED |
| B-S3 | 16_1 "In big blocks, RaptorQ's decoder mostly peels." | in the standard's decoder, most intermediate symbols are solved one at a time in the sparse first phase, and few are parked | RFC 6330 §5.4.2.2 and §5.4.2.4 | SOURCED, qualitatively. §5.4.2.4 says the i rows solved in the first phase are "generally much larger" in number than the u parked (inactivated) columns. A first-phase step that is not a peel (r ≥ 2) adds r − 1 ≥ 1 to u, which starts at P, so there are at most u − P such steps; with u much smaller than i, most steps are peels. It fails at small K′: at K′ = 10, P = 10 of L = 27 symbols are parked from the start (B-F19), hence "in big blocks"; at K′ = 10,017, P = 158 of 10,269. The RFC gives no number, and the count of columns parked during decoding is still unmeasured (D5) |
| B-S4 | 03_8 "a packet's number tells the phone which pieces it mixed"; lower-third "the ESI is the seed; the block size travels in a header" | the ESI fixes the mix through the standard's shared generator | RFC 6330 §5.3.1 (repair ISI = ESI + K′ − K), §5.3.4 and §5.3.5.4 (Tuple[K′, X] depends only on K′ and the ISI), §5.2 (the 1,024 fixed random numbers), §3.3.2 (F and T travel in the OTI) | SOURCED. The "pieces" are intermediate symbols (15_2 lower-third), and K′ follows from the header |
| B-S5 | 03_8 "like water from a fountain, so these are called fountain codes" | the name comes from the fountain metaphor | Byers, Luby, Mitzenmacher and Rege, "A Digital Fountain Approach to Reliable Distribution of Bulk Data", ACM SIGCOMM '98, §III; RFC 6330 abstract | SOURCED |
| B-S6 | 13_1 "This design really exists."; lower-third "random linear network coding" | cards with random amounts from a finite field are a real, named design | Ho, Médard, Koetter, Karger, Effros, Shi and Leong, "A Random Linear Network Coding Approach to Multicast", IEEE Trans. Inf. Theory 52(10), Oct. 2006, p. 4413 (abstract, Fig. 1); RFC 8681 (Standards Track, January 2020), abstract, §3.6, §3.7.1; RFC 8406 (IRTF, Informational) §2 and §4.1 | SOURCED. Ho et al. code at every network node; the video's sender-only case is what RFC 8406 calls End-to-End Coding, "a special case of Network Coding". RFC 8681's codes mix a sliding window rather than the whole file, over GF(2⁸) with the same polynomial x⁸ + x⁴ + x³ + x² + 1 |
| B-S7 | 01_5 "the requests swamp the sender" (unchecked in the walk) | feedback implosion | RFC 3453 §1 (Informational, December 2002) | SOURCED |
| B-S8 | lower-thirds 07_6 "it's why hash mixers use odd multipliers" and 08_4 "finite fields exist exactly for prime-power sizes" (unchecked in the walk) | standard facts | Wikipedia, fetched 2026-10-11: "Hash function" (Fibonacci hashing), "Universal hashing" (multiply-shift), "Finite field" | SOURCED from secondary sources only. The arithmetic beside them is computed (B-F10: 3·171 = 513; B-F12) |
| B-S9 | 15_1, 15_2, 16_3, 17_5, 14_8 | the RFC sentences the walk asked to quote | RFC 6330 §5.2, §5.3.3.4.2, §5.4.2.1, §1, §4.4.2 | SOURCED |

## Rows added at R2 (2026-10-11)

The R2 persona review (`docs/reviews/raptorq_gf256/R2.md`) changed words and pictures; every number those changes
speak or show, and every exact version they send to the description (`description.md` beside this file), is keyed
below. Every key is computed in `oracle.py`; the two literals are sourced constants with `sourced` in their keys.
Nothing in the 397 earlier keys changed.

### Keys added under existing rows

| row | beat | keys and values | how computed |
|---|---|---|---|
| B-F6 | 13_6 lower-third "with ten pieces the sum stops at 1/1023: 1.6057" | `B-F6.last_term_denominator_K10` = **1023** (the screen's 1.6057 and 1.6067 are `B-F6.mean_spares_gf2_K10` and `B-F6.mean_spares_gf2`) | 2¹⁰ − 1 |
| B-F7 | 13_4 "Fix every amount but one … exactly one choice of that amount lands in the lit part"; picture "255 land outside, 1 in" | exhaustive at two pieces: for all **257** lit lines of GF(256)², and each piece whose own card is off the line, every value of the other amount leaves exactly **1** of the 256 values of this one on the line (`B-F7.last_slot_exactly_one_amount_lit_exhaustive_K2_ok` = 1) | a 256 × 256 membership table per line (numpy) |
| B-F9 | 16_5 lower-third, corrected (Ruth R3) | the ring of cards 2X + 2Y has rank **4** over GF(256) (stuck), its ten amounts multiply to 2¹⁰ = **116**, its ratios b/a to **1**: the R1 wording ("amounts multiply to 1") fails here, the ratio rule holds | `rank_gf256_py`; `pow_g` |
| B-F9 | 13_7 "A card whose amounts are only zeros and ones is still an ex-or card, even in the byte field" (Ruth R18) | 300 seeded random 6 × 6 0/1 matrices have the same rank over GF(2) and over GF(256), by three implementations (`…_300_random_ok` = 1) | `random.Random(1307)` |
| B-F10 | 07_6 lower-third "it's why hash mixers use odd multipliers" (Dev dev-14) | **SOURCED** (B-S10): Linux's `GOLDEN_RATIO_32` = 0x61C88647 = **1,640,531,527** and `GOLDEN_RATIO_64` = 0x61C8864680B583EB; both are odd and have inverses mod 2³² and 2⁶⁴ (`…_ok` = 1) | `pow(g, -1, 2**w)` |
| B-F13 | 11_7 table-multiply lower-third (Dev dev-03) | of the **65,025** nonzero pairs, the largest log sum is **508** (so EXP needs 510 entries) and **32,131** have LOG[a] + LOG[b] ≥ 256, where a `u8` sum wraps; the lower-third's multiply with `u16` logs equals the field on all 65,536 pairs (`…_ok` = 1) | exhaustive |
| B-F15 | 14_1 lower-third "even ten times faster, still ≈ 1.2 h; the matrix alone is 2.5 GB; double the pieces, eight times the work" (Dev dev-09) | 50,000² = **2.5×10⁹** bytes; at 10¹⁰ steps a second **1.157 h**; (2K)³/K³ = **8** | exact |
| B-F17 | 14_7 "On average, about seventy of them aren't in any card"; caption "≈ 70 of the 10,000 pieces" (Ruth R6, Dev dev-04) | at 2% extra, N = 10,200 cards: a piece is in no card with probability (1 − 4.816642/10,000)^10,200 = **0.7342%**, i.e. **73.42** pieces on average, one in **136.2**; at N = K (the old wording) one in **123.5** | each card of degree d misses a given piece with probability 1 − d/K, independently |
| B-F21 | 15 "Model" note → description (Ruth R5); 17_2's reason → description (Maya M1) | the **shape model** at K′ = 10 (S + K′ + h uniform 0/1 rows and H = 10 uniform byte rows over L = 27 unknowns): failure **0.48939%**, **2.3576×10⁻⁵**, **1.2964×10⁻⁷** at h = 0, 1, 2; at h = 0 the ex-or part is **0.097624%** and the byte part **0.39215%**, and 1 − (1 − ex-or)(1 − byte) equals the rank sum exactly (`…_ok` = 1); h0/h1 = **207.6**; at h = 1 the extra ex-or row is useful with probability **0.99805**, and that case (a byte card left spare) is **64.85%** of the remaining failures | exact `Fraction` sum over the rank r of the 0/1 rows (the standard rank-r count), times P(H uniform byte rows span the other L − r directions) |
| B-F22 | 17_3 lower-third (Dev dev-10); the description's two harnesses (Ruth R9) | dividing by 256 twice predicts 6,316/256² = **0.0964** failures in 10⁶ at K + 2; a Poisson count with that mean is ≥ 1 with probability **9.19%**; 646/10⁵ (B-F21) against 6,316/10⁶ (B-F22): pooled two-proportion z = **0.547** | exact; `expm1` |
| B-F26 | 10_2 hint "Three times a byte is double it, ex-or itself"; 10_3 picture (Maya M4) | 3a = 2a ⊕ a under the rotation fold for all 256 bytes (`…_ok` = 1); 2·255 = **255** (11111111 rotates to itself); 255 ⊕ 255 = **0** | exhaustive |

### New extra rows

| id | beat | keys and values | source and check |
|---|---|---|---|
| B-X7 | 15_1–15_3, 15_6, 17_2: the precode (Ruth R1, Maya M2, M13, M22, Dev dev-01) | Every one of the L intermediate symbols ("every piece, old or new") has a nonzero entry in some precode row, for **all 477** K′ (`B-X7.precode_covers_every_intermediate_symbol_all_477_ok` = 1; built row by row for K′ = 10, 10,017 and 56,403 as a second route, `…_direct_ok` = 1). Each of C[0..B−1] goes into three **distinct** LDPC rows for all 477 K′ (S is an odd prime and a = 1 + ⌊i/S⌋ < S; the largest a is **2** at K′ = 10 and **41** at 10,017). Extra pieces S + H: **17** at ten pieces, **252** at 10,017; ex-or cards L − H: **17** of 27 and **10,258** of 10,269. Model: with exactly K′ = 10,017 packets, about **85.5** of the W = 10,111 LT symbols are in no packet (each packet's LT part hits a given one with probability d/W) | RFC 6330 §5.3.3.3 (quoted below). The dense HDPC part MT·Γ needs the V tables, which the oracle does not transcribe, so the coverage check leaves it out and is an undercount |
| B-X8 | 14_8 "Any set of exactly enough packets rebuilds the picture"; lower-third "Reed–Solomon over GF(256): N ≤ 257 (q + 1)" (Ruth R18, Dev dev-14) | The doubly extended Reed–Solomon code with K = 3 and N = **257** columns, (1, a, a²) for the 256 bytes a and (0, 0, 1): **3,000** seeded triples of columns all have rank 3 (`random.Random(1408)`, `…_ok` = 1). At K = 2 the 257 columns (1, a), (0, 1) are all **257** directions of GF(256)², so every pair rebuilds (`…_ok` = 1) and no 258th column can be added | computed; no textbook was opened for the MDS bound (Dev's candidates are unverified and not cited) |

### Source rows

| id | beat | claim | source (read 2026-10-11) | status |
|---|---|---|---|---|
| B-S10 | 07_6 lower-third "it's why hash mixers use odd multipliers" (Dev dev-14) | real multiplicative hashes multiply by an odd constant | Linux `include/linux/hash.h` (https://github.com/torvalds/linux/blob/master/include/linux/hash.h; the latest commit touching it, `fd0a1462405b`, is dated 2022-01-20) | SOURCED, primary (code). Quoted below. It replaces B-S8's Wikipedia pages as the main source of this lower-third; B-S8 still backs "finite fields exist exactly for prime-power sizes". The file also cites Knuth vol. 3 §6.4 for the golden-ratio choice; Knuth was not opened and is not cited |
| B-S11 | 17_7 credits lower-third "LT codes: Luby, 2002 · Raptor codes: Shokrollahi and Luby · RFC 6330 (2011): Luby, Shokrollahi, Watson, Stockhammer, Minder" (Ruth R20) | who made the codes | RFC 6330 header (p. 1) and §9.2 Informative References (p. 68), from the same file as above (sha256 `87f6da89…21ace`, re-downloaded 2026-10-11) | SOURCED as RFC 6330 cites them. The papers were not opened. RFC 6330's [RaptorCodes] is Shokrollahi and Luby's 2011 monograph, not the 2006 IEEE Trans. Inf. Theory paper Ruth named; that paper was not read and is not cited |

### Quotes added at R2

- **RFC 6330 §5.3.3.3, p. 22** (B-X7; 15_2 "picked so that some fixed cards all have the result zero"): *"The
  pre-coding relationships amongst the L intermediate symbols are defined by requiring that a set of S+H linear
  combinations of the intermediate symbols evaluate to zero."* and *"The P PI symbols consist of the H HDPC symbols
  together with a set of U = P-H of the other K' intermediate symbols. The W LT symbols consist of the S LDPC symbols
  together with W-S of the other K' intermediate symbols."* The LDPC loop, p. 23: *"a = 1 + floor(i/S)"*, *"b = i %
  S"*, *"D[b] = D[b] + C[i]"*, *"b = (b + a) % S"* (three times), then for i = 0, …, S−1 *"D[i] = D[i] + C[W+a] +
  C[W+b]"* with a = i % P, b = (i+1) % P. The HDPC relations, p. 24: *"Transpose[C[K'+S], ..., C[K'+S+H-1]] + MT *
  GAMMA * Transpose[C[0], ..., C[K'+S-1]] = 0"*.
- **RFC 6330 header, p. 1** (B-S11): authors *"M. Luby"*, *"A. Shokrollahi"*, *"M. Watson"*, *"T. Stockhammer"*, *"L.
  Minder"*; *"August 2011"*. **§9.2, p. 68**: *"[LTCodes] Luby, M., "LT codes", Annual IEEE Symposium on Foundations
  of Computer Science, pp. 271-280, November 2002."* and *"[RaptorCodes] Shokrollahi, A. and M. Luby, "Raptor Codes",
  Foundations and Trends in Communications and Information Theory: Vol. 6: No. 3-4, pp. 213-322, 2011."*
- **Linux `include/linux/hash.h`** (B-S10): *"This hash multiplies the input by a large odd number and takes the high
  bits."*; *"Although a random odd number will do, it turns out that the golden ratio phi = (sqrt(5)-1)/2, or its
  negative, has particularly nice properties."*; `#define GOLDEN_RATIO_32 0x61C88647` and `#define GOLDEN_RATIO_64
  0x61C8864680B583EBull`.

## Simplifications → exact (plan §5.1 rule 11; R2 Ruth R4)

Every simplification the video speaks or shows, its exact version, and where a viewer finds it. "Description" means
`description.md` beside this file, under the same beat id; the viewer-facing wording lives there.

| beat | simplification | exact version | home | backing |
|---|---|---|---|---|
| 01_1 | "randomly loses about one packet in a hundred" | each phone loses each packet independently with probability 1% | description | B-F2, B-F24 |
| 01_2, 17_4, 17_7 | "almost every phone that catches just two packets more than it needs" | RFC 6330 §5.8: at most 10⁻⁶ at K′ + 2, averaged over ESIs drawn uniformly from the whole range, for each K′ in Table 2; our runs drew from a fixed pool of the K originals and 20,000 repair packets; a phone that loses random packets from an originals-first stream holds a third mix that neither covers; that stream measured since R3 (B-F22 stream): 3 failures in 5,000,000 phones at 1,002 pieces with two extra | 17_4 lower-third; description | B-F23, B-F22 |
| 01_6 | "Most phones need two or three loops. The unluckiest of a million need five or six." | 99.005% of phones finish in loop 2 or 3; all 10⁶ are done by loop 5 with probability 0.3679 and by loop 6 with 0.9900 | description | B-F24 |
| 01_8 | "about ninety-eight thousand", "≈ 23,500", "10,002" | K·H_K = 97,876.06 caught (98,864.7 sent); the carousel's 23,492.65 caught (23,729.95 sent); 10,002 caught (10,103.0 sent), all averages at 1% loss | screen ("average"); description | B-F1, B-F24 |
| 03_5 | "useful" | linearly independent of the cards held; the bucket's filled slots are the rank | description | — |
| 03_8 | "a packet's number tells the phone which pieces it mixed" | the ESI fixes the ISI, and Tuple[K′, ISI] fixes the mix | 03_8 lower-third | B-S4 |
| 03_9 | "an internet standard since twenty eleven" | RFC 6330, IETF Proposed Standard, August 2011 | name card | B-S1 |
| 05_2 | "With many pieces it fails seventy-one percent" | 1 − ∏(1 − 2⁻ⁱ), i = 1…K: 0.710930 at K = 10, 0.711212 in the limit (model: uniform 0/1 cards) | 05_2 lower-third; "model" label | B-F4, B-F5 |
| 05_3, 05_4 | "Almost all the danger is in the last few slots" | the chance of failing outside the last three slots is 11.99% | description | B-F5.fail_gf2_h3 |
| 06_2 | "each one after that roughly halves it" | 71.1% → 42.2% → 23.0% → 12.0%: factors 1.68, 1.84, 1.92, tending to 2 | description | B-F5 |
| 06_3 | "about twenty spares" | the smallest h with failure ≤ 10⁻⁶ is 20 (9.54×10⁻⁷) | description | B-F28 |
| 06_5 | "a card would hardly ever be useless" | with uniform amounts in a field, a last-slot card is useless with probability 1/256 (13_4); with wrap-around amounts it is still 1/2 (07) | description | B-F7, B-F25 |
| 07_5, 07_6 | "only the last bit matters" | a matrix over Z/256 is invertible iff its determinant is odd iff it is invertible mod 2 | 07_6 lower-third | B-F25 |
| 08_4 | "is called a field" | finite fields exist exactly for prime-power sizes | 08_4 lower-third | B-S8 |
| 10_1, 10_3 | "the rotation" | reduction mod x⁸ + 1 = (x + 1)⁸, which has zero divisors (3·255 = 0) | description | B-F26 |
| 10_4 | "a prime polynomial" | irreducible over GF(2); 0x11D is also primitive, 0x11B (AES) is not | 10 and 11_2 lower-thirds | B-F12 |
| 10_6 | "So bytes form a field, the byte field" | GF(2)[x]/(x⁸ + x⁴ + x³ + x² + 1) ≅ GF(2⁸); all fields of order 256 are isomorphic; the polynomial is a choice of labels | 10_6 and 10 lower-thirds | B-F11, B-F12 |
| 11_2 | "you visit every nonzero byte" | 2 is primitive under 0x11D; under 0x11B its order is 51 | 11_2 lower-third | B-F12 |
| 11_7 | "That's how the standard multiplies" | RFC 6330 §5.7.2 defines × through OCT_EXP and OCT_LOG; implementations may use other kernels | description | B-S2 |
| 11_7 | "every nonzero byte's partner sits at its mirror image" | log a⁻¹ ≡ 255 − log a (mod 255); 1 is its own partner; 0 has none | screen | B-F13 |
| 13_4 | "Only one time in two fifty-six … exactly one choice of that amount" | fix the other amounts; for a piece whose own card is not lit, exactly one of its 256 amounts lands in the lit part | 13_4 lower-third | B-F7 (exhaustive at K = 2) |
| 13_5 | "every extra byte card divides the risk by two fifty-six" | union bound 256⁻ʰ/255; exact 3.9215×10⁻³, 1.5319×10⁻⁵, 5.984×10⁻⁸ at h = 0, 1, 2 (model) | 13_5 lower-third | B-F8, B-F27 |
| 13_6 | "Ex-or needs one or two spares on average" | 1.6057 at ten pieces; 1.6067 in the limit (Σ 1/(2ⁱ − 1), irrational, Erdős 1948); byte cards 0.0039 | screen; 13_6 lower-third | B-F6 |
| 13_7 | "still an ex-or card, even in the byte field" | a 0/1 matrix has the same rank over GF(2) and over GF(256) | description | B-F9 |
| 14_1 | "about the number of pieces, cubed, over three" | Gaussian elimination takes about K³/3 multiply-adds: 4.17×10¹³ at K = 50,000 | screen; 14_1 lower-third | B-F15 |
| 14_2 | "About half mix two, and a few mix many" | RFC Table 1: P(2) = 1/2; P(d) ≈ 1/(d(d − 1)) for d = 2…29; mean 4.82 | 14_2 lower-third | B-F16 |
| 14_6 | "most runs get stuck around ninety-eight percent"; "over a hundred pieces" | 5 seeded runs at 2% extra: 98.24–98.84% (one collapsed at 15.8%), 116–176 pieces missing | caption; description | B-F18 |
| 14_7 | "On average, about seventy of them" | 10,000 (1 − 4.8166/10,000)^10,200 = 73.4 | description | B-F17 |
| 14_8 | "Any set of exactly enough packets rebuilds"; "around two hundred fifty-six" | Reed–Solomon codes are MDS; over GF(256), N ≤ 257 (q + 1); RaptorQ allows 2²⁴ ESIs per block | 14_8 lower-third | B-X8, B-X3 |
| 15_1 | "of about seven pieces on average" | 7.152 intermediate symbols per packet: Table 1's mean degree 4.82 plus 2.34 PI symbols (Rand[X, 3, 2] modelled as uniform) | screen; description | B-F16 |
| 15_2 | "the sender adds some extra pieces … fixed cards all have the result zero" | S + H extra intermediate symbols; S LDPC and H HDPC relations evaluate to zero; packets are ex-or cards over all L symbols; the sender solves for all of them so the first K′ packets are the originals | 15_2 lower-third | B-S9, B-X7 |
| 15_2 | "every piece is in some free card" | every intermediate symbol has a nonzero entry in a precode row, for all 477 K′; at 10,017 pieces about 85 of the 10,111 LT symbols are in no packet when exactly K′ arrive (model) | description | B-X7 |
| 15_6 | "only ten to sixteen free cards are byte cards"; "hundreds of others" | H ∈ [10, 16] over the 477 K′ of Table 2; S = 241 at K′ = 10,017 | 15_6 lower-third (blocks); screen | B-F19 |
| 15_3 | "they fill it up to the last ten to sixteen slots" | L − H = K′ + S ex-or rows at every K′: 17 of 27 at ten pieces, 10,258 of 10,269 at 10,017 | screen | B-X7 |
| 15_3 | "useless only one time in two fifty-six" | model (uniform byte rows); the real HDPC rows are structured: random byte rows fail 0.685%, RFC rows 0.646% at K′ = 10 | description | B-F21 |
| 15 | (description only) the shape model | random rows in RaptorQ's shape at K′ = 10 fail 0.489% = 1 − (1 − 0.0976%)(1 − 0.392%) | description | B-F21 |
| 15_5 | "about six in a thousand" | 646 / 100,000 = 0.646% (our RFC 6330 constraint-matrix harness, ESIs uniform over 2²⁴); the decoder's 0.632% (17_1) agrees, z = 0.55 | caption; description | B-F21, B-F22 |
| 16_1 | "In big blocks, RaptorQ's decoder mostly peels" | RFC §5.4.2.4: the rows solved one at a time are "generally much larger" in number than the inactivated columns | description | B-S3 |
| 16_3 | "it works exactly when the cards pin down every piece" | decoding succeeds iff rank A = L, in any elimination order | 16_3 lower-third | B-S9 |
| 16_5 | the ring | a ring of cards a·X + b·Y is stuck iff the ratios b/a multiply to 1 going around | 16_5 lower-third | B-F9 |
| 17_1 | "a real RaptorQ decoder a million times" | raptorq 2.0.0 (Rust, cberner), K = 10, 10⁶ trials per h, packets drawn from a fixed pool of the 10 originals and 20,000 repair packets | caption; description | B-F22 |
| 17_2 | "One more ex-or packet usually fills one more slot, leaving a byte card spare, and that divides the risk by about two fifty-six" | shape model: the extra ex-or row is useful with probability 0.998, that case is 65% of the remaining failures, and the model's factor is 208; measured 6,316 / 24 = 263 | description | B-F21, B-F22 |
| 17_3 | "too rare to measure precisely" | 1 in 10⁶, 95% interval 2.53×10⁻⁸ to 5.57×10⁻⁶; ÷256 twice predicts 0.096, and ≥ 1 happens 9.2% of the time | screen; 17_3 lower-third | B-F22, B-F23 |
| 17_6 | "a standard for next-generation TV broadcasts"; "older sibling … two thousand five"; "Linux's disk arrays"; "every QR code" | ATSC A/331:2021 names RaptorQ, and its FEC is optional (Annex A.4.1); 3GPP chose Raptor (RFC 5053) for MBMS in 2005; Linux md RAID-6 (Anvin); ISO/IEC 18004:2006 §6.5.2 (only that edition read) | cards; description | B-S1, open items 2–4 |
| 17_7 | "because a polynomial refuses to factor" | p irreducible ⇒ GF(2)[x]/(p) has no zero divisors ⇒ every nonzero row of the table is a reshuffle ⇒ it contains 1 | 10 lower-third | B-F11, B-F12 |

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

### Quotes added at the claim walk (gate G3)

All RFC 6330 quotes are from https://www.rfc-editor.org/rfc/rfc6330.txt, re-downloaded on 2026-10-11 (sha256
`87f6da89…21ace`, the same file as above); page numbers are the RFC's own.

- **§3.2, p. 5** (B-X3): *"Encoding Symbol ID (ESI): 24-bit unsigned integer. A non-negative integer identifier for
  the encoding symbols within the packet."*
- **§4.4.2, p. 11** (B-X3, B-S9 for 14_8): *"A packet may contain any number of symbols from the same source block."*
  and *"Note that it is not necessary for the receiver to know the total number of repair packets."*
- **Header and §1, p. 1 and p. 4** (B-S1, B-S5, B-S9 for 17_5): *"Category: Standards Track"*; *"The RaptorQ code
  described herein is a next generation of the Raptor code described in RFC 5053 [RFC5053]."*; *"RaptorQ is a
  fountain code, i.e., as many encoding symbols as needed can be generated on the fly by the encoder from the source
  symbols of a block."*; *"The code described in this document is a systematic code; that is, the original unmodified
  source symbols, as well as a number of repair symbols, can be sent from sender to receiver."* The abstract (p. 1):
  *"RaptorQ is also a fountain code, i.e., as many encoding symbols as needed can be generated on the fly by the
  encoder from the source symbols of a source block of data."*
- **RFC Editor records** (B-S1), `https://www.rfc-editor.org/rfc/rfc6330.json`: `"status": "PROPOSED STANDARD"`,
  `"obsoleted_by": []`; `https://www.rfc-editor.org/rfc/rfc5053.json` (Raptor): `"status": "PROPOSED STANDARD"`,
  `"obsoleted_by": []`.
- **§5.7.2, p. 63** (B-S2): *"The multiplication of two octets is defined with the help of two tables OCT_EXP and
  OCT_LOG, which are given in Section 5.7.3 and Section 5.7.4, respectively. … For two octets u and v, we define
  u * v = 0, if either u or v are 0, OCT_EXP[OCT_LOG[u] + OCT_LOG[v]] otherwise."*
- **§5.4.2.2, pp. 33–34** (B-S3): the parameters *"i and u, which are initialized to 0 and P, the number of PI
  symbols"*; *"Let r be the minimum integer such that at least one row of A has exactly r nonzeros in V."*; *"Finally,
  i is incremented by 1 and u is incremented by r-1, which completes the step."*
- **§5.4.2.4, p. 35** (B-S3): *"The number of rows i of the submatrix U_upper is generally much larger than the number
  of columns u of U_upper."*
- **§5.2, p. 17** (B-S4, B-S9 for 15_1): *"The encoder produces repair symbols from the intermediate block using an
  efficient process, where each such repair symbol is the exclusive-or of a small number of intermediate symbols from
  the block."*; *"This generator is based on a fixed set of 1024 random numbers that must be available to both sender
  and receiver."*
- **§5.3.1, p. 19** (B-S4): *"a repair symbol ISI is simply the repair symbol ESI plus K'-K."*
- **§5.3.4, p. 27** (B-S4): *"the repair symbol with ISI X (X >= K') is generated by applying the generator Enc[K',
  (C[0], C[1], ..., C[L-1]), (d, a, b, d1, a1, b1)] defined in Section 5.3.5.3 to the L intermediate symbols C[0],
  C[1], ..., C[L-1] using the tuple (d, a, b, d1, a1, b1)=Tuple[K',X]."*
- **§5.3.5.4, p. 30** (B-S4): *"The tuple generator Tuple[K',X] takes the following inputs: K': the number of source
  symbols in the extended source block; X: an ISI"* (list layout flattened).
- **§3.3.2, p. 5** (B-S4): *"Transfer Length (F): 40-bit unsigned integer."* and *"Symbol Size (T): 16-bit unsigned
  integer."*, both in the Common FEC Object Transmission Information.
- **§5.3.3.4.2, p. 25** (B-S9 for 15_2): *"D denote the column vector consisting of S+H zero symbols followed by the K'
  source symbols C'[0], C'[1], ..., C'[K'-1]."*
- **§5.4.2.1, pp. 31–32** (B-S9 for 16_3): *"It is clear that C can be decoded if and only if the rank of A is L."* and
  *"The order in which Gaussian elimination is performed to form the decoding schedule has no bearing on whether or
  not the decoding is successful."*

**ATSC A/331:2021**, "Signaling, Delivery, Synchronization, and Error Protection", 19 January 2021
(https://www.atsc.org/wp-content/uploads/2021/01/A331-2021-Signaling-Delivery-Sync-FEC.pdf), backing B-S1:
- §1: *"This document specifies protocols used for delivery and synchronization of media and non-timed data in the
  ATSC 3.0 system."*
- Normative reference [28]: *"IETF: RFC 6330, “RaptorQ Forward Error Correction Scheme for Object Delivery,” Internet
  Engineering Task Force, Reston, VA, August, 2011."*
- Annex A.3.5.2: *"In accordance with RFC 6330 [28] Section 3.2, the FEC Payload ID for the RaptorQ FEC Scheme used
  for repair flows is composed of a Source Block Number (SBN) and an Encoding Symbol ID"*.
- Annex A.4.2.4: *"The FEC building block is used according to RFC 6330 [28], but only repair packets are delivered."*
  Figure A.1.2 labels the repair path *"FEC Scheme Raptor(Q) (RFC6330)"*. Annex A.4.1 says delivery objects *"may be
  protected with FEC"*: the FEC is optional, and RaptorQ is the scheme it names.

**Byers, Luby, Mitzenmacher and Rege**, "A Digital Fountain Approach to Reliable Distribution of Bulk Data", ACM
SIGCOMM '98 (copy read: https://people.cs.rutgers.edu/~rmartin/teaching/fall04/cs552/readings/by98.pdf; the SIGCOMM '98
programme page https://conferences2.sigcomm.org/sigcomm/1998/tp/abs_05.html lists the paper), §III, backing B-S5:
*"We metaphorically describe the stream of encoding packets produced by the server in this idealized solution as a
digital fountain. The digital fountain has properties similar to a fountain of water for quenching thirst: drinking a
glass of water, irrespective of the particular drops that fill the glass, quenches one’s thirst."*

**Ho et al. 2006**, IEEE Trans. Inf. Theory 52(10), 4413 (copy read: a co-author's,
https://www.comp.nus.edu.sg/~bleong/publications/itrandom-final.pdf), backing B-S6: abstract, *"We present a
distributed random linear network coding approach for transmission and compression of information in general
multisource multicast networks. Network nodes independently and randomly select linear mappings from inputs onto output
links over some field."*; Fig. 1 caption, *"the coefficients are randomly chosen elements of a finite field."*

**RFC 8681** (Standards Track, January 2020; https://www.rfc-editor.org/rfc/rfc8681.txt), backing B-S6:
- Abstract: *"This document describes two fully specified Forward Erasure Correction (FEC) Schemes for Sliding Window
  Random Linear Codes (RLC), one for RLC over the Galois Field (a.k.a., Finite Field) GF(2), a second one for RLC over
  the Galois Field GF(2^(8)), each time with the possibility of controlling the code density."*
- §3.6: *"In particular, when DT equals 15 the function guaranties that all coefficients are nonzero (i.e., maximum
  density)."*
- §3.7.1: *"The following irreducible polynomial is used for GF(2^(8)): x^(8) + x^(4) + x^(3) + x^(2) + 1"*.

**RFC 8406** (IRTF, Informational, June 2018; https://www.rfc-editor.org/rfc/rfc8406.txt), backing B-S6: §2, *"Network
Coding: A system where coding can be performed at the source as well as at intermediate forwarding nodes (all or a
subset of them). End-to-End Coding is regarded as a special case of Network Coding."*; §4.1, *"Random Linear Coding
(RLC): Particular case of Linear Coding using a set of random coding coefficients."*

**RFC 3453** (Informational, December 2002; https://www.rfc-editor.org/rfc/rfc3453.txt), §1, backing B-S7: *"However,
for one-to-very-many reliability protocols, ARQ has limitations, including the feedback implosion problem because many
receivers are transmitting back to the sender, and the need for a back channel to send these requests from the
receiver."*

**reveng CRC catalogue**, https://reveng.sourceforge.io/crc-catalogue/1-15.htm (page "Last updated 11 December 2024"),
entry CRC-8/SAE-J1850, class "attested", backing B-X5: `width=8 poly=0x1d init=0xff refin=false refout=false
xorout=0xff check=0x4b residue=0xc4 name="CRC-8/SAE-J1850"`, citing *"SAE Standard J1850 (15 May 1994), Excerpt"*,
Section 5.4.1 for the definition and Table 1 for 7 codewords (`0000000059`, `F2018337`, `0FAA005579`, `00FF5511B8`,
`332255AABBCCDDEEFFCB`, `926B558C`, `FFFFFFFF74`; the last byte of each is the CRC). The SAE standard itself was not
opened.

**FIPS 197-upd1**, Advanced Encryption Standard, "Published November 26, 2001; Updated May 9, 2023"
(https://doi.org/10.6028/NIST.FIPS.197-upd1), §4.2, backing B-X6: *"the resulting polynomial is reduced modulo the
following fixed polynomial: m(x) = x8 + x4 + x3 + x + 1. (4.3)"* (superscripts and the "fi" ligature lost in the text
layer) and the worked
example *"{57} • {13} = {57} • ({01} ⊕ {02} ⊕ {10}) = {57} ⊕ {ae} ⊕ {07} = {fe}. (4.7)"*.

**Wikipedia** (secondary; raw wikitext fetched 2026-10-11), backing B-S8:
- "Hash function", §Fibonacci hashing: *"The multiplier should be odd, so the least significant bit of the output is
  invertible modulo 2^w."*
- "Universal hashing", multiply-shift: *"the hash functions are parametrised over odd positive integers a < 2^w"* and
  *"odd integers have inverses in the ring Z_{2^w}"* (markup removed).
- "Finite field": *"A finite field of order q exists if and only if q is a prime power p^k (where p is a prime number
  and k is a positive integer)."* (markup removed).

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
- **The claim walk's seeded keys have the same limit:** the 13_6 race (`B-F6.race_K10_seeded.*`,
  `random.Random(1306)`), the loop rule (`B-F9.cycle_rule_*`, `random.Random(1605)`) and the carousel simulation
  (`B-F24.carousel_mc_*`, `random.Random(18)`). The exact keys beside them (`B-F6.mean_spares_gf*_K10`,
  `B-F24.carousel_mean_packets_*`) use no random numbers.
- **R2's seeded keys, likewise:** `B-F9.rank_01_matrices_same_over_gf2_and_gf256_300_random_ok`
  (`random.Random(1307)`) and `B-X8.rs_K3_N257_every_seeded_triple_rebuilds_ok` (`random.Random(1408)`). Both are
  `_ok` checks that hold for any draw, so a Rust port may use its own generator.
- **B-F14, B-F16 and B-F19 depend on the RFC tables transcribed into `oracle.py`.** The transcription is guarded in
  three ways:
  - a sha256 of Table 2 (`B-F19.table2_transcription_hash_ok`);
  - the RFC's own primality statement for S and W;
  - exact agreement of OCT_EXP/OCT_LOG with the computed field and of Table 1 with `misc_facts.py`.
- **The `.source` and `.provenance_check` keys are strings.** They describe provenance and are not numbers to compare.

## Not done or not verified

- **The recorded runs were not repeated** (B-F21, B-F22). Repeating them needs `raptorq==2.0.0` from PyPI and 15+ minutes.
- **ATSC 3.0: only A/331:2021 was read** (at the claim walk, B-S1). Later editions of A/331 were not checked.
- **Only the 2006 edition of ISO/IEC 18004 was read.** The 2024 edition was not checked.
- **The official Intel SDM was not opened for GFNI.** The plan does not use GFNI.
- **No key for the 03_8 picture "packet #48213 → pieces {…}".** The pieces would come from RFC 6330's Tuple[K′, X],
  which needs the V0–V3 tables (§5.5); the oracle does not transcribe them.
- **How many columns RaptorQ's decoder parks while decoding is unmeasured, and so is its decode time** (D5; R2 Dev
  dev-02, deferred to bead mx-e-b-qyy.10: the PyPI `raptorq` 2.0.0 harness is not installed on this machine). B-S3
  rests on the RFC's qualitative "generally much larger"; only the P symbols parked from the start are computed
  (B-F19). No RaptorQ decode cost is spoken; 15_1 states the packets' density (B-F16) instead.
- **No run covers the phones' own packet mix** (R2 Ruth R2): originals first, each lost independently at 1%, then
  repair packets. The standard's figures (B-F23) and our runs (B-F22, a fixed pool of the K originals and 20,000 repair packets) are
  for other mixes; the description says so. The run (the existing `rq_mc.py` harness at K = 1,002 with that loss model) is
  deferred to bead mx-e-b-qyy.10.
- **The 15 "Model" note is keyed since R2** (`B-F21.shape_model.*`); it stays in the description, not on screen.
- **Not opened, so not cited** (R2): Dev's RAID-DP / EVENODD papers (dev-07), Knuth's TAOCP vol. 3 and MacWilliams
  and Sloane (dev-14), Ruth's "Shokrollahi, IEEE Trans. Inf. Theory 2006" (R20), and the LT and Raptor papers
  themselves (B-S11 cites them only as RFC 6330 lists them).
- **SAE J1850 itself and Williams's CRC paper were not opened**; B-X5 rests on the catalogue's attested entry and
  its codewords.
