# WASM-target audit of the governed closure (fm-7wm.4, R15)

**Status:** re-run against `Cargo.lock` `d5593dcf` and `SUITE.lock`
`e4758123`, for the franken_markdown repin from `3ed1d332` to `126a5304`
(fm-15k9, fm-tex-layout-oracle-bkbc). No package version changes in this graph.
**The release package gate still FAILS at its size budget** (fm-8j70), so this
audit does not claim a passing package.

- **VERIFIED (mechanical):** `cargo tree -p fmn-wasm --target
  wasm32-unknown-unknown --edges normal --locked` is unchanged at 134 lines with
  no fmd or franken_markdown package. The lock change moves only the three
  franken_markdown packages' rev, none of which is in that graph.
- The `--locked` wasm32 build and wasm-smoke were not re-run: no package in the
  wasm graph changed since the run below.

Earlier re-run, 2026-10-09, against `Cargo.lock` `0d6dfdaa` and `SUITE.lock`
`af634982`, for the franken_markdown repin from `bd00a247` to `3ed1d332`
(fm-tex-layout-oracle-bkbc, fm-aia1): the same mechanical check, unchanged at 134 lines.

Earlier re-run, 2026-10-09, against `Cargo.lock` `726da0b5` and `SUITE.lock`
`a6e3520b`, for the franken_markdown repin from `798de11b` to `bd00a247`
(fm-tex-layout-oracle-bkbc): the same mechanical check, unchanged at 134 lines.

Earlier re-run, 2026-10-09, against `Cargo.lock` `7fcc7866` and `SUITE.lock`
`4385c9af`, for the franken_markdown repin from `e4e8e2ee` to `798de11b`
(fm-tex-layout-oracle-bkbc): the same mechanical check, unchanged at 134 lines.

Earlier re-run, 2026-10-09, against `Cargo.lock` `1e9b139f` and `SUITE.lock`
`d7608aa8`, for the franken_markdown repin from `d664c138` to `e4e8e2ee`
(fm-tex-layout-oracle-bkbc): the same mechanical check, unchanged at 134 lines.

Earlier re-run, 2026-10-09, against `Cargo.lock` `c88db1db` and `SUITE.lock`
`1a4972d1`, for the franken_markdown repin from `a8aab0db` to `d664c138`
(fm-tex-layout-oracle-bkbc): the same mechanical check, unchanged at 134 lines.

Earlier re-run, 2026-10-08, against `Cargo.lock` `ce8fdfe2` and `SUITE.lock`
`565f56fe`, for the franken_markdown repin from `a3578a0f` to `a8aab0db`
(fm-binom-delimiters-glyph-gaps-ed1k): the fmd-math drawn paren and `\tbinom`/`\dbinom`. No
package version changes in this graph.
**The release package gate still FAILS at its size budget** (fm-8j70), so this
audit does not claim a passing package.

- **VERIFIED (mechanical):** `cargo tree -p fmn-wasm --target
  wasm32-unknown-unknown --edges normal --locked` is unchanged at 134 lines with
  no fmd or franken_markdown package. The lock change moves only the three
  franken_markdown packages' rev, none of which is in that graph.
- The `--locked` wasm32 build and wasm-smoke were not re-run: no package in the
  wasm graph changed since the run below.

Earlier re-run, 2026-10-07, against `Cargo.lock` `b40a0537` and `SUITE.lock`
`a162ded1`, for the franken_markdown repin from `0a028127` to `a3578a0f` (fm-y5fl):
fmd-math `\ldots`/`\cdots` as three punctuation dots, `\pmod`, and text-mode spaces. No package version changes.
**The release package gate still FAILS at its size budget** (fm-8j70), so this
audit does not claim a passing package.

- **VERIFIED (mechanical):** `cargo tree -p fmn-wasm --target
  wasm32-unknown-unknown --edges normal --locked` is unchanged at 134 lines with
  no fmd or franken_markdown package. The lock change moves only the three
  franken_markdown packages' rev, none of which is in that graph.
