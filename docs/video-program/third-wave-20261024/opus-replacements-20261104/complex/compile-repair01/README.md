# compile-repair01 for complex_analysis_visual

A minimal repair of build02's 15 compiler errors, made from the complete
`complex-opus-rebuild-build02/command.log`. Nothing has been compiled: rustc is the only
real check of this patch, and root runs it.

The baseline is current canonical, which is increment06 except for root's formatting of
`thumbnail.rs` and `score.rs`. Neither file is touched here. `oliver-plan01` is untouched.

## Files

| File | What it is |
|---|---|
| `compile-repair01.patch` | Unified patch. From the repo root: `patch -p1 -i .../compile-repair01.patch` |
| `explainer_corpus/complex_analysis_visual/src/chapters/*.rs` | The nine patched files, whole |
| `baseline.json` | sha256 of the nine canonical files, the files read to make the repair, and the script |
| `check_compile_repair.py` | The text checks. `python3 -I -B check_compile_repair.py REPO EMPTY_SCRATCH` |
| `check-run01.txt` | The output of those checks and of the older desk checks |

## The repairs

| Log line | Code | Site in canonical | Repair |
|---|---|---|---|
| 7 | E0283 | `one_derivative.rs:431` | Drop `.collect()`: `polyline` takes `impl IntoIterator<Item = Vec3>` |
| 38 | E0283 | `one_derivative.rs:249` | The same |
| **not in the log** | E0283 | `one_derivative.rs:255` | The same, see below; rustfmt then puts the call on one line |
| 69 | E0308 | `closing.rs:199` | `Ok(... .build()?)`: `From<DashError> for fmn::Error` is at `fmn/src/lib.rs:1377` |
| 158 | E0308 | `the_bump.rs:199` | The same |
| 90 | E0308 | `draw.rs:800` `Walker::walk` | `stage.play(..)?; Ok(())` |
| 101 | E0308 | `draw.rs:633` `Painter::to` | The same |
| 113 | E0308 | `draw.rs:638` `Painter::enhance_to` | The same |
| 124, 141 | E0599 | `draw.rs:272`, `:283` | Import `explainer_kit::Timed`, one line |
| 188 | E0373 | `counting.rs:666` | `move`: the closure only needs the `f64` radius |
| 179 | E0515 | `closing.rs:411` | Test helper `get` becomes `fact(key).unwrap()` |
| 212 | E0515 | `two_planes.rs:652` | The same |
| 221 | E0515 | `turning.rs:440` | The same |
| 230 | E0515 | `gallery.rs:849` | The same |
| 239 | E0515 | `loop_sums.rs:596` | The same |

**The E0515 helpers.** `draw::fact` returns `&'static str` from the crate's existing `OnceLock`
around the same `crate::facts::facts`, so `get` keeps its signature and no caller changes. A
missing key still panics the test. No test, assertion, string or expected value changed:
`check_compile_repair.py` compares the counts and every string literal in the nine files.

**Line 255.** It has the same `polyline(... .collect())` form as line 249, in the same closure.
That the log omits it fits rustc reporting only the first ambiguity in a body. Leaving it would
have cost one more build.

## What the log cannot show yet

rustc does not borrow-check a body that failed type checking, so these nine bodies have had no
borrow check at all: `TheBump::construct`, `Closing::construct`, `OneDerivative::construct`,
`orientation`, `fade_in_all`, `fade_out_all`, `Painter::to`, `Painter::enhance_to` and
`Walker::walk`. Ambiguity errors in them may also have been suppressed.

I read each of them for moved values, closures reaching updaters by reference, stage borrows
held across a call, and other uninferable `collect`, `parse` or `into` calls. I found nothing.
`Frame2`, `Plane` and `Func` are `Copy`, which those bodies depend on (`plot.rs:80`,
`draw.rs:384`, `model.rs:356`). This is reading, not a borrow check, so rustc may still find more
there. If it does, send me the log and I will repair it in a new path.

## Later build steps, not reached and not repaired here

1. **`cargo test`.** None of the 95 tests has ever run. Failures there are the likeliest next stop.
2. **Clippy, one certain hit.** `turning.rs:483` has the literal `-3.14`. `clippy::approx_constant`
   is deny-by-default, so `-D warnings` will stop on it. It is left out because this patch is
   limited to the logged error classes and the line is a test bound. `-3.14` is the top of a
   one-line lower third, so the same bound can be written `-FRAME_Y_RADIUS + 0.2 + 0.66`,
   if the module has `FRAME_Y_RADIUS` in scope; that is unchecked.
3. **Clippy and `cargo build`, otherwise unknown.** Lints run only once the crate compiles, so
   none has ever been seen for this crate. Items used only by tests would be dead code in the
   plain `bin` build. `desk_check.py` and `unused_check.py` found none, but they read text only.

## Checks that ran

All are text checks or rustfmt. `check-run01.txt` has the output.

- The patch regenerates byte for byte, applies with `patch -p1` to a scratch copy of canonical's
  `src`, gives exactly the nine candidate files, and reverses back to canonical.
- The 15 error headers are at the listed log lines, and each has a repair row.
- A control confirms the scans for the old forms fire on canonical. They find none in any of the
  17 registered modules after the repair.
- Each of the 18 repair rows, undone alone, is refused.
- `rustfmt --check` (nightly-2026-08-31, the repo's `rustfmt.toml`) accepts the whole module tree.
  It was run on a scratch copy, because `cargo fmt` is root's.
- `scene_crosscheck`, `supplements`, `unused_check`, `arity_check`, `scope_check`, `shape_check`
  and `oracle.py` give the same results before and after, apart from shifted line numbers and five
  more calls counted.
- `desk_check.py` reports one new line, `UNUSED src/chapters/draw.rs Timed`. That is a false
  positive: `Timed` is a trait imported for `.rt`, and that text check looks for the name.
  `scope_check` still reports its expected false positives.

## Not touched

Canonical, increment06, `oliver-plan01`, Beads, `script.tsv`, all takes and every prior output.
The log's first line, the `unclosed table` error, comes from a fixture inside the asupersync git
checkout and did not stop the build, so it is not in the count of 15.

## Patch order

Apply this first. `reading-repair01` will be made against canonical plus this patch, and applied
after it.
