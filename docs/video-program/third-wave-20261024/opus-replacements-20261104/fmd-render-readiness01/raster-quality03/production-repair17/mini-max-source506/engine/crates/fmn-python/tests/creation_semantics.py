"""Compiled-extension acceptance for live creation and reveal choreography."""
import math
import numpy as np
import manimlib as m


class NonlinearReveal(m.ShowPartial):
    def get_bounds(self, alpha):
        self.samples.append(float(alpha))
        return (0., alpha * alpha)
    def __init__(self, mob, **kwargs):
        self.samples = []
        super().__init__(mob, rate_func=m.linear, **kwargs)


scene = m.Scene()
curve = m.Line((0., 0., 0.), (4., 0., 0.))
reveal = NonlinearReveal(curve, run_time=1., final_alpha_value=.5)
scene.play(reveal)
assert .5 in reveal.samples, reveal.samples
assert np.allclose(curve.get_end(), [1., 0., 0.]), curve.get_end()
assert not curve.is_changing()

# A rule can agree with the obsolete four probes and disagree elsewhere.
class BetweenProbes(m.ShowPartial):
    def get_bounds(self, alpha):
        return (0., alpha + 8 * math.prod(alpha - p for p in (.125, .375, .625, .875)))

probe_scene = m.Scene()
probe_curve = m.Line((0., 0., 0.), (4., 0., 0.))
probe_animation = BetweenProbes(probe_curve, rate_func=m.linear, final_alpha_value=.5)
probe_scene.play(probe_animation)
assert np.allclose(probe_curve.get_end(), [4 * probe_animation.get_bounds(.5)[1], 0., 0.])
assert not np.isclose(probe_curve.get_end()[0], 2.)

# A stock class with a later instance override must not use its native kind.
patched_scene = m.Scene()
patched_curve = m.Line((0., 0., 0.), (4., 0., 0.))
patched = m.ShowCreation(patched_curve, rate_func=m.linear)
patched.get_bounds = lambda alpha: (.25, .75)
patched_scene.play(patched)
assert np.allclose(patched_curve.get_start(), [1., 0., 0.])
assert np.allclose(patched_curve.get_end(), [3., 0., 0.])

# Nested lagged/sequential groups still run on the native rational clock.
nested_scene = m.Scene()
a = m.Line((0., 0., 0.), (4., 0., 0.))
b = m.Line((0., 1., 0.), (4., 1., 0.))
first = NonlinearReveal(a, run_time=.5, final_alpha_value=.5)
second = NonlinearReveal(b, run_time=.5, final_alpha_value=.75)
nested_scene.play(m.Succession(m.AnimationGroup(first), second))
assert first.samples and second.samples
assert np.allclose(a.get_end(), [1., 0., 0.])
assert np.allclose(b.get_end(), [2.25, 1., 0.])

# The surface path must call Surface's UV partial operation, not a vector
# reveal against incompatible structured records.
surface_scene = m.Scene()
surface = m.ParametricSurface(lambda u, v: (u, v, 0.), resolution=(3, 3))
surface_start = surface.copy()
expected_surface = surface.copy()
expected_surface.pointwise_become_partial(surface_start, .2, .8)
surface_reveal = m.ShowCreation(surface, rate_func=m.linear)
surface_reveal.get_bounds = lambda alpha: (.2, .8)
surface_scene.play(surface_reveal)
assert surface.data.dtype == expected_surface.data.dtype
assert np.allclose(surface.get_points(), expected_surface.get_points())

# Retained passing flashes restore reusable full geometry. They also have
# the public ShowPartial hierarchy, including already-published imports.
from manimlib.animation.indication import ShowPassingFlash as QualifiedPassing
assert QualifiedPassing is m.ShowPassingFlash
assert issubclass(m.ShowPassingFlash, m.ShowPartial)
passing_scene = m.Scene()
passing_curve = m.Line((0., 0., 0.), (4., 0., 0.))
passing_points = passing_curve.get_points().copy()
passing_scene.play(m.ShowPassingFlash(passing_curve, remover=False))
assert np.array_equal(passing_curve.get_points(), passing_points)
assert passing_curve in passing_scene.mobjects

