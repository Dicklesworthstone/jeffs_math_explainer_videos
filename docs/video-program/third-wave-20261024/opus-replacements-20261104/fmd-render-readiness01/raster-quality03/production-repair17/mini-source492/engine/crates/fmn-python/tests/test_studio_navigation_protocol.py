"""Run the shipped camera gateway against storage-only native-operation doubles.

Real projected pixels and worker key transitions are checked by studio_live.py.
These tests protect the opt-in boundary and existing host-window precedence.
"""
import ast
from pathlib import Path
from types import SimpleNamespace
import unittest

import numpy as np


class StudioCameraGatewayTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        source = Path(__file__).parents[1] / "python" / "manimlib_bootstrap.py"
        tree = ast.parse(source.read_text())
        scene = next(n for n in tree.body if isinstance(n, ast.ClassDef) and n.name == "Scene")
        motion = next(n for n in scene.body if isinstance(n, ast.FunctionDef) and n.name == "on_mouse_motion")
        cls.code = compile(ast.Module(body=[motion], type_ignores=[]), str(source), "exec")

    def setUp(self):
        self.keys = set()
        self.calls = []
        self.stopped = False
        dispatcher = SimpleNamespace(is_key_pressed=lambda key: key in self.keys)
        globals = dict(_np=np, EventType=SimpleNamespace(MouseMotionEvent="motion"),
                       _scene_event_stopped=lambda *a, **kw: self.stopped,
                       _event_dispatcher=lambda: dispatcher,
                       _pinned_manim_config=lambda: SimpleNamespace(
                           key_bindings=SimpleNamespace(pan="f", pan_3d="d")))
        exec(self.code, globals)
        self.motion = globals["on_mouse_motion"]
        frame = SimpleNamespace(shift=lambda p: self.calls.append(("shift", p.tolist())),
                                to_fixed_frame_point=lambda p, relative: np.asarray(p),
                                increment_theta=lambda v: self.calls.append(("theta", v)),
                                increment_phi=lambda v: self.calls.append(("phi", v)))
        self.scene = SimpleNamespace(window=None, frame=frame, pan_sensitivity=0.5,
                                     mouse_point=SimpleNamespace(move_to=lambda p: None))
        self.scene.get_window = lambda: self.scene.window

    def move(self):
        self.motion(self.scene, np.array([1., 2., 3.]), np.array([2., 4., 0.]))

    def test_offline_hover_does_not_consume_live_dispatcher_key_state(self):
        self.keys.add(ord("f"))
        self.move()
        self.assertEqual(self.calls, [])

    def test_live_pan_and_orbit_reuse_existing_native_camera_operations(self):
        self.scene._fmn_studio_live_input = True
        self.keys.add(ord("f"))
        self.move()
        self.assertEqual(self.calls, [("shift", [-2., -4., 0.])])
        self.keys.clear()
        self.move()
        self.assertEqual(len(self.calls), 1)
        self.keys.add(ord("d"))
        self.move()
        self.assertEqual(self.calls[1:], [("theta", -1.), ("phi", 2.)])

    def test_existing_host_window_remains_the_key_authority(self):
        self.scene._fmn_studio_live_input = True
        self.scene.window = SimpleNamespace(is_key_pressed=lambda key: False)
        self.keys.add(ord("f"))
        self.move()
        self.assertEqual(self.calls, [])

    def test_consumed_motion_does_not_navigate(self):
        self.scene._fmn_studio_live_input = True
        self.keys.add(ord("d"))
        self.stopped = True
        self.move()
        self.assertEqual(self.calls, [])


if __name__ == "__main__":
    unittest.main()
