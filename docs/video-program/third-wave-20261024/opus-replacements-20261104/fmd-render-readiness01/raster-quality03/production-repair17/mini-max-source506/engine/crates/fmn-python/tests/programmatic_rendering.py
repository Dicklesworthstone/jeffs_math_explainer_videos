"""Installed-wheel programmatic output acceptance; no substitute native engine."""
import hashlib
import pathlib
import struct
import tempfile
import wave
import zlib

import numpy as np
import manimlib as m
from fmn_python import render_scene, render_session


def read_png(path):
    payload = pathlib.Path(path).read_bytes()
    assert payload[:8] == b"\x89PNG\r\n\x1a\n"
    offset, packed, dimensions = 8, bytearray(), None
    ended = False
    while offset < len(payload):
        size = int.from_bytes(payload[offset:offset + 4], "big")
        assert offset + 12 + size <= len(payload)
        kind = payload[offset + 4:offset + 8]
        body = payload[offset + 8:offset + 8 + size]
        crc = int.from_bytes(payload[offset + 8 + size:offset + 12 + size], "big")
        assert zlib.crc32(kind + body) & 0xffffffff == crc
        offset += 12 + size
        if kind == b"IHDR":
            assert dimensions is None
            width, height, depth, color, compression, filtering, interlace = struct.unpack(">IIBBBBB", body)
            assert (depth, color, compression, filtering, interlace) == (8, 6, 0, 0, 0)
            dimensions = (width, height)
        elif kind == b"IDAT":
            packed.extend(body)
        elif kind == b"IEND":
            assert size == 0 and offset == len(payload)
            ended = True
            break
    assert ended and dimensions is not None
    width, height = dimensions
    raw = zlib.decompress(packed)
    stride = width * 4
    assert len(raw) == height * (stride + 1)
    decoded = bytearray(height * stride)
    for y in range(height):
        mode = raw[y * (stride + 1)]
        assert mode in (0, 1, 2, 3, 4)
        row = raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)]
        for x, value in enumerate(row):
            left = decoded[y * stride + x - 4] if x >= 4 else 0
            up = decoded[(y - 1) * stride + x] if y else 0
            corner = decoded[(y - 1) * stride + x - 4] if y and x >= 4 else 0
            estimate = left + up - corner
            paeth = min((left, up, corner), key=lambda candidate: abs(estimate - candidate))
            predictor = (0, left, up, (left + up) // 2, paeth)[mode]
            decoded[y * stride + x] = (value + predictor) & 255
    return np.frombuffer(decoded, dtype=np.uint8).reshape(height, width, 4)


def y4m_frames(path):
    header, payload = pathlib.Path(path).read_bytes().split(b"\n", 1)
    assert header == b"YUV4MPEG2 W96 H54 F8:1 Ip A1:1 C420mpeg2", header
    size = 96 * 54 * 3 // 2
    assert len(payload) % (size + 6) == 0
    frames = []
    for offset in range(0, len(payload), size + 6):
        assert payload[offset:offset + 6] == b"FRAME\n"
        frames.append(np.frombuffer(payload[offset + 6:offset + 6 + 96 * 54], dtype=np.uint8).reshape(54, 96))
    return frames


def receipt_matches_file(result):
    payload = result.destination.read_bytes()
    assert len(payload) == result.bytes
    assert hashlib.sha256(payload).hexdigest() == result.digest
    assert result.certified is False


root = pathlib.Path(tempfile.mkdtemp(prefix="fmn-programmatic-render-"))


class MovingSquare(m.Scene):
    default_camera_config = dict(resolution=(96, 54), fps=8)

    def construct(self):
        square = m.Square(side_length=1, fill_color=m.WHITE, fill_opacity=1, stroke_width=0)
        square.shift(2 * m.LEFT + m.UP)
        self.add(square)
        self.wait(1 / 8)
        self.play(square.animate.shift(4 * m.RIGHT), run_time=1 / 4, rate_func=m.linear)


scene = MovingSquare()
frame = scene.frame
result = scene.render(root / "motion.y4m", threads=1)
assert result.frame_count == 3 and result.fps == 8 and result.resolution == (96, 54)
assert scene.frame is frame and scene.render_result is result
receipt_matches_file(result)
frames = y4m_frames(result.destination)
assert len(frames) == 3
centers = []
for luma in frames:
    ys, xs = np.nonzero(luma > 200)
    assert len(xs) > 10 and ys.mean() < 24
    centers.append(float(xs.mean()))
assert centers[0] < centers[1] < centers[2]
assert centers[1] - centers[0] > 5 and centers[2] - centers[1] > 5

# A second class instance has its own Stage, generation, clock and buffers.
repeat = render_scene(MovingSquare, root / "repeat.y4m", threads=4)
receipt_matches_file(repeat)
assert repeat.destination.read_bytes() == result.destination.read_bytes()

sequence = render_scene(MovingSquare, root / "sequence", threads=1)
files = sorted(sequence.destination.glob("*.png"))
assert sequence.format == "png_sequence" and len(files) == sequence.frame_count == 3
images = [read_png(path) for path in files]
assert all(image.shape == (54, 96, 4) for image in images)
assert not np.array_equal(images[0], images[-1])

still = render_scene(MovingSquare, root / "still.png", threads=1)
assert still.frame_count == 1
receipt_matches_file(still)
assert read_png(still.destination)[:, :, :3].max() > 200

gif = render_scene(MovingSquare, root / "motion.gif", threads=1)
assert gif.frame_count == 3 and gif.destination.read_bytes()[:6] == b"GIF89a"
receipt_matches_file(gif)

# Manual Scene use records on the native clock without invoking run/construct.
imperative = m.Scene(camera_config=dict(resolution=(96, 54), fps=8))
with render_session(imperative, root / "imperative.y4m", threads=1) as session:
    imperative.add(m.Square(fill_opacity=1))
    imperative.wait(1 / 4)
assert session.result.frame_count == 2
assert len(y4m_frames(session.result.destination)) == 2

configured = MovingSquare(file_writer_config=dict(
    save_last_frame=True, output_directory=str(root / "configured"), file_name="chosen",
))
configured_result = configured.render(threads=1)
assert configured_result.destination == root / "configured/chosen.png"
assert configured_result.format == "png"
receipt_matches_file(configured_result)


class Failing(MovingSquare):
    def construct(self):
        super().construct()
        raise RuntimeError("failure-after-real-capture")

    def tear_down(self):
        self.torn_down = True


failing = Failing()
failed = root / "not-published.y4m"
try:
    failing.render(failed, threads=1)
except RuntimeError as error:
    assert "failure-after-real-capture" in str(error)
else:
    raise AssertionError("failing construction was accepted")
assert failing.torn_down and not failed.exists()
assert not hasattr(failing, "_fmn_owned_render_session")

existing = root / "preserved.png"
existing.write_bytes(b"preexisting-destination")
try:
    render_scene(MovingSquare, existing, threads=1)
except Exception:
    pass
else:
    raise AssertionError("native publication overwrote an existing destination")
assert existing.read_bytes() == b"preexisting-destination"


class Early(MovingSquare):
    def construct(self):
        self.add(m.Square(fill_color=m.WHITE, fill_opacity=1))
        raise m.EndScene("normal stop")


early = render_scene(Early, root / "early.png", threads=1)
assert early.frame_count == 1
receipt_matches_file(early)
assert read_png(early.destination)[:, :, :3].max() > 200

tone = root / "tone.wav"
with wave.open(str(tone), "wb") as output:
    output.setparams((1, 2, 48000, 0, "NONE", "not compressed"))
    output.writeframes(struct.pack("<h", 1000) * 4800)


class Sound(m.Scene):
    default_camera_config = dict(resolution=(96, 54), fps=8)

    def construct(self):
        self.add_sound(str(tone))
        self.wait(1 / 4)


audio = render_scene(Sound, root / "soundtrack.wav", threads=1)
receipt_matches_file(audio)
assert audio.frame_count is None and audio.sample_frames == 12000
with wave.open(str(audio.destination), "rb") as decoded:
    assert (decoded.getnchannels(), decoded.getframerate(), decoded.getnframes()) == (2, 48000, audio.sample_frames)
    assert any(decoded.readframes(decoded.getnframes()))

print(f"programmatic native output acceptance passed: 11 cases; artifacts retained at {root}")
