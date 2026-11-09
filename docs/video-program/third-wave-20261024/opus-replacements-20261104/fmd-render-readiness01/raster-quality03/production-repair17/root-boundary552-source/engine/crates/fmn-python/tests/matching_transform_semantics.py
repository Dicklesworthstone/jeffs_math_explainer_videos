"""Real-extension matching acceptance; run by check_portal_runtime.sh.

No fixture storage or source injection: the imported production module owns
all geometry, scene membership, record alignment, and animation execution.
"""
import numpy as np
import manimlib as ml
from manimlib.animation.transform_matching_parts import TransformMatchingParts as QualifiedParts


def test_parts_hooks_select_actual_motion():
    calls, samples = [], []
    class AuthoredTransform(ml.Transform):
        def interpolate_submobject(self, submobject, start, target, alpha):
            result = super().interpolate_submobject(submobject, start, target, alpha)
            if submobject is self.mobject:
                samples.append((float(alpha), submobject.get_center().copy()))
            return result
    class AuthoredParts(ml.TransformMatchingParts):
        def find_pairs_with_matching_shapes(self, sources, targets):
            calls.append(("find", len(sources), len(targets)))
            return super().find_pairs_with_matching_shapes(sources, targets)
        def add_transform(self, source, target):
            calls.append(("add", source, target))
            return super().add_transform(source, target)
    source = ml.Square()
    target = ml.Square().shift(2 * ml.RIGHT)
    unrelated = ml.Circle().shift(4 * ml.UP)
    animation = AuthoredParts(source, target, matched_pairs=[(source, target)],
                             match_animation=AuthoredTransform, run_time=4 / 30, rate_func=ml.linear)
    assert QualifiedParts is ml.TransformMatchingParts
    assert isinstance(animation, ml.AnimationGroup)
    assert calls[0] == ("add", source, target)
    assert calls[1] == ("find", 0, 0)
    scene = ml.Scene()
    scene.add(source, unrelated)
    scene.play(animation)
    assert samples, "custom matching Transform never executed"
    midpoints = [center for alpha, center in samples if np.isclose(alpha, .5)]
    assert midpoints, "the real frame loop never sampled the midpoint"
    np.testing.assert_allclose(midpoints[0], [1., 0., 0.], atol=1e-7)
    assert source not in scene.mobjects
    assert animation.mobject not in scene.mobjects
    assert sum(obj is target for obj in scene.mobjects) == 1
    assert unrelated in scene.mobjects


def test_explicit_group_claim_and_custom_mismatch():
    made = []
    class Mismatch(ml.Transform):
        def __init__(self, source, target, **kwargs):
            made.append((source, target))
            super().__init__(source, target, **kwargs)
    source = ml.VGroup(ml.Square().shift(ml.LEFT), ml.Circle().shift(ml.RIGHT))
    target = ml.VGroup(ml.Circle().shift(ml.LEFT), ml.Square().shift(ml.RIGHT))
    animation = ml.TransformMatchingParts(source, target, matched_pairs=[(source, target), (source[0], target[0])],
                                         mismatch_animation=Mismatch, run_time=2 / 30)
    assert made == [(source, target)]
    assert len(animation.animations) == 1
    scene = ml.Scene()
    scene.add(source)
    scene.play(animation)
    assert target in scene.mobjects and source not in scene.mobjects


def test_stock_native_matches_and_unmatched_fades():
    source = ml.VGroup(ml.Square().shift(ml.LEFT), ml.Circle().shift(ml.RIGHT))
    target = ml.VGroup(ml.Square().shift(2 * ml.UP), ml.Triangle().shift(2 * ml.DOWN))
    animation = ml.TransformMatchingShapes(source, target, run_time=3 / 30)
    kinds = [anim._native_kind for anim in animation.animations]
    assert "transform" in kinds
    # Unmatched pieces fade to/from points, as transform_matching_parts.py
    # builds them. The point fades are the Reference's FadeOut/FadeIn
    # subclasses, so they ride the fade_out/fade_in kinds.
    classes = [type(anim) for anim in animation.animations]
    assert ml.FadeOutToPoint in classes and ml.FadeInFromPoint in classes
    assert {"fade_out", "fade_in"} <= set(kinds)
    scene = ml.Scene()
    scene.add(source)
    scene.play(animation)
    assert sum(obj is target for obj in scene.mobjects) == 1
    assert source not in scene.mobjects and animation.mobject not in scene.mobjects
    assert len(target.family_members_with_points()) == 2


