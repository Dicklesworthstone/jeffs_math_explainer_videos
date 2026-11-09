# W5 wasm tier-1 browser demo (fm-l97, §10.7)

The `fmn-wasm` crate compiles to `wasm32-unknown-unknown`, rendering the
fixed primitive-corpus scenes (`circle_shift`, `parametric_wave`,
`orbit_duet`) to a `<canvas>` via `ImageData`. The default demo and npm import
are single-threaded. Wasm is not in the certified matrix, but standard-mode
determinism holds (same scene, same seed, same build ⇒ byte-identical RGBA8;
every transcendental comes from `fmn-dmath`, ADR-0014).

This directory is the demo's one home (`demo/wasm/`); there is no parallel
`examples/wasm/`.

## Build

`wasm-pack` 0.15.0 is the verified documented path
(it vendors its own pinned `wasm-bindgen` CLI + `wasm-opt`):

```sh
wasm-pack build --target web --out-dir ../../demo/wasm/pkg crates/fmn-wasm
```

If `wasm-pack` is unavailable, the equivalent two-step path is (requires
`wasm-bindgen-cli` 0.2.127, `cargo install wasm-bindgen-cli --version 0.2.127`
— installing it is a deliberate local action, not part of this repo):

```sh
cargo build --target wasm32-unknown-unknown --release -p fmn-wasm
wasm-bindgen --target web --out-dir demo/wasm/pkg \
    target/wasm32-unknown-unknown/release/fmn_wasm.wasm
```

(`demo/wasm/pkg/` is a build artifact; wasm-pack's generated `.gitignore`
keeps it out of version control.)

## Run

Any static file server works; ES modules require http(s), not `file://`:

```sh
python3 -m http.server 8080 --directory demo/wasm
# open http://localhost:8080/
```

The page constructs a scene, then scrubs/plays captured frames through the
caller-buffer-reusing `render_into` path into `ImageData`. wasm-bindgen still
marshals the mutable slice through WebAssembly memory; this avoids a fresh
returned JS array per frame but is not a zero-copy JS/WASM transfer.

## Tier 2: the FMTL/1 timeline player (fm-oee)

`player.html` is the tier-2 demo: it consumes a serialized timeline bundle
(`docs/FMNT1_TIMELINE_BUNDLE.md`) with `FmnPlayer` — no scene code in the
browser — and scrubs/seeks it (slider, play, and one button per authored
label). Pure segments reconstruct each frame from begin/end snapshots
through the contract's record-lerp law; stateful segments restore recorded
per-frame snapshots.

Export the bundle (deterministic; reruns rewrite identical bytes), then
serve as above and open `player.html`:

```sh
cargo run -p fmn-wasm --example export_bundle   # writes demo/wasm/bundle.fmtl
python3 -m http.server 8080 --directory demo/wasm
# open http://localhost:8080/player.html
```

The bundle's size is recorded with headroom in `SIZE_BUDGET.tsv`
(`demo-timeline-bundle` row), enforced by a host test.

### Open your own recording

Use **Open recording** to load a local `.fmtl` file without uploading it. Both
minor-0 planar bundles and minor-1 camera-bearing recordings use the shared
reader; the latter retain recorded camera motion, lighting, background and
sample policy. The output canvas owns the viewport aspect ratio. Engine
identity mismatches still refuse: rebuild the player and re-export the scene
from the same source closure rather than bypassing the check.

The browser admits at most 256 MiB of input and checks local file size before
reading it. Loading a replacement renders its first frame before releasing the
current player. A failed or superseded load keeps the prior movie, and replaced
Wasm instances are freed. **Load demo** switches back to `bundle.fmtl`.

Playback follows elapsed presentation time at the bundle FPS, independent of
monitor refresh rate. Pause/resume and seeks reset the presentation anchor;
a late browser callback catches up without retiming the recorded scene.

The dependency-free transport and loading tests run without a Wasm build:

```sh
node --test demo/wasm/playback.test.mjs demo/wasm/player-session.test.mjs
```

These JavaScript tests exercise clocks and session ownership with a test player;
they do not replace Rust camera regression tests or real-browser package checks.

## npm package and real-browser release gate

The publishable artifact uses wasm-pack's `bundler` target, carries its README,
the engine/font manifest, and all license texts, and records the exact source
commit in `package.json`. The release gate builds that package from scratch,
checks workspace/npm/runtime version lockstep, enforces raw and gzip size
budgets, verifies the exact `npm pack` and `npm publish --dry-run` inventories,
installs the tarball into a fresh consumer, bundles that consumer with webpack,
and renders both a primitive scene and `bundle.fmtl` in real headless Chromium:

```sh
scripts/check_wasm_package.sh
```

The verified tool versions are wasm-pack 0.15.0, webpack 5.109.2, and
webpack-cli 7.2.2. The script never publishes and never deletes its task
directory; its final line names the preserved evidence root. Set
`FMN_WASM_PACKAGE_ROOT` to choose a new, nonexistent evidence directory.
If those exact webpack packages live in a task-local `node_modules`, set
`FMN_WEBPACK` to its `.bin/webpack`, `FMN_NODE_PATH` to that `node_modules`,
and `FMN_NODE` to the real Node executable (not Bun's partial `node` shim).

The same package also carries a distinct `fmn-wasm/threads` export. It builds
with atomics and imported shared memory, creates one scene instance per module
worker, and distributes independent whole-frame renders across the pool while
returning results in request order. Each document that imports it must serve:

```text
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

Startup checks both `crossOriginIsolated` and the instantiated
`WebAssembly.Memory`; a wrong deployment receives the typed
`FMN_WASM_CROSS_ORIGIN_ISOLATION_REQUIRED` refusal instead of a serial
fallback. The root `fmn-wasm` import remains the explicit serial choice. The
package gate exercises both the isolated two-worker success path and a
non-isolated refusal in real Chromium. It does not claim single-frame
parallelism, certified-mode output, cross-browser coverage, or a measured
speedup.

The threads build additionally requires the pinned nightly's `rust-src`, the
exact wasm-bindgen CLI 0.2.127, and wasm-opt 117. The repository toolchain file
installs `rust-src`; set `FMN_WASM_BINDGEN` and `FMN_WASM_OPT` when those exact
tools are not on `PATH`.

## Artifact size (R19)

`crates/fmn-wasm/SIZE_BUDGET.tsv` records the measured sizes and the
enforced budget (measured + 10% headroom); a host test in the crate
(`size_budget_within_headroom`) fails if a rebuilt artifact exceeds it.