uncreate_scene = m.Scene()
uncreate_curve = m.Line((0., 0., 0.), (4., 0., 0.))
uncreate_scene.play(m.Uncreate(uncreate_curve, remover=False))
assert uncreate_curve in uncreate_scene.mobjects
assert np.allclose(uncreate_curve.get_start(), uncreate_curve.get_end())

# A failure after native playback has opened must not strand suspension or
# contaminate the next native play with a still-active reveal lifecycle.
class RevealFailure(RuntimeError):
    pass
class FailingReveal(m.ShowPartial):
    def get_bounds(self, alpha):
        if alpha > 0:
            raise RevealFailure("live-reveal-failure")
        return (0., 1.)

failure_scene = m.Scene()
failure_curve = m.Line((0., 0., 0.), (4., 0., 0.))
failed = FailingReveal(failure_curve, suspend_mobject_updating=True)
try:
    failure_scene.play(m.AnimationGroup(failed))
except RevealFailure as error:
    assert str(error) == "live-reveal-failure"
else:
    raise AssertionError("authored reveal callback was silently bypassed")
assert not failure_curve._is_updating_suspended()
assert not failure_curve.is_changing()
failed.abort()
failure_scene.play(failure_curve.animate.shift((1., 0., 0.)))
assert np.isfinite(failure_curve.get_points()).all()

# DrawBorderThenFill must execute an authored outline and then fill from it
# back into the saved initial appearance, using native field interpolation.
class CustomOutline(m.DrawBorderThenFill):
    def get_outline(self):
        self.outline_calls = getattr(self, "outline_calls", 0) + 1
        return super().get_outline().set_stroke(width=8.)

border_scene = m.Scene()
border_square = m.Square(fill_opacity=.8, stroke_width=4.)
border_animation = CustomOutline(border_square, rate_func=m.linear, final_alpha_value=.75)
border_scene.play(border_animation)
assert border_animation.outline_calls == 1
assert np.isclose(border_square.get_fill_opacity(), .4)
assert np.isclose(border_square.get_stroke_width(), 6.)
assert not border_square.is_changing()

# Direct lifecycle use supports nonmonotonic interpolation; coming back
# from the fill phase must not leave fill opacity on the growing outline.
seek_square = m.Square(fill_opacity=.8, stroke_width=4.)
seek = m.DrawBorderThenFill(seek_square, rate_func=m.linear)
seek.begin()
seek.interpolate(.8)
assert seek_square.get_fill_opacity() > 0
seek.interpolate(.2)
assert seek_square.get_fill_opacity() == 0
assert np.isclose(seek_square.get_stroke_width(), 2.)
seek.finish()
assert np.isclose(seek_square.get_fill_opacity(), .8)
assert np.isclose(seek_square.get_stroke_width(), 4.)

# Native glyph families exercise point-free group roots and per-glyph lag.
class AuthoredWrite(m.Write):
    def get_outline(self):
        self.outline_calls = getattr(self, "outline_calls", 0) + 1
        return super().get_outline()

write_scene = m.Scene()
text = m.Text("Draw", font_size=24)
glyphs = list(text.family_members_with_points())
write = AuthoredWrite(text, stroke_width=6.)
write_scene.play(m.AnimationGroup(write))
assert write.outline_calls == 1
assert list(text.family_members_with_points()) == glyphs
assert all(np.isfinite(glyph.get_points()).all() for glyph in glyphs)
assert all(glyph.get_fill_opacity() > .9 for glyph in glyphs)

