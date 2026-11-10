"""Public matching planning/lifecycle contracts with explicit native doubles.

These tests execute the production adapter, not a native renderer. Real native
geometry and frame semantics are covered by matching_transform_semantics.py.
"""
import importlib.util
from itertools import repeat
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

_PATH = Path(__file__).resolve().parents[1] / "python/fmn_python/matching.py"
_SPEC = importlib.util.spec_from_file_location("matching_under_test", _PATH)
matching = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(matching)


def namespace():
    class Mobject:
        def __init__(self, shape="square", children=()):
            self.shape, self.children, self._scene = shape, list(children), None
        def get_family(self):
            return [self, *(member for child in self.children for member in child.get_family())]
        def family_members_with_points(self):
            return [member for member in self.get_family() if member.shape is not None]
        def has_same_shape_as(self, other):
            return [member.shape for member in self.family_members_with_points()] == [member.shape for member in other.family_members_with_points()]
        def get_center(self):
            return (1., 2., 3.)
        def _is_bound(self):
            return self._scene is not None

    class Animation:
        _native_kind = "transform"
        def __init__(self, source, target=None, **kwargs):
            self.mobject, self.target_mobject, self.kwargs = source, target, kwargs
            self.events = []
        def begin(self):
            self.events.append("begin")
        def finish(self):
            self.events.append("finish")
        def clean_up_from_scene(self, scene):
            self.events.append("cleanup")
        def abort(self):
            self.events.append("abort")

    class Transform(Animation):
        pass
    class FadeOutToPoint(Animation):
        _native_kind = "fade_out_to_point"
    class FadeInFromPoint(Animation):
        _native_kind = "fade_in_from_point"
    class NativeAnimation(Animation):
        pass
    class AnimationGroup(NativeAnimation):
        def __init__(self, *animations, **kwargs):
            self.animations, self.group_options = list(animations), kwargs
            self.mobject, self._composition_driver = None, None
        def begin(self):
            self._composition_driver = tuple(self.animations)
            for child in self.animations:
                child.begin()
        def finish(self):
            for child in self.animations:
                child.finish()
        def abort(self):
            if self._composition_driver is not None:
                for child in self.animations:
                    child.abort()
            self._composition_driver = None
        def clean_up_from_scene(self, scene):
            for child in self.animations:
                child.clean_up_from_scene(scene)
            self._composition_driver = None
    class TransformMatchingParts(NativeAnimation):
        _native_kind = "transform_matching_parts"
        def __init__(self, *args, **kwargs):
            raise NotImplementedError("legacy native-only constructor")
        def _native_params(self):
            return {"matched_pairs": self.matched_pairs}
    class TransformMatchingShapes(TransformMatchingParts):
        _native_kind = "transform_matching_shapes"

    def ensure_root(animation):
        if animation.mobject is None:
            animation.mobject = Mobject(None, [child.mobject for child in animation.animations])
        return animation.mobject
    return SimpleNamespace(
        __engine__="FrankenManim", Mobject=Mobject, Animation=Animation, Transform=Transform,
        AnimationGroup=AnimationGroup, TransformMatchingParts=TransformMatchingParts,
        TransformMatchingShapes=TransformMatchingShapes, FadeOutToPoint=FadeOutToPoint,
        FadeInFromPoint=FadeInFromPoint, _linear_rate=lambda a: a,
        _requires_python_animation=lambda animation: False,
        _fmn_animated_mobjects=lambda animation: [animation.mobject],
        _fmn_ensure_composition_root=ensure_root,
    )


