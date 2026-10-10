# Bakery-algorithm verification scripts

These scripts back every model-checked number and trace in `../MEMO.md`. They need pure Python 3 with no dependencies. Timings were measured with CPython 3.14.6 on an Apple M4 Pro.

**Running them:** use `python3 -I <path>/<script>.py` from any working directory. Each script adds its own folder to `sys.path`, so it works even under `-I`. Add `-B` if you don't want a `__pycache__/` left behind.

**Where results live:** saved runs are in `outputs/`.

**Relationship to the memo:** the files here are the authoritative versions. Memo Appendix A reproduces them, lightly condensed in A.3–A.5 (comments, the `fmt` helper, and the printing in `__main__`). The `sys.path` line was added when the scripts were moved here.

## Scripts

- **`bakery_mc.py`** — the core library, not run directly. It is the exhaustive BFS explorer for the Bakery algorithm with **atomic** or **safe** registers (a read overlapping a write returns any value in its domain), tickets capped at K. It covers five variants: `correct`, `no_choosing`, `strict_nums`, `le_nums`, `double_fetch`. It also provides the FCFS ghost check, Assertion-2 check, liveness (reachability of a CS entry), and `replay()` for step-validated traces.
- **`run_sweep.py [safety|liveness|fcfs|big]`** — exhaustive state-space sweeps. Reproduces memo §3.2 (the state-count table), the liveness lines, and the FCFS + Assertion-2 lines.
  - Runtimes: `safety` ≈ 3.5 min; `liveness` ≈ 6 s; `fcfs` ≈ 25 s.
  - `big` takes over 12 min and was not re-run here: N=4, K=4 atomic alone took 712 s, and the N=4, K=3 safe run never finished (stopped by hand).
  - Saved runs: `outputs/sweep_safety.txt`, `sweep_live.txt`, `sweep_fcfs.txt`, `sweep_big.txt`.
- **`scenarios.py`** — about 0.1 s. Step-by-step replays, where every step must be a legal transition: (a) no-choosing violation, (a′) the same schedule with choosing, (b1) the strict-`<` violation, (b2) the `≤` deadlock, (c)/(c′) equal tickets, (d) the 70-step four-process walkthrough, and (e1)–(e3) the safe-register garbage reads. Reproduces memo §3.3–§3.5. Saved run: `outputs/scenarios_out.txt`.
- **`leapfrog.py`** — about 0.1 s. Two processes never let the bakery empty; tickets climb 1, 2, 3, 4, 5, 6 (memo §3.4, "Leapfrog"). Saved run: `outputs/leapfrog_out.txt`.
- **`df_trace.py`** — about 3 s. Double-fetch max: OK for N=2 safe (K = 6 and 9; 2,224 and 4,597 states). For N=3 safe it violates mutual exclusion, with a 33-step shortest trace (memo §3.5, §3.2). Saved run: `outputs/df_trace_out.txt`.
- **`bakery_tso.py [--n3]`** — about 5 s by default. An x86-TSO store-buffer model with an MFENCE after `choosing:=1` (F1) and/or after `choosing:=0` (F2). It reproduces memo §3.6 for N=2, K=4: no fence fails (12 steps); F1 only fails (18); F2 only fails (20); F1+F2 is OK (1,135 states). `--n3` adds the unfenced N=3 runs, which did not finish within 10 minutes and were not re-run. Saved run: `outputs/bakery_tso_out.txt`.
- **`tso3.py`** — about 5 s. The fenced TSO runs: N=2, K=6, B=4 gives 1,811 states, OK; N=3, K=3, B=3 gives 37,580 states, OK (memo §3.6). Saved run: `outputs/tso3.txt`.
- **`tree_count.py`** — about 1 s. Exact schedule-tree counts by memoized DP (memo §3.7, §7): lost update 6/4; naive flag 14/8; N=2 bakery 28/4 without choosing and 322/0 with it; N=3 792,987/312,492 without and 2,126,816,238/0 with. Saved run: `outputs/tree_count_out.txt`.
- **`crosscheck.py`** — about 0.2 s. Independent brute-force enumeration of the N=2 one-shot counts, used to cross-check `tree_count.py`. Saved run: `outputs/crosscheck_out.txt`.
- **`quick.py`** — under 1 s. A smoke test: N=2, K=4 for all four main variants, atomic and safe, with liveness and Assertion 2. Saved run: `outputs/quick_out.txt`.
- **`quick3.py N K atomic|safe VARIANT`** — a single exploration, e.g. `3 4 safe correct` (≈ 9 s, 92,716 states, OK).
- **`extract.py PAGE.html OUT_RSC.txt OUT_BODY.txt`** — under 1 s. A parsing helper that pulled the text out of the owner's article HTML for memo §1. The downloaded page itself is deliberately not kept here.
- **`comma_operator_check.js`** — `node comma_operator_check.js`, instant. Shows that `(t[j], j) < (t[i], i)` evaluates to `j < i` in JavaScript (memo §0 item 9, §1).
- **`owner_snippet_lock.ts`** — the article's `lock()` snippet. `tsc --noEmit --strict owner_snippet_lock.ts` (tsc 5.9.3) reports nothing, which supports the claim that the compiler does not flag the comma operator here.
