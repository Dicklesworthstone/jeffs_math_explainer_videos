"""Actual native camera capture, canonical replay and direct-render comparison.

The embedded Rust test provides the two native decode/render helpers. The
installed CLI companion imports Authored from this file without running these
helper-dependent checks. No native engine or binding is substituted.
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
        self.callback_calls = 0
        self.frame.shift((0.2, -0.1, 0.0)).scale(0.6).rotate(0.4, axis=m.RIGHT)
        self.frame.set_field_of_view(0.9)
        self.camera.background_color = "#152536"
        self.camera.light_source.move_to((-3, 4, 6))
        cube = m.Cube(side_length=1.5, color=m.BLUE)
        marker = m.Square(side_length=0.4, fill_opacity=1, stroke_width=0, color=m.YELLOW)
        marker.shift((1.2, 0.3, 0.5))
        def follow(mob, dt):
            self.callback_calls += 1
            mob.move_to((1.2, 0.3 + self.time, 0.5))
        marker.add_updater(follow)
        self.add(cube, marker)
        # A noncatalog callable uses the existing callback-driven camera route.
        self.play(m.Rotate(self.frame, angle=0.9, axis=m.UP),
                  run_time=0.25, rate_func=UnhashableRate())
        self.camera.background_color = "#361525"
        self.camera.light_source.move_to((5, 3, 4))
        self.wait(0.125)
        self.wait(0)


class Static(m.Scene):
    def construct(self):
        self.frame.rotate(0.7, axis=m.RIGHT).rotate(0.4, axis=m.UP).scale(0.6)
        self.add(m.Cube(side_length=1.5, color=m.YELLOW))


def native_acceptance(replay_frames, png_pixels):
    root = Path(tempfile.mkdtemp(prefix="fmn-camera-bundle-native-"))
    print(f"retaining camera bundle evidence: {root}")
    for fps in (8, 24):
        scene = Authored()
        destination = root / f"camera-{fps}.fmtl"
        receipt = export_bundle(scene, destination, camera=True, resolution=(96, 54), fps=fps)
        data = destination.read_bytes()
        assert receipt.frame_count == 3 * fps // 8
        assert receipt.segment_count == 3
        assert receipt.camera_track and receipt.as_dict()["fmtl_minor"] == 1
        assert receipt.bytes == len(data)
        assert receipt.digest == hashlib.sha256(data).hexdigest()
        indices = list(range(receipt.frame_count))
        replay = replay_frames(data, indices, 96, 54, 1)
        assert replay[0] != replay[1], "camera motion did not reach the pixels"
        assert replay[0] != replay[-1], "view/background changes disappeared"
        assert replay == replay_frames(data, indices, 96, 54, 4)
        assert list(reversed(replay)) == replay_frames(data, indices[::-1], 96, 54, 2)
        calls = scene.callback_calls
        assert calls > 0
        replay_frames(data, [0, len(indices) - 1, 0], 96, 54, 1)
        assert scene.callback_calls == calls, "replay invoked authored callbacks"
        direct = root / f"direct-{fps}"
        render_scene(Authored, direct, format="png_sequence", resolution=(96, 54), fps=fps, threads=1)
        assert replay == [png_pixels(str(path)) for path in sorted(direct.glob("*.png"))]
        again = root / f"again-{fps}.fmtl"
        assert export_bundle(Authored, again, camera=True, resolution=(96, 54), fps=fps).digest == receipt.digest
        exact = export_bundle(Authored, root / f"exact-{fps}.fmtl", camera=True,
                              resolution=(96, 54), fps=fps, max_output_bytes=len(data))
        assert exact.bytes == len(data)
        small = root / f"small-{fps}.fmtl"
        try:
            export_bundle(Authored, small, camera=True, resolution=(96, 54), fps=fps,
                          max_output_bytes=len(data) - 1)
        except RuntimeError:
            pass
        else:
            raise AssertionError("camera export bypassed the output cap")
        assert not small.exists()

    for cls in (Static, m.Scene):
        path = root / (cls.__name__ + ".fmtl")
        receipt = export_bundle(cls, path, camera=True, resolution=(96, 54), fps=8)
        assert receipt.frame_count == receipt.segment_count == 1
        replay = replay_frames(path.read_bytes(), [0], 96, 54, 1)
        direct = root / (cls.__name__ + "-direct")
        render_scene(cls, direct, format="png_sequence", resolution=(96, 54), fps=8, threads=1)
        assert replay == [png_pixels(str(p)) for p in sorted(direct.glob("*.png"))]

    # The existing browser-compatible default still refuses the uncarried view.
    flat = root / "flat.fmtl"
    try:
        export_bundle(Static, flat, resolution=(96, 54), fps=8)
    except (m._CapabilityError, RuntimeError):
        pass
    else:
        raise AssertionError("planar export dropped a camera")
    assert not flat.exists()

    class Swallowed(m.Scene):
        def construct(self):
            self.frame.rotate(0.2)
            try:
                self.wait(0.5)
            except RuntimeError:
                pass
    partial = root / "partial.fmtl"
    try:
        export_bundle(Swallowed, partial, camera=True, fps=8, max_frames=1)
    except RuntimeError:
        pass
    else:
        raise AssertionError("swallowed camera capture failure published partial output")
    assert not partial.exists()

    audio = root / "audio.fmtl"
    try:
        with BundleExportSession(m.Scene(), audio, camera=True, fps=8) as session:
            session.scene.add_sound("not-decoded-for-bundle.wav")
    except (m._CapabilityError, RuntimeError):
        pass
    else:
        raise AssertionError("camera mode silently stripped audio")
    assert not audio.exists()

    protected = root / "protected.fmtl"
    protected.write_bytes(b"user artifact")
    try:
        export_bundle(Static, protected, camera=True)
    except FileExistsError:
        pass
    else:
        raise AssertionError("camera export overwrote an existing output")
    assert protected.read_bytes() == b"user artifact"
    raced = root / "raced.fmtl"
    try:
        with BundleExportSession(m.Scene(), raced, camera=True) as session:
            session.scene.add(m.Cube())
            raced.write_bytes(b"concurrent winner")
    except FileExistsError:
        pass
    else:
        raise AssertionError("camera export won an occupied destination")
    assert raced.read_bytes() == b"concurrent winner"


# Embedded execution supplies real native helpers. Imported scene definitions
# are shared with the installed-wheel/CLI acceptance program.
if "_test_camera_bundle_frames" in globals():
    native_acceptance(_test_camera_bundle_frames, _test_png_pixels)
elif __name__ == "__main__":
    raise RuntimeError("run via the embedded native suite or camera_bundle_cli_native.py")
