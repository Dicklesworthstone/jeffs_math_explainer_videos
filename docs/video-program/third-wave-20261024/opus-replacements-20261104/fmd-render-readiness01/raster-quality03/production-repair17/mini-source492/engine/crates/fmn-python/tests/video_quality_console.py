"""Console and API quality controls through the real installed extension.

Parsing refusals must happen before authored source imports. Successful cases
use the ordinary single/batch/subdivision/paired owners and inspect their
native invocation receipts. The separate video_quality.py suite reads SEI.
"""
import contextlib
import io
import json
import os
from pathlib import Path
import shutil
import tempfile

import manimlib as m
from fmn_python import render_scene
from fmn_python.console_rendering import try_render_cli


def invocations(value):
    if isinstance(value, dict):
        for key, child in value.items():
            if key == "ffmpeg_invocations":
                yield from child
            else:
                yield from invocations(child)
    elif isinstance(value, list):
        for child in value:
            yield from invocations(child)


def assert_quality(report, encodes, soundtrack_muxes, crf=16):
    """Each published clip is exactly one libx264 encode carrying the console
    quality flags. A subdivided MP4/MOV clip also muxes its scene-window
    soundtrack, exact-duration silence when no cue exists
    (docs/python/subdivided-recording.md). The receipt marks that stream-copy
    job with encoder None; it must copy the encoded video, so it can neither
    re-encode at another quality nor pass for a second encode (fm-3s7c)."""
    actual = list(invocations(report))
    encoded = [invocation for invocation in actual if invocation["encoder"] is not None]
    muxed = [invocation for invocation in actual if invocation["encoder"] is None]
    assert len(encoded) == encodes, report
    assert len(muxed) == soundtrack_muxes, report
    for invocation in encoded:
        assert invocation["encoder"] == "libx264", invocation
        argv = invocation["argv"]
        for flag, value in (("-crf", str(crf)), ("-preset", "slow"), ("-tune", "animation")):
            assert argv.count(flag) == 1 and argv.index(flag) > argv.index("-i") + 1, argv
            assert argv[argv.index(flag) + 1] == value, argv
        assert "-b:v" not in argv and "-vf" not in argv, argv
    for invocation in muxed:
        argv = invocation["argv"]
        assert argv.count("-c:v") == 1 and argv[argv.index("-c:v") + 1] == "copy", argv
        assert argv.count("-c:a") == 1 and argv[argv.index("-c:a") + 1] == "aac", argv
        for flag in ("-crf", "-preset", "-tune", "-b:v", "-vf"):
            assert flag not in argv, argv


ffmpeg = shutil.which("ffmpeg")
if os.environ.get("FMN_REQUIRE_FFMPEG") == "1" or os.environ.get("FMN_REQUIRE_FULL_INPUTS") == "1":
    assert ffmpeg, "real ffmpeg required for console quality acceptance"

