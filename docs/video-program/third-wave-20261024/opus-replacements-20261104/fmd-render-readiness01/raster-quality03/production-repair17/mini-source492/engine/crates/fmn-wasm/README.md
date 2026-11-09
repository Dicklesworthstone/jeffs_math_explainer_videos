# fmn-wasm

`fmn-wasm` is FrankenManim's browser package. Its default import is the compact
single-threaded artifact. It exposes:

- `FmnScene`, which constructs one of the fixed primitive-corpus scenes and
  renders captured frames through Lumen to RGBA8 pixels;
- `FmnPlayer`, which loads an FMTL/1 serialized timeline, seeks without replay,
  and renders it through the same path; and
- `engine_version()`, the engine/package version compiled into the module.

The npm package is a bundler-targeted ES module. A typical consumer is:

```js
import { FmnScene, engine_version } from "fmn-wasm";

console.log(`FrankenManim ${engine_version()}`);
const scene = new FmnScene("circle_shift", 640, 360);
const pixels = new Uint8Array(scene.width * scene.height * 4);
scene.render_into(0, pixels);
context.putImageData(
  new ImageData(new Uint8ClampedArray(pixels.buffer), scene.width, scene.height),
  0,
  0,
);
```

`render_into` reuses caller-owned JS storage, avoiding a new returned
`Uint8Array` on every frame. The wasm-bindgen boundary still copies between JS
and WebAssembly memory; this API does not claim a zero-copy JS/WASM transfer.

For independent-frame throughput, an explicit `fmn-wasm/threads` subpath
provides a shared-memory worker pool:

```js
import { createThreadedScene } from "fmn-wasm/threads";

const scene = await createThreadedScene("circle_shift", 640, 360, { threads: 4 });
try {
  const frames = await scene.renderFrames([0, 1, 2, 3]);
  // Promise results remain in the requested frame order.
} finally {
  await scene.close();
}
```

This is frame-batch parallelism: each worker owns an independent `FmnScene`
instance and renders whole frames through the same Lumen path. All instances
share one instantiated Wasm memory, and package startup verifies the actual
memory buffer is a `SharedArrayBuffer` before arming workers. It does not claim
that one frame is internally parallel.

FMTL/1 bundles carry an engine identity and are rejected before use when that
identity does not match the player. The npm package version is kept in lockstep
with the Rust workspace version, and the release staging metadata records the
exact source commit.

The package is self-contained: it makes no CDN requests and carries the engine
license, the bundled-font manifest, and all corresponding license texts. It is
standard-mode only and is not part of FrankenManim's certified platform matrix.

The threads subpath is a distinct atomics/shared-memory artifact. Its document
must be cross-origin isolated with both
`Cross-Origin-Opener-Policy: same-origin` and
`Cross-Origin-Embedder-Policy: require-corp`. Without those headers it throws
`ThreadedWasmUnavailableError` with code
`FMN_WASM_CROSS_ORIGIN_ISOLATION_REQUIRED`; it never silently falls back. Use
the default `fmn-wasm` import when serial fallback is the intended policy.

Repository build, browser-smoke, size-budget, and npm dry-run instructions live
in [`demo/wasm/README.md`](../../demo/wasm/README.md).
