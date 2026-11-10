"""Point-cloud subclassing through native records, primitive identity and output."""
from pathlib import Path
import copy
import gc
import tempfile
import unittest
import weakref

import numpy as np
import manimlib as m
from fmn_python import render_session


class TaggedCloud(m.DotCloud):
    data_dtype = [*m.DotCloud.data_dtype, ('temperature', np.float32, (1,))]

    def init_points(self):
        self.set_points([[-1, 0, 0], [0, 1, 0], [1, 0, 0]])
        self.data['temperature'][:] = [[10], [20], [30]]


def kernel(points, *, color=m.BLUE, radius=.2, glow=0., opacity=.7):
    result = m.DotCloud.__new__(m.DotCloud)
    m._install_live_state(result)
    specs = result._build_dot_cloud(m._native_shell_factory, points, color,
                                    opacity, radius, glow, 2.)
    assert not specs
    result.radius, result.glow_factor, result.anti_alias_width = radius, glow, 2.
    result.init_uniforms()
    return result


def render(path, cloud, threads=1):
    scene = m.Scene()
    with render_session(scene, path, resolution=(64, 40), fps=24, threads=threads) as session:
        scene.add(cloud)
        scene.wait(.125)
    assert session.result.frame_count == 3
    return path.read_bytes()


class PointCloudLifecycleTests(unittest.TestCase):
    def test_hooks_run_in_order_on_all_cloud_classes(self):
        for base in (m.DotCloud, m.TrueDot, m.GlowDots, m.GlowDot):
            with self.subTest(base=base.__name__):
                events = []
                class Hooks:
                    def init_data(self):
                        events.append('data')
                        super().init_data()
                    def init_points(self):
                        events.append('points')
                        self.set_points([[1, 2, 0]])
                    def init_uniforms(self):
                        events.append('uniforms')
                        super().init_uniforms()
                    def init_colors(self):
                        events.append('colors')
                        super().init_colors()
                class Cloud(Hooks, base):
                    pass
                cloud = Cloud()
                self.assertEqual(events, ['data', 'points', 'uniforms', 'colors'])
                np.testing.assert_array_equal(cloud.get_points(), [[0, 0, 0]])

    def test_none_retains_authored_points_but_default_is_origin(self):
        for base in (m.DotCloud, m.GlowDots):
            class Cloud(base):
                def init_points(self):
                    self.set_points([[1, 2, 0], [-1, -2, 0]])
            self.assertEqual(Cloud().get_num_points(), 1)
            np.testing.assert_array_equal(Cloud(points=None).get_points(), [[1, 2, 0], [-1, -2, 0]])
            self.assertEqual(Cloud(points=[]).get_num_points(), 0)

    def test_custom_record_fields_survive_constructor_copy_and_transform(self):
        cloud = TaggedCloud(points=None, radius=.25, color=m.RED)
        self.assertIn('temperature', cloud.data.dtype.names)
        np.testing.assert_array_equal(cloud.data['temperature'].ravel(), [10, 20, 30])
        duplicate = cloud.copy()
        np.testing.assert_array_equal(duplicate.data, cloud.data)
        duplicate.data['temperature'][:] = -1
        np.testing.assert_array_equal(cloud.data['temperature'].ravel(), [10, 20, 30])
        cloud.shift(m.RIGHT)
        np.testing.assert_array_equal(cloud.get_points(), [[0, 0, 0], [1, 1, 0], [2, 0, 0]])
        np.testing.assert_array_equal(cloud.get_radii(), [[.25]] * 3)

    def test_explicit_points_use_public_setter_after_color_and_radius_hooks(self):
        events = []
        class Cloud(m.DotCloud):
            def init_points(self):
                self.set_points([[1, 0, 0]])
            def init_colors(self):
                events.append('colors')
                super().init_colors()
                self.set_color(m.BLUE)
            def set_radius(self, radius):
                events.append(('radius', radius))
                return super().set_radius(radius)
            def set_points(self, points):
                events.append(('points', tuple(np.asarray(points).reshape(-1))))
                return super().set_points(points)
        cloud = Cloud(points=[[2, 0, 0]], radius=.4, color=m.RED)
        self.assertEqual(events, [('points', (1, 0, 0)), 'colors', ('radius', .4), ('points', (2, 0, 0))])
        np.testing.assert_array_equal(cloud.get_points(), [[2, 0, 0]])
        self.assertEqual(cloud.get_color(), m.BLUE)

    def test_native_builder_geometry_style_and_glow_are_preserved(self):
        for points in ([], [[0, 0, 0]], [[-1, .5, 0], [1, -.5, .2]]):
            for glow in (0., 2.):
                with self.subTest(points=points, glow=glow):
                    cloud = m.DotCloud(points, radius=.2, color=m.BLUE, opacity=.7, glow_factor=glow)
                    expected = kernel(points, glow=glow)
                    for name in ('point', 'radius', 'rgba', 'glow_factor'):
                        np.testing.assert_array_equal(cloud.data[name], expected.data[name])
                    self.assertEqual(cloud.uniforms['anti_alias_width'], 2.)

    def test_pgroup_retains_hook_children_and_argument_identity(self):
        child, decoration = m.TrueDot(m.LEFT), m.TrueDot(m.RIGHT)
        events = []
        class Group(m.PGroup):
            data_dtype = [*m.PGroup.data_dtype, ('tag', np.float32, (1,))]
            def init_data(self):
                events.append('data')
                self.add(decoration)
            def init_points(self):
                events.append('points')
            def init_uniforms(self):
                events.append('uniforms')
                super().init_uniforms()
            def init_colors(self):
                events.append('colors')
                super().init_colors()
        group = Group(child)
        self.assertEqual(events, ['data', 'points', 'uniforms', 'colors'])
        self.assertEqual(list(group), [decoration, child])
        self.assertIn('tag', group.data.dtype.names)
        with self.assertRaisesRegex(Exception, 'PMobject'):
            Group(m.Circle())

    def test_hook_children_and_native_updaters_remain_owned(self):
        decoration = m.TrueDot(m.UP)
        def update(mob, dt):
            mob.shift(dt * m.RIGHT)
        class Cloud(m.DotCloud):
            def init_data(self):
                self.add(decoration)
                self.add_updater(update, call=False)
            def init_points(self):
                self.set_points([[0, 0, 0]])
        cloud = Cloud(points=None)
        self.assertIs(cloud.submobjects[0], decoration)
        self.assertIn(update, cloud.updaters)
        scene = m.Scene()
        scene.add(cloud)
        scene.wait(.125)
        self.assertGreater(cloud.get_points()[0, 0], 0.)
        self.assertIs(cloud.submobjects[0], decoration)

    def test_empty_cloud_retains_radius_glow_and_renderability_after_append(self):
        cloud = m.GlowDots(points=None, radius=.3)
        self.assertEqual(cloud.get_num_points(), 0)
        cloud.add_points([[0, 0, 0], [1, 0, 0]])
        np.testing.assert_allclose(cloud.get_radii(), [[.3], [.3]])
        np.testing.assert_array_equal(cloud.data['glow_factor'], [[2.], [2.]])
        with tempfile.TemporaryDirectory() as tmp:
            actual = render(Path(tmp) / 'actual.y4m', cloud)
            expected = render(Path(tmp) / 'expected.y4m', kernel([[0, 0, 0], [1, 0, 0]],
                              radius=.3, color=m.YELLOW, glow=2., opacity=1.))
            self.assertEqual(actual, expected)

    def test_finite_point_tables_take_one_reduction(self):
        # fm-ztjj work counter: a finite table is proven finite and
        # f32-representable by one reduction, so constructing a GlowDot calls
        # neither isfinite nor finfo (the earlier check called both every
        # time). A non-finite table still gets the exact refusal.
        m.GlowDot()
        calls = []
        originals = {name: getattr(np, name) for name in ("isfinite", "finfo")}

        def counting(name):
            def wrapper(*args, **kwargs):
                calls.append(name)
                return originals[name](*args, **kwargs)
            return wrapper

        for name in originals:
            setattr(np, name, counting(name))
        try:
            m.GlowDot(m.ORIGIN, radius=.075, glow_factor=1)
        finally:
            for name, original in originals.items():
                setattr(np, name, original)
        self.assertEqual(calls, [])
        for points, message in (([[np.nan, 0, 0]], "finite"),
                                ([[1e39, 0, 0]], "f32-representable")):
            with self.assertRaisesRegex(ValueError, message):
                m.DotCloud(points)

    def test_invalid_parameters_refuse_before_hooks(self):
        events = []
        class Cloud(m.DotCloud):
            def init_data(self):
                events.append('data')
        for name in ('radius', 'glow_factor', 'anti_alias_width'):
            for value in (-1., float('nan'), float('inf'), 1e40):
                with self.subTest(name=name, value=value), self.assertRaises(ValueError):
                    Cloud(**{name: value})
        self.assertEqual(events, [])

    def test_original_hook_exception_propagates_and_new_construction_recovers(self):
        error = RuntimeError('authored point-cloud failure')
        for name in ('init_data', 'init_points', 'init_uniforms', 'init_colors'):
            def fail(self):
                raise error
            Cloud = type('Cloud', (m.DotCloud,), {name: fail})
            with self.assertRaises(RuntimeError) as raised:
                Cloud()
            self.assertIs(raised.exception, error)
        self.assertEqual(m.DotCloud().get_num_points(), 1)

    def test_initialization_guard_rejects_recursion_and_live_reconstruction(self):
        class Reentrant(m.DotCloud):
            def init_points(self):
                m.DotCloud.__init__(self)
        with self.assertRaisesRegex(RuntimeError, 'progress'):
            Reentrant()
        cloud = m.DotCloud()
        scene = m.Scene()
        scene.add(cloud)
        before = cloud.data.copy()
        with self.assertRaisesRegex(RuntimeError, 'detached'):
            m.DotCloud.__init__(cloud)
        np.testing.assert_array_equal(cloud.data, before)

    def test_native_finalization_refuses_invalid_records_without_mutation(self):
        class Raw(m.Mobject):
            data_dtype = [('point', 3), ('radius', 1), ('rgba', 4), ('glow_factor', 1)]
        for name, value in (('point', float('nan')), ('radius', -1.), ('glow_factor', -1.)):
            raw = Raw()
            raw.set_points([[0, 0, 0]])
            raw.data[name][:] = value
            before = raw.data.tobytes()
            with self.assertRaises(ValueError):
                m._validate_point_cloud(raw)
            self.assertEqual(raw.data.tobytes(), before)
        scene = m.Scene()
        cloud = m.DotCloud()
        scene.add(cloud)
        with self.assertRaisesRegex(RuntimeError, 'detached'):
            m._validate_point_cloud(cloud)

    def test_saved_state_and_collected_families_survive_native_handoff(self):
        class Cloud(m.DotCloud):
            def init_points(self):
                self.set_points([[0, 0, 0]])
                self.save_state()
        cloud = Cloud(points=None)
        self.assertIsNotNone(cloud.saved_state)
        child = m.TrueDot()
        cloud.add(child)
        owners = weakref.ref(cloud), weakref.ref(child)
        del cloud, child
        gc.collect()
        self.assertEqual(tuple(ref() for ref in owners), (None, None))

    def test_rendered_custom_points_match_independent_native_cloud_at_all_worker_counts(self):
        points = [[-1, 0, 0], [0, 1, 0], [1, 0, 0]]
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            expected = render(root / 'expected.y4m', kernel(points))
            negative = render(root / 'negative.y4m', kernel([[0, 0, 0]]))
            self.assertNotEqual(expected, negative)
            for threads in (1, 4, 16):
                actual = render(root / f'actual-{threads}.y4m', TaggedCloud(
                    points=None, radius=.2, color=m.BLUE, opacity=.7), threads)
                self.assertEqual(actual, expected)


def run_native_point_cloud_lifecycle():
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(PointCloudLifecycleTests))
    if not result.wasSuccessful():
        raise AssertionError('native point-cloud lifecycle acceptance failed')


if __name__ == '__main__':
    run_native_point_cloud_lifecycle()
