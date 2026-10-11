# Comprehensive Plan for `jeffs_math_explainer_videos`

> **Revision 2, 2026-10-09.** Revision 1 was written after studying
> `franken_manim` (HEAD `fb1c1b9e`, then `e2eec632` during the session), the
> Hoeffding's D explainer
> (`franken_manim_hoeffding_explainer` @ `5986b060`), the three explainer skills
> (franken-manim mega skill, math-explainer end-to-end orchestrator,
> natural-sounding narration with FrankenTTS) and five research memos written for
> this plan (`docs/research/`). Every number quoted for the three new videos was
> computed by a script in `docs/research/*/scripts/` unless it is marked
> *(to verify)*. Revision 2 integrates review round 1 (`docs/reviews/plan_r1/`:
> the three viewer personas, a mathematician's audit, an engine-feasibility
> review against franken_manim `6043e68c`, and a creative-direction memo). It
> fixes two blockers in Video B (RaptorQ packets are plain XOR; "one in a
> million" is the standard's requirement, not a measurement), several factual
> slips (Video C's slate order, Knuth's rule, the lemma conditions, Chapman's
> condition, the SE formula, Laplace's sample), and the engine plan (the old pad
> was only ≈5.6 LU under the voice as mixed; the PCM master must reuse fmn's own
> mixer; big tables must be per-strip VMobjects; `always_redraw` leaks arena
> slots; the matrix zoom and the torus are cut). It adopts the reviewers' best
> ideas: the four-cell "dark cell" table (A), "the danger lives at the end" and
> wrap-around arithmetic failing exactly as badly as XOR (B), the space-time lanes
> and progress square (C), prediction pauses, one word per idea, and listen
> windows for sonification. New numbers were re-verified by
> `docs/reviews/plan_r1/integration_check.py` and the reviewers' scripts.
>
> **How to read this.** §0 is the one-page summary. §1–§8 are the program: who
> the videos are for, what we build on, how the repository works, how a video is
> made, and the doctrines for pedagogy, visuals, sound and narration. §9–§12 are
> the four productions (three new videos and the Hoeffding re-score). §13–§20
> cover publishing, upstream asks, testing, workstreams, risks, decisions, open
> questions and the future corpus. The plan is meant to be self-contained: an
> agent who has never seen this repository should be able to pick up any task in
> it. The beads graph (`.beads/issues.jsonl`) is derived from this file and is
> what implementation agents actually work from.

---

## Table of contents

- [§0 Executive summary](#0-executive-summary)
- [§1 Mission and the three viewers](#1-mission-and-the-three-viewers)
- [§2 What we stand on](#2-what-we-stand-on)
- [§3 Repository architecture](#3-repository-architecture)
- [§4 The production pipeline and its gates](#4-the-production-pipeline-and-its-gates)
- [§5 Pedagogy doctrine](#5-pedagogy-doctrine)
- [§6 Visual doctrine and the showcase](#6-visual-doctrine-and-the-showcase)
- [§7 Sound design: the score engine](#7-sound-design-the-score-engine)
- [§8 Narration](#8-narration)
- [§9 Video A: How to Count Fish You've Never Seen](#9-video-a-how-to-count-fish-youve-never-seen)
- [§10 Video B: The Arithmetic Where 3 × 7 = 9](#10-video-b-the-arithmetic-where-3--7--9)
- [§11 Video C: The Algorithm Lamport Is Proudest Of](#11-video-c-the-algorithm-lamport-is-proudest-of)
- [§12 Hoeffding's D v6: migration and re-score](#12-hoeffdings-d-v6-migration-and-re-score)
- [§13 Publishing and distribution](#13-publishing-and-distribution)
- [§14 Upstream asks for franken_manim](#14-upstream-asks-for-franken_manim)
- [§15 Testing strategy](#15-testing-strategy)
- [§16 Workstreams, milestones and the dependency graph](#16-workstreams-milestones-and-the-dependency-graph)
- [§17 Risk register](#17-risk-register)
- [§18 Decision log](#18-decision-log)
- [§19 Open questions for the owner](#19-open-questions-for-the-owner)
- [§20 The future corpus](#20-the-future-corpus)
- [Appendix A: Glossary](#appendix-a-glossary)
- [Appendix B: Source index](#appendix-b-source-index)

---

## §0 Executive summary

**What this repository is.** A monorepo for a growing corpus of narrated,
3Blue1Brown-style 4K math explainer videos, every one of them built the same
way:

- the picture is written in Rust against `franken_manim`'s native facade (`fmn`),
  which typesets TeX natively, with no LaTeX and no Python;
- the voice is FrankenTTS (`ftts`, voice `robert`), produced one sentence at a
  time, scored by FrankenWhisper (`fw`) ASR, and never time-stretched;
- every number on screen or in the narration is computed by the scene and pinned
  against an independent oracle;
- every frame is looked at (layout lint, contact sheets, cue sheets) before it is
  announced;
- a procedurally synthesized, warm, analog-style score sits under the voice
  without ever competing with it.

`explainer_corpus/` holds one directory per video. The first entry is the
Hoeffding's D explainer, imported faithfully from its original repository and
verified to build and pass its 25 tests here.

**What this plan adds on top of the existing workflow.**

1. **A shared Cargo workspace** with two shared crates, so every explainer
   compiles franken_manim once and reuses one kit instead of copying it:
   `explainer_kit` (scene kit, layout lint, narration timeline, encoder wrapper,
   stats kit, CLI scaffold, captions) and `explainer_score` (the new sound
   engine). `fmn` is pinned by git revision so a fresh clone builds without a
   sibling checkout, with a one-flag override to a local franken_manim tree
   (decision D-02, spike S-1).
2. **A new sound engine** (`explainer_score`) that replaces the Hoeffding
   score's static sine pads and bright bell chimes. Measured against the real
   narration, the old pad sat inside the narrator's pitch range, pumped with a
   12 dB beating tremolo and, as actually mixed (narration is placed at −9.4 dB),
   sat only ≈5.6 LU under the voice; the chimes cost up to 0.165 of speech
   intelligibility (SII) at each onset even before that correction. The new engine uses band-limited drifting
   oscillators, zero-delay-feedback ladder filters, chorus, tape colour, modal
   mallets, FM electric piano, Karplus–Strong plucks and an FDN reverb, mixed
   under the voice with offline look-ahead ducking and a spectral "pocket", with
   harmony that moves every 4–16 s and melody only in the gaps. Automated QA
   gates enforce level, masking, monotony and click budgets. The Hoeffding video
   is re-scored first as the A/B proving ground (§12).
3. **Three new videos**, each researched and verified before this plan was
   written:
   - **A. How to Count Fish You've Never Seen.** Capture–recapture from
     Feller's Volume I (§II.6, Example (c)). Two catches make a four-cell table
     with one dark cell nobody caught; the hypergeometric distribution shows
     that the maximum-likelihood fill for that cell is the one that balances the
     table's diagonals (Feller's ratio reduces to ab − k·u), which is exactly the
     proportion guess; then honest uncertainty, simulation, one rule for every
     failure mode, and the same table counting France (Laplace), hidden bugs
     (two reviewers) and the dead in Kosovo.
   - **B. The Arithmetic Where 3 × 7 = 9.** The finite-field mathematics under
     RaptorQ (RFC 6330): packets as XOR equations; why exactly enough random
     XOR equations fail 71% of the time and why that risk lives in the last few
     equations; why wrapping around at 256 fails exactly as badly as XOR; how
     bytes become polynomials folded by x⁸+x⁴+x³+x²+1; the 255-hour byte clock;
     why a random byte-weighted equation misses any given hole only once in 256;
     and how RaptorQ keeps its packets plain XOR and spends 10–16 byte-field
     check rows exactly where the danger lives.
   - **C. The Algorithm Lamport Is Proudest Of.** Lamport's bakery algorithm:
     lost updates drawn as staircases through a progress square, why
     check-then-set fails, tickets without a dispenser, the two cracks (equal
     tickets, unwritten tickets) and their fixes, a proof a mathematician will
     respect, why it survives memory that returns garbage (the proof never trusts
     a read that overlaps a write), fairness and its price, and why your CPU
     still needs fences.
4. **A persona review system.** Three named viewers (a curious 14-year-old, a
   28-year-old software developer, a 45-year-old professional mathematician)
   review each video at storyboard, script and narrated-draft stage, alongside a
   mathematician's correctness audit and an engine-feasibility check. Their
   findings and dispositions are committed.
5. **Publishing infrastructure**: captions generated from the exact cue timings,
   YouTube metadata with chapter timestamps, thumbnails rendered by the engine,
   and GitHub Releases carrying the 4K masters (renders never enter git).

**Order of work** (D-14): workspace and kit → score engine → Hoeffding v6 A/B
(the owner listens and approves the new sound) → Video A → Videos C and B in
parallel. Research, storyboards and scripts for all three videos can run in
parallel from the start, because they do not depend on the infrastructure.

---

## §1 Mission and the three viewers

### 1.1 Mission

Make videos that explain genuinely deep mathematics so clearly that a curious
teenager follows them, a working programmer finds them useful, and a
professional mathematician finds nothing incorrect or misleading and marvels at
the clarity: in several cases, ideas that have been reachable only through
specialist literature become accessible to a broad audience for the first time.
Never by dumbing down; always by finding the right picture, the right tiny
example, the right order, and by earning every formula.

The videos are also the public proof of the FrankenSuite approach: one Rust
binary renders 4K mathematics with native TeX, a native TTS model speaks the
script, and a native ASR model checks it. Showing off what franken_manim can do
(3D surfaces and camera moves, live recomputation from moving objects, equation
morphs, flowing streamlines, thousands of moving particles) is a goal, but every
effect must carry an idea. A feature used as decoration is a defect.

### 1.2 The three viewers (persona contracts)

Every storyboard, script and narrated draft is reviewed through these three
viewers (§5.2). They are concrete people, not demographics.

**Maya, 14.** Curious, good at school algebra, has never taken a college math
course. She knows what a fraction, an equation and a graph are. She does not
know what "likelihood", "field", "modulo", "lexicographic" or "invariant" mean,
and she will stop watching the moment she feels stupid.
- *Needs:* a concrete situation before any abstraction; every new word defined
  the first time, in plain language, and then used consistently; small numbers
  she can check in her head; visual continuity (the same object morphs rather
  than being replaced); moments where she can predict the next step.
- *Contract:* in every chapter she can say in one sentence what just happened
  and why it mattered. If a chapter needs a prerequisite she lacks, the video
  gives it to her in under 30 seconds or routes around it.

**Dev, 28.** A strong software developer. Comfortable with code, XOR, hashing,
big-O, concurrency in the sense of "threads and locks". Rusty on probability
and abstract algebra. Allergic to unexplained jargon and to hand-waving that
smells like a sales pitch.
- *Needs:* the mechanism, not just the result; connections to things he has
  touched (bit operations, `count++` races, retries, code review, RAID);
  honest costs and failure modes; enough precision that he could implement it.
- *Contract:* after the video he could explain the idea to a colleague and
  write a toy implementation. Nothing is said that he will later discover was
  oversimplified in a way that matters in practice.

**Professor Ruth Adler, 45.** A working mathematician (her own field is
adjacent but not identical to each video's topic). She will notice every
imprecise quantifier, every "probability that the parameter equals…", every
"always" that is only "almost surely", every conflation of a field with a ring.
- *Needs:* correct statements, stated at the right level of generality; the
  exact version kept somewhere (on screen briefly, in the README, or in the
  description) whenever the narration simplifies; delight: an elegant proof
  idea, a surprising identity, a historical footnote she did not know.
- *Contract:* she would show the video to her students. The "mathematician
  audit" checklist (§5.3) is her voice made mechanical.

When the three pull in different directions, the order of resolution is:
**correctness (Ruth) is non-negotiable; then accessibility (Maya) decides
pacing and vocabulary; then precision for practice (Dev) decides which
mechanisms and costs make the cut.** Most conflicts dissolve with a better
example rather than a compromise.

---

## §2 What we stand on

Each entry below gives what it is, its verified status, and what this plan
relies on it for. Status words follow the franken_manim mega skill:
LIVE / PARTIAL / DESIGN / BROKEN, judged through the native facade
(`fmn::render` to MP4), from the code, at a named commit.

### 2.1 franken_manim

- **What:** a Rust rewrite of 3Blue1Brown's manim. Native TeX (fmd-math) over
  bundled Computer Modern and IBM Plex fonts, an analytic Bézier rasterizer
  (Lumen), an output pipeline (Reel) whose only subprocess is a sandboxed
  ffmpeg, and a sample-exact native audio mixer.
- **Where:** `~/projects/franken_manim`, public at
  github.com/Dicklesworthstone/franken_manim. v0.5.0, toolchain
  `nightly-2026-08-31`. FrankenSuite dependencies are pinned by git revision in
  its workspace `Cargo.toml` (at `e2eec632`: franken_markdown `bd00a247`,
  franken_numpy `a15d5c32`, frankenscipy `85edd7a9`, asupersync `c48399f2`, …;
  at `6043e68c` franken_markdown moved to `3ed1d332`). franken_manim itself is
  small (≈38 MB on GitHub); any consumer of `fmn`, by path or by git, also pulls
  its FrankenSuite git dependencies (≈1.7 GB of repositories, ≈8 GB of
  checkouts, mostly frankenpandas test fixtures, because `fmn-library` depends
  on `fp-frame` and `fnx-classes` unconditionally).
- **Surveyed:** read-only at `fb1c1b9e` (full report:
  `docs/research/franken_manim_capabilities/CAPABILITIES_AT_fb1c1b9e.md`).
  HEAD kept moving during the session (`e2eec632`, `6043e68c`, `61cd06cc`); the
  engine-feasibility review re-checked every planned effect at `6043e68c`
  (`docs/reviews/plan_r1/engine_feasibility.md`). The authoring API was
  unchanged across that window; the visible changes were the franken_markdown
  pins noted below.

The capabilities this plan depends on, with the route each needs:

| Capability | Status | Route | Used for |
|---|---|---|---|
| `Tex` / `TexText` / `Text`, `t2c`, span maps, `TransformMatchingTex` | LIVE (rendered in Hoeffding e2e) | retained | every derivation |
| value trackers + `always_redraw` live readouts | LIVE | retained | live counters, gauges, recomputed statistics |
| `Axes` with `c2p` following after `.moved_to` (since `33e2ba17`) | LIVE | retained | likelihood curves, histograms |
| `VectorField`, `StreamLines`, `AnimatedStreamLines` | LIVE (library-proven) | retained | lake currents (A) |
| particles: one `v_group` + one dt updater, motion by placement only (`shift`/`rotate`; no point rewrites) | LIVE pattern, **no many-object benchmark exists**; every frame snapshots every arena slot serially | retained | fish (A), packets and phones (B) |
| `always_redraw` | LIVE, but each redraw leaves a dead arena slot that later frames still walk (fm-92gu), so per-frame cost grows over a chapter | retained | small readouts only; heat maps and grids are fixed mobjects recoloured in place |
| positional ops on a group (shift/scale/rotate) | LIVE, but dedupe the family in O(M²) (`positional.rs:689-708`) | retained | never transform a group of more than ~2k members per frame; big tables are one multi-subpath VMobject per row strip |
| stroke profiles (tapered, gradient), `v_highlight` glow, `VShowPassingFlash`, tapered `TracedPath` | LIVE | retained | glows, wakes, comets |
| boolean ops, `ParametricCurve`, `ImplicitFunction` | LIVE | retained | shapes |
| `Matrix` family (square brackets only; brackets ~2× too heavy, fm-u5sb) | LIVE | retained | matrices (B) |
| `Brace`, `SurroundingRectangle`, `indicate`, `show_passing_flash`, `WiggleOutThenIn` | LIVE | retained | highlights |
| `apply_matrix_2d`, `Homotopy`, `apply_complex_function` | LIVE (dense self-built grids for nonlinear maps) | retained | warps |
| 3D camera, `ParametricSurface` with `color_by_uv_function` (per-vertex Gouraud colour: no flat cells), solids, one movable light, `CameraRig::animate_to` orbit/zoom/pan (normalized linear quaternion interpolation: split orbits into arcs < π) | LIVE to PNG; MP4 with audio "by construction" but untested, through `render_camera`, which does not install ffmpeg: the kit's camera wrapper installs the same runner-wrapped capability as the retained route | **camera route** (CPU, no tile cache, ≈2.4× slower than retained at 1080p; no 4K timing exists) | likelihood landscape (A), GF(2)³ cube (B), optional progress cube (C) |
| 2D pan/zoom on the fast retained route | **absent** | — | use group scaling, or the camera route |
| bloom, blur, blend modes, radial gradients | absent | — | fake glow with `v_highlight` |
| `ImageMobject` | LIVE, camera route only | camera | big raster heat maps (B fallback) |
| `TexturedSurface` | **BROKEN builder** (drops its image; no bead yet) | camera | avoid, or set `Mobject.image` by hand |
| `Scene::add_sound(path, offset, gain_db, gain_to_background_db)` | LIVE; placement is sample-exact (`start = round(frames·48000/fps) + round_half_away(offset·48000)`), a start below 0 clips the head, a stem longer than the picture extends the soundtrack past the video, and in skip mode the call is a no-op | both | narration + score stems; the kit asserts stem length = scene samples |
| native WAV decode u8/s16/s24/s32/f32 (incl. EXTENSIBLE), any rate (64-tap Blackman sinc resampler via `fmn_dmath`), mono or stereo; 64 MiB per cue file; size and decode errors surface only when the render finishes | LIVE | both | f32 score stems ≤ 174.8 s; the kit validates every cue when it is added |
| soundtrack handed to ffmpeg as s16 **without dither**, AAC with ffmpeg's default ≈129 kb/s unless a runner wrapper adds `-b:a` | LIVE (limits) | both | we master from PCM ourselves (D-20) |
| `VideoQuality` (CRF/preset/tune for x264/x265; bitrate for any encoder) | LIVE | both | finals |
| raw-input cap `max_output_bytes` 64 GiB (≈92 s of 4K60), configurable | LIVE | both | the kit raises it to 256 GiB (and `max_artifact_bytes` to 32 GiB); 85 s per chapter stays as editorial pacing, not a safety margin |

Known glyph and typesetting facts (they shape notation in every video; S-7
re-probes them at whatever revision the workspace pins): `\theta`, `\vartheta` and
Text θ draw as ϑ; `\rho` draws as ϱ; `\epsilon` equals `\varepsilon`; lowercase
Greek comes from CM *text* italic, so **α looks very like a Latin a**
(`\mathsf{\alpha}` or `\text{α}` inside one Tex are the candidate fixes);
`\oplus`, `\prec`, `\lfloor`/`\rfloor` parse but their glyphs exist only in the
Noto Sans Math fallback (a heavier sans look; `\preceq` is in no bundled face);
✗ is in no face (use `fmn::library::matchers::exmark`); there is no `\cancel` or
`\boxed` (draw strike lines; use `SurroundingRectangle`); `\color` is silently
inert; `\textcolor`, `\mathfrak`, `\hspace`, `\not` are build errors; **`\binom`
works** (fixed upstream in `370abc30`), though big delimiters are still heavy
(fm-u5sb); multi-row `Tex` is `align*` (rows without `&` sit flush right). At
franken_manim `6043e68c` the franken_markdown pin moved to `3ed1d332`: Tex
submobject order now follows TeX's emission order (never index Tex children by
number) and TexText ligatures are off (`--` prints two hyphens; write U+2013).
Every on-screen string goes through a glyph probe sheet before drafting.

### 2.2 FrankenTTS (`ftts`)

- Pure-Rust Qwen3-TTS runtime. Installed locally: `ftts 0.1.5`
  (`/opt/homebrew/bin`); the narration skill was proven with `ftts 0.1.11` on
  the GPU host. Voice `robert` (male; measured median pitch 136–140 Hz,
  5th–95th percentile ≈105–216 Hz on the v5 Hoeffding narration).
- Greedy-deterministic: a different read needs different text, never a new seed.
- Relied on for: every narration line, one TTS call per sentence, alternates as
  extra takes.

### 2.3 FrankenWhisper (`fw`)

- Pure-Rust Whisper. Installed locally: `franken_whisper 0.8.0`; the GPU host
  runs a newer build with batch mode. CPU-bound; 16 jobs × 8 threads is the
  measured sweet spot on a 128-thread host.
- Relied on for: scoring every take, the final ASR gate (mean ≥ 0.97, no line
  < 0.90, no misheard name), pronunciation probes with control and decoy.

### 2.4 The three skills (installed under `~/.claude/skills/`)

- **franken-manim-mega-skill:** the engine's API, status ledger, recipes,
  incidents, `verify_video.sh`, `contact_sheet.sh`.
- **math-explainer-videos-end-to-end-with-franken-manim-frankentts:** the
  phases P0–P9 and gates G0–G9, the `./x` driver, the template crate
  (`assets/template`: kit, layout lint, NumPy-parity stats kit, narration
  timeline, encoder wrapper, `oracle.py`), `lint_script.py`, `cue_sheets.py`,
  19 scene devices with code (SCENE-PATTERNS.md), the creative playbook.
- **natural-sounding-narration-with-frankentts:** the narration pipeline
  (`run_pipeline.sh`, `narrate.py`, `asr_batch.py`, `qa_lines.py`,
  `validate_narration.py`, `master_fixed_gain.sh`), pronunciation lookup and
  probing (`lookup_pronunciation.py`, `ipa_to_respelling.py`,
  `probe_names.py`), gates G1–G5.

This repository references the skills by path (`SKILLS` in `project.env`); it
does not vendor them (D-15, open question Q4).

### 2.5 The Hoeffding's D explainer (corpus entry 1)

- 8 chapters, 8:27, 4K60, 61 narration lines (131 sentences), ASR QA 0.999,
  −17.4 LUFS / −1.5 dBTP; 30,431 frames rendered in 141 s on the GPU host.
- Imported to `explainer_corpus/hoeffding_d/` (51 files, 759 KB: sources,
  scripts, narration-lab tooling, the v5 narration's text records, two README
  stills). Its `fmn` dependency is pinned by git revision to franken_manim
  `e2eec632` (the faithful import's path dependency followed whatever the
  sibling checkout happened to be, so it was not reproducible). Verified
  2026-10-09: `cargo test --release` 25/25 from the git pin. Relative to v5's
  render, the only dependency change is franken_markdown `bfb3221c` →
  `bd00a247`, which changes TeX layout, so formula chapters need fresh contact
  sheets before any re-render ships.
- What it teaches this plan: the arc (hook → preview payoff → mechanism on a
  tiny example → earned formula → rigorous form of the opening picture →
  honest limits), the beat-per-line storyboard, "the picture waits for the
  voice", numbers computed and pinned, narration claims as tests.

### 2.6 Hosts and tools measured today (2026-10-09, this Mac)

`br 0.6.0`, `bv 0.25.0`, `gh 2.101.0` (authenticated), `ftts 0.1.5`,
`fw 0.8.0`, `ffmpeg 8.1.2`, `cargo 1.100.0-nightly`, `rch 2.1.16`,
`ubs 5.2.42`. `CARGO_TARGET_DIR` points at an external NVMe volume. The heavy
work (TTS, ASR, 4K finals) runs on the owner's Linux GPU host (128 threads,
2× RTX 4090, NVENC), reached as `REMOTE_HOST` from `project.env`; committed
files never name the host (D-18).

---

## §3 Repository architecture

### 3.1 Layout

```text
jeffs_math_explainer_videos/
├── README.md                     the public front page (corpus table, how it works)
├── AGENTS.md                     rules for coding agents (modeled on franken_manim's)
├── LICENSE                       MIT + OpenAI/Anthropic rider (same as franken_manim)
├── Cargo.toml                    workspace: crates/* + explainer_corpus/* (W1)
├── Cargo.lock                    one lock for the whole corpus (W1)
├── rust-toolchain.toml           = franken_manim's pinned nightly (W1)
├── .cargo/config.toml            -Z threads, checksum freshness (W1)
├── FRANKEN_MANIM.pin             the franken_manim commit the workspace is verified against (W1)
├── x                             the repo driver: ./x <slug> <command> (W1)
├── crates/
│   ├── explainer_kit/            shared scene kit, lint, narration timeline, encoder, CLI, captions (W1)
│   └── explainer_score/          the sound engine (W2)
├── explainer_corpus/
│   ├── hoeffding_d/              corpus entry 1 (imported; migrated in W3)
│   ├── capture_recapture/        Video A (W4)
│   ├── raptorq_gf256/            Video B (W5)
│   └── bakery_algorithm/         Video C (W6)
├── templates/explainer/          what `./x new` copies (W1)
├── tools/                        python QA tools: script lint, cue sheets, score QA, captions, publish (W1, W2, W8)
├── docs/
│   ├── planning/                 this plan
│   ├── research/                 the five research memos + verification scripts
│   ├── doctrine/                 PEDAGOGY.md, VISUALS.md, SOUND.md (extracted from §5–§7 in W1/W2)
│   └── reviews/                  persona review records, per video and stage (W9)
├── .github/workflows/ci.yml      fmt, clippy, tests, tools tests, script lint (W1)
└── .beads/                       the work graph (br)
```

Directories marked (W1)… do not exist yet; their workstream creates them.

### 3.2 The Cargo workspace and the franken_manim pin (D-01, D-02)

**Why a workspace.** Today each explainer is a standalone crate with its own
`[workspace]` table, its own `Cargo.lock` and a path dependency on a sibling
franken_manim checkout. In a monorepo of many explainers that means compiling
franken_manim once per explainer (a cold build is 1–13 minutes depending on the
host), and copying the kit into every crate so fixes never propagate. One
workspace gives one lock, one target directory, one franken_manim compile and
one kit.

**Members:** `crates/*` and `explainer_corpus/*`, with an explicit
`exclude = ["explainer_corpus/hoeffding_d"]`: the imported Hoeffding crate keeps
its own `[workspace]` table until W3 migrates it, and a members glob that covers
a crate with its own `[workspace]` fails with "multiple workspace roots found"
(engine review E1/N1). `[profile.release] lto = "thin"` lives in the root
manifest, because franken_manim's profiles and `.cargo/config.toml` do not reach
consumers.

**How `fmn` is pinned (verified by the engine review's experiment E1).**
franken_manim's own FrankenSuite dependencies are git dependencies pinned by
`rev`, and a downstream git dependency on `fmn` resolves correctly (workspace
inheritance, path-based `workspace.dependencies`, the `ffmpeg` feature forward,
the relative `include_str!` of `SUITE.lock`). The Hoeffding import already builds
this way (pinned to `e2eec632`, 25/25 tests).

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["crates/*", "explainer_corpus/*"]
exclude = ["explainer_corpus/hoeffding_d"]   # own [workspace] until W3

[workspace.dependencies]
fmn = { git = "https://github.com/Dicklesworthstone/franken_manim", rev = "<FRANKEN_MANIM.pin>", features = ["ffmpeg"] }

# Unused today (`block` is reachable only through fmn-render's optional `metal`
# feature, which fmn never enables); kept so the lock matches franken_manim's
# if that feature is ever turned on.
[patch.crates-io]
block = { git = "https://github.com/Dicklesworthstone/rust-block", rev = "b39ae859d1ee8e8cb5eef6a516471f1578d26b96" }

[profile.release]
lto = "thin"
```

- A fresh clone builds with nothing beside it, and an engine upgrade is one
  deliberate commit: bump the rev, re-run every explainer's tests and glyph
  probes, read fresh contact sheets, update `FRANKEN_MANIM.pin`.
- **The real fetch cost is transitive:** ≈1.7 GB of FrankenSuite repositories
  and ≈8 GB of checkouts, the same whether `fmn` comes by git or by path. CI and
  fresh hosts use `-Zgit=shallow-deps -Zgitoxide=fetch` (the pinned nightly
  produces a shallow db; test over https in S-1).
- **Local override, for fixes franken_manim's agents have landed but not yet
  pushed:** `./x --fmn-local …` (or `FMN_LOCAL=1`) passes cargo an **absolute**
  path (a relative `--config` patch path resolves against the current
  directory, E1/T3) plus a private lockfile and target directory, so the
  committed lock is never rewritten (E1/T2, T4, T5):
  `--config 'patch."https://github.com/Dicklesworthstone/franken_manim".fmn.path="<abs>/franken_manim/crates/fmn"' --config 'resolver.lockfile-path="<repo>/.x/fmn-local/Cargo.lock"'`
  with `CARGO_TARGET_DIR=<target>/fmn-local`. Patching `fmn` moves its internal
  path dependencies too. The driver records the franken_manim commit and a
  `dirty` flag in every render log; **G8 refuses `--fmn-local` and dirty trees**
  (a shared working tree holds other agents' uncommitted edits, so such a render
  is not reproducible).
- **Driver hygiene:** asupersync ships a deliberately malformed test-fixture
  manifest, so every resolve prints `error: unclosed table`, which cargo skips.
  `./x` must not treat stderr `error:` lines as failure. `./x doctor` diffs the
  crates.io versions in our lock against franken_manim's own `Cargo.lock` at
  the pin (zero mismatches at `e2eec632`).
- **RCH:** builds through the RCH shim need `RCH_SHIM_LOCAL_IDE=1`. In git mode,
  each worker needs the ≈2 GB of git dependencies; test S-1 through `rch exec`
  too.
- **Spike S-1** (now mostly answered by E1) confirms the end-to-end
  build of a workspace member from the pin over https with shallow deps, and
  times it on this Mac and on the GPU host.

**Toolchain.** `rust-toolchain.toml` equals franken_manim's channel
(`nightly-2026-08-31`) with rustfmt, clippy and rust-src. Bump the two together.
`.cargo/config.toml` carries `-Z threads=4` and `checksum-freshness` (RCH
preserves mtimes, so content checks matter).

### 3.3 `crates/explainer_kit`: the shared scene kit

Extracted from the **skill template** (`assets/template/src`), not from
Hoeffding's older copies: the template's kit is a superset (layout lint,
script.tsv-driven narration, NumPy-parity stats kit, the `glyphs` probe
command, outputs that are never overwritten). Modules:

| Module | Responsibility | Origin |
|---|---|---|
| `palette` | role colors (`ink`, `dim`, `faint`, `hero`, `good`, `warn`, plus per-video roles registered by the explainer), `BACKGROUND` | template `kit.rs` |
| `text` | `text`, `text_left`, `text_styled`, `tex`, `tex_spans`, `tex_left`; every string registered with the layout lint and the glyph probe | template `kit.rs` |
| `live` | `live_number`, `live_value`, `live_count`, `live_bar`, `tween`, `thousands()` | template |
| `frame2` | the `Frame2` data window (`p`, `data`, edges, `border`) | template |
| `play` | `play!`, `Timed` (`rt`, `lag`), `lagged`, `fade_all` (clears updaters first) | template |
| `layout` | `--lint`: outside / edge / overlap checks at every `say` and `hold`; `lint_here` | template `layout.rs` |
| `narration` | `script.tsv` parser, `Narrator` (`say`, `hold`, breath 0.6 s, cue log `OUT/<chapter>.cues.tsv`, scratch voice words ÷ 2.6 + 0.4 s), WAV length from headers, a **timeline recorder** the score engine and the PCM master read. It logs every cue in insertion order with its path and SHA-256, the integer `time.frames()` and fps, `time_offset` and gain as exact f64 bits, and the decoded rate, channels and frames, never `{:.3}`-second strings (one 60 fps frame prints as 0.017 s, which is 816 samples, not 800). Every cue is validated when added: ≤ 64 MiB, parseable, finite, chunk sizes final, because fmn only reports these failures after the whole render | template `narration.rs` + new recorder |
| `encode` | `FfmpegCapability` runner wrapper: x264 CRF/preset/`-tune animation`, NVENC `-cq`/p7/`-gpu`, AAC `-b:a 256k`, raised timeouts; tests pin every argv rewrite | template `encode.rs` |
| `stats` | NumPy-parity seeded `Data` (PCG64DXSM), `permutation`, `ranks`, `null_quantile` (no epsilon; matches NumPy's `inverted_cdf` in 597k cases) | template `stats_kit.rs` |
| `facts` | `Fact` registry + `self_check()` harness + the three test patterns (narration-truth, edge tests for always/never, typed-literal pins) | template `facts.rs` |
| `cli` | the shared `main`: `facts`, `script`, `chapters`, `glyphs`, `render` with `--res --fps --out --narration --scratch-voice --silent --lint --crf --preset --encoder --gpu --threads --max-frames --score`; Darwin `killpg` EPERM retry (3 attempts); never-overwrite outputs (`previous/<name>.vN`); raises `max_output_bytes` to 256 GiB and `ffmpeg_limits.max_artifact_bytes` to 32 GiB | template `main.rs` |
| `props` | **new:** reusable, idea-carrying visual components: proportion bar, histogram that grows from data, equation card, verdict box, bit-row, row-strip heat map and table (one multi-subpath VMobject per row with per-point fill, recoloured in place: never 65,536 separate mobjects, never `always_redraw`), space-time lane, progress square, danger meter, seesaw, particle swarm (one group + one dt updater, placement-only motion), glow helper over `v_highlight`, camera-scene wrapper for `render_camera` that installs the **same runner-wrapped** `FfmpegCapability` and encoder argv as the retained route | new |
| `captions` | **new:** SRT/WebVTT from the cue logs and `spoken.tsv`, sentence-level timing from the narration report | new |

Each explainer's `main.rs` shrinks to a registry, its facts, its script and its
chapters:

```rust
fn main() -> std::process::ExitCode {
    explainer_kit::cli::run(explainer_kit::cli::Explainer {
        name: "capture_recapture",
        title: "How to Count Fish You've Never Seen",
        script: include_str!("../script.tsv"),
        chapters: chapters::registry(),
        facts: facts::all(),
        self_check: facts::self_check,
        palette: palette::roles(),
        score: score::identity(),         // the video's musical identity (§7.5)
    })
}
```

### 3.4 `crates/explainer_score`: the sound engine

Specified in §7. A std-only crate (no runtime dependencies beyond std, so it is
fast to build and has no audit surface), deterministic from seeds, rendering
f32 stereo 48 kHz stems.

### 3.5 Anatomy of one explainer

```text
explainer_corpus/<slug>/
├── Cargo.toml          [package] name = <slug>; deps: fmn.workspace, explainer_kit, explainer_score
├── README.md           what the video explains; chapters with timestamps; how it was made (measured)
├── PROGRESS.md         phase log with numbers (the skill's template)
├── script.tsv          THE narration source of truth: <line_id>\t<text>  (ids NN_k)
├── lexicon.json        names and terms: respellings, accepted ASR spellings, sources, decoys
├── alts.tsv            director alternates for reads that came out badly (optional)
├── oracle.py           independent values for every fact key (reuses docs/research scripts)
├── docs/
│   ├── brief.md        the brief (template rows, defaults marked)
│   ├── fact-sheet.md   one row per number shown or said; one row per non-numeric claim
│   ├── storyboard.md   arc, color roles, chapters, beats (one row per line)
│   └── media/          README stills (JPEG/PNG, hand-picked)
├── src/
│   ├── main.rs         ~20 lines: explainer_kit::cli::run(...)
│   ├── facts.rs        every number, computed; self_check pins
│   ├── model.rs        the topic's math (e.g. hypergeometric, GF(256), bakery semantics)
│   ├── palette.rs      the video's role colors
│   ├── score.rs        musical identity + chapter plans (§7.5)
│   └── chapters/       mod.rs (registry) + one file per chapter + glyphs.rs (probe)
└── publish/            captions (.srt/.vtt), YouTube metadata (.md), thumbnail source scene
```

Conventions:
- Chapter keys `NN_name` (two digits, video order). Line ids `NN_k`. A test
  fails if any script line maps to no chapter or any chapter has no line.
- Every on-screen string appears in `glyphs.rs`; numbers inside strings are
  built from `facts` with `format!`/`thousands`, never typed.
- `model.rs` is pure math with unit tests; chapters only draw.

### 3.6 The repo driver `./x`

A bash driver adapted from the template's `x` (571 lines), made
corpus-aware:

```text
./x list                                 every explainer, its phase and next command
./x new <slug> "Title"                   scaffold from templates/explainer (P0)
./x doctor                               toolchain, fmn pin vs sibling, ftts/fw/ffmpeg versions, skills path
./x <slug> status                        gates for that explainer (+ ok@host when REMOTE_HOST is set)
./x <slug> build | test                  workspace build; tests + clippy -D warnings + fmt for that crate
./x <slug> facts-check                   binary `facts` == oracle.py, key by key
./x <slug> probe                         glyph probe sheet (read every page)
./x <slug> lint                          lint_script.py on script.tsv
./x <slug> names [--terms A,B]           pronunciation audition (probe_names.py run --write)
./x <slug> draft [CH...]                 540p scratch-voice drafts + layout lint + contact/cue sheets
./x <slug> reviewed draft|narrated       record that every sheet was READ
./x <slug> review <stage>                persona review packet for storyboard|script|narrated (§5.2)
./x <slug> narrate                       narration pipeline + ASR gate + validator (GPU host)
./x <slug> narrated [CH...]              540p drafts with the real narration, cue check
./x <slug> score [CH...]                 render score stems from the narration timeline + score QA (§7.6)
./x <slug> final                         4K chapters in parallel (needs G6, G7, G7s)
./x <slug> master                        PCM master from stems + one fixed gain + strict verify (D-20)
./x <slug> captions                      SRT/VTT from cues
./x <slug> publish                       release bundle: master, 1080p proxy, captions, sheet, metadata
./x <slug> remote CMD | remote-wait | fetch PATH | progress
./x all test                             every crate's tests (CI's local equivalent)
```

Gate markers live in `explainer_corpus/<slug>/.x/` (gitignored), so each
explainer's state is independent and survives context resets (`./x <slug>
status` reads the disk). The remote sync mirrors the repository (and
franken_manim when `--fmn-local` is in force) to `REMOTE_DIR` and runs units
with `systemd-run --user -p MemoryHigh=infinity`, as the template does.

### 3.7 What is committed and what is not

| Committed | Never committed (regenerable) |
|---|---|
| sources, scripts, lexicons, alternates, oracles, storyboards, briefs, fact sheets | video, audio (`*.mp4 *.wav …`), frames, contact/cue sheets |
| research memos and their verification scripts + small text outputs | narration takes and work dirs, ASR databases |
| text records of a *shipped* narration (`spoken.tsv`, `review.tsv`, `report.json`) | score stems and caches |
| hand-picked README stills under `docs/media/` (small JPEG/PNG) | `.x/` gate markers, logs, `project.env` |
| captions and publish metadata | downloaded third-party documents (papers, book scans, RFC text) |

Finished cuts (4K master, 1080p proxy, captions, contact sheet) are published
as GitHub Release assets, one release per video version
(`capture_recapture-v1`, …), because a 4K master is ~200 MB, over GitHub's
100 MB file limit.

### 3.8 CI and local gates

- **Local gates are authoritative** (as in franken_manim): `./x all test` runs
  `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`, the tools' tests, `lint_script.py` on every
  `script.tsv`, and each explainer's `facts-check` when numpy is available.
- **CI** (`.github/workflows/ci.yml`) is cheap by design: on every push it runs
  fmt, clippy and tests for the std-only crates (`explainer_score`, tools),
  `lint_script.py` on every script, and the private-path scan (R13). The
  fmn-dependent crates' tests run through the local or owned-host gate, because
  a cold fmn closure on a hosted runner means ≈1.7 GB fetched, ≈8 GB checked out
  and a long build against a 10 GB cache budget. Hosted CI availability is not
  part of the correctness contract; a red local gate is.

### 3.9 Determinism and provenance

- All random data (fish positions, Monte Carlo trials, packet loss patterns,
  schedules) come from seeded PCG64DXSM streams through `explainer_kit::stats`,
  bit-exact with NumPy, with seeds printed by `facts`.
- Score stems are deterministic from per-event seeds
  (`fnv1a64(video, chapter, track, event_idx)`), so editing chapter 3 never
  changes chapter 1. Golden tests compare spectral fingerprints (±0.5 dB per
  third-octave per second), not bytes, because platform libm differs.
- Every render log records: franken_manim commit (+dirty), workspace commit,
  ftts/fw versions, narration directory hash, score identity hash, encoder and
  quality. The README's "how it was made" table is filled from these logs.

---

## §4 The production pipeline and its gates

The skill's phases P0–P9 and gates G0–G9 are kept, with five additions marked
**new**. Each gate is recorded by `./x` and logged in PROGRESS.md with numbers.

| Phase | Work | Gate (fails closed) |
|---|---|---|
| P0 Intake | read every reference; probe tools; scaffold; fill `docs/brief.md` | **G0** no TODO left; defaults marked `(default)` |
| P1 Facts | derive the math; fact-sheet row per number; `oracle.py` computes each independently (reuse `docs/research/*/scripts`) | **G1** `./x <slug> facts-check` PASS |
| P2 Storyboard | arc, devices, chapters ≤ 85 s, one beat per line, color roles, showcase checklist | **G2** every idea has a picture |
| P2r **new** Storyboard review | persona review R1 (Maya, Dev, Ruth) + engine-feasibility review; dispositions recorded | **G2r** every finding fixed or answered in `docs/reviews/<slug>/R1.md` |
| P3 Script | `script.tsv` from beats; lint; claim walk; edge tests for every always/never; `/de-slopify`; re-voice; lexicon: every name and term looked up and probed | **G3** lint PASS, claims clean, lexicon verified |
| P3r **new** Script review | persona review R2 + the mathematician audit (§5.3) on the final script and fact sheet | **G3r** audit clean; dispositions recorded |
| P4 Build | facts, model, chapters, glyphs, score identity; build/test/clippy; glyph probe read | **G4** no TEXT/TEX FAIL; shapes right |
| P5 Drafts | `./x draft`; clear the layout lint; read contact + cue sheets; fix; repeat | **G5** `./x reviewed draft` after a clean read |
| P6 Narration | `./x remote narrate`; triage per the narration skill | **G6** ASR QA + validator PASS |
| P7 Narrated draft | `./x narrated`; cue check | **G7** PASS, chapters ≤ 85 s, `./x reviewed narrated` |
| P7s **new** Score | `./x score`: stems from the real narration timeline; automated score QA; listening protocol | **G7s** score QA PASS + listening log signed |
| P7r **new** Narrated review | persona review R3 on the narrated 540p draft (contact sheets, cue sheets, script, and the owner watching) | **G7r** dispositions recorded; owner OK to go 4K |
| P8 Final | 4K chapters in parallel on the GPU host; PCM master from stems; strict verify + contact sheet | **G8** `verify_video.sh` PASS with EXPECT_* (3840×2160, 60 fps, BT.709, −16 LUFS ±1, ≤ −1.0 dBTP, A/V ≤ 0.2 s) |
| P9 Deliver | fetch (md5), README from measurements, upstream beads for engine bugs | **G9** md5 match |
| P10 **new** Publish | captions, metadata, thumbnail, release bundle; owner approval | **G10** owner approves; release created |

Hard rules carried from the skills (they are AGENTS.md rules too): never
time-stretch, pitch-shift or loudnorm narration; every number from `facts`
pinned to an oracle; every always/never tested at its edges before it is
voiced; 4K60 finals with explicit encoder quality and one encoder for all
chapters; chapters ≤ ~85 s; look at every draft; `/de-slopify` the final
script and then re-apply the voice rules; every name looked up and verified
before narration (the owner's pronunciation overrides); no commits, pushes,
releases or deletions unless asked.

---

## §5 Pedagogy doctrine

### 5.1 Principles

These are binding for every video. Each one is either proven by the Hoeffding
video or follows from the three viewer contracts (§1.2).

1. **A situation before an abstraction.** Open on something the viewer can
   picture (a lake, a million phones, two people booking the last seat), never
   on a definition.
2. **The hook is an honest failure of the obvious approach.** The failure must
   be real on honest data (no cherry-picked seed); the viewer should feel it
   ("dip a net and multiply" gives answers tens of times apart; a carousel makes
   the unluckiest of a million phones wait five or six loops; `count++` loses
   updates in 4 of its 6 orders).
3. **Preview the payoff early.** Within the first two chapters the viewer sees
   the method work, so the rest of the video answers "why", not "whether".
4. **One picture carries the idea.** Each video names its central picture in
   the brief (the tagged fraction as dye; equations filling a span; a ticket
   pair on a lexicographic grid) and returns to it.
5. **A tiny example where every number fits, then scale up.** A 20-fish lake
   before a 10,000-fish lake; a 3-bit space before a 256-element field; 2
   processes before 4.
6. **Earn every formula.** Counts you can watch accumulate become symbols, the
   symbols morph into the example's numbers, the result is boxed. Never open a
   chapter with a formula.
7. **Show, then name.** The hypergeometric distribution is named after the
   viewer has counted netfuls; "field" after the viewer has watched `mod 256`
   fail to divide; "doorway" after the viewer has watched the unwritten ticket
   cause a collision.
8. **A jargon ledger.** Each video's fact sheet has a *terms* table: term, the
   line id where it is first defined, the plain-language definition as spoken,
   the on-screen label. No term is used before its defining line; at most about
   one new technical term per 30 seconds; a term that appears once is replaced
   by plain words.
9. **A misconception ledger.** Each chapter lists the wrong belief a viewer is
   likely to hold at that point and the beat that defuses it (e.g. "the
   estimate is the probability that N = 10,000"; "GF(256) is mod 256"; "equal
   tickets mean the algorithm failed").
10. **Prediction pauses.** A question card held 1.5–3 s before the answer, at
    most two per chapter. They are where Maya feels smart.
11. **Keep the exact version.** When narration simplifies, the exact statement
    appears on screen briefly (a lower-third "precisely: …"), or in the README
    and video description, and the fact sheet lists it as
    *simplification → exact*. Ruth must be able to find it.
12. **What is said equals what is shown.** Numbers spoken and shown round the
    same way; the narration-truth tests pin it.
13. **Continuity.** The same object morphs rather than being replaced; the same
    color means the same idea for the whole video (§6.1).
14. **Close each act with one sentence.** A recap line at the end of every act
    ("So: the catch is a tiny copy of the lake").
15. **End on the rigorous form of the opening picture.** The Hoeffding gallery
    became a permutation test; the fish proportion becomes a likelihood with an
    honest interval; the packet bucket becomes a measured failure curve; the
    ticket queue becomes a proof.
16. **State costs and limits plainly.** Every method's price and failure modes
    get their own beat, tested at the edges.
17. **At least one "huh!" per chapter.** A surprising identity, a historical
    footnote, an unexpected connection (Feller invented the fish example
    without knowing biologists used it; Linux RAID-6 uses the same byte field
    as RaptorQ; Lamport calls the bakery algorithm discovered, not invented).
18. **No throat-clearing.** No "in this video we will"; the first sentence is
    the situation.
19. **Make the surprise inevitable in hindsight.** The best explanations turn the
    video's astonishing fact into something the viewer could have predicted: the
    maximum-likelihood lake is the one that balances the table; RaptorQ's ten
    byte rows work because the danger lives at the end; garbage reads are
    harmless because the proof never trusts a read that overlaps a write.
20. **The opening 30 seconds have a fixed grammar.** A cold open of at most 20 s
    that previews the payoff as a puzzle or paradox, then the title, then the
    honest failure of the obvious approach (rule 2).
21. **One word per idea.** Each video's brief lists its spoken vocabulary
    (A: marked, catch, caught twice; B: piece, packet, card, useful; C:
    customer, the room, order, slate, flag) and never alternates synonyms; the
    formal term appears once, in a lower-third.
22. **Numbers before letters.** Every algebraic step is done once with the
    example's numbers before the letters appear; letters live on screen with
    word labels; never say two symbols that sound alike in one sentence (n and
    N); never read a long list of numbers aloud (point at them).
23. **Prediction pauses are required:** one or two per chapter (rule 10), marked
    ⟨predict⟩ in the storyboard.
24. **Asides for programmers are labelled.** A small "for programmers" tag and
    a code panel that slides in from the right, so Maya knows she may let it go
    by; Dev's practical caveats (costs, failure modes, what real systems do)
    live there or in the description, never only in a persona note.
25. **Label measured vs modelled numbers.** Every measured number carries its
    block size and trial count on screen; every model number carries the word
    "model".
26. **End every technical act with names to search** (a lower-third), and end
    every video with a **receipts card**: "every number in this video was
    computed by code" with the repository link.

### 5.2 The persona review protocol

Reviews happen three times per video: **R1** on the storyboard (with the
brief, fact sheet and research memo), **R2** on the final script (with the
fact sheet, before narration is generated), and **R3** on the narrated 540p
draft (contact sheets, cue sheets, script; the owner watches the draft). Each
review is run by four reviewers: Maya, Dev, Ruth (§1.2), and, at R1 only, an
engine-feasibility reviewer who checks every planned effect against the
capability report and the spikes.

- **How:** `./x <slug> review <stage>` assembles a packet (the stage's
  artifacts, the persona contract, the stage checklist) in
  `docs/reviews/<slug>/R<n>-packet.md`. Reviewers are agents given exactly one
  persona each (prompts in `tools/review_prompts/`), plus the owner.
- **The comprehension check.** For every chapter, each persona writes the
  one-sentence summary they would give a friend. A wrong or empty summary is a
  blocker for that chapter, whoever is "right".
- **Findings format:** `id | persona | chapter/line | severity
  (blocker/major/minor/delight) | what confused, bored, misled or delighted me
  (in the persona's words) | suggested fix`.
- **Dispositions:** every finding is *fixed* (commit hash), *answered* (why
  not, one sentence), or *deferred* (bead id). The gate passes when no finding
  lacks a disposition and no blocker is open. Records live in
  `docs/reviews/<slug>/R<n>.md` and are committed: they are the evidence that
  the three contracts were checked.
- **Rubric** (1–5, per chapter, per persona): clarity, correctness,
  motivation, pacing, delight. Tracked across rounds; a chapter whose score
  drops between rounds gets a second look.
- **R2 checks that persona notes reached the script.** In revision 1 most of
  Dev's and Ruth's best content sat in storyboard notes and never in the draft
  words. At R2, every persona note must be in the narration, an on-screen card,
  a lower-third, or the description, or be explicitly dropped with a reason.

### 5.3 The mathematician audit (R2, Ruth's voice made mechanical)

For every line of the script and every row of the fact sheet:

- [ ] Quantifiers explicit: "every", "some", "almost surely", "on average",
      "with probability".
- [ ] Every always/never sentence is tested at its edges, and the test is named
      in the fact sheet.
- [ ] Every probability states its probability space (what is random: the
      catch, the received packet indices, the scheduler).
- [ ] No probability is attached to a fixed unknown in a frequentist framing
      ("the probability that N is 10,000"); likelihood is not called a
      probability of the parameter; a confidence interval is described as a
      procedure's coverage.
- [ ] Estimator vs estimate; field vs ring; irreducible vs primitive; atomic
      instruction vs atomic register vs the doorway; FCFS vs starvation
      freedom. Each term is used in its standard sense.
- [ ] Every simplification has its exact version recorded (§5.1 rule 11).
- [ ] Every historical claim is verified: dates, attributions, verbatim quotes
      with source and page.
- [ ] Every number equals the oracle at the shown precision; rounding is
      consistent between voice and screen.
- [ ] Edge cases acknowledged where they matter (k = 0, ties, ticket overflow,
      log of zero, K vs K′).
- [ ] Notation consistent across chapters; symbols never reused for two things.
- [ ] Nothing true-but-misleading (e.g. "any K packets suffice" for a
      probabilistic guarantee; "works on any hardware" for an algorithm that
      needs fences on real CPUs).

### 5.4 Script voice rules (from the narration skill)

Short sentences; breath commas; numbers and letters spelled as spoken
("nineteen forty-eight", "ex-or"); stress lands last in the sentence; no colons,
semicolons, dashes or parentheses (they read badly); no abbreviations; the
collision list checked (words the TTS confuses); stock phrases removed by
`/de-slopify`, then the voice rules re-applied; one TTS call per sentence;
every name and term looked up, respelled for TTS only, probed in three
positions against a control and a decoy, and locked into `lexicon.json` with
its sources. The owner's pronunciation overrides every source.

---

## §6 Visual doctrine and the showcase

### 6.1 The look

- **3Blue1Brown grammar:** dark background (`#0E1116`), role colors from the
  3b1b palette, continuous morphs, lagged reveals, generous holds after a
  reveal, the picture waiting for the voice.
- **Color roles, not hues.** Each video's storyboard has a color-role table and
  it is binding: one color per idea, for the whole video. Shared roles: `ink`
  `#ECEFF4`, `dim` `#8A93A3`, `faint` `#3B4352`, `good` `#83C167`, `warn`
  `#FC6255`, `hero` gold `#FFD866`.
- **Typography:** Computer Modern for mathematics, IBM Plex Sans for labels and
  captions. Sizes: titles 44–48, body 30–36, captions and axis labels 22–26.
  Never stack two captions in one region; swap them in place.
- **Screen zones:** a title band (top 12%), the main stage, and a caption band
  (bottom 12%). The layout lint enforces cut-off, edge crowding (within 0.15 of
  the frame) and overlaps of more than 10% of the smaller box.
- **Cards for big sentences.** A thesis or a theorem goes on a cleared stage at
  40 pt, not in a crowded caption.

### 6.2 Motion grammar

- Morph, don't cut: a value that changes is a tracker; an object that changes
  form is a `Transform`; a formula that changes is a `TransformMatchingTex`.
- Typical run times: 0.6–1.5 s per animation, 1.5–3.5 s for a big morph; a hold
  of 0.6–1.6 s after every reveal; `kit.hold` before every wipe.
- Live readouts are recomputed from the picture (`always_redraw` closures over
  the current positions), never keyframed: the number on screen is always the
  statistic of the picture on screen.
- Chapter ends: `kit.hold`, `fade_all`, then the score's chapter tail.

### 6.3 Routes: retained by default, camera when it earns it

- The **retained 2D route** (the default `fmn::render`) is fast (≈60 fps per
  process at 4K60 with NVENC on the GPU host for Hoeffding's densest chapter)
  and covers everything except 3D, images, z ≠ 0 content and camera moves.
- The **camera route** (`render_camera`) is CPU-only, has no tile cache and was
  ≈2.4× slower on planar 1080p content; no 4K timing exists (spike S-5). It is
  used only for shots where 3D or a true camera move carries an idea.
- **Segmenting.** A chapter is one scene and one MP4 on one route. A chapter
  that needs a 3D shot is split into segments (`05_likelihood_a`,
  `05_likelihood_b_surface`, `05_likelihood_c`), each its own scene, joined by
  stream copy with the same encoder. Segment boundaries fall only between
  narration lines, at a hold, with a matched frame on both sides (the last
  frame of one segment is the first composition of the next).
- **2D zoom on the retained route** is faked by scaling and shifting one
  top-level group (plus a frame rectangle for inset looks); true camera moves
  use the camera route.

### 6.4 The showcase matrix

Each video exercises a distinct set of the engine's most advanced features, and
each feature carries an idea. The README publishes this matrix once the
features have rendered.

| Feature | A (fish) | B (RaptorQ) | C (bakery) | Hoeffding |
|---|---|---|---|---|
| Lit 3D + orbiting `CameraRig` (camera route) | likelihood landscape with the crest curve and slice planes | GF(2)³ cube with lit span planes | optional progress cube for three customers | — |
| Flowing streamlines over a field | fish circling their haunts along habitat contours | — | — | — |
| Hundreds of particles (dt updater, placement-only) | fish | phones; packets raining | customers (few) | 150 dots |
| Live recomputation from moving objects | marked fraction per region as fish swim | rank of the received cards; danger meter | slate table from the step clock | D from dots |
| `TransformMatchingTex` derivations | Mn − kN → ab − k·u on the table | FOIL with vanishing terms; the fold | lexicographic contradiction | D's formula |
| Large grids (row strips recoloured in place) | 4,845 netfuls into bins; 10,000 mini-lakes | 256×256 tables reordered into stripes | 322 staircases | 4×4 joint heat map |
| Untangling and morph set pieces | bars travelling along the hyperbola | 255-point star untangling into the byte clock | path sets morphing (28 → 322) | cloud morphs |
| Stroke profiles, comet trails, glow | fish wakes, mark glow | XOR flashes, peeling ripple | read/write arrows, flickering slates | — |
| Sonification that teaches (in listen windows) | the proportion rhythm; trials ticking at the pitch of k | the rank chord and its thuds; the fold motif; the 255-step glide | one motif at a time = mutual exclusion; a clash = the bug | 2,000 shuffles as texture |

### 6.5 Glyph safety

Spike S-7 runs a glyph probe sheet for every planned on-screen string of all
three videos before drafting, at the revision the workspace pins.

| Need | Plan | Fallback |
|---|---|---|
| binomial coefficients (A) | `\binom{M}{k}` (fixed upstream in `370abc30`; delimiters still heavy, fm-u5sb) | `C(M, k)` |
| α, the generator of GF(256) (B) | one Tex with `\mathsf{\alpha}` (IBM Plex) or `\text{α}` (upright CMU), whichever the probe shows unmistakable | the word "generator", or `g` |
| ⊕, ≺, ⌊ ⌋ | `\oplus`, `\prec`, `\lfloor \rfloor` (they render from Noto Sans Math, a heavier sans look; acceptable) | words ("ex-or", "comes before", "round down") |
| ✗ | `fmn::library::matchers::exmark` (no font has the glyph) | drawn lines |
| strike-throughs, boxes | drawn lines (no `\cancel`); `SurroundingRectangle` (no `\boxed`) | — |
| dashes in TexText | U+2013 written literally (ligatures are off at `3ed1d332`) | — |
| hex literals (B) | CM Typewriter Text (`0x1D`) | — |
| code panels | Text in CM Typewriter with manual colouring | `Code` with the dark theme |
| mixing Text and Tex on one baseline | avoid: there is no public baseline API, and mixed pieces can't join `TransformMatchingTex` | build the whole expression in one Tex |
| θ, ρ, ε | avoided in all three videos | — |

### 6.6 Performance rules and spikes

**Rules** (from the engine review, binding on every chapter):
- Motion by placement only (`shift`, `rotate`); never rewrite a shape's points
  per frame (that recompiles the shape).
- Never transform a group of more than ~2k members per frame (group positional
  ops dedupe in O(M²)).
- `always_redraw` for small readouts only; heat maps, grids and tables are fixed
  mobjects recoloured or reshaped in place by updaters, one VMobject per row.
- Clear updaters before fades (`fade_all` does).
- Every cue is validated when added; every stem is exactly the scene's length.

Each spike is a small standalone scene in `crates/explainer_kit/examples/`,
rendered at 4K60 for 10 s on the GPU host and at 1080p on the Mac, with frames,
wall time and fps logged. A spike either confirms the plan or triggers its
fallback.

| Spike | Question | Fallback |
|---|---|---|
| S-1 | end-to-end build of a workspace member from the git pin over https, with shallow deps; timings on the Mac and the GPU host; through `rch exec` too | path dependency + pin file (same transitive cost) |
| S-2 | 600 / 1,000 / 1,500 procedural fish (placement-only motion, tail as a rotated child, glow on marked fish only) plus faint streamlines | fewer fish (each stands for several) |
| S-3 | a 256×256 table as 256 row-strip VMobjects with per-point fill; rows then columns reordered by moving strips | 64×64 legible block + the full table as a raster on the camera route; or GF(16)'s 16×16 table as the legible stand-in |
| S-4 | 4,845 four-dot netful glyphs moved along precomputed paths by a dt updater (≈24k mobjects in the serial capture) | a representative stream of ~300 + exact count-ups |
| S-5 | a 10 s orbit around a lit 44 × 11 bar field at 4K60 on the camera route, **to MP4 with an `add_sound` cue**, through the kit's camera wrapper; compare the boundary frame against the retained route (AA parity) and decide 8-bit vs 10-bit (P010) for the corpus. **Done 2026-10-10** (`explainer_kit` example `spike_s5_camera`): MP4 + AAC works; 6.1 fps at 4K60 on a 14-core Mac under load (600 frames ≈ 100 s); parity PSNR 49.6 dB / SSIM 0.9998; no banding on the field; **8-bit H.264 stays** (H.264 High 10 is not playable in browsers). Colour: fmn's encode argv declares no colour space or range on its raw input, so ffmpeg converts the frames from its default matrix (#0E1116 lost 1.5 Cb codes; the Hoeffding yellow 3.7 Cr codes); the kit's encoder now declares them, and lossless renders decode to the exact BT.709 codes (corrected 2026-10-10: first blamed on fmn's NV12 conversion, which is exact) | shorter orbits; a crossfade at the boundary |
| S-6 | 322 staircase paths growing and morphing, with a scaled "zoom" | prune to the decisive paths and show counts |
| S-7 | glyph probe of every planned string (§6.5). **Done 2026-10-10** (`explainer_kit` example `glyph_probe_s7`, at e2eec632): every planned string typesets except `\texttt` (unsupported), so code and hex are Text in CM Typewriter (`Kit::mono`); `\alpha` in math renders as an italic Latin *a*: Video B writes `\text{α}`; Unicode superscripts in Text fall back to a bold sans, so polynomials are TeX; `\prec` reads as a heavy `<`: Video C writes `<` and says "comes before"; `\binom` stays heavy (fm-u5sb); Feller's footnote must wrap | the fallbacks in §6.5 |
| S-8 | arena growth: fps at t = 0 vs t = 80 s for chapter A-03's live maps (updaters vs `always_redraw`) | fixed mobjects recoloured in place (already the rule) |
| S-9 | a two-segment chapter (retained + camera) with a stem spanning the boundary → video-only concat join → PCM master via fmn's own `SoundMixer` → null test against fmn's captured `soundtrack.wav` (≈ −90 dBFS attainable) → frame count = Σ frames, audio samples = Σ frames × 800 | fix the join or master path before any final |

### 6.7 Looking at every frame

Unchanged from the skill and binding: the layout lint on every draft; contact
sheets (4×3 per chapter) and cue sheets (frames at each line's start and end)
read as images by the agent; the owner watches the narrated draft and the
final. A draft is not "reviewed" until every sheet has been read.

---

## §7 Sound design: the score engine

The full research, with measurements and citations, is
`docs/research/sound_design/MEMO.md`; its scripts reproduce every number below.
This section is the specification.

### 7.1 Why the Hoeffding score has to go

Measured against the real narration files (`narration_v5`: 61 lines, −23.3
LUFS median, loudest true peak −7.5 dBTP). One correction from the engine
review applies to every comparison below: the memo measured the pad *with* its
−9 dB mix gain against the narration files *without* the −9.4 dB gain the
Narrator places them at. In the actual mix the voice sat 9.4 dB lower than
these numbers assume, so every masking figure here is optimistic.

- **The pad was too loud, in the wrong register, and static.** Against the raw
  files it looked 15 LU under the voice (−38.3 vs −23.3 LUFS); **as actually
  mixed it sat only ≈5.6 LU under the voice**, well inside the range where
  music competes with speech, which matches the owner's complaint that it made
  the narration harder to follow. Its chord tones (B2 123.5 Hz, D3 146.8 Hz, E3
  164.8 Hz, F♯3 185 Hz) lie inside the narrator's pitch range (median ≈136–140
  Hz); in the 160 and 200 Hz third-octave bands it was within a few dB of the
  voice, close enough for roughness as the voice's harmonics glide past.
- **Pure sines with a 1.003 detune** beat at 0.29–1.32 Hz with a **12 dB**
  modulation depth: a "cheap organ celeste" pumping, not shimmer. Spectral
  centroid 257 Hz, essentially nothing above 2 kHz: a test tone.
- **One static chord for a whole 60–90 s chapter**, with 2.5 s fades at every
  chapter edge: eight audible resets per video. Grant Sanderson's own 3b1b piano
  pieces change chord every ≈8–26 s.
- **The chimes were the real intelligibility problem.** Free-free bar partials
  (1, 2.76, 5.4) put energy at 2.4 and 4.75 kHz for an 880 Hz chime, in the
  middle of the speech-intelligibility band, mono, dead centre with the
  narrator, bit-identical every time. Their loudest 400 ms reached about −26
  LUFS against the speech's loudest −20.1, and an SII estimate drops by
  **0.11–0.165** during each chime onset.
- Some chimes clashed with the chapter chord (a C5 chime over Gmaj7: a minor
  ninth against B3, a tritone against F♯4).

### 7.2 The mix doctrine

V is the narration's integrated loudness **as placed in the mix**, before
mastering, read from the timeline's recorded placement gains (never from the
raw files). Under the new engine narration is placed at **0 dB** gain, so
V ≈ −23 LUFS: sentences arrive at −23 LUFS with peaks ≤ −7.5 dBTP, and a music
stem capped at −12 dBTP cannot push the sum into clipping. (The template's
−9.4 dB narration gain was a balance for the old pad; it goes away with it.)

| Situation | Music level (short-term, 3 s) |
|---|---|
| under speech | **V − 18 LU** (allowed −16 to −22) |
| an authored gap ≥ 2 s | V − 10 LU |
| music-only (title, outro, chapter cards) | up to V − 4 LU |
| any cue during speech, loudest 400 ms | ≤ the speech's loudest 400 ms − 18 LU, low-passed ≤ 2.2 kHz, panned ≥ 30% off centre |
| music stem true peak | ≤ −12 dBTP (the voice, at −7.5 dBTP, always sets the master gain) |

- **Duck offline with perfect look-ahead.** The narration timeline is known
  before the score is synthesized. Voice activity per placed line (10 ms frames,
  active when RMS > line loudness − 25 dB, dilated ±30 ms) gives speech
  regions; regions closer than 1.2 s merge; each region ducks the bed by 8 dB
  with a 0.30 s raised-cosine attack starting 0.08 s early, holds 0.25 s past
  the end and releases over 1.2 s. The duck is baked into the stem.
- **Carve a pocket.** While ducked, a linear-phase FIR (1,023 taps) crossfades
  in: −5 dB bell centred at 2 kHz (≈1–4 kHz), plus −3 dB from 300 Hz to 1 kHz
  on the mid channel only, opening the centre for the narrator while the sides
  stay wide. Same phase on both paths, so the crossfade is an exact,
  click-free time-varying EQ.
- **Never use `gain_to_background`.** franken_manim applies it as a step gain
  on the already-mixed sum, with no ramps, in insertion order.
- **Melody only in the gaps.** Under speech the music holds still (pedals,
  sustained chords, slow harmonic motion); melodic phrases start only in
  authored gaps. A changing melody under speech disrupts verbal memory (the
  changing-state effect), and background music measurably reduces learning
  from multimedia lessons.
- **Authored gaps.** The narration pacing (0.6 s breath, pauses ≤ 0.9 s) merges
  a whole chapter into one speech region, so the music can only answer in holds
  the author writes: `kit.music_breath(stage, 2.5)` at act boundaries and after
  big reveals, roughly once every 60–90 s.
- **Key line.** At most twice per video, the music drops to silence 0.5 s
  before a crucial sentence.

### 7.3 Hard rules (rendering fails if violated)

1. No sustained music fundamentals between 95 and 250 Hz during speech (the v5
   narrator's 5th–95th percentile pitch is ≈105–216 Hz); bass stays below
   95 Hz, upper voices from C4 (262 Hz) up (B3, at 247 Hz, would sit inside the band).
2. Under speech, music content above 2.5 kHz is at least 12 dB down (low-pass
   plus pocket).
3. No pure-sine voices above 100 Hz: at least three partials, chorus or noise.
4. Attacks ≥ 3 ms for mallets and ≥ 8 ms for plucks during speech; nothing
   faster except deliberate ticks low-passed under 3 kHz.
5. Cue density: ≤ 1 foreground cue per sentence during speech; inter-onset
   ≥ 1.5 s; ≤ 6 cues per 30 s; streams faster than 6/s become textures.
6. No identical cue buffer twice within 10 s: round-robin variants plus seeded
   micro-variation (±1.5 dB, ±8 ms, ±10% decay, ±0.1 pan, ±15% mallet
   hardness). Pitches stay in tune; only timbre varies.
7. Cue pitches come from the current chord's pentatonic subset; minor ninths
   against any chord tone are flagged.
8. No melodic phrase begins during speech.

### 7.4 Engine architecture (`crates/explainer_score`, std only)

```text
rng       SplitMix64/xoshiro; seed = fnv1a64(video, chapter, track, event_idx)
units     dB/LU/Hz/MIDI; frames <-> samples exactly as fmn (48 kHz; 800 samples per 60 fps frame)
theory    scales and modes, chords as pitch-class sets, a voice-leading solver, motif transforms
osc       PolyBLEP saw/pulse/triangle, sine, white/pink noise, Ornstein-Uhlenbeck pitch drift
filter    TPT one-pole, Simper SVF, ZDF 4-pole ladder (exact zero-delay solve + ADAA tanh), RBJ biquads,
          linear-phase FIR designer, DC blocker
env       exponential ADSR, raised-cosine ramps, breakpoint automation
fx        ADAA tanh saturation, Juno and Solina-style chorus, tape (wow/flutter delay, head bump, HF shelf),
          FDN reverb (8 lines, Householder feedback, damping, modulation), Dattorro plate, ping-pong delay,
          mid/side, constant-power pan
instr     Pad (Juno-style), Strings (Solina), FmEp (2-op FM electric piano with a ratio-14 tine),
          FeltPiano (additive, stiff-string partials), Mallet {Marimba 1:3.9:9.2, Vibes 1:4:10 + motor tremolo,
          Kalimba, WarmGlass}, Pluck (Karplus-Strong, allpass tuning, pick-position comb), Bass,
          Whoosh, Riser, Bubble (Minnaert), Tick
score     Identity (key, mode, tempo, palette, leitmotif in scale degrees, seed), ChapterPlan (harmony,
          intensity arc, cues anchored to narration line ids and authored marks)
timeline  LinePlacement {id, start_frame, wav, gain}, marks, breaths, fps, total frames
duck      VAD on the placed narration WAVs -> regions -> duck and pocket curves
mix       buses (bed, colour, cue, sfx) -> sends -> duck + pocket -> stem; BS.1770 meter; true peak (4x)
wav       f32 / s24 / s16+TPDF writer; 64 MiB guard; frame-aligned splitting
qa        the §7.6 metrics -> JSON; hard-rule violations fail the render
master    PCM master: narration placements + stems summed in f32 per chapter, chapters concatenated,
          one fixed gain = min(-16 - I, -1.5 - TP), AAC 256k once (D-20)
```

**Integration with the scenes: two passes.** Chapters render in parallel
processes, so a stem made "at the end of construct" cannot see the neighbouring
chapter's narration (duck releases, tails and phrases that cross a chapter
boundary). So scoring is two-pass:

1. **Timeline pass.** Every scene runs construct-only (no frames) and its
   `Narrator` dumps the timeline: line placements with exact integer frames and
   gains, authored marks (`kit.mark(stage, "reveal_mle")`), authored music
   breaths (`kit.music_breath(stage, secs)`), listen windows, and the scene's
   total frames. `add_sound` is a no-op in skip mode, so the recorder is the
   kit's own, not the engine's.
2. **Score pass.** `./x <slug> score` renders the music for each act (or the
   whole video) from the combined timeline, then slices it at frame-aligned
   sample boundaries (800 samples per 60 fps frame) into
   `stems/<chapter>.music.f32.wav`, each exactly its scene's length. Scenes then
   place their stem with `add_sound(path, -stage.scene().time().to_f64(),
   Some(0.0), None)` at the end of construct; the offset always comes from the
   scene clock, never from the kit's own accumulator. A chapter's tail is a
   final `wait`, so the picture covers it.

Stems are cached by a hash of (identity, plan, timeline, narration WAV digests,
crate version); because the master concatenates PCM (§7.7), seams are exact and
the seam rule in §7.6 applies only to drafts.

**DSP details** (formulas, parameter ranges and citations are in the memo §3):
PolyBLEP saws (aliasing −55.5 dB vs −20.3 dB naive at 1.78 kHz); per-oscillator
OU drift with 2–4 cent stationary spread and ±5–9 cent static detune; ladder
cutoff `clamp(2.5–4 × f0 + env, 250, 2000 Hz)` under speech (up to 6 kHz in
gaps), resonance k 0.4–1.5, never self-oscillating; Juno chorus 3.5 ± 1.8 ms
with opposite-phase triangle LFOs near 0.5 and 0.83 Hz; FDN T60 2.5 s for pads,
1.6 s for mallets, 0.9 s for a "bakery room"; tape wow 0.08% at 0.8 Hz and
flutter 0.03% at 8 Hz, head bump +1.5 dB at 70 Hz, −2 dB shelf at 10 kHz; no
hiss under speech and no vinyl crackle anywhere (it is a stream of 2–8 kHz
clicks).

**Performance.** A std-only prototype rendered 10 minutes of 48 kHz stereo
(375 voices, up to 10 at once, 2 PolyBLEP saws + sub + saturated ladder each,
through chorus and an 8-line FDN) in 6.27 s of CPU on one M4 Pro core. Chapters
and buses render in parallel with `std::thread::scope`.

### 7.5 Musical identities

Borrow the *grammar* of 3b1b-style music (major-seventh and added-ninth colour,
a Lydian raised fourth for wonder, chords lasting 8 s or more, piano plus soft
synths), never a specific piece. Each video has a fixed identity; variation
lives inside it.

- **Harmonic plan per chapter:** chords of 4–16 s; changes only in gaps or in
  intra-sentence pauses ≥ 0.2 s, never within 150 ms after a speech onset; a
  voice-leading solver minimizes total motion with the bass in E1–E♭2 (one
  octave, so every root has a bass note; E1–D2 left E♭ chords without one) and upper
  voices from C4 (262 Hz) up, holding common tones; pedal points under dense
  derivations.
- **Intensity arc per chapter (0–1)** drives density, cutoff, register spread
  and reverb send: intro motif at V − 6 then down to the bed at the first line;
  development at density 1–2; thin out 1 s before each reveal; reveal cue in the
  gap; the main result gets the fullest 4–8 s; the outro resolves the motif.

| | A: fish | B: RaptorQ | C: bakery | Hoeffding v6 |
|---|---|---|---|---|
| Key / mode | F major with Lydian B♮ (curiosity); D Dorian for uncertainty | E major with Lydian A♯; E pedal under algebra | B♭ major; whole-tone/augmented tension for races, resolved at `choosing` | D major (as v5's chords), now moving |
| Metre / tempo | 6/8, dotted quarter = 56 | 96 BPM; 16th arpeggios in gaps only | 104 BPM, light swing (56%) | 72 BPM, free |
| Harmony | Fmaj9 → Dm9 → B♭maj9(♯11) → C7sus4 → F/A | Emaj9 → C♯m9 → Amaj9(♯11) → B6sus4 | B♭maj7 → Gm9 → E♭maj9(♯11) → F7sus4 | Dmaj9 → Bm9 → Gmaj9(♯11) → Asus4 |
| Palette | Juno "water" pad (cutoff on a random walk), kalimba = untagged fish, vibraphone with slow motor = tagged fish, felt piano motif, warm bass, bubbles | warm-glass FM bell (low index, harmonic ratios), Juno PWM strings, Karplus–Strong pluck, sub bass, tape wow, plate, ping-pong in gaps | one timbre per process: felt piano, marimba, pizzicato, FM electric piano; small-room reverb (T60 0.9 s) | felt piano + Juno pad + soft vibraphone |
| Leitmotif | 5̂–1̂–2̂–3̂ (C–F–G–A), left open; resolves to 1̂ at the estimate | from 0x1D's set bits {0,2,3,4}: E–G♯–B–C♯ | "take a number" 1̂–2̂–3̂, "served" 5̂–4̂–2̂–1̂ | 1̂–5̂–6̂–3̂ (rising question), resolves at "dependent" |
| Signature sonification | recapture: each tagged fish rings as vibraphone, so k is *heard*; Monte Carlo trials tick at the pitch of their k, the rain's mode is audible | a byte is an 8-note chord (bits → E4 F♯4 G♯4 B4 C♯5 E5 F♯5 G♯5); XOR toggles notes; ×α steps every note up a slot and an overflow flicks in the 0x1D chord; packets arrive as glass droplets, losses as dull ticks | the ticket number is a scale degree; waiting is a soft repeated note; the critical section is the process playing the motif, so only one motif ever sounds at once: mutual exclusion is audible, and a violation sounds like a bug | the shuffle test's 2,000 permutations as a granular texture; observed D a resolving bell dyad |

**Revision 2 additions** (creative direction, round 1):
- **Listen windows.** A sonification that teaches is useless if it is ducked to
  inaudibility, so each one gets an authored narration gap of ≥ 3 s, marked in
  the storyboard's `hold/listen` column, where the duck releases and the cue
  plays at V − 10 LU or louder.
- **A:** the leitmotif resolves at the table's balance (06), not at the early
  10,000 (unearned there); chapter 03's proportion rhythm (vibraphone against
  kalimba at a constant one-in-ten density for nets of any size); during the
  likelihood sweep, a soft tone whose brightness tracks the score, in a gap only.
- **B:** the 0x1D motif sounds at *every* fold, so the viewer learns "that sound
  means reduction"; the **rank chord** (each useful card adds a chord tone, a
  useless one is a muted thud, crowding in at the end of the XOR run); a
  Shepard-style glide of 255 steps (≈4.7 cents each) for the byte clock that
  climbs one octave and lands home (α²⁵⁵ = 1, heard).
- **C:** swing drops out for chapters 07–08 (rubato, a pedal under the lemmas);
  the "two motifs at once = the bug" demonstrations in 05 and 10 get listen
  windows; spinning stays audible in 09, where it *is* the cost.

### 7.6 Score QA (computed per chapter, JSON report; failing metrics fail `./x score`)

| Metric | Threshold |
|---|---|
| voice − music short-term loudness over speech regions | median ≥ 16 LU; 5th percentile ≥ 12 LU |
| music's loudest 400 ms during speech | ≤ speech's loudest 400 ms − 15 LU |
| SII-lite loss per 400 ms (third-octave bands, ANSI S3.5 weights) | median ≤ 0.01; max ≤ 0.03 |
| music-to-voice band ratio, 1–4 kHz, during speech | ≤ −20 dB |
| music stem true peak (4× oversampled); click detector (onset energy > 8 kHz) | ≤ −12 dBTP; 0 flags |
| monotony: chord changes; repeated 8 s segments | ≥ 1 change per 16 s; no 8 s segment repeated > 3× with correlation > 0.98 |
| busyness: music onsets per second during speech | ≤ 2 (≤ 6 in gaps) |
| roughness from the score itself | no sustained partial pair 15–60 Hz apart both above −20 dB; no exposed beating > 6 dB |
| mono compatibility | L/R correlation ≥ 0 averaged per 3 s |
| sanity | no NaN/inf; DC ≤ −60 dBFS; length = scene samples exactly |
| chapter seams | music ≤ −50 dBFS within ±50 ms of every cut while the AAC-per-chapter path exists |

**Listening protocol** (recorded in PROGRESS.md): laptop speakers, earbuds,
and headphones or monitors, each at normal volume and 10 dB lower, plus a mono
phone check; A/B against a no-music render on three random 20 s excerpts per
chapter, second listener blind; one uninterrupted full watch logging every
moment the music was *noticed* during speech (target: none) and every gap that
felt dead.

### 7.7 The PCM master (D-20)

franken_manim mixes each chapter's soundtrack natively, quantizes it to s16
without dither and encodes AAC per chapter (≈129 kb/s unless the runner wrapper
adds `-b:a`); the old `master.sh` then decoded and re-encoded AAC at 256 kb/s.
That is two lossy generations, undithered quantization, and ≈21 ms gaps at
every concat join. The new master rebuilds the soundtrack from sources instead:

- **Mix with fmn's own mixer.** Add `fmn-output` as a git dependency at the same
  rev (the facade does not re-export `SoundMixer`) and replay every chapter's
  logged cues through it, keeping f32 output. That reproduces fmn's resampler
  (64-tap Blackman sinc via `fmn_dmath`), mono→stereo duplication, `fmn_dmath`
  gains and f64 summation exactly.
- **Null test.** The runner wrapper captures fmn's own s16 `soundtrack.wav` from
  the mux step's work directory; our replay, quantized the same way, must null
  against it to ≈ −90 dBFS. (A null test against fmn's AAC output cannot pass:
  the codec error is far above −60 dBFS.)
- **Join and gain.** Chapters concatenate sample-exactly (each is
  `frames × 800` samples at 60 fps); one fixed gain `min(−16 − I, −1.5 − TP)`;
  AAC 256k once.
- **Video join.** Each chapter remuxed video only (`-map 0:v:0`, dropping its
  AAC, so the concat demuxer offsets each chapter by its picture's exact
  length), concatenated with the concat demuxer and `-c:v copy`; then the PCM
  soundtrack is muxed in. Gates: ffprobe frame count = Σ chapter frames; audio
  samples = Σ frames × 800; decode times strictly increasing and every frame k
  presented at k/60 s (± 1 ms); one lossless `h264_metadata` BT.709 re-tag on
  the joined file. (Revised 2026-10-10: the first design, an Annex-B
  concatenation remuxed with regenerated timestamps, failed on Hoeffding's
  real chapters: the raw stream carries no presentation times, so B-frames
  came back with pts = dts, two leading frames were discarded and the decode
  reported non-monotonic DTS.)
- Spike S-9 proves this whole path before any final.

---

## §8 Narration

The narration skill's pipeline is reused unchanged; only the driver and paths
change.

- **Voice:** FrankenTTS `robert` for every video (continuity across the corpus;
  D-06, open question Q2).
- **Pipeline:** `script.tsv` → lint (`lint_script.py`) → lexicon (lookup,
  respell, probe with control and decoy, lock) → `run_pipeline.sh` on the GPU
  host (plan → TTS one call per sentence → fw ASR → choose takes) → triage with
  `alts.tsv` → `qa_lines.py` (mean ≥ 0.97, no line < 0.90, no misheard name) →
  `validate_narration.py` (no stretch, no clipping, even level, intelligible).
- **Edits allowed:** pause surgery, per-sentence loudness to −23 LUFS, a
  look-ahead peak limiter at loudness + 15 dB, one fixed master gain. Never
  time-stretch, pitch-shift or loudnorm.
- **Pacing hooks the score needs:** `kit.music_breath(stage, secs)` and
  `kit.mark(stage, "label")` (§7.2). Breaths are written into the storyboard's
  `hold` column, so the script and the score agree on where the gaps are.
- **Pronunciation lists** are in each video's section (§9.6, §10.6, §11.6).

---

## §9 Video A: How to Count Fish You've Never Seen

*Slug `capture_recapture`. Research: `docs/research/capture_recapture/MEMO.md`
(Feller's text checked against page scans of the 1st and 3rd editions; every
number computed with exact rational arithmetic by `scripts/verify.py`,
`verify2.py`, `appendix.py` and friends). Revision 2 integrates review round 1
(`docs/reviews/plan_r1/`); the new numbers it introduced were re-verified by
`docs/reviews/plan_r1/integration_check.py` and the reviewers' own scripts.*

### 9.1 Brief

| Field | Value |
|---|---|
| Owner's words | "Use the fishing exercise from Feller book 1 … estimate the total number of fish. Why does this work? How exactly? What's the intuition?" |
| Working title | **How to Count Fish You've Never Seen** (alternative: *How Many Fish Are in the Lake?*) |
| One-sentence takeaway | Two independent catches form a little table (fish caught twice, caught only first, caught only second, and a dark cell nobody caught); the hypergeometric distribution shows that the best guess for the dark cell is the one that balances the table, which is exactly the proportion guess N ≈ M·n/k, and it tells us honestly how far off that guess might be. |
| The hook | A net can't tell you how much of the lake it searched: "dip a net and multiply by the lake" gives answers tens of times apart, because fish crowd into some places and avoid others. The marked fish fix exactly this: they measure the net. |
| The hero picture | **The four-cell table** on dark water: **100 \| 900 / 900 \| ?** (caught twice, marked only, caught only second, never caught). Feller's algebra turns out to *be* balancing its diagonals. The mixing picture (density stays clumpy, the marked fraction goes flat) is the reason the table is legitimate. |
| Worked examples | (1) a 20-fish toy lake (M = 5, nets of 4): all 4,845 netfuls countable; (2) a likelihood toy (M = 10, n = 10, k = 3): best size 33; (3) Feller's lake (M = n = 1,000, k = 100): 8,100 unseen, 10,000 in all. |
| Feller's source | *An Introduction to Probability Theory and Its Applications*, Vol. I, 3rd ed. (1968), Ch. II §6, **Example (c)** "Estimation of the size of an animal population from recapture data", pp. 45–46 (1st ed. 1950: §II.5 Example (e), pp. 37–38). On-screen quotes say which edition they come from. |
| Words (one per idea) | **marked** (Feller's word; our marks glow gold, "Feller used red paint; ours glow so you can see them"), **catch**, **caught twice** (= recaptured; define early), **lake size**, **score** for likelihood after it is named once. Never say "n" and "N" in one sentence; the letters live on screen with word labels. |
| Notation (screen) | N lake size, M marked, n second catch, k caught twice; table cells a = M − k, b = n − k, u = N − M − n + k. Feller's n, n₁, r, k mapped once in 02. Probabilities written P_N(K = k) (N is not random). |
| Length | ~13 min, 10 chapters ≤ 85 s (D-05) |
| Out of scope | open-population models (Jolly–Seber), multi-list log-linear models beyond one sentence, the German tank problem except as an optional one-line contrast |
| Oracle | `oracle.py` imports the research scripts' functions (exact `math.comb` / `fractions.Fraction`; seeded numpy for simulations) |

### 9.2 The arc

| Act | Chapters | What the viewer gets |
|---|---|---|
| I. The trick | 01 cold open + hook, 02 the marks measure the net, 03 why the marks spread evenly | the table, the answer, and why it is legitimate |
| II. Making it exact | 04 netfuls, 05 turning the question around, 06 the table balances | counting, likelihood, and Feller's ratio, which *is* the table's balance |
| III. Honesty | 07 check it by cheating, 08 how sure, 09 one rule for every failure | the estimate's scatter first (lake known), then the inversion (lake unknown), then failure modes |
| IV. Everywhere | 10 the dark cell everywhere | Laplace, code review, Kosovo; the coda |

The 07/08 order is deliberate: the viewer first *watches* estimates scatter
around a lake whose size we know, then turns the question around (the same move
as 05) to ask which sizes could have produced our catch.

### 9.3 Color roles

| Idea | Color | Where |
|---|---|---|
| marked fish, M, "caught twice" k | gold `#FFD866` (hero) | fish glow, table cells, t2c |
| unmarked fish | blue `#58C4DD` | fish |
| the second catch n | green `#83C167` | net, tray, table column |
| the dark cell u, the unknown N, the estimate N̂ | teal `#5CD0B3` on near-black | table, slider, needle |
| likelihood ("score") | purple `#9A72AC` | curves, landscape slice |
| probability over k | blue-grey `#8A93A3` bars | histograms |
| bias / failure | red `#FC6255` (never the marks) | gauges, warnings |

### 9.4 Chapters and beats

Columns: **id** (script line), **draft words**, **picture**, **features**, and
**hold/listen** (an authored music breath, or a ≥ 3 s listen window for a
signature sonification, §7.2). Draft words are a starting point for P3.
Prediction pauses (question card held 1.5–3 s) are marked ⟨predict⟩; every
chapter has one or two.

#### 01_hook: The fish nobody caught (~75 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | A thousand fish are caught, marked, and let go. A week later another thousand are caught. A hundred of them are marked. | Top-down lake; first-catch fish glow gold, second-catch fish green, the hundred caught twice glow both | particle swarm, glow | |
| 01_2 | Between the two nets we've seen nineteen hundred different fish. Now look at the fish no net ever touched. | Everything else fades to dark silhouettes; the lake reorganizes into a 2×2 table **100 \| 900 / 900 \| ?** | morph swarm → table | |
| 01_3 | We can say there are about eight thousand one hundred of them, and we can say how sure we are. | The dark cell fills with "≈ 8,100" | live number | listen 2 s (motif, unresolved) |
| 01_4 | *(title)* | **How to Count Fish You've Never Seen** | title card | hold |
| 01_5 | Why not just count one patch and scale up? This net covers one hundredth of the lake. Four hundred and fifty-one fish, times a hundred, is forty-five thousand one hundred. *(the deep-end spot of the seeded lake, A-F27)* | A net drops; a 10×10 grid with one square lit; the arithmetic | net sweep, live count | |
| 01_6 | Try two other spots, and you get wildly different answers. Fish crowd into weeds and the deep end, and avoid open water. A net can't tell you how much of the lake it really searched. | Three estimates on a plain number line spanning ≥ 30× (one P1 number used everywhere); a density heat map fades in; a "real answer: ?" box | heat map, `indicate` | |

- **Personas.** Maya: the table is a puzzle she can see; the patch arithmetic
  is shown. Dev: sampling bias in one picture. Ruth: the claim is not that
  quadrat sampling is "doomed" (many random quadrats can work) but that one net
  cannot measure its own coverage, which is precisely what marks provide.
- **Facts:** the seeded clustered lake (A-F27) gives the dip estimates; spots
  are fixed and sensible, not seed-shopped.

#### 02_the_marks_measure_the_net (~80 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | In nineteen fifty, the probabilist William Feller put this puzzle in his famous textbook. Funny thing. He didn't know biologists had been doing it for years. | Typeset quote card in our typography, edition named; footnote card "This example was used in the first edition without knowledge that the method is widely used in practice." | Text cards | |
| 02_2 | Feller used red paint. We'll make our marks glow. | Gold marks | — | |
| 02_3 | Here's the first way to read it. Our second net caught a hundred of the thousand marked fish. One in ten. | Bar: 100 of 1,000 marked fish caught | proportion bar prop | |
| 02_4 | So it probably caught about one in ten of *all* the fish. ⟨predict⟩ How many fish is that? | Question card; then 1,000 → "one tenth of ?" | prediction card | |
| 02_5 | A thousand is one tenth of ten thousand. The marked fish measured our net for us. | 10,000 appears; the dark cell = 10,000 − 1,900 = 8,100 | write, table callback | |
| 02_6 | Read the same fact the other way. One fish in ten in our catch is marked, so about one in ten in the lake is. Same equation, two readings. | Numbers first: 100/1,000 = 1,000/?; then the letters with word labels: k/M ≈ n/N ⇔ k/n ≈ M/N | TransformMatchingTex | |
| 02_7 | Why should that be true? How sure can we be? And when does it fail? | Three question cards; they become the act titles | statement cards | hold |

- **Personas.** Maya: she does 1,000 = one tenth of ? herself. Dev: the two
  readings are the same cross-multiplication. Ruth: "the careful math agrees"
  is promised, not asserted; 10,000 is called a best guess.
- **Exact version:** the quote card names its edition (A-F35); "precisely:
  E[K] = nM/N for every catch size, by linearity" at 02_6.

#### 03_why_the_marks_spread (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Think about tasting soup. Stir it, and one spoonful tells you how salty the whole pot is. | Stirred-pot icon | icon | |
| 03_2 | We built this lake, so we can peek. The fisher can't. | Label "only we can see this"; god's-eye view | label | |
| 03_3 | Here's a crowded patch. Fifty fish, five marked. Here's a sparse one. Ten fish, one marked. Both one in ten. | Zoom into two patches with counts | zoom (group scaling) | |
| 03_4 | Watch the marked fish swim. The fish stay crowded where they like to be. But the marks spread out evenly among them. | Split view, titled in words: left "how many fish" (stays clumpy), right "what fraction is marked" (starts hot at the dock, flattens to ≈10%); fish circle their favourite haunts along the habitat's contour lines | **habitat-potential dynamics; contour streamlines; maps from a large simulated population on 3×3 regions, recoloured in place** | listen 3 s (proportion rhythm) |
| 03_5 | Wherever the second net goes, about one fish in ten it catches is marked. The catch is a tiny copy of the lake. | Several nets; each tray's marked fraction jitters near 10% | repeated small catches | |
| 03_6 | That's what makes the table fair. Mixing makes the two catches independent, so the overlap tells the truth. | Table returns; "independent catches" label | callback | hold |

- **Personas.** Ruth: the correct statement of why clumpiness doesn't matter is
  that, after mixing, being marked is independent of where a fish is at the
  second catch; uniform density is not required. Dynamics: overdamped motion in a
  habitat potential U with drift −∇U plus a circulation c·J∇U along U's contours
  and noise; the stationary density ∝ e^(−U) stays clumpy, and marked fish relax
  to the same density, so the marked-fraction map flattens. (Divergence-free
  "curl noise" would wrongly flatten the density too.)
- **Sound:** nets of different sizes come up; the *rhythm* of vibraphone
  (marked) against kalimba (unmarked) keeps the same one-in-ten density: you hear
  a proportion stay constant.

#### 04_netfuls: Every possible netful (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | To see exactly why, shrink the lake. Twenty fish, five of them marked. Our net holds four. | Toy lake: 20 fish, 5 gold | morph from big lake | |
| 04_2 | Random means every group of four is equally likely to be the catch. How many groups are there? | Netfuls flash | — | |
| 04_3 | Twenty choices for the first fish, nineteen for the second, eighteen, seventeen. But the same four fish come out in twenty-four different orders, and that's still one netful. Divide by twenty-four. Four thousand eight hundred forty-five. | Four fish shuffling through their 24 orders; `20·19·18·17 / 24 = 4,845`, then "twenty choose four" | count-up §7 | |
| 04_4 | Sort them by how many marked fish they hold. ⟨predict⟩ Which pile is biggest? | Netful glyphs stream into five bins | **4,845 glyphs (S-4)** | listen 2 s (texture) |
| 04_5 | One marked fish. One in four, same as the lake. The soup again. | 1,365 / 2,275 / 1,050 / 150 / 5; the 1 bin glows | histogram prop | |
| 04_6 | Take the pile with two marked fish. Ten ways to pick two marked fish. For each, a hundred and five ways to pick two plain ones, like shirts and pants. One thousand and fifty. | The ten marked pairs; `C(5,2)·C(15,2) = 10·105 = 1,050` | TransformMatchingTex | |
| 04_7 | Divide every pile by all the possible catches and you get the chances. Twenty-eight percent, forty-seven, twenty-two, three, and almost none. | Each number in the 1,050 line swapped for its letter, one at a time, building `P_N(K=k) = C(M,k) C(N−M, n−k) / C(N,n)`; bars as percents (28.2 / 47.0 / 21.7 / 3.1 / 0.1) | symbolic ← numeric | |
| 04_8 | Its grown-up name is the hypergeometric distribution. If we know the lake, we can predict the catch. But we only know the catch. | Name card; arrow "lake → catch" with a question mark | statement card | hold |

- **Persona notes.** Ruth: lower-third "Σₖ C(5,k)C(15,4−k) = C(20,4)
  (Vandermonde)"; probabilities shown to 4 decimals in the fact sheet, rounded
  percents on screen. Glyph: `\binom` (fixed upstream) is the default notation
  on screen; "C(M, k)" read aloud "M choose k".

#### 05_turning_it_around (~85 s; segments a/b/c)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Say we marked ten, caught ten, and three were marked. ⟨predict⟩ Soup trick first. Ten times ten, over three? | Data card; "≈ 33" | prediction card | |
| 05_2 | Now check it properly. What's the smallest the lake could be? We've already seen seventeen different fish. | 17 marked on an axis | — | |
| 05_3 | For each possible lake size, ask how likely it is to see exactly three marked. Here's twenty. Here's thirty-three. Here's sixty. | Three still frames: the histogram for N = 20, 33, 60 with the k = 3 bar lit | stills first | |
| 05_4 | Now sweep through every size. The height of that bar traces a curve. It peaks at thirty-three, just as the soup said. | N tracker sweeps 17 → 60; histogram morphs; the bar drops a dot onto the curve; peak 33, L ≈ 0.318 | **tracker-driven histogram (small family) + traced curve** | |
| 05_5 | That curve is the likelihood. Think of it as a score for each guess. Picking the top score is called maximum likelihood. | Name card ("score") | statement card | |
| 05_6 | Add up every height and you get about seventeen, not one. Chances of all the options would add to one. So these aren't chances that the lake is a certain size. The lake has one size. They're scores for comparing guesses. | Bars summed: 16.67; Feller 1950 quote card: "It is, therefore, meaningless to ask for the probability that n is greater than, say, 6000." | live sum | |
| 05_7 | *(segment b, camera route)* Here's every lake size and every possible count at once. | Still shot first, axes labelled in words ("lake size", "marked fish caught"); then a slow orbit around the lit bar landscape P_N(K = k) | **lit 3D bars + `CameraRig` orbit split < π (S-5)** | |
| 05_8 | Along the floor runs one curve, where the catch's fraction equals the lake's. Slice at our catch, and the score climbs until it meets the curve, then falls. | The crest curve k = Mn/N drawn on the floor; translucent slice planes (drawn last) at fixed N (chances that add to one) and at k = 3 (the score curve) | slicing planes | |
| 05_9 | Same mountains, two questions. We'll meet that curve again. | Camera settles | camera move | hold |

- **Ruth:** the area identity Σ_N L(N) = Mn/(k(k−1)) is **proved** (beta
  integral; `docs/reviews/plan_r1/ruth_45_audit.md` P1) for k ≥ 2 and infinite
  for k ≤ 1; memorable form "area = estimate / (k − 1)" (33.3/2 = 16.67;
  10,000/99 = 101.01). Show it as a lower-third "precisely: …"; it goes in the
  description too. Every fixed-k slice rises left of the crest curve and falls
  right of it (Feller's ratio); fixed-N slices peak near it, at the mode
  ⌊(n+1)(M+1)/(N+2)⌋.
- **Maya:** never say "probability distribution" without the picture; "score"
  is the spoken word from here on.

#### 06_the_table_balances (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Feller found the top score without drawing anything. Compare each lake size with the one just below it. When does the score still go up? | Two adjacent bars on a seesaw | seesaw prop | |
| 06_2 | Most of the counting cancels, and the comparison comes down to this. | The ratio as a two-second lower-third: `L(N)/L(N−1) = (N−M)(N−n) / (N(N−M−n+k))` ("the cancelling is in the description") | lower-third | |
| 06_3 | ⟨predict⟩ Multiply out the top and the bottom. What survives? | `(N−M)(N−n)` and `N(N−M−n+k)` expand; matching terms strike through in pairs (drawn strike lines) | **term-by-term cancellation** | |
| 06_4 | M times n, minus k times N. If that's positive, the score is still rising. In our small lake, ten times ten is a hundred. Three times thirty-three is ninety-nine: still rising. Three times thirty-four is a hundred and two: falling. The top is thirty-three. | `Mn − kN` with live numbers | live arithmetic | |
| 06_5 | Now write it with the table's four cells. It becomes the marked-only fish times the caught-only fish, minus the overlap times the dark cell. | `Mn − kN = ab − k·u`; the table's two diagonals glow | **TransformMatchingTex onto the table** | |
| 06_6 | So the score keeps climbing as fish are added to the dark cell, until overlap times dark cell catches up with the two side cells. Nine hundred times nine hundred, over a hundred. Eight thousand one hundred. | Seesaw pans k·u and a·b; the dark-cell counter ticks up to 8,100; the beam levels | seesaw + counter | listen 2 s (motif resolves) |
| 06_7 | That balance is exactly what two independent catches look like. The careful argument and the soup argument are the same argument. | Cards merge | statement card | |
| 06_8 | At Feller's numbers the two sides are a million and a million. Perfectly level. A lake of nine thousand nine hundred ninety-nine and a lake of ten thousand tie for the top score. Feller just says ten thousand. | Level beam; the eight-digit fractions only in a lower-third | live fraction | hold |

- **Precisely (lower-third):** "MLE = ⌊Mn/k⌋; a tie with Mn/k − 1 when Mn/k is a
  whole number and k < min(M, n); none when k = 0. Equivalently, the best dark
  cell is ⌊ab/k⌋."
- **Ruth:** Mn − kN = (M−k)(n−k) − k·u is exact algebra (checked at N = 9,999 /
  10,000 / 10,001: +100, 0, −100; toy 49/3 → u = 16 → N = 33).
- **Music:** the leitmotif resolves here, not at 02's 10,000 (unearned there).

#### 07_check_it_by_cheating: Ten thousand lakes (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Let's check all this by cheating. Build a lake where we know the answer, ten thousand fish, and run Feller's experiment ten thousand times. | 100 × 100 grid of tiny lakes flashing their estimates | **10,000 cells as 100 row VMobjects recoloured in place** | listen 3 s (trial rain at the pitch of k) |
| 07_2 | The counts of fish caught twice pile up in a bell around a hundred. | Histogram of k (mean 100, SD 9.0) | histogram prop | |
| 07_3 | But the estimate divides by that count, and dividing bends things. Ninety caught twice gives about eleven thousand one hundred. A hundred and ten gives about nine thousand one hundred. Ten too few pushes harder than ten too many. | The k-bell's bars travel along the hyperbola N = Mn/k (the crest curve from 05) and stretch into a right-skewed histogram | **bars along a curve** | |
| 07_4 | Ninety-five percent of estimates land between about eighty-five hundred and twelve thousand. Their average is about ten thousand and eighty, a little high. | Quantiles 8,475 / 10,000 / 12,048; mean ≈ 10,080 | live stats | |
| 07_5 | *(for programmers)* Two lines of code, and the simulation agrees with the exact calculation. Test your simulation against something you can compute. | Code panel: `k = rng.hypergeometric(M, N-M, n, size=10_000)`, `est = M*n/k` | code panel slides in | |
| 07_6 | With a smaller study it gets worse. In a lake of a thousand, marking and catching fifty, about one study in fourteen catches no marked fish at all. Then the formula asks you to divide by zero. | P(k = 0) = 7.2%; an estimate flies off the edge | red | |
| 07_7 | Douglas Chapman's fix: add one to each count, and subtract one at the end. It's as if one imaginary fish had been caught both times. It never divides by zero, and on average it's almost exactly right once you expect a few fish caught twice. | `(M+1)(n+1)/(k+1) − 1`; small-study means: Chapman ≈ 935 (−6.5%) vs the plain formula, among studies that caught any, ≈ 1,241 (+24%) | TransformMatchingTex | hold |

- **Precisely (lower-third at 07_7):** "E[N̂_C] = N − (N+1)·C(N−M, n+1)/C(N+1,
  n+1): exactly unbiased when M + n ≥ N; −1.1% at E[k] ≈ 4." Proof: absorption
  identity + Vandermonde (Ruth P2).
- **Dev:** "add one" here is not ad hoc smoothing; it is the absorption
  identity (n+1)/(k+1)·C(M,k) = C(M+1, k+1). Planning card: "expect about 10
  fish caught twice for ±30%, about 100 for ±10%."
- **Facts:** the seeded simulation (seed `20261009`) and the exact pushforward
  of the hypergeometric through k ↦ Mn/k agree on quantiles; means agree to
  Monte Carlo tolerance (10,080.0 vs exact 10,082.5).

#### 08_how_sure: Turning it around again (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | We knew the lake and watched the estimates scatter. Now turn it around, like before. We don't know the lake. Which sizes could plausibly have given us a hundred? | Score curve for Feller's numbers (relative to its top) | Axes + `c2p`, FunctionGraph | |
| 08_2 | Could the lake hold only nineteen hundred fish? Then our two catches would be every fish in it, and a catch like ours would have a chance of about five in a one followed by four hundred thirty zeros. | Zeros scroll across the screen | scrolling digits | |
| 08_3 | A million fish? Then one fish in a thousand is marked. We'd expect to catch about one marked fish, not a hundred. | Marker far right | — | |
| 08_4 | Even at the best size, catching exactly a hundred had only about a four percent chance. ⟨predict⟩ Is that a problem? Flip a hundred coins. Exactly fifty heads happens only about eight percent of the time, yet fifty is the most likely count. | Coin inset: P(50 of 100) = 7.96% | inset histogram | |
| 08_5 | Here's a ring toss. Build a range around each study's estimate in a careful way, and it lands around the true size at least ninety-five times in a hundred, whatever the true size is. | Twenty stacked ranges from the 07 simulation; one misses; then the name **confidence interval** | interval stack, then name | |
| 08_6 | For Feller's catch, that range runs from about eighty-four hundred to about twelve thousand one hundred. Lopsided, for the reason we just saw. | [8,400, 12,073] shaded; callback to 07's bent histogram | shading | |
| 08_7 | What makes it tight? Mostly, catching marked fish again. Catch four times as many and the range halves. Catching a big share of the lake helps a little too. Catch every fish and no doubt is left. | Slider on k; the range shrinks; relative error ≈ √((1 − M/N)(1 − n/N)/k) ≈ 9% | tracker-driven interval | hold |

- **Precisely:** "exact test inversion, [8,400, 12,073]; coverage ≥ 95% at
  every N"; Feller's own "8,500 to 12,000" (normal approximation) is a ≈93.6%
  range (footnote).
- **Ruth:** the relative-error formula is the symmetric (hypergeometric)
  delta method; the exact SD of Mn/K (given K > 0) at Feller's numbers is
  **924**; the textbook √(1/k − 1/n) = 9.5% ignores the finite lake and is not
  used. Marking 2,000 and catching 500 is statistically the same experiment as
  marking 500 and catching 2,000 (Feller eq. 6.2): exact SD 894 both ways (a
  delight, one line).
- **Dev:** that 5 × 10⁻⁴³⁰ underflows a float64 (smallest normal ≈ 10⁻³⁰⁸), "which
  is why real code works in logarithms or in ratios like Feller's". In the
  description: SciPy's `hypergeom(M, n, N)` takes (total, marked, draws), the
  opposite letters from the video's.
- **Optional one line (Bayesian):** "Want a probability for the lake size? You
  need a starting belief. With a flat one, 8,536 to 12,186: almost the same
  numbers, a different question."

#### 09_one_rule_for_every_failure (~80 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Every failure comes from one rule. The estimate goes wrong when the two catches stop being independent, or when the lake changes in between. | The table with a gauge on the dark cell | gauge prop | |
| 09_2 | ⟨predict⟩ Some fish are bold and easy to catch, both times. Too big, or too small? | Bold fish glow; quiz card | prediction card | |
| 09_3 | They get caught twice more than their share. The overlap swells, the dark cell shrinks, and the estimate comes out small. About seventy-seven fifty instead of ten thousand. | Overlap cell swells; gauge to ≈7,750 | table distortion | |
| 09_4 | Fish that learn to love the bait do the same. Fish that learn to avoid it push the estimate up. | ≈7,140 and ≈14,490 | gauge | |
| 09_5 | Marks that wash off make marked fish look unmarked. Up. | ≈12,500 | gauge | |
| 09_6 | Newcomers between the catches dilute the marks. You've counted the lake as it is at the second catch. | ≈12,120 | gauge | |
| 09_7 | ⟨predict⟩ What if a third of the fish die between the catches? Nothing happens. Death doesn't care about marks, so the fraction stays the same, and you still learn the lake as it was. | Gauge stays at 10,000 | huh beat | |
| 09_8 | And if the marks never mix, everything depends on where you fish. Feller warned about catches made in one spot within a short time. | The 03 split view, now *not* flattening; ≈6,250 at the release site, ≈25,000 on the far side; Feller caveat card | callback | hold |

- **Ruth:** each direction is a seeded simulation (true lake 10,000, M = n =
  1,000; A-F22). Biologists answer with many recapture rounds and richer models;
  Schnabel's 1938 multi-census estimator was published in the *American
  Mathematical Monthly* (one line, optional).

#### 10_the_dark_cell_everywhere (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | The same table counted France. Every birth was registered, so Laplace treated each newborn as a marked fish. | Stylized outline of France; births as marks | outline asset | |
| 10_2 | In selected towns he counted everyone. About two million people, about seventy-two thousand births a year. One birth per twenty-eight people. Taking France's births as about a million a year, that's about twenty-eight point four million people. | The same template: births = marked, people counted = catch, births among them = caught twice | template morph | |
| 10_3 | He even gave odds. Almost three hundred thousand to one that he was within half a million. | Odds card | — | |
| 10_4 | *(for programmers)* It counts bugs. Two reviewers read the same code separately, without seeing each other's notes. One finds thirty, the other twenty-four, and twelve are the same. Together they found forty-two. The estimate is sixty, so about eighteen are still hiding. | Two lists; Venn; the 2×2 table again | Venn via boolean ops | |
| 10_5 | But twelve overlaps isn't much. An honest range is about forty-seven to ninety-four, so somewhere between five and fifty-two are hiding. And bugs that are easy for everyone make it a floor. That's the bold-fish problem again. | Range bar; callback to 09_3 | callback | |
| 10_6 | And it has counted the dead. During the war in Kosovo, in nineteen ninety-nine, about four thousand four hundred victims were named across several lists. The overlaps between lists estimated about ten thousand four hundred. Years later a full count found nine thousand seven hundred ninety, inside the estimate's range. | Sober typography only; numbers; the range 9,002–12,122; a four-list table with sixteen cells, one dark | quiet card, **silence** under the numbers | hold |
| 10_7 | France, the bugs, Kosovo. Each is the same table, with one cell nobody can see, filled in by the cells we can. | Final lake; the table; motif resolves | coda; receipts card ("every number in this video was computed by code: link") | |

- **Ruth:** Laplace's sample: thirty départements spread evenly over France,
  and within them the communes with the most reliable records, counted by the
  government at his request (2,037,615 people; 215,599 births in 3 years =
  71,866⅓ a year; ratio 28.352845; 28,352,845 for 1,000,000 births, which
  Laplace *assumed*); not a census (checked against the Essai in P1:
  docs/facts/capture_recapture/fact-sheet.md). Kosovo used
  multiple-systems estimation with log-linear models on four lists, not a
  two-list formula. Mills (IBM report, 1972; his collected papers date the
  article 1970) seeded bugs deliberately: painted fish
  (description).
- **Dev:** the two-reviewer formula survives reviewers of different skill (the
  M_t model); it is bugs that differ in difficulty for everyone (M_h) that bias
  it low. Exact 95% range [47, 94] by test inversion; Chapman 58.6 ± 8.4.
- **Optional (cut first if long):** the German tank problem as a contrast
  (serial-number order, not overlap): June 1940, statistical 169, intelligence
  1,000, records 122.

### 9.5 Facts ledger (every number; verified in `docs/research/capture_recapture/scripts/` and `docs/reviews/plan_r1/`)

| id | quantity | value | how verified |
|---|---|---|---|
| A-F1 | C(20,4) | 4,845 = 20·19·18·17/24 | exact |
| A-F2 | netful counts k = 0…4 (N=20, M=5, n=4) | 1,365 / 2,275 / 1,050 / 150 / 5 | exact |
| A-F3 | their probabilities | .2817 / .4696 / .2167 / .0310 / .0010 (screen: 28.2 / 47.0 / 21.7 / 3.1 / 0.1%) | exact rational |
| A-F4 | k = 2 decomposition | C(5,2)·C(15,2) = 10·105 = 1,050 | exact |
| A-F5 | toy best size (M=n=10, k=3) | 33; L(33) = 0.3178; soup 100/3 = 33.3 | exact |
| A-F6 | toy comparisons | Mn − kN: 100 − 99 = +1 at 33; 100 − 102 = −2 at 34 (ratios 529/528, 576/578) | exact |
| A-F7 | likelihood area | Σ_N L(N) = Mn/(k(k−1)): toy 16.667, Feller 101.01; infinite for k ≤ 1 | **proved** (beta integral) + numeric |
| A-F8 | toy exact 95% CI; Chapman | [19, 138]; 29.25 | exact |
| A-F9 | Feller best size | {9,999, 10,000} tie (k < min(M,n)); L = 0.04429 at both | exact |
| A-F10 | Feller comparisons | Mn − kN = +100 / 0 / −100 at 9,999 / 10,000 / 10,001 | exact |
| A-F11 | minimum lake | 1,900; P_1900(K = 100) = (1000!)²/(100!·1900!) ≈ 5.4 × 10⁻⁴³⁰ | lgamma |
| A-F12 | relative likelihoods | 8,500: 0.171; 12,000: 0.152; 1,000,000: 3.8 × 10⁻¹⁶² | lgamma |
| A-F13 | exact 95% CI (test inversion) | [8,400, 12,073] | exact fractions at the boundaries |
| A-F14 | Feller's 8,500–12,000 tails | 0.0347 / 0.0289 (≈93.6% range) | exact |
| A-F15 | SD of k at N = 10,000 | 9.000 (binomial approximation 9.487) | exact |
| A-F16 | spread of the estimate | exact SD of Mn/K given K > 0: 924 (CV 9.2%); symmetric approximation √((1−M/N)(1−n/N)/k) = 9.0% | exact pushforward |
| A-F17 | Chapman (Feller's data) | 9,919.8; SE 883 | exact |
| A-F18 | Chapman bias identity | E[N̂_C] = N − (N+1)·C(N−M, n+1)/C(N+1, n+1) | **proved** + exact fractions |
| A-F19 | simulated quantiles (Feller's lake) | 8,475 / 10,000 / 12,048 (k = 118 / 100 / 83); sim mean 10,080.0 (exact 10,082.5) | seeded sim + exact |
| A-F20 | small study N = 1,000, M = n = 100 | 625 / 1,000 / 2,000; mean 1,102 (exact; the seeded sim gives 1,103) | sim + exact |
| A-F21 | small study N = 1,000, M = n = 50 | P(k = 0) = 7.20% (1 in 13.9); Chapman exact mean 935.2 (−6.5%); plain formula given k > 0: 1,241 (+24%) | exact |
| A-F22 | failure-mode medians | bold fish 7,752; trap-happy 7,143; trap-shy 14,493; marks lost 20% 12,500; +2,000 newcomers 12,122; 30% deaths 10,000; release site 6,250; far side 25,000 | seeded sims |
| A-F23 | Laplace 1802 | 2,037,615 people; 215,599 births / 3 yr = 71,866⅓ / yr; ratio 28.352845; 28,352,845; odds "almost 300000 against 1" | arithmetic + memo quote |
| A-F24 | two reviewers | 30, 24, 12 → 42 seen; estimate 60; 18 hiding; exact 95% [47, 94] (5–52 hiding); Chapman 58.6 ± 8.4 | exact |
| A-F25 | Kosovo | ≈4,400 named; estimate 10,356 (95% 9,002–12,122); later count 9,790 | Ball et al. 2002; Spagat 2014 |
| A-F26 | Bayesian aside (flat prior) | 8,536–12,186; posterior mean (M−1)(n−1)/(k−2) = 10,183.7 | numeric + closed form |
| A-F27 | dip estimates (01) | one net = 1/100 of the lake: weed bed 880 → 88,000; open water 28 → 2,800; deep end 451 → 45,100 (31.4× apart) | seeded clustered lake, fixed spots (docs/facts/capture_recapture/oracle.py) |
| A-F28 | split symmetry | M = 2,000, n = 500 and M = 500, n = 2,000: exact SD 894 both | exact pushforward |
| A-F29 | table identity | Mn − kN = ab − k·u; Feller û = ⌊900·900/100⌋ = 8,100; toy û = ⌊49/3⌋ = 16 | algebra + exact |
| A-F30 | coin analogy | P(50 heads of 100) = 7.96% | exact |
| A-F31 | bending | 10⁶/90 = 11,111; 10⁶/110 = 9,091 | arithmetic |
| A-F32 | float64 | 5.4 × 10⁻⁴³⁰ < 2.2 × 10⁻³⁰⁸ (smallest normal double) | arithmetic |

Non-numeric claims (cite in the fact sheet): Feller's example letter and pages
by edition; footnote 13 wording; "an unknown fixed number" (Feller 3rd ed.) and
the 1950 "meaningless to ask" sentence (A-F35, quoted with its edition);
maximum likelihood developed and named by Fisher (1912–1922, the span of
Aldrich 1997's title), with earlier versions by Lagrange, Daniel Bernoulli,
Laplace and Gauss (Stigler 2007, who says Lambert "got what was arguably the
wrong answer"; checked in P1);
Chapman (1951) and Bailey (1951) cited by Feller; Schnabel 1938 in the
*Monthly*; Petersen (1896) did **not** compute Mn/k himself (Goudie & Goudie
2007), so the video never says "Petersen's formula"; Briand et al. (2000) found
capture–recapture bug estimates tend to underestimate.

### 9.6 Pronunciations to probe

Feller, Laplace, Fisher, Chapman, Schnabel, Petersen, Lincoln, hypergeometric,
Vandermonde, Kosovo, Bayesian, binomial, départements.

### 9.7 Engineering notes (from the engine review)

- **Fish:** procedural shapes built once at the origin and shifted; motion by
  placement only (shift; the tail is a child rotated by placement), never by
  rewriting points; `v_highlight` glow on marked fish only. Spike S-2 sets the
  count.
- **Maps and grids** (03_4, 07_1): fixed mobjects recoloured in place by
  updaters (one VMobject per row), never `always_redraw` (arena slots are never
  reclaimed; spike S-8 measures fps at t = 0 vs t = 80 s).
- **4,845 netfuls** (S-4): dt-updater shifts along precomputed paths; fallback
  a representative stream with exact count-ups.
- **3D landscape** (S-5): 44 × 11 lit bars; slice planes drawn last; orbit split
  into arcs < π; the segment goes through the kit's camera wrapper (same encoder
  argv as the retained route).
- **Notation:** `\binom` by default (fixed upstream); strike-throughs are drawn
  lines (no `\cancel`); boxes are `SurroundingRectangle` (no `\boxed`).
- **Kosovo tone:** Q6.

---

## §10 Video B: The Arithmetic Where 3 × 7 = 9

*Slug `raptorq_gf256`. Research: `docs/research/raptorq_gf256/MEMO.md`
(an RFC 6330 implementation byte-identical to the independent `cberner/raptorq`
crate for K′ = 10, 26, 101; every field fact checked exhaustively; rank
probabilities exact and Monte-Carlo-confirmed). Revision 2 corrects two
blockers from review round 1: RaptorQ's packets are plain XOR (byte arithmetic
lives only in 10–16 check rows), and the one-in-a-million figure is the
standard's requirement, not our measurement. The owner's sources: "The Black
Magic of Liquid Data" (jeffreyemanuel.com/writing/raptorq) and his X post.*

### 10.1 Brief

| Field | Value |
|---|---|
| Owner's words | "The finite field math that underlies RaptorQ fountain codes" |
| Working title | **The Arithmetic Where 3 × 7 = 9** (alternative: *Liquid Data*). 3·7 = 9 holds in every representation of GF(256), because no reduction occurs. |
| One-sentence takeaway | RaptorQ sends plain XOR packets; a random set of exactly enough XOR equations fails to pin down the file 71% of the time, but almost all of that risk lives in the last few equations, so RaptorQ closes those few directions with 10–16 check equations in a 256-element byte field, where a random equation misses any given hole only once in 256; and we can build that field because we multiply bytes as polynomials modulo one that cannot be factored. |
| The hook | A million phones, each losing a different 1 in 100 packets, none able to ask twice. A carousel (send the file on a loop) makes most phones wait two or three loops and the unluckiest five or six. |
| The hero pictures | (1) **The bucket's danger meter:** equations filling a space; the lit (already-known) fraction is near zero for almost the whole fill and spikes only at the end. (2) **The byte clock:** a 255-hour clock where multiplying is adding clicks, set against the wrong 256-hour clock that adds. |
| Worked examples | A=5, B=3, C=7, D=2 with A⊕C=2, B⊕C⊕D=6, A⊕B=6, A⊕B⊕C⊕D=3 (rank 4) and the redundant B⊕C=4; GF(2)³ (8 patterns); 3·7 = 9, 2·128 = 29, 128·4 = 58, 150·3 = 167, 2·142 = 1, 0x57·0x83 = 0x31 (AES 0xC1); the 5-unknown peeling toy (C, A, B, D, E; c = 7); the pentagon (3A = 15 ⇒ A = 5). |
| Words | "piece" (source symbol; a whole chunk, ≈1 KB, not one byte), "packet" (encoding symbol), "card" for one equation on screen, "useful" (= independent), "wrap around" (said "mod" once), "byte field" (named GF(256) once). |
| Length | ~13.5 min, 10 chapters (D-05) |
| Out of scope | the RFC's tuple generator, systematic index tables, robust-soliton derivation, LDPC row construction, Reed–Solomon decoding algorithms |
| Oracle | `oracle.py` imports `docs/research/raptorq_gf256/scripts/`; measured RaptorQ counts are recorded outputs pinned as constants, not recomputed per build |

### 10.2 The arc ("each fix exposes the next problem")

| Act | Chapters | Spine |
|---|---|---|
| I. Packets as equations | 01 hook, 02 equations, 03 the XOR lottery | loops are slow → equations work → but exactly enough random XOR equations fail 71%, and the danger lives at the end |
| II. Building a better number system | 04 wrapping at 256, 05 polynomials, 06 the byte clock | 256 choices per coefficient need division; wrapping around fails exactly as badly as XOR; folding the overflow by a polynomial that can't be factored works |
| III. Spending it wisely | 07 the byte lottery, 08 the price of density, 09 where the danger lives | byte coefficients make redundancy 256× rarer per equation; dense math is slow; RaptorQ keeps packets XOR and spends a few byte rows exactly where the danger lives |
| IV. Payoff | 10 two packets | measured failures, the standard's guarantee, deployments, the closing line |

### 10.3 Color roles

| Idea | Color |
|---|---|
| pieces A–E | a fixed 5-color ramp (blue → teal → green → gold → orange) |
| packets / cards | warm white cards with colored chips for their pieces |
| XOR / GF(2) | cool blue `#58C4DD` |
| the byte field / byte rows | gold `#FFD866` (hero) |
| lost packets | red `#FC6255`, dimmed |
| the lit span ("what we can already make") | translucent teal |
| the fold polynomial 0x11D | orange `#F0AC5F` |
| wrap-around (Z/256) | desaturated purple `#9A72AC` |

### 10.4 Chapters and beats

#### 01_hook: A million phones (~80 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | I'm sending this picture to a million phones. Each one loses a different one packet in a hundred, and none of them can ask twice. | Tower; a field of ~1,000 phone glyphs (each stands for 1,000), each with its own red loss pattern | particle field, seeded losses | |
| 01_2 | Yet almost every phone that catches just two more packets than the picture has pieces rebuilds it exactly. The secret is an arithmetic where three times seven is nine. | `3 × 7 = 9` in chalk | write | listen 2 s |
| 01_3 | *(title)* | **The Arithmetic Where 3 × 7 = 9** | title card | hold |
| 01_4 | A file travels in thousands of small pieces called packets. Lose some, and the usual fix is to ask again. With a million phones, each packet goes missing for about ten thousand of them, and the requests swamp the sender. Broadcast TV has no way to ask at all. | Request arrows flood the tower; then "no return channel" | arrow swarm | |
| 01_5 | So send the file on a loop. Miss a piece, and you wait a whole loop for it. Most phones need two or three loops. The unluckiest of a million need five or six. | Carousel; a loop counter per phone; the long tail | carousel prop | |
| 01_6 | Sending random pieces is worse. It's a sticker album, where the last few take forever because you keep getting doubles. ⟨predict⟩ How many random pieces to see all ten thousand? About ninety-eight thousand. | Sticker-album count-up to 97,876 | count-up §7 | hold |

- **Ruth:** "about ten thousand" assumes 1% loss (said); 97,876 ≈ K·H_K =
  97,876.06 on average (not a ceiling); "two more packets" and "almost every"
  are probabilistic, stated exactly in 10.
- **Facts:** carousel at 1% loss, K = 10,000: mean 2.64 loops; P(all 10⁶ phones
  done) 0.37 by 5 loops, 0.99 by 6 (B-F24).

#### 02_equations: Packets are equations (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | First, how computers write numbers. Each spot is worth double the one to its right. One, two, four, eight. Five is four plus one, so it's one, zero, one. A byte is eight spots, so it holds zero to two fifty-five. | Place values under bit rows | binary primer prop (≈20 s) | |
| 02_2 | Now the trick. Don't send pieces. Send combinations. The combining move is called ex-or. Think of each one as a light switch that flips its spot. Flip twice, and you're back where you started. | Bit rows as light switches; 5 ⊕ 7 = 2; then 2 ⊕ 7 = 5 | bitwise animation | |
| 02_3 | Each packet says which pieces it flipped together, and the result. That's an equation. | Cards A⊕C = 2, B⊕C⊕D = 6, A⊕B = 6, A⊕B⊕C⊕D = 3; the grid view (rows = cards, columns = pieces) appears beside them | equation cards + grid | |
| 02_4 | Solving works like in algebra class. Ex-or card one with card three, and A cancels: B ex-or C is four. Put that into card two, and D is two. And so on. | One step slow, the rest fast-forward; "A? B? C? D?" fill in | TransformMatchingTex | |
| 02_5 | The phone is just a bucket. It doesn't care which packets arrive, only that it gets enough useful ones. Order doesn't matter. | Bucket; a "useful cards" gauge 1 → 4 | gauge | |
| 02_6 | ⟨predict⟩ Here's a fifth card. B ex-or C is four. Useful? No. You could make it yourself from cards one and three. | The card bounces off | — | |
| 02_7 | A sender can make endless fresh combinations, like water from a fountain, so these are called fountain codes. Each packet carries a small number, and both sides feed it to the same random-number generator to know which pieces it combined. | Fountain icon; "packet #48213 → pieces {…}" | — | hold |

- **Lower-thirds:** "each piece is really a chunk, about a kilobyte; every
  operation runs across all its bytes at once"; "lost, not corrupted: checksums
  throw bad packets away (one that slips through would poison the decode)".
- **For programmers:** RAID-5 parity is one of these equations.
- **Ruth:** ex-or of several bits is one if an odd number are one.

#### 03_the_xor_lottery (~85 s; segment b on the camera route)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Use three pieces for a moment. A card either uses a piece or it doesn't, so there are eight possible cards, from nothing at all to A, B and C together. | 8 patterns labelled with letters | — | |
| 03_2 | *(camera)* Put them on the corners of a cube. | 3D cube; corners labelled; "nothing" already lit | **3D cube, orbit < π (S-5)** | |
| 03_3 | With one useful card you can make two things: nothing, or that card. With two, four: nothing, the first, the second, both. Each useful card doubles what you can make. | Lit set grows 1 → 2 → 4 (a plane through "nothing") | lit span | |
| 03_4 | ⟨predict⟩ Two cards in hand. The next card is random. What's the chance it's useless? Four lit corners out of eight. Half. | Random arrows; half land in the lit set | Monte Carlo arrows | |
| 03_5 | For all three cards to be useful: seven eighths, times six eighths, times four eighths. About one in three. With many pieces it settles at twenty-nine percent. So exactly enough random ex-or cards fail about seventy-one percent of the time. | 7/8 · 6/8 · 4/8 = 0.328 → 0.2888 | live product | |
| 03_6 | Watch where the danger is as the bucket fills. For almost the whole fill, a random card is nearly certain to help. All the danger is packed into the last few slots. One in eight. One in four. One in two. Ten pieces or ten million, the answer is about seventy-one percent. | **The danger meter**: lit fraction vs fill, flat near zero, spiking at the end | danger-meter prop | listen 3 s (rank chord + thuds) |
| 03_7 | Spares help, slowly. Each extra card only halves the risk. Two spares still fail one time in four. With a million phones, that's a quarter of a million stuck. Reaching one in a million takes about twenty spares. | Failure 71 / 42 / 23 / 12% and ≈20 spares for 10⁻⁶ | table | |
| 03_8 | And sometimes it's structure, not luck. Five cards around a loop, each the ex-or of two neighbours. Every letter appears twice, so together they cancel to nothing. One of them is always useless. | Pentagon; letters pair up and vanish | pentagon prop | |
| 03_9 | What if a card could use three A's, or a hundred A's? Then multiplying a byte must give a byte, and we must be able to undo it. | Coefficients {0,1} grow to 256 dots | cliffhanger | hold |

- **Ruth:** P(K random 0/1 rows independent) = ∏(1 − 2⁻ⁱ); 0.328 at K = 3, →
  0.288788; the average number of spares is Σ 1/(2ⁱ − 1) = 1.6067, the
  Erdős–Borwein constant (irrational, Erdős 1948 *(verify the citation)*); the
  pentagon is rank 4, a structural failure.
- **Sound:** the **rank chord**: each useful card adds a chord tone; a useless
  one is a muted thud; near the end of the GF(2) run the thuds crowd in.

#### 04_why_not_wrap_around_at_256 (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | On a twelve-hour clock, nine plus five is two. You wrap around. Mathematicians call this mod. | 12-hour clock | clock prop | |
| 04_2 | Bytes can wrap around at two fifty-six. To use three A's in a card, we'll have to undo times three, which means dividing. | 256-hour clock | — | |
| 04_3 | ⟨predict⟩ Double one twenty-eight. The top bit slides off the end, and it's gone. Two times one twenty-eight is zero. | Bits slide left; the top bit drops into a dark gutter; `2·128 ≡ 0` | **fallen-bit animation** | |
| 04_4 | So a card that says two times A is six can't tell you A. It could be three. It could be one thirty-one. | 2·3 = 6 and 2·131 = 262 → 6 | — | |
| 04_5 | Run the lottery with random byte amounts and this arithmetic, and it fails seventy-one percent of the time. Exactly as often as plain ex-or. Only the last bit of each amount was doing anything. | Two counters (XOR and wrap-around) converge to 71% | **lottery race** | |
| 04_6 | A clock with a prime number of hours doesn't have this problem. ⟨predict⟩ On a seven-hour clock, three times what is one? Five. Every row of its times table, except zero's, is a reshuffle of every value. | 7-hour table; then 16-hour vs 7-hour heat maps | small tables | |
| 04_7 | Zero sneaks in because two fifty-six splits into two times one twenty-eight. Seven can't split. But two fifty-seven doesn't fit in a byte. | 256 splits into 2 × 128 beside the 2·128 = 0 card; 257 is written into an 8-bit box and its ninth bit spills out, lit red | factor split, byte box | |
| 04_8 | A number system where you can add, subtract, multiply, and divide by anything but zero is called a field. We need one with exactly two hundred fifty-six members. So catch the falling bit, and fold it back in. | Name card "field"; the gutter bit lifts back | statement card | hold |

- **Precisely (lower-thirds):** "a K×K matrix over Z/256 is invertible ⇔ its
  determinant is odd ⇔ it is invertible mod 2; uniform entries reduce to uniform
  bits, so success = ∏(1 − 2⁻ⁱ)" (B-F25); "finite fields exist exactly for
  prime-power sizes".
- **Big tables** (S-3) are built as row strips (one multi-subpath VMobject per
  row with per-point fill), never 65,536 separate mobjects.

#### 05_bytes_are_polynomials (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Stop reading a byte as a number. Read its bits as the numbers in front of the powers of x. | 10000000 → x⁷; 00000011 → x + 1; place values beneath | bits → Tex morph | |
| 05_2 | Here's the one weird rule. Those numbers in front are only zero or one, and one plus one is zero, like flipping a switch twice. So x plus x is zero. Adding is ex-or again. | `x + x = 0`; column-wise XOR | callback to 02 | |
| 05_3 | ⟨predict⟩ Three is x plus one. Seven is x squared plus x plus one. Multiply them out, remembering one plus one is zero. | FOIL: x³ + 2x² + 2x + 1 → the doubled terms vanish → x³ + 1 = 1001 = 9 | **FOIL with vanishing terms** | |
| 05_4 | Three times seven is nine. | Chalk `3 × 7 = 9` (callback to 01) | — | listen 2 s |
| 05_5 | When a product grows past x to the seventh, it no longer fits. The simplest fold drops the fallen bit into the bottom slot, a rotation. But then three times two fifty-five comes out zero. The fold has to follow a rule that can't be factored. | Rotation fold; `3·255 = 0` mod x⁸+1, because (x+1)(x⁷+…+1) = x⁸+1 | fold animation | |
| 05_6 | The rule RaptorQ uses is x to the eighth equals x to the fourth plus x cubed plus x squared plus one. It's the polynomial version of a prime clock. | `x⁸ = x⁴+x³+x²+1` (0x11D) | the fold | |
| 05_7 | Two times one twenty-eight is x to the eighth. Fold it, and you get sixteen plus eight plus four plus one. Twenty-nine. Not zero. | The fallen bit lands as 00011101 with place values | **overflow-fold animation** (0x1D motif) | |
| 05_8 | And every nonzero byte now has a partner. Two times one forty-two is exactly one. | 142×2 folded step by step; everything cancels except 1 | — | |
| 05_9 | Thirty polynomials could do this job, and a famous encryption standard picked a different one. Then the same two bytes multiply to different answers, so both ends must agree. | 0x57·0x83 = 0x31 (RaptorQ) vs 0xC1 (AES), in decimal 49 vs 193 | side-by-side | hold |

- **Precisely:** "GF(2⁸) ≅ GF(2)[x]/(p) for any irreducible p; all fields of
  order 256 are isomorphic; the polynomial is a choice of labels"; "no zero
  divisors + finitely many bytes ⇒ each row of the table is one-to-one ⇒ it
  contains a 1 (pigeonhole)".
- **For programmers (code panel):**

```rust
fn xtime(a: u8) -> u8 { (a << 1) ^ if a & 0x80 != 0 { 0x1D } else { 0 } }
fn mul(mut a: u8, mut b: u8) -> u8 {      // shift-and-add, no tables
    let mut acc = 0;
    while b != 0 { if b & 1 != 0 { acc ^= a } a = xtime(a); b >>= 1 }
    acc                                   // mul(0x57, 0x83) == 0x31, mul(2, 142) == 1
}
```

#### 06_the_byte_clock (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Start at one and keep doubling. One, two, four, eight, sixteen, thirty-two, sixty-four, one twenty-eight. ⟨predict⟩ What comes next? | Doubling sequence on byte-ordered points | — | |
| 06_2 | Twenty-nine. Keep going, and you visit every nonzero byte exactly once before coming back. Two hundred fifty-five steps. | In byte order the doubling path is a tangled 255-point star | **255-gon star** | |
| 06_3 | Now move every byte to the step where doubling first reached it. The star untangles into a circle. Doubling is one click around. A clock with two hundred fifty-five hours. | Bytes slide to their log positions; the star untangles | **star → circle untangling** | listen 3 s (Shepard glide) |
| 06_4 | To multiply, add clicks, just like two cubed times two to the fourth is two to the seventh. One twenty-eight times four: seven clicks plus two is nine clicks, which is fifty-eight. | 2⁷·2² = 2⁹ = 58, read off the doubling list | slide-rule rings | |
| 06_5 | One fifty is a hundred eighty clicks. Three is twenty-five. Two hundred five clicks. One sixty-seven. That click count is called a logarithm. | Two concentric rings; inner ring rotates 25 clicks | rings | |
| 06_6 | The wrong clock had two hundred fifty-six hours and added. The right one has two hundred fifty-five and multiplies. | The two clocks side by side | rhyme | |
| 06_7 | Every pair at once: here's the whole times table in byte order. Noise. Sort the rows and columns by clicks, and it turns into perfect diagonal stripes. | Table (row strips) → reorder → stripes | **table reorder (S-3)** | |
| 06_8 | With the encryption standard's polynomial, doubling gets stuck in a loop of fifty-one. It's still a field, but RaptorQ's choice lets two visit everyone. | AES star closes after 51 | star comparison | hold |

- **Ruth:** primitive vs irreducible (16 of the 30 degree-8 irreducibles are
  primitive; 0x11D is, 0x11B is not: ord 2 = 51, ord 3 = 255); log of 0 is
  undefined (the code checks zero first); the RFC ships 510-entry OCT_EXP so no
  `mod 255` is needed; the number of elements of each order (1, 2, 4, …, 128) is a power of 2,
  because 255 = 3·5·17 (the orders themselves are 1, 3, 5, 15, 17, 51, 85, 255)
  (delight, lower-third).
- **Cut:** the torus from revision 1 (vertex-coloured surfaces smear the
  diagonals; the byte clock carries the idea better).
- **For programmers (verify before narrating):** Linux RAID-6 computes
  Q = Σ 2ⁱ·Dᵢ in this exact field; under AES's polynomial 2 is not a
  generator: its powers give only 51 distinct weights (Anvin, "The
  mathematics of RAID-6").

#### 07_the_byte_lottery (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Now give every card random byte amounts. This design exists, called random linear network coding. RaptorQ does something cheaper, but first see why it's so good. | Card with byte coefficients | card | |
| 07_2 | Decoding fails only if some change to the file would slip past every card you hold. | A "change" glyph sliding past cards | — | |
| 07_3 | Take one such change. A random byte-weighted card misses it only one time in two fifty-six, because no two nonzero bytes multiply to zero. | Pie: 1/256 sliver vs XOR's 1/2 | pies | |
| 07_4 | So every extra card divides the risk by two fifty-six. With ex-or, each extra card only halves it. | Model: 0.39% / 0.0015% / 0.000006% at K, K+1, K+2 | table, labelled **model** | |
| 07_5 | Let's race them. Keep adding random cards until the system is solvable, a thousand times over. | Histograms of spares needed: XOR spread (mean 1.6), bytes almost all at zero (mean 0.004) | **Monte Carlo race** | |
| 07_6 | One subtlety. A bigger field gains nothing if the amounts are still only zeros and ones. All the gain comes from the other two fifty-four values. | Same 0/1 card in both fields: same rank | Ruth beat | hold |

- **Precisely:** "union bound: P(fail) < 256^−h/255; exact values 0.39215% (the bound, 1/255, is 0.39216%) /
  1.5319×10⁻⁵ / 5.984×10⁻⁸ agree to three digits"; "rank is unchanged by
  extending the field".

#### 08_the_price_of_density (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | So use random byte amounts everywhere? Solving a system where every card uses every piece takes about the number of pieces, cubed, over three. For fifty thousand pieces, tens of trillions of steps. | K³/3 count-up: ≈4.2×10¹³ | count-up | |
| 08_2 | Fast codes keep cards sparse. A few cards carry a single piece. About half carry two. A long tail carry many. | Graph labelled "pieces" / "packets"; degree histogram | graph prop | |
| 08_3 | Then decoding is a chain reaction, like Sudoku. A card with one unknown left gives it away. Remove that piece everywhere, and new single-unknown cards appear. | Peeling cascade on the 5-unknown toy (values, not y's): C, A, B, D, E; the ripple glows | **peeling cascade** | |
| 08_4 | Lose the card that starts it, and nothing moves. | Graph freezes | stall | |
| 08_5 | With exactly enough sparse cards, peeling usually collapses early. Even with a few percent extra, in our simulation it stalls around ninety-eight or ninety-nine percent. One run in five collapsed. | Progress bars stalling; "1 of 5 runs collapsed" | stall bars | |
| 08_6 | *(for programmers)* Reed–Solomon codes are perfect: any K of N packets work. But you must pick N up front, and with byte-sized symbols it tops out near two fifty-six. A fountain never runs dry. | RS vs fountain card; `n ≤ q + k − 1` lower-third | card | hold |

- **For programmers:** "keep, for every piece, a list of the cards that use
  it": the one data structure the peeler needs.
- **Ruth:** peeling is not elimination (it can fail where elimination
  succeeds); the RFC degree table is the ideal soliton 1/(d(d−1)) for
  d = 2…29 quantized to 2⁻²⁰, mean 4.817 (not the robust soliton); a plain
  sparse code with that table leaves e^(−4.82) ≈ 0.81% of pieces untouched at
  N = K.

#### 09_where_the_danger_lives (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Here's RaptorQ's twist. Its packets never use byte amounts. Every packet is a plain ex-or. | XOR cards | — | |
| 09_2 | Before sending, both sides agree on some bonus pieces, built from the real ones by fixed rules. They're never sent. They're free clues. Most clues are cheap ex-ors. Between ten and sixteen of them use byte amounts, even for a block of fifty-six thousand pieces. | Grid schematic: sparse XOR band, a thin gold band of byte rows, XOR packet rows; labelled counts (K′ = 10: a real 27 × 27 grid; K′ = 10,017: a schematic, not a zoom) | matrix schematic | |
| 09_3 | ⟨predict⟩ Remember where the danger lives? The cheap ex-or equations fill almost the whole space and leave a few directions open. The byte clues close them. If those clues were ex-or instead, what would you expect? | Danger meter callback | prediction card | |
| 09_4 | The ex-or lottery all over again. About seventy-one percent failure. With the byte clues, about six failures in a thousand. | Ablation counters: 70.7% vs 0.646% (blocks of 10 pieces) | **ablation counters** | |
| 09_5 | Decoding mostly peels. When it gets stuck, it parks one stubborn piece, calls it c, and carries it along like the x in algebra class. | Inactivation on the toy: C becomes a ghost c; A = 2 ⊕ c, B = 6 ⊕ 2 ⊕ c, D = 6 ⊕ 6 ⊕ 2 (c cancels), E = 14 ⊕ D | ghost node | |
| 09_6 | At the end, the leftover cards and the byte clues pin down the parked pieces. Here c is fifteen ex-or fourteen ex-or six, which is seven, and everything else follows. It's ordinary elimination in a clever order, so it works exactly when the equations are independent. | c = 7; back-substitution A=5, B=3, D=2, E=12 | live solve | |
| 09_7 | Here's what one byte amount does to the loop that ex-or couldn't solve. Make the last card E plus two A. Walk around: with ex-or, A meets itself, and A plus A is zero. The loop says nothing. With the two, A plus two A is three A. Three A is fifteen, so A is five. | Pentagon walk, one edge per beat; `3A = 15 ⇒ A = 5` ("three times five is fifteen here too, but three times seven was nine") | **pentagon walk** | hold |

- **Lower-thirds:** "RaptorQ parks 158 pieces on purpose at 10,017 pieces;
  search: inactivation decoding"; "the sender also runs one solve, to send the
  originals unchanged first".
- **Ruth:** "by design at least H directions stay open" (S + K′ binary rows for
  L = K′ + S + H unknowns); the narration says the binary rows *behave like*
  the lottery, with the measured ablation as evidence; the RFC states success
  iff rank(A) = L, independent of elimination order (§5.4.2.1). A cycle
  aᵢxᵢ + bᵢxᵢ₊₁ is singular iff ∏(bᵢ/aᵢ) = 1 (Ruth P4); 3 here is 1 ⊕ 2.

#### 10_two_packets (~75 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | We ran the real thing a million times, on small blocks of ten pieces. With exactly enough packets, it failed six thousand three hundred sixteen times. About six in a thousand. | Bar with K, trials, failures, interval | measured chart | |
| 10_2 | One extra packet: twenty-four times. Two extra: once. Too rare for us to measure. | Ticks labelled "1 in 100 / 1 in 10,000 / 1 in a million" | log chart in words | |
| 10_3 | The standard promises at most one in a hundred, one in ten thousand, and one in a million, for every block size. | RFC 6330 requirement overlaid | — | |
| 10_4 | The originals go first, unchanged, so a phone with a clean connection does no decoding at all. | Systematic: originals then repair packets | — | |
| 10_5 | It's in the standard for next-generation TV broadcasts. Its older sibling was chosen for mobile broadcast in 2005. And the same byte field protects disk arrays that survive two dead drives. | Text cards (no logos) | cards | |
| 10_6 | Two extra packets buy a million-to-one, because every nonzero byte has a partner to divide by. And that's because a polynomial refuses to factor. | The 0x11D fold glows; the bucket fills; receipts card | coda; names-to-search lower-third (fountain code, LT code, Raptor, RaptorQ, RFC 6330, inactivation decoding, GF(256)) | |

- **Ruth:** the requirement is over uniformly random received packet sets
  (ESIs), at K′ (padded K); our measurement is K = K′ = 10 only (0.631% at 101,
  0.54% at 1,002 at K); exact 95% interval for 1 failure in 10⁶ is
  2.5×10⁻⁸–5.6×10⁻⁶. Model vs reality: 0.39% vs 0.63%, because the real rows
  are mostly sparse XOR.
- **Dev:** ATSC 3.0 (A/331) cites RFC 6330; 3GPP chose Raptor (RFC 5053) for
  MBMS in 2005 (RaptorQ's adoption into TS 26.346 is unconfirmed, so the script
  says "older sibling"); RAID-6 uses 0x11D; QR codes are believed to use the
  same polynomial (285) *(verify against ISO/IEC 18004 before narrating)*.
- **Pitfalls removed from revision 1:** "random equations whose coefficients
  come from a 256-element field" (false for RaptorQ packets); "about one in a
  million, measured" (it was one failure); "the field exists only because…" (we
  can *build* it because…).

### 10.5 Facts ledger (verified in `docs/research/raptorq_gf256/scripts/` and `docs/reviews/plan_r1/`)

| id | quantity | value | source |
|---|---|---|---|
| B-F1 | sticker album, K = 10,000 | K·H_K = 97,876.06 on average (9.79×) | `misc_facts.py`, integration_check |
| B-F2 | misses per packet at 1% loss, 10⁶ phones | ≈10,000 | arithmetic |
| B-F3 | XOR example | A=5, B=3, C=7, D=2 → 2, 6, 6, 3; rank 4; B⊕C = 4 redundant | `toy_examples.py` |
| B-F4 | GF(2) success | 0.5, 0.375, 0.328 (K=3: 7/8·6/8·4/8) → 0.288788 | `rank_probs.py` |
| B-F5 | GF(2) failure at K, +1, +2, +3 | 71.12%, 42.24%, 22.99%, 11.99% | `rank_probs.py` |
| B-F6 | average spares | 1.6067 (GF(2)); 0.00394 (GF(256)) | `rank_probs.py` |
| B-F7 | GF(256) success (model) | ≥ 0.996078 for every K | `rank_probs.py` |
| B-F8 | GF(256) failure (model) at K, +1, +2 | 3.92×10⁻³; 1.53×10⁻⁵; 5.98×10⁻⁸ | `rank_probs.py` |
| B-F9 | pentagon | rank 4 over GF(2); with E + 2A rank 5; 3A = 15 ⇒ A = 5 | `toy_examples.py` |
| B-F10 | Z/256 facts | 2·128 ≡ 0; 2·131 ≡ 6; 127 nonzero non-units; row a has 256/gcd(a,256) values | `gf256_facts.py`, integration_check |
| B-F11 | GF(256) products | 3·7 = 9; 2·128 = 29; 128·3 = 157; 128·4 = 58; 150·3 = 167; 7·7 = 21; 0x57·0x83 = 0x31 (AES 0xC1); 2⁻¹ = 142; 3⁻¹ = 244 | `gf256_facts.py` (all 65,536 pairs) |
| B-F12 | polynomial counts | 30 irreducible, 16 primitive; 0x11D primitive; 0x11B not (ord 2 = 51, ord 3 = 255) | `gf256_facts.py` |
| B-F13 | logs | log 3 = 25, log 29 = 8, log 128 = 7, log 150 = 180; 180 + 25 = 205 → 167 | `gf256_facts.py` |
| B-F14 | RFC tables | OCT_EXP 1, 2, 4, …, 128, 29, 58, 116, 232, 205; = powers of x mod 0x11D | `gf256_facts.py --rfc` |
| B-F15 | dense elimination | ≈K³/3 = 4.2×10¹³ for K = 50,000 | arithmetic |
| B-F16 | RFC degree table | P(2) = 1/2, P(1) ≈ 0.5%, ideal soliton for d = 2…29, mean 4.817; ≈7.15 symbols per repair | `misc_facts.py` |
| B-F17 | untouched (plain sparse code, N = K) | e^(−4.82) ≈ 0.81% | arithmetic |
| B-F18 | peeling | N = K collapses (8–42% recovered); 2–5% extra → ≈98–99%; 1 of 5 runs collapsed at 2% | `lt_peel_sim.py` |
| B-F19 | RFC parameters | H ∈ [10, 16] over 477 K′; K′ = 10,017: S 241, H 11, W 10,111, L 10,269, P 158; K′_max 56,403 | `misc_facts.py` |
| B-F20 | inactivation toy | peel C, A, B, D, E; c = 15 ⊕ 14 ⊕ 6 = 7 | `toy_examples.py` |
| B-F21 | ablation (same harness) | RFC rows 646 / 10⁵ = 0.646% (K′=10); GF(2) rows 7,066 / 10⁴ = 70.7% (K′=10), 702 / 10³ = 70.2% (K′=101) | `rq_ablate.py` (recorded) |
| B-F22 | measured RaptorQ (K = 10, 10⁶ trials, cberner decoder) | 6,316 / 24 / 1 failures at K, K+1, K+2 | `rq_mc.py` (recorded) |
| B-F23 | RFC 6330 §5.8 | ≤ 1/100, ≤ 1/10⁴, ≤ 1/10⁶ at K′, K′+1, K′+2 (average over random ESIs) | RFC text |
| B-F24 | carousel, 1% loss, K = 10,000 | mean 2.64 loops; all 10⁶ phones done: 0.37 by 5 loops, 0.99 by 6 | integration_check (exact formula) |
| B-F25 | Z/256 lottery | invertible ⇔ det odd ⇔ invertible mod 2 ⇒ success = ∏(1−2⁻ⁱ); MC K=8: 0.285 vs 0.290 | theorem + integration_check |
| B-F26 | rotation fold | 3·255 = 0 mod x⁸+1 | algebra |
| B-F27 | union bound | 256^−h/255 matches exact to 3 digits | creative_scripts |
| B-F28 | GF(2) spares for 10⁻⁶ | ≈20 (failure ≈ 2^−h; h = 20: 9.5×10⁻⁷) | integration_check |

### 10.6 Pronunciations to probe

RaptorQ ("raptor queue"), Galois, Reed–Solomon, Shokrollahi, Luby, Erdős,
Borwein, ex-or, ATSC, RAID, "hex one D", polynomial, logarithm, Gaussian.

### 10.7 Engineering notes (from the engine review)

- **Big tables (S-3):** one multi-subpath VMobject per row strip with per-point
  `fill_rgba` (256 mobjects, not 65,536); reorder rows by moving strips with a
  dt updater, swap to column strips at a hold, reorder columns. Never transform
  a group of more than ~2k members per frame (positional ops dedupe in O(M²)).
- **Matrix zoom:** cut. A real 27 × 27 matrix for K′ = 10 plus a schematic of
  K′ = 10,017 with labelled counts (a vector zoom to 10,269 columns would be
  10⁸ cells, far past the retained plan's 2²⁰-curve cap).
- **3D cube (S-5):** camera segment through the kit's camera wrapper; span
  planes translucent and drawn last.
- **α glyph:** probe `\mathsf{\alpha}` and `\text{α}` inside one Tex (S-7);
  ⊕ renders from Noto Sans Math (heavier, acceptable).

---

## §11 Video C: The Algorithm Lamport Is Proudest Of

*Slug `bakery_algorithm`. Research: `docs/research/bakery_algorithm/MEMO.md`
(exhaustive interleaving model checker with atomic and safe-register
semantics, broken variants, FCFS ghost check, liveness, an x86-TSO model; every
Lamport quote verbatim from his papers, publications page, Turing lecture or the
Computer History Museum oral history). Revision 2 integrates review round 1:
the slate-order error, Knuth's rule, the lemma conditions, fence precision, and
a new hero (space-time lanes plus the progress square). The owner's source:
"The Bakery Algorithm: Concurrency without Atomicity"
(jeffreyemanuel.com/writing/bakery_algorithm).*

### 11.1 Brief

| Field | Value |
|---|---|
| Owner's words | "The baker algorithm (see this explainer by me)" |
| Working title | **The Algorithm Lamport Is Proudest Of** (alternative: *Taking Turns When Memory Lies*); "proudest" is supported by the CHM oral history ("perhaps that's why I'm proudest of it") |
| One-sentence takeaway | Customers can take turns safely using only slates each of them writes alone: take a ticket bigger than every ticket you see, break ties by name, and raise a "choosing" flag so nobody is judged before their ticket is written; and the proof never trusts a read that overlaps a write, so it survives memory that returns garbage. |
| The hook | A slate reads 9; someone is rewriting it as 10; glance mid-stroke and you might read 9, or 10, or, Lamport says, 2496. Could computers take turns safely with memory like that? |
| The hero pictures | (1) **Space-time lanes**, one per customer, time running left to right: reads are arrows in, writes arrows out, and the lanes double as a musical score. (2) **The progress square:** every order of two programs is a staircase; the bad orders pass through a red square. The lexicographic grid supports chapter 04. |
| Worked examples | the 6 orders of two increments (4 lose a point); the 14-order sign (8 bad); the 8-step unwritten-ticket collision and the same order with the flag; "both see 5, both take 6"; slates 0, 3, 0, 0, 2 → customer 4 takes 4; a 4-customer run; garbage-read traces e1 and e3; the 12-step store-buffer failure |
| Words | "customer" (lower-third "process"), "the room" (named "critical section" once), "order" (lower-third: counting each read, write and successful wait as one step), "slate" (register), "flag" (choosing), "ticket" |
| Length | ~13 min, 10 chapters (D-05) |
| Out of scope | Paxos itself, bounded-bakery constructions, TLA+ |
| Oracle | `oracle.py` imports `docs/research/bakery_algorithm/scripts/`; long sweeps are recorded outputs |

**Things the script must get right** (memo + round 1):
1. "The first mutual-exclusion algorithm that assumes no lower-level mutual
   exclusion, not even atomic reads and writes"; never "the first correct
   solution" (Dekker, Dijkstra 1965, Knuth 1966, de Bruijn, Eisenberg–McGuire
   came earlier).
2. Lamport did not set out to tolerate garbage: he noticed after writing the
   proof that it never used a value read during a write.
3. Equal tickets are normal; unique is the pair (ticket, id); ids break ties
   only.
4. `choosing` is not a lock and does not stop torn reads.
5. FCFS counts from finishing the doorway; overlapping doorways carry no order.
6. Tickets are unbounded; the 2³⁵ bound assumes reads never return values
   larger than were written.
7. Real CPUs: on x86 two fences per customer suffice (model-checked, small
   cases); Arm is looser; sequentially consistent atomics are the safe default;
   Acquire/Release is **not** enough.
8. The max must read each slate once (double-fetch breaks with 3+ customers when
   reads can return garbage).
9. The owner's TypeScript `(t[j], j) < (t[i], i)` is JavaScript's comma operator.
10. Quotes verbatim only (1974: "The read may return any arbitrary value!";
    Turing lecture: "correct even if reading a number while it is changing from
    9 to 10 obtains the value 2496"; publications page: the 7456 passage).

### 11.2 The arc

| Act | Chapters | Spine |
|---|---|---|
| I. The problem | 01 one plus one is one, 02 the room | a step is a read then a write; check-then-set fails; Dijkstra's problem and Knuth's addition; the harder question |
| II. Building the bakery | 03 take a number, 04 same number, 05 the unwritten ticket, 06 one run | slates and tickets; ties → (ticket, name); unwritten tickets → the flag |
| III. Why it works | 07 the proof, 08 garbage | three places a customer can be; the proof never trusts a read that overlaps a write |
| IV. Costs and legacy | 09 fair at a price, 10 your CPU | FCFS, unbounded tickets, costs; outboxes and fences; clocks and Paxos |

### 11.3 Color roles

| Idea | Color |
|---|---|
| customers 1–5 | five fixed hues (blue, green, orange, purple, pink) for the whole video |
| the room (critical section) | warm red |
| the flag (choosing) raised | amber `#F0AC5F` |
| tickets / slates | chalk white |
| the order ≺ | gold `#FFD866` (hero) scanline |
| garbage reads (overlapping a write) | flickering magenta |
| store buffers ("outboxes") / hardware | cool grey |

### 11.4 Chapters and beats

#### 01_one_plus_one_is_one (~75 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | A slate reads nine. Someone is rewriting it as ten. Glance at it mid-stroke, and you might read nine, or ten, or, Lamport says, two thousand four hundred ninety-six. | A chalk slate frozen mid-stroke; flickering digits; quote card (Turing lecture) | flicker prop | |
| 01_2 | Could computers take turns safely with memory like that? In nineteen seventy-three, that was considered impossible. | Two lanes reach for the same slate; a dated card "1973: considered impossible" (Lamport's Turing lecture, verbatim) | quote card | |
| 01_3 | *(title)* | **The Algorithm Lamport Is Proudest Of** | title card | hold |
| 01_4 | Two programs, running at once, share a scoreboard that reads forty-one. Programmers write x equals x plus one. It isn't an equation. It means make x one bigger. | Scoreboard; two lanes (blue, green); "time runs left to right; a read is an arrow in, a write is an arrow out" | **space-time lanes** | |
| 01_5 | That's two steps. Read the score, then write it back plus one. Here's one unlucky order. Both read forty-one. Both write forty-two. Two points scored, one counted. | Arrows on the lanes; x ends at 42 | step animation | |
| 01_6 | Here's every order at once. Program one's progress runs across, program two's runs up. Every order is a staircase from corner to corner. Six of them. ⟨predict⟩ How many lose a point? | **Progress square**: 2×2 grid, 6 staircases | progress-square prop | |
| 01_7 | Four. Exactly the ones that pass through the middle, where both have read and neither has written. A lock's job is to wall off the middle. | The 4 paths through (1,1) turn red | — | |
| 01_8 | In a real machine the bad orders are rare. But at billions of steps a second, rare means constantly, and the computer can pause either program between any two steps. | The six orders shrink into one long strip of steps racing past; red flecks (bad orders) appear along it again and again | time strip | hold |

- **For programmers:** even `add [x], 1` reads and writes separately unless it
  carries a lock prefix.
- **Ruth:** six orders = C(4,2) interleavings of two 2-step programs (a count of
  orders, not a probability).

#### 02_the_room (~80 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | The fix seems obvious. Only one program at a time may touch the scoreboard. Call that stretch of code the room. Computer scientists say critical section, and the rule is called mutual exclusion. | A room around the scoreboard | room prop | |
| 02_2 | Put a sign on the door. Check that it says vacant, flip it to occupied, go in. | Sign | — | |
| 02_3 | Same problem. Both read vacant before either flips it. Of the fourteen orders, eight put both in the room at once. | Progress square with a red square "both inside"; 14 paths, 8 cross it | progress square | |
| 02_4 | Edsger Dijkstra posed the problem properly in nineteen sixty-five. Never two in the room. Someone who wanders off outside can't block everyone. And fast or slow, it still works. A year later Donald Knuth added one more. Everyone who wants in gets in eventually. | Rule cards | statement cards | |
| 02_5 | Hardware usually cheats with a magic sign you can check and flip in one motion. Dijkstra showed an ordinary sign is enough, if nobody can ever catch it halfway through a flip. | Magic sign vs ordinary sign | — | |
| 02_6 | Here's the harder question. What if a glance can catch the sign mid-flip, as a blur? Hold that thought. | Sign blurs | flicker | hold |

- **Lower-third:** "counting each read, write and successful wait as one step"
  (the 14/8 counts depend on it).
- **Ruth:** the four rules as Lamport states Dijkstra's problem "as extended by
  Knuth"; "atomic" is defined once (one indivisible step) and never used
  loosely.

#### 03_take_a_number (~70 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Lamport's idea came from a bakery. Take a number, and the lowest number is served next. | Bakery counter | scene | |
| 03_2 | But a ticket machine is shared. It needs a room of its own. | Dispenser with a drawn red ✗ (`exmark`) | — | |
| 03_3 | So each customer gets their own slate, and only they write on it. A zero means not in line. | Five slates, five colors | slates prop | |
| 03_4 | To take a number, read every other slate, one at a time, and write one more than the biggest. ⟨predict⟩ Customer four reads zero, three, zero, two. What does she write? | Read arrows; slates 0, 3, 0, 0, 2 | prediction card | |
| 03_5 | Four. Then wait until nobody with a smaller number is still in line. Here that's customer five, then customer two, then customer four. | Slates now 0, 3, 0, 4, 2 (pointed at, not read aloud); service order 5 → 2 → 4 | queue | |
| 03_6 | No shared counter. Nobody ever writes on anyone else's slate. | Each lane's own slate glows in its customer's colour; every arrow from another lane is a read, none a write | lanes, slates | hold |

- **For programmers:** a ticket lock is the machine Lamport avoided: one atomic
  fetch-and-add (Linux x86 FIFO ticket spinlocks, 2008).

#### 04_same_number (~75 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | ⟨predict⟩ Can two customers get the same number? | — | prediction card | |
| 04_2 | Yes. Both look at the slates at the same moment, both see five as the biggest, both write six. | Both read 5, both write 6 | step animation | |
| 04_3 | If equal numbers both go in, both end up in the room. If equal numbers both wait, they bow to each other forever. After you. No, after you. | Two traces: both in the room; deadlock as two customers bowing | traces | |
| 04_4 | The fix is a tie-breaker. Compare tickets, and if they're equal, compare names. Like a dictionary. | (6, 1) before (6, 2) | — | |
| 04_5 | Put every pair on a grid, ticket across and name up, and sweep column by column. That's the serving order. A late customer with the smallest name still waits for every smaller ticket. The name only breaks ties. | Lexicographic grid; gold scanline; (3,1) swept after (2,4) | **lexicographic grid sweep** | hold |

- **Ruth:** ≺ is a strict total order on ℕ × {1…N} (ids distinct); every pair
  has finitely many predecessors.

#### 05_the_unwritten_ticket (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | The second crack is sneakier. Customer one reads customer two's slate, sees zero, and decides on ticket one. But she hasn't written it yet. | Ghost "1" hovering over slate 1 | step-through | |
| 05_2 | Customer two takes ticket one, checks customer one's slate, sees zero, which means not in line, and walks in. | Customer 2 enters | — | |
| 05_3 | Then customer one writes her one, sees a tie, wins it on her name, and walks in too. Both in the room. | Both inside; two motifs clash | 8-step trace (a) | listen 3 s (two motifs at once = the bug) |
| 05_4 | The fix is a flag that says, I'm choosing, don't judge me yet. Raise it before reading, lower it after writing your ticket. Before comparing with someone, wait until their flag is down. | Amber flags | flag prop | |
| 05_5 | Same unlucky order, now with flags. Customer two sees customer one's flag up, and waits. | Scheduler slider replays the order with flags (15 steps) | toggle replay | |
| 05_6 | With two customers and one visit each, the version without flags has twenty-eight possible orders, and four of them put both in the room. With flags, three hundred twenty-two orders, and not one does. | Progress-square paths: 28 (4 through the red square) morph to 322 (none) | **path-set morph (S-6)** | |
| 05_7 | The stretch between raising and lowering the flag is called the doorway. You can't close the gap between looking and writing. Lamport didn't try. He made the gap visible with the flag, and made every race harmless with the tie-breaker. | Doorway shading on the lanes; thesis card | statement card | hold |

- **Engine note:** blocked moves depend on history, so draw paths, not static
  walls. Optional 3D set piece (camera route, spike): for three customers the
  square becomes a cube; sampled staircases thread it as a counter reaches
  2,126,816,238 orders, none entering the "two inside" beams.

#### 06_one_run (~55 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Here's one full run, fast. | Scene big, live slate table small, code panel dimmed ("for programmers") | scene + table + code | |
| 06_2 | Customers three and two arrive together. Customer three stops, because customer two's flag is still up. The flag saves the day. | Fast-forward with a step counter; slow at the beat | step clock (tracker) | |
| 06_3 | Both take ticket one. ⟨predict⟩ Customer four sees one and one. What does she take? Two. And the tie goes to customer two, by name. | — | prediction card | |
| 06_4 | Customer one arrives late with the smallest name and takes three. She waits behind ticket two. Customer two leaves, gets back in line, and takes four. The serving order is exactly the grid's. | (1,2) → (1,3) → (2,4) → (3,1) → (4,2) with the grid sweep | callback | hold |

- **For programmers (code panel aside here, not in 04):** the comma-operator
  trap; ESLint's `no-sequences` catches it, TypeScript doesn't; tuple compare in
  Python and Rust is correct.

#### 07_why_never_two (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Why can two customers never be in the room at once? Picture Blue inside, and Green holding a ticket, waiting or inside. | Lanes; Blue and Green | lanes | |
| 07_2 | Fact one. If you finished choosing before I started, and you're still holding your ticket when I look, my ticket is bigger. I saw yours. | Lemma 1 card | Tex card | |
| 07_3 | Fact two. If you're in the room, everyone else holding a ticket has a bigger pair than yours. | Lemma 2 card | — | |
| 07_4 | ⟨predict⟩ When Blue checked Green's flag, where could Green have been? Three places. | Green's doorway interval slides along the time axis against Blue's flag read | **sliding interval with zones** | |
| 07_5 | Not yet choosing: then by fact one, Green's ticket will be bigger. In the middle of choosing: Green's flag was up, and Blue would have waited. Finished choosing: then Blue read Green's real ticket, and went in only because Blue's pair was smaller. | Zones (a), forbidden middle, (b) | zones | |
| 07_6 | Now suppose both were in the room. Fact two says Blue's pair is smaller than Green's, and Green's is smaller than Blue's. Like being taller than you, and you taller than me. Impossible. | (nᵢ, i) ≺ (nₖ, k) ≺ (nᵢ, i) | TransformMatchingTex | |
| 07_7 | A computer search over every order, up to four customers with small tickets, agrees. But a search only covers small cases. The proof covers all of them. | "N = 4, tickets ≤ 4: 3,027,633 states, no violation" then "every N" | Ruth beat | hold |

- **Precisely:** Lemma 1 and Lemma 2 exactly as in the memo §4, with "in the
  bakery" = holding a nonzero ticket; Assumption M (the max reads each slate
  once).
- **Music:** swing drops out for 07–08 (rubato, a pedal under the lemmas).

#### 08_reading_garbage_safely (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Remember the slate caught mid-stroke? Rewrite oh-nine as ten one digit at a time, and a glance in between reads nineteen, or zero zero. Lamport went further. Assume a glance during a write can return anything at all. | Two-digit rewrite; then wild values | flicker | |
| 08_2 | He called memory like that safe. A funny name, since it's the least safe kind. | "safe" card; atomic and regular on screen for Ruth | register zoo (screen) | |
| 08_3 | Go back to the proof picture, and let every slate flicker while it's written. Every glance that crosses a write turns magenta. | 07's zones; magenta reads | **overlay on the proof zones** | |
| 08_4 | Every magenta glance lands in the first case, where the proof never used the value. In the last case, the ticket was finished and nobody was touching it. Every read the proof trusts happens while nothing is being written. | Zone (a) shaded "value unused" | zones | |
| 08_5 | Here a customer glances mid-write, gets seven, garbage, and takes eight. That only makes her later. Here a read in the middle of a write still sees the old zero, and lets a customer straight in. But the other customer is writing a brand-new ticket, so she started choosing after the first one finished. Fact one: her ticket will be bigger, so she waits. | Traces e1 and e3 | traces | |
| 08_6 | Lamport noticed this only after writing the proof. In his words, the read may return any arbitrary value. A colleague, Anatol Holt, refused to believe it. | Verbatim quote card (1974) | quote card | |
| 08_7 | One caution, from Lamport himself. Read each slate exactly once when finding the biggest. Read one twice, and with three or more customers and garbage reads, it breaks. | Double-fetch trace (33 steps) | trace | hold |

- **Ruth:** safe / regular / atomic per "On Interprocess Communication" (1986);
  the safe-register model check (N ≤ 3; 269,550 states at tickets ≤ 6, zero
  violations) is evidence, the proof is the reason; Lamport's own 1974
  single-bit footnote was a fallacy he later acknowledged; the algorithm stands.
- **Dev:** on a real machine garbage can be enormous (a 64-bit ticket written in
  two halves on a 32-bit chip), which is why the overflow bound in 09 assumes
  reads never come back too big.

#### 09_fair_at_a_price (~75 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | It's also fair. Finish taking your ticket before I start taking mine, and you're served first. Counted from when you finish choosing, not from when you walked up. If our doorways overlap, either of us may go first. | FCFS on the lanes | — | |
| 09_2 | So nobody waits forever, as long as everyone in the room eventually leaves and nobody dies in the doorway. Each other customer can get ahead of you at most once. | Bounded bypass | — | |
| 09_3 | The price: tickets keep growing as long as somebody is always in line. They reset only when the bakery empties. | Odometer 1, 2, 3, 4, 5, 6 | odometer | |
| 09_4 | Lamport did the sum. At one customer a millisecond, after a year the tickets are still below about thirty-four billion, two to the thirty-five. That fits easily in an ordinary computer number. | 31,557,600,000 < 34,359,738,368 | arithmetic card | |
| 09_5 | *(for programmers)* And it isn't cheap. | Costs card: N fixed in advance; 3(N−1) slate reads per entry even alone; spinning; a customer paused mid-doorway stalls everyone; one that dies with its flag up deadlocks everyone; slates sharing a cache line ping-pong (pad each); 2N shared variables, half unbounded, while any read/write lock needs at least N (Burns–Lynch) | card | hold |

- **Precisely:** "if i finishes its doorway before k starts its doorway, i
  enters first"; the 2³⁵ bound assumes "a read of number[i] can never obtain a
  value larger than one which has been written there" (Lamport 1974); 64-bit
  tickets at 10⁹/s last ≈584.5 years.

#### 10_your_cpu (~85 s)

| id | draft words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | The proof is right. But run this on your laptop as written, and it can fail, because your laptop breaks one of the proof's promises. | The proof's promises from 07 as a short list; one ("a write is seen in order") flickers and breaks over a CPU outline | promise list | |
| 10_2 | Each core puts its writes in an outbox and keeps going. Both customers raise their flags and write their tickets, into their outboxes. They read the other slates, see no flags, and walk in. Main memory still says nobody is even choosing. | Store-buffer "outboxes"; memory bar frozen at zeros; both enter (12 steps) | **outbox x-ray** | listen 3 s (two motifs) |
| 10_3 | That's not the cache. Caches agree with each other. It's the queue in front of them. The fix is a fence: wait until your outbox is delivered before you read anyone else's slate. On an Intel or AMD chip, two fences per customer are enough. Arm chips, like the ones in recent Macs, reorder more and need more. | Fence drains a tray with a whoosh | — | |
| 10_4 | *(for programmers)* The easy, correct way is sequentially consistent atomics. Acquire and release are not enough here, because this needs your write to land before your next read. | SeqCst Rust panel (below); card: C/C++/Rust plain shared variables are undefined behaviour, `volatile` doesn't help; Java `AtomicIntegerArray` works, `volatile int[]` doesn't | code panel | |
| 10_5 | In practice people use the hardware's atomic instructions. But the bakery lock still ships in Arm's trusted firmware: on chips without hardware cache coherence, a core waking up with its data cache still off can't use the usual spinlocks. | Text card: Arm Trusted Firmware, "bakery lock: a core waking with its cache off" (no logos) | card | |
| 10_6 | And its trick of a counter, with names to break ties, reappears in how Lamport later ordered events across whole networks, and in the ballot numbers of Paxos, the protocol that keeps replicated databases agreeing. | The 04 grid relabelled (clock value, process); timeline 1974 → 1978 → Paxos | callback | |
| 10_7 | Lamport says he didn't invent the bakery algorithm. He discovered it. And it just seemed to come out of thin air. Perhaps, he says, that's why he's proudest of it. | Verbatim quote cards | coda; receipts card | |

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::SeqCst};
pub fn lock(i: usize, choosing: &[AtomicBool], number: &[AtomicU64]) {
    choosing[i].store(true, SeqCst);
    let t = 1 + number.iter().map(|n| n.load(SeqCst)).max().unwrap(); // each slate read once
    number[i].store(t, SeqCst);
    choosing[i].store(false, SeqCst);
    for j in (0..number.len()).filter(|&j| j != i) {
        while choosing[j].load(SeqCst) { std::hint::spin_loop() }
        loop {
            let nj = number[j].load(SeqCst);
            if nj == 0 || (nj, j) > (t, i) { break }
            std::hint::spin_loop()
        }
    }
}
pub fn unlock(i: usize, number: &[AtomicU64]) { number[i].store(0, SeqCst) }
```

- **Honest data:** "it can fail" is backed by the TSO model (12-step trace).
  A real-hardware experiment (Relaxed atomics on this Mac and on an x86 host,
  violations per million entries) is a bead; the narration claims a measured
  failure only if that experiment produces one.
- **Ruth:** "two fences per customer on x86" is the TSO model result for N ≤ 3
  (bounded), consistent with Abdulla et al. (TACAS 2012); "reappears" rather
  than "became" for the clock lineage (CHM p. 9 supports the counter link).
- **Quotes (verbatim):** "I feel that I did not invent the bakery algorithm, I
  discovered it." (publications page); "the bakery algorithm, it just seemed to
  come out of thin air to me … perhaps that's why I'm proudest of it" (CHM oral
  history: the question on p. 9, the answer on p. 10).

### 11.5 Facts ledger (verified in `docs/research/bakery_algorithm/scripts/` and `docs/reviews/plan_r1/`)

| id | quantity | value | source |
|---|---|---|---|
| C-F1 | lost-update orders | 6, of which 4 pass through the middle and lose a point | `tree_count.py`; `creative_scripts/progress_paths.py` |
| C-F2 | naive sign | 14 orders, 8 both inside (one step per read, write, successful wait) | `tree_count.py` + brute force |
| C-F3 | two customers, one visit | no flag: 28 / 4 bad; with flag: 322 / 0 | `tree_count.py`, `crosscheck.py` |
| C-F4 | three customers | no flag 792,987 / 312,492 bad; correct 2,126,816,238 / 0 | `tree_count.py` |
| C-F5 | broken variants (two customers) | `<` with flag 898 / 432 bad; `≤` 250 / 0 two-inside, 72 deadlocks | `tree_count.py` |
| C-F6 | traces | a (8 steps), a′ (15), b1 (12), b2 (deadlock), c (tie), d (70), e1, e3, leapfrog 1…6, double-fetch (33) | `scenarios.py`, `leapfrog.py`, `df_trace.py` |
| C-F7 | slates example | 0, 3, 0, 0, 2 → customer 4 writes 4 → 0, 3, 0, 4, 2; served 5, 2, 4 | memo §5 |
| C-F8 | state counts | correct N = 4, tickets ≤ 4, atomic: 3,027,633 OK; N = 3, tickets ≤ 6, safe: 269,550 OK | `run_sweep.py` (recorded) |
| C-F9 | TSO | no fence fails in 12 steps; F1 only 18; F2 only 20; F1 + F2 OK (1,135 states) | `bakery_tso.py`, `tso3.py` |
| C-F10 | ticket bound | 31,557,600,000 ms/yr < 2³⁵ = 34,359,738,368 | arithmetic |
| C-F11 | 64-bit tickets at 10⁹/s | ≈584.5 years | arithmetic |
| C-F12 | reads per uncontended entry | 3(N−1) (max, flag waits, ticket waits) | counted from the code |
| C-F13 | JS comma operator | `(t[j], j) < (t[i], i)` evaluates `j < i`; tsc 5.9.3 silent | `comma_operator_check.js` |
| C-F14 | dates and attributions | Dijkstra 1965; Knuth 1966 (eventual entry); Lamport CACM 17(8):453–455, Aug 1974; Interprocess Communication 1986; Nonatomic Operations POPL 1983; Time, Clocks 1978; Turing lecture 2015 ("considered impossible" in 1973; the 2496 sentence) | sources |

### 11.6 Pronunciations to probe

Lamport, Dijkstra (probe hard: the skill found it only partially verified),
Dekker, Knuth ("kuh-NOOTH", with the K sounded), Peterson, Herlihy, Taubenfeld,
Paxos, Anatol Holt, ARM, x86 ("ex eighty-six"), mutex.

### 11.7 Engineering notes

- **One model, many views:** `model.rs` is a Rust port of the checker's
  semantics that replays the verified traces step by step; a test asserts every
  replayed state equals the Python checker's recorded output. Lanes, slates,
  the progress square and the code panel all read the same step clock.
- **Path sets (S-6):** 322 staircases as polylines (about 1k mobjects); group
  zoom is tolerable at this size.
- **Glyphs:** ≺ from Noto Sans Math (heavier); ✗ via `exmark`; code panels in
  CM Typewriter Text; TexText `--` is now two hyphens, so write U+2013.
- **Cast:** friendly geometric characters in the five colors; never caricatures.

## §12 Hoeffding's D v6: migration and re-score

The Hoeffding video is the proving ground for everything new: the workspace,
the shared kit, the score engine and the PCM master. It goes first because its
narration is finished and approved (v5), so any difference the owner hears is
the new score and master, nothing else.

**Steps** (each a bead in W3):

1. **Keep the faithful import building** (done 2026-10-09: `fmn` pinned by git
   revision to franken_manim `e2eec632`, 25/25 tests, no sibling checkout
   needed). It stays excluded from the root workspace until step 2 lands, so
   `explainer_corpus/hoeffding_d` reproduces v5's code exactly.
2. **Migrate onto the workspace.** `fmn.workspace = true`; replace
   `kit.rs`, `narration.rs`, `encode.rs` and the CLI in `main.rs` with
   `explainer_kit`; keep `stats.rs` as the explainer's `model.rs`; move the 61
   lines of `narration::SCRIPT` into `script.tsv` (keeping the original ids `h1`
   … `s9` so the v5 WAVs still match, with an explicit chapter map because those
   ids do not follow the `NN_k` convention); keep every existing test and add
   the kit's registry/script mapping test.
3. **Prove the visuals did not change unintentionally.** Render 540p drafts of
   all 8 chapters with the v5 narration and compare contact sheets against v5
   stills. Expected, intended differences: the franken_markdown pin's TeX
   layout (formula chapters), and the fix for the layout lint's known finding
   (the cut-off "height" label in `04_ranks`). Anything else is a regression.
4. **Write its musical identity** (§7.5: D major moving through Dmaj9 → Bm9 →
   Gmaj9(♯11) → Asus4; felt piano, Juno pad, soft vibraphone; motif 1̂–5̂–6̂–3̂
   resolving at "dependent"; the shuffle test's 2,000 permutations as a granular
   texture) and add authored `music_breath` holds at its act boundaries (after
   `h7`, `g10`, `q8`, `c8`, `f8`, `s9`).
5. **Score it and pass score QA** (G7s) with the v5 narration timeline.
6. **Render 4K finals** on the GPU host (NVENC CQ 16) and build the **PCM
   master** (D-20); verify (G8).
7. **A/B for the owner:** three 30-second excerpts (the hook, the gallery ring,
   the shuffle test) as v5 vs v6, plus the full v6. Record the owner's verdict
   in PROGRESS.md. The score engine's defaults are not frozen until the owner
   approves v6.
8. **Publish** as release `hoeffding_d-v6` (G10), with the v5 → v6 differences
   in the release notes.

v5's WAVs live in the original repository's untracked `narration_v5/` (and on
the GPU host). Copy them into `explainer_corpus/hoeffding_d/narration_v5/`
(WAVs are gitignored; the text records already there are tracked) and record
their md5s in PROGRESS.md.

---

## §13 Publishing and distribution

Publishing is outward-facing, so every step in this section runs only after
the owner approves (G10).

**Per-video release bundle** (`./x <slug> publish` assembles it; a GitHub
Release `<slug>-vN` carries it):
- `<slug>_4k_vN.mp4`: the verified master (3840×2160, 60 fps, H.264, AAC 256k,
  −16 LUFS by fixed gain, BT.709 tags);
- `<slug>_1080p_vN.mp4`: a proxy scaled from the master (x264 CRF 18, audio
  copied);
- `<slug>.en.srt` and `.vtt`: captions from the cue logs and `spoken.tsv`,
  sentence-level timing, ≤ 42 characters per line, ≤ 2 lines (also committed
  under `publish/`);
- `contact_sheet.png`, `SHA256SUMS`, release notes from the README's "how it
  was made" table (franken_manim commit, ftts/fw versions, host class,
  encoder, frames, render wall time, loudness, true peak).

**YouTube package** (committed as `publish/youtube.md`; uploaded by the owner or
on the owner's say-so):
- title and a description with: the one-sentence takeaway, chapter timestamps
  (from the cue logs: each chapter's first frame), sources and further reading
  (the research memo's source list, trimmed), the exact versions of every
  simplification (§5.1 rule 11), a link to the repo and the research folder;
- tags; captions; a thumbnail rendered by the explainer's `thumbnail` scene
  (1280×720, the video's central picture plus a short title in IBM Plex).

**Repository front page:** a README hero illustration (`.webp`, as franken_manim
has) and a GitHub social preview image (1280×640), both made from the corpus's
own stills (W8). Description and topics are set at repo creation.

**Accessibility:** captions on every video; color roles checked for
color-vision deficiency (gold vs blue is safe; red vs green never carries an
idea alone: position, shape or a label always backs it up); text sizes ≥ 22 at
4K.

---

## §14 Upstream asks for franken_manim

Rule (from the skills): never change franken_manim as part of making a video;
work around the gap in this repository and record it. Each ask below gets a
minimal reproduction scene under `crates/explainer_kit/examples/upstream/`, an
evidence file (frames, logs) and the workaround used. **Filing beads in
franken_manim's tracker is outward-facing work in another repository and needs
the owner's go-ahead (Q9).**

| id | Gap | Existing bead | Our workaround |
|---|---|---|---|
| U-1 | `TexturedSurface` silently drops its image (`From<TexturedSurface> for Mobject` sets `image: None`) | none (new finding) | set `Mobject.image` by hand, or avoid textures (the torus uses `color_by_uv_function`) |
| U-2 | the chapter soundtrack is quantized to s16 **without dither** before ffmpeg | none | PCM master from sources (D-20) |
| U-3 | no AAC bitrate knob in the facade (ffmpeg's default ≈129 kb/s) | fm-video-encode-quality-knob-fm85 (stale) | runner wrapper `-b:a 256k`; PCM master |
| U-4 | MP4 color tags probe as unknown on ffmpeg 8.1.2 | none | lossless `h264_metadata` re-tag in the master |
| U-5 | no 2D pan/zoom on the retained route | none | group scaling; camera route |
| U-6 | no bloom/blur/radial gradients | none | `v_highlight` glow; concentric translucent discs |
| U-7 | θ/ρ/ε variants; α looks Latin | fm-adr0028-cm-math-faces-decision-2xyy, fm-cm-math-faces-impl-n12u, fm-5wq.60 | notation choices (§6.5) |
| U-8 | `\binom` delimiters; heavy big delimiters | fm-binom-delimiters-glyph-gaps-ed1k, fm-u5sb | `C(·,·)` |
| U-9 | many-particle performance unmeasured | none | S-2 evidence becomes the bead's data |
| U-10 | camera route has no tile cache; no 4K timing; Metal refused by the facade | fm-metal-route-native-render-pank, fm-sq8.6 | segments; S-5 evidence |
| U-11 | the arena never frees removed mobjects (`always_redraw`-heavy chapters slow over time) | fm-92gu (in progress) | clear updaters; prefer dt updaters for particles |
| U-12 | NumberPlane / ComplexPlane / ThreeDAxes placement with `c2p` following | fm-native-axes-reposition-n1yd | `Frame2`; build planes in place |
| U-13 | Table and BarChart hard-code black rules and axes | none | build from primitives |
| U-14 | every frame snapshots every arena slot serially, and the camera route re-materializes the stage per frame | fm-sq8.5 (serial capture) | fewer, larger mobjects (row strips) |
| U-15 | group positional ops dedupe the family in O(M²) | none | never transform groups over ~2k members per frame |
| U-16 | camera-route MP4 with audio is untested; `render_camera` installs no ffmpeg | none (by design) | kit camera wrapper; S-5 renders one |
| U-17 | no `\cancel`; `\preceq` and ✗ in no bundled face | none | drawn lines; `exmark` |

---

## §15 Testing strategy

"Comprehensive" here means: every number, every claim, every DSP block and
every pipeline step has a test, and every end-to-end path has a script with
detailed, structured logs.

### 15.1 Unit tests

- **explainer_kit:** `Frame2` round trips; layout-lint geometry (outside, edge,
  overlap thresholds on synthetic boxes); `script.tsv` parser (ids, tabs,
  duplicates, empty lines); narration timing (breath, hold, scratch-voice
  formula); cue-log format; encoder argv contract (every rewrite pinned, as
  today); stats NumPy-parity vectors (seeded PCG64DXSM draws, permutations,
  `null_quantile` = NumPy `inverted_cdf`); captions (monotone times, no overlap,
  text equals `spoken.tsv`, line-length limits); never-overwrite output naming.
- **explainer_score:** PolyBLEP aliasing ≤ −50 dB for f0 ≤ 2 kHz; ladder −3 dB
  point within 5% of the requested cutoff and stable for k < 4 up to 0.45 fs;
  SVF responses; FDN T60 within ±10% (Schroeder backward integration);
  Karplus–Strong tuning within ±2 cents; BS.1770 loudness and 4× true-peak
  meters against the EBU Tech 3341/3342 test vectors; duck curves continuous
  with monotone ramps and exact region boundaries; pocket FIR magnitude response
  within ±0.5 dB of spec; WAV writer round trip through franken_manim's own
  decoder (f32/s24/s16); voice-leading solver properties (no voice crosses the
  95–250 Hz exclusion zone during speech; common tones held); determinism (same
  seed, same bytes on one platform); chapter independence (editing chapter k
  leaves other stems' fingerprints unchanged).
- **Each explainer:** `model.rs` against the oracle (exact where exact);
  `facts::self_check` at startup; narration-truth tests (each spoken verdict
  asserted against the rendered data); always/never edge tests; registry/script
  mapping; typed literals pinned to `facts`.

### 15.2 Property and golden tests

- Property: score stems never NaN, never above their peak budget, exactly the
  scene's length; layout lint never reports on an empty scene; captions cover
  every spoken sentence exactly once.
- Golden: score spectral fingerprints (third-octave energy per second + chroma,
  ±0.5 dB) per fixture chapter; a few deterministic frames per chapter at 480p
  hashed per platform as an opt-in regression smoke (franken_manim's standard
  mode is deterministic on one build and platform).

### 15.3 End-to-end scripts (`tools/e2e/`)

Each script writes NDJSON to `logs/e2e/<script>-<UTC timestamp>.ndjson` (one
object per step: `step`, `cmd`, `exit`, `seconds`, key metrics, artifact
paths with sizes and hashes) plus a short human summary, and exits non-zero on
the first failed assertion.

| Script | What it proves |
|---|---|
| `e2e_workspace.sh` | fmt, clippy, every crate's tests, tools tests; per-crate timings |
| `e2e_scaffold.sh` | `./x new` in a temp copy → builds → `facts-check` → `status` gate transitions are right |
| `e2e_draft_smoke.sh <slug>` | one chapter at 480×270@15 with `--lint`: MP4 decodes, frame count = expected, cue log parses, layout report parses, contact sheet written |
| `e2e_score.sh <slug>` | stems from a fixture narration timeline → score QA JSON → every threshold asserted |
| `e2e_master.sh` | PCM master on two short fixture chapters (one retained, one camera segment, a stem spanning the boundary) → video-only concat join → `verify_video.sh` with EXPECT_* → null test of our `SoundMixer` replay against fmn's captured s16 `soundtrack.wav` ≤ −90 dBFS → frame count = Σ frames, samples = Σ frames × 800 (spike S-9 made permanent) |
| `e2e_timeline.sh <slug>` | the construct-only timeline pass: every cue logged with integer frames and exact f64 bits; replayed placements equal fmn's own sample positions |
| `e2e_captions.sh <slug>` | SRT/VTT validity, monotone, aligned to cue logs within one frame |
| `e2e_publish_dryrun.sh <slug>` | the release bundle assembles, checksums verify, nothing is uploaded |

Fixture narration for score tests: the Hoeffding v5 WAVs when present locally;
otherwise synthetic speech-shaped noise bursts (generated by a test helper with
a known long-term spectrum and pause pattern) so CI can run the score tests
without any narration.

---

## §16 Workstreams, milestones and the dependency graph

### 16.1 Workstreams

| W | Name | Contents | Depends on |
|---|---|---|---|
| W0 | Foundation | this repo, license, .gitignore, README, AGENTS.md, research memos, this plan, beads, public GitHub repo | — (done in the founding session, except steady-state plan reviews) |
| W1 | Workspace and kit | S-1, root workspace, toolchain, `explainer_kit` extraction + new `props`/`captions`, `./x` driver, `templates/explainer`, CI, doctrine docs extracted from §5–§6 | W0 |
| W2 | Score engine | `explainer_score` modules, instruments, theory and voice leading, score DSL, ducking and pocket, mixer, WAV writer, QA, PCM master, listening protocol doc | W1 (for integration; DSP modules can start at once) |
| W3 | Hoeffding v6 | §12 steps 2–8 | W1, W2 |
| W4 | Video A | P0–P10 for `capture_recapture` | research (done); W9 for reviews; W1 for P4+; W2/W3 for P7s+ |
| W5 | Video B | P0–P10 for `raptorq_gf256` | same |
| W6 | Video C | P0–P10 for `bakery_algorithm` | same |
| W7 | Spikes and upstream | S-2…S-9 evidence; upstream repro scenes and packets (§14) | W1 (kit examples) |
| W8 | Publishing | captions, release bundling, YouTube package, thumbnails, hero/OG images | W1; per-video finals |
| W9 | Review system | persona prompts, packet assembly, the audit checklist, dispositions format | W0 |

### 16.2 Milestones

| M | Name | Done when |
|---|---|---|
| M0 | Foundation | repo public; plan + beads committed; import verified (2026-10-09) |
| M1 | Workspace and kit | S-1 and S-7 decided; workspace builds; template scaffold works end to end (`e2e_scaffold.sh`) |
| M2 | New sound | score engine passes its tests; Hoeffding v6 passes G7s and G8; the owner approves the A/B |
| M3 | Video A shipped | `capture_recapture-v1` released after G10 |
| M4 | Videos B and C shipped | both released |
| M5 | Corpus v1 | README corpus table and showcase matrix filled from measurements; lessons folded back into templates and doctrine |

### 16.3 The critical path

Pre-production for all three videos (brief → facts → storyboard → R1 → script
→ R2) needs only the research (done) and the review system (W9), so it starts
immediately and runs in parallel. Building (P4) needs the kit (W1) and the
glyph decisions (S-7). The narrated drafts need narration (GPU host). The score
(P7s) needs W2 and, by policy, the owner's approval of the Hoeffding v6 sound
(M2) so that three videos are not scored with an unapproved engine. Finals and
publishing follow. Video A is built first to shake out the kit; B and C follow
in parallel.

```text
W0 ─┬─ W9 ─┬─ A:P0..P3r ─┐
    │      ├─ B:P0..P3r ─┤
    │      └─ C:P0..P3r ─┤
    ├─ W1 (S-1, S-7, kit, x, CI) ─┬─ A:P4..P7r ── A:P7s ── A:P8..P10
    │                             ├─ W7 spikes ─┘           ▲
    │                             └─ W2 score ── W3 Hoeffding v6 (M2) ─┘ (gates A/B/C P7s)
    └─ (B and C P4.. follow A's P5 so kit fixes land once)
```

---

## §17 Risk register

| id | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | franken_manim changes underneath us (it moved 15 commits during this session; TeX layout changed with the franken_markdown pin) | high | medium | pin by rev (D-02); upgrade deliberately with tests + glyph probes + fresh sheets; record commit + dirty in every render log |
| R2 | particle and big-grid effects too slow at 4K | medium | medium | spikes S-2/S-3/S-4 with fallbacks that keep the numbers exact |
| R3 | camera route too slow at 4K | medium | low | segments; short orbits; S-5 |
| R4 | glyph gaps (α, binomials, ≺) | high | medium | S-7; notation fallbacks (§6.5) |
| R5 | TTS mispronounces names (Dijkstra, Erdős, Galois, Shokrollahi, Feller) | medium | medium | lookup + probe + owner override; avoid names that add nothing |
| R6 | the new score does not please the owner | medium | high | A/B on Hoeffding before scoring new videos (M2); identities are data, cheap to change |
| R7 | length creep past ~14 min | medium | medium | 85 s chapter cap; optional beats marked for cutting; R3 checks pacing |
| R8 | a correctness error ships | low | high | oracle + narration-truth tests + mathematician audit + R2; every number from `facts` |
| R9 | the Kosovo example lands badly | low | medium | restraint (§9.4, 10_5); owner decides (Q6) |
| R10 | copyright: book scans, figures, logos | low | medium | short attributed quotes in our typography; figures redrawn from their data; no logos; downloaded documents never committed |
| R11 | shared GPU host contention distorts timings | high | low | never quote a timing without an idle-host measurement |
| R12 | skills change during production (peer sessions republish them) | medium | low | record skill versions (commit / published version) in PROGRESS.md at each phase |
| R13 | private host names or local paths leak into the public repo | medium | medium | D-18; a CI grep for absolute home paths and the host's name; reviewed in every commit |
| R14 | fmn git fetch is huge or slow | medium | low | S-1 fallback to path dependency |
| R15 | third parties cannot narrate without the skills | high | low | document the prerequisite; Q4 |
| R16 | arena growth from `always_redraw` slows long chapters (fm-92gu) | high | medium | the §6.6 rules; S-8 |
| R17 | a camera segment encoded differently from its neighbours corrupts the stream-copy join | medium | high | one runner-wrapped capability for both routes; S-5, S-9 |
| R18 | hosted CI cost of the fmn closure | high | low | CI runs std-only crates; fmn crates gate locally (§3.8) |

---

## §18 Decision log

| id | Decision | Why |
|---|---|---|
| D-01 | One monorepo with a Cargo workspace (`crates/*`, `explainer_corpus/*`) | one franken_manim compile, one lock, one kit; fixes propagate |
| D-02 | Pin `fmn` by git revision; `--fmn-local` override with an absolute path, a private lockfile and target dir; G8 refuses it | fresh clones build; deliberate upgrades; the committed lock never changes under an override (verified by experiment E1) |
| D-03 | Extract shared crates from the skill template, not from Hoeffding's older copies | the template is a superset (layout lint, script.tsv, stats kit, never-overwrite outputs) |
| D-04 | Import Hoeffding faithfully first, migrate it later as v6 | v5 stays reproducible from this repo; the migration is a separate, reviewable step |
| D-05 | ~13 min per new video, 10 chapters ≤ 85 s (the skill's default is 6–9 min) | these topics need the room to be deep without rushing; the cap per chapter keeps 4K renders under the raw-byte limit; owner may veto (Q1) |
| D-06 | Voice `robert` for every video | continuity across the corpus; proven pipeline (Q2) |
| D-07 | Notation: N, M, n, k and table cells a, b, u (fish); `\binom` (fixed upstream) with C(·,·) as fallback; α via `\mathsf{\alpha}` or `\text{α}` in one Tex | glyph safety; consistency with Feller via one mapping card |
| D-08 | Music is ducked offline into the stem; `gain_to_background` is never used | the engine's duck is a step gain on the whole mix with no ramps |
| D-09 | Music stems are f32 stereo 48 kHz WAV, frame-aligned (800 samples per 60 fps frame) | lossless; no dither needed; ≤ 174.8 s per file fits ≤ 85 s chapters |
| D-10 | No melody under speech; cues in gaps; the levels table in §7.2 | intelligibility research; the changing-state effect; v5 measurements |
| D-11 | Persona reviews R1/R2/R3 are gates with committed dispositions | the three viewer contracts are the mission; evidence beats intention |
| D-12 | Renders never enter git; GitHub Releases carry masters | 4K masters exceed GitHub's file limit; renders are regenerable |
| D-13 | Research memos and their verification scripts are committed; each explainer's oracle reuses them | one verified source for every number |
| D-14 | Order: W1 → W2 → Hoeffding v6 A/B → Video A → Videos B and C | proves infrastructure on approved material first; A is the most visual and the simplest math |
| D-15 | The narration skills are referenced by path, not vendored | they are the owner's published skills; vendoring is the owner's call (Q4) |
| D-16 | 3D and camera shots are separate scenes (segments) joined by stream copy with one encoder | the camera route is a whole-render choice and slower |
| D-17 | No changes to franken_manim from this repo; upstream asks go through its beads with the owner's OK | the shared engine tree has other agents' work in progress |
| D-18 | Committed files never name private hosts or absolute home paths | public repository hygiene |
| D-19 | The README describes the system in the present tense and gives every video an explicit status | honest about what exists, without rewriting the README at every milestone |
| D-20 | Master the soundtrack from PCM sources by replaying the logged cues through fmn's own `SoundMixer`, once, with one fixed gain; join video by the concat demuxer over video-only remuxes, timestamps checked against the frame grid (an Annex-B join loses B-frame pts) | avoids double AAC, undithered s16 and seam gaps; a null test against fmn's captured s16 soundtrack proves equivalence |
| D-21 | V is the narration's loudness as placed; the new engine places narration at 0 dB | the old −9.4 dB placement made the pad sit ≈5.6 LU under the voice while the measurements claimed 15 |
| D-22 | Score in two passes (construct-only timeline dump, then per-act music sliced per chapter) | parallel chapter renders cannot see each other's narration; music crosses chapter boundaries |
| D-23 | Large visual families are row-strip VMobjects recoloured in place; `always_redraw` only for small readouts | O(M²) group ops, serial per-frame snapshots and arena growth |
| D-24 | Video B's story: packets are plain XOR, the danger lives at the end, the byte rows close it; wrap-around fails exactly like XOR | the revision 1 story implied byte-coefficient packets, which is false for RaptorQ |
| D-25 | Hero pictures: A the four-cell dark-cell table; B the danger meter + byte clock; C space-time lanes + the progress square | each makes its video's surprise inevitable in hindsight (§5.1 rule 19) |
| D-26 | Working titles: *How to Count Fish You've Never Seen*, *The Arithmetic Where 3 × 7 = 9*, *The Algorithm Lamport Is Proudest Of* | honest, curiosity-provoking, each backed by the video (Q11) |

---

## §19 Open questions for the owner

These have working defaults (in brackets); none blocks pre-production.

- **Q1.** Is ~13 minutes per video right, or should any of the three be split
  into two shorter videos? [keep ~13 min]
- **Q2.** Keep the `robert` voice, or enroll or choose another? [robert]
- **Q3.** Production order after Hoeffding v6: A, then B and C? [yes]
- **Q4.** Vendor the narration skill's scripts into `tools/`, or keep
  referencing the installed skills? [reference]
- **Q5.** Where do finished videos go: a YouTube channel (which one), GitHub
  Releases only, or both? [both, uploads by the owner]
- **Q6.** Include the Kosovo casualty-estimation example in Video A? [yes, with
  restraint]
- **Q7.** How should Feller, Laplace, Dijkstra, Erdős, Galois and Shokrollahi be
  pronounced (the owner's pronunciation overrides every source)? [the
  researched pronunciations, probed]
- **Q8.** Any visual identity for the series (title card style, a recurring
  sign-off) beyond the 3b1b look? [none; each video's motif closes it]
- **Q9.** May agents file beads in franken_manim's tracker for the engine gaps
  in §14? [prepare packets; file only on approval]
- **Q10.** Should the original `franken_manim_hoeffding_explainer` repository
  get a pointer to this monorepo (or be archived)? [leave it untouched]
- **Q11.** Are the working titles right (D-26)? [use them until the owner
  says otherwise]

---

## §20 The future corpus

Topics that reuse what these three build. Each has a one-line hook in the
style of §5.1 rule 2.

| Topic | Hook | Reuses |
|---|---|---|
| HyperLogLog: counting distinct things with 1.5 KB | count a billion distinct visitors by remembering only the longest run of leading zeros | capture–recapture's "estimate the unseen" arc |
| Shamir's secret sharing | split a password so any 3 of 5 people can rebuild it, and 2 learn nothing | GF(256) from Video B |
| Reed–Solomon and QR codes | why a QR code still scans with a coffee stain on it | GF(256), polynomials |
| Lamport clocks and vector clocks | ordering events with no shared clock | (counter, id) from Video C |
| Paxos | agreeing on one value while machines crash | Video C's lanes and proofs |
| The German tank problem | estimating production from captured serial numbers beat intelligence by 6× | Video A's estimation grammar |
| Bloom filters | answering "have I seen this?" with no false negatives | hashing, probability |
| The birthday paradox and hash collisions | 23 people, and why 64-bit hashes collide sooner than you think | combinatorics from Video A |
| Benford's law | why 30% of numbers in the wild start with 1 | logarithms from Video B |
| The secretary problem | stop after 37% and pick the next best | probability, likelihood |
| The Kalman filter | tracking a satellite through noise with two numbers | estimation, uncertainty |
| The fast Fourier transform | multiplying huge numbers by listening to them | sonification, polynomials |
| PageRank and Markov chains | a random surfer ranks the web | particles, flow fields |
| Huffman coding and entropy | the shortest possible code for a message | bits, trees |
| Diffie–Hellman | agreeing on a secret in public | finite fields, logarithms |
| Merkle trees | verifying one block of a terabyte with 40 hashes | trees, hashing |

---

## Appendix A: Glossary

- **Field:** a number system with addition, subtraction, multiplication and
  division by anything nonzero. **Ring:** the same without guaranteed division
  (Z/256 is a ring, not a field).
- **GF(2⁸), GF(256):** the field with 256 elements, represented as polynomials
  over {0,1} of degree < 8 modulo an irreducible degree-8 polynomial (RaptorQ:
  x⁸+x⁴+x³+x²+1, 0x11D). **Irreducible:** cannot be factored. **Primitive:**
  its root x generates every nonzero element.
- **XOR (ex-or):** bitwise addition without carries; addition in GF(2) and in
  GF(2⁸).
- **Rank:** the number of independent equations in a system.
- **Peeling:** decoding by repeatedly solving equations with one unknown left.
  **Inactivation:** parking a stuck unknown as a symbol and solving for it
  later.
- **Hypergeometric distribution:** the distribution of the number of marked
  items in a sample drawn without replacement.
- **Likelihood:** for fixed data, the probability of that data as a function of
  the unknown parameter. **Maximum likelihood estimate (MLE):** the parameter
  value that maximizes it.
- **Confidence interval:** an interval built by a procedure that captures the
  true parameter in at least the stated fraction of repeated studies.
- **Critical section; mutual exclusion:** code at most one process may run at a
  time; the guarantee that it is so.
- **Doorway:** in the bakery algorithm, the stretch from raising `choosing[i]`
  to lowering it.
- **Safe / regular / atomic register:** memory whose reads overlapping a write
  may return anything / the old or the new value / behave as if instantaneous.
- **FCFS:** first-come-first-served, measured from finishing the doorway.
- **TSO:** total store order, x86's memory model, in which each core's writes
  wait in a store buffer.
- **LUFS / LU:** loudness units (EBU R128 / ITU-R BS.1770). **dBTP:** true peak.
- **SII:** the Speech Intelligibility Index (ANSI S3.5).
- **PolyBLEP, ZDF ladder, ADAA, FDN:** band-limited oscillator correction; a
  zero-delay-feedback Moog-style filter; antiderivative anti-aliasing for
  saturation; a feedback-delay-network reverb.

## Appendix B: Source index

- `docs/research/capture_recapture/MEMO.md`: Feller (1st ed. 1950, 3rd ed.
  1968), Laplace (1786, 1802), Petersen (1896) and Goudie & Goudie (2007),
  Lincoln (1930), Schnabel (1938), Chapman (1951), Bailey (1951), Robson &
  Regier (1964), Ball et al. (Kosovo, 2002), Spagat (2014), Briand et al.
  (2000), Mills (1972), Sadinle (2009).
- `docs/research/raptorq_gf256/MEMO.md`: RFC 6330, RFC 3453, RFC 6683, the
  Qualcomm RaptorQ technical overview (2010), Shokrollahi (2006), Shokrollahi &
  Luby (2011), Liva–Paolini–Chiani (2010), ATSC A/331, 3GPP SP-050246, Anvin's
  RAID-6 notes, `cberner/raptorq`, the owner's article, X post and the two AI
  chats it links.
- `docs/research/bakery_algorithm/MEMO.md`: Lamport (1974, 1983, 1986 ×2,
  2015 Turing lecture, publications page, 2016 CHM oral history), Dijkstra
  (1965), Knuth (1966), Peterson (1981), Burns & Lynch (1993), Herlihy (1991),
  Taubenfeld (2004), Aravind & Hesselink (2011), Sewell et al. (2010), Abdulla
  et al. (2012), Arm TF-A, the owner's article.
- `docs/research/sound_design/MEMO.md`: EBU R128 / Tech 3341, ITU-R BS.1770,
  ATSC A/85, Torcoli et al. (2019), Straninger (2020), WCAG 1.4.7, Moreno &
  Mayer (2000), Jones & Macken (1993), Salamé & Baddeley (1989), Vasilev et al.
  (2018), ANSI S3.5, Plomp & Levelt (1965), Zwicker & Terhardt (1980),
  Zavalishin, Huovilainen (2004), Stilson & Smith (1996), Simper, Välimäki &
  Huovilainen (2007), Chowning (1973), Parker–Zavalishin–Le Bivic (2016), Smith
  (PASP), Dattorro (1997), the Sonification Handbook, Rubinetti's published
  3b1b sheets.
- `docs/research/franken_manim_capabilities/CAPABILITIES_AT_fb1c1b9e.md`: the
  engine survey with file:line citations.
