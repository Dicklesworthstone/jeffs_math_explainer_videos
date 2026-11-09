# AGENTS.md — franken_manim

> Guidelines for AI coding agents working in this Rust codebase.

---

## RULE 0 — THE FUNDAMENTAL OVERRIDE PREROGATIVE

If I tell you to do something, even if it goes against what follows below, YOU MUST LISTEN TO ME. I AM IN CHARGE, NOT YOU.

---

## RULE 0.5 - SUITE-WIDE RULES LIVE IN /data/projects/AGENTS.md

The suite-wide rules in **`/data/projects/AGENTS.md`** bind you here too. Read it. Two sections
are load-bearing for perf work and are NOT duplicated below, so they cannot drift out of sync:

- **`## Named Reward-Hacking Patterns (ALL FORBIDDEN)`** — 12 named patterns, several already
  observed in this suite: gate self-weakening (and the exact price of a legitimate gate fix),
  proof-class inflation, golden regeneration reflex, commit-stream pumping, tautological tests,
  easy-lever cherry-picking, close-pump abuse, scope-splitting, spec-editing as progress,
  conformance metastasis, dependency smuggling, bench-path hardcoding.
- **`### Work-Graph Discipline`** — JSONL is truth and `beads.db` is disposable, `br sync
  --import-only` after every pull, single-writer on graph structure, closure on cited evidence
  with blocker beads gated on their named probe, `br dep cycles` stays empty.

The three that most often decide whether a number here is real: a **self-speedup is
MAINTENANCE, not a win** — a win needs the incumbent live in the SAME invocation; **never
weaken a gate to land a change**, and if a gate is genuinely defective, meet the evidence
standard and publish the win/lose split of what the fix admits; and **reporting a loss is a
success** — one line, revert, next lever, no retraction narrative.

---

## RULE NUMBER 1: NO FILE DELETION

**YOU ARE NEVER ALLOWED TO DELETE A FILE WITHOUT EXPRESS PERMISSION.** Even a new file that you yourself created, such as a test code file. You have a horrible track record of deleting critically important files or otherwise throwing away tons of expensive work. As a result, you have permanently lost any and all rights to determine that a file or folder should be deleted.

**YOU MUST ALWAYS ASK AND RECEIVE CLEAR, WRITTEN PERMISSION BEFORE EVER DELETING A FILE OR FOLDER OF ANY KIND.**

---

## Irreversible Git & Filesystem Actions — DO NOT EVER BREAK GLASS

1. **Absolutely forbidden commands:** `git reset --hard`, `git clean -fd`, `rm -rf`, or any command that can delete or overwrite code/data must never be run unless the user explicitly provides the exact command and states, in the same message, that they understand and want the irreversible consequences.
2. **No guessing:** If there is any uncertainty about what a command might delete or overwrite, stop immediately and ask the user for specific approval. "I think it's safe" is never acceptable.
3. **Safer alternatives first:** When cleanup or rollbacks are needed, request permission to use non-destructive options (`git status`, `git diff`, `git stash`, copying to backups) before ever considering a destructive command.
4. **Mandatory explicit plan:** Even after explicit user authorization, restate the command verbatim, list exactly what will be affected, and wait for a confirmation that your understanding is correct. Only then may you execute it.
5. **Document the confirmation:** When running any approved destructive command, record (in the session notes / final response) the exact user text that authorized it, the command actually run, and the execution time.

---

## Branch Policy

- Primary branch is `main`.
- Do not reference `master` in docs/scripts.
- If release instructions require sync, push `main:master` after `main`.

---

## Project Mission

