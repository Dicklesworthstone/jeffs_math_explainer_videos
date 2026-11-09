# FMTL/1 — the tier-2 timeline bundle layout (AUTHORITATIVE CONTRACT, fm-oee)

**Owner decision, 2026-07-30 (GentleBeaver).** The shared reader and writer
(`fmn_scene::TimelineBundle` and the scene-side exporter) both implement
EXACTLY this layout. `fmn-wasm` and `fmn-cli` consume that reader rather than
carrying format-specific decoders. Drift is a bug in the shared codec.

**Name history.** The draft magic `FMNT` collides with
`fmn-render/src/texture.rs`'s texture schema (`Schema::new(*b"FMNT", 1, 1, 0)`) —
this bundle is `FMTL`, format version 1.

## Container

One `fmn_hash::serial` document: `Schema::new(*b"FMTL", 1, 0, 0)`. Field order
below is fixed and serialized in order. Integers are the container's canonical
little-endian; floats are IEEE-754 bits; strings are length-prefixed UTF-8;
`bytes` fields are length-prefixed octets.

## Fields (in order)

1. `engine_version: string` — the certified renderer closure followed by
   `:fmtl-law:<version>`, where the version is
   `fmn_anim::bundle::RECONSTRUCTION_LAW_VERSION`. The player refuses either
   component's mismatch before interpreting any later field. Renderer-only
   identities predate law version 2 and are deliberately refused: re-export
   the source timeline with the matching engine. Version 2 includes typed
   tracker payloads and stable translation-independent affine interpolation;
   an older player cannot safely reconstruct these merely because its
   rasterizer version matches. The field's wire type and FMTL/1 framing do
   not change.
