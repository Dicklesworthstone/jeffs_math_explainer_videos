# Stroke probe 03

RubyRaven, for root, answering Mail 5887. **Nothing here has been compiled, and no speed is measured.** `probe02.sh` and its receipts stay as they are, and the patch is unchanged in `complex-stroke-index-candidate01`. `complex-stroke-index-probe02/HANDOFF.md` still governs the commands, risks and limits; read `probe03.sh` for `probe02.sh` there.

## What changed
Only the jobs gate, plus the version text in the header, usage line and refusal prefix:
- `CARGO_BUILD_JOBS` must be exactly `2`, the value the guard sets. Any other value is refused, and so is unset, because unset would mean cargo's default parallelism outside the guard.
- The variable is left in place, so cargo inherits the guard's limit.
- Every other `CARGO_BUILD_*` variable, and every other refused name, is still refused.

Because of the unset rule, gated steps (`prepare`, `deps`, `base`, `cand`) now run only under the guard or with an explicit `CARGO_BUILD_JOBS=2`. Do not wrap them in `env -u CARGO_BUILD_JOBS`.

## Running it
Stage `complex-stroke-index-probe03/` beside `complex-stroke-index-candidate01/`, because the script reads the patch from that sibling. The commands are probe02's with `probe03.sh`. A `work/` left by prepare02 and prepare03 is fine: both stopped before any copy, so there is no `tree/` to refuse.

## Evidence
| Check, using the guard's six injected variables | Result | What it does not show |
|---|---|---|
| `probe02.sh` under them | your exact `CARGO_BUILD_JOBS` refusal | |
| `probe03.sh` under them, `prepare` | passes the gates and copies | real dependencies |
| Jobs `16`, jobs unset, `CARGO_BUILD_RUSTFLAGS`, `RUSTFLAGS` | each refused, no `tree/` | |
| `deps`, `base`, `cand` under them | fake cargo saw jobs `2` on every call | a real build |

These ran on the existing fake tools, in the same shape as probe02's rehearsal. The real test is your guarded `deps` and build on csd. The cargo, rustc and home-config pins are still your reported values.
