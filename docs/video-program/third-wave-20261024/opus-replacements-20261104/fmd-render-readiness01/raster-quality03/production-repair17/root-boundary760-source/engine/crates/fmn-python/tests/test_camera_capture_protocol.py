"""Control-flow tests; actual rendering is tested by camera_readback.py."""
import importlib.util
from pathlib import Path
from types import SimpleNamespace
import unittest
import numpy as np

_PATH = Path(__file__).resolve().parents[1] / 'python/fmn_python/camera_capture.py'
_spec = importlib.util.spec_from_file_location('camera_capture_under_test', _PATH)
_adapter = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_adapter)


class CameraProtocol(unittest.TestCase):
    def setUp(self):
        class Mobject:
            def __init__(self, *children): self.submobjects = list(children)
        class Camera:
            def __init__(self):
                self._core = object()
                self.frame = SimpleNamespace(_core=object())
                self.background_rgba = [0, 0, 0, 0]
                self.light_source = SimpleNamespace(get_location=lambda: (0, 0, 1))
                self.shape = (2, 1)
                self.refreshes = 0
            def refresh_uniforms(self): self.refreshes += 1
            def get_pixel_shape(self): return self.shape
        calls = []
        class Capture:
            def __init__(self, core, frame, background, light, nodes, rows, roots, threads):
                calls.append((nodes, rows, roots, threads))
                if threads == 0: raise ValueError('refused')
                self.size = (2, 1)
            def pixels(self): return bytes(range(8))
        self.native = SimpleNamespace(Camera=Camera, Mobject=Mobject, _CameraCapture=Capture, _np=np)
        self.calls = calls
        _adapter.install_camera_capture(self.native)
        self.camera = Camera()

    def test_shared_family_is_copied_once(self):
        leaf = self.native.Mobject()
        a, b = self.native.Mobject(leaf), self.native.Mobject(leaf)
        self.camera.capture(a, b)
        nodes, rows, roots, _ = self.calls[-1]
        self.assertEqual(nodes, [a, b, leaf])
        self.assertEqual(rows, [[2], [2], []])
        self.assertEqual(roots, [0, 1])
        self.assertEqual(self.camera.refreshes, 1)

    def test_arrays_are_independent(self):
        first = self.camera.get_pixel_array()
        first[:] = 255
        self.assertEqual(self.camera.get_pixel_array().reshape(-1).tolist(), list(range(8)))
        self.assertEqual(len(self.calls), 1)

    def test_clear_has_no_uniform_callback(self):
        self.camera.clear()
        self.assertEqual(self.camera.refreshes, 0)
        self.assertEqual(self.calls[-1][:3], ([], [], []))

    def test_failure_keeps_snapshot_and_releases_busy(self):
        self.camera.capture()
        snapshot = self.camera.__dict__['_fmn_camera_capture']
        self.camera.capture_threads = 0
        with self.assertRaises(ValueError): self.camera.capture()
        self.assertIs(self.camera.__dict__['_fmn_camera_capture'], snapshot)
        self.assertFalse(self.camera.__dict__['_fmn_capture_busy'])

    def test_wrong_child_refuses_before_native_call(self):
        with self.assertRaises(TypeError): self.camera.capture(self.native.Mobject(object()))
        self.assertFalse(self.calls)

    def test_cyclic_graph_stays_bounded_and_is_passed_for_native_validation(self):
        node = self.native.Mobject()
        node.submobjects.append(node)
        nodes, rows, roots = _adapter._freeze_graph((node,), self.native.Mobject)
        self.assertEqual((len(nodes), rows, roots), (1, [[0]], [0]))

    def test_reentrant_uniform_hook_refuses_and_recovers(self):
        self.camera.refresh_uniforms = lambda: self.camera.capture()
        with self.assertRaisesRegex(RuntimeError, 'recursively'): self.camera.capture()
        self.assertFalse(self.calls)
        self.assertFalse(self.camera.__dict__['_fmn_capture_busy'])

    def test_installer_is_idempotent(self):
        method = self.native.Camera.capture
        _adapter.install_camera_capture(self.native)
        self.assertIs(self.native.Camera.capture, method)

    def test_subclass_capture_hook_remains(self):
        class Authored(self.native.Camera):
            def refresh_uniforms(self): self.refreshes += 10
        camera = Authored()
        camera.capture()
        self.assertEqual(camera.refreshes, 10)

    def test_duplicate_roots_keep_requested_order(self):
        node = self.native.Mobject()
        self.camera.capture(node, node)
        self.assertEqual(self.calls[-1][2], [0, 0])

    def test_node_budget(self):
        prior = _adapter._MAX_NODES
        _adapter._MAX_NODES = 2
        try:
            with self.assertRaisesRegex(ValueError, 'node budget'):
                self.camera.capture(self.native.Mobject(self.native.Mobject(), self.native.Mobject()))
            self.assertFalse(self.calls)
        finally:
            _adapter._MAX_NODES = prior


if __name__ == '__main__': unittest.main()
