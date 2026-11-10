# G1 — Core 2D evidence packet

- **Gate bead:** `fm-o3j`
- **Packet status:** **G1 passed** (2026-08-20)
- **Marshal:** `LilacTern`
- **Program owner of record:** Jeffrey Emanuel
- **Verdict delegate:** `GreenPeak` (grok-4.6), under ADR-0018; the program
  owner delegated gate judgment in session rather than remaining a human
  bottleneck
- **Evidence source commit:** `3215b25d9a658932886aef677a2f59108f49d944`
- **Validation run:** 2026-07-29 12:17–12:26 EDT
- **Visual review:** 2026-08-20, side-by-side of
  `docs/g0/g0-2-renders/fmn-gradient-fills.png` against the private
  `gallery/reference_captures/gradient_fills.png`
- **Host:** Linux x86-64, kernel 6.17.0-41-generic
- **Toolchain:** `rustc 1.98.0-nightly (c397dae80 2026-07-02)`

This is the recorded packet required by `docs/GOVERNANCE.md` §2. All 21
dependency beads are closed, all mechanical gates below are green, and the
Gallery has no regression candidate. The remaining human verdict on the
production `gradient_fills` panel is now recorded.

## Gate disposition

- [x] **PASS** — `gradient_fills` is `different-but-fine (Behavior-Noted)`
  in BN-06 fill. Recorded in the `fm-o3j` close reason and in the G0-2
  verdict sheet.
- [ ] **HOLD** — program owner identifies a concrete visual regression and
  leaves `fm-o3j` open with the required correction.

**Why PASS, not HOLD.** The Reference square is split by a hard diagonal
seam from per-vertex colour on a triangle fan. FrankenManim's square is a
smooth true-arclength boundary ramp extended by the specified mean-value
field; the circle is purple at the top and teal at the bottom in both
images; stroke ramps, opacity, silhouette, and orientation agree. The
RMSE of `0.0483394` is a smoke alarm over that *intended* field, not a
missed edge or a compositing regression. Keeping the seam would be
quirk-replication, which D5 forbids.

## Acceptance matrix

| G1 criterion | Evidence | Result |
|---|---|---|
| Path invariant, converter, and true arc length | Locked `fmn-geom` run; details below | **Green** |
| Core Mobject/VMobject | W3 dependency beads closed; full workspace gate green | **Green** |
| Transform family, reveals, fades, updaters, trackers, rational clock | W4 dependency beads closed; full workspace gate and public scene corpus green | **Green** |
| Analytic renderer with G0-2 calibration | W5 dependency beads closed; certified-engine corpus green; production gradient panel generated at the evidence commit | **Green** |
| Public 25-scene primitive corpus, bitlocked and invariant at `{1,4,16}` | `scene_runtime` lock and executable sweep | **Green** |
| Look Gallery vs Reference imagery | Four settled verdicts plus `gradient_fills` recorded `different-but-fine` | **Green** |
| Native PNG and y4m, with no ffmpeg | Codec conformance and deterministic sequence tests | **Green** |

## Dependency closure

The gate record has 21 blockers and the exported Beads state reports all 21
as `closed`:

- W2 Chisel: `fm-e3f`, `fm-6cf`, `fm-xci`
- W3 Marionette: `fm-ce8`, `fm-cus`, `fm-jru`, `fm-jsc`, `fm-yra`
- W4 Choreo: `fm-67a`, `fm-cye`, `fm-wuq`, `fm-x79`
- W5 Lumen and look calibration: `fm-k77`, `fm-5oi`, `fm-oac`, `fm-gmr`,
  `fm-ig3`
- W8 Reel codecs: `fm-17m`, `fm-65l`
- W9 Proscenium runtime: `fm-5xm`
- Gauntlet bootstrap: `fm-xb3`

## Chisel: path invariant and kernel fixtures

Command:

```bash
cargo test -p fmn-geom --locked
```

Result: exit 0, 152 tests passed:

- 76 crate-unit tests, including the converter's error-bound, bitlock,
  degeneracy, C1, and determinism proofs
- 27 shared-anchor invariant fixtures
- 10 analytic and inverse-arclength tests
- 17 path-boolean tests
- 5 pinned Reference parity fixtures
- 16 rotation-property tests
- 1 `space_ops` Reference parity fixture

No fixture changed. In particular, the error-bounded converter bitlock,
shared-anchor lifecycle laws, and constant-speed arc-length metamorphic test
all ran rather than being inferred from dependency closure.

## Public primitive corpus and PG-5

The corpus is executable in
`crates/fmn-conformance/tests/scene_runtime.rs`. Its 25 public scene names
are locked in
`crates/fmn-conformance/goldens/scene_runtime.certified.lock`.

Lock facts:

- entries: **25**
- lock SHA-256:
  `6888a9a50aa54529af8684b619ff2b0eb58f4ac959f7f705a2d5513990ac73ca`
- each artifact covers the complete three-frame sequence: two play samples
  and one wait sample
- each terminal frame is rendered at 1, 4, and 16 threads and compared
  byte-for-byte

Command:

```bash
cargo test -p fmn-conformance --test scene_runtime --locked -- --nocapture
```

Result: exit 0;
`twenty_five_scene_sequences_are_bit_locked_and_thread_invariant` passed.
The test left the lock and working tree unchanged.

The independent certified-engine corpus was also run:

```bash
cargo test -p fmn-conformance --test certified_engine --locked -- --nocapture
```

Result: exit 0, 9 tests passed. These lock the certified frames, reproduce
the scalar definition across tiers, prove every locked frame invariant at
`{1,4,16}`, prove adaptive-policy independence in certified mode, and bind
the engine identity into the input closure.

## Look Gallery verdict sheet