class PlanningTests(unittest.TestCase):
    def setUp(self):
        self.g = namespace()
        matching.install_matching(self.g)
        self.source, self.target = self.g.Mobject(), self.g.Mobject()

    def test_real_engine_marker_does_not_skip_installation(self):
        self.assertEqual(self.g.__engine__, "FrankenManim")
        self.assertEqual(self.g.TransformMatchingParts.__bases__, (self.g.AnimationGroup,))
        self.g.TransformMatchingParts(self.source, self.target)

    def test_aliases_and_idempotent_installation(self):
        alias = self.g.TransformMatchingShapes
        method = self.g.TransformMatchingParts.begin
        matching.install_matching(self.g)
        self.assertIs(alias, self.g.TransformMatchingShapes)
        self.assertIs(method, self.g.TransformMatchingParts.begin)
        self.assertEqual(alias._native_kind, "transform_matching_shapes")

    def test_shape_predicate_and_authored_factory_dispatch(self):
        calls = []
        def factory(source, target, **kwargs):
            calls.append((source, target, kwargs))
            return self.g.Transform(source, target, **kwargs)
        animation = self.g.TransformMatchingParts(self.source, self.target, match_animation=factory, path_arc=1.5)
        self.assertEqual(calls, [(self.source, self.target, {"path_arc": 1.5})])
        self.assertIs(animation.animations[0].mobject, self.source)
        self.assertIs(animation.animations[0].target_mobject, self.target)
        self.assertTrue(self.g._requires_python_animation(animation))
        self.assertFalse(self.g._requires_python_animation(self.g.Transform(self.source, self.target)))

    def test_custom_transform_interpolation_is_not_lowered_away(self):
        class Authored(self.g.Transform):
            def interpolate_submobject(self, *args):
                raise AssertionError("classification must not probe a callback")
        animation = self.g.TransformMatchingParts(self.source, self.target, match_animation=Authored)
        self.assertTrue(self.g._requires_python_animation(animation.animations[0]))

    def test_instance_override_and_changed_base_dispatch(self):
        child = self.g.Transform(self.source, self.target)
        child.finish = lambda: None
        self.assertTrue(self.g._requires_python_animation(child))
        other = self.g.Transform(self.source, self.target)
        self.g.Animation.begin = lambda self: None
        self.assertTrue(self.g._requires_python_animation(other))

    def test_unchanged_native_specialization_stays_native(self):
        g = namespace()
        class Shipped(g.Transform):
            def interpolate_submobject(self, *args):
                pass
        g.Shipped = Shipped
        matching.install_matching(g)
        self.assertFalse(g._requires_python_animation(Shipped(g.Mobject(), g.Mobject())))

    def test_descriptor_is_not_executed_during_dispatch(self):
        class Authored(self.g.Transform):
            @property
            def finish(self):
                raise AssertionError("classification evaluated an authored descriptor")
        self.assertTrue(self.g._requires_python_animation(Authored(self.source, self.target)))

    def test_explicit_claim_precedes_matcher_and_keeps_metadata(self):
        calls = []
        class Custom(self.g.TransformMatchingParts):
            def add_transform(self, source, target):
                calls.append(("add", source, target))
                return super().add_transform(source, target)
            def find_pairs_with_matching_shapes(self, sources, targets):
                calls.append(("find", len(sources), len(targets)))
                return super().find_pairs_with_matching_shapes(sources, targets)
        animation = Custom(self.source, self.target, matched_pairs=[(self.source, self.target)])
        self.assertEqual(calls, [("add", self.source, self.target), ("find", 0, 0)])
        self.assertEqual(animation._native_params(), {"matched_pairs": [(self.source, self.target)]})

    def test_overlapping_explicit_group_claim_wins(self):
        source = self.g.Mobject(None, [self.source, self.g.Mobject("circle")])
        target = self.g.Mobject(None, [self.target, self.g.Mobject("triangle")])
        calls = []
        def factory(s, t, **kwargs):
            calls.append((s, t))
            return self.g.Transform(s, t, **kwargs)
        animation = self.g.TransformMatchingParts(source, target,
            matched_pairs=[(source, target), (self.source, self.target)], mismatch_animation=factory)
        self.assertEqual(calls, [(source, target)])
        self.assertEqual(len(animation.animations), 1)

    def test_matcher_can_return_an_already_claimed_pair(self):
        class Custom(self.g.TransformMatchingParts):
            def find_pairs_with_matching_shapes(inner, sources, targets):
                return [(self.source, self.target)]
        animation = Custom(self.source, self.target, matched_pairs=[(self.source, self.target)])
        self.assertEqual(len(animation.animations), 1)

    def test_unmatched_pieces_get_directional_fades(self):
        source = self.g.Mobject(None, [self.source, self.g.Mobject("circle")])
        target = self.g.Mobject(None, [self.target, self.g.Mobject("triangle")])
        animation = self.g.TransformMatchingShapes(source, target)
        self.assertEqual([a._native_kind for a in animation.animations],
                         ["transform", "fade_out_to_point", "fade_in_from_point"])
        self.assertEqual(animation.animations[1].target_mobject, target.get_center())
        self.assertEqual(animation.animations[2].target_mobject, source.get_center())

    def test_child_easing_is_not_duplicated_on_group(self):
        easing = lambda a: a * a
        animation = self.g.TransformMatchingParts(self.source, self.target, rate_func=easing, run_time=4, lag_ratio=0.2)
        self.assertIs(animation.animations[0].kwargs["rate_func"], easing)
        self.assertIs(animation.group_options["rate_func"], self.g._linear_rate)
        self.assertEqual(animation.group_options["run_time"], 4)
        self.assertEqual(animation.group_options["lag_ratio"], 0.2)

    def test_generator_matcher_is_frozen_before_claims(self):
        sources, targets = [self.g.Mobject() for _ in range(3)], [self.g.Mobject() for _ in range(3)]
        class Custom(self.g.TransformMatchingParts):
            def find_pairs_with_matching_shapes(self, sources, targets):
                return ((s, t) for s, t in zip(sources, targets))
        animation = Custom(self.g.Mobject(None, sources), self.g.Mobject(None, targets))
        self.assertEqual(len(animation.animations), 3)
        self.assertEqual([a.mobject for a in animation.animations], sources)

    def test_shared_descendants_are_claimed_once(self):
        source = self.g.Mobject(None, [self.source, self.source])
        target = self.g.Mobject(None, [self.target, self.target])
        animation = self.g.TransformMatchingParts(source, target)
        self.assertEqual(len(animation.animations), 1)

    def test_factory_failure_keeps_both_candidate_lists(self):
        failure = RuntimeError("factory")
        def broken(*args, **kwargs):
            raise failure
        animation = object.__new__(self.g.TransformMatchingParts)
        with self.assertRaises(RuntimeError) as caught:
            animation.__init__(self.source, self.target, match_animation=broken)
        self.assertIs(caught.exception, failure)
        self.assertEqual(animation.source_pieces, [self.source])
        self.assertEqual(animation.target_pieces, [self.target])
        self.assertEqual(animation.anims, [])

    def test_factory_must_return_animation(self):
        with self.assertRaisesRegex(TypeError, "return an Animation"):
            self.g.TransformMatchingParts(self.source, self.target, match_animation=lambda *a, **k: None)

    def test_foreign_pairs_and_cross_scene_ownership_refuse(self):
        with self.assertRaisesRegex(ValueError, "foreign source"):
            self.g.TransformMatchingParts(self.source, self.target, matched_pairs=[(self.g.Mobject(), self.target)])
        self.source._scene, self.target._scene = object(), object()
        with self.assertRaisesRegex(ValueError, "multiple Scenes"):
            self.g.TransformMatchingParts(self.source, self.target)

    def test_invalid_inputs(self):
        for pair in [None, (self.source,), (self.source, self.target, self.source), (None, self.target)]:
            with self.subTest(pair=pair), self.assertRaises(TypeError):
                self.g.TransformMatchingParts(self.source, self.target, matched_pairs=[pair])
        with self.assertRaisesRegex(TypeError, "two Mobject families"):
            self.g.TransformMatchingParts(self.source, None)
        with self.assertRaises(TypeError):
            self.g.TransformMatchingParts(self.source, self.target, match_animation=None)

    def test_zero_ink_operands_build_unmatched_fades_or_an_empty_timed_group(self):
        for start, end, kinds in ((None, "square", ["fade_in_from_point"]),
                                   ("square", None, ["fade_out_to_point"]),
                                   (None, None, [])):
            with self.subTest(start=start, end=end):
                animation = self.g.TransformMatchingParts(
                    self.g.Mobject(start), self.g.Mobject(end), run_time=3,
                )
                self.assertEqual([child._native_kind for child in animation.animations], kinds)
                self.assertEqual(animation.group_options["run_time"], 3)
                animation.begin()
                animation.finish()

    def test_pair_and_comparison_budgets(self):
        with patch.object(matching, "_MAX_PARTS", 3), self.assertRaisesRegex(ValueError, "budget"):
            self.g.TransformMatchingParts(self.source, self.target, matched_pairs=repeat((self.source, self.target)))
        with patch.object(matching, "_MAX_COMPARISONS", 0), self.assertRaisesRegex(ValueError, "comparison"):
            self.g.TransformMatchingParts(self.source, self.target)