# Outline refinement uses Marionette's real alignment, not truncated zips.
aligned_scene = m.Scene()
aligned_curve = m.Line((0., 0., 0.), (4., 0., 0.))
class RefinedOutline(m.DrawBorderThenFill):
    def get_outline(self):
        return super().get_outline().insert_n_curves(3)

aligned = RefinedOutline(aligned_curve, rate_func=m.linear)
aligned_scene.play(aligned)
assert np.allclose(aligned_curve.get_start(), [0., 0., 0.])
assert np.allclose(aligned_curve.get_end(), [4., 0., 0.])
assert len(aligned_curve.get_points()) == len(aligned.outline.get_points())

class FailingOutline(m.DrawBorderThenFill):
    def get_outline(self):
        raise RevealFailure("outline-build-failure")

outline_failure_scene = m.Scene()
outline_failure_square = m.Square(fill_opacity=.5)
try:
    outline_failure_scene.play(FailingOutline(outline_failure_square, suspend_mobject_updating=True))
except RevealFailure as error:
    assert str(error) == "outline-build-failure"
else:
    raise AssertionError("authored get_outline was bypassed")
assert not outline_failure_square._is_updating_suspended()
assert not outline_failure_square.is_changing()
outline_failure_scene.play(m.Write(outline_failure_square))

# Authored object lifecycles participate even in a stock creation animation.
import tempfile as _creation_tempfile
import unittest as _creation_unittest
from pathlib import Path as _CreationPath
from types import MethodType as _CreationMethod


