"""Actual portal quality admission and x264 bitstream evidence.

Run by portal_video::quality's production extension test. The bitstream probe
uses ffmpeg only to demux/copy H.264, never to re-encode or infer our settings.
"""
import pathlib
import re
import shutil
import subprocess
import tempfile
import os

import manimlib as m
from fmn_python import render_scene


class QualityPicture(m.Scene):
    def construct(self):
        self.add(m.Square(side_length=2, fill_color=m.BLUE, fill_opacity=1,
                          stroke_width=0))
        self.wait(0.25)


def quality_scene(**settings):
    scene = QualityPicture()
    for name, value in settings.items():
        setattr(scene.file_writer, name, value)
    return scene


def x264_user_data(annex_b):
    """Read user_data_unregistered SEI, excluding unrelated NAL payloads."""
    for nal in re.split(b"\x00\x00(?:\x00)?\x01", annex_b):
        if not nal or nal[0] & 31 != 6:
            continue
        rbsp = nal[1:].replace(b"\x00\x00\x03", b"\x00\x00")
        at = 0
        while at < len(rbsp):
            fields = []
            for _ in range(2):
                value = 0
                while at < len(rbsp):
                    byte = rbsp[at]
                    at += 1
                    value += byte
                    if byte != 255:
                        break
                else:
                    return None
                fields.append(value)
            kind, size = fields
            end = at + size
            if end > len(rbsp):
                return None
            payload = rbsp[at:end]
            if kind == 5 and len(payload) >= 16 and payload[16:].startswith(b"x264 - core "):
                return payload[16:].rstrip(b"\0").decode("utf-8")
            at = end
    return None


def check_probe():
    text = b"x264 - core TEST - options: crf=16.0 subme=8 psy_rd=0.40:0.00"
    payload = b"0123456789abcdef" + text
    nal = b"\x06\x05" + bytes([len(payload)]) + payload + b"\x80"
    assert x264_user_data(b"\x00\x00\x00\x01" + nal) == text.decode()
    assert x264_user_data(b"\x00\x00\x01\x65" + nal[1:]) is None
    assert x264_user_data(b"\x00\x00\x01" + nal[:-3]) is None
    assert x264_user_data(b"\x00\x00\x01\x06\xff") is None


check_probe()
ffmpeg = shutil.which("ffmpeg")
if os.environ.get("FMN_REQUIRE_FFMPEG") == "1" or os.environ.get("FMN_REQUIRE_FULL_INPUTS") == "1":
    assert ffmpeg, "real ffmpeg is required for portal quality acceptance"

with tempfile.TemporaryDirectory(prefix="fmn-portal-quality-") as directory:
    root = pathlib.Path(directory)
    missing = root / "no-encoder"
    failures = [
        ({"video_crf": 52}, "video_crf"),
        ({"video_crf": True}, "not bool"),
        ({"video_crf": 1.5}, "integer"),
        ({"video_preset": "slow -vf eq"}, "video_preset"),
        ({"video_tune": "animation,grain"}, "video_tune"),
        ({"video_bitrate": 0}, "video_bitrate"),
        ({"video_bitrate": False}, "not bool"),
        ({"video_codec": "libx265", "video_tune": "film"}, "libx264"),
        ({"video_codec": "h264_nvenc", "video_crf": 18}, "crf"),
        ({"video_codec": "hevc_videotoolbox", "video_preset": "slow"}, "preset"),
        ({"video_crf": 16, "video_bitrate": 8000000}, "mutually exclusive"),
    ]
    for index, (settings, fragment) in enumerate(failures):
        target = root / f"refused-{index}.mp4"
        target.write_bytes(b"preserve existing movie")
        scene = quality_scene(ffmpeg_bin=str(missing), **settings)
        try:
            render_scene(scene, target, resolution=(96, 54), fps=8, threads=1)
        except (TypeError, ValueError, OverflowError) as error:
            assert fragment in str(error), (settings, str(error))
            assert target.read_bytes() == b"preserve existing movie"
            assert not hasattr(scene, "_fmn_owned_render_session")
        else:
            raise AssertionError(f"incompatible quality was accepted: {settings}")

    # A Python getter failure is not turned into an absent optional setting.
    scene = quality_scene(ffmpeg_bin=str(missing))
    original = scene.file_writer
    marker = RuntimeError("authored quality getter failed")

    class BrokenWriter:
        def __getattr__(self, name):
            return getattr(original, name)

        @property
        def video_crf(self):
            raise marker

    scene.file_writer = BrokenWriter()
    try:
        render_scene(scene, root / "getter.mp4", resolution=(96, 54), fps=8, threads=1)
    except RuntimeError as error:
        assert error is marker, error
    else:
        raise AssertionError("authored descriptor exception was suppressed")
    assert not (root / "getter.mp4").exists()

    if ffmpeg:
        for extension in ("mp4", "mov"):
            for crf in (16, 28):
                scene = quality_scene(ffmpeg_bin=ffmpeg, video_crf=crf,
                                      video_preset="slow", video_tune="animation")
                target = root / f"quality-{crf}.{extension}"
                report = render_scene(scene, target, resolution=(96, 54), fps=8, threads=1)
                assert report.frame_count == 2 and not report.certified
                invocation, = report.ffmpeg_invocations
                assert invocation["encoder"] == "libx264"
                argv = invocation["argv"]
                input_at = argv.index("-i")
                for flag, value in (("-crf", str(crf)), ("-preset", "slow"), ("-tune", "animation")):
                    assert argv.count(flag) == 1 and argv.index(flag) > input_at + 1
                    assert argv[argv.index(flag) + 1] == value, argv
                assert "-b:v" not in argv
                demuxed = subprocess.run(
                    [ffmpeg, "-v", "error", "-nostdin", "-i", str(target), "-map", "0:v:0",
                     "-c:v", "copy", "-bsf:v", "h264_mp4toannexb", "-f", "h264", "pipe:1"],
                    capture_output=True, check=True, timeout=30,
                ).stdout
                sei = x264_user_data(demuxed)
                assert sei is not None, "missing actual x264 SEI"
                words = sei.split()
                assert f"crf={crf}.0" in words and "subme=8" in words, sei
                assert any(word.startswith("psy_rd=0.40:") for word in words), sei
                print(f"portal-quality {extension} crf={crf}: {sei}")

        # CRF zero must not disappear through a truthiness check.
        report = render_scene(quality_scene(ffmpeg_bin=ffmpeg, video_crf=0),
                              root / "lossless.mp4", resolution=(96, 54), fps=8, threads=1)
        argv = report.ffmpeg_invocations[0]["argv"]
        assert argv[argv.index("-crf") + 1] == "0"

        # None and absent attributes preserve the old encoder-default argv.
        report = render_scene(quality_scene(ffmpeg_bin=ffmpeg, video_crf=None,
                                             video_preset=None, video_tune=None, video_bitrate=None),
                              root / "defaults.mp4", resolution=(96, 54), fps=8, threads=1)
        argv = report.ffmpeg_invocations[0]["argv"]
        assert all(flag not in argv for flag in ("-crf", "-preset", "-tune", "-b:v")), argv

print("portal-quality: admission verified; actual x264 bitstream checks=" + str(bool(ffmpeg)))
