# Secondary frame budgets: build commands

For root's guarded native run. Nothing here has been compiled or run. Only the apply and hash steps in section 1, and rustfmt, have been exercised, on copies.

## 0. Base

The patch applies on root's staged tree from validation02, which includes root's frame-clock repair to `native_render.rs` (`d64416e4536cde9a0fbfa7e3e87ac488a36b0b9723c789bb304a68a818ac70cd`). That repair isn't in a patch file, and the hashes below assume it. The new tests sit away from its hunk, so the patch also applies to the plain stack without it, but `native_render.rs` then hashes differently.

Apply to a new copy, outside any git work tree, and leave the staged tree as it is.

```sh
REPO=<the repo root>
S="$REPO/docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-budget01"
ENGINE=<a new copy of the staged engine folder, outside the repo>
export CARGO_INCREMENTAL=0 RCH_SHIM_LOCAL_IDE=1
```

## 1. Apply (replayed on copies, see `checks/patch-replay.out`)

```sh
(cd "$S" && shasum -a 256 -c bundle.sha256)
cd "$ENGINE"
git apply "$S/engine-frame-table-limits.patch"
shasum -a 256 -c "$S/expected-engine.sha256"
```

## 2. Native checks (not run)

Under your guard, with your own target directory. Run them in this order, so the cheap ones fail first.

```sh
rustfmt --check --edition 2024 crates/fmn-render/tests/frame_table_limits.rs crates/fmn/tests/native_render.rs crates/fmn/tests/native_bundle_pipeline.rs
cargo clippy --locked --offline -p fmn-render -p fmn --all-targets -- -D warnings
cargo test --locked --offline -p fmn-render --test frame_table_limits
cargo test --locked --offline -p fmn --test native_bundle_pipeline
cargo test --locked --offline -p fmn --test native_render
cargo test --locked --offline -p fmn-render
cargo check --locked --offline -p fmn-cli --all-targets
```

- `native_render` gains a 1080p test that bins about 16.8 million references: one refused pass and two admitted frames. It needs about 340 MB by arithmetic. Its time is unmeasured, and the tests are unoptimised.
- Plain `cargo test` skips the ignored test below.

## 3. The ignored control (not run)

Root's call whether and when to run it. It holds about 0.51 GB by arithmetic.

```sh
cargo test --locked --offline -p fmn-render --test frame_table_limits -- --ignored a_monotone_budget_above_the_default
```

## 4. Stop table

| Result | Meaning | Do |
|---|---|---|
| Apply or hash fails | The base isn't the tree in section 0 | Stop and compare `native_render.rs` to the hash above |
| Compile or Clippy error | My source is wrong: none of it has been compiled | Send it back. Don't relax a lint |
| `monotone_budgets…` panics at `refused_in_frame` | The frame-local site isn't wired | Send it back |
| `monotone_budgets…` panics with other counts | The demand arithmetic is wrong | Send the counts back. Don't widen the test |
| `binning_budgets…` reports another resource or count | The binning order was misread | Send it back |
| `a_declared_binning_budget…` refused by `working bytes` | The working-bytes arithmetic is wrong | Send the requested value back |
| `a_declared_binning_budget…` frame count isn't 2 | The frame clock differs from root's repair | Send it back |
| An `AllocationFailed` refusal | A named memory guard worked | Note it. It isn't a pass |
| Resource stop from your controller | The estimates above were low | Stop and report the peak |

## 5. Which test catches which mis-wiring

From reading the tests. No mutant was built or run.

| If this were wrong | This should fail |
|---|---|
| `engine.rs` frame-local site left at the default | `monotone_budgets_govern_the_shared_table_and_the_frame_local_pieces` |
| `render_cached_into` table or binning left at the default | `monotone_budgets…`, `binning_budgets…` |
| `capture` doesn't carry the limits | Every `frame_table_limits` test except the defaults |
| Serial compiler not set in `pipeline.rs` | `a_lowered_frame_table_budget_refuses_by_name_and_publishes_nothing` |
| Compiled-worker compiler not set in `pipeline.rs` | `declared_frame_tables_reach_the_compiled_affine_workers_and_not_the_camera_route` |
| A raise clamped to the default | `a_declared_binning_budget…`, and the ignored monotone control |
| A default value changed | `the_default_budgets_are_the_limits_the_engine_always_had` |

## 6. Not covered

A real dense or moving scene through these budgets: its frame time, its actual peak memory, and the full route to an encoded chapter. Those need a film-level probe, which root would choose.
