"""Native-pose protocol tests; fixture storage is not renderer acceptance."""
import ast
import copy
import importlib.util
import math
from pathlib import Path
import unittest
import numpy as np

SOURCE = Path(__file__).resolve().parents[1] / "python" / "manimlib" / "_animation_semantics.py"
spec = importlib.util.spec_from_file_location("camera_semantics_under_test", SOURCE)
semantics = importlib.util.module_from_spec(spec)
spec.loader.exec_module(semantics)


class Core:
    def __init__(self, center=(0., 0., 0.), shape=(16., 8.), fovy=math.pi / 4., orientation=(0., 0., 0., 1.)):
        self._center, self._shape, self._fovy, self._orientation = tuple(center), tuple(shape), fovy, tuple(orientation)
        self.writes = []
    def __copy__(self):
        return Core(self._center, self._shape, self._fovy, self._orientation)
    def center(self):
        return self._center
    def shape(self):
        return self._shape
    def field_of_view(self):
        return self._fovy
    def orientation(self):
        return self._orientation
    def set_center(self, value):
        if len(value) != 3 or not np.isfinite(value).all():
            raise ValueError("nonfinite center")
        self.writes.append("center")
        self._center = tuple(value)
    def set_shape(self, value):
        if len(value) != 2 or not np.isfinite(value).all() or min(value) <= 0:
            raise ValueError("invalid dimensions")
        self.writes.append("shape")
        self._shape = tuple(value)
    def set_field_of_view(self, value):
        if not math.isfinite(value) or not 0 < value < math.pi:
            raise ValueError("invalid field of view")
        self.writes.append("fovy")
        self._fovy = float(value)
    def set_orientation(self, value):
        if len(value) != 4 or not np.isfinite(value).all() or np.linalg.norm(value) == 0:
            raise ValueError("invalid quaternion")
        self.writes.append("orientation")
        self._orientation = tuple(np.asarray(value) / np.linalg.norm(value))


def environment():
    class CameraFrame:
        def __init__(self, **kwargs):
            self._core = Core(**kwargs)
            self.locked_data_keys, self.locked_uniform_keys = set(), set()
            self.revisions = 0
        def note_changed_data(self):
            self.revisions += 1
    return {"CameraFrame": CameraFrame, "_np": np, "_copy": copy,
            "_interpolate": lambda a, b, t: (1 - t) * a + t * b,
            "straight_path": lambda a, b, t: (1 - t) * a + t * b}


def pose(frame):
    core = frame._core
    return core.center(), core.shape(), core.field_of_view(), core.orientation()


