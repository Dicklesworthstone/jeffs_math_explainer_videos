"""Decode real portal GIF/y4m/WAV output with independent standard-library readers."""

import contextlib
import hashlib
import importlib.machinery
import io
import json
import os
import pathlib
import shutil
import struct
import subprocess
import sys
import tempfile
import wave

import numpy as np
import manimlib


def read_gif(path):
    data = pathlib.Path(path).read_bytes()
    assert data[:6] == b"GIF89a"
    width, height, flags = struct.unpack_from("<HHB", data, 6)
    offset = 13

    def palette(count):
        nonlocal offset
        result = np.frombuffer(data[offset:offset + 3 * count], dtype=np.uint8).reshape(count, 3)
        offset += 3 * count
        return result

    def blocks():
        nonlocal offset
        result = bytearray()
        while data[offset]:
            count = data[offset]
            offset += 1
            result.extend(data[offset:offset + count])
            offset += count
        offset += 1
        return result

    global_palette = palette(2 << (flags & 7)) if flags & 128 else None
    frames, delays = [], []
    delay = None
    while data[offset] != 0x3B:
        marker = data[offset]
        offset += 1
        if marker == 0x21:
            label = data[offset]
            offset += 1
            extension = blocks()
            if label == 0xF9:
                assert len(extension) == 4
                delay = int.from_bytes(extension[1:3], "little")
            continue
        assert marker == 0x2C
        left, top, frame_width, frame_height, flags = struct.unpack_from("<HHHHB", data, offset)
        offset += 9
        assert (left, top, frame_width, frame_height) == (0, 0, width, height)
        assert not flags & 64, "reader expects non-interlaced full-canvas frames"
        colors = palette(2 << (flags & 7)) if flags & 128 else global_palette
        minimum = data[offset]
        offset += 1
        packed = blocks()
        clear, stop = 1 << minimum, (1 << minimum) + 1
        table, size, previous, decoded, bit = {}, minimum + 1, b"", bytearray(), 0
        while True:
            code = sum(((packed[(bit + k) // 8] >> ((bit + k) % 8)) & 1) << k for k in range(size))
            bit += size
            if code == clear:
                table = {index: bytes([index]) for index in range(clear)}
                table[clear], table[stop] = b"", b""
                size, previous = minimum + 1, b""
                continue
            if code == stop:
                break
            if code in table:
                word = table[code]
            else:
                assert code == len(table) and previous
                word = previous + previous[:1]
            decoded.extend(word)
            assert len(decoded) <= width * height
            if previous and len(table) < 4096:
                table[len(table)] = previous + word[:1]
                if len(table) == 1 << size and size < 12:
                    size += 1
            previous = word
        assert len(decoded) == width * height
        frames.append(colors[np.frombuffer(decoded, dtype=np.uint8)].reshape(height, width, 3))
        delays.append(delay)
    assert offset == len(data) - 1
    return frames, delays


def read_y4m(path):
    data = pathlib.Path(path).read_bytes()
    header, records = data.split(b"\n", 1)
    assert header == b"YUV4MPEG2 W96 H54 F30:1 Ip A1:1 C420mpeg2"
    size = 96 * 54 * 3 // 2
    assert len(records) % (size + 6) == 0
    frames = []
    for offset in range(0, len(records), size + 6):
        assert records[offset:offset + 6] == b"FRAME\n"
        payload = np.frombuffer(records[offset + 6:offset + 6 + size], dtype=np.uint8)
        frames.append((
            payload[:96 * 54].reshape(54, 96),
            payload[96 * 54:96 * 54 * 5 // 4].reshape(27, 48),
            payload[96 * 54 * 5 // 4:].reshape(27, 48),
        ))
    return frames


def console(*arguments):
    previous = sys.argv
    stdout, stderr = io.StringIO(), io.StringIO()
    sys.argv = ["fmn-python", "--robot", *arguments]
    try:
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            code = manimlib._console_main()
    finally:
        sys.argv = previous
    assert stderr.getvalue() == "", stderr.getvalue()
    result = json.loads(stdout.getvalue())  # Reject decoration or a second JSON record.
    assert result["schema"] == "fmn-python.cli" and result["version"] == 1
    assert result["exit"]["code"] == code
    return code, result


def batch_console(*arguments):
    """A robot batch keeps one receipt on stdout; per-scene progress lines go
    to stderr by design (batch_cli's help), so return them for checking."""
    previous = sys.argv
    stdout, stderr = io.StringIO(), io.StringIO()
    sys.argv = ["fmn-python", "--robot", *arguments]
    try:
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            code = manimlib._console_main()
    finally:
        sys.argv = previous
    result = json.loads(stdout.getvalue())  # Reject decoration or a second JSON record.
    assert result["schema"] == "fmn-python.cli" and result["version"] == 1
    assert result["exit"]["code"] == code
    return code, result, stderr.getvalue().splitlines()


output_root = pathlib.Path(globals().get("_fmn_output_root") or tempfile.mkdtemp(prefix="fmn-native-output-"))
output_root.mkdir(parents=True, exist_ok=True)
source = output_root / "native_scene.py"
source.write_text("""from manimlib import *
class MovingWhiteSquare(Scene):
    def construct(self):
        square = Square(side_length=1, fill_color=WHITE, fill_opacity=1, stroke_width=0)
        square.shift(2 * LEFT + UP)
        self.add(square)
        self.play(square.animate.shift(4 * RIGHT), run_time=3 / 30, rate_func=linear)
class FailingScene(MovingWhiteSquare):
    def construct(self):
        super().construct()
        raise RuntimeError("native-output-after-capture")
class PrimarySwatches(Scene):
    def construct(self):
        for x, color in zip((-3, 0, 3), ("#FF0000", "#00FF00", "#0000FF")):
            square = Square(side_length=2, fill_color=color, fill_opacity=1, stroke_width=0)
            self.add(square.shift(x * RIGHT))
class CameraPan(Scene):
    def construct(self):
        self.add(Square(side_length=1, fill_color=WHITE, fill_opacity=1, stroke_width=0))
        self.wait(1 / 8)
        self.play(self.frame.animate.shift(2 * RIGHT), run_time=1 / 4, rate_func=linear)
class BackgroundChanges(Scene):
    def construct(self):
        for color in ("#FF0000", "#00FF00", "#0000FF"):
            self.set_background_color(color)
            self.wait(1 / 8)
class CameraUpdater(Scene):
    def construct(self):
        self.add(Square(side_length=1, fill_color=WHITE, fill_opacity=1, stroke_width=0))
        self.frame.add_updater(lambda frame, dt: frame.shift(8 * dt * RIGHT), call=False)
        self.wait(3 / 8)
class InvalidCameraAfterCapture(CameraPan):
    def construct(self):
        super().construct()
        self.camera.background_rgba[0] = float("nan")
        self.wait(1 / 8)
class LightChanges(Scene):
    light_class = Point
    def construct(self):
        self.add(Square(side_length=2, fill_color=WHITE, fill_opacity=1,
                        stroke_width=0).set_shading(0, 0, 1))
        self.camera.light_source = self.light_class()
        self.add(self.camera.light_source)
        for z in (10, -10):
            self.camera.light_source.move_to(z * OUT)
            self.wait(1 / 8)
class ReversedLight(Point):
    def get_center(self):
        return -super().get_center()
class OverriddenLightChanges(LightChanges):
    light_class = ReversedLight
class EscapedMarkup(Scene):
    def construct(self):
        escaped = "".join(map(MarkupText.escape_markup_char, "a < b & c > d"))
        self.add(MarkupText(escaped))
class LiteralMarkup(Scene):
    def construct(self):
        self.add(Text("a < b & c > d"))
class CenteredTypesetting(Scene):
    def construct(self):
        formula = Tex(r"e^{i\\pi} + 1 = 0").scale(2)
        label = Text("Hello").scale(2)
        assert max(abs(value) for value in formula.get_center()) < 1e-7
        assert max(abs(value) for value in label.get_center()) < 1e-7
        self.add(formula)
        self.wait(1 / 30)
        self.remove(formula)
        self.add(label)
        self.wait(1 / 30)
class RectangleFlash(Scene):
    def construct(self):
        target = Square().set_fill(WHITE, opacity=1).set_stroke(width=0)
        self.add(target)
        self.play(ShowPassingFlashAround(
            target, stroke_color=BLUE, stroke_width=8, buff=0.3,
            run_time=0.5, rate_func=linear,
        ))
        self.wait(0.125)
class Soundtrack(Scene):
    def construct(self):
        from pathlib import Path
        cue = Path(__file__).with_name("cue.wav")
        quiet = Path(__file__).with_name("quiet.wav")
        ticks = []
        marker = Dot()
        marker.add_updater(lambda mob, dt: ticks.append(dt))
        self.add(marker)
        self.wait(1 / 32)
        self.add_sound(str(cue))
        self.add_sound(str(quiet), time_offset=4 / 48000,
                       gain=-6.020599913279624, gain_to_background=-6.020599913279624)
        self.wait(1 / 32)
        assert any(dt > 0 for dt in ticks), "WAV route skipped sampled updaters"
class MissingSound(Soundtrack):
    def construct(self):
        super().construct()
        self.add_sound(__file__ + ".missing.wav")
class MalformedSound(Soundtrack):
    def construct(self):
        super().construct()
        self.add_sound(__file__)
class OversizedSound(Soundtrack):
    def construct(self):
        from pathlib import Path
        self.add_sound(str(Path(__file__).with_name("cue.wav")), time_offset=3601)
class FailingSound(Soundtrack):
    def construct(self):
        super().construct()
        raise RuntimeError("soundtrack-after-composition")
class VideoSoundtrack(Scene):
    def construct(self):
        from pathlib import Path
        square = Square(side_length=1, fill_color=WHITE, fill_opacity=1, stroke_width=0)
        self.add(square.shift(UP))
        self.wait(1 / 4)
        # The encoder has already accepted frames before this cue exists.
        self.add_sound(str(Path(__file__).with_name("tone.wav")))
        self.wait(1 / 4)
""")

reports, artifacts = {}, {}
publication_failures = thread_replays = 0
for format_name in ("gif", "y4m"):
    destination = output_root / ("motion." + format_name)
    arguments = (str(source), "MovingWhiteSquare", "--format", format_name,
                 "--resolution", "96x54", "--fps", "30", "--threads", "1")
    code, report = console(*arguments, "--video_dir", str(destination))
    assert code == 0, report
    # BN-02: binary f64 0.1 is above 1/10, so the fixed 30 fps clock emits
    # four samples to cover it. The fourth holds the clamped final state.
    assert report["format"] == format_name and report["frame_count"] == 4, report
    assert report["resolution"] == [96, 54] and report["fps"] == 30
    payload = destination.read_bytes()
    assert report["bytes"] == len(payload)
    assert report["digest"] == hashlib.sha256(payload).hexdigest()  # ubs:ignore — public artifact checksum, not authentication or secret material.
    reports[format_name], artifacts[format_name] = report, destination
    # A second render at a different thread count must retain actual bytes.
    repeated = output_root / ("repeat." + format_name)
    code, repeat = console(*arguments, "--threads", "4", "--video_dir", str(repeated))
    assert code == 0 and repeat["frame_count"] == 4, repeat
    assert repeated.read_bytes() == payload
    thread_replays += 1
    # Fail after captured frames: neither a new artifact nor a partial overwrite.
    for exists in (False, True):
        failed = output_root / (f"failed-{exists}." + format_name)
        if exists:
            failed.write_bytes(b"preserve-existing-destination")
        code, failure = console(str(source), "FailingScene", *arguments[2:], "--video_dir", str(failed))
        assert code == 5 and failure["kind"] == "scene-execution-failed", failure
        assert "native-output-after-capture" in failure["message"]
        if exists:
            assert failed.read_bytes() == b"preserve-existing-destination"
        else:
            assert not failed.exists()
        publication_failures += 1

def verify_motion(gif_frames, y4m_frames, delays):
    assert len(gif_frames) == len(y4m_frames) == 4
    assert delays == [3, 4, 3, 3], delays  # Rational 30 fps in centiseconds.
    centers = []
    for rgb, luma in zip(gif_frames, y4m_frames):
        assert np.array_equal(rgb[:, :, 0], rgb[:, :, 1])
        assert np.array_equal(rgb[:, :, 1], rgb[:, :, 2])
        expected_luma = 16 + rgb[:, :, 0].astype(float) * 219 / 255
        assert np.max(np.abs(luma.astype(float) - expected_luma)) <= 1.1
        ys, xs = np.nonzero(luma > 200)
        assert len(xs) > 10
        assert ys.mean() < 54 / 2 - 3, "output was vertically reflected"
        centers.append(float(xs.mean()))
    assert centers[0] < 48 < centers[2], centers
    assert centers[1] - centers[0] > 5 and centers[2] - centers[1] > 5, centers
    assert np.array_equal(gif_frames[2], gif_frames[3])
    assert np.array_equal(y4m_frames[2], y4m_frames[3])
    return centers


gif_frames, delays = read_gif(artifacts["gif"])
y4m_planes = read_y4m(artifacts["y4m"])
# The motion witness is achromatic in both planar chroma planes.
assert all(np.all(cb == 128) and np.all(cr == 128) for _, cb, cr in y4m_planes)
y4m_frames = [luma for luma, _, _ in y4m_planes]
centers = verify_motion(gif_frames, y4m_frames, delays)
# The witness must reject plausible wrong output even when the formats agree:
# a shared vertical reflection and a shared reversal of frame publication.
for wrong_gif, wrong_y4m in (
    ([frame[::-1] for frame in gif_frames], [frame[::-1] for frame in y4m_frames]),
    (gif_frames[::-1], y4m_frames[::-1]),
):
    try:
        verify_motion(wrong_gif, wrong_y4m, delays)
    except AssertionError:
        pass
    else:
        raise AssertionError("motion witness accepted plausible wrong output")

# White motion alone cannot detect swapped RGB or chroma channels. Saturated
# primary interiors have these independent limited-range BT.709 code values.
for format_name in ("gif", "y4m"):
    code, primary_report = console(
        str(source), "PrimarySwatches", "--format", format_name,
        "--resolution", "96x54", "--fps", "30",
        "--video_dir", str(output_root / ("primaries." + format_name)),
    )
    assert code == 0 and primary_report["frame_count"] == 1, primary_report
primary_gif, _ = read_gif(output_root / "primaries.gif")
primary_y4m = read_y4m(output_root / "primaries.y4m")
assert len(primary_gif) == len(primary_y4m) == 1
primary_rgb = primary_gif[0]
primary_luma, primary_cb, primary_cr = primary_y4m[0]
for color, codes, x_bounds in (
    ((255, 0, 0), (63, 102, 240), (0, 32)),
    ((0, 255, 0), (173, 42, 26), (32, 64)),
    ((0, 0, 255), (32, 240, 118), (64, 96)),
):
    _, primary_xs = np.nonzero(np.all(primary_rgb == color, axis=2))
    assert len(primary_xs) > 32 and x_bounds[0] < primary_xs.mean() < x_bounds[1], (
        "primary color moved from its authored position", color,
    )
    interior_blocks = 0
    for y in range(2, 52, 2):
        for x in range(2, 94, 2):
            # Stay inside constant-color neighborhoods, away from AA and
            # chroma-siting boundaries; compare decoded planes, not metadata.
            if not np.all(primary_rgb[y - 1:y + 3, x - 1:x + 3] == color):
                continue
            observed = (primary_luma[y, x], primary_cb[y // 2, x // 2], primary_cr[y // 2, x // 2])
            assert all(abs(int(actual) - expected) <= 1 for actual, expected in zip(observed, codes)), (color, observed)
            interior_blocks += 1
    assert interior_blocks >= 8, (color, interior_blocks)

# NV12's even geometry is enforced without silently changing the request.
odd_destination = output_root / "odd.y4m"
code, odd = console(str(source), "MovingWhiteSquare", "--format", "y4m",
                    "--resolution", "95x54", "--video_dir", str(odd_destination))
assert code == 6 and odd["kind"] == "render-start-failed", odd
assert not odd_destination.exists()
code, unsupported = console("missing-source.py", "--format", "video")
assert code == 4 and unsupported["kind"] == "render-capability-unavailable", unsupported

# Independent PCM inputs make the expected placement and overlap analytic:
# 1/32 second is sample 1500 at 48 kHz; the second cue starts four samples later.
for name, amplitude in (("cue.wav", 8192), ("quiet.wav", 4096)):
    with wave.open(str(output_root / name), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(48000)
        output.writeframes(struct.pack("<16h", *([amplitude] * 16)))

wav_destination = output_root / "soundtrack.wav"
wav_arguments = (str(source), "Soundtrack", "--format", "wav", "--fps", "32", "--threads", "1")
code, wav_report = console(*wav_arguments, "--video_dir", str(wav_destination))
assert code == 0 and wav_report["engine"] == "native-sound-mixer", wav_report
assert wav_report["sample_frames"] == wav_report["frame_count"] == 3000, wav_report
assert wav_report["sample_rate"] == 48000 and wav_report["channels"] == 2
with wave.open(str(wav_destination), "rb") as soundtrack:
    assert (soundtrack.getnchannels(), soundtrack.getsampwidth(), soundtrack.getframerate()) == (2, 2, 48000)
    assert soundtrack.getnframes() == 3000
    samples = np.frombuffer(soundtrack.readframes(3000), dtype="<i2").reshape(3000, 2)
expected = np.zeros(3000, dtype=np.int32)
expected[1500:1504] = 8192
expected[1504:1516] = 6144
expected[1516:1520] = 2048
assert np.array_equal(samples[:, 0], samples[:, 1]), "mono cue was not duplicated to stereo"
assert np.max(np.abs(samples[:, 0].astype(np.int32) - expected)) <= 1, "cue placement or dB gain is wrong"
wav_bytes = wav_destination.read_bytes()
assert wav_report["bytes"] == len(wav_bytes)
assert wav_report["digest"] == hashlib.sha256(wav_bytes).hexdigest()  # ubs:ignore — public artifact checksum, not authentication or secret material.
wav_replay = output_root / "soundtrack-repeat.wav"
code, replay = console(*wav_arguments, "--threads", "4", "--video_dir", str(wav_replay))
assert code == 0 and replay["sample_frames"] == 3000, replay
assert wav_replay.read_bytes() == wav_bytes
reports["wav"] = wav_report
thread_replays += 1

for scene_name, failure_code, message in (
    ("MissingSound", 6, "missing.wav"),
    ("MalformedSound", 6, "unrecognized audio container"),
    ("OversizedSound", 6, "budget"),
    ("MovingWhiteSquare", 6, "at least one Scene.add_sound"),
    ("FailingSound", 5, "soundtrack-after-composition"),
):
    for exists in (False, True):
        failed = output_root / f"{scene_name}-{exists}.wav"
        if exists:
            failed.write_bytes(b"preserve-existing-soundtrack")
        code, failure = console(str(source), scene_name, *wav_arguments[2:], "--video_dir", str(failed))
        assert code == failure_code and message in failure["message"], failure
        if exists:
            assert failed.read_bytes() == b"preserve-existing-soundtrack"
        else:
            assert not failed.exists()
        publication_failures += 1

camera_destination = output_root / "camera-pan.gif"
code, camera_report = console(str(source), "CameraPan", "--format", "gif",
                              "--resolution", "96x54", "--fps", "8", "--threads", "1",
                              "--video_dir", str(camera_destination))
assert code == 0 and camera_report["frame_count"] == 3, camera_report
camera_frames, _ = read_gif(camera_destination)
assert len(camera_frames) == 3
camera_centers = []
for camera_frame in camera_frames:
    camera_y, camera_x = np.nonzero(camera_frame.mean(axis=2) > 200)
    assert len(camera_x) > 10
    camera_centers.append(float(camera_x.mean()))
assert camera_centers[0] > camera_centers[1] > camera_centers[2], camera_centers
assert camera_centers[0] - camera_centers[-1] > 8, camera_centers

updater_destination = output_root / "camera-updater.gif"
code, updater_report = console(str(source), "CameraUpdater", "--format", "gif",
                               "--resolution", "96x54", "--fps", "8", "--threads", "1",
                               "--video_dir", str(updater_destination))
assert code == 0 and updater_report["frame_count"] == 3, updater_report
updater_frames, _ = read_gif(updater_destination)
assert len(updater_frames) == 3
updater_centers = []
for updater_frame in updater_frames:
    updater_y, updater_x = np.nonzero(updater_frame.mean(axis=2) > 200)
    assert len(updater_x) > 10
    updater_centers.append(float(updater_x.mean()))
assert np.allclose(np.diff(updater_centers), [-6.75, -6.75], atol=0.75), updater_centers

invalid_camera_destination = output_root / "invalid-camera.gif"
invalid_camera_destination.write_bytes(b"keep prior camera output")
code, invalid_camera_report = console(str(source), "InvalidCameraAfterCapture", "--format", "gif",
                                      "--resolution", "96x54", "--fps", "8", "--threads", "1",
                                      "--video_dir", str(invalid_camera_destination))
assert code == 6 and "background must be finite" in invalid_camera_report["message"], invalid_camera_report
assert invalid_camera_destination.read_bytes() == b"keep prior camera output"

background_destination = output_root / "background-changes.gif"
code, background_report = console(str(source), "BackgroundChanges", "--format", "gif",
                                  "--resolution", "96x54", "--fps", "8", "--threads", "1",
                                  "--video_dir", str(background_destination))
assert code == 0 and background_report["frame_count"] == 3, background_report
background_frames, _ = read_gif(background_destination)
assert len(background_frames) == 3
for background_frame, rgb in zip(background_frames, ([255, 0, 0], [0, 255, 0], [0, 0, 255])):
    assert np.array_equal(background_frame, np.broadcast_to(rgb, (54, 96, 3)))

light_samples = []
for light_scene, light_name in (("LightChanges", "light-changes.gif"),
                                ("OverriddenLightChanges", "overridden-light.gif")):
    light_destination = output_root / light_name
    code, light_report = console(str(source), light_scene, "--format", "gif",
                                 "--resolution", "96x54", "--fps", "8", "--threads", "1",
                                 "--video_dir", str(light_destination))
    assert code == 0 and light_report["frame_count"] == 2, light_report
    light_frames, _ = read_gif(light_destination)
    assert len(light_frames) == 2
    light_samples.append([float(frame[26:28, 47:49].mean()) for frame in light_frames])
assert light_samples[0][0] > 230 and light_samples[0][1] < 25, light_samples
assert np.allclose(light_samples[1], light_samples[0][::-1], atol=1), light_samples

markup_pngs = []
for markup_scene, markup_name in (("EscapedMarkup", "escaped-markup.png"),
                                   ("LiteralMarkup", "literal-markup.png")):
    markup_destination = output_root / markup_name
    code, markup_report = console(str(source), markup_scene, "--format", "png",
                                   "--resolution", "192x108", "--threads", "1",
                                   "--video_dir", str(markup_destination))
    assert code == 0 and markup_report["frame_count"] == 1, markup_report
    markup_pngs.append(markup_destination.read_bytes())
    markup_image = manimlib.ImageMobject(str(markup_destination))
    xs = ((np.arange(192) + 0.5) / 192 - 0.5) * markup_image.get_width()
    ys = ((np.arange(108) + 0.5) / 108 - 0.5) * markup_image.get_height()
    assert any(max(markup_image.point_to_rgb((x, y, 0))) > 0.5
               for y in ys for x in xs), "markup image contains no visible glyphs"
assert markup_pngs[0] == markup_pngs[1], "escaped markup changed literal text rendering"

centered_destination = output_root / "centered-typesetting.gif"
code, centered_report = console(str(source), "CenteredTypesetting", "--format", "gif",
                               "--resolution", "480x270", "--fps", "30", "--threads", "1",
                               "--video_dir", str(centered_destination))
assert code == 0 and centered_report["frame_count"] == 2, centered_report
centered_frames, _ = read_gif(centered_destination)
assert len(centered_frames) == 2
for centered_frame in centered_frames:
    rows, columns = np.nonzero(centered_frame.max(axis=2) > 32)
    assert len(rows) > 100, "centered typesetting contains no substantial visible glyphs"
    assert abs((columns.min() + columns.max() + 1) / 2 - 240) <= 2
    assert abs((rows.min() + rows.max() + 1) / 2 - 135) <= 2

rectangle_destination = output_root / "rectangle-flash.gif"
code, rectangle_report = console(str(source), "RectangleFlash", "--format", "gif",
                                 "--fps", "8", "--resolution", "192x108", "--threads", "1",
                                 "--video_dir", str(rectangle_destination))
assert code == 0 and rectangle_report["frame_count"] == 5, rectangle_report
rectangle_frames, rectangle_delays = read_gif(rectangle_destination)
assert len(rectangle_frames) == 5 and all(delay > 0 for delay in rectangle_delays)
rectangle_blue_counts = []
for frame in rectangle_frames:
    rgb = frame.astype(np.int16)
    rectangle_blue_counts.append(np.count_nonzero(
        (rgb[:, :, 2] > rgb[:, :, 0] + 40) & (rgb[:, :, 1] > rgb[:, :, 0] + 40)
    ))
    assert np.all(frame[54, 96] > 230), "flash removed or obscured its target"
assert max(rectangle_blue_counts[1:-1]) > 4, rectangle_blue_counts
assert rectangle_blue_counts[-1] == 0, "flash helper survived cleanup"

video_formats = video_frames = 0
ffmpeg = shutil.which("ffmpeg")
# Absence is always exercised, including on fully qualified encoder hosts.
prior_path = os.environ.get("PATH")
try:
    os.environ["PATH"] = str(output_root)
    missing_destination = output_root / "missing-encoder.mp4"
    missing_destination.write_bytes(b"preserve missing-encoder destination")
    code, missing_encoder = console(str(source), "MovingWhiteSquare", "--format", "mp4",
                                    "--resolution", "96x54", "--threads", "1",
                                    "--video_dir", str(missing_destination))
    assert code == 4 and missing_encoder["exit"]["identity"] == "capability", missing_encoder
    assert "ffmpeg" in missing_encoder["message"] and "native" in missing_encoder["message"]
    assert missing_destination.read_bytes() == b"preserve missing-encoder destination"
finally:
    if prior_path is None:
        os.environ.pop("PATH", None)
    else:
        os.environ["PATH"] = prior_path

if os.environ.get("FMN_REQUIRE_FFMPEG") == "1":
    assert ffmpeg, "required real ffmpeg acceptance cannot run without the encoder"

if ffmpeg:
    def decode_video(path, audio=False):
        output = (["-map", "0:a:0", "-ac", "2", "-ar", "48000", "-f", "s16le"]
                  if audio else ["-map", "0:v:0", "-frames:v", "16", "-pix_fmt", "rgb24", "-f", "rawvideo"])
        return subprocess.run([ffmpeg, "-v", "error", "-nostdin", "-i", str(path),
                               *output, "pipe:1"], capture_output=True, timeout=30, check=False)

    for format_name in ("mp4", "mov"):
        destination = output_root / ("motion." + format_name)
        arguments = (str(source), "MovingWhiteSquare", "--format", format_name,
                     "--resolution", "96x54", "--fps", "30", "--threads", "1")
        code, report = console(*arguments, "--video_dir", str(destination))
        assert code == 0 and report["frame_count"] == 4, report
        assert report["certified"] is False
        invocations = report["ffmpeg_invocations"]
        assert len(invocations) == 1, "no-cue video must not fabricate an audio track"
        assert invocations[0]["tool_sha256"] == hashlib.sha256(pathlib.Path(ffmpeg).read_bytes()).hexdigest()  # ubs:ignore — public executable fingerprint, not a secret or authentication token.
        assert invocations[0]["bound_tool_path"] != invocations[0]["tool_path"]
        assert invocations[0]["encoder"] == "libx264"
        assert invocations[0]["process_mechanism"] and invocations[0]["process_policy_version"] > 0
        decoded = decode_video(destination)
        assert decoded.returncode == 0, decoded.stderr
        frames = np.frombuffer(decoded.stdout, dtype=np.uint8).reshape(-1, 54, 96, 3)
        assert len(frames) == 4
        video_centers = []
        for frame in frames:
            ys, xs = np.nonzero(frame.mean(axis=2) > 200)
            assert len(xs) > 10 and ys.mean() < 54 / 2 - 3
            video_centers.append(float(xs.mean()))
        assert video_centers[0] < 48 < video_centers[2]
        assert video_centers[1] - video_centers[0] > 5
        assert video_centers[2] - video_centers[1] > 5
        assert decode_video(destination, audio=True).returncode != 0
        for scene_name, expected_code in (("FailingScene", 5), ("MissingSound", 6)):
            failed = output_root / f"{scene_name}.{format_name}"
            failed.write_bytes(b"preserve existing encoded video")
            code, error = console(str(source), scene_name, *arguments[2:], "--video_dir", str(failed))
            assert code == expected_code, error
            assert failed.read_bytes() == b"preserve existing encoded video"
        video_formats += 1
        video_frames += len(frames)

    tone = (np.sin(2 * np.pi * 1000 * np.arange(6000) / 48000) * 8192).astype("<i2")
    with wave.open(str(output_root / "tone.wav"), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(48000)
        output.writeframes(tone.tobytes())
    destination = output_root / "soundtrack.mp4"
    code, report = console(str(source), "VideoSoundtrack", "--format", "mp4",
                           "--resolution", "96x54", "--fps", "8", "--threads", "1",
                           "--video_dir", str(destination))
    assert code == 0 and report["frame_count"] == 4, report
    assert len(report["ffmpeg_invocations"]) == 2
    first, mux = report["ffmpeg_invocations"]
    assert first["bound_tool_path"] == mux["bound_tool_path"]
    assert any(pair == ["-c:v", "copy"] for pair in (mux["argv"][i:i+2] for i in range(len(mux["argv"]) - 1)))
    decoded = decode_video(destination, audio=True)
    assert decoded.returncode == 0, decoded.stderr
    audio = np.frombuffer(decoded.stdout, dtype="<i2").reshape(-1, 2).astype(float)
    # AAC is lossy and can retain a padded final packet. Check the actual
    # signal and call-site timing, never encoded byte identity or exact PCM.
    assert 24000 <= len(audio) < 25024
    assert np.sqrt(np.mean(audio[1000:10000] ** 2)) < 32
    assert np.sqrt(np.mean(audio[13000:17000] ** 2)) > 4000
    assert np.sqrt(np.mean(audio[20000:23000] ** 2)) < 32
    correlation = np.corrcoef(audio[13000:17000, 0], tone[1000:5000].astype(float))[0, 1]
    assert correlation > 0.95, "late scene cue lost its sample-clock position"

    # Certified portal reproducibility and provenance manifest verification (§16.7, docs/INPUT_CLOSURE.md)
    native = getattr(manimlib, "_native", manimlib)
    extension = getattr(native, "__file__", None)
    has_installed_wheel = bool(
        extension and any(str(extension).endswith(suffix) for suffix in importlib.machinery.EXTENSION_SUFFIXES)
    )
    if has_installed_wheel:
        for cert_format in ("png", "wav"):
            cert_dest = output_root / f"certified_scene.{cert_format}"
            scene_name = "Soundtrack" if cert_format == "wav" else "MovingWhiteSquare"
            cert_args = (
                str(source), scene_name, "--format", cert_format,
                "--reproducible", "--video_dir", str(cert_dest),
            )
            code, cert_report = console(*cert_args)
            assert code == 0, cert_report
            assert cert_report.get("manifest") is not None, cert_report
            assert cert_report.get("closure_digest") is not None, cert_report
            manifest_dir = cert_dest.parent / (cert_dest.name + ".manifest")
            assert manifest_dir.is_dir(), manifest_dir
            manifest_fmnp = manifest_dir / "manifest.fmnp"
            manifest_txt = manifest_dir / "manifest.txt"
            assert manifest_fmnp.is_file(), manifest_fmnp
            assert manifest_txt.is_file(), manifest_txt
            txt_content = manifest_txt.read_text()
            assert cert_report["closure_digest"] in txt_content

            # Second run to a different destination with same source produces bit-identical output
            cert_dest2 = output_root / f"certified_scene_2.{cert_format}"
            code2, cert_report2 = console(
                str(source), scene_name, "--format", cert_format,
                "--reproducible", "--video_dir", str(cert_dest2),
            )
            assert code2 == 0, cert_report2
            assert cert_dest.read_bytes() == cert_dest2.read_bytes(), f"certified {cert_format} output drifted across runs"
            assert cert_report["closure_digest"] == cert_report2["closure_digest"], "closure digest drifted"

            # Preflight no-clobber refusal if manifest destination exists
            code3, err_report3 = console(*cert_args)
            assert code3 in (2, 4), err_report3
    else:
        # Without an installed wheel payload, certified portal runs must fail closed
        # with capability code 4 rather than certifying synthetic or unhashed binaries.
        for cert_format in ("png", "wav"):
            cert_dest = output_root / f"certified_scene.{cert_format}"
            scene_name = "Soundtrack" if cert_format == "wav" else "MovingWhiteSquare"
            code, cert_report = console(
                str(source), scene_name, "--format", cert_format,
                "--reproducible", "--video_dir", str(cert_dest),
            )
            assert code == 4, cert_report
            assert cert_report["kind"] == "render-capability-unavailable", cert_report
            assert "portal runtime input closure unavailable" in cert_report["message"], cert_report

    # Capability refusal: video format with --reproducible must fail closed
    bad_video_dest = output_root / "bad_cert.mp4"
    code_bad, bad_report = console(
        str(source), "MovingWhiteSquare", "--format", "mp4",
        "--reproducible", "--video_dir", str(bad_video_dest),
    )
    assert code_bad == 4, bad_report

    # Reproducible batches (a8d883dc) give every completed scene its own
    # manifest and keep the aggregate uncertified. Scenes run in sorted order,
    # so FailingScene stops the batch (exit 5) after the scenes before it.
    batch_root = output_root / "batch_out"
    code_batch, batch_report, batch_progress = batch_console(
        str(source), "--write_all", "--format", "png",
        "--reproducible", "--video_dir", str(batch_root),
    )
    if has_installed_wheel:
        batch = batch_report["batch"]
        assert code_batch == 5, batch_report
        assert batch["reproducible_requested"] and not batch["certified"], batch_report
        statuses = {outcome["name"]: outcome["status"] for outcome in batch["outcomes"]}
        assert statuses["FailingScene"] == "failed", statuses
        done = [name for name, status in statuses.items() if status == "succeeded"]
        assert done and done == sorted(done) and all(name < "FailingScene" for name in done), statuses
        assert all(status == "not_run" for name, status in statuses.items() if name > "FailingScene")
        for name in done:
            sidecar = batch_root / (name + ".png.manifest")
            assert (batch_root / (name + ".png")).is_file(), name
            assert (sidecar / "manifest.fmnp").is_file() and (sidecar / "manifest.txt").is_file(), name
        assert batch_progress == [
            f"fmn-python: {outcome['name']}: {outcome['status']}: {outcome['destination']}"
            for outcome in batch["outcomes"] if outcome["status"] != "not_run"
        ], batch_progress
    else:
        # Without an installed wheel payload nothing may be certified.
        assert code_batch != 0, batch_report
        assert not (batch_report.get("batch") or {}).get("all_scenes_certified"), batch_report

native_output_report = {"frames": len(gif_frames), "formats": len(reports),
                        "publication_failures": publication_failures,
                        "thread_replays": thread_replays, "centers": centers,
                        "sample_frames": samples.shape[0], "video_formats": video_formats,
                        "video_frames": video_frames, "video_capability_refusals": 1}
