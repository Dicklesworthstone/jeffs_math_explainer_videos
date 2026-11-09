# BN-14 — Sound is placed on the sample grid, not a millisecond clock

**Status:** Draft. Landed in W8 (fm-0m7); becomes Final when G4's complete
native output pipeline and A/V corpus pass.

## What changed

Classic manim hands sound to pydub at `int(1000 * time)` milliseconds. That
truncates every placement to a millisecond, accumulates a separate audio clock
beside the frame clock, and raises when the resulting position is negative.
The selected pydub/ffmpeg resampler and quantizer are also host-installation
details rather than scene provenance.

FrankenManim keeps the `Scene.add_sound(sound_file, time_offset=0, gain=None,
gain_to_background=None)` surface, but Reel derives the placement directly from
Choreo's rational frame time:

```text
nearest_sample(frame * sample_rate / fps) + nearest_sample(time_offset)
```

Both roundings are exact over the received integers/f64, with ties away from
zero. There is no accumulated audio time. A pre-zero start clips the prefix at
sample zero; an overlay past the current end extends the output.

PCM conversion uses the named `blackman-windowed-sinc-64` resampler. It is a
64-tap Blackman-windowed sinc evaluated from the exact output-index/input-rate
ratio, with a measured contract of at most 0.1 dB passband loss through 80% of
the lower Nyquist frequency and at least 60 dB rejection from 150% onward.
Mono duplicates to stereo; stereo averages to mono. Other layouts are named
refusals rather than guessed.

## What stayed familiar

- `gain` is foreground gain in dB.
- `gain_to_background` is dB ducking applied to the already-mixed background
  during the new sound's active interval.
- Sounds mix in `add_sound` call order, so ducking and overlay order remain
  observable in the same direction as the Reference.
- The mix extends when a sound runs past its previous end.

## Defined output policy

Every sample sees the same cue order. Worker count only partitions disjoint
timeline ranges, so `{1,4,16}` workers produce identical PCM and WAV bytes.
Final over-range samples clamp to `[-1, 1]`; `MixReport.clipped_samples` is the
warning counter. Certified s16 WAV defaults to no dither. The optional TPDF
dither is seeded and counter-derived per sample, so it is reproducible and
schedule-independent. Native f32 WAV is not dithered.

Non-WAV assets decode through the fingerprinted ffmpeg capability and fail with
the named native-alternative capability error when ffmpeg is absent. That
decoded product, like every ffmpeg product, is outside certification; the
native mix and certified WAV artifact are inside it.

## Native video composition

The native Rust rendering API and standalone CLI use the same
`PreparedSceneSoundtrack` composer. It reads each authored source path once
under the asset byte cap and retains that immutable source snapshot for all
uses of the cue. Native WAV decoding, channel conversion, resampling, gain,
ducking and output quantization remain Reel's operations. Other supported
containers use the injected video boundary for decoding, including its
ffmpeg fingerprint, timeout, log cap and artifact cap.

Ordinary live video runs the scene once. Lifecycle events record omitted
frame intervals; sound call-site frames are rebased onto the emitted video
timeline before the relative offset is applied. `add_sound` calls made while
skipping remain an early no-op, so their paths are not read. The mix spans at
least the emitted picture and retains a longer cue tail.

The CLI's `--subdivide` already performs a deterministic counting pass for
its compiled-in native programs. It also collects that pass's complete cue
list and decodes it once before starting the clips. Each clip receives the
corresponding half-open absolute sample window of that whole-scene mix:
background phase and ducking are preserved, cue tails stop at the clip end,
and a cue authored later with a negative offset can affect an earlier clip.
The start and end scene positions are rounded to samples independently, so
adjacent clips neither lose nor repeat a rounding sample. This is offline
whole-scene composition; the portal's live per-call clips see the cues
available when their own recording boundary closes.

The completed native PCM is supplied before the video sink prepares its
output. The existing second ffmpeg stage uses `-c:v copy -c:a aac`; audio
decode, mix, or mux failure prevents publication of the failing video
generation. Encoded video remains outside certification.

CLI robot records include `soundtrack` (cue count, sample frames, rate,
channels, clipping), `audio_inputs` (source and normalized PCM SHA-256),
`ffmpeg_invocations` (encode, then optional mux), and `audio_decodes`.
Decoder receipts are separate because decoding can overlap a streaming
video encoder. C1 binds each source path once even when several cues reuse
it; C9 binds the audio identities and all media invocation receipts. Native
Rust `RenderArtifact` likewise separates `audio_decodes` from its existing
encode/mux `ffmpeg` list.

## Migration guidance

- Code that expected integer-millisecond placement should remove that
  assumption. At 48 kHz, FrankenManim addresses all 48 samples within each
  millisecond.
- A negative effective start no longer raises. The inaudible pre-zero prefix is
  discarded and the remainder starts at sample zero.
- Do not rely on a host pydub/ffmpeg resampler, quantizer, or channel guess.
  Compare native WAV semantics, not those installation-dependent bytes.
- If clipping is intentional, inspect and acknowledge the clipping counter;
  otherwise reduce cue gains. Wraparound is never available.

## Evidence

- `crates/fmn-scene/src/runtime.rs`: the four-argument request surface, exact
  `RationalTime`, skip no-op, and insertion-ordered request queue.
- `crates/fmn-output/src/sound.rs`: placement arithmetic, resampler, channel
  matrix, gain/duck order, clipping, dithering, WAV golden, quality bounds, and
  thread-count equivalence.
- `crates/fmn-output/src/ffmpeg.rs` and `tests/boundary.rs`: sandboxed non-WAV
  decode and fake-ffmpeg capability proof.
- `crates/fmn-conformance/tests/sound_pipeline.rs`: frame-N click placement and
  identical certified WAV bytes at `{1,4,16}` workers.
- `crates/fmn/src/rendering/soundtrack.rs`: shared source snapshots, skipped
  time, offsets, and rational-window parity with resampling and ducking.
- `crates/fmn-cli/src/lib.rs`, `tests::video_soundtrack`: the live CLI render,
  deferred mux, subdivision windows, compressed input receipts, and failure
  publication contract through a scripted process capability.
- `crates/fmn-conformance/tests/e2e_scenarios.rs`,
  `lifecycle.scene_soundtrack_windows.v1`: real scene clock and skip events
  through the shared audio composer and adjacent sample windows.
- Pinned Reference
  `manimlib/scene/scene.py::add_sound` and
  `manimlib/scene/scene_file_writer.py::add_sound`.
