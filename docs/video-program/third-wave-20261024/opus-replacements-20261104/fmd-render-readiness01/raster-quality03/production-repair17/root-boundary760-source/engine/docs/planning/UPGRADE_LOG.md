# Dependency Upgrade Log

**Initial audit date:** 2026-08-11
**Last updated:** 2026-08-16
**Dependency-graph audit HEAD:** `fe7f1b19a61dbebbb70188f7224db7bbfb50aee6`
**Security migration commit:** `64a0458e5de0596e2cb89ed7415c2b78cdccb260`
**PyO3 final validation source:** the exact tree recorded by that commit
**WASM migration:** `fm-oqxs` (this recorded change)
**Project:** FrankenManim
**Manifests:** root workspace, `fuzz`, `wasm-smoke`, and the non-member G0 spikes

## Outcome

| Result | Count |
|---|---:|
| Direct external dependency or patch records audited | 10 |
| Ordinary latest-stable updates available | 0 |
| Already current | 2 |
| Governed exact pins preserved | 6 |
| Deliberate migrations requiring attention | 0 |
| Updates applied | 2 |

This repository does not admit routine semver drift. `SUITE.lock`,
`SUITE_ALLOWLIST.tsv`, ADR-0008, and `docs/GOVERNANCE.md` require exact
dependency identity and a Gauntlet-backed review for every admitted change.
The initial audit therefore rejected routine semver drift. The user then
explicitly approved the governed PyO3 security migration tracked by fm-kg6g;
that isolated migration is now applied and validated. The later `fm-oqxs`
tranche applies the sole remaining Cargo root delta, wasm-bindgen 0.2.127,
with its coupled lock/allowlist closure and WASM-specific proof.

## Applied

### PyO3: 0.26.0 -> 0.29.2

**Status:** Applied by `fm-kg6g` at `64a0458e5de0596e2cb89ed7415c2b78cdccb260`.

`cargo audit` found two vulnerabilities in the exact ADR-0015 pin:

- [RUSTSEC-2026-0176](https://rustsec.org/advisories/RUSTSEC-2026-0176):
  out-of-bounds reads in `PyList` and `PyTuple` iterator `nth`/`nth_back`;
- [RUSTSEC-2026-0177](https://rustsec.org/advisories/RUSTSEC-2026-0177):
  missing `Sync` bound on `PyCFunction::new_closure` closures.

Both are fixed by PyO3 0.29.0 or newer. The latest stable release inspected
was 0.29.2 (2026-08-05). The portal's current production tests passed, but that
does not remove vulnerable code from the dependency closure.

This was not a pin-only update. The migration touched the approved 12 files
and:

- amended ADR-0015 while retaining `abi3 = off` and the exact three-item
  project-authored unsafe boundary;
- declared `#[pymodule(gil_used = true)]` for production and spike modules;
- replaced the unifiable `pyo3/extension-module` feature with the
  process-scoped `PYO3_BUILD_EXTENSION_MODULE` build setting;
- migrated the G0-5 spike's removed `.downcast()` calls to `.cast()`;
- refreshed the production and spike locks plus every affected exact
  allowlist record; and
- revalidated embedded and extension imports, the unchanged G0-5 suite,
  NumPy live views and detach, copy/deepcopy/pickle, subclass/MRO dispatch,
  weakrefs and dictionary cycles, method-cache invalidation, PG-8 state
  goldens, the governed closure, both lockfiles' security audits, and the full
  repository test gate.

PyO3 0.29.1/0.29.2 also contain directly relevant fixes for populated
`#[pyclass(dict)]` deallocation leaks, `__dict__` reference cycles, type-object
reference leaks, weak-reference pointer arithmetic, and interpreter-shutdown
soundness. The user explicitly approved the library-updater circuit breaker
before implementation; no partial compatibility shim or split-version graph
was introduced.

### wasm-bindgen: 0.2.126 -> 0.2.127

**Status:** Applied by `fm-oqxs` in this recorded change.

0.2.127 was the only ordinary Cargo root version delta reported by the initial
audit. Its changelog is predominantly additive bindings and fixes, and current
FrankenManim usage is limited to the established macro/`JsError` surface. The
exact pin remains classified `deliberate+gauntlet` in `SUITE_ALLOWLIST.tsv`.

The isolated tranche updates both workspace consumers and `wasm-smoke`,
refreshes both lockfiles, reconciles every affected
`wasm-bindgen`/macro/shared/`js-sys`/`web-sys` allowlist row, and passes the
workspace, wasm32 runtime smoke, native-vs-WASM bit-equivalence, bundle-size,
and full repository gates. A same-source A/B build measured a 156-byte raw-WASM
increase (639,566 -> 639,722 bytes, 0.024%) and no optimized web-package change
(454,043 bytes under both versions). `SIZE_BUDGET.tsv` was rebaselined because
the July measurements predated substantial Lumen, scene, and WASM capability;
the dependency itself accounts for only the measured 156-byte raw delta.

The documented WASM packaging tool was also refreshed from wasm-pack 0.13.1
to 0.15.0 after verifying the official Linux release asset's SHA-256
(`c09f971ecaed9a2efc80fdcea7a00ef6b53c7fadc8c57d1f61b53a6aa66b668a`).
The latest tool built the 0.2.127 web package successfully and reproduced the
same 454,043-byte optimized artifact. wasm-pack is a developer-side tool, not
a shipped Cargo dependency, so this does not change the governed runtime graph.

## Preserved governed pins

| Dependency | Exact authority | Reason preserved |
|---|---|---|
| `asupersync` | Git `c48399f20a9780ef420de64fe504ff922e5afe5e` | `SUITE.lock` foundation pin |
| `fmd-font`, `fmd-math` | `franken_markdown` Git `82588865c453b175cb1263b36e30f5b9b1941a2e` | `SUITE.lock` foundation pin |
| `fsci-integrate` | `frankenscipy` Git `5b1441b13a0997901ad2f9835c30072f87ca93b2` | `SUITE.lock` foundation pin |
| `ft-kernel-metal` | `frankentorch` Git `523aaf827faf538aa541126ee222fcd7af348410` | sanctioned Accelerator Annex gateway |
| `block` patch | `rust-block` Git `b39ae859d1ee8e8cb5eef6a516471f1578d26b96` | reviewed Objective-C FFI correction |

The G0-8 accelerator spike intentionally retains its historical
`ft-kernel-metal` proof revision rather than tracking the later production pin.
Its manifest comment was corrected during this audit to make that authority
boundary explicit.

FrankenSQLite 0.3.1 is not a direct or transitive dependency of this workspace,
so there is no FrankenSQLite version record to change. Asupersync remains the
exact suite pin above rather than taking an unrelated registry update.

## Already current

- `rustix =1.1.4` (test-only Studio PTY/dev closure)
- `libfuzzer-sys =0.4.13` (isolated fuzz workspace)

The generated WASM demo `package.json` has no dependencies and no Node lockfile.
The separately shipped Python portal now has an exact Maturin/NumPy packaging
manifest; those host-side build/runtime requirements do not enter the Cargo
runtime closure.

## Security and policy checks

- Root `cargo audit`: exited 0 with RUSTSEC-2026-0176 and
  RUSTSEC-2026-0177 absent. It retains the already-known allowed warning for
  unmaintained `paste 1.0.15` through pinned `nalgebra/simba` and Metal.
- `cargo audit --file spikes/g0-5-python-ext/Cargo.lock`: exited 0 with no
  advisories or warnings across the independent 18-package spike lock.
- `cargo test -p fmn-conformance --test governed_closure`: 14 passed. This is
  the authoritative exact allowlist/checksum/pin gate.
- The initial `cargo outdated --workspace --depth 1` found only the
  `wasm-bindgen` patch delta. The post-migration root-dependency audit found no
  remaining Cargo root update. Cargo printed the known malformed asupersync
  test-fixture diagnostic while walking that checkout.

## Executable evidence on the migrated graph

- `cargo test -p fmn-python`: 16 passed, including the complete embedded
  Python bridge acceptance. Expected unsendable cross-thread refusal
  diagnostics were caught by the passing negative tests.
- The host-ABI extension built with `PYO3_BUILD_EXTENSION_MODULE=1`, imported
  directly from `libmanimlib.so`, exposed all 663 unique public root names,
  and passed the same complete `tests/bridge.py` acceptance externally.
  `ldd` showed no `libpython` dependency.
- The unchanged G0-5 `test_extensibility.py` suite passed 29/29 after a release
  extension build on PyO3 0.29.2.
- PG-8's committed bundle replay and canonical state-golden test passed; the
  full workspace run reported 14 passed and its one real timing producer
  ignored by design.
- `FMN_E2E_FULL=1 cargo test -p fmn-conformance --test e2e_scenarios`: all 5
  fast/full catalog and regression-drill tests passed.
- `cargo test` on the final migration tree exited 0 across the complete
  workspace and doctest set.
- `cargo fmt --check`, `cargo check --all-targets`, and
  `cargo clippy --all-targets -- -D warnings`: each exited 0 on the final
  migration tree.
- `python3 scripts/video_corpus.py verify`: exited 0; all 8 allowlisted sources
  reproduced byte-for-byte at their locked pins.
- `ubs` over all 12 migration files exited 0 with no critical findings.

WASM-specific proof for `fm-oqxs`:

- `cargo test -p fmn-conformance --test governed_closure`: 14 passed against
  the refreshed exact versions, checksums, features, and policy rows.
- `cargo check --target wasm32-unknown-unknown -p fmn-wasm`: exited 0.
- `./wasm-smoke/run.sh`: exited 0 in a real Node runtime and retained the
  expected deterministic digest `1f248a71347b82aa`.
- `FMN_E2E_FULL=1 cargo test -p fmn-conformance --test e2e_scenarios`: all 5
  fast/full catalog and regression-drill tests passed.
- The raw and optimized same-source A/B measurements are recorded above and in
  `crates/fmn-wasm/SIZE_BUDGET.tsv`; the refreshed budget test passed.

The first isolated local all-target check exhausted temporary storage while
compiling unchanged WASM dependencies, and two RCH retries stalled with fresh
heartbeats but no compiler progress. Neither is treated as source evidence.
The local rerun with bounded build concurrency completed successfully and is
the recorded final result.

## Commands used

```text
cargo outdated --workspace --depth 1
cargo outdated --root-deps-only
cargo search <direct dependency>
cargo info <direct dependency>
cargo audit
cargo audit --file spikes/g0-5-python-ext/Cargo.lock
cargo test -p fmn-python
PYO3_BUILD_EXTENSION_MODULE=1 cargo build -p fmn-python
python3 - <<'PY'
# import the built libmanimlib.so, then compile/exec tests/bridge.py
PY
PYO3_BUILD_EXTENSION_MODULE=1 cargo build --release  # G0-5 spike
python3 spikes/g0-5-python-ext/py/test_extensibility.py
cargo test -p fmn-conformance --test governed_closure
cargo test -p fmn-conformance --test perf_pg8 \
  committed_pg8_baseline_bundles_replay_through_the_verifier -- --exact
FMN_E2E_FULL=1 cargo test -p fmn-conformance --test e2e_scenarios
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
python3 scripts/video_corpus.py verify
./wasm-smoke/run.sh
wasm-pack 0.15.0 build --target web --out-dir <fresh-directory> crates/fmn-wasm
ubs <the 12 migration files>
```

---

## 2026-08-15 — W11 Python build/runtime tools

The first `fm-vsq` wheel tranche adds two exact Python-side authorities without
changing the governed Cargo runtime graph:

| Dependency | Exact version | Role |
|---|---:|---|
| Maturin | 1.14.1 | PEP 517 build backend and local wheel builder |
| NumPy | 2.5.2 | host-side runtime dependency of the optional portal |

Maturin is exact-pinned in `pyproject.toml` because wheel layout and tags are
release inputs. It is not present in a shipped wheel's runtime dependency set.
NumPy remains outside Cargo and the standalone `fmn` artifact; the wheel
metadata pins it because the audited structured-buffer bridge is part of this
specific CPython 3.13 portal contract.

Observed proof for the resulting Linux x86-64 artifact:

- Maturin 1.14.1 built
  `franken_manim-0.1.0-cp313-cp313-manylinux_2_34_x86_64.whl` from the locked
  Cargo graph with CPython 3.13.7.
- Archive inspection found the nested full-ABI extension, both authored Python
  packages, console entry point, CycloneDX SBOM, engine license, and all three
  bundled-font OFL texts.
- A new virtual environment installed only the wheel plus NumPy 2.5.2. It
  imported the exact 663-name wildcard surface, ran version and scene-list
  robot records, completed a one-frame construct-only lifecycle probe, and
  refused ordinary render syntax with capability exit 4 because no production
  frame sink is wired yet.
- The repeated commit-`5cdc8cb` build was byte-identical and measured
  3,255,554 bytes against the recorded 10 MiB per-wheel budget; its SHA-256
  is recorded in
  `docs/dist/PYTHON_WHEEL_SIZE.tsv`.
- The 2,942,065-byte sdist includes both API schema files and rebuilt into a
  wheel successfully. The rebuilt wheel is not byte-identical to the direct
  wheel: Maturin prunes unrelated workspace members in the sdist manifest,
  changing the native artifact identity, and its generated CycloneDX SBOM
  retains absolute source URIs. That reproducible-release gap remains open
  under `fm-vsq`; the successful round trip is functional evidence only.

This is focused Linux packaging evidence, not the cross-platform matrix or a
pixel-render proof. `fm-vsq` remains open for those acceptance items.
