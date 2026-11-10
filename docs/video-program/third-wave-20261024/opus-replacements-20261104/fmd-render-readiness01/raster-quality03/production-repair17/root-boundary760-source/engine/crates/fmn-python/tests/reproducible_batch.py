"""Installed-native reproducible batch exports; no renderer or runtime doubles.

Exercises actual PNG/still/sequence bytes, guarded sessions, per-scene native
manifests and the real parser/source-loader console route. Outputs are retained.
"""
from contextlib import redirect_stdout, redirect_stderr
import io
import json
from pathlib import Path
import tempfile

import manimlib as ml
from fmn_python.batch_rendering import render_scenes
from fmn_python.console_rendering import try_render_cli

_ROOT = Path(tempfile.mkdtemp(prefix="fmn-reproducible-batch-native-"))
_SOURCE = {"reproducible_batch.py": Path(__file__).read_bytes()}


class StaticLeft(ml.Scene):
    def construct(self):
        self.add(ml.Dot().shift(ml.LEFT))


class StaticRight(ml.Scene):
    def construct(self):
        self.add(ml.Square().shift(ml.RIGHT))


class Moving(ml.Scene):
    def construct(self):
        dot = ml.Dot().shift(ml.LEFT)
        self.add(dot)
        self.play(dot.animate.shift(2 * ml.RIGHT), run_time=0.5, rate_func=ml.linear)


def _check(report):
    assert report.ok and report.all_scenes_certified
    data = report.as_dict()
    assert data["reproducible_requested"] and data["all_scenes_certified"]
    assert data["certified"] is False, "no new aggregate artifact certificate"
    for outcome in report.outcomes:
        receipt = outcome.result
        assert receipt.certified and len(receipt.closure_digest) == 64
        assert receipt.manifest == receipt.destination.with_name(receipt.destination.name + ".manifest") / "manifest.fmnp"
        assert receipt.manifest.is_file() and receipt.manifest.stat().st_size > 0
        assert receipt.manifest.with_name("manifest.txt").is_file()


def test_native_stills_match_at_one_and_four_threads():
    jobs = {"Left": StaticLeft, "Right": StaticRight}
    reports = [render_scenes(jobs, _ROOT / f"stills-{threads}", format="png", resolution=(96, 54),
                            fps=4, threads=threads, reproducible=True, sources=_SOURCE)
               for threads in (1, 4)]
    for report in reports:
        _check(report)
        for outcome in report.outcomes:
            assert outcome.result.frame_count == 1
            assert outcome.destination.read_bytes().startswith(b"\x89PNG\r\n\x1a\n")
    for left, right in zip(*[report.outcomes for report in reports]):
        assert left.result.digest == right.result.digest
        assert left.destination.read_bytes() == right.destination.read_bytes()
    assert reports[0].outcomes[0].destination.read_bytes() != reports[0].outcomes[1].destination.read_bytes()


def test_native_sequences_keep_frame_order_and_per_scene_publication():
    reports = [render_scenes({"First": Moving, "Second": Moving}, _ROOT / f"sequences-{threads}",
                            resolution=(96, 54), fps=4, threads=threads,
                            reproducible=True, sources=_SOURCE) for threads in (1, 4)]
    outputs = []
    for report in reports:
        _check(report)
        scenes = []
        for outcome in report.outcomes:
            frames = [path.read_bytes() for path in sorted(outcome.destination.glob("*.png"))]
            assert len(frames) == outcome.result.frame_count == 2
            assert len(set(frames)) == 2, "must emit actual changing frames"
            assert all(frame.startswith(b"\x89PNG\r\n\x1a\n") for frame in frames)
            scenes.append(frames)
        outputs.append(scenes)
    assert outputs[0] == outputs[1]


def test_native_console_named_and_write_all_batches_publish_manifests():
    source = _ROOT / "scenes.py"
    source.write_text(
        'from manimlib import Scene, Dot, Square, RIGHT\n'
        'print("authored stdout")\n'
        'class Alpha(Scene):\n'
        '    def construct(self):\n'
        '        self.add(Dot())\n'
        'class Beta(Scene):\n'
        '    def construct(self):\n'
        '        self.add(Square().shift(RIGHT))\n'
    )
    for label, selection, expected in (("named", ["Beta", "Alpha"], ["Beta", "Alpha"]),
                                       ("all", ["--write_all"], ["Alpha", "Beta"])):
        stdout, stderr = io.StringIO(), io.StringIO()
        with redirect_stdout(stdout), redirect_stderr(stderr):
            code = try_render_cli(ml._native, ["--robot", str(source), *selection,
                "--reproducible", "--format", "png", "--resolution", "96x54", "--fps", "4",
                "--threads", "1", "--video_dir", str(_ROOT / label)])
        assert code == 0, (code, stdout.getvalue(), stderr.getvalue())
        lines = stdout.getvalue().splitlines()
        assert len(lines) == 1, stdout.getvalue()
        report = json.loads(lines[0])["batch"]
        assert report["all_scenes_certified"] and report["reproducible_requested"]
        assert [row["name"] for row in report["outcomes"]] == expected
        for row in report["outcomes"]:
            assert Path(row["result"]["manifest"]["path"]).is_file()
            assert Path(row["destination"]).read_bytes().startswith(b"\x89PNG\r\n\x1a\n")
        assert "authored stdout" in stderr.getvalue()


def test_native_batch_manifest_collision_is_preconstruction():
    constructed = []
    class Recorded(StaticLeft):
        def __init__(self):
            constructed.append("constructed")
            super().__init__()
    root = _ROOT / "collision"
    root.mkdir()
    existing = root / "Second.png.manifest"
    existing.write_bytes(b"preserve previous content")
    try:
        render_scenes({"First": Recorded, "Second": Recorded}, root, format="png",
                      reproducible=True, sources=_SOURCE, resolution=(96, 54))
    except FileExistsError:
        pass
    else:
        raise AssertionError("existing manifest was accepted")
    assert not constructed
    assert existing.read_bytes() == b"preserve previous content"
    assert not (root / "First.png").exists()


assert getattr(ml, "__franken_manim__", False), "requires the installed native FrankenManim portal"
_cases = sorted((name, case) for name, case in globals().items()
                if name.startswith("test_") and callable(case))
assert len(_cases) == 4
for _name, _case in _cases:
    _case()
    print("reproducible batch native acceptance passed:", _name)
print("retaining native batch outputs:", _ROOT)