- The `--locked` wasm32 build and wasm-smoke were not re-run: no package in the
  wasm graph changed since the run below.

Earlier re-run, 2026-10-07, against `Cargo.lock` `a1f1461f` and `SUITE.lock`
`c4d868ff`, for the franken_markdown repin from `bfb3221c` to `0a028127` (fm-5wq.56):
fmd-math text-mode quote and dash ligatures. No package version changes.
**The release package gate still FAILS at its size budget** (fm-8j70), so this
audit does not claim a passing package.

- **VERIFIED (mechanical):** `cargo tree -p fmn-wasm --target
  wasm32-unknown-unknown --edges normal --locked` is unchanged at 134 lines with
  no fmd or franken_markdown package. The lock change moves only the three
  franken_markdown packages' rev, none of which is in that graph.
- The `--locked` wasm32 build and wasm-smoke were not re-run: no package in the
  wasm graph changed since the run below.

Earlier re-run, 2026-10-06, against `Cargo.lock` `90803e2a` and `SUITE.lock`
`1b2c97d0`, for the franken_markdown repin from `f059cac6` to `bfb3221c` (fm-5wq.60):
fmd-math rule 11 `\root` placement. No package version changes.
**The release package gate still FAILS at its size budget** (fm-8j70), so this
audit does not claim a passing package.

- **VERIFIED (mechanical):** `cargo tree -p fmn-wasm --target
  wasm32-unknown-unknown --edges normal --locked` is unchanged at 134 lines with
  no fmd or franken_markdown package. The lock change moves only the three
  franken_markdown packages' rev, none of which is in that graph.
- The `--locked` wasm32 build and wasm-smoke were not re-run: no package in the
  wasm graph changed since the run below.

Earlier re-run, 2026-10-06, against `Cargo.lock` `c3737f70` and `SUITE.lock`
`f2118084`, for the franken_markdown repin from `4c8caa25` to `f059cac6` (fm-djcw):
PDF inline `$…$` through fmd-math `Layout`. fmd-font and fmd-math are
byte-identical between the two revisions, and no package version changes.
**The release package gate still FAILS at its size budget** (fm-8j70), so this
audit does not claim a passing package.

- **VERIFIED (mechanical):** `cargo tree -p fmn-wasm --target
  wasm32-unknown-unknown --edges normal --locked` is unchanged at 134 lines with
  no fmd or franken_markdown package. The lock change moves only the three
  franken_markdown packages' rev. `wasm-smoke/Cargo.lock` is unchanged.
- **VERIFIED (mechanical):** `cargo build --locked -p fmn-wasm --target
  wasm32-unknown-unknown` passes through rch, and `wasm-smoke/run.sh` reports
  `wasm-smoke OK` with the unchanged Node digest `1f248a71347b82aa` and semantic
  oracle masks `111111` (witness) and `101000` (planted mirror).
- `scripts/check_wasm_package.sh` was not re-run: the wasm graph is unchanged.

Earlier re-run, 2026-10-06, against `Cargo.lock` `f719a42b` and `SUITE.lock`
`9fc53227`, for fm-5wq. fmn-conformance gains a dev-dependency on fmn-wasm, so
its semantic sanity oracles can read the browser player's frame path. The lock
change adds only that edge to fmn-conformance's dependency list; no package,
source or checksum changes. wasm-smoke also gains in-VM semantic oracles.
**The release package gate still FAILS at its size budget** (fm-8j70), so this
audit does not claim a passing package.

- **VERIFIED (mechanical):** `cargo tree -p fmn-wasm --target
  wasm32-unknown-unknown --edges normal --locked` is unchanged at 134 lines with
  no fmd or franken_markdown package. `wasm-smoke/Cargo.lock` is unchanged.
- `cargo build --locked -p fmn-wasm --target wasm32-unknown-unknown` exited 0.
  `wasm-smoke/run.sh` (Node 22.2.0) exited 0 with the unchanged Node digest
  `1f248a71347b82aa`. Its new semantic-oracle mask is `111111` for the witness,
  and `101000` for the planted y-down mirror: only the left-dot and background
  oracles survive a vertical flip.
