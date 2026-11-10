"""Curved-arrow initialization on real Marionette records and native geometry."""
import copy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import numpy as np
import manimlib as m
from manimlib.mobject.geometry import (
    ArcBetweenPoints as QualifiedBetween,
    CurvedArrow as QualifiedCurved,
    CurvedDoubleArrow as QualifiedDouble,
)


class ReferenceCurve(m.Arc):
    """Pinned 6199a00d public tip protocol over independent native Arc data.

    geometry.py:TipableVMobject.reset_endpoints_based_on_tip and
    mobject.py:1299/get_start_and_end. This control never constructs a
    CurvedArrow and never calls the changed reset or endpoint-fit methods.
    """
    def get_start_and_end(self):
        p = self.get_points()
        return p[0].copy(), p[-1].copy()

    def put_start_and_end_on(self, start, end):
        import math
        start, end = np.array(start, dtype=float), np.array(end, dtype=float)
        a, b = (np.array(p, dtype=float) for p in self.get_start_and_end())
        before, after = b - a, end - start
        self.scale(np.linalg.norm(after) / np.linalg.norm(before), about_point=a)
        self.rotate(math.atan2(after[1], after[0]) - math.atan2(before[1], before[0]))
        self.rotate(math.atan2(before[2], np.linalg.norm(before[:2]))
                    - math.atan2(after[2], np.linalg.norm(after[:2])),
                    axis=np.array([-after[1], after[0], 0.]))
        return self.shift(start - self.get_start())

    def reset_endpoints_based_on_tip(self, tip, at_start):
        if self.get_length() != 0:
            self.put_start_and_end_on(tip.get_base() if at_start else self.get_start(),
                                      self.get_end() if at_start else tip.get_base())
        return self


def reference_curve(start, end, angle=m.PI / 2, double=False, **kwargs):
    obj = ReferenceCurve(angle=angle, **kwargs)
    if angle == 0:
        obj.set_points_as_corners([m.LEFT, m.RIGHT])
    obj.put_start_and_end_on(start, end)
    obj.add_tip()
    if double:
        obj.add_tip(at_start=True)
    return obj


