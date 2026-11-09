"""Composite delegation tests; the geometry/scene doubles are not native proof."""
from __future__ import annotations

import copy
import importlib.util
import math
from pathlib import Path
from types import SimpleNamespace
import unittest

import numpy as np

SOURCE = Path(__file__).resolve().parents[1] / "python/fmn_python/composite_effects.py"
spec = importlib.util.spec_from_file_location("composite_effects_under_test", SOURCE)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def table():
    class Mobject:
        def __init__(self, points=()):
            self.points = np.asarray(points, dtype=float).reshape(-1, 3)
            self.submobjects, self.updaters = [], []
            self.calls = []

        def __iter__(self):
            return iter(self.submobjects)

        def copy(self):
            return copy.deepcopy(self)

        def get_center(self):
            parts = [obj.points for obj in self.get_family() if len(obj.points)]
            if not parts:
                return np.zeros(3)
            points = np.concatenate(parts)
            return (points.min(axis=0) + points.max(axis=0)) / 2

        def get_family(self):
            return [self, *(part for child in self for part in child.get_family())]

        def shift(self, vector):
            for member in self.get_family():
                member.points += vector
            return self

        def rotate(self, angle, about_point):
            c, s = math.cos(angle), math.sin(angle)
            matrix = np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]])
            for member in self.get_family():
                member.points = (member.points - about_point) @ matrix.T + about_point
            return self

        def move_to(self, point):
            target = point.get_center() if isinstance(point, Mobject) else point
            return self.shift(np.asarray(target) - self.get_center())

        def set_stroke(self, **kwargs):
            self.stroke = dict(kwargs)
            return self

        def add_updater(self, updater):
            self.updaters.append(updater)
            updater(self)
            return self

        def update(self):
            for updater in self.updaters:
                updater(self)

    class VGroup(Mobject):
        def __init__(self, *members):
            super().__init__()
            self.submobjects = list(members)

        def add(self, member):
            self.submobjects.append(member)
            return self

    class Line(Mobject):
        def __init__(self, start, end):
            super().__init__([start, end])

    class Animation:
        def __init__(self, mobject, **kwargs):
            self.mobject = mobject
            self.kwargs = dict(kwargs)
            self.events = []
            self.run_time = kwargs.get("run_time", 1.0)

        def begin(self):
            self.events.append("begin")

        def update_mobjects(self, dt):
            self.events.append(("helpers", dt))

        def interpolate(self, alpha):
            self.events.append(("alpha", alpha))

        def finish(self):
            self.events.append("finish")

        def clean_up_from_scene(self, scene):
            self.events.append(("cleanup", scene))

        def abort(self):
            self.events.append("abort")

    class AnimationGroup(Animation):
        def __init__(self, *animations, run_time=-1, lag_ratio=0, group=None, **kwargs):
            self.animations = list(animations)
            self.mobject = self.group = group
            self.run_time, self.lag_ratio = run_time, lag_ratio
            self.kwargs = kwargs
            self._composition_authored_root = group is not None

        def _ensure_runtime_defaults(self):
            self.normalized = True

        def get_all_mobjects(self):
            return self.mobject

        def begin(self):
            for child in self.animations:
                child.begin()

        def update_mobjects(self, dt):
            for child in self.animations:
                child.update_mobjects(dt)

        def interpolate(self, alpha):
            for child in self.animations:
                child.interpolate(alpha)

        def finish(self):
            for child in self.animations:
                child.finish()

        def clean_up_from_scene(self, scene):
            for child in self.animations:
                child.clean_up_from_scene(scene)

        def abort(self):
            for child in self.animations:
                child.abort()

    class LaggedStart(AnimationGroup):
        pass

    class Flash(AnimationGroup):
        def begin(self):
            raise AssertionError("obsolete leaf-style Flash.begin")

        def interpolate_mobject(self, alpha):
            raise AssertionError("obsolete synthetic Flash reveal")

    for name in ("interpolate", "finish", "update_mobjects", "clean_up_from_scene", "abort",
                 "_ensure_runtime_defaults", "get_all_mobjects"):
        setattr(Flash, name, Flash.begin)

    class LaggedStartMap(LaggedStart):
        pass

    class ShowCreationThenDestruction(Animation):
        pass

    class Broadcast(LaggedStart):
        _native_kind = "broadcast"

        def __init__(self, point, *, run_time=3, lag_ratio=.2, **kwargs):
            self.point = point
            self.circles = VGroup(Mobject([[1, 0, 0]]), Mobject([[2, 0, 0]]))
            self.circles.marker = "original rings"
            super().__init__(*(Animation(ring) for ring in self.circles),
                             run_time=run_time, lag_ratio=lag_ratio, **kwargs)

    class ClockPassesTime(AnimationGroup):
        def __init__(self, clock, run_time=5, **kwargs):
            super().__init__(Animation(clock.hour), Animation(clock.minute),
                             run_time=run_time, **kwargs)
            # This is the old split identity: native specs use mobject while
            # only the legacy Python group attribute remembers the clock.
            self.group = self.clock = clock

    class FlashyFadeIn(AnimationGroup):
        _native_kind = "flashy_fade_in"

        def __init__(self, vmobject, *, fade_lag=0, **kwargs):
            self.outline = vmobject.copy()
            self.fade_lag = fade_lag
            super().__init__(Animation(vmobject), Animation(self.outline), **kwargs)

    def ensure_root(animation):
        if animation.mobject is None:
            animation.mobject = VGroup(*(child.mobject for child in animation.animations))
        animation.group = animation.mobject
        return animation.mobject

    return SimpleNamespace(
        Mobject=Mobject, VGroup=VGroup, Line=Line, Animation=Animation,
        AnimationGroup=AnimationGroup, LaggedStart=LaggedStart,
        Flash=Flash, LaggedStartMap=LaggedStartMap,
        Broadcast=Broadcast, ClockPassesTime=ClockPassesTime, FlashyFadeIn=FlashyFadeIn,
        _fmn_ensure_composition_root=ensure_root,
        ShowCreationThenDestruction=ShowCreationThenDestruction,
        _np=np, _YELLOW="#FFFF00", _ORIGIN=np.zeros(3), _RIGHT=np.array([1., 0., 0.]),
    )


