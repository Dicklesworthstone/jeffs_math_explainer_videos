"""Installed-wheel builder playback over real native geometry and clocks."""
import numpy as np
import manimlib as m


def close(actual, expected):
    assert np.allclose(actual, expected, atol=2e-5), (actual, expected)


def authored_path():
    scene, square, samples, calls = m.Scene(), m.Square(), [], []
    scene.add(square)
    square.add_updater(lambda obj: samples.append(tuple(obj.get_center())), call=False)
    def path(start, end, alpha):
        calls.append(alpha)
        return (1 - alpha) * start + alpha * end + 4 * alpha * (1 - alpha) * m.UP
    scene.play(square.animate(run_time=.25, rate_func=m.linear, path_func=path).shift(2 * m.RIGHT))
    close(square.get_center(), 2 * m.RIGHT)
    assert calls and max(point[1] for point in samples) > .5


def final_alpha():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    scene.play(square.animate(run_time=.125, rate_func=m.linear, final_alpha_value=.25).shift(4 * m.RIGHT))
    close(square.get_center(), m.RIGHT)
    assert square in scene.mobjects


def removal():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    scene.play(square.animate(run_time=.125, rate_func=m.linear, remover=True).shift(m.RIGHT))
    assert square not in scene.mobjects
    close(square.get_center(), m.RIGHT)


def timed_window_and_suspension():
    scene, square, samples = m.Scene(), m.Square(), []
    scene.add(square)
    square.add_updater(lambda obj: samples.append((scene.get_time(), obj.get_x())), call=False)
    scene.play(square.animate(run_time=.25, rate_func=m.linear, time_span=(.125, .25)).shift(4 * m.RIGHT))
    assert len(samples) >= 4
    assert all(abs(x) < 2e-5 for time, x in samples if 0 < time < .125), samples
    close(square.get_x(), 4)
    calls = []
    square.clear_updaters()
    square.add_updater(lambda obj, dt: calls.append(dt), call=False)
    scene.play(square.animate(run_time=.125, rate_func=m.linear, suspend_mobject_updating=True).shift(m.RIGHT))
    assert calls and all(dt == 0 for dt in calls), calls
    assert not square._is_updating_suspended()


def nested_builder():
    scene, left, right = m.Scene(), m.Square(), m.Square().shift(3 * m.RIGHT)
    scene.add(left, right)
    scene.play(m.AnimationGroup(
        left.animate(run_time=.125, rate_func=m.linear, final_alpha_value=.5).shift(2 * m.UP),
        right.animate(run_time=.125, rate_func=m.linear).shift(m.RIGHT),
    ))
    close(left.get_center(), m.UP)
    close(right.get_center(), 4 * m.RIGHT)


def build_override():
    scene, square, seen = m.Scene(), m.Square(), []
    scene.add(square)
    target = square.copy().shift(m.RIGHT)
    product = m.Transform(square, target, run_time=.125, rate_func=m.linear)
    class Builder(m._AnimationBuilder):
        def build(self):
            seen.append(self)
            return product
    builder = Builder(square)
    scene.play(builder)
    assert seen == [builder]
    close(square.get_center(), m.RIGHT)


def explicit_transform_endpoint():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    scene.play(m.Transform(square, square.copy().shift(2 * m.RIGHT),
                           run_time=.125, rate_func=m.linear, final_alpha_value=.5))
    close(square.get_center(), m.RIGHT)


def mixed_camera_builder():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    frame = scene.frame
    core = frame._core
    scene.play(frame.animate(run_time=.125, rate_func=m.linear).shift(m.RIGHT),
               square.animate(run_time=.125, rate_func=m.linear, final_alpha_value=.5).shift(2 * m.UP))
    close(frame.get_center(), m.RIGHT)
    close(square.get_center(), m.UP)
    assert scene.frame is frame and frame._core is core


