"""Installed wheel -> FMTL -> standalone fmn, with real Y4M frame comparison.

Requires FMN_TEST_BIN naming the freshly built native fmn. Nothing is mocked;
missing native capabilities or binaries fail this test rather than skip it.
Artifacts are retained on failure so the exact source/outputs can be examined.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import textwrap

import manimlib  # Fail before creating fixtures when the native wheel is absent.


ROOT = Path(tempfile.mkdtemp(prefix="fmn-bundle-console-native-"))
print(f"retaining bundle console evidence: {ROOT}")
BINARY = Path(os.environ["FMN_TEST_BIN"]).resolve(strict=True)
SOURCE = ROOT / "authored_scene.py"
SOURCE.write_text(textwrap.dedent('''\
    from pathlib import Path
    import manimlib as m
    print("authored import chatter")

    def event(name):
        with Path(__file__).with_suffix(".events").open("a") as output:
            output.write(name + "\\n")

    class Portable(m.Scene):
        def setup(self):
            event("setup")
        def construct(self):
            event("construct")
            print("authored construct chatter")
            moving = m.Square(side_length=1, fill_opacity=1, stroke_width=0)
            moving.shift(2 * m.LEFT)
            follower = m.Square(side_length=0.4, fill_opacity=1, stroke_width=0)
            follower.add_updater(lambda mob, dt: mob.move_to(moving.get_center() + m.UP))
            self.add(moving, follower)
            self.play(moving.animate.shift(3 * m.RIGHT), run_time=0.125,
                      rate_func=lambda alpha: alpha * alpha)
            self.wait(0.125)
            self.wait(0)
        def tear_down(self):
            event("tear_down")

    class Unrepresentable(m.Scene):
        def construct(self):
            self.camera.frame.shift(m.RIGHT)
            self.add(m.Square())
'''))
EVENTS = SOURCE.with_suffix(".events")
PYTHON = [sys.executable, "-m", "fmn_python"]


def run(argv, *, expected=0):
    process = subprocess.run(
        argv, cwd=ROOT, text=True, capture_output=True, timeout=180,
    )
    assert process.returncode == expected, (
        argv, process.returncode, process.stdout, process.stderr,
    )
    lines = process.stdout.splitlines()
    assert len(lines) == 1, (argv, process.stdout, process.stderr)
    report = json.loads(lines[0])
    return report, process.stderr


def python_scene(destination, fps, format="fmtl", scene="Portable", expected=0):
    return run(PYTHON + [
        "--robot", str(SOURCE), scene, "--format", format,
        "--resolution", "96x54", "--fps", str(fps),
        "--video_dir", str(destination),
    ], expected=expected)


for fps in (24, 30, 60):
    artifact = ROOT / f"authored-{fps}.fmtl"
    receipt, chatter = python_scene(artifact, fps)
    assert receipt["kind"] == "bundle-export" and receipt["exported"]
    assert "authored import chatter" in chatter and "authored construct chatter" in chatter
    frame_count = 2 * ((fps + 7) // 8)
    assert receipt["frame_count"] == frame_count
    assert receipt["bundle"]["segment_count"] == 3
    assert receipt["bundle"]["certified_source"] is False
    data = artifact.read_bytes()
    assert len(data) == receipt["bundle"]["bytes"]
    assert hashlib.sha256(data).hexdigest() == receipt["bundle"]["sha256"]
    events = EVENTS.read_bytes()
    assert events.splitlines()[-3:] == [b"setup", b"construct", b"tear_down"]
    outputs = []
    for threads in (1, 4):
        directory = ROOT / f"replayed-{fps}-{threads}"
        run([str(BINARY), "--robot", str(artifact), "--format", "y4m",
             "--resolution", "96x54", "--fps", str(fps), "--threads", str(threads),
             "--video_dir", str(directory)])
        output = (directory / f"authored-{fps}.y4m").read_bytes()
        header, body = output.split(b"\n", 1)
        assert header == f"YUV4MPEG2 W96 H54 F{fps}:1 Ip A1:1 C420mpeg2".encode()
        frame_bytes = 96 * 54 * 3 // 2
        assert len(body) == frame_count * (frame_bytes + 6)
        assert body[:6] == b"FRAME\n"
        assert body[6:6 + frame_bytes] != body[-frame_bytes:], "animation is not visible"
        assert EVENTS.read_bytes() == events, "standalone playback executed authored Python"
        outputs.append(output)
    assert outputs[0] == outputs[1], "worker count changed the output"
    direct = ROOT / f"direct-{fps}.y4m"
    python_scene(direct, fps, format="y4m")
    assert direct.read_bytes() == outputs[0], "direct and exported/replayed frames differ"
    # The same program recreates byte-identical bundles with a fresh Scene.
    duplicate = ROOT / f"duplicate-{fps}.fmtl"
    python_scene(duplicate, fps)
    assert duplicate.read_bytes() == data
    # Reject a collision before running source a second time.
    events = EVENTS.read_bytes()
    failed, _ = python_scene(artifact, fps, expected=6)
    assert failed["phase"] == "start" and not failed["artifact_published"]
    assert artifact.read_bytes() == data and EVENTS.read_bytes() == events

bad = ROOT / "camera.fmtl"
failed, _ = python_scene(bad, 24, scene="Unrepresentable", expected=4)
assert failed["exit"]["identity"] == "capability" and not bad.exists()
print("installed-wheel FMTL export, standalone replay and direct Y4M frame equality passed")
