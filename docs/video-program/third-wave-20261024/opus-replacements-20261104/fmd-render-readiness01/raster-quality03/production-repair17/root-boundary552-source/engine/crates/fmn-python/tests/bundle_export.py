"""Real native Python capture -> FMTL reader -> Lumen acceptance.

Executed by portal_studio::bundle::tests with two test-only native reader/
renderer functions. No native operation is mocked and no format is decoded in
Python. PNG comparison is on decoded RGBA, independent of compression level.
"""
from pathlib import Path
import hashlib
import tempfile

import manimlib as m
from fmn_python import BundleExportSession, export_bundle, render_scene


class UnhashableRate:
    __hash__ = None
    def __call__(self, alpha):
        return alpha * alpha


class Authored(m.Scene):
    def construct(self):
        self.calls = 0
        moving = m.Square(side_length=1, fill_opacity=1, stroke_width=0)
        moving.shift(2 * m.LEFT)
        follower = m.Square(side_length=0.4, fill_opacity=1, stroke_width=0)
        def follow(mob, dt):
            self.calls += 1
            mob.move_to(moving.get_center() + m.UP)
        follower.add_updater(follow)
        self.add(moving, follower)
        self.play(moving.animate.shift(3 * m.RIGHT), run_time=0.125,
                  rate_func=UnhashableRate())
        self.wait(0.125)
        self.play()  # no phantom segment
        self.wait(0)  # a real zero-frame segment


with tempfile.TemporaryDirectory(prefix="fmn-python-fmtl-") as temporary:
    root = Path(temporary)
    for fps in (24, 30, 60):
        scene = Authored()
        destination = root / f"authored-{fps}.fmtl"
        receipt = export_bundle(scene, destination, resolution=(96, 54), fps=fps)
        data = destination.read_bytes()
        assert receipt.digest == hashlib.sha256(data).hexdigest()
        assert receipt.bytes == len(data)
        assert receipt.frame_count == 2 * ((fps + 7) // 8)
        assert receipt.segment_count == 3
        assert receipt.as_dict()["certified_source"] is False
        calls = scene.calls
        indices = list(range(receipt.frame_count))
        replay = _test_bundle_frames(data, indices, 96, 54, 1)
        assert replay == _test_bundle_frames(data, indices, 96, 54, 4)
        assert list(reversed(replay)) == _test_bundle_frames(data, list(reversed(indices)), 96, 54, 2)
        assert scene.calls == calls, "replay executed authored callbacks"
        assert replay[0] != replay[-1], "animation did not change its visible output"
        # Deterministic authoring and independent direct renderer execution.
        again = root / f"again-{fps}.fmtl"
        assert export_bundle(Authored, again, resolution=(96, 54), fps=fps).digest == receipt.digest
        direct = root / f"direct-{fps}"
        render_scene(Authored, direct, format="png_sequence", resolution=(96, 54), fps=fps, threads=1)
        assert replay == [_test_png_pixels(str(path)) for path in sorted(direct.glob("*.png"))]
        exact = export_bundle(Authored, root / f"exact-{fps}.fmtl", resolution=(96, 54), fps=fps,
                              max_output_bytes=len(data))
        assert exact.bytes == len(data)
        too_small = root / f"too-small-{fps}.fmtl"
        try:
            export_bundle(Authored, too_small, resolution=(96, 54), fps=fps,
                          max_output_bytes=len(data) - 1)
        except RuntimeError:
            pass
        else:
            raise AssertionError("output budget did not reject encoding")
        assert not too_small.exists()

    class Static(m.Scene):
        def construct(self):
            self.add(m.Square(fill_opacity=1))
    static = export_bundle(Static, root / "static.fmtl", resolution=(96, 54), fps=30)
    assert static.frame_count == static.segment_count == 1

    class Ends(m.Scene):
        def construct(self):
            self.add(m.Square())
            self.wait(0.125)
            raise m.EndScene()
    assert export_bundle(Ends, root / "end.fmtl", fps=24).frame_count == 3

    class Swallows(m.Scene):
        def construct(self):
            try:
                self.wait(0.25)
            except RuntimeError:
                pass
    path = root / "partial.fmtl"
    try:
        export_bundle(Swallows, path, max_frames=1, fps=24)
    except RuntimeError:
        pass
    else:
        raise AssertionError("swallowed capture failure published a partial bundle")
    assert not path.exists()

    for name, change in (("camera shift", lambda scene: scene.camera.frame.shift(m.RIGHT)),
                         ("depth", lambda scene: scene.add(m.Square().shift(m.OUT))),
                         ("audio", lambda scene: scene.add_sound("not-decoded-for-fmtl.wav"))):
        path = root / "unsupported.fmtl"
        try:
            with BundleExportSession(m.Scene(), path, resolution=(96, 54), fps=24) as session:
                change(session.scene)
        except (m._CapabilityError, RuntimeError):
            pass
        else:
            raise AssertionError(f"unrepresentable side channel was dropped: {name}")
        assert not path.exists()

    protected = root / "protected.fmtl"
    protected.write_bytes(b"existing user artifact")
    try:
        export_bundle(Static, protected)
    except FileExistsError:
        pass
    else:
        raise AssertionError("export clobbered an existing artifact")
    assert protected.read_bytes() == b"existing user artifact"

    # The final native atomic create, not only begin-time probing, is authoritative.
    raced = root / "raced.fmtl"
    try:
        with BundleExportSession(m.Scene(), raced) as session:
            session.scene.add(m.Square())
            raced.write_bytes(b"concurrent winner")
    except FileExistsError:
        pass
    else:
        raise AssertionError("publication race overwrote another writer")
    assert raced.read_bytes() == b"concurrent winner"
