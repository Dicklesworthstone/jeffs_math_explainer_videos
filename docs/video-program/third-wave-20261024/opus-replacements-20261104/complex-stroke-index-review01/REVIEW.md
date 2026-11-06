# Stroke index review 01

WhiteSwan, for root (Mails 5826, 5845-5847, 5875, 5887). This is a review of `complex-stroke-index-candidate01/stroke-index.patch` and `complex-stroke-index-probe03/probe03.sh`, read against the engine at pin `a399d626` and root's real guard. **Nothing was compiled, and nothing ran on csd. No speed is claimed.** I had read Ruby's identity argument (Mail 5778) before reading the source, so this isn't a blind review.

## Verdict
- **Patch:** I found no correctness defect, and it needs no change.
- **`probe03.sh`:** `prepare`, `deps`, `base` and `cand` can go ahead.
- **Before `ab64`:** fix F1, or accept the manual check below. The steps it affects are `ab64`, `ab4k`, `film` and `profile`.

## Findings

### F1. High: a stop doesn't reach a render started under the inner `timeout`
`probe03.sh` lines 185, 207 and 228 start the work under their own `timeout`. GNU `timeout` moves itself into a new process group. The guard's `stop_owned` signals only the child's group, and root's outer `/usr/bin/timeout` signals only its own, so neither reaches that new group.

A resource stop or an outer timeout therefore ends the job and releases the exclusive lane while the render keeps running, writing to disk with no guard for up to 6 hours (`ab`) or 12 hours (`film`). The next job would then be timed against that render.

`checks/outputs/stop-reach.txt` shows this with the guard's process shape, on GNU `timeout` 9.12 on this Mac:

| Inner command | Guard stop | Outer `timeout` expiry |
|---|---|---|
| `timeout 21600 bash ...` (probe03) | **render survives** | **render survives** |
| no inner `timeout` | render stops | render stops |
| `timeout --foreground 21600 bash ...` | render stops | not tested |

**Fix, for root to decide:** delete `timeout 21600`, `timeout 43200` and `timeout 3600` from those three lines, and let root's outer `/usr/bin/timeout` be the only time limit on each step. I don't recommend `--foreground` for `ab`, because the command it would time is a shell whose children, by its manual, aren't timed out.

The change needs no rebuild, because the `ab` steps read the existing `$WORK/bin/base` and `$WORK/bin/cand`.

**If root keeps `probe03.sh`:** after any stop or timeout on those four steps, run `pgrep -af "$WORK/bin/(base|cand)"` before starting anything else, and end only the survivors.

### F2. Low: one environment check would avoid more spent job names
`compgen -e` is sorted on both bash versions I tried (3.2.57 and 5.3.9), so the csd refusals stopped the loop at `CARGO_BUILD_JOBS`. `CARGO_HOME`, `CARGO_PROFILE_*`, `RUSTC*` and `RUSTFLAGS` sort later, so the gate hasn't yet reached them on csd. Since that shell exported jobs 16, it may export others.

Running `printenv | grep -E '^(CARGO|RUST)'` once, in the shell that starts the guard, shows them all at once.

### F3. Low: the refusal list has gaps
It doesn't cover `CARGO_TARGET_<triple>_*`, `CARGO_UNSTABLE_*` or `RUSTC_BOOTSTRAP`. If both sides see the same environment, these can't separate `base` from `cand`, but they could separate both from production. The F2 check would show any of them.

### F4. Low: a rerun of an `ab` step truncates its log
`tee` empties `$LOGS/$name.log` before `ab_bench.sh` refuses the non-empty folder. Under the guard, the earlier job's `command.log` keeps the same text, so nothing is lost. Run outside the guard, it would be.

### F5. Low: `film` could pass on empty decodes
`film` checks `[ -s ]` on a `framemd5` file, which a header alone satisfies. `ab_bench.sh` counts frame lines and `film` doesn't, so two empty decodes would compare equal. An exit 0 with no frames is unlikely, and the step is optional, but this should be fixed before `film` counts as evidence.

## Guard environment against the helper (Mail 5887)
Source: the guard's lines 140-142. `guard_csd01.py` changes none of these.

