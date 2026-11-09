"""Native contour regeneration, topology changes, callback refusal and real frames."""
from pathlib import Path
import tempfile
import unittest

import manimlib as m
import numpy as np


def implicit(function, **kwargs):
    return m.ImplicitFunction(function, x_range=(-1, 1), y_range=(-1, 1),
                              min_depth=2, max_quads=31, **kwargs)


class LiveImplicitTests(unittest.TestCase):
    def test_refresh_observes_current_field_and_preserves_function_identity(self):
        level = [.125]
        function = lambda x, y: y-level[0]
        obj = implicit(function, color=m.BLUE, stroke_width=5)
        before = obj.get_points().copy()
        level[0] = .375
        self.assertIsNone(obj.init_points())
        self.assertFalse(np.array_equal(obj.get_points(), before))
        np.testing.assert_allclose(obj.get_points()[:, 1], .375, atol=1e-6)
        self.assertIs(obj.func, function)
        np.testing.assert_allclose(obj.get_stroke_widths(), 5)
        from manimlib.mobject.functions import ImplicitFunction
        self.assertIs(ImplicitFunction, m.ImplicitFunction)

    def test_live_updates_keep_native_owner_children_updaters_and_clock(self):
        level = [.125]
        obj = implicit(lambda x, y: y-level[0], stroke_width=6)
        child = m.Dot()
        callback = lambda current, dt: None
        obj.add(child).add_updater(callback, call=False)
        scene = m.Scene()
        scene.add(obj)
        before_time = scene.get_time()
        level[0] = .375
        obj.init_points()
        self.assertIs(obj._scene, scene)
        self.assertTrue(obj._is_bound())
        self.assertEqual(tuple(scene.mobjects), (obj,))
        self.assertIs(obj.submobjects[0], child)
        self.assertIs(obj.updaters[0], callback)
        self.assertEqual(scene.get_time(), before_time)
        np.testing.assert_allclose(obj.get_stroke_widths(), 6)

    def test_empty_contours_recover_current_style_defaults(self):
        visible = [True]
        obj = implicit(lambda x, y: y-.125 if visible[0] else 1., stroke_width=4)
        visible[0] = False
        obj.init_points()
        self.assertEqual(obj.get_num_points(), 0)
        obj.set_stroke(m.YELLOW, width=7, opacity=.6)
        visible[0] = True
        obj.init_points()
        self.assertGreater(obj.get_num_points(), 2)
        np.testing.assert_allclose(obj.get_stroke_widths(), 7)
        np.testing.assert_allclose(obj.get_stroke_opacities(), .6)
        np.testing.assert_allclose(obj.get_points()[:, 1], .125, atol=1e-6)

    def test_new_components_match_fresh_native_extraction(self):
        split = [False]
        def field(x, y):
            if split[0]:
                return ((x+.5)**2+y*y-.057)*((x-.5)**2+y*y-.057)
            return x*x+y*y-.23
        obj = m.ImplicitFunction(field, x_range=(-1, 1), y_range=(-1, 1),
                                  min_depth=4, max_quads=256)
        self.assertEqual(len(obj.get_subpaths()), 1)
        old = obj.get_points()
        before = old.copy()
        split[0] = True
        obj.init_points()
        self.assertEqual(len(obj.get_subpaths()), 2)
        reference = m.ImplicitFunction(field, x_range=(-1, 1), y_range=(-1, 1),
                                       min_depth=4, max_quads=256)
        np.testing.assert_array_equal(obj.get_points(), reference.get_points())
        np.testing.assert_array_equal(old, before)
        old[:] = 100
        np.testing.assert_array_equal(obj.get_points(), reference.get_points())

    def test_unchanged_record_count_retains_live_views(self):
        level = [.125]
        obj = implicit(lambda x, y: y-level[0])
        view = obj.get_points()
        count = len(view)
        level[0] = .25
        obj.init_points()
        self.assertEqual(obj.get_num_points(), count)
        np.testing.assert_array_equal(view, obj.get_points())
        np.testing.assert_allclose(view[:, 1], .25, atol=1e-6)

    def test_recipe_controls_and_smoothing_are_live(self):
        obj = implicit(lambda x, y: x*x+y*y-.23)
        obj.x_range, obj.y_range = (-.75, .75), (-.75, .75)
        obj.min_depth, obj.max_quads, obj.use_smoothing = 3, 100, True
        obj.init_points()
        reference = m.ImplicitFunction(obj.func, x_range=obj.x_range, y_range=obj.y_range,
            min_depth=obj.min_depth, max_quads=obj.max_quads, use_smoothing=True)
        np.testing.assert_array_equal(obj.get_points(), reference.get_points())

    def test_undefined_regions_remain_missing_after_refresh(self):
        level = [.125]
        for missing in (float('nan'), float('inf'), -float('inf')):
            obj = implicit(lambda x, y: missing if x < 0 else y-level[0])
            level[0] = .25
            obj.init_points()
            points = obj.get_points()
            self.assertGreater(len(points), 0)
            self.assertTrue(np.isfinite(points).all())
            self.assertTrue((points[:, 0] >= 0).all())
            np.testing.assert_allclose(points[:, 1], level[0], atol=1e-6)

    def test_failed_callback_preserves_records_exception_and_recovers(self):
        for error in (LookupError('field'), KeyboardInterrupt('cancel'), SystemExit('stop')):
            state = {'fail': False, 'level': .125}
            calls = []
            def field(x, y):
                calls.append((x, y))
                if state['fail']:
                    raise error
                return y-state['level']
            obj = implicit(field)
            before, view = obj.data.copy(), obj.get_points()
            state['fail'] = True
            calls.clear()
            with self.assertRaises(type(error)) as caught:
                obj.init_points()
            self.assertIs(caught.exception, error)
            self.assertEqual(len(calls), 1)
            np.testing.assert_array_equal(obj.data, before)
            np.testing.assert_array_equal(view, before['point'])
            state.update(fail=False, level=.375)
            obj.init_points()
            np.testing.assert_allclose(obj.get_points()[:, 1], .375, atol=1e-6)

    def test_callback_geometry_edits_are_retained_not_overwritten(self):
        state = {'edit': False, 'done': False}
        def field(x, y):
            if state['edit'] and not state['done']:
                state['done'] = True
                obj.shift((1, 0, 0))
            return y-.125
        obj = implicit(field)
        before = obj.get_points().copy()
        state['edit'] = True
        with self.assertRaisesRegex(RuntimeError, 'changed during sampling'):
            obj.init_points()
        np.testing.assert_allclose(obj.get_points(), before+(1, 0, 0), atol=1e-6)
        obj.init_points()
        np.testing.assert_allclose(obj.get_points(), before, atol=1e-6)

    def test_changed_recipe_or_family_is_not_partly_published(self):
        for change in ('function', 'controls', 'family', 'owner'):
            state = {'edit': False, 'done': False}
            scene, child = m.Scene(), m.Dot()
            replacement = lambda x, y: y-.375
            def field(x, y):
                if state['edit'] and not state['done']:
                    state['done'] = True
                    if change == 'function':
                        obj.func = replacement
                    elif change == 'controls':
                        obj.max_quads += 1
                    elif change == 'family':
                        obj.add(child)
                    else:
                        scene.add(obj)
                return y-.125
            obj = implicit(field)
            before = obj.get_points().copy()
            state['edit'] = True
            with self.assertRaisesRegex(RuntimeError, 'changed during sampling'):
                obj.init_points()
            np.testing.assert_array_equal(obj.get_points(), before)
            if change == 'family':
                self.assertIs(obj.submobjects[0], child)
            if change == 'owner':
                self.assertIs(obj._scene, scene)
            if change == 'function':
                self.assertIs(obj.func, replacement)

    def test_reentrancy_and_active_animation_refuse_before_publication(self):
        recurse = [False]
        def field(x, y):
            if recurse[0]:
                obj.init_points()
            return y-.125
        obj = implicit(field)
        before = obj.get_points().copy()
        recurse[0] = True
        with self.assertRaisesRegex(RuntimeError, 'reenter'):
            obj.init_points()
        np.testing.assert_array_equal(obj.get_points(), before)
        recurse[0] = False
        animation = m.Transform(obj, obj.copy().shift((0, .5, 0)))
        animation.begin()
        with self.assertRaisesRegex(RuntimeError, 'active animation'):
            obj.init_points()
        animation.finish()
        obj.init_points()
        np.testing.assert_array_equal(obj.get_points(), before)

    def test_copy_and_saved_state_keep_independent_geometry(self):
        level = [.125]
        obj = implicit(lambda x, y: y-level[0])
        obj.save_state()
        copied = obj.copy(deep=True)
        before = obj.get_points().copy()
        level[0] = .375
        copied.init_points()
        np.testing.assert_array_equal(obj.get_points(), before)
        obj.init_points()
        np.testing.assert_array_equal(obj.get_points(), copied.get_points())
        obj.restore()
        np.testing.assert_array_equal(obj.get_points(), before)
        np.testing.assert_allclose(copied.get_points()[:, 1], .375, atol=1e-6)

    def test_invalid_native_controls_fail_before_field_evaluation(self):
        calls = []
        obj = implicit(lambda x, y: calls.append((x, y)) or y-.125)
        before = obj.get_points().copy()
        for key, value in (('max_quads', 65537), ('min_depth', 32), ('x_range', (0, 0))):
            old = getattr(obj, key)
            setattr(obj, key, value)
            calls.clear()
            with self.assertRaises(ValueError):
                obj.init_points()
            self.assertEqual(calls, [])
            np.testing.assert_array_equal(obj.get_points(), before)
            setattr(obj, key, old)

    def test_real_moving_contours_equal_an_independent_line_at_one_four_threads(self):
        class ContourScene(m.Scene):
            reference = False
            def construct(self):
                level = m.ValueTracker(-.5)
                if self.reference:
                    obj = m.Line((-1, -.5, 0), (1, -.5, 0), buff=0)
                    obj.add_updater(lambda current: current.put_start_and_end_on(
                        (-1, level.get_value(), 0), (1, level.get_value(), 0)), call=False)
                else:
                    obj = implicit(lambda x, y: y-level.get_value())
                    obj.add_updater(lambda current: current.init_points(), call=False)
                obj.set_stroke(m.BLUE, width=4, opacity=1).set_fill(opacity=0)
                obj.set_joint_type('no_joint')
                self.add(obj)
                self.play(level.animate.set_value(.5), run_time=1, rate_func=m.linear)
        with tempfile.TemporaryDirectory(prefix='fmn-live-implicit-') as directory:
            outputs = []
            for reference, threads in ((True, 1), (False, 1), (False, 4)):
                scene = ContourScene()
                scene.reference = reference
                path = Path(directory) / f'{reference}-{threads}.y4m'
                receipt = scene.render(path, format='y4m', resolution=(96, 54), fps=4, threads=threads)
                self.assertEqual(receipt.frame_count, 4)
                outputs.append(path.read_bytes())
            self.assertEqual(outputs[0], outputs[1])
            self.assertEqual(outputs[1], outputs[2])


def run_live_implicit():
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(LiveImplicitTests))
    if not result.wasSuccessful():
        raise AssertionError('live implicit native acceptance failed')


if __name__ in ('__main__', '<run_path>'):
    run_live_implicit()
