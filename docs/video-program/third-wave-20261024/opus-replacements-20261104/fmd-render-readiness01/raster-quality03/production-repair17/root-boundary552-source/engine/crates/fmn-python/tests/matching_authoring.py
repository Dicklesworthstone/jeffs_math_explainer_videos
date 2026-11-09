"""Installed-native matching authoring, intermediate frames and failure paths.

Registered in check_portal_runtime.sh; no source injection or native doubles.
"""
from pathlib import Path
import tempfile

import numpy as np
import manimlib as ml


def test_string_hook_renders_translated_text_at_actual_sample_points():
    def render(destination, threads):
        calls, samples = [], []
        class RecordedTransform(ml.Transform):
            def interpolate_mobject(self, alpha):
                result = super().interpolate_mobject(alpha)
                # Empty group parents precede their glyph children in the
                # family walk. Observe the complete frame, not the parent's
                # submobject hook before the descendants have interpolated.
                samples.append((float(alpha), self.mobject.get_center().copy()))
                return result
        class AuthoredStrings(ml.TransformMatchingStrings):
            def matching_blocks(self, source, target, matched_keys, key_map):
                calls.append((source, target))
                return super().matching_blocks(source, target, matched_keys, key_map)
        scene = ml.Scene()
        with scene.render_session(destination, format="png_sequence", resolution=(160, 90),
                                  fps=4, threads=threads) as session:
            source = ml.Text("move").move_to(ml.LEFT)
            target = ml.Text("move").move_to(ml.RIGHT)
            source_spans, target_spans = list(source._string_sub_spans), list(target._string_sub_spans)
            scene.add(source)
            animation = AuthoredStrings(source, target, match_animation=RecordedTransform,
                                        key_map={"move": "move"}, run_time=1, rate_func=ml.linear)
            scene.play(animation)
            assert len(calls) == 1, "constructor must call the authored block hook exactly once"
            assert animation.matched_pairs == [], "inferred blocks are not explicit native pair claims"
            assert source._string_sub_spans == source_spans and target._string_sub_spans == target_spans
            assert target in scene.mobjects and source not in scene.mobjects
        assert session.result.frame_count == 4
        midpoints = [point for alpha, point in samples if np.isclose(alpha, .5)]
        assert midpoints, "native playback never invoked the authored midpoint"
        np.testing.assert_allclose(midpoints[0], [0, 0, 0], atol=1e-6)
        frames = [path.read_bytes() for path in sorted(Path(destination).glob("*.png"))]
        assert len(frames) == 4 and len(set(frames)) > 1, "requires actual changing native images"
        return frames
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        assert render(root / "one", 1) == render(root / "four", 4), "thread count changed matching output"


def test_custom_block_plan_controls_real_children_and_native_metadata():
    calls = []
    class FirstOnly(ml.TransformMatchingStrings):
        def matching_blocks(self, source, target, matched_keys, key_map):
            calls.append("blocks")
            return [(source._string_submobject(0), target._string_submobject(0))]
    source, target = ml.Text("ab"), ml.Text("ba").shift(ml.RIGHT)
    animation = FirstOnly(source, target, run_time=2 / 30)
    # The matched block transforms; the leftovers fade to/from points
    # (FadeOutToPoint/FadeInFromPoint are the Reference's FadeOut/FadeIn
    # subclasses, riding the fade_out/fade_in kinds).
    assert [child._native_kind for child in animation.animations] == [
        "transform", "fade_out", "fade_in",
    ]
    assert [type(child) for child in animation.animations[1:]] == [
        ml.FadeOutToPoint, ml.FadeInFromPoint,
    ]
    params = animation._native_params()
    assert "source_keys" in params and "target_keys" in params
    scene = ml.Scene()
    scene.add(source)
    scene.play(animation)
    assert calls == ["blocks"]
    assert target in scene.mobjects and source not in scene.mobjects


