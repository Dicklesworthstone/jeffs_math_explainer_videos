# Stroke x index candidate 01

RubyRaven, for root, answering Mail 5750. **Nothing here has been compiled, and no speed is measured.** The frozen engine, the live job and production are untouched. This is an isolated candidate; adopting it is a separate decision.

## The problem
Root's annotation (Mail 5786) puts the time inside `PreparedStroke::shade` on the slab compares and the scan loop, with 0% shown in the solver. Reading the source shows why it is so large.

`composite_pixel_supersampled` (`engine.rs:2561`) visits `samples² × every draw binned to the tile × every segment`, and `stroke_sample` has no slab test. The film's `aa_px = 0` makes `effective_aa_band` 1e-8, so every strip stays complexity-eligible and the saturated shortcut at `engine.rs:2522` cannot fire.

## The change
The patch is `stroke-index.patch`: two files of `fmn-render`, 78 production lines added and 3 removed in `stroke.rs`, 12 added and 4 removed in `engine.rs`, plus 5 tests. It adds one `bool` per draw, with no arena pool and no allocation.

- **Eligibility:** `segments_are_x_sorted` needs at least 32 segments, finite slabs, and non-decreasing left and right slab edges. Every other stroke runs today's scan.
- **`x_candidates`:** for an eligible stroke, a `p` outside the aggregate's y-range returns nothing. Otherwise two `partition_point` searches give the contiguous run the x half of the slab test admits.
- **`nearest`:** the loop body is unchanged, including its own four-sided slab test. It only runs over that run, in shape order.

**Why the output cannot move:** every skipped segment would have hit `continue` with no state change. The rest run in the original order through the same body, so `excess < best` keeps the same earliest winner and the same `s` bits. A `NaN` query coordinate keeps the whole range, because the scan admits it everywhere. The proof uses slab values only, so the geometry behind them does not matter.

**Left alone on purpose:** `row_span`, AA, density, classification, fills and tile sizing.

| File | Frozen | Candidate |
|---|---|---|
| `crates/fmn-render/src/stroke.rs` | `e0feeab4dfbc0ead…` | `668867dffefe1378…` |
| `crates/fmn-render/src/engine.rs` | `85bdc6b721cf40fb…` | `de6551fc9f697b4a…` |

### Why not backport upstream
Upstream fixed nearby problems after our pin `a399d626`:
- `2a384ff2` adds an aggregate-slab skip in `stroke_sample`, and it landed with a self-measured A/B. This patch's y early return does the same job inside `nearest`, for all three callers, and refuses non-finite slabs.
- `fe382ffc` adds tile-binned CSR indexes. It sits on the retained-frame merge, so I don't expect a clean apply (not attempted). A 16 px tile also holds about half a strip's curves at 64×36.

**Correction to Mail 5778:** I said `fe382ffc` "allocates per draw". It builds its index in arena pools, which don't allocate once warm. It does still do per-draw index work this patch avoids.

## Evidence so far
| Check | Result | What it does not show |
|---|---|---|
| `rustfmt --check` on both candidate files | clean; both parse | that they compile |
| Patch on pristine frozen bytes | forward equals the candidate; reverse equals frozen | |
| `checks/mirror_x_index.py`, real `blend_points` geometry with f32 records | PASS: 948 of 948 strips across two plates and two resolutions accepted; 60,771 points visit the same segments in the same order; 240 random strokes, 119 accepted; 7 of 7 plants caught | Rust behaviour; it mirrors the logic only |
| `probe01.sh` rehearsal with a fake `rustup` and film binary | all steps but `profile` ran, using the real `ab_bench.sh` and real patch; wrong order, re-runs and planted output differences all refused | a real build or render |
| Root's source read of the patch core (Mail 5801) | no objection | tests |