class CompositeEffectsProtocol(unittest.TestCase):
    def setUp(self):
        self.native = table()
        self.identities = (self.native.Flash, self.native.LaggedStartMap)
        module.install_composite_effects(self.native)

    def test_flash_owns_original_line_group(self):
        flash = self.native.Flash([2, 3, 0], num_lines=4)
        self.assertIs(flash.mobject, flash.lines)
        self.assertIs(flash.group, flash.lines)
        self.assertIs(flash.get_all_mobjects(), flash.lines)
        self.assertTrue(flash._composition_authored_root)
        self.assertEqual(flash._native_kind, "animation_group")
        self.assertEqual(len(flash.animations), 4)
        self.assertTrue(all(a.mobject is b for a, b in zip(flash.animations, flash.lines)))

    def test_radial_geometry_and_style(self):
        flash = self.native.Flash([0, 0, 0], num_lines=4, flash_radius=3,
                                  line_length=2, color="blue", line_stroke_width=7)
        expected = [[[1, 0, 0], [3, 0, 0]], [[0, 1, 0], [0, 3, 0]],
                    [[-1, 0, 0], [-3, 0, 0]], [[0, -1, 0], [0, -3, 0]]]
        for line, points in zip(flash.lines, expected):
            np.testing.assert_allclose(line.points, points, atol=1e-14)
        self.assertEqual(flash.lines.stroke, {"color": "blue", "width": 7})

    def test_numeric_point_runs_every_child_lifecycle(self):
        flash = self.native.Flash([1, 2, 0], num_lines=2)
        scene = object()
        flash._ensure_runtime_defaults()
        flash.begin()
        flash.update_mobjects(.125)
        flash.interpolate(.25)
        flash.finish()
        flash.clean_up_from_scene(scene)
        self.assertTrue(flash.normalized)
        for animation in flash.animations:
            self.assertEqual(animation.events,
                             ["begin", ("helpers", .125), ("alpha", .25), "finish", ("cleanup", scene)])
        self.assertNotIn("interpolate_mobject", vars(self.native.Flash))

    def test_moving_point_retains_author_created_animations(self):
        native, calls = self.native, []
        target = native.Mobject([[2, 1, 0]])
        class AuthoredFlash(native.Flash):
            def create_line_anims(self):
                calls.append(self.lines)
                return [native.Animation(line) for line in self.lines]
        flash = AuthoredFlash(target, num_lines=4)
        flash.begin()
        flash.update_mobjects(.2)
        flash.interpolate(.4)
        self.assertEqual(calls, [flash.lines])
        self.assertTrue(all(type(child) is native.Animation for child in flash.animations))
        for child in flash.animations:
            self.assertEqual(child.events, ["begin", ("helpers", .2), ("alpha", .4)])
        target.shift([3, -2, 0])
        flash.lines.update()
        np.testing.assert_allclose(flash.lines.get_center(), target.get_center())

    def test_point_rebinding_remains_live(self):
        flash = self.native.Flash([0, 0, 0], num_lines=4)
        flash.point = np.array([4, -1, 2])
        flash.lines.update()
        np.testing.assert_allclose(flash.lines.get_center(), flash.point)
        self.assertEqual(len(flash.lines.updaters), 1)

    def test_caller_owned_point_array_and_list_remain_live(self):
        for point in (np.array([1., 2., 0.]), [1., 2., 0.]):
            flash = self.native.Flash(point, num_lines=4)
            self.assertIs(flash.point, point)
            point[0] = 9.
            flash.lines.update()
            np.testing.assert_allclose(flash.lines.get_center(), [9., 2., 0.])

    def test_authored_line_factory_and_super_dispatch(self):
        native, calls = self.native, []
        class Custom(native.Flash):
            def create_lines(self):
                calls.append("factory")
                return native.VGroup(native.Line([0, 0, 0], [0, 2, 0]))
            def begin(self):
                calls.append("begin")
                super().begin()
        flash = Custom([0, 0, 0])
        flash.begin()
        self.assertEqual(calls, ["factory", "begin"])
        self.assertEqual(flash.animations[0].events, ["begin"])

    def test_later_group_base_override_remains_live_for_flash(self):
        native, calls = self.native, []
        original = native.AnimationGroup.begin
        def changed(group):
            calls.append(group)
            original(group)
        native.AnimationGroup.begin = changed
        flash = native.Flash([0, 0, 0], num_lines=2)
        flash.begin()
        self.assertEqual(calls, [flash])
        self.assertTrue(all(child.events == ["begin"] for child in flash.animations))

    def test_child_failure_is_not_swallowed_by_synthetic_reveal(self):
        native = self.native
        error = RuntimeError("authored child interpolation failed")
        class Broken(native.Animation):
            def interpolate(self, alpha):
                raise error
        class Custom(native.Flash):
            def create_line_anims(self):
                return [Broken(next(iter(self.lines)))]
        flash = Custom(native.Mobject([[0, 0, 0]]))
        flash.begin()
        with self.assertRaises(RuntimeError) as caught:
            flash.interpolate(.5)
        self.assertIs(caught.exception, error)
        flash.abort()
        self.assertEqual(flash.animations[0].events[-1], "abort")

    def test_group_options_are_not_forwarded_to_children(self):
        rate = lambda alpha: alpha**2
        flash = self.native.Flash([0, 0, 0], num_lines=2, run_time=3, lag_ratio=.4,
                                  rate_func=rate, remover=True, time_span=(1, 2))
        self.assertEqual((flash.run_time, flash.lag_ratio), (3, .4))
        self.assertIs(flash.kwargs["rate_func"], rate)
        self.assertTrue(flash.kwargs["remover"])
        self.assertTrue(all(child.kwargs == {} for child in flash.animations))

    def test_input_refusals_before_factory(self):
        native, calls = self.native, []
        class Guard(native.Flash):
            def create_lines(self):
                calls.append(True)
                return super().create_lines()
        cases = [({"num_lines": 0}, ValueError), ({"num_lines": 1.5}, TypeError),
                 ({"line_length": -1}, ValueError), ({"flash_radius": math.inf}, ValueError),
                 ({"line_stroke_width": math.nan}, ValueError)]
        for kwargs, error in cases:
            with self.assertRaises(error):
                Guard([0, 0, 0], **kwargs)
        for point in ([0, 0], [0, math.inf, 0]):
            with self.assertRaises(ValueError):
                Guard(point)
        self.assertFalse(calls)

    def test_invalid_line_factory_names_error(self):
        class Bad(self.native.Flash):
            def create_lines(self):
                return object()
        with self.assertRaisesRegex(TypeError, "create_lines"):
            Bad([0, 0, 0])

    def test_map_preserves_caller_root_and_child_identity(self):
        native = self.native
        children = [native.Mobject([[i, 0, 0]]) for i in range(3)]
        root = native.VGroup(*children)
        root.updaters.append(lambda group: None)
        created = []
        def factory(child, **kwargs):
            animation = native.Animation(child, **kwargs)
            created.append(animation)
            return animation
        animation = native.LaggedStartMap(factory, root, run_time=7, lag_ratio=.3, marker="leaf")
        self.assertIs(animation.mobject, root)
        self.assertIs(animation.group, root)
        self.assertTrue(animation._composition_authored_root)
        self.assertEqual((animation.run_time, animation.lag_ratio), (7, .3))
        self.assertEqual(animation.animations, created)
        self.assertTrue(all(a.mobject is b for a, b in zip(created, children)))
        self.assertTrue(all(child.kwargs == {"marker": "leaf"} for child in created))
        self.assertEqual(len(root.updaters), 1)
        self.assertEqual(animation.kwargs, {})

    def test_map_factory_membership_mutations_follow_the_group_iterator(self):
        native, visited = self.native, []
        children = [native.Mobject([[i, 0, 0]]) for i in range(3)]
        root = native.VGroup(*children)
        def factory(child):
            visited.append(child)
            root.submobjects.clear()
            return native.Animation(child)
        animation = native.LaggedStartMap(factory, root)
        self.assertEqual(visited, children[:1])
        self.assertEqual(len(animation.animations), 1)
        self.assertIs(animation.group, root)

    def test_map_rejects_invalid_factory_or_root_without_calls(self):
        native = self.native
        with self.assertRaisesRegex(TypeError, "constructor"):
            native.LaggedStartMap(None, native.VGroup())
        with self.assertRaisesRegex(TypeError, "Mobject"):
            native.LaggedStartMap(lambda child: self.fail("called"), [])

    def test_map_propagates_authored_factory_failure(self):
        native = self.native
        error = RuntimeError("factory failed")
        def factory(child):
            raise error
        with self.assertRaises(RuntimeError) as caught:
            native.LaggedStartMap(factory, native.VGroup(native.Mobject()))
        self.assertIs(caught.exception, error)

    def test_empty_map_keeps_empty_original_group(self):
        native = self.native
        root = native.VGroup()
        animation = native.LaggedStartMap(lambda child: self.fail("called"), root)
        self.assertIs(animation.mobject, root)
        self.assertEqual(animation.animations, [])

    def test_broadcast_preserves_original_rings_and_member_factories(self):
        effect = self.native.Broadcast([1, 0, 0], run_time=4, lag_ratio=.3)
        self.assertIs(effect.mobject, effect.circles)
        self.assertIs(effect.group, effect.circles)
        self.assertEqual(effect.group.marker, "original rings")
        self.assertTrue(effect._composition_authored_root)
        self.assertEqual(effect._native_kind, "lagged_start")
        self.assertEqual((effect.run_time, effect.lag_ratio), (4, .3))
        self.assertTrue(all(child.mobject is ring for child, ring in zip(effect.animations, effect.circles)))

    def test_clock_group_includes_face_not_just_animated_hands(self):
        native = self.native
        face = native.Mobject([[0, 0, 0]])
        hour, minute = native.Mobject([[1, 0, 0]]), native.Mobject([[2, 0, 0]])
        clock = native.VGroup(face, hour, minute)
        clock.hour, clock.minute = hour, minute
        effect = native.ClockPassesTime(clock, run_time=2)
        self.assertIs(effect.mobject, clock)
        self.assertIs(effect.group, clock)
        self.assertIn(face, effect.mobject.get_family())
        self.assertEqual([child.mobject for child in effect.animations], [hour, minute])
        self.assertTrue(effect._composition_authored_root)

    def test_flashy_fade_root_contains_live_subject_and_original_outline(self):
        native = self.native
        subject = native.Mobject([[3, 4, 0]])
        effect = native.FlashyFadeIn(subject, fade_lag=.25, run_time=3)
        self.assertEqual(list(effect.group), [subject, effect.outline])
        self.assertIs(effect.animations[0].mobject, subject)
        self.assertIs(effect.animations[1].mobject, effect.outline)
        self.assertEqual((effect.run_time, effect.fade_lag), (3, .25))
        self.assertEqual(effect._native_kind, "animation_group")
        self.assertTrue(effect._composition_authored_root)

    def test_explicit_effect_group_is_never_overwritten(self):
        native = self.native
        custom = native.VGroup()
        subject = native.Mobject([[0, 0, 0]])
        clock = native.VGroup(subject)
        clock.hour = clock.minute = subject
        effects = [native.Broadcast([0, 0, 0], group=custom),
                   native.FlashyFadeIn(subject, group=custom),
                   native.ClockPassesTime(clock, group=custom)]
        for effect in effects:
            self.assertIs(effect.group, custom)
            self.assertIs(effect.mobject, custom)

    def test_effects_delegate_nested_child_phases(self):
        native = self.native
        broadcast = native.Broadcast([0, 0, 0])
        fade = native.FlashyFadeIn(native.Mobject([[0, 0, 0]]))
        outer = native.AnimationGroup(broadcast, fade)
        outer.begin()
        outer.update_mobjects(.125)
        outer.interpolate(.75)
        outer.finish()
        for effect in (broadcast, fade):
            for child in effect.animations:
                self.assertEqual(child.events, ["begin", ("helpers", .125), ("alpha", .75), "finish"])

    def test_wrapped_constructor_signature_and_subclass_calls_are_kept(self):
        import inspect
        native, calls = self.native, []
        self.assertIn("fade_lag", inspect.signature(native.FlashyFadeIn).parameters)
        class Authored(native.Broadcast):
            def __init__(self, *args, **kwargs):
                calls.append("before")
                super().__init__(*args, **kwargs)
                calls.append(self.group)
        effect = Authored([0, 0, 0])
        self.assertEqual(calls, ["before", effect.circles])

    def test_installer_does_not_duplicate_effect_roots(self):
        native = self.native
        effect = native.Broadcast([0, 0, 0])
        root, constructor = effect.group, native.Broadcast.__init__
        module.install_composite_effects(native)
        self.assertIs(native.Broadcast.__init__, constructor)
        self.assertIs(effect.group, root)

    def test_installer_is_idempotent_and_keeps_public_classes(self):
        method = self.native.Flash.__init__
        module.install_composite_effects(self.native)
        self.assertIs(self.native.Flash.__init__, method)
        self.assertEqual(self.identities, (self.native.Flash, self.native.LaggedStartMap))
        self.assertEqual(method.__name__, "__init__")
        self.assertEqual(method.__qualname__, self.native.Flash.__qualname__ + ".__init__")


if __name__ == "__main__":
    unittest.main()
