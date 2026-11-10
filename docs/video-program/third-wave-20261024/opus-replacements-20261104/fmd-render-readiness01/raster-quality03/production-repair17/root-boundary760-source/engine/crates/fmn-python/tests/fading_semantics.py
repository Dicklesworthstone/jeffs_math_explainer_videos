"""Real installed-wheel acceptance; no substitute geometry or scene drivers."""
import numpy as np
from manimlib import (
    Scene, Square, Circle, Rectangle, VGroup, Text, FadeTransform,
    FadeTransformPieces, Transform, AnimationGroup, Succession, linear, RIGHT,
)


def near(actual, expected):
    np.testing.assert_allclose(actual, expected, atol=2e-5, rtol=2e-5)


def grouped_midpoint():
    source = Square(side_length=2, fill_opacity=1, stroke_width=0)
    target = Rectangle(width=4, height=2, fill_opacity=1, stroke_width=0).shift(4*RIGHT)
    animation = FadeTransform(source, target, rate_func=linear)
    assert isinstance(animation, Transform)
    assert animation.mobject[0] is source and animation.mobject[1] is not target
    animation.begin(); animation.interpolate(.5)
    near(animation.mobject[0].get_center(), [2, 0, 0])
    near(animation.mobject[1].get_center(), [2, 0, 0])
    near(animation.mobject[0].get_width(), 3)
    near(animation.mobject[0].get_fill_opacity(), .5)
    near(animation.mobject[1].get_fill_opacity(), .5)
    animation.abort()


def native_play_cleanup():
    scene = Scene()
    source, target = Square(fill_opacity=1), Circle(fill_opacity=1).shift(3*RIGHT)
    scene.add(source)
    before = source.get_points().copy()
    scene.play(FadeTransform(source, target), run_time=.125)
    assert target in scene.mobjects and source not in scene.mobjects
    near(source.get_points(), before)
    assert len(scene.mobjects) == 1


def ghost_hook_and_nested_timing():
    calls = []
    class Custom(FadeTransform):
        def ghost_to(self, source, target):
            calls.append((self.stretch, self.dim_to_match))
            super().ghost_to(source, target)
    scene = Scene()
    a, b, c, d = Square(), Circle().shift(RIGHT), Square().shift(-3*RIGHT), Circle().shift(-RIGHT)
    scene.add(a, c)
    first = Custom(a, b, stretch=False, dim_to_match=0, run_time=.125)
    second = FadeTransform(c, d, run_time=.125)
    scene.play(Succession(first, AnimationGroup(second)))
    assert calls == [(False,0),(False,0)]
    assert b in scene.mobjects and d in scene.mobjects


def authored_path_and_zero_rate():
    samples = []
    def path(a,b,t):
        samples.append(t)
        return (1-t)*a+t*b + np.array([0., 4*t*(1-t), 0.])
    source = Square(side_length=2, fill_opacity=1)
    target = Square(side_length=2, fill_opacity=1).shift(4*RIGHT)
    animation = FadeTransform(source, target, path_func=path, rate_func=lambda a:.5)
    animation.begin()
    near(source.get_center(), [2,1,0])
    near(source.get_fill_opacity(), .5)
    assert samples and all(value == .5 for value in samples)
    animation.abort()


def unequal_piece_families():
    source = VGroup(Square().shift(-RIGHT), Circle().shift(RIGHT))
    target = VGroup(Circle().shift(-2*RIGHT), Square(), Circle().shift(2*RIGHT))
    scene = Scene(); scene.add(source)
    before = [part.get_points().copy() for part in source]
    animation = FadeTransformPieces(source, target, stretch=False, dim_to_match=0)
    scene.play(animation, run_time=.125)
    assert target in scene.mobjects and source not in scene.mobjects
    assert len(source) == 2
    for part, points in zip(source,before):
        near(part.get_points(),points)


def text_crossfade():
    scene = Scene()
    source, target = Text("a+b"), Text("x+y=z").shift(RIGHT)
    scene.add(source)
    scene.play(FadeTransformPieces(source,target),run_time=.125)
    assert target in scene.mobjects and source not in scene.mobjects
    assert source.get_string() == "a+b" and target.get_string() == "x+y=z"


def failure_releases_suspension():
    failure = RuntimeError("authored ghost failure")
    class Broken(FadeTransform):
        def ghost_to(self, source, target):
            raise failure
    source, target = Square(), Circle()
    animation = Broken(source,target,suspend_mobject_updating=True)
    scene = Scene(); scene.add(source)
    try:
        scene.play(animation,run_time=.125)
    except RuntimeError as error:
        assert error is failure
    else:
        raise AssertionError("ghost callback was bypassed")
    assert not animation.mobject._is_updating_suspended()
    assert not source._is_updating_suspended()
    assert target not in scene.mobjects