- `scripts/check_wasm_package.sh` was not re-run: the wasm graph is unchanged.

Earlier re-run, 2026-10-05, against `Cargo.lock` `1b192ac9` and `SUITE.lock`
`9fc53227`, for the franken_markdown repin from `68fe29b8` to `4c8caa25`
(fm-tex-metrics-glyphs-ru72): fmd-math's rule-17 italic corrections plus
upstream editor/WASM-profile work. No package version changes.
**The release package gate still FAILS at its size budget** (fm-8j70), so this
audit does not claim a passing package.

- **VERIFIED (mechanical):** `cargo tree -p fmn-wasm --target
  wasm32-unknown-unknown --edges normal --locked` is 134 lines and contains no
  fmd or franken_markdown package. The lock change moves only the three
  franken_markdown packages' rev. `wasm-smoke/Cargo.lock` has no
  franken_markdown entry and is unchanged.
- `cargo build --locked -p fmn-wasm --target wasm32-unknown-unknown` exited 0.
  `wasm-smoke/run.sh` (Node 22.2.0) exited 0 with the unchanged Node digest
  `1f248a71347b82aa`.
- `scripts/check_wasm_package.sh` was not re-run: the wasm graph is unchanged.

Earlier re-run, 2026-10-04, against `Cargo.lock` `38e2003d` and `SUITE.lock`
`bb530724`, for the franken_markdown repin from `e911be2a` to `68fe29b8`
(fm-tex-display-style-nclg). That repin covers the display-integral fix and the
upstream fmd-font 0.3.3 / fmd-math 0.1.2 / franken_markdown 0.5.0 release.

- **VERIFIED (mechanical):** `cargo tree -p fmn-wasm --target
  wasm32-unknown-unknown --edges normal --locked` is 134 lines, the same
  graph as below, and contains no fmd or franken_markdown package. The lock
  change touches only those three packages. `wasm-smoke/Cargo.lock` has no
  franken_markdown entry and is unchanged.
- `cargo build --locked -p fmn-wasm --target wasm32-unknown-unknown` exited 0.
  `wasm-smoke/run.sh` (Node 22.2.0) exited 0 with the unchanged Node digest
  `1f248a71347b82aa`.
- `scripts/check_wasm_package.sh` was not re-run: wasm-bindgen and wasm-opt
  are absent on this host, and the wasm graph is unchanged.

Earlier re-run, 2026-09-28, against `Cargo.lock` `d3bec838` and `SUITE.lock`
`9858f262`, for the frankenscipy repin from `5a7aafa2` to `85edd7a9` (fm-9esi):
- the fsci-sparse `npz` feature gate, which removes 11 packages none of which are
  in the wasm graph;
- the num_jac `sqrt` thresholds.

**The release package gate still FAILS at its size budget** (fm-8j70), so this
audit does not claim a passing package.

- **VERIFIED (mechanical):** the path-normalized `cargo tree -p fmn-wasm --target
  wasm32-unknown-unknown --edges normal --locked` is identical to the `5a7aafa2`
  tree below (134 lines) once the rev string is normalized. `wasm-smoke/Cargo.lock`
  moves only its three fsci entries to the new rev.
- `cargo build --locked -p fmn-wasm --target wasm32-unknown-unknown` exited 0.
  `wasm-smoke/run.sh` (Node 22.2.0) exited 0 with the unchanged Node digest
  `1f248a71347b82aa`.
- `scripts/check_wasm_package.sh` was not re-run, for the same reason as below:
  the qualified wasm-pack 0.15.0 is absent on this host, and the wasm graph is
  unchanged.

Earlier re-run, 2026-09-28, against `Cargo.lock` `02ce9708` and `SUITE.lock`
`8fed8803`, for the frankenscipy repin from `5b1441b1` to `5a7aafa2` (fm-9esi).
**The release package gate still FAILS at its size budget** (fm-8j70), so this
audit does not claim a passing package.

