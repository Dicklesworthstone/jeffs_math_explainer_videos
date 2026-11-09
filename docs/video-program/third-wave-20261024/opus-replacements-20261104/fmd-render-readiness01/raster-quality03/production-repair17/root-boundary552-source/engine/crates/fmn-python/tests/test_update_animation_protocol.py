"""MaintainPositionRelativeTo construction/play timing over fixture storage."""
import copy
import importlib.util
import inspect
from pathlib import Path
import sys
import types
import unittest
import numpy as np

ROOT = Path(__file__).resolve().parents[1]
python_dir = str(ROOT / "python")
if python_dir not in sys.path:
    sys.path.insert(0, python_dir)
import fmn_python.movement as movement


def _method(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    setattr(cls, name, function)


def _cancel_preserving(animation, error):
    try:
        animation.abort()
    except BaseException as cleanup:
        error.add_note(str(cleanup))


def _install_lifecycle(g, cls, prepare):
    def abort(self):
        if getattr(self, "_movement_active", False):
            self._movement_active = False
            self.mobject.set_animating_status(False)
    def defaults(self):
        super(cls, self)._ensure_runtime_defaults()
    def begin(self):
        self._movement_active = True
        self._movement_finished = False
        self._ensure_runtime_defaults()
        prepare(self)
        try:
            super(cls, self).begin()
        except BaseException as error:
            _cancel_preserving(self, error)
            raise
    def finish(self):
        try:
            super(cls, self).finish()
            self._movement_active = False
            self._movement_finished = True
        except BaseException as error:
            _cancel_preserving(self, error)
            raise
    def interpolate(self, alpha):
        try:
            return super(cls, self).interpolate(alpha)
        except BaseException as error:
            _cancel_preserving(self, error)
            raise
    def update_mobjects(self, dt):
        return super(cls, self).update_mobjects(dt)
    def cleanup(self, scene):
        return super(cls, self).clean_up_from_scene(scene)
    for name, function in {
        "abort": abort, "_ensure_runtime_defaults": defaults, "begin": begin,
        "finish": finish, "interpolate": interpolate,
        "update_mobjects": update_mobjects, "clean_up_from_scene": cleanup,
    }.items():
        _method(cls, name, function)


import fmn_python.update_animations as adapter


def environment():
    native = types.ModuleType("manimlib.update_fixture")
    g = vars(native)

    class Mobject:
        def __init__(self, x=0, *children):
            self.center = np.array([float(x), 0.0, 0.0])
            self.submobjects = list(children)
            self.animating = False
            self._scene = None
        def copy(self):
            return copy.deepcopy(self)
        def get_center(self):
            return self.center.copy()
        def shift(self, vector):
            self.center += np.asarray(vector)
            return self
        def get_family(self):
            return [self] + [member for child in self.submobjects for member in child.get_family()]
        def set_animating_status(self, value):
            self.animating = bool(value)
        def _is_updating_suspended(self):
            return False
        def suspend_updating(self, recurse=True):
            del recurse
        def resume_updating(self, recurse=True, call_updater=True):
            del recurse, call_updater
        def update(self, dt=0):
            del dt

    class Animation:
        _native_kind = None
        def __init__(self, mobject, final_alpha_value=1, remover=False, **kwargs):
            del kwargs
            self.mobject = mobject
            self.final_alpha_value = final_alpha_value
            self.remover = remover
            self.run_time = 1
            self.rate_func = lambda t: t
            self.time_span = None
            self.lag_ratio = 0
            self.suspend_mobject_updating = False
        def _ensure_runtime_defaults(self):
            pass
        def create_starting_mobject(self):
            return self.mobject.copy()
        def begin(self):
            self.mobject.set_animating_status(True)
            self.starting_mobject = self.create_starting_mobject()
            self.interpolate(0)
        def finish(self):
            self.interpolate(self.final_alpha_value)
            self.mobject.set_animating_status(False)
        def interpolate(self, alpha):
            return self.interpolate_mobject(alpha)
        def interpolate_mobject(self, alpha):
            del alpha
        def update_mobjects(self, dt):
            del dt
        def get_all_mobjects(self):
            return self.mobject, self.starting_mobject
        def get_all_families_zipped(self):
            return zip(*(mobject.get_family() for mobject in self.get_all_mobjects()))
        def get_all_mobjects_to_update(self):
            return [self.starting_mobject]
        def clean_up_from_scene(self, scene):
            if self.remover:
                scene.remove(self.mobject)

    class NativeAnimation(Animation):
        pass

    class MaintainPositionRelativeTo(NativeAnimation):
        _native_kind = "maintain_position_relative_to"
        _target_attr = "tracked_mobject"
        def __init__(self, mobject, tracked_mobject=None, **kwargs):
            if not isinstance(mobject, Mobject) or not isinstance(tracked_mobject, Mobject):
                raise TypeError
            super().__init__(mobject, **kwargs)
            self.tracked_mobject = tracked_mobject

    class AnimationGroup(Animation):
        def __init__(self, *animations):
            self.animations = list(animations)
            super().__init__(Mobject())

    class Builder:
        pass

    class Scene:
        def __init__(self):
            self.calls = []
            self.removed = []
        def play(self, *animations, **kwargs):
            result = [g["_requires_python_animation"](animation) for animation in animations]
            self.calls.append((result, kwargs))
            return self.calls[-1]
        def remove(self, mobject):
            self.removed.append(mobject)

    class ForeignStageError(ValueError):
        pass

    g.update(
        Animation=Animation, Mobject=Mobject,
        MaintainPositionRelativeTo=MaintainPositionRelativeTo,
        AnimationGroup=AnimationGroup, _AnimationBuilder=Builder, Scene=Scene,
        _requires_python_animation=lambda animation: not getattr(animation, "_native_kind", None),
        prepare_animation=lambda animation: animation,
        _ForeignStageError=ForeignStageError,
    )
    return native


class UpdateAnimationProtocolTests(unittest.TestCase):
    def setUp(self):
        self.native = environment()
        adapter.install_update_animations(self.native)
        self.follower = self.native.Mobject(1)
        self.tracked = self.native.Mobject(0)

    def test_offset_is_captured_at_animation_construction(self):
        animation = self.native.MaintainPositionRelativeTo(self.follower, self.tracked)
        self.tracked.shift([10, 0, 0])
        animation.begin()
        self.assertEqual(self.follower.get_center()[0], 11)

    def test_live_tracking_reads_current_tracked_position(self):
        animation = self.native.MaintainPositionRelativeTo(self.follower, self.tracked)
        animation.begin()
        self.tracked.shift([2, 0, 0])
        animation.interpolate(.5)
        self.assertEqual(self.follower.get_center()[0], 3)

    def test_stock_maintain_uses_callback_route(self):
        animation = self.native.MaintainPositionRelativeTo(self.follower, self.tracked)
        self.assertTrue(vars(self.native)["_requires_python_animation"](animation))

    def test_remover_cleanup_uses_shared_animation_protocol(self):
        scene = self.native.Scene()
        animation = self.native.MaintainPositionRelativeTo(
            self.follower, self.tracked, remover=True
        )
        animation.begin()
        animation.finish()
        animation.clean_up_from_scene(scene)
        self.assertEqual(scene.removed, [self.follower])

    def test_cross_scene_tracking_refuses_before_interpolation(self):
        animation = self.native.MaintainPositionRelativeTo(self.follower, self.tracked)
        self.follower._scene = object()
        self.tracked._scene = object()
        with self.assertRaises(self.native._ForeignStageError):
            animation.begin()
        self.assertFalse(self.follower.animating)

    def test_tracked_get_center_override_remains_live(self):
        parent = self.native.Mobject
        class Tracked(parent):
            calls = 0
            def get_center(self):
                type(self).calls += 1
                return super().get_center()
        tracked = Tracked(0)
        animation = self.native.MaintainPositionRelativeTo(self.follower, tracked)
        before = Tracked.calls
        animation.begin()
        self.assertGreater(Tracked.calls, before)

    def test_installation_is_idempotent(self):
        play = self.native.Scene.play
        requires = vars(self.native)["_requires_python_animation"]
        adapter.install_update_animations(self.native)
        self.assertIs(self.native.Scene.play, play)
        self.assertIs(vars(self.native)["_requires_python_animation"], requires)


if __name__ == "__main__":
    unittest.main()