def remover_and_saved_identity():
    scene = Scene()
    source, target = Square(), Circle()
    before = source.get_points().copy()
    animation = FadeTransform(source,target,remover=True)
    source.shift(2*RIGHT).save_state()
    saved = source.saved_state
    scene.play(animation,run_time=.125)
    assert source.saved_state is saved
    near(source.get_points(),before)
    assert target not in scene.mobjects and source not in scene.mobjects


for case in (grouped_midpoint, native_play_cleanup, ghost_hook_and_nested_timing,
             authored_path_and_zero_rate, unequal_piece_families, text_crossfade,
             failure_releases_suspension, remover_and_saved_identity):
    case()


from manimlib import VFadeIn, VFadeOut, VFadeInThenOut, FadeIn, FadeOut


def vector_only_changes_opacity():
    source = Square(fill_opacity=.8, stroke_opacity=.6)
    animation = VFadeIn(source, rate_func=linear)
    animation.begin()
    source.shift(2*RIGHT).set_stroke(width=7)
    points = source.get_points().copy()
    animation.interpolate(.25)
    near(source.get_points(), points)
    near(source.get_fill_opacity(), .2)
    near(source.get_stroke_opacity(), .15)
    near(source.get_stroke_width(), 7)
    animation.abort()


def authored_vector_lag_hook_runs():
    calls = []
    class Custom(VFadeIn):
        def get_sub_alpha(self, alpha, index, count):
            calls.append((alpha,index,count))
            return .25
    scene = Scene()
    source = Square(fill_opacity=.8)
    scene.add(source)
    scene.play(Custom(source), run_time=.125)
    assert calls
    near(source.get_fill_opacity(), .2)


def nested_vector_out_restores_configured_endpoint():
    class Custom(VFadeOut):
        def interpolate_submobject(self, current, start, alpha):
            super().interpolate_submobject(current, start, alpha)
    assert issubclass(VFadeOut,VFadeIn)
    source = Square(fill_opacity=.8)
    scene = Scene(); scene.add(source)
    before = source.get_points().copy()
    scene.play(AnimationGroup(Custom(source,remover=False,final_alpha_value=.5,rate_func=linear)),run_time=.125)
    near(source.get_fill_opacity(), .4)
    near(source.get_points(), before)


def geometric_fadeout_retains_partial_endpoint():
    source = Square(fill_opacity=.8)
    scene = Scene(); scene.add(source)
    scene.play(FadeOut(source,shift=2*RIGHT,remover=False,final_alpha_value=.5,rate_func=linear),run_time=.125)
    assert source in scene.mobjects
    near(source.get_center(), RIGHT)
    near(source.get_fill_opacity(), .4)


def geometric_fadein_remover_uses_shared_cleanup():
    source = Square(fill_opacity=.8)
    scene = Scene()
    scene.play(FadeIn(source,remover=True,final_alpha_value=.5,rate_func=linear),run_time=.125)
    assert source not in scene.mobjects
    near(source.get_fill_opacity(), .4)


def vector_mobject_getter_is_not_bypassed():
    class CustomSquare(Square):
        def get_fill_opacity(self):
            return .4
    source = CustomSquare(fill_opacity=1)
    scene = Scene(); scene.add(source)
    scene.play(VFadeIn(source,final_alpha_value=.5,rate_func=linear),run_time=.125)
    near(source.data["fill_rgba"][:,3], .2)


def vector_callback_failure_unwinds():
    failure = RuntimeError("vector callback failure")
    class Broken(VFadeIn):
        def interpolate(self, alpha):
            if alpha > 0:
                raise failure
            super().interpolate(alpha)
    source = Square()
    scene = Scene(); scene.add(source)
    animation = Broken(source,suspend_mobject_updating=True)
    try:
        scene.play(AnimationGroup(animation),run_time=.125)
    except RuntimeError as error:
        assert error is failure
    else:
        raise AssertionError("vector callback was bypassed")
    assert not source._is_updating_suspended()
    animation.abort()


def presuspended_vector_child_is_not_resumed():
    child = Square()
    child.suspend_updating()
    group = VGroup(child, Circle())
    animation = VFadeIn(group,suspend_mobject_updating=True)
    animation.begin(); animation.finish()
    assert child._is_updating_suspended()
    assert not group._is_updating_suspended()
    assert not group[1]._is_updating_suspended()


for case in (vector_only_changes_opacity, authored_vector_lag_hook_runs,
             nested_vector_out_restores_configured_endpoint, geometric_fadeout_retains_partial_endpoint,
             geometric_fadein_remover_uses_shared_cleanup, vector_mobject_getter_is_not_bypassed,
             vector_callback_failure_unwinds, presuspended_vector_child_is_not_resumed):
    case()