- This lock change does reach the wasm graph, through fmn-geom's solver gateway.
  **VERIFIED (mechanical):** the path-normalized `cargo tree -p fmn-wasm --target
  wasm32-unknown-unknown --edges normal --locked` goes from 130 to 134 lines:
  - moved to the new rev: `fsci-linalg`, `fsci-fft` and `fsci-runtime` (0.1.0 to
    0.2.0);
  - moved version: `nalgebra` 0.34.2 to 0.35.0, `simba` 0.9.1 to 0.10.2, `wide`
    0.7.33 to 1.7.1, and `blake3` 1.8.5 to 1.8.7;
  - added: `rayon` 1.12.0, `rayon-core` 1.13.0, `crossbeam-deque` 0.8.8,
    `crossbeam-epoch` 0.9.21, `crossbeam-utils` 0.8.22 and `either` 1.17.0
    (fsci-linalg's thread pool);
  - dropped: `arrayref`, `paste` and `safe_arch`.

  `wasm-smoke/Cargo.lock` was re-resolved for the same fsci entries only. Every
  package has its `SUITE_ALLOWLIST.tsv` row.
- `cargo build --locked -p fmn-wasm --target wasm32-unknown-unknown` exited 0.
  `wasm-smoke/run.sh` (Node 22.2.0) exited 0 with the unchanged Node digest
  `1f248a71347b82aa`: the probe's frames are byte-identical. The probe does not
  reach fsci-linalg's n >= 128 parallel factorizations. How rayon behaves there on
  wasm32 (no spawnable threads in the serial build) is **ASSESSED**, not verified.
- `scripts/check_wasm_package.sh` was not re-run. The qualified wasm-pack 0.15.0 is
  not installed on this host (0.13.1 is), and the gate has stopped at the fm-8j70
  size budget since `b492564b`. A size proxy, the release `fmn_wasm.wasm` cdylib
  before and after the bump:
  - raw: 789,937 to 790,184 bytes (+247);
  - after Binaryen 117 `wasm-opt -Oz`: 583,513 to 583,520 bytes (+7).

  So the repin does not move the fm-8j70 breach materially. The qualified bundler
  artifact is still unmeasured.

Earlier re-run, 2026-09-26, against `Cargo.lock` `7207b6fe` and `SUITE.lock`
`33ee2568`, for the build-identity digest of ADR-0025
(fm-certified-closure-integrity-4fei). **The release package gate still FAILS at
its size budget** (fm-8j70), so this audit does not claim a passing package.

- The lock change adds one workspace edge: `fmn-cli` now lists `fmn-hash`, as a
  build- and dev-dependency for the uncommitted-state digest. No package is
  added, and `fmn-cli` is not in any wasm graph. **VERIFIED (mechanical):** the
  path-normalized output of `cargo tree -p fmn-wasm --target wasm32-unknown-unknown
  --edges normal --locked` is identical at `636e7fd3` (lock `1947c1f5`) and in the
  changed tree (130 lines, with no `fmn-cli`). `SUITE.lock` and
  `wasm-smoke/Cargo.lock` are unchanged.
- `cargo build --locked -p fmn-wasm --target wasm32-unknown-unknown` exited 0.
  `wasm-smoke/run.sh` exited 0 with the unchanged Node digest `1f248a71347b82aa`.
- `scripts/check_wasm_package.sh` was not re-run: the wasm graph is unchanged, and
  the gate has stopped at the fm-8j70 size budget since `b492564b`.

Earlier re-run, 2026-09-25, against `Cargo.lock` `1947c1f5` and `SUITE.lock`
`33ee2568`, for the franken_markdown repin from `82588865` to `e911be2a`
(fm-fmd-repin-g2-truth-y3gr). **The release package gate still FAILS at its size
budget** (fm-8j70), so this audit does not claim a passing package.

- The lock change moves only `fmd-font` (0.1.0 to 0.3.2), `fmd-math` (0.1.0 to
  0.1.1) and `franken_markdown` (0.3.5 to 0.4.5) to the new rev, adding no
  packages. None of the three is in any wasm graph. **VERIFIED (mechanical):** the
  path-normalized output of `cargo tree -p fmn-wasm --target wasm32-unknown-unknown
  --edges normal --locked` is identical before and after (130 lines, with no fmd
  crate), and `wasm-smoke/Cargo.lock` is unchanged.
- `cargo build --locked -p fmn-wasm --target wasm32-unknown-unknown` exited 0.
  `wasm-smoke/run.sh` exited 0 with the unchanged Node digest `1f248a71347b82aa`.
- `scripts/check_wasm_package.sh` was not re-run: the wasm graph is unchanged, and
  the gate has stopped at the fm-8j70 size budget since `b492564b`.

Earlier re-run, 2026-09-24, against `Cargo.lock` `6ce5886a`, at `acd19004`.
The measurements ran on that commit's pre-rebase twin `86f2896d`; the only
commits rebased beneath it, f1d4afd5 and 92d3b889, touch `crates/fmn` and CI,
which are outside every wasm graph. **The release package gate still FAILS at
its size budget**, so this audit does not claim a passing package.

- The lock change behind this re-run is d121ddd5. It adds five workspace edges to
  the `fmn` umbrella crate (`fmn-codec`, `fmn-frame`, `fmn-output`, `fmn-render`,
  `fmn-runtime`), and `fmn` is not in any wasm graph. **VERIFIED (mechanical):**
  `git diff ad0b7d25 HEAD -- Cargo.lock` is those five lines, and `SUITE.lock` and
  `wasm-smoke/Cargo.lock` are unchanged. The path-normalized output of `cargo tree
  -p fmn-wasm --target wasm32-unknown-unknown --edges normal --locked` is identical
  at `ad0b7d25` and `acd19004` (130 lines).
- `cargo build --locked -p fmn-wasm --target wasm32-unknown-unknown` exited 0.
  `wasm-smoke/run.sh` exited 0 with the unchanged Node digest `1f248a71347b82aa`.
- `scripts/check_wasm_package.sh` was not re-run. It has failed at the size
  budget since `b492564b` (fm-8j70) and stops before the Chromium stage, and the
  wasm graph is unchanged. The qualified wasm-pack 0.15.0 bundler artifact at
  `acd19004` is 555,763 bytes against the 514,521-byte budget. The attribution
  is on fm-8j70.

Earlier re-run, 2026-09-24, against `Cargo.lock` `b97f1d76` from a clean worktree
at `c5f50e1e`, with the qualified tools (wasm-pack 0.15.0, wasm-bindgen 0.2.127,
wasm-opt 117, webpack 5.109.2 / webpack-cli 7.2.2):

- The lock change behind that re-run is a single edge: `fmn-python` now depends on
  `fsci-opt` (6bc0ddb2), which the lock already held. `fmn-python` is not in any
  wasm graph. **VERIFIED (mechanical):** `git diff 2649c18b HEAD -- Cargo.lock` is
  that one line, and `SUITE.lock` and `wasm-smoke/Cargo.lock` are unchanged.
- `cargo tree -p fmn-wasm --target wasm32-unknown-unknown --edges normal --locked`
  exited 0, and `cargo build --locked -p fmn-wasm --target wasm32-unknown-unknown`
  exited 0.
- `wasm-smoke/run.sh` exited 0 with the unchanged Node digest `1f248a71347b82aa`,
  after 3837dac0 restored its build (it had missed fm-sq8.9's `ScreenMap::y_up`).
- `scripts/check_wasm_package.sh` exited 1 at the size budget: the bundler wasm is
  555,759 bytes against the 514,521-byte `crates/fmn-wasm/SIZE_BUDGET.tsv` budget.
  The breach is older than this lock change. The same build at `b492564b` (before
  fm-sq8.9) is already 554,530 bytes; fm-sq8.9 adds 1,229. The growth landed after
  the 2026-09-17 pass below and needs bisecting (bead fm-8j70). The
  Chromium stage was not reached. The browser digests and the
  `certified-cpu:scalar:6` identity recorded below predate fm-sq8.9, which flips
  front-door frames to +Y-up and bumps the renderer to `scalar:7`; they are
  historical until the package gate passes again.

Previous full pass: re-derived 2026-09-17 on the local qualified-tool host at commit
`efdf8e4041bbbe9f2a63b4b91f30b59286861b89`. The locked dependency tree, wasm32
target build, locked Node smoke, and the complete serial / shared-memory package
and Chromium gate all exited 0 on a **clean tree** (`source_dirty=false`, receipt
schema `fmn-wasm-package-receipt/2`). Node smoke digest `1f248a71347b82aa`;
browser digests `f5b7f4b9` / `c864b03d` / `8d9ea9c9`.
The always-on `wasm_audit_is_bound_to_current_locks` Gauntlet test fails when
either authority changes. That forces this audit to be re-run and its outcome
recorded, instead of leaving a plausible but stale "current pins" claim behind.

- `SUITE.lock` SHA-256: `e47581236910eb502eafa57e6ae4f99698a7cd81fbb8d8a5216824e27330bdda`
- `Cargo.lock` SHA-256: `d5593dcfa0beaa9a63109a058089c98a5f2c5b0dceca196efaa31a056cb5826d`
- Auxiliary `wasm-smoke/Cargo.lock` SHA-256: `4c58a1bb52b3e6e833fcc0cc50cfbf086a3b8889e4790480c291b64b44382f1a`

The 0.5.0 release bump re-bound both lock identities above. That bump changed
only the lockstep workspace package versions (0.4.0 -> 0.5.0, path packages
without a registry source); no third-party package, source or checksum in either
lock changed, so the dependency graph audited here is unchanged. The full wasm
package/browser gate was not re-run for that version-only change.

Method labels are deliberately narrow:

- **VERIFIED (build):** compiled for `wasm32-unknown-unknown` by a named gate.
- **VERIFIED (execution):** instantiated and exercised in a named JavaScript or
  browser runtime.
- **VERIFIED (mechanical):** derived from the committed manifests, locks, and
  governed-closure checker without claiming runtime behavior.
- **ASSESSED:** a future path whose design posture is known but whose artifact
  is not part of the current compiled graph.

## Shipped serial and shared-memory graphs

The tree derivation command `cargo tree -p fmn-wasm --target
wasm32-unknown-unknown --edges normal --locked` exited 0. Cargo also emitted a
diagnostic about the intentionally malformed upstream asupersync fixture;
that diagnostic did not prevent tree generation or the subsequent builds.
Both packaged execution variants compiled the workspace crates below with
the exact `nightly-2026-08-31` toolchain.

| workspace packages | verdict | executable evidence |
|---|---|---|
| `fmn-wasm`, `fmn-anim`, `fmn-cache`, `fmn-codec`, `fmn-config`, `fmn-core`, `fmn-dmath`, `fmn-frame`, `fmn-geom`, `fmn-hash`, `fmn-mobject`, `fmn-platform`, `fmn-render`, `fmn-scene` | **VERIFIED (build)** | `scripts/check_wasm_package.sh` runs wasm-pack 0.15.0 over the actual `fmn-wasm` cdylib; `scripts/check.sh` also compiles the render-axis crates directly for `wasm32-unknown-unknown` |

`fmn-core` consumes `fnp-random-core` 0.2.0 at
`a15d5c32e9330b555b0a653058bcfcff22fcb4ec` for the governed RNG.
fmn-geom enters with its doctrine-D4 solver gateway: the pinned
frankenscipy crates `fsci-linalg` 0.2.0, `fsci-fft` 0.2.0, and `fsci-runtime`
0.2.0 at `85edd7a99f2a6487ef35135e135b9959cc0010c1` (the last bringing its
audit-ledger blake3 hashing chain; fsci-linalg brings the rayon/crossbeam
pool), plus their
transitive numeric and serialization stacks (`nalgebra` 0.35.0 with the
`simba`/`matrixmultiply`/`wide`/`approx`/num-* chain, `serde`
1.0.229 with derive, `serde_json` 1.0.151 with `preserve_order`, and the
wasm-bindgen family already listed above). The complete admission — exact
versions, checksums, features, licenses, proc-macro/build-script posture, and
unsafe reviews for every package this gateway adds — is enforced row-by-row by
`SUITE_ALLOWLIST.tsv`; `workspace_closure_is_exactly_the_governed_universe`
checks the primary and auxiliary locks on every `cargo test` run.

The pinned nightly has `wasm32-unknown-unknown` installed on the release host.
This is exercised rather than inferred: `scripts/check.sh` target-compiles the
render axis and `wasm-smoke/run.sh` builds a fresh release probe, instantiates it
under Node, checks deterministic render bytes, reads the browser clock shim,
and verifies that process access fails closed. The 2026-09-08 run under Node
22.22.1 emitted the locked smoke digest `1f248a71347b82aa`, reported
`process=capability-absent`, and used the one-CPU browser topology shim.

## Product surfaces

| surface | verdict | current boundary |
|---|---|---|
| Tier 1: `FmnScene` fixed-scene renderer | **VERIFIED (execution)** | The packaged ESM/TypeScript surface enumerates `circle_shift`, `parametric_wave`, and `orbit_duet`, renders RGBA8 into caller-reused buffers, and precisely refuses a wrong destination length. wasm-bindgen copies the mutable JS view into Wasm memory; this is buffer reuse, not a zero-copy claim. |
| Tier 2: `FmnPlayer` FMTL/1 player | **VERIFIED (execution)** | The packaged player parses the governed timeline bundle, refuses engine-major mismatches, exposes labels/seek state, and renders RGBA8 through the same semantic Lumen path. |
| npm bundler artifact | **VERIFIED (execution)** | `scripts/check_wasm_package.sh` builds the self-contained package, checks its exact file/license inventory and recorded size budgets, runs `npm pack` plus `npm publish --dry-run`, installs the tarball into a fresh consumer, bundles it with webpack 5.109.2, and exercises both tiers through Canvas write/readback in headless Chrome. The current diagnostic receipt and its limits are recorded below. |
| threads artifact | **VERIFIED (execution)** | The separate `fmn-wasm/threads` export is built with atomics and imported shared memory. Its coordinator compiles the module once, verifies the instantiated buffer is a `SharedArrayBuffer`, passes the same module and memory to two module workers, and renders independent whole frames concurrently. The Chromium gate byte-compares serial and threaded results, checks repeatability, and proves a document without COOP/COEP receives `FMN_WASM_CROSS_ORIGIN_ISOLATION_REQUIRED` before worker startup. This is frame-batch parallelism, not intra-frame parallelism. |

The browser/package evidence proves compilation, packaging, deterministic
same-build execution, and the tested failure paths. It does **not** prove
browser performance, cross-browser behavior, the certified platform matrix,
cross-platform bit identity, registry publication, or service-worker/CDN
deployment. WASM remains standard-mode only.

### Current package receipt

The release-grade run exited 0 in 111.9 seconds on a clean clone of commit
`efdf8e4041bbbe9f2a63b4b91f30b59286861b89` (`source_dirty=false`); retained
evidence is `.rch-results/wasm-package-20260917-fm7wm6/release-receipt.json`
(evidence root `/tmp/fmn-fm7wm6-release-20260917`). An independent dirty-tree
diagnostic run at the same commit produced byte-identical browser digests,
retained at `.rch-results/wasm-package-20260917-fm7wm6/receipt.json`. The
gate script now accepts both npm receipt shapes (npm 10 bare object and the
npm 9 `{package-name: receipt}` mapping); this host's npm is 11.19.1.
No registry publication occurred.

| Identity or result | Executed value |
|---|---|
| Tools | wasm-pack 0.15.0; wasm-bindgen 0.2.127; Binaryen 117; Node 22.22.1; npm 11.19.1; webpack 5.109.2 / CLI 7.2.2; Chrome 153.0.8010.47 |
| npm tarball | 409,605 bytes; SHA-256 `2a7a7f4de062b4f112619baebec2aa79db953c78e4c303dc179013ea3c2d2a18` |
| Serial Wasm | 487,503 raw bytes; 194,668 gzip bytes |
| Shared-memory Wasm | 487,117 raw bytes; 195,145 gzip bytes |
| `FmnScene` | 45 rendered frames; digest `f5b7f4b9` |
| `FmnPlayer` | 45 rendered frames; digest `c864b03d`; engine identity `certified-cpu:scalar:6` |
| Workers | 2 workers over shared memory; 4 frames; digest `8d9ea9c9`; serial/threaded byte equality passed |
| Isolation refusal | `FMN_WASM_CROSS_ORIGIN_ISOLATION_REQUIRED` before worker startup in the nonisolated document |

Chrome ran with its sandbox enabled. The asupersync malformed-fixture cargo
diagnostic is the intentionally non-fatal upstream diagnostic tracked by
bead fm-asupersync-fixture-diagnostic-5bxf; it did not affect tree
generation or any build. The player engine identifier names the semantic
renderer serialized in the timeline; it does not change the browser's
standard-only certification scope.

Historical evidence: the v0.4.0 release-commit receipt passed at
`d1e4274aa32ba5c59934029db397bbbeb9bfa18c` with `source_dirty=false`; its
395,038-byte tarball had SHA-256
`f531940b5849ed2f3e091424931581bf4450e2811ec66e850aebfe3a76dc371d`.
The 2026-09-08 a40c4a88 receipt (npm 10.9.4, Chrome 152, tarball
`41a94cd2…cf85`) qualified the August locks and is superseded by the
receipt above.

## Intentionally absent from the current wasm graph

| packages or capability | posture |
|---|---|
| `fmn-studio`, `fmn-cli` | host front doors: supervisor/subprocess, terminal, filesystem, and CLI behavior; the browser package is a separate W11 artifact |
| `fmn-python`, PyO3, NumPy | supported host-CPython portal only; never part of a standalone browser artifact |
| `asupersync` | host-side multi-scene batch farms and scheduler laboratory; never in the frame loop |
| `frankentorch`, `ft-kernel-metal`, CUDA/Metal annexes | native standard-mode accelerator annexes; no GPU annex enters this wasm package |
| `fmn-output`, ffmpeg | host publication and optional external-tool boundary; browser frames are returned to the consumer instead |
| `fmn-text`, `fmn-tex`, `fmn-library`, `fmd-font`, `fmd-math`, the complete bundled font set | not reached by the current three-scene Tier-1 package or FMTL demo graph; future browser text/font consumption must be built, size-measured, and re-audited before it is called verified |
| Remaining `franken_numpy` / `frankenscipy` crates; `franken_networkx`, `frankenpandas` | only `fnp-random-core` and the three named `fsci-*` crates are consumed; other subsets require their own target build and governed-closure update |

## Standing rules

1. A crate enters the shipped wasm graph only after an actual
   `wasm32-unknown-unknown` build and the governed closure both pass.
2. Any `SUITE.lock` or `Cargo.lock` change invalidates this audit mechanically.
   Update the recorded digests only after re-running the tree, target build,
   Node smoke, and every affected browser/package check.
3. An absent host-only package is not evidence that it is wasm-compatible.
   Future text, fonts, or suite libraries stay **ASSESSED** until their real
   artifact is compiled and exercised.
4. Threads remain a separate explicit subpath behind atomics, shared memory,
   and cross-origin isolation. The default import stays serial and no fallback
   is implicit. Certified mode and native accelerator annexes remain explicit
   refusals in the browser package.