def test_invalid_native_span_cannot_be_bypassed_by_authored_matcher():
    calls = []
    class Authored(ml.TransformMatchingStrings):
        def matching_blocks(self, *args):
            calls.append("blocks")
            return []
    source, target = ml.Text("é"), ml.Text("é")
    previous = source._string_sub_spans
    try:
        source._string_sub_spans = [(0, 1)]
        try:
            Authored(source, target)
        except Exception as error:
            assert "UTF-8" in str(error), error
        else:
            raise AssertionError("invalid UTF-8 source span accepted")
        assert calls == []
    finally:
        source._string_sub_spans = previous


def test_matching_callback_failure_keeps_source_and_allows_next_play():
    failure = RuntimeError("authored matching interpolation failed")
    class Broken(ml.Transform):
        def interpolate_submobject(self, current, start, target, alpha):
            if alpha > 0:
                raise failure
            return super().interpolate_submobject(current, start, target, alpha)
    source, target = ml.Square(), ml.Square().shift(ml.RIGHT)
    scene = ml.Scene()
    scene.add(source)
    animation = ml.TransformMatchingParts(source, target, match_animation=Broken,
                                         suspend_mobject_updating=True, run_time=2 / 30)
    try:
        scene.play(animation)
    except RuntimeError as error:
        assert error is failure
    else:
        raise AssertionError("authored matching failure disappeared")
    assert target not in scene.mobjects
    assert not source._is_updating_suspended()
    assert animation._composition_driver is None
    scene.play(ml.Transform(source, source.copy().shift(ml.UP), run_time=2 / 30))
    np.testing.assert_allclose(source.get_center(), [0, 1, 0], atol=1e-6)


def test_matching_implicitly_adopts_unadded_text_tex_and_group_pieces():
    factories = (
        (lambda: (ml.Text("move"), ml.Text("more").shift(ml.RIGHT)), ml.TransformMatchingStrings),
        (lambda: (ml.Tex("x + y"), ml.Tex("z - x").shift(ml.RIGHT)), ml.TransformMatchingTex),
        (lambda: (ml.Square(), ml.Square().shift(ml.RIGHT)), ml.TransformMatchingShapes),
        (lambda: (ml.VGroup(ml.Square(), ml.VGroup(ml.Circle().shift(ml.LEFT))),
                  ml.VGroup(ml.Square().shift(ml.UP), ml.VGroup(ml.Circle().shift(ml.RIGHT)))),
         ml.TransformMatchingParts),
    )
    for create, animation_type in factories:
        source, target = create()
        scene, unrelated = ml.Scene(), ml.Dot().shift(3 * ml.UP)
        scene.add(unrelated)
        assert not source._is_bound() and not target._is_bound()
        animation = animation_type(source, target, run_time=.125)
        scene.play(animation)
        assert scene.mobjects == [unrelated, target]
        assert animation._composition_driver is None
        scene.wait(.125)


def test_bound_source_removal_failure_is_not_treated_as_detached():
    source, target = ml.Text("same"), ml.Text("same").shift(ml.RIGHT)
    failure, removals = RuntimeError("bound source removal refused"), []
    class RefusingScene(ml.Scene):
        def remove(self, *objects):
            if any(obj is source for obj in objects):
                removals.append(objects)
                raise failure
            return super().remove(*objects)
    scene = RefusingScene()
    scene.add(source)
    animation = ml.TransformMatchingStrings(source, target, run_time=.125)
    try:
        scene.play(animation)
    except RuntimeError as error:
        assert error is failure
    else:
        raise AssertionError("bound source failure was silently ignored")
    assert len(removals) == 1 and target not in scene.mobjects
    displayed = [member for root in scene.mobjects for member in root.get_family()]
    assert all(any(member is piece for member in displayed)
               for piece in source.family_members_with_points())
    assert animation._composition_driver is None
    animation.clean_up_from_scene(scene)
    assert len(removals) == 1, "failed authored cleanup was retried"