def test_matching_inside_mixed_succession():
    source = ml.Square()
    target = ml.Square().shift(ml.RIGHT)
    animation = ml.TransformMatchingParts(source, target, run_time=2 / 30, rate_func=ml.linear)
    marker = ml.Circle().shift(3 * ml.UP)
    finish = marker.copy().shift(ml.RIGHT)
    scene = ml.Scene()
    scene.add(source, marker)
    scene.play(ml.Succession(animation, ml.Transform(marker, finish, run_time=2 / 30, rate_func=ml.linear)))
    np.testing.assert_allclose(marker.get_center(), finish.get_center(), atol=1e-7)
    assert any(target is member for root in scene.mobjects for member in root.get_family())


def test_matching_exception_does_not_publish_target():
    class BrokenTransform(ml.Transform):
        def interpolate_submobject(self, submobject, start, target, alpha):
            if alpha > 0:
                raise ValueError("matching authored failure")
            return super().interpolate_submobject(submobject, start, target, alpha)
    source, target = ml.Square(), ml.Square().shift(ml.RIGHT)
    animation = ml.TransformMatchingParts(source, target, match_animation=BrokenTransform, run_time=3 / 30)
    scene = ml.Scene()
    scene.add(source)
    try:
        scene.play(animation)
    except ValueError as error:
        assert "matching authored failure" in str(error)
    else:
        raise AssertionError("an authored matching exception was swallowed")
    assert target not in scene.mobjects
    # A failed matching play must not poison the next native segment.
    scene.play(ml.Transform(source, source.copy().shift(ml.UP), run_time=2 / 30))


def test_string_matching_override_reaches_scene_play():
    calls = []
    class AuthoredStrings(ml.TransformMatchingStrings):
        def matching_blocks(self, source, target, matched_keys, key_map):
            calls.append((source, target))
            return [(source._string_submobject(0), target._string_submobject(0))]
    source, target = ml.Text("ab"), ml.Text("ba").shift(2 * ml.RIGHT)
    animation = AuthoredStrings(source, target, run_time=3 / 30)
    assert calls == [(source, target)]
    assert isinstance(animation, ml.TransformMatchingParts)
    assert animation.animations[0].mobject is source._string_submobject(0)
    assert animation.animations[0].target_mobject is target._string_submobject(0)
    scene = ml.Scene()
    scene.add(source)
    scene.play(animation)
    assert source not in scene.mobjects and target in scene.mobjects


def test_multi_glyph_rename_uses_native_spans():
    source, target = ml.Text("foo+foo"), ml.Text("bar+bar").shift(ml.UP)
    animation = ml.TransformMatchingStrings(source, target, key_map={"foo": "bar"}, run_time=3 / 30)
    matches = [child for child in animation.animations if child._native_kind == "transform"]
    assert len(matches) == 3
    assert len(matches[0].mobject.family_members_with_points()) == 3
    scene = ml.Scene()
    scene.add(source)
    scene.play(animation)
    assert target in scene.mobjects and source not in scene.mobjects


def test_reordered_tex_retains_native_semantic_keys():
    source = ml.Tex("x+y", isolate=["x", "y"])
    target = ml.Tex("y+x", isolate=["x", "y"]).shift(ml.UP)
    animation = ml.TransformMatchingTex(source, target, run_time=3 / 30)
    assert isinstance(animation, ml.TransformMatchingParts)
    assert any(child._native_kind == "transform" for child in animation.animations)
    scene = ml.Scene()
    scene.add(source)
    scene.play(animation)
    assert target in scene.mobjects and source not in scene.mobjects


def run_matching_acceptance():
    assert getattr(ml, "__franken_manim__", False), "requires the real FrankenManim extension"
    cases = sorted((name, value) for name, value in globals().items()
                   if name.startswith("test_") and callable(value))
    assert cases, "zero matching-transform acceptance cases"
    for name, case in cases:
        case()
        print("matching acceptance passed:", name)
    print("matching-transform native cases:", len(cases))


run_matching_acceptance()
