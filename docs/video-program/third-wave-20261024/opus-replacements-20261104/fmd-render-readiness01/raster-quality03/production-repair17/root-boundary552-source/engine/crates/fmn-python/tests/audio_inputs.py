"""Real Scene.add_sound decoding, native mix, atomic publication and receipts."""
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import runpy
import shutil
import struct
import subprocess
import tempfile
import wave

import numpy as np
import manimlib as m
from fmn_python import render_scene
from fmn_python.console_rendering import try_render_cli


def float_wav(path, samples, channels, rate):
    payload = samples if isinstance(samples, bytes) else np.asarray(samples, dtype="<f4").tobytes()
    fmt = struct.pack("<HHIIHH", 3, channels, rate, rate * channels * 4, channels * 4, 32)
    path.write_bytes(b"RIFF" + struct.pack("<I", 36 + len(payload)) + b"WAVEfmt " +
                     struct.pack("<I", 16) + fmt + b"data" + struct.pack("<I", len(payload)) + payload)


def sound_scene(asset, executable="ffmpeg"):
    class Sound(m.Scene):
        def construct(self):
            self.wait(1 / 8)
            self.add_sound(str(asset), gain=-3)
            self.wait(1 / 16)
            self.add_sound(str(asset), time_offset=-4 / 48000, gain=-12, gain_to_background=-6)
            self.wait(5 / 16)
    return Sound(file_writer_config={"ffmpeg_bin": str(executable)})


def render(asset, destination, executable="ffmpeg", threads=1):
    return render_scene(sound_scene(asset, executable), destination,
                        resolution=(96, 54), fps=16, threads=threads)


def pcm(path):
    with wave.open(str(path), "rb") as audio:
        assert (audio.getframerate(), audio.getnchannels(), audio.getsampwidth()) == (48000, 2, 2)
        return np.frombuffer(audio.readframes(audio.getnframes()), dtype="<i2").reshape(-1, 2)


