"""Real engine state must survive reentrant SceneFileWriter negotiation."""
import os
import pathlib
import tempfile

import manimlib as m
from fmn_python import render_scene, render_session


class MutatingWriter:
    def __init__(self, original, operation):
        self.original, self.operation = original, operation

    def __getattr__(self, name):
        if name == "video_codec":
            self.operation()
        return getattr(self.original, name)


with tempfile.TemporaryDirectory(prefix="fmn-video-ownership-") as directory:
    root = pathlib.Path(directory)
    old_path = os.environ.get("PATH")
    os.environ["PATH"] = str(root)
    try:
        for mutation in ("adopt", "advance"):
            scene = m.Scene()
            square = m.Square()
            operation = (lambda: scene.add(square)) if mutation == "adopt" else (
                lambda: scene.increment_time(0.125))
            scene.file_writer = MutatingWriter(scene.file_writer, operation)
            target = root / (mutation + ".mp4")
            target.write_bytes(b"prior artifact")
            try:
                render_scene(scene, target, resolution=(96, 54), fps=8, threads=1)
            except RuntimeError as error:
                assert "mutates engine state" in str(error), str(error)
            else:
                raise AssertionError("writer callback invalidated an accepted pristine scene")
            assert target.read_bytes() == b"prior artifact"
            assert scene.root_count() == (1 if mutation == "adopt" else 0)
            assert scene.time == (0 if mutation == "adopt" else 0.125)
            assert not hasattr(scene, "_fmn_owned_render_session")

        # An already-owned, otherwise pristine generation must be rejected
        # before an unavailable ffmpeg lookup, and must remain publishable.
        scene = m.Scene()
        png = root / "owner.png"
        with render_session(scene, png, resolution=(96, 54), fps=8, threads=1) as owner:
            try:
                scene._begin_native_output(str(root / "intruder.mp4"), "mp4", 96, 54, 8, 1, 0)
            except RuntimeError as error:
                assert "already active" in str(error), str(error)
            else:
                raise AssertionError("a second output acquired an owned generation")
            assert scene._fmn_owned_render_session is owner
            scene.add(m.Square(fill_opacity=1))
        assert owner.result.frame_count == 1 and png.read_bytes().startswith(b"\x89PNG")
        assert not (root / "intruder.mp4").exists()
    finally:
        if old_path is None:
            os.environ.pop("PATH", None)
        else:
            os.environ["PATH"] = old_path