def test_foreign_bound_source_still_refuses_before_publication():
    from manimlib.exceptions import ForeignStageError
    source, target = ml.Tex("x"), ml.Tex("y")
    owner, scene = ml.Scene(), ml.Scene()
    owner.add(source)
    animation = ml.TransformMatchingTex(source, target, run_time=.125)
    try:
        scene.play(animation)
    except ForeignStageError:
        pass
    else:
        raise AssertionError("matching crossed Scene ownership")
    assert target not in scene.mobjects and source in owner.mobjects



# Matching is also the transition into and out of an empty formula/chart.
# Pinned Reference transform_matching_parts.py builds the unmatched fades
# without requiring either operand to have points.
def test_empty_shape_transitions_use_the_existing_fades_and_cleanup():
    for cls in (ml.TransformMatchingParts, ml.TransformMatchingShapes):
        for initial, final in ((False, True), (True, False), (False, False)):
            for preadded in (False, True):
                source = ml.VGroup(ml.Square()) if initial else ml.VGroup()
                target = ml.VGroup(ml.Circle()) if final else ml.VGroup()
                marker = ml.Dot().shift(3 * ml.UP)
                scene = ml.Scene().add(marker)
                if preadded:
                    scene.add(source)
                animation = cls(source, target, run_time=.125, rate_func=ml.linear)
                expected = ([ml.FadeOutToPoint] if initial else
                            [ml.FadeInFromPoint] if final else [])
                assert [type(child) for child in animation.animations] == expected
                scene.play(animation)
                assert scene.mobjects == [marker, target]
                assert np.isclose(scene.time, 4 / 30)
                assert animation._composition_driver is None
                animation.clean_up_from_scene(scene)
                assert scene.mobjects == [marker, target], 'cleanup must be idempotent'


def test_empty_strings_keep_native_span_metadata_without_fake_glyphs():
    for text_type, matcher in ((ml.Text, ml.TransformMatchingStrings),
                               (ml.MarkupText, ml.TransformMatchingStrings),
                               (ml.Tex, ml.TransformMatchingTex),
                               (ml.TexText, ml.TransformMatchingTex)):
        for initial, final in (('', 'xy'), ('xy', ''), ('', '')):
            source, target = text_type(initial), text_type(final)
            old = [(list(obj._string_sub_spans), [list(p) for p in obj._string_sub_paths])
                   for obj in (source, target)]
            animation = matcher(source, target, run_time=.125)
            params = animation._native_params()
            assert bool(params['source_keys']) == bool(initial)
            assert bool(params['target_keys']) == bool(final)
            scene = ml.Scene().add(source)
            scene.play(animation)
            assert scene.mobjects == [target]
            assert [(list(obj._string_sub_spans), obj._string_sub_paths)
                    for obj in (source, target)] == old
            assert bool(target.family_members_with_points()) == bool(final)


def test_whitespace_and_invisible_tex_are_legitimate_zero_ink_operands():
    for empty in (ml.Text(' \t\n'), ml.Tex(r'\phantom{x}')):
        assert not empty.family_members_with_points()
        source = ml.Text('word')
        animation = ml.TransformMatchingStrings(source, empty, run_time=.125)
        scene = ml.Scene().add(source)
        scene.play(animation)
        assert scene.mobjects == [empty]
        assert not empty.family_members_with_points()