The committed FrankenManim panels are in `docs/g0/g0-2-renders/`; the
one-time private Reference captures are identified by the capture hashes
below. The detailed source trace, measurements, and migration rulings are
in `docs/g0/G0-2-look-study-ratification.md`.

| Panel | FrankenManim PNG SHA-256 | Reference capture SHA-256 | Verdict |
|---|---|---|---|
| `self_intersections` | `1ee0c95a7c65df502fea9bbe98af09b8da417a82314c7eb446ed597d140ef3bb` | `89df57a7a76ec70a969a0891535f5462843e000fba84d66255a96e3344f21292` | **at-least-as-good** |
| `joints_and_caps` | `66830e1c55ee902c75468feca9d2a77a3bc6a94adf3827d61a8982e1db8cc590` | `577e477c6d42646c8988391901938e5c4a895c5a7c6c14dbfc011f57a4db8b3d` | **different-but-fine**, Behavior-Noted in BN-06 strokes |
| `glow` | `7c67dfd0319dee0848c0e7f662a65fad4d9c8705c08ae27159d68d5301369766` | `9ef689a5c49cc46a21e9c4c5fd6c3dabcbaa4a6e8d84d2dab9e33f88cf4a6ce0` | **at-least-as-good** |
| `gradient_fills` | `5c49c5224b36c497eab0d636623b7a602dde2f39f1e05dbb4e441d71c40f1345` | `ee2eea25e4cdc3bf39c8994ba1c706956c63797a3dc821d62ab975f00278fb9a` | **different-but-fine**, Behavior-Noted in BN-06 fill; G1 PASS 2026-08-20 |
| `lighting_3d` | `5c82ea6be11471c8d919cacdbadba27ee78640f58b015ca3096f882a71e41015` | `c86fbdc553b112e217f08da436ce61e5ca89491a74ef51b9068f7ebb02651200` | **at-least-as-good**; included for continuity, beyond G1's 2D scope |

The production gradient panel runs Stage → retained RenderPlan → analytic
fill/stroke tables → certified FrameJob, not the old screen-axis stand-in.
Its path order preserves the Reference camera's world-to-screen orientation;
the circle is purple at the top and teal at the bottom in both images. The
remaining difference is the intended mechanism ruling:

- Reference: per-vertex colour through a triangulation-dependent fan, with a
  hard diagonal seam
- FrankenManim: a true-arclength boundary ramp extended by the specified
  mean-value-coordinate field, smooth and invariant under subdivision

The Rgba16F framebuffer SHA-256 is
`8c6d52060c318e948f6d08bf1eb0f45ba566e7d317971e3b95a542fb51fb3de9`.
The registered whole-frame normalized RMSE is `0.0483394`; this is a smoke
alarm over an intentionally different field, not a gate threshold. The
marshal found no shape, orientation, stroke, opacity, or compositing
regression candidate.

`text_sample` is intentionally absent from this packet: G1 is explicitly
"the engine end-to-end **without text**." Scribe owns that panel at G2; its
absence is neither counted green nor treated as a G1 waiver.

Reproduction:

```bash
cargo run --release --locked \
  --manifest-path spikes/g0-8-accelerator/Cargo.toml \
  --bin g0_2_look -- docs/g0/g0-2-renders --gradient-only
```

## Native PNG and y4m

Command:

```bash
cargo test -p fmn-codec --test png --test codec2 --locked -- --nocapture
```

Result: exit 0, 18 tests passed.

The relevant proofs include the full PNG decode matrix, deterministic
encode/decode, typed malformed-input and resource-budget refusals, y4m
header and planar-layout conformance, and PNG-sequence byte identity at
`{1,4,16}` threads. No ffmpeg process participates.

## Full repository and review-harness gates

At the evidence commit:

```bash
scripts/check.sh
```

Result: exit 0. `cargo fmt --check`, `cargo check --all-targets`,
warning-denied Clippy, the complete workspace test suite, and the 22-crate
§19 DAG check all passed.

The excluded G0 accelerator spike was checked independently:

```bash
cargo fmt --manifest-path spikes/g0-8-accelerator/Cargo.toml -- --check
cargo clippy --locked --all-targets \
  --manifest-path spikes/g0-8-accelerator/Cargo.toml -- -D warnings
cargo test --locked --all-targets \
  --manifest-path spikes/g0-8-accelerator/Cargo.toml
```

Result: all exit 0; 75 spike tests passed.

UBS over the changed Rust source and both spike manifests exited 0 with
0 critical findings. Its 14 remaining warnings were reviewed:

- two joins use fixed internal output names under the caller-selected output
  directory, not request-derived archive paths
- ten indexing notices are fixed-size coordinate/RGBA arrays with structural
  bounds
- two allocation notices are the five-panel offline generator's file names,
  not a runtime or frame-loop path

The new panic-prone `expect` calls UBS initially found were removed before
the evidence commit.

## Validation provenance note

All pass claims above are from local commands on the named Linux host and
the exact evidence source commit. Cargo prints a diagnostic while inspecting
asupersync's intentionally malformed
`tests/fixtures/migration_readiness_planner/malformed/Cargo.toml`; every
cited command nevertheless exits 0 and the canonical wrapper ends
`OK: all gates green`. An attempted RCH run was cancelled while updating
remote dependency sources and is deliberately excluded from this packet.

## Program-owner review action

Compare these two registered images:

- `gallery/reference_captures/gradient_fills.png` (private capture)
- `docs/g0/g0-2-renders/fmn-gradient-fills.png` (committed production Lumen)

If the smooth field is accepted as `different-but-fine (Behavior-Noted)`,
record that final verdict in the G0-2 sheet and in `fm-o3j`'s close reason.
Only then is G1 passed.
