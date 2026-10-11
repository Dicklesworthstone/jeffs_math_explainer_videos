# Verification scripts for the RaptorQ / GF(256) memo

These scripts reproduce the numbers in `../MEMO.md`. All of them run from any working directory and import their siblings by their own location.

## Setup

- **Python:** 3.10 or newer. The memo runs used 3.14.
- **Python packages:** `numpy` (the memo used 2.4.1) for most scripts. `rq_validate.py` and `rq_mc.py` also need the PyPI package `raptorq==2.0.0`, which is Python bindings for the Rust crate https://github.com/cberner/raptorq. Install with `python3 -m venv /tmp/rqvenv && /tmp/rqvenv/bin/pip install numpy raptorq==2.0.0`, then run those two scripts with `/tmp/rqvenv/bin/python -I ...`.
- **RFC 6330 text:** scripts that take `--rfc` need it. It is not stored in the repo. Download it with `curl -sL https://www.rfc-editor.org/rfc/rfc6330.txt -o rfc6330.txt` and pass its path.

## Scripts

Runtimes were measured on a heavily loaded machine (load average ~275 on 14 cores), so wall time is pessimistic.

| Script | What it verifies | Run | Runtime | Memo numbers it reproduces |
|---|---|---|---|---|
| `rfc6330_data.py` | Parser module for RFC 6330 Table 2, OCT_EXP/OCT_LOG and V0..V3. Used by other scripts; running it directly prints a summary. | `python3 -I rfc6330_data.py rfc6330.txt` | < 1 s | 477 K' values; 510/255-entry tables |
| `gf256np.py` | Module: numpy GF(256) with 0x11D, plus `rank256` / `rank2`. | (imported) | n/a | n/a |
| `gf256_facts.py` | Exhaustive field facts: RFC tables are exactly the powers of x mod 0x11D; worked products (2·128=29, 0x57·0x83=0x31 vs AES 0xC1, 128·3=157, 150·3=167, 2⁻¹=142); 30 irreducible / 16 primitive polynomials; order of 2 under 0x11B = 51; element-order histogram; subfields; Hankel log-ordering; xtime. | `python3 -I gf256_facts.py --rfc rfc6330.txt` (`--rfc` optional) | ~15 s | §2.1 (field), §2.4, ledger rows 1–2 |
| `rank_probs.py` | Exact full-rank probabilities ∏(1−q^-i) for q=2 and 256; the K+h failure table; ratios; expected extra equations (1.6067 / 0.00394). | `python3 -I rank_probs.py` | < 1 s | §2.3, ledger row 3 |
| `mc_random.py` | Monte Carlo check of `rank_probs.py` with random (k+h)×k matrices. | `python3 -I mc_random.py 20 20000` (args: k, trials) | ~1 min at 20000 trials | §2.3 MC values (0.712 / 0.429 / 0.223 / 0.119; 0.00395) |
| `rq6330.py` | Module: independent implementation of the RFC 6330 constraint matrix A (LDPC, HDPC = MT·Γ, LT rows from Tuple/Enc), with HDPC-ablation modes. | (imported; call `rq6330.load(rfc_path)`) | n/a | n/a |
| `rq_validate.py` | Byte-exact cross-check of `rq6330.py` against the cberner/raptorq encoder for K'=10, 26, 101; full rank of the systematic A for the first N K' values. | `python -I rq_validate.py --rfc rfc6330.txt [--kcount 120]` | ~10 s with `--kcount 30`; minutes with 120 | §2.2 "60/60, 76/76, 151/151 byte-identical; full rank for first 120 K'" |
| `rq_rust/` (Rust, `cargo run --release --bin …`) | The same cberner decoder as a crate (`raptorq = "=2.0.0"`): `rq_stream` (the video's hook stream: originals first, 1% i.i.d. loss, K + h received), `rq_cost` (one decode's wall time), `rq_pool` (`rq_mc.py`'s pool draw). | `rq_stream 1002 200 5000000 4 20261011`; `rq_cost 50000 8 5`; `rq_pool 1002 20000 20000 4 2026` | ~20 min on 4 threads for the 5M stream run; seconds for the others | `results/rq_rust_*.txt` (fact sheet B-F22 stream, cost) |
| `rq_mc.py` | Decode-failure rate of the real cberner/raptorq decoder at K, K+1, K+2 random received symbols. | `python -I rq_mc.py 10 1000000 20000` (args: K, trials, repair-pool size) | ~25 s at 20000 trials; ~15 min at 10⁶ | §2.2 table (0.632%, 2.4×10⁻⁵, 10⁻⁶; K=101, 1002) |
| `rq_ablate.py` | Rank failure of RaptorQ's A at K'+h with HDPC rows as specified (`gf256`) or swapped for random dense GF(2) (`gf2random`) or GF(256) (`gf256random`) rows. | `python3 -I rq_ablate.py --rfc rfc6330.txt 10 10000 gf2random 0` (args: K', trials, mode, h) | ~3–10 ms per trial at K'=10; ~0.1 s at K'=101 | §2.2 ablation (0.646% / 0.685% vs 70.7%, 42.7%, 22.9%; K'=101 70.2%, 41.6%, 24.4%) |
| `toy_examples.py` | Video toy examples: article's 4 packets (rank 4); 5-unknown peeling order and ripple; inactivation (c=7); 5-cycle rank 4 over GF(2) and GF(256), 5 with one twist (A=15/3=5); P(product=1) ≈ 1/255; Z/256 row collapse; Erdős–Borwein sum. | `python3 -I toy_examples.py` | < 10 s | §2.3 nuance, §2.5 toy and cycle, Ch. 2/7/8 examples |
| `misc_facts.py` | RFC Table 1 equals the ideal soliton 1/(d(d−1)) for d=2..29; mean degree 4.817; ≈7.15 symbols per repair; Table 2 S/H/W/L/P rows; H ∈ 10..16; coupon-collector numbers (97,876; e^-5; e^-4.82); 50000³; feedback-implosion exponent; MC standard errors. | `python3 -I misc_facts.py --rfc rfc6330.txt` (`--rfc` optional) | < 1 s | §2.1 parameters and degree table, §2.5, §2.6 |
| `lt_peel_sim.py` | Pure LT peeling with the RFC degree table, K=10,000, N=K(1+ε): recovered and never-touched fractions; expected equations to full rank for small k. Seeded. | `python3 -I lt_peel_sim.py` | a few minutes on a loaded machine | §2.5 peeling-stall numbers (≈98–99% at 2–5% overhead) |

## `results/`

- **`original_*.txt`:** raw outputs of the long Monte Carlo runs quoted in the memo. These are the cberner `rq_mc` runs and the `rq_ablate` runs.
  - Files marked `_partial` were stopped before their K+2 pass finished, so only the h=0 and h=1 lines exist. The memo quotes only those lines.
- **`rerun_*.txt`:** re-runs from this directory on 2026-10-09, done after moving the scripts here to confirm they still work. For the deterministic scripts these are complete outputs.
- **`rerun_smoke_*.txt`:** short runs with reduced trial counts. They are only a check that each script runs; they have too few trials to reproduce the memo's estimates.
- **`rq_validate.py --kcount 120` was not re-run here.** It was stopped after 5 minutes on the loaded machine. The original run in the research session completed with no failures. The `--kcount 30` re-run, which includes the byte-exact cross-check, passed.

## Not included

These were deliberately left out:
- Downloaded third-party material: the RFC text, the papers and PDFs, the HTML of the owner's article, and the AI chat transcripts.
- One-off retrieval helpers (HTML-to-text, chat-payload extraction).
- Job launchers that hard-coded the session scratchpad.
- The virtualenv.

The owner's article, the X post and the shared chats are cited by URL in the memo.
