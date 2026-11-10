# BN-01 — One RNG: reproducible within FrankenManim, not across engines

**Status:** Draft (W1, fm-m1u). The stated condition is met (fm-3xk's
classifier consumes `RngRoot`), but the 2026-10-04 review against real
behaviour found the migration guidance wrong for the Python portal, and the
portal half stays Draft until its gate (G4a, fm-boe) passes.

## What changed

Classic manim consults **two** seeded legacy streams — CPython's
`random` and NumPy's legacy `RandomState` — wired through whatever import
happens to touch them. Feature changes shift unrelated draw sequences,
and "same seed" means different pictures across manim versions.

FrankenManim has exactly one stream: **PCG64DXSM seeded through NumPy's
`SeedSequence`**, bit-exact against NumPy for explicit seeds (locked by
`crates/fmn-core/fixtures/rng_vectors.txt`, generated from NumPy itself).
On top of it:

- **Named substreams** (`root.substream("stream_lines")` …): every
  subsystem draws from its own spawn-key-derived stream. Adding a
  consumer can never shift an existing consumer's draws.
- **Keyed per-frame forks**: a frame's stream is a pure function of
  `(substream, frame_index)` — never a sequential pull consumed in
  scheduler completion order. This is what makes frame-parallel
  rendering replay-identical by construction; completion-order RNG is
  permanently refused (D-18, §10.5).
- **Snapshot/restore** of full generator state feeds SceneState and the
  replay journal.
- Render-affecting map iteration uses ordered maps only
  (`fmn_core::rng::OrderedMap`; hash-map order would smuggle
  nondeterminism past the RNG discipline).

## Migration guidance

- A seeded scene reproduces **within FrankenManim** — same seed, same
  build, same bits, any thread count.
- **Native fmn and the Rust API** draw only from PCG64DXSM substreams.
  They do not reproduce the Reference's draws, because the legacy
  streams are gone there by design. Seed through the config
  (`determinism.seed`).
- **The fmn-python portal** keeps the Reference's scene-level seeding for
  source-unedited scenes:
  - `Scene.__init__` seeds CPython's `random` and NumPy's legacy global
    state with `Scene.random_seed` (default 0, `None` disables it).
  - The Reference helpers draw from those generators: `random_color`
    from NumPy, `random_bright_color` from `random`.
  - So a scene's own `random`/`np.random` draws and those helpers give
    the Reference's numbers for the same seed.
  - Randomness inside native engine code still uses the named
    substreams.
- Draws a Python scene makes itself are the scene's business. The input
  closure captures them only insofar as §16.7 documents.

## Evidence

- `crates/fmn-core/src/rng.rs` (implementation + doctrine notes),
  `crates/fmn-core/tests/rng_parity.rs` (NumPy bit-parity: SeedSequence
  words, draws, doubles, substream and fork spawn keys), unit tests for
  independence / call-order invariance / snapshot round-trip.
- Implementation detail pinned by vectors: NumPy's PCG64DXSM seeds with
  the standard 128-bit-multiplier srandom, then transitions with the DXSM
  cheap multiplier at runtime.
