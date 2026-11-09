"""Real Reel output, independently decoded by ffmpeg; not a protocol double."""
import os
import pathlib
import shutil
import subprocess
import tempfile

import numpy as np
import manimlib as m
from fmn_python import render_scene


class Swatch(m.Scene):
    # The wire-format fidelity checks below were calibrated on a black field:
    # against the default #333333 the red swatch differs almost only in
    # chroma, which x264 smears at 96x54.
    default_camera_config = {"background_color": "#000000"}

    def construct(self):
        self.add(m.Square(side_length=2, fill_color="#FF0000", fill_opacity=1,
                          stroke_width=0).shift(m.UP))
        self.wait(2 / 8)


class LateAlpha(Swatch):
    def construct(self):
        super().construct()
        self.camera.background_rgba[3] = 0
        self.wait(1 / 8)


def options(**writer):
    return {"file_writer_config": writer}


def rejected(root, name, *, transparent=False, format="mov", **writer):
    target = root / name
    marker = b"existing artifact must survive failed negotiation"
    target.write_bytes(marker)
    scene = Swatch(camera_config={"background_opacity": 0 if transparent else 1},
                   **options(**writer))
    try:
        render_scene(scene, target, format=format, resolution=(96, 54), fps=8, threads=1)
    except (RuntimeError, ValueError, OSError) as error:
        assert target.read_bytes() == marker
        assert not hasattr(scene, "_fmn_owned_render_session")
        return str(error)
    raise AssertionError("invalid output profile succeeded")


ffmpeg = shutil.which("ffmpeg")
if os.environ.get("FMN_REQUIRE_FFMPEG") == "1":
    assert ffmpeg, "real ffmpeg is required for negotiated-video acceptance"
with tempfile.TemporaryDirectory(prefix="fmn-video-options-") as directory:
    root = pathlib.Path(directory)
    # These are negotiation errors even when no ffmpeg is present. Validate
    # them before consulting the locator and without touching the destination.
    before = os.environ.get("PATH")
    os.environ["PATH"] = str(root)
    try:
        assert "pixel_format" in rejected(root, "bad-wire.mov", pixel_format="yuv444p")
        assert "encoder name" in rejected(root, "bad-codec.mov", video_codec="x264 -vf eq")
        assert "MOV" in rejected(root, "alpha.mp4", format="mp4", transparent=True)
        assert "alpha wire" in rejected(root, "alpha-nv12.mov", transparent=True,
                                        video_codec="qtrle", pixel_format="nv12")
        assert "qtrle" in rejected(root, "alpha-codec.mov", transparent=True,
                                   video_codec="libx265", pixel_format="rgba")
        assert "gamma" in rejected(root, "gamma.mov", gamma=1.2)
    finally:
        if before is None:
            os.environ.pop("PATH", None)
        else:
            os.environ["PATH"] = before

    if ffmpeg:
        def decode(path, pixel_format="rgba"):
            result = subprocess.run(
                [ffmpeg, "-v", "error", "-nostdin", "-i", str(path), "-map", "0:v:0",
                 "-frames:v", "8", "-pix_fmt", pixel_format, "-f", "rawvideo", "pipe:1"],
                capture_output=True, timeout=30, check=True,
            )
            return np.frombuffer(result.stdout, dtype=np.uint8).reshape(-1, 54, 96, 4)

        # An explicit executable with spaces must work without a PATH lookup.
        executable = root / "encoder with spaces" / "ffmpeg"
        executable.parent.mkdir()
        shutil.copy2(ffmpeg, executable)
        prior_path = os.environ.get("PATH")
        os.environ["PATH"] = str(root)
        alpha_frames = []
        try:
            for wire in ("yuv420p", "rgba", "bgra"):
                scene = Swatch(camera_config={"background_opacity": 0},
                               **options(ffmpeg_bin=str(executable), pixel_format=wire))
                target = root / (wire + ".mov")
                report = render_scene(scene, target, resolution=(96, 54), fps=8, threads=1)
                assert report.frame_count == 2 and not report.certified
                invocation, = report.ffmpeg_invocations
                assert invocation["encoder"] == "qtrle"
                argv = invocation["argv"]
                assert argv[argv.index("-pix_fmt") + 1] == ("rgba" if wire == "yuv420p" else wire)
                assert "vflip" not in argv and "-vf" not in argv
                frames = decode(target)
                assert len(frames) == 2
                assert np.all(frames[:, 0, 0, 3] == 0), "transparent background was flattened"
                ys, xs = np.nonzero(frames[0, :, :, 3] > 250)
                assert len(xs) > 25 and ys.mean() < 24, "orientation or opaque content lost"
                assert frames[0, ys, xs, 0].mean() > 245, "RGBA/BGRA channel order corrupted"
                assert frames[0, ys, xs, 2].mean() < 5
                alpha_frames.append(frames)
            assert all(np.array_equal(alpha_frames[0], frame) for frame in alpha_frames[1:])
        finally:
            if prior_path is None:
                os.environ.pop("PATH", None)
            else:
                os.environ["PATH"] = prior_path

        for wire, codec in (("nv12", "auto"), ("rgba", "libx264"),
                            ("bgra", "libx264"), ("p010le", "libx264")):
            target = root / ("opaque-" + wire + ".mp4")
            report = render_scene(Swatch, target, resolution=(96, 54), fps=8, threads=1,
                                  scene_kwargs=options(pixel_format=wire, video_codec=codec))
            invocation, = report.ffmpeg_invocations
            assert invocation["encoder"] == "libx264"
            argv = invocation["argv"]
            formats = [argv[i + 1] for i, word in enumerate(argv[:-1]) if word == "-pix_fmt"]
            assert formats == [wire, "yuv420p10le" if wire == "p010le" else "yuv420p"], formats
            frames = decode(target)
            assert len(frames) == 2 and np.all(frames[:, :, :, 3] == 255)
            assert frames[0, :, :, 0].max() >= 240, f"wire={wire}, codec={codec}, max={frames[0, :, :, 0].max()}"

        assert "does not offer encoder" in rejected(root, "unknown.mov", video_codec="fmn_missing_codec")
        target = root / "late-alpha.mp4"
        try:
            render_scene(LateAlpha, target, resolution=(96, 54), fps=8, threads=1)
        except ValueError as error:
            assert "alpha" in str(error)
        else:
            raise AssertionError("an opaque generation silently dropped dynamic alpha")
        assert not target.exists(), "failed generation published a partial video"