class CreationObjectHooks(_creation_unittest.TestCase):
    def test_partial_lifecycle_hooks_are_dispatched_without_admission_effects(self):
        for animation_class in (m.ShowCreation, m.Uncreate, m.ShowPassingFlash):
            for name in ("copy", "get_family", "set_animating_status",
                         "suspend_updating", "resume_updating", "update"):
                with self.subTest(animation=animation_class.__name__, hook=name):
                    calls = []
                    def hook(self, *args, _name=name, **kwargs):
                        calls.append(_name)
                        return getattr(super(Authored, self), _name)(*args, **kwargs)
                    Authored = type("Authored", (m.Line,), {name: hook})
                    source = Authored(m.LEFT, m.RIGHT)
                    animation = animation_class(source, suspend_mobject_updating=True)
                    calls.clear()
                    self.assertTrue(m._requires_python_animation(animation))
                    self.assertEqual(calls, [], "classification must not call authored hooks")
                    m.Scene().play(animation, run_time=.125, rate_func=m.linear)
                    self.assertIn(name, calls)
                    self.assertFalse(source._is_updating_suspended())

    def test_creation_uses_authored_starting_copy_in_every_frame(self):
        calls, samples = [], []
        class Authored(m.Line):
            def copy(self, *args, **kwargs):
                calls.append(self)
                return super().copy(*args, **kwargs).shift(m.UP)
        source, scene = Authored(m.ORIGIN, 3*m.RIGHT), m.Scene()
        recorder = m.Mobject()
        recorder.add_updater(lambda obj, dt: samples.append(
            (scene.time, source.get_start().copy(), source.get_end().copy())) if dt > 0 else None,
            call=False)
        scene.add(source, recorder)
        scene.play(m.ShowCreation(source), run_time=.125, rate_func=m.linear)
        self.assertEqual(calls, [source])
        self.assertEqual(len(samples), 4)
        for time, start, end in samples:
            np.testing.assert_allclose(start, m.UP, atol=1e-6)
            np.testing.assert_allclose(end, [3*min(time/.125, 1), 1, 0], atol=1e-6)

    def test_flash_and_uncreate_keep_removal_and_authored_copy_semantics(self):
        class Authored(m.Line):
            def copy(self, *args, **kwargs):
                return super().copy(*args, **kwargs).shift(m.UP)
        for cls in (m.Uncreate, m.ShowPassingFlash):
            with self.subTest(animation=cls.__name__):
                source, scene = Authored(m.LEFT, m.RIGHT), m.Scene()
                scene.add(source)
                scene.play(cls(source), run_time=.125)
                self.assertNotIn(source, scene.mobjects)
                self.assertFalse(source._is_updating_suspended())
                np.testing.assert_allclose(source.get_start(), m.LEFT+m.UP, atol=1e-6)
                # Uncreate finishes at reverse-eased zero (a collapsed path);
                # PassingFlash explicitly restores the complete starting copy.
                end = m.LEFT if cls is m.Uncreate else m.RIGHT
                np.testing.assert_allclose(source.get_end(), end+m.UP, atol=1e-6)

    def test_partial_family_override_controls_selected_child(self):
        class Authored(m.VGroup):
            def get_family(self, recurse=True):
                # The second child intentionally does not participate in reveals.
                return [self, self[0]] if recurse and len(self) else [self]
        first, second = m.Line(m.ORIGIN, 4*m.RIGHT), m.Line(m.UP, m.UP+4*m.RIGHT)
        source = Authored(first, second)
        second_before = second.get_points().copy()
        animation = m.ShowCreation(source, final_alpha_value=.5, lag_ratio=0, rate_func=m.linear)
        m.Scene().play(animation, run_time=.125)
        np.testing.assert_allclose(first.get_end(), 2*m.RIGHT, atol=1e-6)
        np.testing.assert_array_equal(second.get_points(), second_before)

    def test_deep_shared_object_hooks_are_found_without_traversing_authored_family(self):
        calls, partial_calls = [], []
        class Authored(m.Line):
            def get_family(self, recurse=True):
                calls.append(self)
                return super().get_family(recurse)
            def pointwise_become_partial(self, *args, **kwargs):
                partial_calls.append(self)
                return super().pointwise_become_partial(*args, **kwargs)
        child = Authored()
        root = m.VGroup(m.VGroup(child), m.VGroup(child))
        animation = m.ShowCreation(root)
        calls.clear()
        self.assertTrue(m._requires_python_animation(animation))
        self.assertEqual(calls, [])
        m.Scene().play(animation, run_time=.125)
        # The root's own family traversal is native and does not dispatch a
        # child's get_family. The selected child's actual reveal still must.
        self.assertEqual(calls, [])
        self.assertIn(child, partial_calls)
        self.assertIs(root[0][0], root[1][0])

    def test_copy_descriptor_is_not_run_by_reveal_admission(self):
        class Authored(m.Line):
            @property
            def copy(self):
                raise AssertionError("an admission probe ran the copy descriptor")
        source = Authored()
        for cls in (m.ShowCreation, m.Uncreate, m.ShowPassingFlash, m.DrawBorderThenFill, m.Write):
            with self.subTest(animation=cls.__name__):
                self.assertTrue(m._requires_python_animation(cls(source)))

    def test_family_descriptor_is_not_run_by_creation_or_writing_admission(self):
        class Authored(m.Square):
            @property
            def get_family(self):
                if getattr(self, "refuse_family", False):
                    raise AssertionError("an admission probe ran the family descriptor")
                return super().get_family
        for cls in (m.ShowCreation, m.Uncreate, m.ShowPassingFlash, m.DrawBorderThenFill, m.Write):
            source = Authored()
            animation = cls(source)
            source.refuse_family = True
            with self.subTest(animation=cls.__name__):
                self.assertTrue(m._requires_python_animation(animation))

    def test_animation_descriptors_are_not_run_during_admission(self):
        class Window(m.ShowCreation):
            @property
            def get_bounds(self):
                raise AssertionError("classification evaluated a reveal window")
        self.assertTrue(m._requires_python_animation(Window(m.Line())))
        class Subject(m.ShowCreation):
            @property
            def mobject(self):
                if getattr(self, "refuse_subject", False):
                    raise AssertionError("classification evaluated a subject descriptor")
                return self.subject
            @mobject.setter
            def mobject(self, value):
                self.subject = value
        animation = Subject(m.Line())
        animation.refuse_subject = True
        self.assertTrue(m._requires_python_animation(animation))

    def test_shared_animation_base_hooks_remain_live(self):
        original = m.Animation.begin
        calls = []
        def begin(animation):
            calls.append(animation.mobject)
            return original(animation)
        try:
            m.Animation.begin = begin
            for cls in (m.ShowCreation, m.Uncreate, m.ShowPassingFlash,
                        m.DrawBorderThenFill, m.Write, m.Transform):
                with self.subTest(animation=cls.__name__):
                    source = m.Line()
                    animation = cls(source, m.Line().shift(m.UP)) if cls is m.Transform else cls(source)
                    calls.clear()
                    self.assertTrue(m._requires_python_animation(animation))
                    self.assertEqual(calls, [])
                    m.Scene().play(animation, run_time=.125)
                    self.assertIn(source, calls)
        finally:
            m.Animation.begin = original

    def test_authored_easing_is_evaluated_on_the_actual_creation_clock(self):
        rates, samples = [], []
        class Authored(m.Line):
            def copy(self, *args, **kwargs):
                return super().copy(*args, **kwargs).shift(m.UP)
        def rate(alpha):
            rates.append(alpha)
            return alpha*alpha
        with _creation_tempfile.TemporaryDirectory() as tmp:
            scene = m.Scene()
            with scene.render_session(_CreationPath(tmp)/"rate.y4m", format="y4m",
                                      resolution=(32,20), fps=48):
                source = Authored(m.ORIGIN, 3*m.RIGHT)
                probe = m.Mobject()
                probe.add_updater(lambda obj, dt: samples.append((scene.time, source.get_end().copy()))
                                  if dt > 0 else None, call=False)
                scene.add(source, probe)
                scene.play(m.ShowCreation(source), run_time=.125, rate_func=rate)
            self.assertEqual(len(samples), 6)
            for time, endpoint in samples:
                alpha = min(time/.125, 1.)
                self.assertTrue(any(abs(value-alpha) < 1e-12 for value in rates))
                np.testing.assert_allclose(endpoint, [3*alpha*alpha, 1, 0], atol=1e-6)

    def test_instance_and_late_base_changes_are_observed(self):
        source = m.Line()
        animation = m.ShowCreation(source)
        self.assertFalse(m._requires_python_animation(animation))
        calls = []
        original = source.copy
        source.copy = _CreationMethod(lambda self: (calls.append(self), original())[1], source)
        self.assertTrue(m._requires_python_animation(animation))
        m.Scene().play(animation, run_time=.125)
        self.assertEqual(calls, [source])
        original_base = m.Mobject.copy
        try:
            m.Mobject.copy = lambda self, *args, **kwargs: original_base(self, *args, **kwargs)
            self.assertTrue(m._requires_python_animation(m.ShowCreation(m.Square())))
        finally:
            m.Mobject.copy = original_base

    def test_stock_reveals_keep_native_admission(self):
        class Plain(m.Square):
            pass
        for cls in (m.ShowCreation, m.Uncreate, m.ShowPassingFlash):
            for source in (m.Line(), m.Square(), Plain(), m.VGroup(m.Line(), m.Circle())):
                with self.subTest(animation=cls.__name__, source=type(source).__name__):
                    self.assertFalse(m._requires_python_animation(cls(source)))

    def test_copy_failure_releases_reveal_transients_and_scene_can_continue(self):
        error = RuntimeError("authored creation snapshot failed")
        class Authored(m.Line):
            fail = True
            def copy(self, *args, **kwargs):
                if self.fail:
                    raise error
                return super().copy(*args, **kwargs)
        for cls in (m.ShowCreation, m.Uncreate, m.ShowPassingFlash, m.DrawBorderThenFill, m.Write):
            with self.subTest(animation=cls.__name__):
                source, scene = Authored(), m.Scene()
                scene.add(source)
                before = source.get_points().copy()
                with self.assertRaises(RuntimeError) as caught:
                    scene.play(cls(source, suspend_mobject_updating=True), run_time=.125)
                self.assertIs(caught.exception, error)
                self.assertFalse(source.is_changing())
                self.assertFalse(source._is_updating_suspended())
                np.testing.assert_array_equal(source.get_points(), before)
                source.fail = False
                scene.play(m.ShowCreation(source), run_time=.125)

    def test_nested_reveal_snapshots_follow_the_preceding_transform(self):
        calls = []
        class Authored(m.Line):
            def copy(self, *args, **kwargs):
                calls.append(self.get_start().copy())
                return super().copy(*args, **kwargs)
        source = Authored(m.ORIGIN, m.RIGHT)
        scene = m.Scene().add(source)
        animation = m.Succession(
            m.Transform(source, m.Line(2*m.RIGHT, 3*m.RIGHT), run_time=.125),
            m.ShowCreation(source, run_time=.125))
        scene.play(m.AnimationGroup(animation), rate_func=m.linear)
        np.testing.assert_allclose(calls[-1], 2*m.RIGHT, atol=1e-6)
        np.testing.assert_allclose(source.get_end(), 3*m.RIGHT, atol=1e-6)

    def test_border_and_write_honor_authored_style_and_alignment_hooks(self):
        for cls in (m.DrawBorderThenFill, m.Write):
            calls = []
            class Authored(m.Square):
                def align_data_and_family(self, other):
                    calls.append("align")
                    return super().align_data_and_family(other)
                def refresh_joint_angles(self):
                    calls.append("finish")
                    return super().refresh_joint_angles()
                def set_stroke(self, *args, **kwargs):
                    calls.append("stroke")
                    return super().set_stroke(*args, **kwargs)
            source = Authored(fill_opacity=.75)
            animation = cls(source)
            calls.clear()
            self.assertTrue(m._requires_python_animation(animation))
            self.assertEqual(calls, [])
            m.Scene().play(animation, run_time=.125)
            self.assertIn("align", calls)
            self.assertIn("finish", calls)
            self.assertIn("stroke", calls)
            self.assertAlmostEqual(source.get_fill_opacity(), .75)

    def test_actual_reveal_frames_match_independent_native_controls(self):
        class Authored(m.Line):
            def copy(self, *args, **kwargs):
                return super().copy(*args, **kwargs).shift(m.UP)
        def render(path, custom, threads, displaced=False):
            scene = m.Scene()
            with scene.render_session(path, format="y4m", resolution=(64,40), fps=24,
                                      threads=threads) as output:
                source = (Authored if custom else m.Line)(-2*m.RIGHT, 2*m.RIGHT)
                if displaced:
                    source.shift(m.UP)
                scene.add(source)
                scene.play(m.ShowCreation(source), run_time=.125, rate_func=m.linear)
            self.assertEqual(output.result.frame_count, 3)
            return path.read_bytes()
        with _creation_tempfile.TemporaryDirectory() as tmp:
            root = _CreationPath(tmp)
            expected = render(root/"expected.y4m", False, 1, displaced=True)
            for threads in (1,4,16):
                self.assertEqual(render(root/f"actual-{threads}.y4m", True, threads), expected)
            self.assertNotEqual(render(root/"wrong.y4m", False, 1), expected)
            payloads = expected.split(b"FRAME\n")[1:]
            self.assertEqual(len(payloads), 3)
            self.assertNotEqual(payloads[0], payloads[1])


_creation_result = _creation_unittest.TextTestRunner(verbosity=2).run(
    _creation_unittest.defaultTestLoader.loadTestsFromTestCase(CreationObjectHooks))
if not _creation_result.wasSuccessful():
    raise AssertionError("creation object lifecycle acceptance failed")
