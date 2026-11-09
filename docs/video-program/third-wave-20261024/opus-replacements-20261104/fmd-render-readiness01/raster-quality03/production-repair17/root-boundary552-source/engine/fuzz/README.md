# fuzz/ — coverage-guided fuzzing of the untrusted-input parsers (fm-ntp, fm-tdp)

Class-`fuzz` tooling under ADR-0003: this crate is **not a workspace
member** (its own empty `[workspace]` table keeps it out of the root
build graph). It depends on the crates under test (`fmn-codec`,
`fmn-cache`, `fmn-hash`, `fmn-geom`, `fmn-config`, `fmn-tex`,
`fmn-library`, `fmn-text`, and `fmd-font`/`fmd-math` at the SUITE.lock
revision) plus a pinned `libfuzzer-sys`. Its `Cargo.lock` is committed
and walked by the governed-closure check
(`crates/fmn-conformance/tests/governed_closure.rs`). It runs as a
**scheduled CI job, never a merge gate** (`scripts/check.sh` does not
call it).

## Targets

Ten coverage-guided harnesses over the §14.2 untrusted-input surfaces.
Each asserts the contract (§16.5/R14): for any input, refuse with a
typed error **or** succeed inside the declared budget — never panic,
never hang, never overallocate.

| Target | Entry point | Assertions beyond totality |
|---|---|---|
| `inflate_bytes` | `fmn_codec::inflate_bytes(data, MAX_OUTPUT)` | decompressed output `<= MAX_OUTPUT` (1 MiB campaign cap) |
| `decode_png` | `fmn_codec::decode_png(data, &PngLimits{..})` | accepted `width*height <= max_pixels` (4 MP) and `rgba.len() == width*height*4`; `max_chunks = 512` |
| `decode_jpeg` | `fmn_codec::decode_jpeg(data, &JpegLimits{..})` | accepted `width*height <= max_pixels` (4 MP) and `rgba.len() == width*height*4` |
| `decode_entry_envelope` | `fmn_cache::decode_entry_envelope(envelope, kind, &address, Limits::DEFAULT)` | an accepted payload is no longer than its envelope |
| `namespace_and_object_paths` | `fmn_cache::validate_namespace_name`, `fmn_cache::object_relative_path` | accepted names are 1..=64 bytes of `[a-z0-9_-]`, alphanumeric first; object paths are exactly two lowercase-hex components that join to the digest |
| `svg_document` | `fmn_geom::svg::SvgDocument::parse_with_limits(data, &SvgLimits::default())` | totality only |
| `yaml_config` | `fmn_config::yaml::parse_with_limits(text, Limits::default())` (UTF-8 inputs) | totality only |
| `tex_math` | `fmd_math::parse(text)` (UTF-8 inputs) | totality only |
| `ttf_font_parse` | `fmd_font::Font::parse(bytes)` | totality only |
| `path_boolean` | `fmn_library::boolean_ops::{union, difference}` (and `_with_options`) over two paths built from f32 point triples | totality only |

Hangs are policed by libFuzzer's `-timeout` and allocation by
`-rss_limit_mb`, besides the budget assertions above. Resolved findings
keep their reproducer and write-up under `fuzz/findings/` (the
2026-08-30 SVG UTF-8 boundary panic, with regression tests in fmn-geom).

## Running locally

With the cargo-fuzz toolchain (the primary, coverage-instrumented path):

```sh
cargo install cargo-fuzz --version 0.13.1   # not --locked: see CI below
cargo fuzz run inflate_bytes        # Ctrl-C to stop
cargo fuzz run decode_png   -- -max_total_time=300
cargo fuzz run decode_jpeg  -- -max_total_time=300
```

The whole campaign, exactly as CI runs it (60s per target by default;
knobs documented in the script header):

```sh
bash scripts/fuzz_scheduled.sh
FMN_FUZZ_SECONDS=600 bash scripts/fuzz_scheduled.sh   # longer session
```

If `cargo-fuzz` is not installed, the script falls back to
`cargo build --release --manifest-path fuzz/Cargo.toml --target-dir fuzz/target`
and drives the libFuzzer binaries directly — degraded guidance (no
coverage flags), but the budget assertions, hang timeout, rss cap, and
corpus replay all still hold.

## CI

`.github/workflows/ci.yml`, job `fuzz-codec`, `if: github.event_name ==
'schedule'` (the weekly Monday cron shared with the other scheduled
jobs): installs the pinned toolchain, `cargo install cargo-fuzz
--version 0.13.1` (not `--locked`: cargo-fuzz 0.13.1's own lockfile pins
a rustix the pinned nightly refuses to compile), then
`bash scripts/fuzz_scheduled.sh`. Any crash fails the job; the reproducer
lands in `fuzz/artifacts/` (and is logged base64 by libFuzzer) — that is a
**finding**, a real bug to fix at the source in the crate under test,
never a reason to weaken a harness. The lane is skipped on push events by
design.

Longer recorded campaigns, with per-target tables, live in
`fuzz/campaigns/` (for example `fuzz/campaigns/2026-09-29.md`).

## Corpus policy

- `fuzz/corpus/<target>/` is committed. Seeds are the interesting inputs
  copied from `crates/fmn-codec/tests/fixtures` (every PNG/JPEG fixture,
  plus the DEFLATE/zlib streams) and coverage-grown units from completed
  runs.
- Grow it deliberately: after a long local session, minimize before
  committing — `cargo fuzz cmin <target>` — and prefer a small corpus of
  high-coverage units over bulk.
- Crash reproducers never go in the corpus; they are fixed at the
  source, then a regression fixture lands in the fmn-codec test suite.
- `fuzz/target/` and `fuzz/artifacts/` are gitignored build/reproducer
  sinks; `fuzz/Cargo.lock` is committed and governed (class=`fuzz` rows
  in `SUITE_ALLOWLIST.tsv`).
