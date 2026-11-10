# Python video quality through the native encoder

The Python portal reads four optional attributes from `scene.file_writer`
before it starts an MP4/MOV generation: `video_crf`, `video_preset`,
`video_tune`, and `video_bitrate`. They become the same typed `VideoQuality`
used by the Rust facade. There is no process-runner decorator, raw encoder
argument string, or separate Python encoder implementation.

```python
from manimlib import Scene, Square
from fmn_python import render_scene

class Picture(Scene):
    def construct(self):
        self.add(Square())
        self.wait(1)

scene = Picture()
scene.file_writer.video_crf = 16
scene.file_writer.video_preset = "slow"
scene.file_writer.video_tune = "animation"
render_scene(scene, "picture.mp4", resolution=(1920, 1080), fps=60)
```

Install these settings before rendering begins. Missing attributes and `None`
request the previous encoder defaults. CRF zero is a real request, not a false
value to discard. `video_bitrate` is an integer **bits-per-second target**, not
a strict file-size ceiling, and cannot be combined with CRF. The shared
[quality policy](VIDEO_ENCODING_QUALITY.md) validates the selected encoder,
container, preset and tune; software-only controls on hardware encoders fail
by name. Strings are exact catalog names, never argv fragments. Booleans and
lossy float-to-integer conversions are refused. Explicit quality settings on
native PNG/GIF/Y4M/SVG/WAV requests are errors, not ignored preferences.

The adapter covers ordinary portal generations and recorded/subdivided video
exports, which both use `PortalVideoConfig`. It preserves the ownership checks
around Python getters and propagates authored getter/conversion failures.
Every successful video receipt's `ffmpeg_invocations` contains the actual
resolved encoder and complete arguments. Video remains outside certification.

## Console controls

```sh
fmn-python scene.py Picture --format mp4 --crf 16 --preset slow --tune animation --video_dir picture.mp4
fmn-python scene.py Second First --format mp4 --crf=16 --preset=slow --tune=animation --video_dir movies
fmn-python scene.py Picture --format mp4 --vcodec h264_nvenc --video-bitrate 12000000 --video_dir hardware.mp4
```

Both `--flag value` and `--flag=value` work for these four new controls. A flag
may be specified only once; CRF and bitrate are mutually exclusive. Numeric
ranges, duplicate options, malformed values and incompatible native/explicit
transparent output modes fail before source import with the existing usage
exit. Exact preset/tune names and codec compatibility remain the native
negotiator's responsibility before encoder discovery, after constructing the
selected scene. No catalog or compatibility matrix is duplicated in Python.

The flags apply to single and named multiple scenes, `--write_all`,
`--subdivide`, and video primaries paired with `--save-last-frame`. They do not
become scene constructor keywords: zero-argument custom constructors continue
to work, and the lifecycle still runs once. Existing output overrides carry
the settings through every owner. Batch checkpoints already bind the complete
output-option mapping, so changing a requested quality control changes that
checkpoint plan. Final paired PNG capture uses its native still path and does
not inherit video compression settings.

## Validation entry points

```sh
python crates/fmn-python/tests/video_cli_unit.py
FMN_REQUIRE_FFMPEG=1 cargo test --locked -p fmn-python --lib portal_video_quality -- --test-threads=1
```

The standalone lexer tests need no extension. The permanent extension suites
exercise admission before executable discovery, descriptor errors and existing
output preservation, then render MP4/MOV at two CRFs. The independent bitstream
check demuxes H.264 with stream copy and reads x264 user-data SEI; it does not
infer settings from our requested-option receipt alone. Console coverage checks
pre-import errors and all the output-owner modes above, including constructor
counts and final PNG publication. Real encode cases need ffmpeg and become
mandatory under `FMN_REQUIRE_FFMPEG=1` or `FMN_REQUIRE_FULL_INPUTS=1`.
Syntax/isolated parser checks are not extension or end-to-end rendering proof.

This does not yet configure AAC bitrate or expose these controls in standalone
Rust `fmn` CLI/config files. No encoder default or visual-quality claim changes.
