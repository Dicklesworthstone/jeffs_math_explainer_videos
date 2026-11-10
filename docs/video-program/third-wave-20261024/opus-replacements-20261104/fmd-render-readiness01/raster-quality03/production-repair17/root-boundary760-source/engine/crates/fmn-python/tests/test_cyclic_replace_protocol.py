"""CyclicReplace control-flow tests; native rendering is covered separately.

The storage doubles below check delegation, identity and dispatch only. They
are not a substitute for the installed-wheel acceptance in cyclic_replace.py.
"""
from __future__ import annotations

import copy
import importlib.util
from pathlib import Path
from types import MethodType, SimpleNamespace
import unittest


SOURCE = Path(__file__).resolve().parents[1] / "python/fmn_python/playback.py"
spec = importlib.util.spec_from_file_location("cyclic_playback_under_test", SOURCE)
playback = importlib.util.module_from_spec(spec)
spec.loader.exec_module(playback)


def table():
    class Mobject:
        def __init__(self, center=0, name=""):
            self.center, self.name = center, name

        def copy(self):
            return copy.deepcopy(self)

        def move_to(self, other):
            self.center = other.center
            return self

    class Group(Mobject):
        constructions = 0

        def __init__(self, *objects):
            super().__init__()
            Group.constructions += 1
            self.submobjects = list(objects)

        def __len__(self):
            return len(self.submobjects)

        def __iter__(self):
            return iter(self.submobjects)

        def __getitem__(self, index):
            return self.submobjects[index]

    class Animation:
        pass

    class Transform(Animation):
        _native_kind = "transform"
        _target_attr = "target_mobject"
        replace_mobject_with_target_in_scene = False

        def __init__(self, mobject, target_mobject=None, path_arc=0,
                     path_arc_axis=(0, 0, 1), path_func=None, **kwargs):
            self.mobject, self.target_mobject = mobject, target_mobject
            self.path_arc, self.path_arc_axis = path_arc, path_arc_axis
            self.path_func = path_func
            self.final_alpha_value = 1
            self.remover = False
            self.rate_func = None
            self.__dict__.update(kwargs)

        def create_target(self):
            return self.target_mobject

        def _native_target(self):
            self.target_mobject = self.create_target()
            return self.target_mobject

        def _native_params(self):
            return {"path_arc": self.path_arc, "path_arc_axis": self.path_arc_axis}

        def begin(self):
            self.target_mobject = self.create_target()

    class CyclicReplace(Transform):
        _native_kind = "cyclic_replace"
        _target_attr = None

        def _native_params(self):
            raise AssertionError("the targetless lowering must not be used")

    class Swap(CyclicReplace):
        _native_kind = "swap"

    def linear(value):
        return value

    previous_calls = []

    def requires(animation):
        previous_calls.append(animation)
        # The old shared baseline does not know this new create_target yet.
        return True

    return SimpleNamespace(
        Mobject=Mobject, Group=Group, Animation=Animation, Transform=Transform,
        CyclicReplace=CyclicReplace, Swap=Swap, _RATE_FUNC_NAMES={linear: "linear"},
        linear=linear, _requires_python_animation=requires,
        previous_calls=previous_calls,
    )


