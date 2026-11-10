"""Installed-wheel batch acceptance: real native motion, receipts and cancellation."""
import contextlib
import hashlib
import io
import json
import pathlib
import struct
import sys
import tempfile

import manimlib
from fmn_python import BatchRenderError, RenderJob, render_scenes
from fmn_python.__main__ import main


def read_y4m(path, width=96, height=54, fps=8):
    payload = pathlib.Path(path).read_bytes()
    header, records = payload.split(b"\n", 1)
    assert header == f"YUV4MPEG2 W{width} H{height} F{fps}:1 Ip A1:1 C420mpeg2".encode()
    size = width * height * 3 // 2
    assert records and len(records) % (size + 6) == 0
    frames = []
    for offset in range(0, len(records), size + 6):
        assert records[offset:offset + 6] == b"FRAME\n"
        frame = records[offset + 6:offset + 6 + size]
        assert all(value == 128 for value in frame[width * height:]), "white motion must have neutral chroma"
        frames.append(frame[:width * height])
    return frames


def centers(frames, width=96):
    result = []
    for frame in frames:
        positions = [index % width for index, value in enumerate(frame) if value > 200]
        assert len(positions) > 10, "expected a visible white square"
        result.append(sum(positions) / len(positions))
    return result


class Motion(manimlib.Scene):
    def __init__(self, direction=1, **kwargs):
        super().__init__(**kwargs)
        self.direction = direction
    def construct(self):
        square = manimlib.Square(side_length=1, fill_color=manimlib.WHITE, fill_opacity=1, stroke_width=0)
        square.shift(-2 * self.direction * manimlib.RIGHT)
        self.add(square)
        self.wait(1 / 8)
        self.play(square.animate.shift(4 * self.direction * manimlib.RIGHT), run_time=1 / 4, rate_func=manimlib.linear)


class Failed(Motion):
    def construct(self):
        super().construct()
        raise RuntimeError("batch-failure-after-native-capture")


class Interrupted(Motion):
    def construct(self):
        super().construct()
        raise KeyboardInterrupt("batch-interrupted-after-native-capture")


class Early(Motion):
    def construct(self):
        super().construct()
        raise manimlib.EndScene("normal early completion")


def render(jobs, directory, **kwargs):
    return render_scenes(jobs, directory, format=kwargs.pop("format", "y4m"),
                         resolution=(96, 54), fps=8, threads=kwargs.pop("threads", 1), **kwargs)


def check_receipt(outcome):
    result = outcome.result
    data = outcome.destination.read_bytes()
    assert outcome.status == "succeeded" and not result.certified
    assert result.bytes == len(data)
    assert result.digest == hashlib.sha256(data).hexdigest()


def motion_and_thread_replay(root):
    jobs = [RenderJob("right", Motion, {"direction": 1, "random_seed": 7}),
            RenderJob("left", Motion, {"direction": -1, "random_seed": 11})]
    first = render(jobs, root / "one")
    replay = render(jobs, root / "four", threads=4)
    assert first.ok and replay.ok
    for direction, a, b in zip((1, -1), first.outcomes, replay.outcomes):
        check_receipt(a)
        check_receipt(b)
        frames = read_y4m(a.destination)
        xs = centers(frames)
        assert len(frames) == a.result.frame_count == 3
        assert all(direction * (end - start) > 5 for start, end in zip(xs, xs[1:])), xs
        assert min(xs) < 48 < max(xs), xs
        assert a.destination.read_bytes() == b.destination.read_bytes()
        assert a.result.seed == b.result.seed


def fail_fast_preserves_completed_output(root):
    try:
        render({"first": Motion, "broken": Failed, "later": Motion}, root)
    except BatchRenderError as error:
        report = error.result
        assert "after-native-capture" in str(error.__cause__)
    else:
        raise AssertionError("a failed scene was reported as successful")
    assert [item.status for item in report.outcomes] == ["succeeded", "failed", "not_run"]
    check_receipt(report.outcomes[0])
    assert not (root / "broken.y4m").exists()
    assert not (root / "later.y4m").exists()


def keep_going_uses_a_fresh_generation(root):
    report = render({"first": Motion, "broken": Failed, "later": Motion}, root, continue_on_error=True)
    assert not report.ok
    assert [item.status for item in report.outcomes] == ["succeeded", "failed", "succeeded"]
    check_receipt(report.outcomes[2])
    assert len(read_y4m(report.outcomes[2].destination)) == 3
    assert not (root / "broken.y4m").exists()


