# The Input-Closure Specification (§16.7) — Draft 1

> Normative draft (fm-xb3, W1's "input-closure definition" deliverable).
> Status: **the certified-matrix half is FROZEN** by G0-6 (fm-zn9, 2026-07-25,
> ADR-0010) — see §5. The enumerated closure remains **DRAFT** until G4b
> (fm-yp0) enforces it end to end. This document is the definition every
> "certified" claim is tested against, and changes to it are reviewed like
> schema changes (versioned, deliberate, Gauntlet-diffed).

## 1. Purpose

`--reproducible` promises: **the complete content-hashed input closure ⇒
bit-identical raw frames, canonical PNGs, and WAV across the certified
matrix, at any thread count, forever.** A promise like that is only testable
if "input closure" is enumerated exhaustively: anything that can influence an
output bit is either *in the closure* (hashed, journaled, and reproduced) or
*proven inert* (unable to change certified bits by construction — e.g. thread
count, under the §10.5 parallelism contract). There is no third category. An
influence discovered outside the closure is a certification bug of the
highest severity: it means two runs could differ while their manifests claim
they cannot.

## 2. The enumerated closure

Every item below contributes to the **closure digest** (§4). Items marked
*(structural)* are hashed as canonical serialized documents via fmn-hash;
items marked *(bytes)* are hashed as raw byte streams.

| # | Item | Form |
|---|---|---|
| C1 | Scene sources and every transitively loaded module: native registrations/artifacts for standalone `fmn`, or the Python file/module set loaded by the separate `fmn-python` portal | bytes, per file, ordered by virtual path |
| C2 | Engine identity: the franken_manim build identity and the full `SUITE.lock` contents. The identity is `git:<commit>` for a clean checkout; `git:<commit>+dirty:<sha256>` when compiled inputs differ from the commit, with the digest naming that exact uncommitted state; or an explicit `FMN_BUILD_ID`. Certified output refuses `unidentified:` builds (no `.git`, no `FMN_BUILD_ID`) and `+unverified` builds (git could not report the tree), because they cannot name their sources (ADR-0025) | bytes |
| C3 | Toolchain: the exact pinned nightly (from `SUITE.lock`) and Cargo profile; for a Python-portal run only, the CPython implementation/version, ABI policy, `fmn-python` wheel version, and NumPy version. The target triple and the SIMD build tier's target-feature set form a separate C3 item at virtual path `platform/target` (§4) | structural |
| C4 | Configuration: the fully-resolved config **bytes** after precedence (defaults → user file → CLI), not the file paths | bytes |
| C5 | RNG seeds: the root seed and the named-substream layout version (BN-01) | structural |
| C6 | Assets and fonts: content hash of every asset and font file actually read, keyed by virtual path (bundled fonts included — bundling is not exemption). Native `fmn` and the portal list every face compiled into the binary at `bundled-font/<name>`, a sound superset of the faces a scene reads. On the portal route, every regular file a Python scene reads (with `open()`, NumPy, PIL, `SVGMobject`, `ImageMobject`, …) is a C6 item at `read/<sha256>/<basename>`; an effect this cannot capture refuses certification (§2, portal effects) | bytes, per file |
| C7 | Execution-engine and backend identities: the semantic renderer version, execution engine (`certified` requires the certified CPU engine), and — in `standard` provenance only — annex device/driver identities | structural |
| C8 | Locale and timezone as visible to the engine (certified runs pin `C`/UTC; the pin itself is recorded) | structural |
| C9 | Capability policy: which capability traits were live (fs/process/clock/AssetFetcher implementations by identity, not by pointer), whether the separately hosted Python portal was present, and, when the ffmpeg boundary is used, the process-mechanism identity + policy version and ffmpeg fingerprint (path + content hash + versioned native-image format/architecture attestation + version line) | structural |
| C10 | The determinism mode itself (`standard` vs `certified`) and the declared certified configuration (fixed tile dims, fixed in-flight budget, etc.) | structural |

**Explicitly outside the closure (proven inert under §10.5):** thread count,
render-team topology, scheduling order, machine load, hardware identity of
the CPU (within a certified target), and wall-clock time. **Explicitly
excluded from certification (by construction):** every ffmpeg *product*;
ffmpeg's identity still enters provenance via C9.

ADR-0017 makes portal absence structural for native runs: standalone `fmn`
does not embed, locate, or spawn CPython, so Python runtime state is proven
inert and C3/C9 encode the portal as absent. For `fmn-python` runs, the portal
fields and all imported module bytes are mandatory closure inputs; forgetting
one is a certification bug, not permission to weaken the native claim.

**Portal effects (plan §15.5).** A Python scene can reach inputs no native
scene can. During a certified portal render, `fmn_python.effect_audit`
observes CPython's audit events. For the certified CLI, observation starts
before the scene module is imported, so import-time reads count.
- Every regular file the scene reads is hashed when it is opened and becomes
  a C6 item named by content and basename. Such names are host-path-free, so
  equal inputs give equal closures on every host. Each file is hashed again
  before publication, and a file that changed during the render is refused.
- The native SVG reader raises the same `open` event Python's `open()`
  does.
- Effects the closure cannot capture refuse certification with a named
  capability error, before any artifact is published:
  - a read of a missing or non-regular file (`/dev/urandom`, a FIFO);
  - a directory listing;
  - a subprocess;
  - a network connection or name lookup;
  - an SQLite connection.
- These are not scene inputs:
  - module loading, which is C1 through SceneSource, plus the runtime
    identity;
  - reads inside the interpreter's library directories and the installed
    portal;
  - source-line reads for diagnostics.
- Not observed, because they raise no audit event: environment variables,
  the host clock, and entropy (`os.urandom`, unseeded `random`).

## 3. Hashing rules

1. **One algorithm.** SHA-256 (fmn-hash's owned implementation, FIPS 180-4).
2. **Structural items** are serialized with fmn-hash's canonical Writer
   (versioned schema, defined field order, little-endian, canonicalized
   floats, trailing checksum) and hashed as those bytes. Field order changes
   are breaking (`major` bump) per D-17.
3. **Byte items** are hashed raw, then bound to their **virtual path** (the
   path the scene sees, not the host path) by hashing
   `serialize(virtual_path, content_digest)` structurally.
4. **Aggregation is ordered.** The closure digest is the SHA-256 of the
   canonical serialization of the item list, ordered by item number then by
   virtual path (byte-lexicographic). No unordered folding anywhere.
5. **Absence is encoded.** An item that does not apply (e.g. no ffmpeg
   invoked) is serialized as an explicit `absent` marker, never skipped —
   otherwise "absent" and "forgot to hash" would collide.
6. **Directory outputs use a canonical tree digest.** A PNG sequence hashes
   the domain `fmn-png-sequence-tree/v1`, then each published member in frame
   order as `(leaf-name length, leaf name, byte length, raw bytes)`, followed
   by the frame count. The atomic `FMN_COMPLETE` publication marker is not an
   image member and is excluded. Single-file outputs hash their exact bytes.

## 4. The sidecar provenance manifest (schema sketch)

Emitted next to every certified artifact (and, reduced, for standard runs).
Serialized both as the canonical binary document (schema family `FMNP`) and
as a human-readable text rendering. Fields:

```
manifest_version        u16.u16 (schema major.minor)
mode                    standard | certified
closure_digest          sha256          — §3.4, the headline value
semantic_digest         sha256          — the closure without `platform/` items (derived)
items[]                 (item_id, virtual_path?, digest, detail?)
                        item_id is the C1–C10 class; one class may hold several
                        items, and (item_id, virtual_path) is unique
engine                  franken_manim commit/release + SUITE.lock digest
toolchain               nightly version, target triple, target-features
execution               engine id, SIMD tier, declared certified config
outputs[]               (artifact name in its output directory — never a host
                        path — kind, sha256)
                        kind ∈ {raw_frames, canonical_png, wav, encoded*}
                        (*encoded artifacts are listed, marked uncertified)
journal_ref?            replay-journal id when the Studio/journal is live
```

**Semantic digest and the cross-platform check.** Items whose virtual path
begins with `platform/` describe the build platform, not the render's input:
today the target triple and target-feature set (C3 `platform/target`). They
stay in the closure digest, so a manifest still names the exact executable.
The *semantic digest* is the same canonical aggregation over every other item
(schema `FMNP` 12), so two certified platforms that render the same input
share it. The certified promise is checkable from manifests alone: equal
semantic digests, both certified, imply equal certified output digests.
`fmn_output::ProvenanceManifest::compare` reports exactly that
(`certified_bits_agree`), and `fmn-manifest-compare A.fmnp B.fmnp` exits 0
when it holds, 1 when the input matches but a certified output differs (a
determinism failure), and 2 when the inputs differ (not comparable).

Two manifests with equal `closure_digest` and equal certified platform MUST
list identical `outputs[]` digests for certified artifact kinds. That
sentence is the whole product promise, and it is what G4b's CI enforces
across the matrix.

## 5. The certified matrix — FROZEN by G0-6 (fm-zn9, 2026-07-25)

| Platform | Status | Basis |
|---|---|---|
| **linux-x86-64** | **certified** | measured bit-identical, G0-6; **re-measured on the engine's corpus, fm-ig3** |
| **linux-aarch64** | **certified** | measured bit-identical, G0-6 (qemu-user; see the caveat below); **re-measured, fm-ig3** |
| **macos-aarch64** | **certified** | measured bit-identical, G0-6; **re-measured, fm-ig3** |
| windows-x86-64 | **out of the certified matrix** (ADR-0019) | functional CI only; bit-certification ruled out until measured on native Windows hardware — revisit trigger named in the ADR |

Adding a platform to the certified list is an ADR, not a CI-config change: the
list *is* the promise `--reproducible` makes, and widening it silently would
mean shipping a promise nobody measured.

**Re-measured on the real engine, 2026-07-26 (fm-ig3).** G0-6's evidence was one
frame from a spike that is not a workspace member. The certified CPU engine's own
corpus — three frames carrying gradient fills, a winding hole, a self-intersecting
pentagram whose centre fills under the nonzero rule, all four joint settings
including two miter-limit escapes, round caps at three widths, an arc-length
taper, both hinted fill routes and the unhinted general path, an inner border, a
backstroke and a sub-AA hairline — is locked as **one** `Scope::Certified` lock
and passes on all three legs:

| Platform | libc | Execution | Result |
|---|---|---|---|
| linux-x86-64 | glibc | native | blessed here; 3/3 artifacts |
| linux-aarch64 | musl | qemu-user (cross-compiled) | 3/3 identical |
| macos-aarch64 | Darwin libSystem | native, M4 Pro | 3/3 identical, plus 631 other tests green |

The two W1 self-goldens (`geom_lifecycle.v1`, `stage_lifecycle.v1`) were measured
identical on the same three legs and **graduated from per-platform locks to one
certified lock** in the same pass. PG-5's {1,4,16}-thread sweep runs inside each
leg, so thread-count inertness is measured on real multicore hardware rather than
inferred from a single machine.

**The certified raster arithmetic is floating-point.** ADR-0010 resolves OQ-1:
`f64`/`f32` screen-space arithmetic with fmn-dmath transcendentals, fixed-order
reductions and no FMA is bit-identical across the matrix, so §10.5's canonical
fixed-point raster boundary is **retired, not deferred**, and no precision or
rounding constants enter this closure. The four properties that result rests on
— fmn-dmath owning every certified transcendental, no FMA contraction,
fixed-order reductions, IEEE-754 basic operations only — are load-bearing parts
of the closure, not implementation details: an engine that broke any of them
would produce a manifest claiming reproducibility it no longer has.

**One caveat, recorded rather than buried.** G0-6's linux-aarch64 leg ran under
qemu-user emulation; no aarch64 Linux hardware is reachable by this program
today. QEMU's softfloat is IEEE-754 correct for the basic operations, and
nothing on the certified path uses anything weaker, so the leg is valid evidence
about the *arithmetic* — but it is not evidence about an aarch64 Linux
*machine*, and W1's CI matrix (fm-sol) replaces it with real hardware when any
becomes available. Until then, linux-aarch64's certified status rests on
emulated execution and this sentence.

## 6. Consumers

- **G4b (fm-yp0)** implements enforcement end to end.
- **W9's replay journal (fm-y7u)** records RNG substream states and content
  hashes of everything read — its hash rules are §3's, not its own.
- **W10's perf rig** keys versioned baselines by closure-relevant identity
  (engine, tier, config) using the same serialization.
- **The self-golden rig** (crates/fmn-conformance/src/golden.rs) is the
  mechanical template: content hashes via fmn-hash, per-platform lock files
  now, and — since §5 is frozen and the arithmetic is portable — **one
  certified lock (`Scope::Certified`) for artifacts on the certified path**.
  G0-6 already demonstrates the shape: its frame digest is locked as a single
  cross-platform constant rather than three per-platform ones, so the property
  fails on whichever machine breaks it instead of waiting for a sweep.
  **Live since 2026-07-26 (fm-ig3):** `goldens/certified_engine.certified.lock`
  carries the certified CPU engine's raw-frame corpus under that scope, and
  `tests/certified_engine.rs` runs PG-5's {1,4,16} thread sweep over it per
  commit.
- **C7 and C10 have an implementation** (fm-ig3): `fmn_render::engine::journal`
  serializes the execution-engine identity — engine, SIMD tier, semantic renderer
  version — together with the declared certified configuration, through fmn-hash's
  canonical Writer under §3.2's rules. The tile dimensions are in it deliberately:
  the analytic fill's per-tile carry is a different *association* of the same sum
  than a longer row's prefix, so tiling is invariant to floating-point tolerance
  rather than to bits, and a certified run pins the dimensions instead of resting
  on an invariance the arithmetic does not owe (ADR-0013). `EngineKind::certifiable`
  is where "certified requires the certified CPU engine" stops being prose.
- **ADR-0010's four properties are enforced, not assumed** (fm-ig3):
  `tests/certified_arithmetic.rs` sweeps every certified crate on every commit for
  platform transcendentals and for hand-written FMA. It found sixteen live libm
  call sites when it was written, two of them inside the frame G0-6 itself hashed;
  closing them needed ADR-0014, which makes fmn-dmath the DAG root so the funnel is
  reachable from every crate that computes.