def authored_transform_lifecycle():
    # The endpoints deliberately differ, but authored interpolate is a no-op.
    # A native endpoint lerp would visibly move the square and skip the hooks.
    scene, square, events = m.Scene(), m.Square(), []
    scene.add(square)
    before = square.get_points().copy()

    class StationaryTransform(m.Transform):
        def begin(self):
            events.append("begin")
            super().begin()

        def interpolate(self, alpha):
            events.append(("interpolate", float(alpha)))

        def update_mobjects(self, dt):
            events.append(("update", float(dt)))
            super().update_mobjects(dt)

        def finish(self):
            events.append("finish")
            super().finish()

        def clean_up_from_scene(self, owner):
            events.append("cleanup")
            super().clean_up_from_scene(owner)

    animation = StationaryTransform(square, square.copy().shift(2 * m.RIGHT),
                                    run_time=.25, rate_func=m.linear)
    scene.play(animation)
    assert np.array_equal(square.get_points(), before)
    assert events[0] == "begin" and events[-1] == "cleanup", events
    assert "finish" in events
    assert any(isinstance(event, tuple) and event[0] == "update" and event[1] > 0
               for event in events), events
    assert any(isinstance(event, tuple) and event[0] == "interpolate" and 0 < event[1] < 1
               for event in events), events
    assert not square._is_updating_suspended()


def authored_mobject_inside_stock_transform():
    for grouped in (False, True):
        samples = []

        class ArchedSquare(m.Square):
            def interpolate(self, start, end, alpha, path_func=None):
                result = super().interpolate(start, end, alpha, path_func)
                self.shift(4 * alpha * (1 - alpha) * m.UP)
                samples.append((float(alpha), tuple(self.get_center())))
                return result

        scene, square, circle = m.Scene(), ArchedSquare(), m.Circle().shift(3 * m.RIGHT)
        root = m.VGroup(square, circle)
        scene.add(root)
        animation = m.Transform(root, root.copy().shift(2 * m.RIGHT),
                                run_time=.25, rate_func=m.linear)
        entry = m.AnimationGroup(animation) if grouped else animation
        scene.play(entry)
        close(square.get_center(), 2 * m.RIGHT)
        close(circle.get_center(), 5 * m.RIGHT)
        assert samples and max(center[1] for _, center in samples) > .5, samples
        assert any(0 < alpha < 1 for alpha, _ in samples), samples
        assert not square._is_updating_suspended()


def stock_transform_keeps_native_admission():
    native = getattr(m, "_native", m)
    root = m.VGroup(m.Square(), m.Circle().shift(3 * m.RIGHT))
    animation = m.Transform(root, root.copy().shift(m.RIGHT))
    assert not native._requires_python_animation(animation)


for case in (authored_path, final_alpha, removal, timed_window_and_suspension,
             nested_builder, build_override, explicit_transform_endpoint, mixed_camera_builder,
             authored_transform_lifecycle, authored_mobject_inside_stock_transform,
             stock_transform_keeps_native_admission):
    case()
    print("builder playback acceptance:", case.__name__)

# A native Transform is legal only when its objects' lifecycle is native too.
# These are actual-extension tests; the control shapes are built independently.
import tempfile as _hooks_tempfile
import unittest as _hooks_unittest
from pathlib import Path as _HooksPath
from types import MethodType as _HookMethod