`franken_manim` is a **sovereign, deterministic, programmatic mathematical-animation engine in pure Rust** — a ground-up rebuild of manim (Grant Sanderson's engine behind 3Blue1Brown) on the Dicklesworthstone FrankenSuite. Its native Rust library, CLI (`fmn`), and live Studio ship without CPython. The separately installed `fmn-python` PyO3 portal presents the `manimlib` surface inside a supported host CPython so existing manim scene code runs source-unedited.

The contract is **API compatibility and semantic fidelity — deliberately not output identity**. Under manim's familiar names, FrankenManim does the *correct* thing: `MoveAlongPath` moves at true constant speed, `get_arc_length` returns the arc length, colors composite in a defined color model, the clock never drifts. The Reference — `3b1b/manim` @ `6199a00d4c1b1127ebe45cb629c3f22538b10e13` — is a design oracle and an aesthetic bar, **never a pixel warden**. Every deliberate divergence is a documented **Behavior Note** (§16.8).

The leapfrogs, in brief:

- **One-binary native installation.** The Rust API, standalone `fmn`, and Studio need no LaTeX, dvisvgm, Pango, fontconfig, system fonts, or CPython. Typesetting — text *and* TeX-style mathematics — is native, built on bundled fonts (Computer Modern included) and **fmd-math**, a clean-room TeX-math layout engine that lands as workspace crates in franken_markdown. The optional `fmn-python` wheel is outside this one-binary claim and requires a supported host CPython plus NumPy.
- **One external tool.** `ffmpeg`, sandboxed, optional (native y4m/PNG-sequence/GIF outputs exist), used only for video encode/mux/transcode. There is no second tool and no carve-out for one. The standalone engine never locates or launches CPython; the optional Python portal is imported by its host interpreter.
- **Certified determinism.** `--reproducible` yields bit-identical raw frames, canonical PNGs, and WAV across the certified platform matrix from a content-hashed input closure. Encoded video is equivalence-classed, never bit-promised.
- **Pinned bits, free scheduler.** One semantic renderer (Lumen) over multiple execution engines — certified CPU, fast CPU (SIMD tiers), and a standard-only Accelerator Annex (Metal/CUDA via frankentorch) — where every schedule reproduces certified bits exactly and everything that can't is quarantined to `standard` and labeled.
- **Farm-class scaling from a single scene.** Pure-segment frame parallelism, pipelined frame stages, and topology-aware render teams saturate a 96-core workstation while certified output stays identical at any thread count.

**The single source of truth for what we are building and why is [`COMPREHENSIVE_PLAN_FOR_THE_DESIGN_OF_FRANKEN_MANIM.md`](docs/planning/COMPREHENSIVE_PLAN_FOR_THE_DESIGN_OF_FRANKEN_MANIM.md)** (Revision 4). Read it before writing any subsystem.

### What we stand on (the FrankenSuite substrate)

- `franken_numpy` (`fnp-*`) — arrays, dtypes (the `RecordBuffer`'s native representation), linalg, `.npy` fixtures, and **the one RNG**: PCG64DXSM, bit-exact to NumPy for explicit seeds.
- `frankenscipy` (`fsci-*`) — `linear_sum_assignment` + `cdist` (shape matching), adaptive RK45 with dense output (StreamLines), banded/dense solves (smoothing), quadrature (arc length); scipy `Rotation` *conventions* kept as API semantics.
- `franken_markdown` (`fmd`, plus the new **`fmd-font`** and **`fmd-math`** crates this program contributes upstream) — the typography stack: sfnt parsing + glyf outline decoding, bundled OFL faces (Computer Modern, IBM Plex Sans, CM Typewriter, Noto Sans Math), the syntax highlighter, and the native TeX-math layout engine.
- `franken_networkx` (`fnx-*`) / `frankenpandas` (`fp-*`) — enhanced-tier Graph and Data mobjects; never blocking core gates.
- `frankentorch` (`ft-*`) — the **only** GPU gateway (Accelerator Annex: Metal now, CUDA via the upstream ledger); `NeuralNetworkMobject` content.
- `asupersync` — multi-scene `batch` farms and the deterministic lab for scheduler tests; **never in the frame loop**.

Everything is commit-pinned in `SUITE.lock`; CI builds only from the lock; upgrades are deliberate and Gauntlet-diffed.

---

## Product Shape

The project must be all four at once:
1. A reusable Rust library — idiomatic but recognizably manim: snake_case, `Default` config structs + builders generated from the one API schema; the fluent surface is whatever G0's compiling prototype proves.
2. A CLI `fmn` keeping the Reference's flag surface where it still means something, plus `--reproducible`, `fmn batch` (asupersync), and `fmn doctor` (capabilities, ffmpeg fingerprint, derived ExecutionPlan).
3. The **Studio** — supervisor + isolated scene-worker subprocess, crash isolation, journal replay, timeline scrubbing, inspector (family tree, live record fields, span maps), multipart-PNG preview stream, a kitty/sixel TUI; loopback-only with an explicit security model.
4. **fmn-python** — the optional, separately installed `manimlib`-compatible PyO3 bridge, loaded by a supported host CPython: real subclassing with MRO/override dispatch, writable `__dict__`, live NumPy views under the §8.2 view protocol, `copy`/`deepcopy`/pickle. Source compatibility and correct, beautiful output — explicitly not frame reproduction.

---

## Spec-First Workflow

Implementation follows the plan, not ad-hoc invention. Read in this order:
1. [`COMPREHENSIVE_PLAN_FOR_THE_DESIGN_OF_FRANKEN_MANIM.md`](docs/planning/COMPREHENSIVE_PLAN_FOR_THE_DESIGN_OF_FRANKEN_MANIM.md) — the Reference anatomy (§1), the foundation audit (§2), the dependency & safety doctrine (§3), the product contract (§4), all ten subsystems (§6–§14), the two front doors (§15), the Gauntlet (§16), the performance model and CI gates (§17), the crate map (§19), the workstreams and gates (§20), the risk register (§21), and the decision log (§23).
2. **The decision log (§23)** — D-01 … D-24 are binding unless amended there. The open questions (OQ-1 … OQ-12) each have an owner gate/workstream; do not silently resolve one in code.
3. **Appendix A** (the 257-class census), **Appendix B** (kept look vs replaced mechanism), and **Appendix C** (the Reference-defect register and its rulings) — the normative contracts library work must honor.

**Hard rule: gates are integration checkpoints, not scope reductions.** There is no MVP. G0's eight spikes (object model, look study, fmd-math architecture, corpus harvest, Python extensibility, determinism, dependency closure, accelerator proof) retire the load-bearing unknowns **before** W2–W11 freeze interfaces. A workstream may implement a *subset* of a final abstraction — never a substitute for it.

---

## The FrankenManim Engineering Doctrine (READ THIS BEFORE WRITING CODE)

These are the constitutional rules from §3 and §10.5 of the plan. Violating any of them is a revert.

1. **The governed closure (D1).** Authoritative fmn crates introduce **no new unreviewed direct runtime dependencies**. Allowed: `std`, the exact pinned nightly, and the FrankenSuite (fnp, fsci, fmd + fmd-font + fmd-math, fnx, fp, ft, asupersync). The complete transitive closure is pinned and per-package allowlisted (checksum, features, license, build-script/proc-macro status, unsafe-audit status, owner); CI fails on any unlisted package. Pre-authorized beyond the suite: PyO3 (fmn-python only), clap (`cli`), wasm-bindgen (`wasm`).
2. **One external tool (D2).** **ffmpeg is the only subprocess the engine will ever invoke** — encode, mux, transcode — under the full security protocol (argv-only, private temp dirs, timeouts + process-tree cancellation, output-size limits, env allowlist, atomic publication, path *and content hash* into provenance). Its absence yields a **capability error naming the alternative** — never a silent format substitution. No TeX, no font tooling, no downloader. Network exists only behind the host-provided `AssetFetcher` trait; no TLS in core. `fmn` never embeds, locates, or spawns CPython; Python scene files enter only through the separately installed portal that a host CPython imports.
3. **The unsafe posture (D3).** `#![forbid(unsafe_code)]` in every authoritative crate. The only non-forbid crate is `fmn-python` (PyO3 expansion). SIMD is `std::simd` in **crate-wide build tiers** (portable / x86-64-v3 / x86-64-v4 / aarch64+NEON) selected by SUITE.lock flags and `cfg(target_feature)`; ADR-0016 forbids function-level `#[target_feature]`, and no per-call `unsafe` dispatch trampoline exists anywhere.
4. **Not built here (D4).** Arrays/RNG → fnp. ODE/quadrature/assignment/solves → fsci. Fonts, math typesetting, highlighting, PDF → fmd. Graphs → fnx. Dataframes → fp. Tensors/GPU → ft. Concurrency → asupersync. Video encoding → ffmpeg. FrankenManim's owned surface: the geometry kernel, the mobject engine, the animation engine, the scene runtime, the rasterizer, text layout over fmd-font, the manim-facing typesetting integration, codecs/cache/platform, and the output pipeline.
5. **Correct by default, documented when different (D5).** Where behavior differs semantically from the Reference — true arclength, one RNG, the rational clock, color science, fixed Reference bugs (Appendix C) — the difference is deliberate, correct, and recorded in the **Behavior Notes** with migration guidance. There is no quirk-replication obligation anywhere in the program.
6. **The parallelism contract (D18, §10.5 a–f).** A frame's bits are a pure function of *(begin-state snapshot, α, frame-indexed RNG state, input closure)* — never of thread count, scheduling order, or machine load. Tiles are write-disjoint and composite in fixed order; reductions are fixed-order and lane-count-independent; no FMA or fast-math on certified paths; frames emit in frame-index order; every engine/backend identity is journaled into the input closure. **Three permanent refusals:** GPU work in the certified path; adaptive/variable frame sampling; per-thread RNG consumed in completion order.
7. **Semantics are sacred; the scheduler is free.** The six-step frame order (animation `update_mobjects(dt)` → `interpolate(alpha)` → time advance → scene updaters → capture → emit), manim's nominal sample points on the `RationalFrameClock`, the shared-anchor quad-path invariant, the render-order model, copy semantics, and the `.animate` builder rules are all **exact semantics**. Quality/backend knobs (AA policy, thread count, execution engine, pixel format) may change speed, never meaning.
8. **Correctness and beauty outrank speed.** The Gauntlet's order of operations is fixed: instrument (§17.1) → eliminate work (§10.8 retained plan) → pipeline (§17.4) → vectorize (§17.3) → offload (§17.5). A faster path that drifts a self-golden or regresses the Look Gallery is reverted, not landed.

---

## The Ten Subsystems (names you must use)

| Name | Crate(s) | What it is |
|---|---|---|
| **Substrate** | fmn-core/dmath/hash/config/platform | constants, color, the one RNG (PCG64DXSM + named substreams + keyed per-frame forks), deterministic transcendentals, canonical hashing, config, capability traits + HardwareTopology |
| **Chisel** | fmn-geom | the geometry kernel: shared-anchor quad paths, one error-bounded cubic→quad converter, true arclength, path booleans (flatten-first), SVG document processor, isolines, ear-clip |
| **Marionette** | fmn-mobject | Stage arena + generational handles + CoW snapshots; the RecordBuffer + view protocol + lazy revisioned render mirrors; family/positional API; manim copy semantics; updaters (corrected) + `.animate` |
| **Choreo** | fmn-anim | the Animation contract; the RationalFrameClock; the six-step frame order + FramePacket freeze; five mechanisms → 80 classes; segment purity classification for frame parallelism |
| **Lumen** | fmn-render | one semantic renderer, multiple engines: analytic winding fill, true curve-distance strokes, kept 3b1b look constants, adaptive AA, the compiled retained render IR, the retained compositor, the Accelerator Annex |
| **Scribe** | fmn-text, fmn-tex (⇄ fmd-font, fmd-math) | native shaping/layout/markup; Tex/TexText over fmd-math with native span maps (the two-render alignment hack is dead); preamble packs; the coverage ratchet |
| **Menagerie + Atlas** | fmn-library | the 161-class mobject library; coordinate systems; fields; de-TeX'd Brace/Matrix/Decimal/marks; enhanced fnx/fp mobjects |
| **Proscenium** | fmn-scene, fmn-studio, fmn-cli | scene runtime, events, InteractiveScene, supervisor + worker iteration, replay journal + effect model, the Studio, the CLI |
| **Reel** | fmn-output, fmn-frame/codec/cache | frame buffers + pixel formats, native PNG/JPEG/GIF/WAV/y4m codecs, deterministic parallel DEFLATE, the negotiated ffmpeg boundary v2, the ordered async emitter, the sound mixer, the content-addressed cache |
| **Gauntlet** | fmn-conformance | the symbol-granular Parity Ledger, the one API schema, correctness oracles, self-goldens, the Look Gallery, the engine-equivalence suite, fuzzing, the PG performance gates |

Dependency edges point strictly downward per §19; feature axes `wasm`, `accel-annex` (`metal`/`cuda`), `batch`, `cli` default-off.

---

## Determinism Contract (the short version)

- **`standard`**: deterministic given a seed on a given build/platform; best-effort across platforms; fast paths, SIMD tiers, FMA, annex engines all allowed — always labeled, never silent.
- **`certified`** (`--reproducible`): the content-hashed input closure (§16.7) ⇒ bit-identical raw frames, canonical PNGs, and WAV across the certified matrix (linux-x86-64, linux-aarch64, macos-aarch64; windows-x86-64 excluded by ADR-0019). fmn-dmath transcendentals only; canonical raster arithmetic; the scalar path is the definition every SIMD tier must match bit-for-bit. ffmpeg products are excluded from certification **by construction**.
- Thread-count-independent output is verified at {1,4,16} threads per commit and {32,96}+ weekly (PG-5).
- Every adaptive choice (`standard`-only autotuning, annex selection) is journaled; `certified` runs its fixed declared configuration.

---

## Code Editing Discipline

### No Script-Based Changes
**NEVER** run a script that mass-edits code files. Brittle regex transforms create more problems than they solve. Make code changes manually (use parallel subagents for many simple changes; do subtle/complex changes methodically yourself).

### No File Proliferation
Revise existing files in place. **NEVER** create `rendererV2.rs` / `chisel_improved.rs` / `anim_enhanced.rs`. New files are reserved for genuinely new functionality; the bar is incredibly high.

---

## Backwards Compatibility

We are in early development with **no users**. Do things the **RIGHT** way with **NO TECH DEBT**. Never create compatibility shims or wrappers for deprecated APIs. Just fix the code directly. (The two standing exceptions, per the plan: the *manim* API surface itself — which is the product contract, governed by the Parity Ledger and Behavior Notes, not by this rule — and durable serialization formats, versioned from day one per §6.7.)

---

## Toolchain

- Rust 2024 edition on an **exact pinned nightly** recorded in `SUITE.lock` (no "or later") — required for `std::simd`; build tiers use SUITE.lock's crate-wide target-feature flags under ADR-0016.
- `#![forbid(unsafe_code)]` at every authoritative crate root; `fmn-python` is the single exception (PyO3 expansion) and never relaxes anyone else's forbid.
- `fmn-python` wheels target the CPython ABI policy in ADR-0015 and require host CPython + NumPy; those portal requirements never enter the standalone `fmn` artifact.
- Cargo only, workspace per §19's crate map; cross-repo work (fmd-font, fmd-math) rides `UPSTREAM_LEDGER.md` and SUITE.lock — FrankenManim CI builds both repos from the lock.
- SIMD build tiers are capped at {portable, x86-64-v3, x86-64-v4, aarch64+NEON}; per-commit CI runs the two pinned profiles, the full matrix runs weekly (R22).

---

## Mandatory Checks After Substantive Changes

```bash
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
ubs $(git diff --name-only)
```

If any check fails, fix root causes before handing off.

### The `cargo test` gate (green-bar requirement)

`cargo test` is a **hard gate**: it MUST exit `0` before any change is handed off or a bead is closed. The convenience wrapper `scripts/check.sh` runs the complete local/owned-host policy in order and stops on the first failure. Hosted GitHub Actions availability is not part of the correctness contract; never substitute an unavailable hosted run for the exact local gate, and never treat hosted quota exhaustion as a product failure.

Beyond the bare gate, **every Gauntlet plane in §16 and every performance gate in §17.2 is a permanent release gate** — self-goldens are bit-locked and blocking; the engine-equivalence suite blocks engine changes; PG-5 determinism runs per commit; a Look Gallery regression blocks the gate that introduced it. A release may bypass a gate only with a public, expiring waiver.

---

## Testing Policy — the Gauntlet (plan §16–§17)

This is a design pillar with its own budget, not a QA appendix.

- **The e2e registration doctrine (fm-fjq, binding on every epic).** Unit tests live in-crate with the bead; **each feature bead also registers at least one e2e scenario when it lands user-visible behavior** — as data over `fmn-conformance/src/e2e.rs`'s API (scenario specs in `crates/fmn-conformance/tests/e2e_scenarios.rs`, grouped by class: render-matrix, determinism, failure-path, lifecycle, parity). The harness runs the fast tier per commit and the full matrix under `FMN_E2E_FULL=1`; every scenario gets deterministic logs, and every red run auto-bundles an FMNA repro. New surfaces register their first scenario with their owning bead (Python/Studio are pending placeholders, never stubbed).
- **Correctness oracles.** Analytic ground truths (arc length vs closed forms, boolean identities, winding invariants, TeX layout vs the published Appendix-G parameters, color round-trips); property/metamorphic tests restricted to valid laws; **structural fixtures against the Reference** where formulas intentionally coincide (constructor point arrays, family shapes, positional results — loose f32 tolerances, since we compute in f64).
- **Self-goldens.** FrankenManim's own outputs, bit-locked per platform (cross-platform under `certified`): geometry snapshots at lifecycle points plus frame hashes for the primitive and feature corpora. **The regression gate that actually blocks merges.**
- **The Look Gallery.** Side-by-side renders vs captured Reference imagery, human-reviewed, with perceptual metrics (SSIM, edge-distance, local-error percentiles) as smoke alarms, never hard gates. Verdicts: at-least-as-good / different-but-fine (Behavior-Noted) / regression (fix).
- **The engine-equivalence suite.** Fast-CPU and annex engines vs certified-engine reference frames under a versioned visual-equivalence budget; blocking for engine changes.
- **Fuzzing.** SVG, TTF, YAML-subset, TeX strings, PNG/JPEG — with resource-budget assertions (decompression bombs, nesting depth, pathological intersections, malicious font tables are DoS surfaces even in safe Rust). fmd-math must error precisely on arbitrary token streams — never hang, never garble.
- **Performance gates (PG-1 … PG-8 + PG-A)** on pinned bare-metal profiles: end-to-end vs the Python Reference (≤0.5× wall-clock at G2, ≤0.35× at G4), rasterizer throughput, 60 fps 1080p preview / 30 fps 4K export, <150 ms cold start, PG-5 determinism, memory + **zero steady-state per-frame heap allocations**, typesetting latency (<3 ms cold / <100 µs cached per formula), and per-class Python binding budgets. Annex profiles (PG-A) gate annex changes only — the CPU engine must stand on its own so acceleration can never mask a core regression.
- **The coverage ratchet.** fmd-math's occurrence-weighted TeX-construct coverage (from the G0 corpus harvest of the pinned `3b1b/videos` tree) is a public, headline project metric. An unsupported construct is a precise, named error — never silence, never garbage.

---

## Agent Ergonomics Requirements

CLI robot mode must be: stable versioned schema, deterministic where possible, explicit exit codes, line-oriented NDJSON, easy to pipe. Do not mix human decoration with machine output in robot mode. `fmn doctor` reports capabilities (ffmpeg fingerprint + hardware encoders, fonts, cache, the derived ExecutionPlan). Every bug report is a deterministic replay: one-command repro bundles (scene + input closure) and the sidecar provenance manifest are first-class DX artifacts (§18).

The repository's own operational interfaces follow the same rule. `.beads/issues.jsonl` is authority; `agent_brief.py` is the bounded situational projection; `agent_next.py` is the sole autonomous claim planner; `agent_claim_guard.py` issues the graph/policy/schema-bound token; `agent_claim.py` owns the bounded guarded mutation; and `generate_agent_brief.py` is the deterministic human rendering. Do not infer a claim from a raw issue list, from `bv` rank alone, or from the removed broad `agent_brief.py --format next` spelling.

---

## Program Governance (R9) — read before claiming a bead

The governance machinery lives in [`docs/GOVERNANCE.md`](docs/GOVERNANCE.md); the two rules you touch every session:

- **The activation and integrity check (GOVERNANCE.md §1).** At most **4 workstreams active** (≥1 bead `in_progress`) at once; G0 counts as one. Before any claim, validate the whole graph, inspect the canonical plan, and issue a guarded token. Exit `1` means graph/activation state is unsafe, exit `2` means malformed input, exit `3` means no claimable leaf, exit `4` means a stale token, and exit `5` means no verified executor receipt. Never activate work from a failed or empty plan.
- **The handoff checklist (GOVERNANCE.md §4).** The checkable form of "Landing the Plane" below — a handoff violating it is not a handoff.

```bash
python3 scripts/agent_brief.py --format json --check >/dev/null
python3 scripts/agent_next.py --format json --check
token="$(python3 scripts/agent_claim_guard.py --require)"
issue="${token##*:}"
```

Inspect `br show "$issue"`, current `main`, Agent Mail, file reservations, and active peers before invoking the executor. The token is not a lease.

Amendments to the plan's decision log (D-01…D-24), OQ resolutions, and policy rulings under standing rules land as **ADRs** (`docs/adr/NNNN-*.md`, template + worked examples in `docs/adr/`), with the plan trued up in the same commit. Upstream-bound primitives ride [`UPSTREAM_LEDGER.md`](UPSTREAM_LEDGER.md) under the §2.9 ritual (GOVERNANCE.md §6).

---

## Session Completion ("Landing the Plane")

The checkable version of this list is `docs/GOVERNANCE.md` §4. Before finishing a work session you MUST:
1. File beads issues for remaining work (anything needing follow-up).
2. Run quality gates (if code changed) — tests, clippy, fmt, `ubs`.
3. Update issue status — close finished work, update in-progress.
4. `br sync --flush-only` to export beads to JSONL, then `git add .beads/`.
5. Re-run `python3 scripts/agent_next.py --format json --check` after tracker mutation so the handoff names current graph state rather than the pre-change plan.
6. When a next leaf exists, issue a fresh `agent_claim_guard.py --require` token for the handoff; never reuse a pre-mutation token.
7. Hand off — summarize what changed, exact source commit, gates actually run + results, remaining risks/gaps, and the next concrete claimable leaf if one exists.

---

## MCP Agent Mail — Multi-Agent Coordination

A mail-like layer for agents to coordinate via MCP tools/resources: identities, inbox/outbox, searchable threads, advisory file reservations with human-auditable Git artifacts.

- **Register identity:** `ensure_project(project_key=<abs-path>)` → `register_agent(project_key, program, model)`.
- **Reserve files before editing:** `file_reservation_paths(project_key, agent_name, ["crates/fmn-render/**"], ttl_seconds=3600, exclusive=true, reason="fm-###")`.
- **Communicate with threads:** `send_message(..., thread_id="fm-###")`, `fetch_inbox`, `acknowledge_message`.
- **Prefer macros:** `macro_start_session`, `macro_prepare_thread`, `macro_file_reservation_cycle`, `macro_contact_handshake`.
- Common pitfalls: `"from_agent not registered"` → `register_agent` in the right `project_key` first; `"FILE_RESERVATION_CONFLICT"` → adjust patterns / wait / use non-exclusive.

---

## Beads (br) — Dependency-Aware Issue Tracking

This project uses [beads_rust](https://github.com/Dicklesworthstone/beads_rust) (`br`). Issues live in `.beads/` and are tracked in git. **`br` is non-invasive — it NEVER runs git.** After `br sync --flush-only`, manually `git add .beads/ && git commit`.

```bash
# Validate authority and issue the only autonomous claim token.
python3 scripts/agent_brief.py --format json --check >/dev/null
python3 scripts/agent_next.py --format json --check
token="$(python3 scripts/agent_claim_guard.py --require)"
issue="${token##*:}"

# Inspect the selected Bead and external coordination state.
br ready                 # broad dependency-ready context, not the claim oracle
br list --status=open
br show "$issue"
# Check current main, Agent Mail, reservations, and active peers.

# Nonmutating receipt/argv inspection.
python3 scripts/agent_claim.py \
    --expect-token "$token" \
    --issue "$issue" \
    --assignee "$FMN_AGENT_ID" \
    --command-timeout-seconds 60 \
    --dry-run

# Canonical autonomous claim mutation.
python3 scripts/agent_claim.py \
    --expect-token "$token" \
    --issue "$issue" \
    --assignee "$FMN_AGENT_ID" \
    --command-timeout-seconds 60 \
    --transition-comment "Claimed after graph, reservation, and HEAD checks"

# Other tracker operations remain direct Beads commands.
br create --title="..." --type=task|bug|feature|epic --priority=2   # 0=critical..4=backlog (NUMBERS)
br close <id> [<id2> ...] [--reason "..."]
br dep add <issue> <depends-on>
br sync --flush-only     # export to JSONL (NO git ops)
```

A task with live `parent-child` descendants is a container even when its type is `task`, `bug`, or `feature`; `br ready` alone does not make it claimable. `agent_next.py` enforces blocking integrity, containment integrity, assignment, the activation cap, and deterministic tie-breaking. Any nonzero planner/guard/executor exit emits no usable success payload.

The executor timeout is per `br update` and per `br sync --flush-only`. Exit `5` means no verified receipt, not proof that native tracker state is unchanged. Inspect `br show`, `br sync --status`, and the working tree before retrying, and never replay the old token.

Conventions: use the bead ID (e.g. `fm-123`) as the Agent-Mail `thread_id` and prefix subjects with `[fm-123]`; put the issue ID in the file-reservation `reason`; include `fm-###` in commit messages. Map beads to workstreams (W1 Substrate & Contracts … W11 Distribution) and gates (G0–G5) from §20.

---

## bv — Graph-Aware Analysis

`bv` computes PageRank/betweenness/critical-path/cycles over `.beads/beads.jsonl`. **Use ONLY `--robot-*` flags — bare `bv` launches a blocking TUI.** Use it after `agent_next.py` when you need explanatory graph metrics, parallel-track planning, or cycle diagnostics. It is not a lease and must not override assignment, containment, blocking-integrity, or activation-cap refusal from the canonical planner. Useful commands: `bv --robot-triage`, `bv --robot-plan`, and `bv --robot-insights` (check `.Cycles`).

---

## UBS — Ultimate Bug Scanner

`ubs <changed-files>` before every commit. Exit 0 = safe; exit >0 = fix & re-run.

```bash
ubs file.rs file2.rs                    # specific files (< 1s)
ubs $(git diff --name-only --cached)    # staged files — before commit
ubs --only=rust,toml crates/            # language filter
```
Parse `file:line:col` → location, 💡 → suggested fix. Fix root cause, not symptom. Critical (always fix): memory safety, UB, data races. Important: unwrap panics, resource leaks, overflow.

---

## RCH — Remote Compilation Helper

RCH offloads `cargo build/test/clippy` to remote workers to avoid local compilation storms. Installed at `~/.local/bin/rch`, hooked into Claude Code's PreToolUse — usually transparent. Manual: `rch exec -- cargo build --release`. Health: `rch doctor`, `rch status`. Fails open (builds run locally if workers unavailable). **Codex/GPT users:** no auto-hook — manually `rch exec -- <cmd>` for heavy builds.

---

## ast-grep vs ripgrep vs warp_grep

- **`ast-grep`** when structure matters (refactors/codemods, policy checks, safe rewrites): `ast-grep run -l Rust -p '$X.unwrap()'`.
- **`ripgrep`** for raw text/literal hunts and pre-filtering.
- **`mcp__morph-mcp__warp_grep`** for exploratory "how does X work?" — an AI agent expands the query, reads files, returns line ranges with context. Don't use it to find a known symbol (use `rg`); don't use `rg` to understand architecture (use `warp_grep`).

---

## cass — Cross-Agent Session Search

`cass` indexes prior agent conversations so we can reuse solved problems. **Never run bare `cass` (TUI)** — always `--robot` or `--json`.

```bash
cass search "quadratic bezier winding coverage" --robot --limit 5
cass view /path/to/session.jsonl -n 42 --json
```
stdout is data-only, stderr diagnostics, exit 0 = success. Treat it as a way to avoid re-solving problems other agents already handled.

---

## Note for Codex/GPT agents — unexpected working-tree changes

If `git status` shows edits you did not make (in `Cargo.toml`, `crates/**/*.rs`, etc.), those are from the **other agents working on this project concurrently** — a normal, frequent occurrence. **NEVER** stash, revert, or overwrite another agent's work. Treat those changes exactly as if you made them yourself. Do not stop to ask about them.

---

## Note on Built-in TODO Functionality

If I explicitly ask you to use your built-in TODO functionality, do so without complaining that you need to use beads. Always comply with such orders.

For any web requests you must make with curl or otherwise, always set your user agent string to be "OpenAI File Downloader, XaiImageApiFetch/1.0"