with tempfile.TemporaryDirectory(prefix="fmn sound inputs ") as directory:
    root = Path(directory)
    native_asset = root / "native.wav"
    float_wav(native_asset, [0.125, -0.25] * 128, 2, 48000)
    # A missing optional tool cannot turn PCM-only WAV export into a capability
    # failure. Its native decode is still named in the artifact receipt.
    native = render(native_asset, root / "native-output.wav", root / "missing-ffmpeg")
    assert native.sample_frames == 24000 and native.certified is False
    assert native.ffmpeg_invocations == ()
    assert len(native.audio_inputs) == 2
    for fact in native.audio_inputs:
        assert fact["decoder"] == "native-wav" and fact["decoder_sha256"] is None
        assert fact["source_sha256"] == hashlib.sha256(native_asset.read_bytes()).hexdigest()
        assert fact["sample_frames"] == 128
    facts = native.as_dict()
    facts["audio_inputs"][0]["source_sha256"] = "changed-copy"
    assert native.audio_inputs[0]["source_sha256"] != "changed-copy"

    # Reject writer callbacks that record a cue during preflight rather than
    # replacing their scene engine and silently losing the cue.
    side_effect_scene = m.Scene()
    class RecordingWriter:
        @property
        def ffmpeg_bin(self):
            side_effect_scene.add_sound(str(native_asset))
            return "ffmpeg"
    side_effect_scene.file_writer = RecordingWriter()
    try:
        side_effect_scene._begin_native_output(str(root / "preflight.wav"), "wav", 96, 54, 16, 1, 0)
    except RuntimeError as error:
        assert "mutates engine state" in str(error), error
    else:
        raise AssertionError("writer callback's sound state was erased")
    assert len(side_effect_scene._sound_request_facts()) == 1
    assert not (root / "preflight.wav").exists()

    # Path resolution belongs to the cue call site, not whichever cwd a later
    # authored callback happens to leave active when the render finishes.
    previous_cwd = Path.cwd()
    elsewhere = root / "elsewhere"
    elsewhere.mkdir()
    class ChangingCwd(m.Scene):
        def construct(self):
            os.chdir(root)
            self.add_sound("native.wav")
            os.chdir(elsewhere)
            self.wait(1 / 8)
    try:
        result = render_scene(ChangingCwd, root / "cwd.wav", fps=16, threads=1)
    finally:
        os.chdir(previous_cwd)
    # Receipts carry canonical paths (macOS /var -> /private/var).
    assert result.audio_inputs[0]["path"] == str(native_asset.resolve())

    ffmpeg = shutil.which("ffmpeg")
    if not ffmpeg:
        assert not os.environ.get("FMN_REQUIRE_FFMPEG"), "required ffmpeg is unavailable"
        print("portal-audio: native WAV and ownership verified; external decode unavailable")
    else:
        fixtures = root / "fixtures"
        runpy.run_path(_audio_fixture_generator)["make"](fixtures)
        # A real native executable at a path containing spaces, not shell
        # tokenization, is the same configured capability for WAV and movies.
        explicit = root / "decoder with spaces"
        shutil.copyfile(ffmpeg, explicit)
        explicit.chmod(0o700)
        tool_hash = hashlib.sha256(explicit.read_bytes()).hexdigest()
        for row in (fixtures / "manifest.tsv").read_text().splitlines():
            asset_name, reference_name, channels, rate = row.split("\t")
            asset = fixtures / asset_name
            baseline = root / f"{asset_name}.reference.wav"
            float_wav(baseline, (fixtures / reference_name).read_bytes(), int(channels), int(rate))
            golden_path = root / f"{asset_name}.golden.wav"
            render(baseline, golden_path, root / "unneeded-decoder")
            outputs = []
            for threads in (1, 4):
                destination = root / f"{asset_name}.{threads}.wav"
                receipt = render(asset, destination, explicit, threads)
                assert destination.read_bytes() == golden_path.read_bytes(), asset_name
                assert receipt.sample_frames == 24000
                assert len(receipt.ffmpeg_invocations) == 2
                assert len(receipt.audio_inputs) == 2
                for fact in receipt.audio_inputs:
                    assert fact["source_sha256"] == hashlib.sha256(asset.read_bytes()).hexdigest()
                    assert fact["channels"] == int(channels) and fact["sample_rate"] == int(rate)
                    assert fact["decoder"] == "ffmpeg" and fact["decoder_sha256"] == tool_hash
                    assert len(fact["pcm_sha256"]) == 64
                for invocation in receipt.ffmpeg_invocations:
                    assert invocation["tool_sha256"] == tool_hash
                    assert invocation["tool_path"] == str(explicit)
                    assert invocation["encoder"] == "pcm_f32le"
                    assert "-ar" not in invocation["argv"] and "-ac" not in invocation["argv"]
                outputs.append(receipt.digest)
            assert outputs[0] == outputs[1], "thread count changed native mixed WAV"

        # Captured PATH survives authored callbacks. PCM-only operation above
        # proves this does not eagerly probe ffmpeg for every generation.
        old_path = os.environ.get("PATH")
        class ChangedPath(m.Scene):
            def construct(self):
                os.environ["PATH"] = str(root / "empty-bin")
                self.add_sound(str(fixtures / "flac.asset"))
                self.wait(1 / 8)
        try:
            result = render_scene(ChangedPath, root / "captured-path.wav", fps=16, threads=1)
        finally:
            if old_path is None:
                os.environ.pop("PATH", None)
            else:
                os.environ["PATH"] = old_path
        assert result.audio_inputs[0]["decoder"] == "ffmpeg"

        # Decoder failures preserve an already existing destination. No cue is
        # silently dropped and no successful-looking truncated WAV is emitted.
        bad = root / "bad.flac"
        bad.write_bytes(b"fLaCbroken")
        playlist = root / "playlist.m3u"
        playlist.write_text("#EXTM3U\nhttps://example.invalid/voice.mp3\n")
        for asset, executable in [(bad, explicit), (playlist, root / "missing-decoder"),
                                  (fixtures / "flac.asset", root / "missing-decoder")]:
            destination = root / "preserved.wav"
            destination.write_bytes(b"existing soundtrack")
            try:
                render(asset, destination, executable)
            except (RuntimeError, ValueError, OSError):
                pass
            else:
                raise AssertionError("invalid audio was silently accepted")
            assert destination.read_bytes() == b"existing soundtrack"

        # The complete movie path consumes compressed cues, publishes audio and
        # video together, and records the decoder between encode and mux facts.
        for extension in ("mp4", "mov"):
            destination = root / f"soundtrack.{extension}"
            receipt = render(fixtures / "flac.asset", destination, explicit)
            assert receipt.frame_count == 8 and len(receipt.ffmpeg_invocations) == 4
            assert receipt.ffmpeg_invocations[0]["encoder"] == "libx264"
            assert [v["encoder"] for v in receipt.ffmpeg_invocations[1:3]] == ["pcm_f32le"] * 2
            assert "copy" in receipt.ffmpeg_invocations[-1]["argv"]
            decoded = subprocess.run([ffmpeg, "-v", "error", "-i", str(destination),
                                      "-map", "0:a:0", "-c:a", "pcm_s16le", "-f", "s16le", "pipe:1"],
                                     capture_output=True, check=True, timeout=30).stdout
            samples = np.frombuffer(decoded, dtype="<i2").reshape(-1, 2)
            expected = pcm(root / "flac.asset.golden.wav")
            assert 24000 <= len(samples) < 25024
            assert np.sqrt(np.mean(samples[1000:5000].astype(float) ** 2)) < 32
            for channel in (0, 1):
                assert np.corrcoef(samples[6500:11000, channel], expected[6500:11000, channel])[0, 1] > 0.97

        # Both single and batch console owners accept the audio decoder flag,
        # including paths with spaces, without constructor keyword rewrites.
        source = root / "scene.py"
        source.write_text("from manimlib import *\nclass First(Scene):\n"
                          "    def construct(self):\n"
                          f"        self.add_sound({str(fixtures / 'mp3.asset')!r})\n"
                          "        self.wait(0.25)\nclass Second(First):\n    pass\n")
        for names, target in [(["First"], root / "console.wav"),
                              (["First", "Second"], root / "batch")]:
            stdout, stderr = io.StringIO(), io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                code = try_render_cli(m, ["--robot", str(source), *names, "--format", "wav",
                                         "--fps", "16", "--threads", "1", "--ffmpeg_bin", str(explicit),
                                         "--video_dir", str(target)])
            assert code == 0, (stdout.getvalue(), stderr.getvalue())
            report = json.loads(stdout.getvalue())
            assert report["exit"]["code"] == 0
            if len(names) == 1:
                assert len(report["audio_inputs"]) == 1 and len(report["ffmpeg_invocations"]) == 1
            else:
                assert (target / "First.wav").is_file() and (target / "Second.wav").is_file()
        print("portal-audio: eight compressed formats, 1/4-thread mixes, WAV/movie/console/batch and failure preservation verified")