The patch has 5 new Rust tests, none run yet:
- bit-exactness on a strip across all three kernel routes, at every slab edge ±1 ulp
- earliest of tied segments wins
- refusals: short, reversed, swapped, mirrored, NaN and infinite strokes
- non-finite queries
- a film-like plate that must reach SSAA, compared with the scan at every sample position

**Comparison model, from the mirror. These are not times:**

| Slab comparisons per plate frame | 64×36 | 3840×2160 |
|---|---|---|
| Today | 9.77e8 (92% in SSAA) | 2.78e10 (49% in SSAA) |
| With the index | 3.88e6 | 5.64e8 |
| Unchanged `row_span`, share of what is left | 29% | 35% |

**Ceiling:** root's single 3 s sample has `shade` at 74.91%, so removing all of it gives at most about 4× overall, about 0.8 fps at 64×36. The other 25% (SHA256, repaint, preparation) is untouched. The real figure could be lower, and it can only come from the `ab64` step.

## Running it on csd
Root owns transfer, launch and any guard around each step. `probe01.sh` writes only under `$WORK`, and pins every hash above plus `ab_bench.sh` (`9a3e6084…`), so bring that file from the franken-manim skill's `scripts/`.

```bash
export WORK=<new empty folder on the data disk> AB=<path>/ab_bench.sh
export SOURCE=<frozen folder holding engine/, repo-view/, engine-budget.toml>
export CRATE=<stage07 complex_analysis_visual folder>
export CONFIG=<repository .cargo/config.toml>
bash probe01.sh prepare   # copies, rewrites 2 absolute-path files, proves offline dependencies
bash probe01.sh base      # release build, unpatched
bash probe01.sh cand      # patch, fmn-render lib tests, release build with the same flags
bash probe01.sh ab64      # alternating timing, 3 rounds, then lossless identity
bash probe01.sh ab4k      # the same at 4K; THREADS defaults to 24
```

Optional:
- `bash probe01.sh profile` shows what is hot next. It needs `perf`, and it was not rehearsed.
- `bash probe01.sh film` runs lossless identity on all 15 chapters at 64×36, 1 fps, both binaries in parallel.

**Before `prepare`, please check on csd:**
- `prepare`'s `cargo metadata --offline` is the gate for the dependency caches. If it fails, csd's cargo registry and git caches need filling, for both the film and engine workspaces. The film build and `fmn-render`'s tests resolve through different lock files.
- `rustup run nightly-2026-08-31 cargo` must resolve inside the toolchain. `prepare` refuses otherwise, so the RCH shim can't take the build.

**Build and flag choices:**
- Both binaries use production's flags and one target folder, built in sequence. Only `fmn-render` and its dependents rebuild for the candidate.
- Identity uses `--crf 0`. Lossy x264 could hide a ±1 LSB difference; lossless decoded `framemd5` cannot.
- 64×36 timing uses 3 fps and `--max-frames 120`, the route of `complex-film-cells4-routes01`, so its baseline is comparable. Repaints per frame are higher at 3 fps than in the 60 fps timeline, so `shade`'s share may differ from your sample. `profile` runs at 60 fps for that reason.

## Reading the results
Stop at the first failure:

| Step | Pass | If it fails |
|---|---|---|
| `cand` | compiles; all 5 new tests and all existing `fmn-render` tests pass, including the pinned `(29, 34)` and `13_204` AA work measurements | send me the log; I repair in a new folder |
| `ab64`, `ab4k` | `IDENTITY PASS` | **candidate rejected at that resolution**; send both `framemd5` files; do not use it |
| timing | quote only `mean base`, `mean cand`, and their ratio from the log, for that command on that host | a small ratio means the rest of the frame dominates; run `profile` before any more engine work |

Passing these still doesn't justify a production engine:
- `ab64` and `ab4k` exercise chapter 04 only. Run `film` first, because any other stroke of at least 32 sorted segments also takes the index.
- A new engine means new source pins, a new binary and a new qualification, so it needs a new `qualify`/chain decision. None of that is prepared here.
- No listening, picture or AV claim follows from any of this.