class CyclicReplaceProtocol(unittest.TestCase):
    def setUp(self):
        self.native = table()
        self.g = vars(self.native)
        self.original_classes = (self.native.CyclicReplace, self.native.Swap)
        playback._install_cyclic_replace(self.g)

    def operands(self):
        return [self.native.Mobject(x, label) for x, label in
                ((-4, "left"), (1, "middle"), (8, "right"))]

    def test_real_group_retains_original_operands_and_defers_target(self):
        objects = self.operands()
        animation = self.native.CyclicReplace(*objects)
        self.assertIsInstance(animation.mobject, self.native.Group)
        self.assertEqual(list(animation.mobject), objects)
        self.assertTrue(all(a is b for a, b in zip(animation.mobject, objects)))
        self.assertIsNone(animation.target_mobject)
        self.assertEqual(animation.path_arc, 1.5707963267948966)

    def test_three_operand_direction_and_style_are_preserved(self):
        objects = self.operands()
        animation = self.native.CyclicReplace(*objects)
        target = animation.create_target()
        self.assertEqual([obj.center for obj in target], [1, 8, -4])
        self.assertEqual([obj.name for obj in target], ["left", "middle", "right"])
        self.assertEqual([obj.center for obj in objects], [-4, 1, 8])
        self.assertTrue(all(a is not b for a, b in zip(target, objects)))

    def test_begin_samples_edits_after_construction(self):
        objects = self.operands()
        animation = self.native.Swap(*objects)
        objects[1].center = 17
        animation.begin()
        self.assertEqual([obj.center for obj in animation.target_mobject], [17, 8, -4])
        objects[1].center = 29
        animation.begin()
        self.assertEqual([obj.center for obj in animation.target_mobject], [29, 8, -4])

    def test_swap_two_operands(self):
        left, right = self.operands()[:2]
        animation = self.native.Swap(left, right, path_arc=0)
        target = animation._native_target()
        self.assertEqual([obj.center for obj in target], [1, -4])
        self.assertIs(animation.target_mobject, target)
        self.assertEqual(animation._native_params(),
                         {"path_arc": 0, "path_arc_axis": (0, 0, 1)})
        self.assertEqual(animation._native_kind, "transform")
        self.assertEqual(animation._target_attr, "target_mobject")

    def test_path_and_transform_options_forward_without_reconstruction(self):
        def path(start, end, alpha):
            return start
        animation = self.native.Swap(*self.operands(), path_func=path,
                                     path_arc_axis=(1, 0, 0), run_time=3,
                                     lag_ratio=.2, suspend_mobject_updating=True)
        self.assertIs(animation.path_func, path)
        self.assertEqual(animation.path_arc_axis, (1, 0, 0))
        self.assertEqual(animation.run_time, 3)
        self.assertEqual(animation.lag_ratio, .2)
        self.assertTrue(animation.suspend_mobject_updating)
        self.assertTrue(self.native._requires_python_animation(animation))

    def test_stock_cycles_defer_to_the_ordered_callback_driver(self):
        for cls in (self.native.CyclicReplace, self.native.Swap):
            animation = cls(*self.operands(), rate_func=self.native.linear)
            self.assertTrue(self.native._requires_python_animation(animation))
            self.assertIsNone(animation.target_mobject)
        self.assertEqual(self.native.previous_calls, [])

    def test_unrelated_dispatch_is_preserved(self):
        animation = self.native.Transform(self.native.Mobject())
        self.assertTrue(self.native._requires_python_animation(animation))
        self.assertEqual(self.native.previous_calls, [animation])

    def test_authored_subclass_target_is_called_by_native_target_seam(self):
        parent = self.native.CyclicReplace
        calls = []
        class Custom(parent):
            def create_target(self):
                calls.append(self)
                return super().create_target()
        animation = Custom(*self.operands())
        self.assertTrue(self.native._requires_python_animation(animation))
        target = animation._native_target()
        self.assertEqual(calls, [animation])
        self.assertEqual([obj.center for obj in target], [1, 8, -4])

    def test_instance_and_class_hook_replacement_select_callbacks(self):
        animation = self.native.Swap(*self.operands())
        animation.begin = MethodType(lambda self: None, animation)
        self.assertTrue(self.native._requires_python_animation(animation))
        animation = self.native.Swap(*self.operands())
        self.native.CyclicReplace.begin = lambda self: None
        self.assertTrue(self.native._requires_python_animation(animation))

    def test_nonstandard_endpoints_and_removal_select_callbacks(self):
        for options in ({"final_alpha_value": .25}, {"remover": True},
                        {"replace_mobject_with_target_in_scene": True}):
            animation = self.native.Swap(*self.operands(), **options)
            self.assertTrue(self.native._requires_python_animation(animation))

    def test_unhashable_authored_rate_selects_callbacks(self):
        class Rate:
            __hash__ = None
            def __call__(self, alpha):
                return alpha * alpha
        animation = self.native.Swap(*self.operands(), rate_func=Rate())
        self.assertTrue(self.native._requires_python_animation(animation))

    def test_tagged_arc_paths_still_defer_target_planning(self):
        def arc(start, end, alpha):
            return start
        arc._fmn_path_arc = .5
        animation = self.native.Swap(*self.operands(), path_func=arc)
        self.assertTrue(self.native._requires_python_animation(animation))
        self.assertIsNone(animation.target_mobject)

    def test_invalid_operands_do_not_create_partial_group(self):
        before = self.native.Group.constructions
        for objects, error in (((), ValueError), ((self.native.Mobject(),), ValueError),
                               ((self.native.Mobject(), object()), TypeError)):
            with self.assertRaises(error):
                self.native.CyclicReplace(*objects)
        self.assertEqual(self.native.Group.constructions, before)

    def test_mutated_empty_group_has_empty_target(self):
        animation = self.native.Swap(*self.operands())
        animation.mobject.submobjects.clear()
        self.assertEqual(len(animation.create_target()), 0)

    def test_class_and_qualified_method_identities_are_preserved(self):
        self.assertEqual(self.original_classes, (self.native.CyclicReplace, self.native.Swap))
        self.assertTrue(issubclass(self.native.Swap, self.native.CyclicReplace))
        self.assertEqual(self.native.CyclicReplace.create_target.__name__, "create_target")
        self.assertEqual(self.native.CyclicReplace.create_target.__module__,
                         self.native.CyclicReplace.__module__)
        self.assertEqual(self.native.CyclicReplace.create_target.__qualname__,
                         self.native.CyclicReplace.__qualname__ + ".create_target")


if __name__ == "__main__":
    unittest.main()