| Name | Value | In `probe03.sh` |
|---|---|---|
| `CARGO_BUILD_JOBS` | `2` | Required and kept. Ruby's version also refuses unset, which is stricter than my 5886 proposal, and I agree with it. |
| `CARGO_TARGET_DIR` | the work tree | Overridden to `$WORK/target`, and to `$WORK/target-test` for tests. |
| `RUSTUP_TOOLCHAIN` | the pinned nightly | Same toolchain, and `rustup which` names it explicitly. As I recall cargo's source, it uses this only when `rustup` is on `PATH`. |
| `RAYON_NUM_THREADS` | `4` | Doesn't set render threads. The film's lock shows only `fsci-linalg` and `fsci-sparse` using `rayon`, and no engine `Cargo.toml` names it. `--threads` or `available_parallelism` sets the count. |
| `RCH_SHIM_LOCAL_IDE` | `1` | Re-exported. The shim isn't on `PATH`. |
| `FW_ENC_ATTN_OUT_I8I32` | `0` | Not read by any engine crate I searched. Its name suggests a franken_whisper setting. |

The guard passes everything else through from its parent, which is why F2 matters.

**Real csd evidence:** the environment loop is the last check in `gates()`, and `prepare02` was refused there. By that order, csd has already passed the toolchain folder, `command -v` for cargo and rustc, both SHA pins, the rustc commit and the home config pin.

## What held

**Pixel equivalence, from `stroke.rs`:**
- `nearest` keeps nothing between iterations except `best`, `best_s` and `admitted`.
- A skipped segment would have hit `continue`, and kept segments run in the original order, so `excess < best` keeps the same earliest winner and the same `s` bits.
- `shade` reaches the segments only through `nearest`, and an empty run returns `None` exactly where the scan's `admitted` stays false.
- The aggregate slab is the min and max of the same segment slabs, so the y early return is sound once every slab is finite.
- The proof needs only IEEE comparisons, so `-0.0` and tied edges change nothing, and it doesn't need `left <= right` on a slab.
- A `NaN` or infinite `p` takes the full range.

**Independent logic mirror (`checks/logic_mirror.py`):** 3,000 random sorted strokes and 515,349 queries, with signed zeros, ties, one-ulp neighbours and non-finite coordinates. The run equals the scan's admitted set, in order, on every query, and all 4 planted refusals are refused. This checks the selection logic only, not Rust or the solver.

**No missed paths:**
- `StrokeRef` is built in one place, and `from_parts` has one production call, which the patch changes.
- All three `stroke_of` callers get the index.
- Nothing outside `stroke.rs` and `engine.rs` in `fmn-render` builds these types; `arena.rs` only mentions one in a comment.
- `row_span` is unchanged.
- No test pins `size_of` for these types.

**`IDENTITY PASS` can't come from reused pixels.** The only production use of the persistent store I found is typesetting, through `TexSession`. Sharing `$WORK/cache` between `base` and `cand` can't replay frames.

**Source identity:** the patch applied here to the pinned files gives `668867dffefe1378…` and `de6551fc9f697b4a…`, equal to `probe03.sh`'s pins.

**Test source:** by reading only, every engine API the five tests use that I looked up exists, with those arguments and traits. That includes `Segment` and `Mob` being `Copy`, `AaStats: Debug`, `ScreenMap::y_up`, `set_anti_alias_width` and `Subcell`. `index.x_candidates(middle)` should stay under its limit of 17: about 9 segments at the tightest map, using `STROKE_WIDTH_CONVERSION = 0.01` from `fmn-core`.

**Chapter 04 timing:** the plate is painted in cue `04_1`, in the chapter's first beat, so the `--max-frames` runs should reach it. I read the order of the code and didn't measure a frame.

**Resources:** the guard's 4 GiB per job is the sum of `jobs/` and work-tree growth. The frozen inputs are about 45 MiB, so `prepare` is small. If the guard stops `base` or `cand`, a rerun gets a fresh 4 GiB window and cargo keeps finished units, as Ruby says. The 20 GiB total still applies. The build sizes are unmeasured.

## Not established
- Whether the patch compiles, or whether any test passes. That includes the fixture's `ssaa2x_cells + ssaa4x_cells > 0` assertion, which I couldn't settle by reading.
- Whether csd's offline caches are complete.
- Whether identity survives a ±1 difference that rounds away in RGB-to-YUV. `framemd5` sees decoded YUV, so only the patch's tests see the raw coverage bits.
- Any speed, or any production decision. `ab64` and `ab4k` cover chapter 04 only.
- That my `timeout` test behaves the same on csd's coreutils, though the behaviour matches how I remember the source and the manual.

## Checks
From the repository root, with any empty scratch folder:

```
python3 -I -B checks/orphan_test.py <scratch>
python3 -I -B checks/outer_test.py <scratch>
python3 -I -B checks/logic_mirror.py 3000
```

Their output is in `checks/outputs/`.
