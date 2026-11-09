# Explicit video encoding quality

`RenderOptions::video_quality`, the CLI, and optional configuration keys control
compressed video independently of scene resolution, frame sampling, and Lumen's
raw-frame renderer. The resolved policy reaches
the existing negotiated `VideoJob` and `encode_argv` through the ordinary
streaming ffmpeg boundary. It is not an argv-rewriting process wrapper.

## Native Rust usage

```rust
use fmn::rendering::{EncoderPreset, EncoderTune, RenderFormat, RenderOptions, VideoQuality};

let mut options = RenderOptions::with_format("lecture.mp4", RenderFormat::Mp4)?;
options.video_quality = VideoQuality {
    crf: Some(16),
    preset: Some(EncoderPreset::Slow),
    tune: Some(EncoderTune::Animation),
    bitrate: None,
};
// Pass options to render(), render_with_fs(), render_camera(), or render_bundle().
// Those entry points share the native render sink and one encoder negotiation.
```

This is an explicit settings example, not a new default or a visual-quality
certificate. `VideoQuality::default()` sets every field to `None`: absent
configuration overrides, no `-crf`, `-preset`, `-tune`, or `-b:v` is added.
The new controls do not fix a color-transfer/tag mismatch or establish HDR,
codec availability, cross-platform encoding identity, or a renderer speedup.

The configured codec remains `options.config.file_writer.video_codec`. CRF
accepts integers from 0 through 51 for `libx264`/`libx265`. Presets are the closed
catalog `ultrafast`, `superfast`, `veryfast`, `faster`, `fast`, `medium`, `slow`,
`slower`, `veryslow`, `placebo`. Tunes accept one catalog value; comma-separated
combinations and arbitrary encoder parameter strings are refused.

| Encoder | Admitted controls |
| --- | --- |
| `libx264` | CRF or target bitrate, preset, film/animation/grain/stillimage/psnr/ssim/fastdecode/zerolatency tune |
| `libx265` | CRF or target bitrate, preset, animation/grain/psnr/ssim/fastdecode/zerolatency tune |
| `h264_videotoolbox`, `hevc_videotoolbox`, `h264_nvenc`, `hevc_nvenc`, `av1_nvenc` | Target bitrate only; software CRF/preset/tune requests fail explicitly |

`bitrate: Some(12_000_000)` means twelve million **bits per second**, not bytes
and not a strict output-size ceiling. It is an alternative to CRF; supplying
both is an error. Artifact/input ceilings are still independently enforced by
[the native job-limit policy](NATIVE_VIDEO_LIMITS.md). Unsupported encoders,
zero bitrate, alpha-preserving MOV, and native PNG/GIF/Y4M reject explicit video
quality settings rather than quietly dropping them. Catalog string parsers
accept only exact canonical spellings, not raw argv or shell fragments.

## CLI and configuration files

The same controls are available for registered native scenes and compiled FMTL
inputs. A quality flag selects video output and implies `--write_file`:

```sh
fmn --crf 16 --preset slow --tune animation --video_dir videos @builtin circle_shift.v1
fmn --format video --video-bitrate 12000000 --vcodec h264_nvenc @builtin circle_shift.v1
```

| CLI flag | Configuration key | Meaning |
| --- | --- | --- |
| `--crf 16` | `file_writer.video_crf` | Software x264/x265 CRF, integer 0–51 |
| `--preset slow` | `file_writer.video_preset` | One preset from the software encoder catalog |
| `--tune animation` | `file_writer.video_tune` | One tune supported by the selected software encoder |
| `--video-bitrate 12000000` | `file_writer.video_bitrate` | Positive target bitrate in bits per second, at most 4,294,967,295 |

For reusable encoder defaults, pass a file with `--config_file quality.yml`:

```yaml
file_writer:
  video_crf: 16
  video_preset: slow
  video_tune: animation
```

These keys are optional and accept `null` to clear an earlier file layer. The
CLI overrides configuration one field at a time. Selecting `--video-bitrate`
clears a configured CRF; selecting `--crf` clears a configured bitrate. Supplying
both flags, or both non-null values in the effective configuration, is an error.
Explicit `RenderOptions::video_quality` fields follow the same override rules;
`effective_video_quality()` exposes the resolved policy before rendering.

File settings apply when video output is requested; use `--format video` when
the controls come entirely from a configuration file. They do not select a
format by themselves or affect PNG, GIF, Y4M, WAV, or SVG output. Explicit quality
flags on those formats, including `--gif`, are refused. Encoder compatibility
and configured values are checked before ffmpeg discovery or scene execution.

## Shared negotiation and provenance

Lower-level callers use `VideoJob::with_quality`. Existing `VideoJob` struct
literals and its legacy `crf` field remain supported. A configured CRF and a
legacy CRF must agree; conflicting values fail rather than follow accidental
precedence. `EncoderChoice::Configured` binds a resolved encoder name to the
policy. Exhaustive matches over that public enum must handle the new variant;
consumers should use `resolved_encoder()` when they only need its name.

Validation is repeated at encoder resolution and argument generation, including
for callers who construct or mutate a configured enum directly. Encoder options
are emitted once, after the raw input and output codec and before the output
path. Frame dimensions/rate, pixel conversion, color description, mux stream
copy, private executable binding, and atomic no-clobber publication are unchanged.

A successful native video receipt includes the resolved
`RenderArtifact::video_quality`, and the existing ffmpeg invocation provenance
records the exact encoder argv. An
empty policy in a receipt means that the encoder's defaults were requested;
it does not infer a particular default CRF from the installed tool.

CLI provenance includes the settings in the resolved configuration input (C4)
and records the exact emitted encoder arguments. Python controls are documented
in [Python video quality](PYTHON_VIDEO_QUALITY.md). AAC audio bitrate is not yet
configurable through this policy.

## Regression evidence

```sh
cargo test --locked -p fmn-output --lib negotiate::quality
cargo test --locked -p fmn-output --features ffmpeg-test-fixture --test video_quality_boundary
cargo test --locked -p fmn-config --test config optional_video_quality
cargo test --locked -p fmn-cli --lib video_quality
cargo test --locked -p fmn-conformance --test e2e_scenarios cli_video_quality_admission_scenario_passes
FMN_REQUIRE_FULL_INPUTS=1 cargo test --locked -p fmn --test native_video_quality -- --nocapture
```

The CLI/configuration tests cover layering, explicit overrides, rate-mode
switching, optional-key typing, actual negotiated argv, changed configuration
bytes, and named refusals before tool discovery. The pure tests cover the
complete CRF/preset/tune combinations, out-of-range
values, hardware refusals, conflicting rate modes, mutation revalidation and
unchanged default argv. The native fake executable records what the actual
streaming child received. The supported-host test renders MP4/MOV through the
native API, with and without a camera and using both configured and explicit
quality policies, then checks x264's user-data SEI inside
`mdat`: requested CRF, the slow preset's `subme=8`, and animation's `psy_rd=0.40`.
The bounded test reader exercises AVC emulation-prevention bytes and refuses
truncated NAL/box lengths; it does not mistake unrelated metadata text for SEI.
Only an executed passing test run is evidence for the Rust integration.