class CleanupTests(unittest.TestCase):
    setUp = PlanningTests.setUp

    def test_cleanup_order_and_exactly_once(self):
        animation = self.g.TransformMatchingParts(self.source, self.target)
        events = []
        scene = SimpleNamespace(remove=lambda *objects: events.append(("remove", objects)),
                                add=lambda *objects: events.append(("add", objects)))
        self.source._scene = scene
        animation.animations[0].clean_up_from_scene = lambda scene: events.append(("child",))
        animation.begin()
        animation.finish()
        animation.clean_up_from_scene(scene)
        animation.clean_up_from_scene(scene)
        self.assertEqual(events, [("child",), ("remove", (animation.mobject, self.source)), ("add", (self.target,))])

    def test_detached_wrapper_has_no_native_handle_to_remove(self):
        source = self.g.Mobject(None, [self.source])
        animation = self.g.TransformMatchingParts(source, self.target)
        events = []
        scene = SimpleNamespace(remove=lambda *objects: events.append(("remove", objects)),
                                add=lambda *objects: events.append(("add", objects)))
        self.source._scene = scene  # Native leaf adoption does not bind its parent wrapper.
        animation.begin()
        animation.finish()
        animation.clean_up_from_scene(scene)
        self.assertEqual(events, [("remove", (animation.mobject,)), ("add", (self.target,))])
        self.assertFalse(source._is_bound())

    def test_failed_child_cleanup_never_publishes_target(self):
        animation = self.g.TransformMatchingParts(self.source, self.target)
        failure, calls = ValueError("cleanup"), []
        def cleanup(scene):
            calls.append("cleanup")
            raise failure
        animation.animations[0].clean_up_from_scene = cleanup
        scene = SimpleNamespace(remove=lambda *a: calls.append("remove"), add=lambda *a: calls.append("add"))
        animation.begin()
        animation.finish()
        with self.assertRaises(ValueError) as caught:
            animation.clean_up_from_scene(scene)
        self.assertIs(caught.exception, failure)
        animation.clean_up_from_scene(scene)
        self.assertEqual(calls, ["cleanup"])
        self.assertIsNone(animation._composition_driver)

    def test_failed_removal_never_publishes_target(self):
        animation = self.g.TransformMatchingParts(self.source, self.target)
        calls = []
        def remove(*objects):
            raise RuntimeError("remove")
        scene = SimpleNamespace(remove=remove, add=lambda *a: calls.append("add"))
        animation.begin()
        animation.finish()
        with self.assertRaises(RuntimeError):
            animation.clean_up_from_scene(scene)
        self.assertEqual(calls, [])

    def test_abort_and_premature_cleanup(self):
        animation = self.g.TransformMatchingParts(self.source, self.target)
        with self.assertRaises(RuntimeError):
            animation.clean_up_from_scene(None)
        animation.begin()
        animation.abort()
        self.assertIsNone(animation._composition_driver)
        self.assertEqual(animation.animations[0].events, ["begin", "abort"])
        with self.assertRaises(RuntimeError):
            animation.clean_up_from_scene(None)


if __name__ == "__main__":
    unittest.main()