def test_empty_match_still_executes_public_planning_and_lifecycle_hooks():
    calls = []
    class Authored(ml.TransformMatchingStrings):
        def matching_blocks(self, source, target, matched_keys, key_map):
            calls.append(('plan', source, target))
            return super().matching_blocks(source, target, matched_keys, key_map)
        def begin(self):
            calls.append('begin')
            return super().begin()
        def interpolate(self, alpha):
            calls.append(('alpha', float(alpha)))
            return super().interpolate(alpha)
        def finish(self):
            calls.append('finish')
            return super().finish()
        def clean_up_from_scene(self, scene):
            calls.append('cleanup')
            return super().clean_up_from_scene(scene)
    source, target = ml.Text(''), ml.Text('')
    animation = Authored(source, target, run_time=.125)
    scene = ml.Scene().add(source)
    scene.play(animation)
    assert calls.count(('plan', source, target)) == 1
    assert calls.count('begin') == calls.count('finish') == calls.count('cleanup') == 1
    assert any(isinstance(call, tuple) and call[0] == 'alpha' and 0 < call[1] < 1
               for call in calls), 'the empty transition was short-circuited instead of timed'
    assert scene.mobjects == [target]


def test_empty_matches_compose_with_native_succession_and_repeat():
    source, blank, target = ml.Text('a'), ml.Text(''), ml.Text('b')
    first = ml.TransformMatchingStrings(source, blank, run_time=.125)
    second = ml.TransformMatchingStrings(blank, target, run_time=.125)
    marker = ml.Square().shift(2 * ml.DOWN)
    scene = ml.Scene().add(source, marker)
    scene.play(ml.Succession(first, second), run_time=.25)
    assert target in scene.mobjects and blank not in scene.mobjects and source not in scene.mobjects
    assert np.isclose(scene.time, 8 / 30)
    scene.play(ml.Transform(marker, marker.copy().shift(ml.RIGHT)), run_time=.125)
    np.testing.assert_allclose(marker.get_center(), [1, -2, 0], atol=1e-6)
    # Reuse a completed all-empty plan; no duplicate target or stale driver.
    a, b = ml.Text(''), ml.Text('')
    animation = ml.TransformMatchingStrings(a, b, run_time=.125)
    scene.add(a)
    scene.play(animation)
    scene.play(animation)
    assert sum(obj is b for obj in scene.mobjects) == 1
    assert animation._composition_driver is None


def test_skipping_empty_matches_replaces_roots_without_rendering_ink():
    for start, end in (('', 'a'), ('a', ''), ('', '')):
        source, target = ml.Text(start), ml.Text(end)
        scene = ml.Scene(skip_animations=True).add(source)
        scene.play(ml.TransformMatchingStrings(source, target, run_time=.125))
        assert scene.mobjects == [target]
        assert np.isclose(scene.time, 4 / 30)


def test_empty_match_failure_does_not_publish_and_next_native_play_works():
    failure = RuntimeError('empty matching finish failed')
    class Broken(ml.TransformMatchingStrings):
        def finish(self):
            super().finish()
            raise failure
    source, target = ml.Text(''), ml.Text('')
    marker = ml.Square()
    scene = ml.Scene().add(source, marker)
    animation = Broken(source, target, run_time=.125)
    try:
        scene.play(animation)
    except RuntimeError as error:
        assert error is failure
    else:
        raise AssertionError('empty matching swallowed an authored exception')
    assert target not in scene.mobjects
    assert animation._composition_driver is None
    scene.play(ml.Transform(marker, marker.copy().shift(ml.UP)), run_time=.125)
    np.testing.assert_allclose(marker.get_center(), [0, 1, 0], atol=1e-6)


def test_point_free_foreign_owners_are_checked_before_any_frame():
    from manimlib.exceptions import ForeignStageError
    for initial, final in ((False, True), (True, False), (False, False)):
        for foreign_side in (0, 1):
            source = ml.VGroup(ml.Square()) if initial else ml.VGroup()
            target = ml.VGroup(ml.Circle()) if final else ml.VGroup()
            animation = ml.TransformMatchingParts(source, target, run_time=.125)
            # Change ownership after construction, as source-unedited scene
            # setup can do. Empty operands must not escape leaf admission.
            foreign = (source, target)[foreign_side]
            owner = ml.Scene().add(foreign)
            scene = ml.Scene()
            before = [obj.get_all_points().copy() for obj in (source, target)]
            try:
                scene.play(animation)
            except ForeignStageError:
                pass
            else:
                raise AssertionError('matching accepted a foreign point-free operand')
            assert scene.time == 0
            assert target not in scene.mobjects and foreign in owner.mobjects
            for obj, points in zip((source, target), before):
                np.testing.assert_array_equal(obj.get_all_points(), points)