with tempfile.TemporaryDirectory(prefix="fmn-video-quality-console-") as directory:
    root = Path(directory)
    imported = root / "imported.txt"
    constructed = root / "constructed.txt"
    source = root / "scene.py"
    source.write_text(
        "from pathlib import Path as _FSPath\nfrom manimlib import *\n"
        f"_FSPath({str(imported)!r}).write_text('imported')\n"
        "class First(Scene):\n"
        "    def __init__(self):\n"
        "        super().__init__()\n"
        f"        with _FSPath({str(constructed)!r}).open('a') as stream:\n"
        "            stream.write(type(self).__name__ + '\\n')\n"
        "    def construct(self):\n"
        "        self.add(Square(side_length=2, fill_color=BLUE, fill_opacity=1, stroke_width=0))\n"
        "        self.wait(1 / 8)\n"
        "        self.wait(1 / 8)\n"
        "class Second(First):\n    pass\n",
        encoding="utf-8",
    )

    def console(*arguments):
        stdout, stderr = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            code = try_render_cli(m, ["--robot", str(source), "--resolution", "96x54",
                                     "--fps", "8", "--threads", "1", *arguments])
        # One terminal robot object; no authored output or partial JSON leaks.
        report = json.loads(stdout.getvalue())
        assert report["exit"]["code"] == code, report
        return code, report

    target = root / "unchanged.mp4"
    target.write_bytes(b"existing output")
    failures = [
        ("--format", "mp4", "--crf=52"),
        ("--format", "mp4", "--crf", "-1"),
        ("--format", "mp4", "--crf", "1.5"),
        ("--format", "mp4", "--crf", "0", "--crf=0"),
        ("--format", "mp4", "--preset", "slow -vf eq"),
        ("--format", "mp4", "--preset=slow", "--preset", "slow"),
        ("--format", "mp4", "--video-bitrate=0"),
        ("--format", "mp4", "--video-bitrate=4294967296"),
        ("--format", "mp4", "--crf=0", "--video-bitrate=12000000"),
        ("--format", "png", "--crf=16"),
        ("--format", "gif", "--preset=slow"),
        ("--format", "wav", "--tune=animation"),
        ("--format", "mov", "-t", "--crf=16"),
        ("--format", "mp4", "--crf"),
    ]
    for args in failures:
        code, report = console("First", "--video_dir", str(target), *args)
        assert code == 2, (args, report)
        assert target.read_bytes() == b"existing output"
        assert not imported.exists() and not constructed.exists(), (args, report)

    # Explicit writer controls cannot silently disappear on a native format.
    class Unexecuted(m.Scene):
        def construct(self):
            raise AssertionError("native-format refusal must precede construct")

    for format in ("png", "png_sequence", "svg", "gif", "y4m", "wav"):
        scene = Unexecuted()
        scene.file_writer.video_crf = 0
        destination = root / ("native-refusal-" + format)
        try:
            render_scene(scene, destination, format=format, resolution=(96, 54), fps=8, threads=1)
        except ValueError as error:
            assert "video quality" in str(error), error
        else:
            raise AssertionError("a native format silently discarded video quality")
        assert not destination.exists()

    if ffmpeg:
        # (name, selectors, constructors, encodes, soundtrack muxes). The
        # scene has no sound cues: whole-scene exports carry no audio track,
        # and each of the two subdivided clips muxes its silent window.
        cases = [
            ("single.mp4", ("First", "--format", "mp4"), ["First"], 1, 0),
            ("single.mov", ("First", "--format", "mov"), ["First"], 1, 0),
            ("named", ("Second", "First", "--format", "mp4"), ["Second", "First"], 2, 0),
            ("all", ("--write_all", "--format", "mov"), ["First", "Second"], 2, 0),
            ("clips", ("First", "--format", "mp4", "--subdivide"), ["First"], 2, 2),
            ("paired.mp4", ("First", "--format", "mp4", "--save-last-frame"), ["First"], 1, 0),
            ("paired-clips", ("First", "--format", "mp4", "--subdivide", "--save-last-frame"), ["First"], 2, 2),
        ]
        for name, selectors, expected_constructors, encodes, muxes in cases:
            constructed.write_text("", encoding="utf-8")
            destination = root / name
            code, report = console(*selectors, "--video_dir", str(destination),
                                   "--ffmpeg_bin", ffmpeg,
                                   "--crf=16", "--preset", "slow", "--tune=animation")
            assert code == 0, report
            assert_quality(report, encodes, muxes)
            assert constructed.read_text().splitlines() == expected_constructors, report
            if "--save-last-frame" in selectors:
                still = (destination.with_name(destination.name + ".png") if "--subdivide" in selectors
                         else destination.with_suffix(".png"))
                assert still.read_bytes().startswith(b"\x89PNG\r\n\x1a\n"), report

print("portal-video-quality-console: pre-import refusals verified; native output modes=" + str(bool(ffmpeg)))
