"""Real installed Python camera export -> standalone native FMTL -> Y4M.

Requires a matching wheel and FMN_TEST_BIN; no native boundary is replaced.
The standalone process has an empty PATH and cannot locate host Python.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from fmn_python import export_bundle, render_scene
from camera_bundle_export import Authored


ROOT = Path(tempfile.mkdtemp(prefix="fmn-camera-bundle-cli-"))
print(f"retaining camera bundle CLI evidence: {ROOT}")
BINARY = Path(os.environ["FMN_TEST_BIN"]).resolve(strict=True)
ENV = {"PATH": ""}
if "SystemRoot" in os.environ:
    ENV["SystemRoot"] = os.environ["SystemRoot"]

for fps in (8, 24):
    artifact = ROOT / f"camera-{fps}.fmtl"
    authored = Authored()
    receipt = export_bundle(authored, artifact, camera=True, resolution=(96, 54), fps=fps)
    assert receipt.frame_count == 3 * fps // 8
    assert receipt.camera_track
    data = artifact.read_bytes()
    assert receipt.digest == hashlib.sha256(data).hexdigest()
    calls = authored.callback_calls
    assert calls > 0
    outputs = []
    for threads in (1, 4):
        directory = ROOT / f"native-{fps}-{threads}"
        argv = [str(BINARY), "--robot", str(artifact), "--format", "y4m",
                "--resolution", "96x54", "--fps", str(fps), "--threads", str(threads),
                "--video_dir", str(directory)]
        result = subprocess.run(argv, env=ENV, cwd=ROOT, stdin=subprocess.DEVNULL,
                                capture_output=True, text=True, timeout=180)
        assert result.returncode == 0, (argv, result.stdout, result.stderr)
        assert not result.stderr, result.stderr
        assert len(result.stdout.splitlines()) == 1, result.stdout
        report = json.loads(result.stdout)
        assert report["frames"] == receipt.frame_count
        assert report["source"] == "compiled"
        movie = (directory / f"camera-{fps}.y4m").read_bytes()
        header, body = movie.split(b"\n", 1)
        assert header == f"YUV4MPEG2 W96 H54 F{fps}:1 Ip A1:1 C420mpeg2".encode()
        size = 96 * 54 * 3 // 2
        assert len(body) == receipt.frame_count * (size + 6)
        assert body[:6] == b"FRAME\n"
        assert body[6:6 + size] != body[-size:], "animated view disappeared"
        assert authored.callback_calls == calls, "replay called authored scene code"
        outputs.append(movie)
    assert outputs[0] == outputs[1], "renderer worker count changed recorded views"
    direct = ROOT / f"direct-{fps}.y4m"
    render_scene(Authored, direct, format="y4m", resolution=(96, 54), fps=fps, threads=1)
    assert direct.read_bytes() == outputs[0], "camera bundle replay differs from direct output"

print("camera-bearing FMTL authoring, native CLI replay and direct Y4M equality passed")
