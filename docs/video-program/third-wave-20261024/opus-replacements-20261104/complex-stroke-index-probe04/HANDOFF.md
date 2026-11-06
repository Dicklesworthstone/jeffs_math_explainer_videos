# Stroke probe 04

RubyRaven, for root, answering Mail 5906. **Nothing here has been compiled, and no speed is measured.** Earlier versions and receipts stay as they are, and the patch is unchanged in `complex-stroke-index-candidate01`. `complex-stroke-index-probe02/HANDOFF.md` still governs the commands, risks and limits, and `complex-stroke-index-probe03/HANDOFF.md` the jobs gate. Read `probe04.sh` for the older script names.

## Launch change needed: drop `--foreground` from the outer limit
Use plain `/usr/bin/timeout N` for `ab64`, `ab4k`, `film` and `profile`. Mail 5907 says the outer limit now carries `--foreground`, and on expiry that stops only the direct child. The job then exits 124, the guard treats it as finished, and the render carries on unguarded after the lane is released. That is White's F1 again, by a different route.

Each step started as the guard starts a job, with a sleeper as the render:

| Outer command | Guard stop (`killpg`) | Outer limit expires |
|---|---|---|
| `timeout --foreground`, `probe03.sh` | render survives | not tried |
| `timeout --foreground`, `probe04.sh` | render stops | **render survives** |
| plain `timeout`, `probe04.sh` | render stops | render stops |

Plain `timeout` still stays in the guard's group here: the guard makes it a session leader, so it can't move into a group of its own.

If baseline `build01` is under `--foreground` and its limit expires, `cargo` may keep running in the same way. Check for a survivor before the next job. A guard stop is not affected.

## What changed in `probe04.sh`
- The inner `timeout` is gone from `ab`, `film` and `profile`, and nothing in the script sets a time limit now. The outer command is the only one, so set it per step. The old inner limits were 6 h for `ab`, 12 h per side for `film`, which now runs serially, and 1 h for `profile`.
- `film` now needs a non-comment line in the baseline `framemd5`, so two header-only decodes can't pass. An empty file is refused the same way, and the `cmp` that follows holds the candidate to the same records.
- Version text changed in the header, usage line and refusal prefix.

Gates, flags and every build line are byte-identical to `probe03.sh`, so a `base` built by `probe03.sh` is valid for `cand` here.

## Running it
Stage `complex-stroke-index-probe04/` beside `complex-stroke-index-candidate01/`. Keep the same `WORK`. Do not rerun `prepare`, `deps` or a finished `base`. After `build01` ends, run `cand`, then `ab64`, then `ab4k`, as guarded jobs with no duplicates. `cand` needs `$WORK/bin/base`, the later steps need both binaries, and `ab64` and `ab4k` need `ab_bench.sh` staged.

## Evidence
All of this is on fake tools, run on this Mac with GNU `timeout` 9.12. Nothing here ran on csd.

| Check | Result | What it does not show |
|---|---|---|
| The table above, on `ab64` and `film` | as shown; `probe03.sh` reproduced F1 first | real render or csd behaviour |
| `ab64` and `film` with no inner timeouts | `IDENTITY PASS`, `FILM IDENTITY PASS: 15 chapters` | a real render |
| Header-only decodes on both sides | `probe04 refused: no decoded frames` | |
| `timeout` followed by a number in `probe04.sh` | 0 lines | |

`profile` shares the same shape but was not run, because this Mac has no `perf`.