print("portal-video: profile refusals verified; decoded alpha/channel/wire tests=" + str(bool(ffmpeg)))

# The installed console owner, with the real extension, must propagate the
# same profile through single and batch generations without rewriting source.
from fmn_python.console_rendering import try_render_cli
import contextlib
import io
import json

with tempfile.TemporaryDirectory(prefix="fmn-video-console-") as directory:
    root = pathlib.Path(directory)
    source = root / "scene.py"
    source.write_text("from manimlib import *\n"
                      "class First(Scene):\n"
                      "    def __init__(self):\n        super().__init__()\n"
                      "    def construct(self):\n"
                      "        self.add(Square(side_length=2, fill_color=RED, fill_opacity=1, stroke_width=0).shift(UP))\n"
                      "        self.wait(2 / 8)\n"
                      "class Second(First):\n    pass\n")

    def console_output(*arguments):
        stdout, stderr = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            code = try_render_cli(m, ["--robot", str(source), "--resolution", "96x54",
                                      "--fps", "8", "--threads", "1", *arguments])
        report = json.loads(stdout.getvalue())
        assert report["exit"]["code"] == code
        return code, report

    target = root / "still.png"
    code, report = console_output("First", "-s", "--transparent", "--video_dir", str(target))
    assert code == 0 and report["frame_count"] == 1, report
    if ffmpeg:
        png = decode(target)
        assert len(png) == 1 and png[0, 0, 0, 3] == 0
        target = root / "single.mov"
        code, report = console_output("First", "--format", "mov", "-t", "--pix_fmt", "bgra",
                                      "--vcodec", "qtrle", "--ffmpeg_bin", ffmpeg,
                                      "--video_dir", str(target))
        assert code == 0 and report["frame_count"] == 2, report
        assert report["ffmpeg_invocations"][0]["encoder"] == "qtrle"
        assert np.all(decode(target)[:, 0, 0, 3] == 0)
        batch = root / "batch"
        code, report = console_output("Second", "First", "--format", "mov", "-t",
                                      "--vcodec", "qtrle", "--pix_fmt", "rgba",
                                      "--video_dir", str(batch))
        assert code == 0, report
        for name in ("First", "Second"):
            frames = decode(batch / (name + ".mov"))
            assert len(frames) == 2 and np.all(frames[:, 0, 0, 3] == 0)
    print("portal-video-console: transparent PNG and native single/batch MOV accepted")
