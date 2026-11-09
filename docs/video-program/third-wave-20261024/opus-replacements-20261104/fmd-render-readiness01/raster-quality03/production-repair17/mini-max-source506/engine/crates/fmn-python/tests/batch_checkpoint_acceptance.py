"""Installed-wheel recovery acceptance using real native output and CLI sessions."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import sys
import struct
import tempfile
import traceback
import wave

import manimlib
from fmn_python import BatchRenderError, render_scenes
from fmn_python.__main__ import main

CALLS = []
FAIL = True
SOUND = None


class Motion(manimlib.Scene):
    def construct(self):
        CALLS.append("motion")
        if SOUND is not None:
            self.add_sound(str(SOUND))
        square = manimlib.Square(side_length=1, fill_color=manimlib.WHITE,
                                fill_opacity=1, stroke_width=0)
        self.add(square)
        self.wait(1 / 8)
        self.play(square.animate.shift(2 * manimlib.RIGHT), run_time=1 / 4,
                  rate_func=manimlib.linear)


class Recoverable(Motion):
    def construct(self):
        CALLS.append("recoverable")
        super().construct()
        if FAIL:
            raise RuntimeError("fail after native frame capture")


def inventory(path):
    files = sorted(path.rglob("*")) if path.is_dir() else [path]
    return [(str(file), file.stat().st_mtime_ns, hashlib.sha256(file.read_bytes()).hexdigest())
            for file in files if file.is_file()]


def real_output_resume(root, format, *, save_last_frame=False):
    global FAIL, SOUND
    FAIL = True
    SOUND = None
    root.mkdir(parents=True)
    if format == "wav":
        # WAV publication deliberately refuses scenes without add_sound cues.
        # This is a test input fixture, not an alternative output encoder.
        SOUND = root / "source.wav"
        with wave.open(str(SOUND), "wb") as stream:
            stream.setnchannels(1)
            stream.setsampwidth(2)
            stream.setframerate(48000)
            stream.writeframes(struct.pack("<hh", 4000, -4000) * 3000)
    CALLS.clear()
    jobs = {"first": Motion, "recover": Recoverable, "last": Motion}
    options = dict(format=format, resolution=(96, 54), fps=8, threads=1,
                   checkpoint=root / "progress.json", resume_key="native-fixture-v1",
                   **({"save_last_frame": True} if save_last_frame else {}))
    try:
        render_scenes(jobs, root / "output", **options)
    except BatchRenderError as error:
        partial = error.result
    else:
        raise AssertionError("authored failure did not fail the batch")
    assert [row.status for row in partial.outcomes] == ["succeeded", "failed", "not_run"], (format, partial.as_dict())
    first = partial.outcomes[0]
    receipt = first.result.primary if save_last_frame else first.result
    before_still = inventory(first.result.still_destination) if save_last_frame else None
    if save_last_frame:
        assert first.result.completed and first.result.still.frame_count == 1
        assert before_still and first.result.still.digest == before_still[0][2]
    before = inventory(first.destination)
    assert before, "expected a real native publication"
    if format != "png_sequence":
        assert receipt.digest == before[0][2]
    if format == "wav":
        assert receipt.sample_frames > 0
        assert len(receipt.audio_inputs) == 1
        assert receipt.audio_inputs[0]["decoder"] == "native-wav"
        assert receipt.audio_inputs[0]["source_sha256"] == hashlib.sha256(SOUND.read_bytes()).hexdigest()
        assert not receipt.ffmpeg_invocations
    FAIL = False
    CALLS.clear()
    result = render_scenes(jobs, root / "output", resume=True, **options)
    assert result.ok and CALLS == ["recoverable", "motion", "motion"], CALLS
    assert inventory(first.destination) == before, "completed output was rewritten"
    if save_last_frame:
        assert inventory(first.result.still_destination) == before_still, "final PNG was rewritten"
    assert result.outcomes[0].result.as_dict() == first.result.as_dict()
    CALLS.clear()
    replay = render_scenes(jobs, root / "output", resume=True, **options)
    assert replay.as_dict() == result.as_dict() and CALLS == []
    assert not replay.as_dict()["certified"]


def invoke(arguments):
    previous = sys.argv
    output, diagnostics = io.StringIO(), io.StringIO()
    try:
        sys.argv = ["fmn-python", "--robot", *map(str, arguments)]
        with contextlib.redirect_stdout(output), contextlib.redirect_stderr(diagnostics):
            code = main()
    finally:
        sys.argv = previous
    report = json.loads(output.getvalue())
    assert report["exit"]["code"] == code
    return code, report


def real_cli_resume(root, *, save_last_frame=False):
    root.mkdir(parents=True)
    source, marker, ready = root / "scenes.py", root / "calls.txt", root / "ready"
    source.write_text(f'''from manimlib import *
from pathlib import Path
marker = Path({str(marker)!r})
ready = Path({str(ready)!r})
class A(Scene):
    def construct(self):
        with marker.open("a") as stream: stream.write("A\\n")
        self.add(Square(fill_opacity=1))
class B(Scene):
    def construct(self):
        with marker.open("a") as stream: stream.write("B\\n")
        self.add(Circle(fill_opacity=1))
        if not ready.exists(): raise RuntimeError("retry me")
''')
    arguments = [source, "--write_all", "--format", "png", "--resolution", "96x54", "--fps", "8", "--threads", "1",
                 "--video_dir", root / "output", "--checkpoint", root / "progress.json", "--resume-key", "cli-native-v1"]
    if save_last_frame:
        arguments[arguments.index("--format") + 1] = "gif"
        arguments.append("--save-last-frame")
    code, report = invoke(arguments)
    assert code == 5 and report["batch"]["counts"]["succeeded"] == 1, report
    original = inventory(root / "output/A.png")
    movie = inventory(root / "output/A.gif") if save_last_frame else None
    ready.write_text("retry failed B only")
    code, report = invoke([*arguments, "--resume"])
    assert code == 0 and report["batch"]["counts"]["succeeded"] == 2, report
    assert marker.read_text().splitlines() == ["A", "B", "B"]
    assert inventory(root / "output/A.png") == original
    if save_last_frame:
        assert inventory(root / "output/A.gif") == movie
        assert report["batch"]["outcomes"][0]["result"]["completed"] is True
    code, report = invoke([*arguments, "--resume"])
    assert code == 0 and marker.read_text().splitlines() == ["A", "B", "B"], report
    (root / "output/A.png").write_bytes(b"damaged-output")
    code, report = invoke([*arguments, "--resume"])
    assert code == 2 and "modified" in report["message"], report
    assert marker.read_text().splitlines() == ["A", "B", "B"]


with tempfile.TemporaryDirectory(prefix="fmn-native-recovery-") as directory:
    root = Path(directory)
    failures = []
    for format in ("png", "png_sequence", "svg", "y4m", "gif", "wav"):
        try:
            real_output_resume(root / format, format)
        except Exception:
            failures.append(format)
            traceback.print_exc()
        else:
            print(f"native batch recovery passed: {format}", flush=True)
    try:
        real_cli_resume(root / "cli")
    except Exception:
        failures.append("cli")
        traceback.print_exc()
    else:
        print("native batch recovery passed: cli", flush=True)
    for format in ("gif", "y4m", "png_sequence", "wav"):
        try:
            real_output_resume(root / ("paired-" + format), format, save_last_frame=True)
        except Exception:
            failures.append("paired-" + format)
            traceback.print_exc()
        else:
            print(f"native paired recovery passed: {format}", flush=True)
    try:
        real_cli_resume(root / "paired-cli", save_last_frame=True)
    except Exception:
        failures.append("paired-cli")
        traceback.print_exc()
    else:
        print("native paired recovery passed: cli", flush=True)
    assert not failures, "native batch recovery failed: " + ", ".join(failures)
print("native batch recovery acceptance passed: six single formats, four paired formats, and both CLI modes")