class TransformObjectHooks(_hooks_unittest.TestCase):
    def test_object_lifecycle_hooks_select_callbacks_without_admission_calls(self):
        for name in ("copy", "is_aligned_with", "align_data_and_family", "align_family",
                     "align_data", "align_points", "has_updaters", "lock_matching_data",
                     "lock_data", "lock_uniforms", "unlock_data", "set_animating_status",
                     "suspend_updating", "resume_updating", "update"):
            with self.subTest(hook=name):
                calls = []
                def hook(self, *args, _name=name, **kwargs):
                    calls.append(_name)
                    return getattr(super(Authored, self), _name)(*args, **kwargs)
                Authored = type("Authored", (m.Square,), {name: hook})
                square = Authored()
                animation = m.Transform(square, m.Square().shift(m.RIGHT),
                                        suspend_mobject_updating=True)
                calls.clear()
                self.assertTrue(m._requires_python_animation(animation))
                self.assertEqual(calls, [], "route selection must not execute object hooks")
                scene = m.Scene().add(square)
                scene.play(animation, run_time=.1, rate_func=m.linear)
                self.assertIn(name, calls)
                self.assertFalse(square._is_updating_suspended())

    def test_authored_starting_copy_changes_every_sample(self):
        copies, samples = [], []
        class Authored(m.Square):
            def copy(self, *args, **kwargs):
                copies.append(self)
                return super().copy(*args, **kwargs).shift(m.UP)
        square, target, scene = Authored(), m.Square().shift(m.RIGHT), m.Scene()
        probe = m.Mobject()
        probe.add_updater(lambda obj, dt: samples.append((scene.time, square.get_center().copy()))
                          if dt > 0 else None, call=False)
        scene.add(square, probe)
        scene.play(m.Transform(square, target), run_time=.125, rate_func=m.linear)
        self.assertEqual(copies, [square])
        self.assertEqual(len(samples), 4)
        for time, center in samples:
            alpha = min(time/.125, 1.)
            np.testing.assert_allclose(center, [alpha, 1-alpha, 0], atol=2e-6)
        np.testing.assert_allclose(square.get_center(), m.RIGHT, atol=1e-6)

    def test_unaligned_target_copy_is_authored_without_mutating_target(self):
        copies = []
        class Target(m.Triangle):
            def copy(self, *args, **kwargs):
                copies.append(self)
                return super().copy(*args, **kwargs).shift(m.UP)
        square, target, scene = m.Square(), Target().shift(m.RIGHT), m.Scene()
        before = target.get_points().copy()
        expected = m.Triangle().shift(m.RIGHT + m.UP)
        self.assertFalse(square.is_aligned_with(target))
        scene.play(m.Transform(square, target), run_time=.1, rate_func=m.linear)
        self.assertTrue(copies and copies[0] is target)
        np.testing.assert_array_equal(target.get_points(), before)
        np.testing.assert_allclose(square.get_center(), expected.get_center(), atol=1e-6)

    def test_instance_copy_patch_is_seen_between_plays(self):
        square, scene = m.Square(), m.Scene()
        scene.add(square)
        first = m.Transform(square, m.Square().shift(m.RIGHT))
        self.assertFalse(m._requires_python_animation(first))
        scene.play(first, run_time=.1)
        calls = []
        original = square.copy
        def copy(self, *args, **kwargs):
            calls.append(self)
            return original(*args, **kwargs)
        square.copy = _HookMethod(copy, square)
        second = m.Transform(square, m.Square())
        self.assertTrue(m._requires_python_animation(second))
        scene.play(second, run_time=.1)
        self.assertEqual(calls, [square])

    def test_deep_shared_families_detect_object_hooks(self):
        class Authored(m.Square):
            def update(self, *args, **kwargs):
                return super().update(*args, **kwargs)
        child = Authored()
        root = m.Group(m.Group(child), m.Group(child))
        animation = m.Transform(root, root.copy())
        self.assertTrue(m._requires_python_animation(animation))
        m.Scene().play(animation, run_time=.1)
        self.assertIs(root[0][0], root[1][0])

    def test_target_family_descriptor_is_not_evaluated_for_admission(self):
        class Authored(m.Square):
            @property
            def copy(self):
                raise AssertionError("copy descriptor ran while choosing a route")
        target = Authored()
        animation = m.Transform(m.Square(), m.Group(target))
        self.assertTrue(m._requires_python_animation(animation))

    def test_late_base_copy_patch_is_seen_through_shipped_override(self):
        original = m.Mobject.copy
        calls = []
        def copy(self, *args, **kwargs):
            calls.append(self)
            return original(self, *args, **kwargs)
        source = m.Square()
        animation = m.Transform(source, m.Square().shift(m.UP))
        try:
            m.Mobject.copy = copy
            self.assertTrue(m._requires_python_animation(animation))
            m.Scene().play(animation, run_time=.1)
            self.assertIn(source, calls)
        finally:
            m.Mobject.copy = original

    def test_stock_objects_and_unmodified_subclasses_stay_native(self):
        class Plain(m.Square):
            pass
        for source in (m.Square(), Plain(), m.Circle(), m.Group(m.Square(), m.Circle()),
                       m.ValueTracker(0)):
            with self.subTest(kind=type(source).__name__):
                animation = m.Transform(source, source.copy())
                self.assertFalse(m._requires_python_animation(animation))

    def test_reinitialization_does_not_recapture_authored_methods(self):
        from fmn_python.initialization import initialize
        native = getattr(m, "_native", m)
        self.assertNotIn("_fmn_finalize_transform_dispatch", vars(native))
        square = m.Square()
        original = square.copy
        square.copy = lambda *args, **kwargs: original(*args, **kwargs)
        animation = m.Transform(square, m.Square())
        self.assertTrue(m._requires_python_animation(animation))
        self.assertIs(initialize(native), native)
        self.assertTrue(m._requires_python_animation(animation))
        self.assertNotIn("_fmn_finalize_transform_dispatch", vars(native))

    def test_copy_failure_keeps_original_error_and_releases_transients(self):
        error = RuntimeError("authored snapshot failure")
        class Authored(m.Square):
            fail = True
            def copy(self, *args, **kwargs):
                if self.fail:
                    raise error
                return super().copy(*args, **kwargs)
        square, scene = Authored(), m.Scene()
        scene.add(square)
        before = square.get_points().copy()
        with self.assertRaises(RuntimeError) as caught:
            scene.play(m.Transform(square, m.Square(), suspend_mobject_updating=True), run_time=.1)
        self.assertIs(caught.exception, error)
        self.assertFalse(square._is_updating_suspended())
        self.assertFalse(square.is_changing())
        np.testing.assert_array_equal(square.get_points(), before)
        square.fail = False
        scene.play(m.Transform(square, m.Square().shift(m.UP)), run_time=.1)
        np.testing.assert_allclose(square.get_center(), m.UP, atol=1e-6)

    def test_cleanup_failure_keeps_original_error_and_recovers(self):
        error = RuntimeError("authored unlock failure")
        class Authored(m.Square):
            fail = True
            def unlock_data(self):
                super().unlock_data()
                if self.fail:
                    raise error
                return self
        square, scene = Authored(), m.Scene()
        scene.add(square)
        with self.assertRaises(RuntimeError) as caught:
            scene.play(m.Transform(square, m.Square().shift(m.RIGHT)), run_time=.1)
        self.assertIs(caught.exception, error)
        self.assertFalse(square._is_updating_suspended())
        self.assertFalse(square.is_changing())
        square.fail = False
        scene.play(m.Transform(square, m.Square()), run_time=.1)
        np.testing.assert_allclose(square.get_center(), m.ORIGIN, atol=1e-6)

    def test_nested_succession_uses_live_object_snapshots(self):
        calls = []
        class Authored(m.Square):
            def copy(self, *args, **kwargs):
                calls.append(self.get_center().copy())
                return super().copy(*args, **kwargs)
        square, scene = Authored(), m.Scene()
        scene.add(square)
        animation = m.Succession(m.Transform(square, m.Square().shift(m.RIGHT), run_time=.1),
                                 m.Transform(square, m.Square().shift(2*m.RIGHT), run_time=.1))
        scene.play(m.AnimationGroup(animation), rate_func=m.linear)
        self.assertTrue(any(np.allclose(value, m.ORIGIN) for value in calls))
        self.assertTrue(any(np.allclose(value, m.RIGHT) for value in calls))
        np.testing.assert_allclose(square.get_center(), 2*m.RIGHT, atol=1e-6)

    def test_rendered_snapshots_match_independent_native_control(self):
        class Authored(m.Square):
            def copy(self, *args, **kwargs):
                return super().copy(*args, **kwargs).shift(m.UP)
        def render(path, kind, threads):
            scene = m.Scene()
            with scene.render_session(path, format="y4m", resolution=(64, 40), fps=24,
                                      threads=threads) as output:
                source = Authored() if kind == "authored" else m.Square()
                if kind == "control":
                    source.shift(m.UP)
                scene.add(source)
                scene.play(m.Transform(source, m.Square().shift(m.RIGHT)),
                           run_time=.125, rate_func=m.linear)
            self.assertEqual(output.result.frame_count, 3)
            return path.read_bytes()
        with _hooks_tempfile.TemporaryDirectory() as tmp:
            root = _HooksPath(tmp)
            expected = render(root/"control.y4m", "control", 1)
            for threads in (1, 4, 16):
                self.assertEqual(render(root/f"authored-{threads}.y4m", "authored", threads), expected)
            self.assertNotEqual(render(root/"unchanged.y4m", "unchanged", 1), expected)
            payloads = expected.split(b"FRAME\n")[1:]
            self.assertEqual(len(payloads), 3)
            self.assertNotEqual(payloads[0], payloads[1])


_hooks_result = _hooks_unittest.TextTestRunner(verbosity=2).run(
    _hooks_unittest.defaultTestLoader.loadTestsFromTestCase(TransformObjectHooks))
if not _hooks_result.wasSuccessful():
    raise AssertionError("native Transform object-hook acceptance failed")