def interrupt_cancels_without_erasing_progress(root):
    try:
        render({"first": Motion, "stop": Interrupted, "later": Motion}, root, continue_on_error=True)
    except KeyboardInterrupt as error:
        report = error.render_batch_result
    else:
        raise AssertionError("KeyboardInterrupt was swallowed")
    assert [item.status for item in report.outcomes] == ["succeeded", "cancelled", "not_run"]
    check_receipt(report.outcomes[0])
    assert not (root / "stop.y4m").exists()
    assert not (root / "later.y4m").exists()


def existing_destination_is_not_overwritten(root):
    root.mkdir(parents=True)
    destination = root / "second.png"
    destination.write_bytes(b"preserve-existing-destination")
    class Never(manimlib.Scene):
        def __init__(self):
            raise AssertionError("preflight must precede construction")
    try:
        render({"first": Never, "second": Never}, root, format="png")
    except FileExistsError:
        pass
    else:
        raise AssertionError("existing output was admitted")
    assert destination.read_bytes() == b"preserve-existing-destination"
    assert not (root / "first.png").exists()


def early_completion_and_sequences(root):
    report = render({"early": Early, "ordinary": Motion}, root, format="png_sequence")
    assert report.ok
    for outcome in report.outcomes:
        frames = sorted(outcome.destination.glob("*.png"))
        assert len(frames) == outcome.result.frame_count == 3
        for frame in frames:
            data = frame.read_bytes()
            assert data[:8] == b"\x89PNG\r\n\x1a\n"
            assert struct.unpack_from(">II", data, 16) == (96, 54)


def invoke(*arguments):
    previous = sys.argv
    output, errors = io.StringIO(), io.StringIO()
    sys.argv = ["fmn-python", "--robot", *map(str, arguments)]
    try:
        with contextlib.redirect_stdout(output), contextlib.redirect_stderr(errors):
            code = main()
    finally:
        sys.argv = previous
    report = json.loads(output.getvalue())
    assert report["exit"]["code"] == code
    return code, report, errors.getvalue()


def cli_publishes_multiple_real_pngs(root):
    root.mkdir(parents=True)
    source = root / "scenes.py"
    source.write_text('''from manimlib import *
print("batch-native-source-loaded")
class B(Scene):
    def construct(self):
        self.add(Square(side_length=1, fill_color=WHITE, fill_opacity=1, stroke_width=0).shift(2 * RIGHT))
class A(Scene):
    def construct(self):
        self.add(Square(side_length=1, fill_color=WHITE, fill_opacity=1, stroke_width=0).shift(2 * LEFT))
''')
    code, report, diagnostics = invoke(source, "--write_all", "--format", "png", "--resolution", "96x54",
                                       "--fps", "8", "--threads", "1", "--video_dir", root / "output")
    assert code == 0 and report["all_succeeded"]
    assert diagnostics.count("batch-native-source-loaded") == 1
    outcomes = report["batch"]["outcomes"]
    assert [item["name"] for item in outcomes] == ["A", "B"]
    images = []
    for item in outcomes:
        data = pathlib.Path(item["destination"]).read_bytes()
        assert data[:8] == b"\x89PNG\r\n\x1a\n"
        assert struct.unpack_from(">II", data, 16) == (96, 54)
        assert item["result"]["digest"] == hashlib.sha256(data).hexdigest()
        images.append(data)
    assert images[0] != images[1]


def cli_keep_going_is_nonzero_with_real_success(root):
    root.mkdir(parents=True)
    source = root / "scenes.py"
    source.write_text('''from manimlib import *
class A(Scene):
    def construct(self):
        self.add(Square(fill_opacity=1))
        self.wait(1 / 8)
        raise RuntimeError("batch-native-scene-failure")
class B(Scene):
    def construct(self):
        self.add(Square(fill_opacity=1))
''')
    code, report, _ = invoke(source, "--write_all", "--keep-going", "--format", "png", "--resolution", "96x54",
                             "--fps", "8", "--threads", "1", "--video_dir", root / "output")
    assert code == 5 and not report["all_succeeded"]
    assert report["batch"]["counts"] == dict(succeeded=1, failed=1, cancelled=0, not_run=0)
    assert not (root / "output/A.png").exists()
    assert (root / "output/B.png").read_bytes()[:8] == b"\x89PNG\r\n\x1a\n"


CASES = (motion_and_thread_replay, fail_fast_preserves_completed_output,
         keep_going_uses_a_fresh_generation, interrupt_cancels_without_erasing_progress,
         existing_destination_is_not_overwritten, early_completion_and_sequences,
         cli_publishes_multiple_real_pngs, cli_keep_going_is_nonzero_with_real_success)
with tempfile.TemporaryDirectory(prefix="fmn-native-batch-") as directory:
    for case in CASES:
        case(pathlib.Path(directory) / case.__name__)
print(f"native batch acceptance passed: {len(CASES)} cases")