class NativeCurvedLifecycle(unittest.TestCase):
    def assert_points(self, actual, expected):
        np.testing.assert_allclose(actual, expected, rtol=2e-5, atol=2e-5)

    def test_all_initialization_phases_precede_fitting_and_tips(self):
        for base, tips in ((m.ArcBetweenPoints, 0), (m.CurvedArrow, 1),
                           (m.CurvedDoubleArrow, 2)):
            with self.subTest(base=base.__name__):
                events = []
                class Authored(base):
                    def init_data(self):
                        events.append("data")
                        super().init_data()
                    def init_points(self):
                        events.append("points")
                        self.recipe = (self.radius, self.start_angle, self.arc_center.copy())
                        super().init_points()
                    def init_uniforms(self):
                        events.append("uniforms")
                        super().init_uniforms()
                        self.uniforms["custom_uniform"] = 17
                    def init_colors(self):
                        events.append("colors")
                        super().init_colors()
                    def put_start_and_end_on(self, *args, **kwargs):
                        events.append("fit")
                        return super().put_start_and_end_on(*args, **kwargs)
                    def add_tip(self, *args, **kwargs):
                        events.append("tip")
                        return super().add_tip(*args, **kwargs)
                obj = Authored(2*m.LEFT, 3*m.RIGHT, radius=2, start_angle=.3,
                               arc_center=m.UP, n_components=12)
                # Tip fitting may call put_start_and_end_on again. The initial
                # fit must nevertheless follow colors and precede the first tip.
                self.assertEqual(events[:5], ["data", "points", "uniforms", "colors", "fit"])
                self.assertEqual(events.count("points"), 1)
                self.assertEqual(events.count("tip"), tips)
                self.assertEqual(obj.recipe[:2], (2.0, .3))
                self.assert_points(obj.recipe[2], m.UP)
                self.assertEqual(obj.uniforms["custom_uniform"], 17)
                self.assert_points(obj.get_start(), 2*m.LEFT)
                expected = (reference_curve(2*m.LEFT, 3*m.RIGHT, double=tips == 2,
                            radius=2, start_angle=.3, arc_center=m.UP, n_components=12)
                            if tips else None)
                self.assert_points(obj.get_end(), expected.get_end() if tips else 3*m.RIGHT)

    def test_qualified_exports_keep_the_original_class_objects(self):
        self.assertIs(QualifiedBetween, m.ArcBetweenPoints)
        self.assertIs(QualifiedCurved, m.CurvedArrow)
        self.assertIs(QualifiedDouble, m.CurvedDoubleArrow)
        self.assertIs(m.CurvedDoubleArrow.__bases__[0], m.CurvedArrow)

    def test_authored_point_run_is_fitted_instead_of_replaced(self):
        points = np.array([[-1.,0.,0.], [-.5,2.,0.], [0.,1.,0.],
                           [.5,2.,0.], [1.,0.,0.]])
        class Authored(m.ArcBetweenPoints):
            def init_points(self):
                self.set_points(points)
        obj = Authored(m.DOWN, 3*m.UP, angle=.8)
        expected = m.VMobject().set_points(points)
        expected.put_start_and_end_on(m.DOWN, 3*m.UP)
        self.assertEqual(obj.get_num_points(), 5)
        self.assert_points(obj.get_points(), expected.get_points())

    def test_zero_angle_uses_the_reference_line_before_tip_creation(self):
        for cls, count in ((m.ArcBetweenPoints, 0), (m.CurvedArrow, 1),
                           (m.CurvedDoubleArrow, 2)):
            with self.subTest(cls=cls.__name__):
                obj = cls(2*m.LEFT, 2*m.RIGHT, angle=0)
                self.assert_points(obj.get_start(), 2*m.LEFT)
                expected_end = (reference_curve(2*m.LEFT, 2*m.RIGHT, angle=0, double=True).get_end()
                                if count == 2 else 2*m.RIGHT)
                self.assert_points(obj.get_end(), expected_end)
                self.assertEqual(len(obj.get_tips()), count)
                self.assert_points(obj.get_points()[:, 1], 0)

    def test_custom_dtype_and_hook_owned_children_survive(self):
        class Authored(m.CurvedDoubleArrow):
            data_dtype = m.VMobject.data_dtype + [("weight", 1)]
            def init_data(self):
                self.resize(1)
                self.set_field("weight", 0, [.75])
                self.marker = m.VectorizedPoint(m.ORIGIN)
                self.add(self.marker)
        obj = Authored(2*m.LEFT, 2*m.RIGHT, angle=.8)
        self.assertIn("weight", obj.data.dtype.names)
        self.assert_points(obj.data["weight"], .75)
        self.assertIs(obj[0], obj.marker)
        self.assertEqual(len(obj.get_tips()), 2)
        self.assertIn(obj.tip, obj.submobjects)
        self.assertIn(obj.start_tip, obj.submobjects)
        self.assertIsNot(obj.tip, obj.start_tip)

    def test_cooperative_mixin_and_later_base_patch_are_observed(self):
        seen = []
        class Mixin:
            def init_points(self):
                seen.append("mixin")
                super().init_points()
        class Authored(Mixin, m.CurvedArrow):
            pass
        original = m.Arc.__init__
        def patched(obj, *args, **kwargs):
            seen.append("arc")
            original(obj, *args, **kwargs)
        m.Arc.__init__ = patched
        try:
            obj = Authored(m.LEFT, m.RIGHT)
        finally:
            m.Arc.__init__ = original
        self.assertEqual(seen, ["arc", "mixin"])
        self.assert_points(obj.get_end(), m.RIGHT)

    def test_authored_tip_factory_is_used_for_both_ends(self):
        calls = []
        class Authored(m.CurvedDoubleArrow):
            def get_unpositioned_tip(self, **kwargs):
                calls.append("tip")
                tip = super().get_unpositioned_tip(**kwargs)
                tip.custom_tag = len(calls)
                return tip
        obj = Authored(2*m.LEFT, 2*m.RIGHT, color=m.BLUE)
        self.assertEqual(calls, ["tip", "tip"])
        self.assertEqual((obj.tip.custom_tag, obj.start_tip.custom_tag), (1, 2))
        self.assertEqual(obj.tip.get_fill_color(), m.BLUE)
        self.assertEqual(obj.start_tip.get_fill_color(), m.BLUE)

    def test_hook_failure_stops_before_fitting_or_publishing_tips(self):
        for phase in ("init_data", "init_points", "init_uniforms", "init_colors"):
            with self.subTest(phase=phase):
                seen, error = [], RuntimeError("authored " + phase)
                def fail(obj):
                    seen.append(phase)
                    raise error
                class Authored(m.CurvedDoubleArrow):
                    def put_start_and_end_on(self, *args, **kwargs): seen.append("fit")
                    def add_tip(self, **kwargs): seen.append("tip")
                setattr(Authored, phase, fail)
                with self.assertRaises(RuntimeError) as caught:
                    Authored(m.LEFT, m.RIGHT)
                self.assertIs(caught.exception, error)
                self.assertEqual(seen, [phase])

    def test_copies_preserve_tip_aliases_without_reinitializing(self):
        calls = []
        class Authored(m.CurvedDoubleArrow):
            def init_points(self):
                calls.append("points")
                super().init_points()
        obj = Authored(2*m.LEFT, 2*m.RIGHT)
        original = obj.get_points().copy()
        for clone in (obj.copy(), copy.deepcopy(obj)):
            self.assertIs(type(clone), Authored)
            self.assertIsNot(clone.tip, obj.tip)
            self.assertIsNot(clone.start_tip, obj.start_tip)
            self.assertIn(clone.tip, clone.submobjects)
            self.assertIn(clone.start_tip, clone.submobjects)
            clone.shift(m.UP)
        self.assertEqual(calls, ["points"])
        np.testing.assert_array_equal(obj.get_points(), original)

    def test_public_construction_matches_independent_native_arc_and_tip_composition(self):
        for angle in (-.8, 0, .8):
            obj = m.CurvedDoubleArrow(2*m.LEFT, 2*m.RIGHT, angle=angle, color=m.BLUE)
            # Do not use any of the three constructors under test as oracle.
            expected = ReferenceCurve(angle=angle, color=m.BLUE)
            if angle == 0:
                expected.set_points_as_corners([m.LEFT, m.RIGHT])
            expected.put_start_and_end_on(2*m.LEFT, 2*m.RIGHT)
            expected.add_tip()
            expected.add_tip(at_start=True)
            for actual, reference in zip(obj.get_family(), expected.get_family()):
                np.testing.assert_array_equal(actual.data, reference.data)
            self.assertEqual(len(obj.get_family()), len(expected.get_family()))

    def test_default_shapes_match_reference_tip_accommodation(self):
        # Pinned geometry.py/Mobject public fitting sequence, not an image
        # golden generated from this implementation. These measurements also
        # distinguish the old true-arclength trimming (-0.4142) from fitting.
        single = m.CurvedArrow(m.LEFT, m.RIGHT)
        self.assert_points(single.get_bounding_box()[0], [-1, -0.52151060, 0])
        self.assert_points(single.get_arc_center(), [0, 0.75251263, 0])
        double = m.CurvedDoubleArrow(m.LEFT, m.RIGHT)
        self.assert_points(double.tip.get_vertices(), [
            [1.1685964, .31602028, 0], [.8419899, .08429817, 0],
            [1.1580101, -.08429817, 0],
        ])
        self.assert_points(double.start_tip.get_tip_point(), m.LEFT)
        # The Reference moves the already attached end tip when fitting the
        # second tip. Do not pin the old, incorrect end-at-RIGHT assertion.
        self.assertGreater(double.get_end()[0], 1.1)

    def test_angle_getters_measure_through_the_tip_aware_endpoints(self):
        # geometry.py Arc.get_start_angle/get_stop_angle (pinned 6199a00d):
        # angle_of_vector(self.get_start()/get_end() - self.get_arc_center()) % TAU,
        # through TipableVMobject's tip-aware accessors (fm-8pj7). The controls are
        # the Reference class sweep's own measurements of CurvedArrow(LEFT, RIGHT)
        # and CurvedDoubleArrow(LEFT, RIGHT) (reference_classes.v1.ndjson, 1e-3
        # quanta): its get_start_angle getter and the polar angle of its end tip's
        # first point about its measured arc center.
        import math

        def polar(point, center):
            return math.atan2(point[1] - center[1], point[0] - center[0]) % math.tau

        tol = 2e-3
        single = m.CurvedArrow(m.LEFT, m.RIGHT)
        double = m.CurvedDoubleArrow(m.LEFT, m.RIGHT)
        cases = (
            (single.get_start_angle(), 3.787),
            (single.get_stop_angle(), polar((1.0, 0.0), (0.0, 0.753))),
            (double.get_start_angle(), 3.805),
            (double.get_stop_angle(), polar((1.169, 0.316), (-0.035, 0.755))),
        )
        for actual, reference in cases:
            self.assertAlmostEqual(actual, reference, delta=tol)
        # Planted negative: the shaft's own anchors (the measurement this replaced)
        # sit outside the tolerance, so the checks above tell the two apart.
        center = double.get_arc_center()
        self.assertGreater(abs(polar(m.VMobject.get_start(double), center) - 3.805), 100 * tol)
        center = single.get_arc_center()
        self.assertGreater(
            abs(polar(m.VMobject.get_end(single), center) - polar((1.0, 0.0), (0.0, 0.753))),
            100 * tol)

        # Dispatch is through the public, overridable accessor.
        class Pinned(m.CurvedArrow):
            def get_start(self):
                return self.get_arc_center() + m.UP

        self.assertAlmostEqual(Pinned(m.LEFT, m.RIGHT).get_start_angle(), math.pi / 2, places=9)

        # An untipped arc reads its first and last anchors, in [0, TAU).
        arc = m.Arc(start_angle=-1.0, angle=2.0)
        center = arc.get_arc_center()
        self.assertAlmostEqual(arc.get_start_angle(), polar(arc.get_points()[0], center), places=12)
        self.assertAlmostEqual(arc.get_start_angle(), math.tau - 1.0, places=6)
        self.assertAlmostEqual(arc.get_stop_angle(), polar(arc.get_points()[-1], center), places=12)

    def test_endpoint_pair_is_owned_shaft_data_not_tip_positions(self):
        for cls in (m.CurvedArrow, m.CurvedDoubleArrow):
            with self.subTest(cls=cls.__name__):
                obj = cls(m.LEFT, m.RIGHT)
                points = obj.get_points().copy()
                start, end = obj.get_start_and_end()
                np.testing.assert_array_equal([start, end], points[[0, -1]])
                self.assertGreater(np.linalg.norm(end - obj.get_end()), .1)
                start[:] = 99
                end[:] = -99
                np.testing.assert_array_equal(obj.get_points(), points)
                self.assertAlmostEqual(obj.get_length(), np.linalg.norm(points[-1] - points[0]), places=5)

    def test_fitting_keeps_control_point_count_across_angles_and_tip_shapes(self):
        for cls, double in ((m.CurvedArrow, False), (m.CurvedDoubleArrow, True)):
            for angle in (-2.4, -.3, 0, .3, 2.4):
                with self.subTest(cls=cls.__name__, angle=angle):
                    options = dict(angle=angle, n_components=11, color=m.YELLOW)
                    tip_style = dict(cls.tip_config, length=.23, width=.17)
                    with patch.object(cls, "tip_config", tip_style), patch.object(
                            ReferenceCurve, "tip_config", tip_style):
                        obj = cls([-2, -.2, 0], [1.7, .8, .5], **options)
                        expected = reference_curve([-2, -.2, 0], [1.7, .8, .5],
                                                    double=double, **options)
                    self.assertEqual(obj.get_num_points(), 3 if angle == 0 else 23)
                    self.assertEqual(len(obj.get_family()), len(expected.get_family()))
                    for actual, control in zip(obj.get_family(), expected.get_family()):
                        self.assert_points(actual.get_points(), control.get_points())

    def test_scene_bound_refitting_matches_detached_and_reference_protocol(self):
        for cls, double in ((m.CurvedArrow, False), (m.CurvedDoubleArrow, True)):
            for target in (([-2, .3, .4], [1, 1.7, -.2]), ([0, -1, 0], [0, 2, 1])):
                with self.subTest(cls=cls.__name__, target=target):
                    detached = cls(m.LEFT, m.RIGHT, color=m.BLUE)
                    bound = cls(m.LEFT, m.RIGHT, color=m.BLUE)
                    control = reference_curve(m.LEFT, m.RIGHT, double=double, color=m.BLUE)
                    scene = m.Scene()
                    scene.add(bound)
                    family = tuple(bound.get_family())
                    views = [member.get_points() for member in family]
                    for obj in (detached, bound, control):
                        self.assertIs(obj.put_start_and_end_on(*target), obj)
                    self.assertEqual(tuple(bound.get_family()), family)
                    for actual, unbound, expected, view in zip(
                            bound.get_family(), detached.get_family(), control.get_family(), views):
                        self.assert_points(actual.get_points(), unbound.get_points())
                        self.assert_points(actual.get_points(), expected.get_points())
                        self.assert_points(view, actual.get_points())
                        self.assertEqual(actual.get_fill_color(), expected.get_fill_color())

    def test_authored_fitting_hooks_execute_for_owned_curved_arrows(self):
        calls = []
        class Authored(m.CurvedDoubleArrow):
            def scale(self, *args, **kwargs):
                calls.append("scale")
                return super().scale(*args, **kwargs)
            def rotate(self, *args, **kwargs):
                calls.append("rotate")
                return super().rotate(*args, **kwargs)
            def shift(self, *args, **kwargs):
                calls.append("shift")
                return super().shift(*args, **kwargs)
        obj = Authored(m.LEFT, m.RIGHT)
        scene = m.Scene()
        scene.add(obj)
        calls.clear()
        obj.put_start_and_end_on(2*m.LEFT, 3*m.UP)
        self.assertEqual(calls, ["scale", "rotate", "rotate", "shift"])

    def test_refitting_preserves_custom_record_lanes_and_decorations(self):
        class Authored(m.CurvedArrow):
            data_dtype = m.VMobject.data_dtype + [("weight", 1)]
            def init_data(self):
                self.resize(1)
                self.set_field("weight", 0, [.375])
                self.marker = m.Square(side_length=.2).shift(m.UP)
                self.add(self.marker)
        obj = Authored(m.LEFT, m.RIGHT)
        obj.data["weight"][:] = np.arange(len(obj.data))[:, None]
        weights = obj.data["weight"].copy()
        scene = m.Scene()
        scene.add(obj)
        marker, tip = obj.marker, obj.tip
        points = marker.get_points().copy()
        obj.put_start_and_end_on(2*m.LEFT, 3*m.UP)
        self.assertIs(obj.marker, marker)
        self.assertIs(obj.tip, tip)
        self.assertIn(marker, obj.submobjects)
        self.assertFalse(np.array_equal(marker.get_points(), points))
        np.testing.assert_array_equal(obj.data["weight"], weights)

    def test_invalid_endpoints_refuse_before_any_family_mutation(self):
        for bound in (False, True):
            obj = m.CurvedDoubleArrow(m.LEFT, m.RIGHT)
            scene = m.Scene()
            if bound:
                scene.add(obj)
            family = tuple(obj.get_family())
            snapshots = [member.data.copy() for member in family]
            for invalid in ([float("nan"), 0, 0], [0, float("inf"), 0], [1e39, 0, 0], [1]):
                with self.subTest(bound=bound, invalid=invalid):
                    with self.assertRaises((ValueError, TypeError, IndexError)):
                        obj.put_start_and_end_on(m.LEFT, invalid)
                    self.assertEqual(tuple(obj.get_family()), family)
                    for member, data in zip(family, snapshots):
                        np.testing.assert_array_equal(member.data, data)

    def test_existing_line_and_ordinary_arc_trimming_is_unchanged(self):
        line = m.Line(2*m.LEFT, 2*m.RIGHT, buff=0)
        line.add_tip()
        self.assert_points(line.get_start_and_end(), [2*m.LEFT, 2*m.RIGHT])
        self.assertAlmostEqual(line.get_length(), 4)
        arc = m.ArcBetweenPoints(2*m.LEFT, 2*m.RIGHT, angle=m.PI)
        length = arc.get_arc_length()
        arc.add_tip()
        self.assertLess(arc.get_arc_length(), length)
        self.assertNotIn("get_start_and_end", m.ArcBetweenPoints.__dict__)

    def test_refit_and_tip_removal_during_native_animation_match_control_frames(self):
        directory = Path(tempfile.mkdtemp(prefix="fmn-curved-refit-"))
        def render(name, threads, control):
            scene = m.Scene()
            destination = directory / name
            with scene.render_session(destination, format="png_sequence", fps=4,
                                      resolution=(96,54), threads=threads):
                obj = (reference_curve(m.LEFT, m.RIGHT, double=True, color=m.BLUE)
                       if control else m.CurvedDoubleArrow(m.LEFT, m.RIGHT, color=m.BLUE))
                scene.add(obj)
                scene.play(m.ApplyMethod(obj.put_start_and_end_on, 2*m.LEFT, 2*m.UP,
                                         rate_func=m.linear), run_time=.5)
                removed = obj.pop_tips()
                self.assertEqual(len(removed), 2)
                self.assertEqual(len(obj.submobjects), 0)
                scene.play(m.Transform(obj, obj.copy().shift(m.RIGHT),
                                       rate_func=m.linear), run_time=.5)
            return [path.read_bytes() for path in sorted(destination.glob("*.png"))]
        control = render("reference", 1, True)
        self.assertEqual(len(control), 4)
        self.assertGreater(len(set(control)), 1)
        for workers in (1, 4, 16):
            self.assertEqual(render(str(workers), workers, False), control)
        print("retained curved refit frames:", directory)

    def test_authored_curved_geometry_animates_at_one_and_four_threads(self):
        directory = Path(tempfile.mkdtemp(prefix="fmn-curved-lifecycle-"))
        def render(name, threads, authored):
            scene = m.Scene()
            destination = directory / name
            with scene.render_session(destination, format="png_sequence", fps=4,
                                      resolution=(96,54), threads=threads):
                class Authored(m.CurvedArrow):
                    def init_points(self):
                        self.set_points([[-1,0,0], [-.5,2,0], [0,1,0], [.5,2,0], [1,0,0]])
                if authored:
                    obj = Authored(2*m.LEFT, 2*m.RIGHT, color=m.BLUE)
                else:
                    obj = ReferenceCurve(color=m.BLUE)
                    obj.set_points([[-1,0,0], [-.5,2,0], [0,1,0], [.5,2,0], [1,0,0]])
                    obj.put_start_and_end_on(2*m.LEFT, 2*m.RIGHT)
                    obj.add_tip()
                scene.add(obj)
                scene.play(m.Transform(obj, obj.copy().shift(m.UP), rate_func=m.linear), run_time=1)
            return [p.read_bytes() for p in sorted(destination.glob("*.png"))]
        frames = render("one", 1, True)
        self.assertEqual(len(frames), 4)
        self.assertGreater(len(set(frames)), 1)
        self.assertEqual(frames, render("four", 4, True))
        self.assertEqual(frames, render("oracle", 1, False))
        print("retained curved lifecycle frames:", directory)


if __name__ == "__main__":
    unittest.main()
else:
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(NativeCurvedLifecycle))
    if not result.wasSuccessful():
        raise AssertionError("native curved lifecycle acceptance failed")