class CameraPoseTests(unittest.TestCase):
    def setUp(self):
        self.g = environment()
        semantics._install_camera_pose(self.g)
        self.Frame = self.g["CameraFrame"]
        self.start = self.Frame()
        self.end = self.Frame(center=(4., 2., -2.), shape=(8., 4.), fovy=math.pi / 2., orientation=(0., 0., 1., 0.))
        self.live = self.Frame()
    def test_public_identity_is_preserved(self):
        cls = self.Frame
        semantics._install_camera_pose(self.g)
        self.assertIs(self.g["CameraFrame"], cls)
        self.assertEqual(cls.interpolate.__qualname__, cls.__qualname__ + ".interpolate")
    def test_shared_initializer_calls_pose_installer(self):
        tree = ast.parse(SOURCE.read_text())
        install = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "install")
        self.assertTrue(any(isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
                            and node.func.id == "_install_camera_pose" for node in ast.walk(install)))
    def test_pan_zoom_fov_and_orientation_midpoint(self):
        core = self.live._core
        self.assertIs(self.live.interpolate(self.start, self.end, .5), self.live)
        self.assertIs(self.live._core, core)
        self.assertEqual(core.center(), (2., 1., -1.))
        self.assertEqual(core.shape(), (12., 6.))
        self.assertAlmostEqual(core.field_of_view(), 3 * math.pi / 8.)
        np.testing.assert_allclose(core.orientation(), (0., 0., math.sqrt(.5), math.sqrt(.5)))
    def test_endpoints(self):
        for alpha, expected in [(0., self.start), (1., self.end)]:
            self.live.interpolate(self.start, self.end, alpha)
            self.assertEqual(pose(self.live), pose(expected))
    def test_extrapolation_is_not_silently_clamped(self):
        self.live.interpolate(self.start, self.end, 1.5)
        self.assertEqual(self.live._core.center(), (6., 3., -3.))
        self.assertEqual(self.live._core.shape(), (4., 2.))
    def test_path_receives_five_reference_control_points(self):
        calls = []
        def path(a, b, alpha):
            calls.append((a.copy(), b.copy(), alpha))
            return (1 - alpha) * a + alpha * b
        self.live.interpolate(self.start, self.end, .25, path)
        a, b, alpha = calls[0]
        self.assertEqual(alpha, .25)
        np.testing.assert_array_equal(a, [[0, 0, 0], [-8, 0, 0], [8, 0, 0], [0, -4, 0], [0, 4, 0]])
        self.assertEqual(b.shape, (5, 3))
    def test_closed_excursion_with_equal_endpoint_pose(self):
        def path(a, b, alpha):
            return (1 - alpha) * a + alpha * b + [0., 4 * alpha * (1 - alpha), 0.]
        self.live.interpolate(self.start, self.start, .5, path)
        self.assertEqual(self.live._core.center(), (0., 1., 0.))
        self.live.interpolate(self.start, self.start, 1., path)
        self.assertEqual(pose(self.live), pose(self.start))
    def test_custom_path_controls_frame_extents(self):
        def path(a, b, alpha):
            result = (1 - alpha) * a + alpha * b
            result[1, 0] -= alpha
            result[2, 0] += alpha
            return result
        self.live.interpolate(self.start, self.end, .5, path)
        self.assertEqual(self.live._core.shape(), (13., 6.))
    def test_callback_cannot_mutate_endpoint_storage_through_arrays(self):
        before = pose(self.start), pose(self.end)
        def path(a, b, alpha):
            a[:] = b
            return a
        self.live.interpolate(self.start, self.end, .5, path)
        self.assertEqual((pose(self.start), pose(self.end)), before)
    def test_point_lock_preserves_pan_and_zoom_but_not_orientation(self):
        self.live.locked_data_keys.add("point")
        self.live.interpolate(self.start, self.end, .5, lambda *args: self.fail("locked path called"))
        self.assertEqual(self.live._core.center(), (0., 0., 0.))
        self.assertEqual(self.live._core.shape(), (16., 8.))
        self.assertNotEqual(self.live._core.orientation(), self.start._core.orientation())
        self.assertNotIn("center", self.live._core.writes)
    def test_uniform_locks_do_not_rewrite_locked_values(self):
        self.live.locked_uniform_keys.update(("orientation", "fovy"))
        self.live.interpolate(self.start, self.end, .5)
        self.assertEqual(self.live._core.writes, ["center", "shape"])
        self.assertEqual(self.live._core.orientation(), self.start._core.orientation())
        self.assertEqual(self.live._core.field_of_view(), self.start._core.field_of_view())
    def test_all_locks_do_not_dirty_pose(self):
        self.live.locked_data_keys.add("point")
        self.live.locked_uniform_keys.update(("orientation", "fovy"))
        self.live.interpolate(self.start, self.end, .5)
        self.assertEqual(self.live._core.writes, [])
        self.assertEqual(self.live.revisions, 0)
    def test_quaternion_normalization_happens_once_on_live_input(self):
        calls = []
        original = self.live._core.set_orientation
        self.live._core.set_orientation = lambda value: (calls.append(value), original(value))[-1]
        self.live.interpolate(self.start, self.end, .5)
        self.assertEqual(calls, [(0., 0., .5, .5)])
    def test_wrong_endpoint_type_is_rejected(self):
        with self.assertRaisesRegex(TypeError, "CameraFrame endpoints"):
            self.live.interpolate(self.start, object(), .5)
    def test_nonfinite_alpha_is_rejected(self):
        for alpha in (math.nan, math.inf, -math.inf):
            with self.assertRaisesRegex(ValueError, "alpha must be finite"):
                self.live.interpolate(self.start, self.end, alpha)
        self.assertEqual(self.live._core.writes, [])
    def test_noncallable_path_is_rejected(self):
        with self.assertRaisesRegex(TypeError, "must be callable"):
            self.live.interpolate(self.start, self.end, .5, 7)
    def test_bad_path_shapes_and_nonfinite_output_are_atomic(self):
        before = pose(self.live)
        for result in [np.zeros(3), np.zeros((4, 3)), np.full((5, 3), math.nan), np.full((5, 3), math.inf)]:
            with self.assertRaisesRegex(ValueError, "finite \\(5, 3\\)"):
                self.live.interpolate(self.start, self.end, .5, lambda *args: result)
            self.assertEqual(pose(self.live), before)
        self.assertEqual(self.live._core.writes, [])
    def test_invalid_dimensions_are_atomic(self):
        before = pose(self.live)
        with self.assertRaisesRegex(ValueError, "dimensions"):
            self.live.interpolate(self.start, self.end, .5, lambda a, b, t: np.zeros((5, 3)))
        self.assertEqual(pose(self.live), before)
        self.assertEqual(self.live._core.writes, [])
    def test_invalid_field_of_view_is_atomic_after_valid_pan(self):
        before = pose(self.live)
        end = self.Frame(center=(100, 0, 0), fovy=3.)
        with self.assertRaisesRegex(ValueError, "field of view"):
            self.live.interpolate(self.start, end, 2.)
        self.assertEqual(pose(self.live), before)
        self.assertEqual(self.live._core.writes, [])
    def test_invalid_quaternion_is_atomic_after_other_valid_fields(self):
        before = pose(self.live)
        end = self.Frame(center=(100, 0, 0), orientation=(0., 0., 0., -1.))
        with self.assertRaisesRegex(ValueError, "quaternion"):
            self.live.interpolate(self.start, end, .5)
        self.assertEqual(pose(self.live), before)
        self.assertEqual(self.live._core.writes, [])
    def test_real_transform_begin_does_not_lock_empty_camera_records(self):
        from types import SimpleNamespace
        tree = ast.parse(SOURCE.read_text())
        install = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "install")
        begin = next(node for node in install.body if isinstance(node, ast.FunctionDef) and node.name == "transform_begin")
        def initialize(animation):
            animation.starting_mobject = copy.deepcopy(animation.mobject)
            animation.interpolate(0.)
        namespace = {"g": self.g, "Animation": SimpleNamespace(begin=initialize), "uses_python_path": lambda a: False}
        exec(compile(ast.Module(body=[begin], type_ignores=[]), str(SOURCE), "exec"), namespace)
        self.Frame.is_aligned_with = lambda frame, other: True
        self.Frame.align_data_and_family = lambda *args: None
        self.Frame.has_updaters = lambda frame: False
        self.Frame.lock_matching_data = lambda frame, *args: frame.locked_data_keys.add("point")
        animation = SimpleNamespace(mobject=self.live, target_mobject=self.end,
                                    _ensure_runtime_defaults=lambda: None, init_path_func=lambda: None,
                                    create_target=lambda: self.end, check_target_mobject_validity=lambda: None)
        animation.interpolate = lambda alpha: self.live.interpolate(animation.starting_mobject, self.end, alpha)
        namespace["transform_begin"](animation)
        animation.interpolate(.5)
        self.assertEqual(self.live._core.center(), (2., 1., -1.))
        self.assertNotIn("point", self.live.locked_data_keys)

    def test_path_exception_is_not_replaced_and_is_atomic(self):
        error = RuntimeError("authored path failed")
        def path(*args):
            raise error
        with self.assertRaises(RuntimeError) as caught:
            self.live.interpolate(self.start, self.end, .5, path)
        self.assertIs(caught.exception, error)
        self.assertEqual(self.live._core.writes, [])


if __name__ == "__main__":
    unittest.main()