def test_empty_span_admission_does_not_hide_missing_or_inconsistent_maps():
    calls = []
    class Authored(ml.TransformMatchingStrings):
        def matching_blocks(self, *args):
            calls.append('plan')
            return []
    for mode in ('missing', 'path_only', 'bad_utf8', 'extra_path'):
        source = ml.Text('') if mode == 'path_only' else ml.Text('é')
        target = ml.Text('')
        if mode == 'missing':
            source._string_sub_spans, source._string_sub_paths = [], []
        elif mode == 'path_only':
            source._string_sub_paths = [[0]]
        elif mode == 'bad_utf8':
            source._string_sub_spans = [(0, 1)]
        else:
            source._string_sub_paths.append([0])
        try:
            Authored(source, target)
        except Exception as error:
            assert 'span' in str(error).lower() or 'utf-8' in str(error).lower(), error
        else:
            raise AssertionError('empty matching admitted a corrupt native span map: ' + mode)
        assert calls == [], 'invalid provenance reached the authored matcher'


def test_empty_matches_do_not_bypass_user_option_validation():
    from manimlib.exceptions import TexError
    source, target = ml.Text(''), ml.Text('')
    for options in ({'match_animation': None}, {'mismatch_animation': None},
                    {'key_map': {'a': 3}}, {'matched_keys': [3]},
                    {'matched_pairs': [(object(), object())]}):
        try:
            ml.TransformMatchingStrings(source, target, **options)
        except (TypeError, ValueError, TexError):
            pass
        else:
            raise AssertionError('empty plan accepted invalid options: ' + repr(options))


def test_empty_matching_frames_equal_independent_native_fades():
    def render(destination, transition, use_matcher, threads):
        scene = ml.Scene()
        with scene.render_session(destination, format='png_sequence', resolution=(96, 54),
                                  fps=8, threads=threads):
            source = ml.Text('xy') if transition == 'clear' else ml.Text('')
            target = ml.Text('xy') if transition == 'create' else ml.Text('')
            scene.add(ml.Dot().shift(3 * ml.UP), source)
            if use_matcher:
                scene.play(ml.TransformMatchingStrings(source, target, run_time=.25,
                                                      rate_func=ml.linear))
            else:
                fades = [ml.FadeOutToPoint(part, target.get_center(), rate_func=ml.linear)
                         for part in source.family_members_with_points()]
                fades += [ml.FadeInFromPoint(part, source.get_center(), rate_func=ml.linear)
                          for part in target.family_members_with_points()]
                if fades:
                    scene.play(ml.AnimationGroup(*fades), run_time=.25)
                else:
                    scene.wait(.25)
                scene.remove(source)
                scene.add(target)
            scene.wait(.125)  # Observe the exact endpoint after cleanup.
        return [p.read_bytes() for p in sorted(Path(destination).glob('*.png'))]
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        for transition in ('create', 'clear', 'blank'):
            control = render(root/(transition+'control'), transition, False, 1)
            assert len(control) == 3
            if transition != 'blank':
                assert len(set(control)) > 1, 'requires real changing output'
            for threads in (1, 4, 16):
                actual = render(root/(transition+str(threads)), transition, True, threads)
                assert actual == control, (transition, threads)



assert getattr(ml, "__franken_manim__", False), "requires the real native FrankenManim portal"
_cases = sorted((name, case) for name, case in globals().items()
                if name.startswith("test_") and callable(case))
assert len(_cases) == 18, "matching native authoring acceptance inventory changed"
for _name, _case in _cases:
    _case()
    print("matching authoring acceptance passed:", _name)
print("matching authoring native cases:", len(_cases))
