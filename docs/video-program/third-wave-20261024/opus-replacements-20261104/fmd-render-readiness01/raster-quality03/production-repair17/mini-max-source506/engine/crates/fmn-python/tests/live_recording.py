"""Native populated-scene recording: no replacement arena, clock or encoder."""
from pathlib import Path
import hashlib
import struct
import tempfile
import wave

import manimlib


def begin(scene, path, format="y4m", threads=1):
    fps, frame = manimlib._portal_scene_clock(scene)
    actual = manimlib._portal_begin_recording(scene, str(path), format, 96, 54, fps, threads)
    assert actual == frame
    assert manimlib._portal_scene_clock(scene) == (fps, frame)
    return frame


def finish(scene, path):
    receipt = scene._finish_render()
    assert Path(receipt[0]) == path
    if path.is_file():
        data = path.read_bytes()
        assert receipt[2] == len(data)
        assert receipt[3] == hashlib.sha256(data).hexdigest()
    return receipt


def y4m_centers(path):
    header, body = path.read_bytes().split(b"\n", 1)
    assert header == b"YUV4MPEG2 W96 H54 F30:1 Ip A1:1 C420mpeg2"
    size = 96 * 54 * 3 // 2
    assert len(body) % (size + 6) == 0
    centers = []
    for offset in range(0, len(body), size + 6):
        assert body[offset:offset + 6] == b"FRAME\n"
        luma = body[offset + 6:offset + 6 + 96 * 54]
        xs = [i % 96 for i, value in enumerate(luma) if value > 200]
        assert len(xs) > 10
        centers.append(sum(xs) / len(xs))
    return centers


def populated_scene_and_repeated_clips(root):
    scene = manimlib.Scene()
    square = manimlib.Square(side_length=1, fill_color=manimlib.WHITE, fill_opacity=1, stroke_width=0)
    square.shift(2 * manimlib.LEFT)
    scene.add(square)
    scene.wait(0.125)  # Binary-exact duration: ceil(30/8) = four samples.
    view = square.data
    first = root / "first.y4m"
    assert begin(scene, first) == 4, manimlib._portal_scene_clock(scene)
    assert scene.mobjects[0] is square
    view["point"][:, 0] += 0.25
    assert abs(square.get_center()[0] + 1.75) < 1e-5
    scene.play(square.animate.shift(3 * manimlib.RIGHT), run_time=0.125, rate_func=manimlib.linear)
    before = manimlib._portal_scene_clock(scene)
    receipt = finish(scene, first)
    assert receipt[1] == 4, receipt
    assert manimlib._portal_scene_clock(scene) == before == (30, 8)
    xs = y4m_centers(first)
    assert len(xs) == 4 and all(b > a + 3 for a, b in zip(xs, xs[1:])), xs
    scene.wait(0.125)
    second = root / "second.y4m"
    assert begin(scene, second, threads=4) == 12
    scene.play(square.animate.shift(3 * manimlib.LEFT), run_time=0.125, rate_func=manimlib.linear)
    assert finish(scene, second)[1] == 4
    xs = y4m_centers(second)
    assert all(b < a - 3 for a, b in zip(xs, xs[1:])), xs
    assert scene.mobjects[0] is square
    assert manimlib._portal_scene_clock(scene) == (30, 16)


def thread_replay_and_native_formats(root):
    outputs = []
    for workers in (1, 4):
        scene = manimlib.Scene()
        scene.add(manimlib.Square(fill_opacity=1))
        scene.wait(0.125)
        path = root / f"replay-{workers}.y4m"
        begin(scene, path, threads=workers)
        scene.wait(0.125)
        finish(scene, path)
        outputs.append(path.read_bytes())
    assert outputs[0] == outputs[1]
    for format in ("gif", "png_sequence"):
        scene = manimlib.Scene()
        scene.add(manimlib.Square(fill_opacity=1))
        scene.wait(0.125)
        path = root / format
        begin(scene, path, format)
        scene.wait(0.125)
        assert finish(scene, path)[1] == 4
        if format == "gif":
            assert path.read_bytes().startswith(b"GIF89a")
        else:
            frames = sorted(path.glob("*.png"))
            assert len(frames) == 4
            assert all(frame.read_bytes().startswith(b"\x89PNG\r\n\x1a\n") for frame in frames)


def audio_is_relative_and_excludes_earlier_cues(root):
    source = root / "cue.wav"
    with wave.open(str(source), "wb") as stream:
        stream.setnchannels(1)
        stream.setsampwidth(2)
        stream.setframerate(48000)
        stream.writeframes(struct.pack("<hh", 4000, -4000) * 800)
    scene = manimlib.Scene()
    scene.add_sound(str(root / "earlier-missing.wav"))
    scene.wait(1)
    path = root / "insert.wav"
    assert begin(scene, path, "wav") == 30
    scene.wait(0.5)
    scene.add_sound(str(source))
    scene.wait(0.5)
    receipt = finish(scene, path)
    assert receipt[1] == 48000, receipt
    with wave.open(str(path), "rb") as stream:
        assert stream.getframerate() == 48000 and stream.getnchannels() == 2
        samples = struct.unpack("<" + "h" * (stream.getnframes() * 2), stream.readframes(stream.getnframes()))
    assert all(value == 0 for value in samples[:24000 * 2])
    assert any(value != 0 for value in samples[24000 * 2:25600 * 2])
    inputs = scene._render_audio_inputs
    assert len(inputs) == 1 and inputs[0]["decoder"] == "native-wav"
    assert not scene._render_invocations
    assert manimlib._portal_scene_clock(scene) == (30, 60)


def cancellation_and_ownership(root):
    scene = manimlib.Scene()
    square = manimlib.Square(fill_opacity=1)
    scene.add(square)
    scene.wait(0.125)
    path = root / "cancelled.y4m"
    begin(scene, path)
    scene.wait(0.125)
    try:
        begin(scene, root / "nested.y4m")
    except RuntimeError:
        pass
    else:
        raise AssertionError("nested generation was admitted")
    scene._abort_render()
    assert not path.exists()
    assert scene.mobjects[0] is square and manimlib._portal_scene_clock(scene) == (30, 8)
    after = root / "after-abort.y4m"
    begin(scene, after)
    scene.wait(0.125)
    assert finish(scene, after)[1] == 4
    previous = after.read_bytes()
    try:
        begin(scene, after)
        scene.wait(0.125)
        finish(scene, after)
    except Exception:
        scene._abort_render()
    else:
        raise AssertionError("existing recording was overwritten")
    assert after.read_bytes() == previous
    for format, fps, threads in (("png", 30, 1), ("svg", 30, 1), ("y4m", 8, 1), ("y4m", 30, 0)):
        before = manimlib._portal_scene_clock(scene)
        destination = root / f"refused-{format}-{fps}-{threads}"
        try:
            manimlib._portal_begin_recording(scene, str(destination), format, 96, 54, fps, threads)
        except (ValueError, RuntimeError):
            pass
        else:
            raise AssertionError("invalid recording was admitted")
        assert manimlib._portal_scene_clock(scene) == before and not destination.exists()


def main():
    with tempfile.TemporaryDirectory(prefix="fmn-live-recording-") as temp:
        root = Path(temp)
        for case in (populated_scene_and_repeated_clips, thread_replay_and_native_formats,
                     audio_is_relative_and_excludes_earlier_cues, cancellation_and_ownership):
            case_root = root / case.__name__
            case_root.mkdir()
            case(case_root)
    print("native live recording acceptance passed: 4 cases")


main()