2. `fps: u32` — the schedule's frame rate (MUST equal the nested plan's fps).
3. `plan: bytes` — the nested `TimelinePlan::to_bytes()` document (FMNA/5).
4. `segments: u32` — MUST equal `plan.segments().len()`.
5. Per segment entry, in authored order:
   a. `kind: u8` — `0` = pure-reconstructible, `1` = stateful (recorded).
   b. If `kind == 0`:
      - `begin: bytes` — the begin-state Stage snapshot (the scene-side
        snapshot machinery's canonical document, verbatim).
      - `end: bytes` — the end-state Stage snapshot, same form.
      - `path: u8` — the PathFunc identity (`0` = straight, then the
        `PathFunc` catalog order; the exporter's table is normative).
      - `rate: u8` — the RateFunc identity (catalog order, same rule).
      - Reconstruction law (normative): for frame alpha, the player computes
        `a = rate(alpha)` and record-interpolates begin→end exactly as
        `fmn_anim::transform::interpolate_fields` does — pointlike fields
        through `path`, every other field linear, locked fields skipped,
        computed in f64 and stored at record precision. Typed tracker lanes
        also interpolate in f64: scalar/logarithmic/complex encodings retain
        their native meaning. **Export rule: the
        writer PROVES reconstructibility before marking a segment kind 0 —
        it computes every emitted frame through the engine and through
        record-lerp, and requires bit-identity; any segment failing the proof
        exports as kind 1 instead. Never guessed.**
   c. If `kind == 1`:
      - `frames: u32`, then per frame: `snapshot: bytes` (verbatim as above).

## Refusals (named errors, never panics)

- Malformed container → the serial reader's typed error.
- `engine_version` mismatch → `EngineMismatch { wanted, found }`.
- `fps`/`segments` disagreement with the nested plan → `PlanInconsistent`.
- Unknown `kind`/`path`/`rate` tag → `PlanInconsistent`.
- A nested plan whose total cannot fit the player's public `u32` frame-count
  surface → `FrameCountUnrepresentable` (never saturation).
- A declared segment or stateful-frame table that cannot fit the remaining
  payload's mandatory field prefixes → `PlanInconsistent`, before reservation.
- Allocator refusal while reserving a payload-validated table →
  `AllocationFailed`; loading fails without constructing a partial player.
- A compiled export plan above `BundleExportLimits::max_frames` →
  `FrameLimitExceeded`, before frame one and before scene mutation. The
  production default is 1,000,000 frames.
- Capture/proof tables and canonical snapshot bytes above the exporter's
  cumulative `BundleExportLimits::max_capture_bytes` budget →
  `CaptureLimitExceeded`; a failed bounded destination reservation →
  `AllocationFailed`. The production capture default is 256 MiB.

## Determinism

Two exports of the same scene run MUST produce identical bytes: fixed field
order, canonical floats, no timestamps, no host paths, no hash-map iteration
(wherever maps appear, serialize in sorted key order).


## Camera-bearing FMTL/1 minor 1

`SceneBundleRecorder::new_render_only_with_camera` records a camera at each
actual capture boundary using `capture_with_camera` (or the explicit terminal
still counterpart). It reuses Lumen's `CameraSample`; it does not serialize
camera-rig handles, callbacks or an alternate clock. Missing camera samples,
mode mismatches and changed FPS poison the recording. Camera bytes and table
storage count toward the same cumulative capture budget as geometry.

These artifacts use schema `(FMTL, 1, 0, 1)`. After the unchanged segment payload,
a `u32` frame count is followed by exactly that many 153-byte camera samples.
The count must equal the nested plan's total output frames. Each sample contains
original pixel dimensions, frame center and shape, quaternion, field of view,
light position, clipping norm, linear RGBA background and adaptive sample count.
The existing canonical trailer binds geometry and camera together. Strict old
readers reject this additive minor version; existing minor-0 writers and their
bytes are unchanged.

The production reader validates count, exact fixed-size payload and camera
invariants before accepting the artifact. `stage_at_with_camera` and shared
`TimelineFrameCache::materialize_with_camera` return geometry and its camera as
one frame result. Geometry-only access explicitly refuses camera-bearing input.
Equal-aspect playback preserves the captured frame shape and orientation bits;
a changed aspect preserves authored frame width. Revisions derive from original
frame indices, not output order or worker completion. No quaternion
renormalization, Euler conversion or callback runs during reconstruction.

Native compiled rendering and the standalone CLI consume the track through the
existing bounded CPU render-team pipeline. Recorded pose, background, lighting
and sample policy are authoritative; `render_bundle` refuses an explicit camera
override. The WASM `FmnPlayer` also consumes the paired reconstruction API and
renders with `RetainedFrameRenderer::render_with_camera`, preserving recorded
pose, background, lighting and sample policy. Its `has_camera_track` getter
identifies this route. Caller-buffer rendering keeps the same validation and
storage-reuse contract; minor-0 bundles retain their existing planar path.
WASM playback remains standard-only, not part of the certified platform matrix.
Other geometry-only consumers must adopt the paired reconstruction API before
claiming support.
Audio and arbitrary external effects are still not represented by this format.

Regression coverage lives in `crates/fmn-conformance/tests/camera_bundle.rs`:
actual camera-rig captures versus decoded native frames at 1/4/16 threads,
random-order shared-worker reconstruction, changed backgrounds, old-reader
refusal, malformed counts/cameras, sticky capture failures and output limits.
These tests are execution evidence only when their recorded run passes; the
format addition does not itself establish cross-platform certification.

The WASM adapter adds original-capture comparisons, random-order seeking,
viewport round trips and caller-buffer/refusal tests in
`crates/fmn-wasm/src/camera_player.rs`. These need a successful Rust test run
before being treated as execution evidence.

The browser demo opens local recordings without uploading them, with a 256 MiB
input admission limit. It renders frame zero before replacing the active movie;
failed and superseded loads retain the prior movie. Playback selects recorded
frames from elapsed presentation time and the bundle FPS, not monitor refresh
rate. A late display may skip presentation frames; this does not resample the
scene, change FMTL reconstruction or alter offline output sampling.
