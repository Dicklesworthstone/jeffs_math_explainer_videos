"""Real Python/IPython orchestration with explicit native image/scene doubles."""
import threading
import unittest

try:
    from IPython.core.formatters import DisplayFormatter
    HAVE_IPYTHON = True
except ImportError:
    HAVE_IPYTHON = False

import test_camera_capture_protocol as camera_fixture
from test_scene_console_protocol import Scene, NATIVE
from fmn_python import SceneConsole


class RichCameraProtocol(unittest.TestCase):
    def setUp(self):
        base = camera_fixture.CameraProtocol()
        base.setUp()
        self.camera, self.native, self.calls = base.camera, base.native, base.calls
        self.native._CameraCapture.png = lambda capture: b"native png fixture"
        self.native._CameraCapture._repr_png_ = lambda capture: capture.png()

    def test_capture_snapshot_returns_exact_new_native_value(self):
        snapshot = self.camera.capture_snapshot(self.native.Mobject())
        self.assertIs(snapshot, vars(self.camera)["_fmn_camera_capture"])
        self.assertIsNone(self.camera.capture())
        self.assertIsNot(snapshot, vars(self.camera)["_fmn_camera_capture"])
        self.assertEqual(snapshot.png(), b"native png fixture")

    def test_png_and_notebook_display_never_reinvoke_capture_callbacks(self):
        self.camera.capture()
        before = len(self.calls), self.camera.refreshes
        for _ in range(3):
            self.assertEqual(self.camera.get_png(), b"native png fixture")
            self.assertEqual(self.camera._repr_png_(), b"native png fixture")
        self.assertEqual(before, (len(self.calls), self.camera.refreshes))

    @unittest.skipUnless(HAVE_IPYTHON, "IPython is not installed")
    def test_real_ipython_png_formatter_uses_native_snapshot_protocol(self):
        snapshot = self.camera.capture_snapshot()
        formatter = DisplayFormatter()
        data, metadata = formatter.format(snapshot, include=["image/png"])
        self.assertEqual(data["image/png"], b"native png fixture")
        self.assertEqual(metadata, {})
        self.assertEqual(len(self.calls), 1)

    def test_png_failure_keeps_existing_native_capture(self):
        snapshot = self.camera.capture_snapshot()
        failure = ValueError("native encoder fixture")
        def fail(value):
            raise failure
        self.native._CameraCapture.png = fail
        with self.assertRaises(ValueError) as raised:
            self.camera.get_png()
        self.assertIs(raised.exception, failure)
        self.assertIs(vars(self.camera)["_fmn_camera_capture"], snapshot)
        self.assertEqual(len(self.calls), 1)


class RichConsoleProtocol(unittest.TestCase):
    def setUp(self):
        self.scene = Scene()
        self.snapshot = object()
        def capture(*roots):
            self.scene.camera.capture(*roots)
            return self.snapshot
        self.scene.camera.capture_snapshot = capture
        self.scene.camera.get_png = lambda: b"native png fixture"
        self.console = SceneConsole(self.scene, _native=NATIVE)
        self.addCleanup(self.console.close)

    def test_preview_is_observational_without_cell_or_checkpoint(self):
        before = self.scene.time, self.scene.num_plays, self.scene.rng
        self.assertIs(self.console.preview(), self.snapshot)
        self.assertEqual(before, (self.scene.time, self.scene.num_plays, self.scene.rng))
        self.assertEqual(self.scene.trace, [])
        self.assertEqual(self.console._cells, 0)
        self.assertEqual(self.console.checkpoint_manager.checkpoint_states, {})
        self.assertEqual(self.scene.camera.captures, [tuple(self.scene.mobjects)])

    def test_explicit_preview_is_available_with_automatic_capture_disabled(self):
        self.console.capture = False
        self.assertIsNone(self.console._repr_png_())
        self.assertIs(self.console.preview(), self.snapshot)
        self.assertEqual(self.scene.trace, [])

    @unittest.skipUnless(HAVE_IPYTHON, "IPython is not installed")
    def test_display_does_not_acquire_console_or_refresh_scene(self):
        formatter = DisplayFormatter()
        data, _ = formatter.format(self.console, include=["image/png"])
        self.assertEqual(data["image/png"], b"native png fixture")
        self.assertNotIn("_fmn_scene_console", vars(self.scene))
        self.assertEqual(self.scene.camera.captures, [])
        self.assertEqual(self.scene.trace, [])

    def test_busy_closed_and_foreign_thread_displays_do_not_touch_native_scene(self):
        def forbidden():
            raise AssertionError("read native Scene from representation")
        self.scene.camera.get_png = forbidden
        self.console._busy = True
        self.assertIsNone(self.console._repr_png_())
        self.console._busy = False
        results = []
        worker = threading.Thread(target=lambda: results.append(self.console._repr_png_()))
        worker.start()
        worker.join()
        self.assertEqual(results, [None])
        self.console.close()
        self.assertIsNone(self.console._repr_png_())

    def test_preview_failure_preserves_exception_and_releases_busy(self):
        error = RuntimeError("native capture fixture")
        self.scene.camera.error = error
        with self.assertRaises(RuntimeError) as raised:
            self.console.preview()
        self.assertIs(raised.exception, error)
        self.assertFalse(self.console._busy)
        self.assertEqual(self.scene.trace, [])
        self.scene.camera.error = None
        self.assertIs(self.console.preview(), self.snapshot)

    def test_preview_refuses_an_active_render_generation(self):
        vars(self.scene)["_fmn_owned_render_session"] = object()
        with self.assertRaisesRegex(RuntimeError, "output generation"):
            self.console.preview()
        self.assertEqual(self.scene.camera.captures, [])
        vars(self.scene).pop("_fmn_owned_render_session")


if __name__ == "__main__":
    unittest.main()
