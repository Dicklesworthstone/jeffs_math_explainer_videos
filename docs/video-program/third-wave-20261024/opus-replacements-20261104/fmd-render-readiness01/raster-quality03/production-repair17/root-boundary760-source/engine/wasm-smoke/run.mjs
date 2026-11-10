// W5 wasm tier 1 (fm-l97): headless smoke driver for the wasm32 foundation.
//
// Instantiates the raw fmn_wasm_smoke.wasm cdylib directly — deliberately
// WITHOUT a wasm-bindgen CLI pass — so the gate proves the Rust foundation
// executes in a real JS wasm VM, independent of glue generation. The
// placeholder imports wasm-bindgen leaves in the raw module are satisfied
// here by hand:
//
//   - `__wbindgen_describe` / the externref-table shims are inert for this
//     probe's scalar-only surface.
//   - The two `__wbg_now_*` imports are `performance.now` and `Date.now`
//     (both are literally `now` after js_name, so their import names are
//     indistinguishable). The driver binds sentinels first, observes which
//     slot each clock probe reads, then re-instantiates with the real JS
//     functions in the right slots.
//
// Usage: node run.mjs <path-to-fmn_wasm_smoke.wasm>   (bun-as-node works)
//
// This file is an ES module (.mjs): `require` does not exist here on real
// Node (bun's node shim tolerates it, which is how a CommonJS-ism once hid
// locally while failing every hosted runner).

import fs from "node:fs";

const wasmPath = process.argv[2];
if (!wasmPath) {
    console.error("usage: node run.mjs <path-to-fmn_wasm_smoke.wasm>");
    process.exit(2);
}
const bytes = fs.readFileSync(wasmPath);

const nowImports = WebAssembly.Module.imports(
    new WebAssembly.Module(bytes),
).filter((entry) => entry.name.startsWith("__wbg_now_"));

function makeImports(nowFns) {
    let slot = 0;
    const placeholder = new Proxy(
        {},
        {
            get(_target, name) {
                if (name === "__wbindgen_describe") {
                    return () => {};
                }
                if (typeof name === "string" && name.startsWith("__wbg_now_")) {
                    const fn = nowFns[slot];
                    slot += 1;
                    return fn;
                }
                // Any future placeholder import fails loud, never silently 0.
                throw new Error(`unsatisfied wasm import: __wbindgen_placeholder__.${String(name)}`);
            },
        },
    );
    return {
        __wbindgen_placeholder__: placeholder,
        __wbindgen_externref_xform__: {
            __wbindgen_externref_table_set_null: () => {},
            __wbindgen_externref_table_grow: () => 0,
        },
    };
}

function fail(message) {
    console.error(`wasm-smoke FAIL: ${message}`);
    process.exit(1);
}

async function main() {
    // Pass 1: sentinel binding to learn which __wbg_now_* slot is which.
    const sentinel = makeImports([() => 111.0, () => 222.0]);
    const probe = await WebAssembly.instantiate(bytes, sentinel);
    let probeExports = probe.instance.exports;
    const monotonicReads = probeExports.clock_probe_monotonic_ms();
    const performanceIndex = monotonicReads === 111.0 ? 0 : 1;

    // Pass 2: the real browser-equivalent clocks in the right slots.
    const slots = [];
    slots[performanceIndex] = () => performance.now();
    slots[1 - performanceIndex] = () => Date.now();
    const instance = (await WebAssembly.instantiate(bytes, makeImports(slots)))
        .instance;
    const ex = instance.exports;

    // 1. The render path executes on wasm32 and is deterministic in-VM:
    //    one scene, two renders, byte-identical canonical frames — and the
    //    scene genuinely draws, so the equality is not vacuous.
    if (ex.render_probe_is_not_background() !== 1) {
        fail("probe scene renders background-only bytes; the determinism proof would be vacuous");
    }
    if (ex.render_probe_repeat_is_byte_identical() !== 1) {
        fail("two single-threaded renders of the same scene differ on wasm32");
    }
    const digestA = ex.render_probe_digest();
    const digestB = ex.render_probe_digest();
    if (digestA !== digestB) {
        fail(`frame digest unstable across renders: ${digestA} vs ${digestB}`);
    }

    // 1b. Semantic sanity oracles (fm-5wq.46): the witness through the +Y-up
    //     map passes all of them, and its planted vertical mirror fails the
    //     orientation, up-dot and fill oracles (bits 0, 1, 2 and 4).
    const oracles = ex.semantic_oracle_mask();
    const mirrored = ex.semantic_oracle_mask_mirrored();
    if (oracles !== ex.semantic_oracle_all()) {
        fail(`semantic oracles failed in the VM: mask ${oracles.toString(2)}`);
    }
    if ((mirrored & 0b10111) !== 0) {
        fail(`a vertically mirrored witness passed orientation oracles: mask ${mirrored.toString(2)}`);
    }

    // 2. The clock capability reads the real host clocks.
    const monotonicMs = ex.clock_probe_monotonic_ms();
    if (!(monotonicMs >= 0 && monotonicMs < 1e9)) {
        fail(`performance.now-backed monotonic reading implausible: ${monotonicMs}`);
    }
    const wallMs = ex.clock_probe_wall_ms();
    if (!(wallMs > 1.6e12)) {
        fail(`Date.now-backed wall reading implausible: ${wallMs}`);
    }

    // 3. The process capability fails closed with the named error.
    if (ex.process_probe_capability_absent() !== 1) {
        fail("NoProcessRunner did not return ProcessError::CapabilityAbsent");
    }

    // 4. The planner sees exactly one logical CPU.
    if (ex.topology_probe_single_threaded() !== 1) {
        fail("HardwareTopology::current() is not the single-CPU wasm shape");
    }

    console.log(
        `wasm-smoke OK: digest=${digestA.toString(16)} ` +
            `monotonic_ms=${monotonicMs.toFixed(3)} wall_ms=${wallMs.toFixed(0)} ` +
            `process=capability-absent topology=1-cpu ` +
            `semantic_oracles=${oracles.toString(2)} mirrored=${mirrored.toString(2)}`,
    );
}

main().catch((error) => fail(error && error.message ? error.message : String(error)));
