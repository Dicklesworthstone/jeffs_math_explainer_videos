"""Real native NumberLine hooks, factory-produced ticks and sampled output."""
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m


class NumberLineLifecycleTests(unittest.TestCase):
    def test_hooks_run_once_before_public_factories(self):
        class Custom(m.NumberLine):
            def init_data(self):
                self.events = ['data']
                self.range_seen = self.x_range
                super().init_data()
            def init_points(self):
                self.events.append('points')
                super().init_points()
            def init_uniforms(self):
                self.events.append('uniforms')
                super().init_uniforms()
            def init_colors(self):
                self.events.append('colors')
                super().init_colors()
            def add_ticks(self):
                self.events.append('ticks')
                return super().add_ticks()
            def add_numbers(self, *args, **kwargs):
                self.events.append('numbers')
                return super().add_numbers(*args, **kwargs)
        line = Custom((-1, 1), include_numbers=True)
        self.assertEqual(line.events, ['data', 'points', 'uniforms', 'colors', 'ticks', 'numbers'])
        self.assertEqual(line.range_seen, (-1., 1., 1.))
        self.assertEqual(len(line.numbers), 3)

    def test_asymmetric_range_is_centered_then_maps_by_live_endpoints(self):
        line = m.NumberLine((-2, 4), unit_size=2, include_ticks=False)
        np.testing.assert_allclose(line.get_center(), m.ORIGIN)
        np.testing.assert_allclose(line.n2p([-2, 0, 4]), [[-6, 0, 0], [-2, 0, 0], [6, 0, 0]])
        line.rotate(m.PI / 2).shift(m.RIGHT)
        np.testing.assert_allclose(line.n2p(0), [1, -2, 0], atol=1e-6)
        self.assertAlmostEqual(line.p2n([1, 6, 0]), 4)

    def test_custom_record_lanes_and_children_survive(self):
        class Custom(m.NumberLine):
            data_dtype = m.VMobject.data_dtype + [('mass', 1)]
            def init_data(self):
                super().init_data()
                self.decoration = m.Dot()
                self.add(self.decoration)
            def init_points(self):
                super().init_points()
                self.data['mass'][:] = 7
            def init_uniforms(self):
                super().init_uniforms()
                self.uniforms['fixed_in_frame'] = 1
        line = Custom((-1, 1))
        self.assertIn(line.decoration, line.submobjects)
        np.testing.assert_allclose(line.data['mass'], 7)
        self.assertEqual(line.uniforms['fixed_in_frame'], 1)
        before = line.get_points().copy()
        scene = m.Scene(); scene.add(line)
        scene.play(line.animate.shift(m.RIGHT), run_time=1/30, rate_func=m.linear)
        np.testing.assert_allclose(line.get_points(), before + m.RIGHT)
        np.testing.assert_allclose(line.data['mass'], 7)

    def test_init_points_override_controls_actual_shape(self):
        class Custom(m.NumberLine):
            def init_points(self):
                self.set_points_as_corners([[-1,0,0],[0,1,0],[1,0,0]])
        line = Custom((-1,1), include_ticks=False)
        self.assertEqual(line.get_num_points(), 5)
        self.assertAlmostEqual(line.get_height(), 1)

    def test_factory_identity_and_big_ticks(self):
        made = []
        class Custom(m.NumberLine):
            def get_tick_range(self):
                return iter([-1, 0, 1])
            def get_tick(self, x, size=None):
                tick = super().get_tick(x, size)
                tick.shift(m.UP).set_color(m.RED)
                made.append(tick)
                return tick
        line = Custom((-1,1), big_tick_numbers=np.array([0]), tick_size=.2, longer_tick_multiple=3)
        self.assertEqual(list(line.ticks), made)
        np.testing.assert_allclose([x.get_height() for x in made], [.4,1.2,.4], atol=1e-6)
        np.testing.assert_allclose([x.get_center()[1] for x in made], 1)
        self.assertTrue(all(x.get_stroke_color()==m.RED for x in made))

    def test_live_tip_and_step_control_tick_range(self):
        line = m.NumberLine((-1,1), include_ticks=False)
        line.x_step = .5; line.include_tip = True
        np.testing.assert_array_equal(line.get_tick_range(), [-1,-.5,0,.5])
        line.include_tip = False
        np.testing.assert_array_equal(line.get_tick_range(), [-1,-.5,0,.5,1])

    def test_tip_config_is_not_limited_to_a_square(self):
        line = m.NumberLine((-1,1), include_tip=True, include_ticks=False,
                            tip_config={'width':.15, 'length':.3})
        self.assertTrue(line.has_tip())
        self.assertAlmostEqual(line.tip.get_width(), .3, places=6)
        self.assertAlmostEqual(line.tip.get_height(), .15, places=6)

    def test_style_aliases_and_custom_uniforms(self):
        line = m.NumberLine((-1,1), color=m.RED, stroke_color=m.GREEN, stroke_width=6,
                            include_ticks=False, flat_stroke=True)
        self.assertEqual(line.get_stroke_color(), m.GREEN)
        self.assertAlmostEqual(line.get_stroke_width(), 6)
        self.assertTrue(line.get_flat_stroke())

    def test_width_does_not_scale_the_tick_lengths(self):
        line = m.NumberLine((-1,1), width=10, unit_size=3, tick_size=.125)
        self.assertAlmostEqual(line.get_length(),10)
        self.assertTrue(all(np.isclose(x.get_length(), .25) for x in line.ticks))

    def test_failure_keeps_existing_tick_group_and_original_exception(self):
        line = m.NumberLine((-1,1)); previous = line.ticks; family = list(line.submobjects)
        error = RuntimeError('authored tick failure'); calls=[]
        def tick(x,size):
            calls.append(x)
            if x == 0: raise error
            return m.Line(m.DOWN, m.UP)
        line.get_tick = tick
        with self.assertRaises(RuntimeError) as caught: line.add_ticks()
        self.assertIs(caught.exception,error)
        self.assertIs(line.ticks,previous)
        self.assertEqual(list(line.submobjects),family)
        self.assertEqual(calls,[-1,0])

    def test_foreign_shared_or_invalid_factory_results_refuse_without_adoption(self):
        line=m.NumberLine((-1,1)); original=list(line.submobjects)
        foreign=m.Line(); scene=m.Scene(); scene.add(foreign)
        for factory in (lambda *a:foreign,lambda *a:line,lambda *a:None):
            line.get_tick=factory
            with self.assertRaises((ValueError,TypeError)): line.add_ticks()
            self.assertEqual(list(line.submobjects),original)
        shared=m.Line();line.get_tick=lambda *a:shared
        with self.assertRaises(ValueError):line.add_ticks()
        self.assertEqual(list(line.submobjects),original)

    def test_budgets_refuse_before_subclass_callbacks(self):
        calls=[]
        class Custom(m.NumberLine):
            def init_points(self):calls.append('points');super().init_points()
        for args in [dict(x_range=(0,1,1e-20)),dict(big_tick_spacing=1e-20),
                     dict(tick_size=float('nan')),dict(x_range=(1,1)),dict(width=-1)]:
            with self.subTest(args=args),self.assertRaises(ValueError):Custom(**args)
        self.assertEqual(calls,[])

    def test_unit_interval_and_axes_use_public_number_line(self):
        class Custom(m.UnitInterval):
            def init_points(self):self.called=True;super().init_points()
        unit=Custom();self.assertTrue(unit.called)
        self.assertAlmostEqual(unit.get_length(),10)
        self.assertEqual(len(unit.get_tick_range()),11)
        class Axis(m.NumberLine):
            def get_tick(self,x,size=None):return super().get_tick(x,size).set_color(m.RED)
        class Axes(m.Axes):
            def create_axis(self,range_terms,axis_config,length):
                axis=Axis(range_terms,width=length,**axis_config);axis.shift(-axis.n2p(0));return axis
        axes=Axes(x_range=(-1,1),y_range=(-1,1))
        self.assertTrue(all(t.get_stroke_color()==m.RED for a in axes.get_axes() for t in a.ticks))

    def test_rendered_tick_hooks_match_independent_geometry_and_thread_count(self):
        def render(path,custom,threads):
            scene=m.Scene()
            with scene.render_session(path,format='png_sequence',resolution=(96,54),fps=4,threads=threads):
                if custom:
                    class RaisedTicks(m.NumberLine):
                        def get_tick(self,x,size=None):return super().get_tick(x,size).shift(.3*m.UP)
                    obj=RaisedTicks((-1,1),tick_size=.1)
                else:
                    obj=m.VGroup(m.Line(m.LEFT,m.RIGHT,stroke_width=2,color=m.DEFAULT_LIGHT_COLOR),*[
                        m.Line([x,.2,0],[x,.4,0],stroke_width=2,color=m.DEFAULT_LIGHT_COLOR) for x in [-1,0,1]])
                scene.add(obj);scene.play(obj.animate.shift(m.RIGHT),run_time=.5,rate_func=m.linear)
            return [f.read_bytes() for f in sorted(path.glob('*.png'))]
        root=Path(tempfile.mkdtemp(prefix='fmn-numberline-'))
        first=render(root/'one',True,1)
        self.assertEqual(first,render(root/'four',True,4))
        self.assertEqual(first,render(root/'expected',False,1))
        self.assertEqual(len(first),2);self.assertNotEqual(first[0],first[1])


if __name__=='__main__':unittest.main()
