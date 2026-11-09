"""Native brace hooks, live regeneration, preserved records and actual pixels."""
from pathlib import Path
import copy
import pickle
import tempfile
import unittest

import manimlib as m
import numpy as np


def native_brace(source, direction=m.DOWN, buff=.2, *, line=False):
    result = m.VMobject.__new__(m.VMobject)
    m._install_live_state(result)
    if line:
        specs, tip = result._build_line_brace(m._native_shell_factory,
                                             source.get_start(), source.get_end(), direction, buff)
    else:
        specs, tip = result._build_brace(m._native_shell_factory, source, direction, buff)
    assert not specs
    return result, tip


class BraceLifecycleTests(unittest.TestCase):
    def test_composite_targets_are_measured_over_their_whole_family(self):
        # fm-5wq.43: the Reference sizes a brace from the target's corners
        # (its family's points). A detached composite once yielded a width-0
        # brace at the origin; the pinned Reference gives Brace(Matrix, DOWN)
        # exactly the matrix's width, centred below it.
        makers = {
            "VGroup": lambda: m.VGroup(m.Square(), m.Circle().shift(3 * m.RIGHT)),
            "Matrix": lambda: m.Matrix([[1, 2], [3, 4]]),
            "Tex": lambda: m.Tex(r"a + b"),
            "DecimalNumber": lambda: m.DecimalNumber(3.14),
            "Axes": lambda: m.Axes(x_range=(-2, 2), y_range=(-1, 1), width=4, height=2),
        }
        for name, make in makers.items():
            for bound in (False, True):
                with self.subTest(target=name, bound=bound):
                    target = make().to_corner(m.UR)
                    if bound:
                        m.Scene().add(target)
                    brace = m.Brace(target, m.DOWN, buff=.2)
                    self.assertAlmostEqual(brace.get_width(), target.get_width(), delta=1e-3)
                    self.assertAlmostEqual(brace.get_center()[0], target.get_center()[0], delta=1e-3)
                    self.assertAlmostEqual(brace.get_top()[1], target.get_bottom()[1] - .2, delta=1e-3)
                    label = brace.get_text("A")
                    self.assertLess(label.get_top()[1], target.get_bottom()[1])

    def test_native_geometry_tip_and_default_paint_are_preserved(self):
        target = m.Rectangle(width=3, height=.75).shift(m.RIGHT)
        for cls, target in ((m.Brace, target), (m.LineBrace, m.Line([-2,1,0],[1,3,0]))):
            for direction in (m.DOWN, m.UP, m.LEFT, np.array([1.,-2.,0.])):
                with self.subTest(cls=cls, direction=direction):
                    obj = cls(target, direction, buff=.15)
                    expected, tip = native_brace(target, direction, .15, line=cls is m.LineBrace)
                    np.testing.assert_array_equal(obj.get_points(), expected.get_points())
                    np.testing.assert_array_equal(obj.data['fill_rgba'], expected.data['fill_rgba'])
                    np.testing.assert_array_equal(obj.data['stroke_width'], expected.data['stroke_width'])
                    self.assertEqual(obj.tip_point_index, tip)
                    np.testing.assert_array_equal(obj.get_tip(), expected.get_points()[tip])

    def test_all_hooks_run_in_order_with_recipe_available(self):
        events=[]
        class Authored(m.Brace):
            def init_data(self):
                events.append(('data',self.buff,self.font_size));super().init_data()
            def init_points(self):
                events.append('points');super().init_points();self.shift(m.RIGHT)
            def init_uniforms(self):
                events.append('uniforms');super().init_uniforms()
            def init_colors(self):
                events.append('colors');super().init_colors();self.set_color(m.RED)
        target=m.Square()
        actual=Authored(target,buff=.3,font_size=24,color=m.BLUE)
        expected,_=native_brace(target,buff=.3);expected.shift(m.RIGHT)
        self.assertEqual(events,[('data',.3,24.),'points','uniforms','colors'])
        np.testing.assert_array_equal(actual.get_points(),expected.get_points())
        self.assertEqual(actual.get_fill_color(),m.RED)

    def test_custom_data_and_decorations_survive_regeneration(self):
        class Authored(m.Brace):
            data_dtype=m.VMobject.data_dtype+[('weight',1)]
            def init_data(self):
                super().init_data();self.marker=m.Dot();self.add(self.marker)
            def init_points(self):
                super().init_points();self.data['weight'][:]=7
            def init_uniforms(self):
                super().init_uniforms();self.uniforms['is_fixed_in_frame']=1
        target=m.Square();obj=Authored(target)
        obj.set_color(m.GREEN)
        weights=obj.data['weight'];points=obj.data['point']
        target.shift(2*m.RIGHT);obj.init_points()
        self.assertIs(obj.submobjects[0],obj.marker)
        np.testing.assert_array_equal(weights,7)
        np.testing.assert_array_equal(points,obj.get_points())
        self.assertEqual(obj.get_fill_color(),m.GREEN)
        self.assertTrue(obj.is_fixed_in_frame())

    def test_bound_regeneration_follows_live_source_without_replacing_identity(self):
        target=m.Square();obj=m.Brace(target)
        scene=m.Scene();scene.add(target,obj)
        before_target=target.get_points().copy()
        target.shift(2*m.RIGHT).scale(.5)
        expected,tip=native_brace(target)
        obj.init_points()
        self.assertIn(obj,scene.mobjects)
        np.testing.assert_array_equal(obj.get_points(),expected.get_points())
        np.testing.assert_array_equal(obj.get_tip(),expected.get_points()[tip])
        np.testing.assert_array_equal(target.get_points(),(before_target+2*m.RIGHT-m.RIGHT*2)*.5+m.RIGHT*2)

    def test_line_brace_samples_public_endpoints_once(self):
        calls=[]
        class Authored(m.Line):
            def get_start(self):calls.append('start');return np.array([0.,1.,0.])
            def get_end(self):calls.append('end');return np.array([2.,3.,0.])
        line=Authored();calls.clear()
        actual=m.LineBrace(line);self.assertEqual(calls,['start','end'])
        expected,_=native_brace(m.Line([0,1,0],[2,3,0]),m.UP,line=True)
        np.testing.assert_array_equal(actual.get_points(),expected.get_points())

    def test_line_brace_subclass_and_replay_use_actual_hooks(self):
        class Authored(m.LineBrace):
            def init_points(self):
                self.calls=getattr(self,'calls',0)+1
                super().init_points();self.shift(m.OUT)
        line=m.Line(m.LEFT,m.RIGHT);obj=Authored(line)
        self.assertEqual(obj.calls,1)
        line.shift(m.UP);obj.init_points()
        expected,_=native_brace(line,m.UP,line=True);expected.shift(m.OUT)
        self.assertEqual(obj.calls,2)
        np.testing.assert_array_equal(obj.get_points(),expected.get_points())

    def test_replacement_point_hook_and_explicit_tip(self):
        class Authored(m.Brace):
            def init_points(self):
                self.set_points_as_corners([[-1,0,0],[0,-1,0],[1,0,0]])
                self.tip_point_index=2
        obj=Authored(m.Square());self.assertEqual(obj.get_num_points(),5)
        np.testing.assert_array_equal(obj.get_tip(),[0,-1,0])
        obj.rotate(m.PI/2, about_point=m.ORIGIN).shift(m.RIGHT)
        np.testing.assert_allclose(obj.get_tip(),[2,0,0],atol=1e-7)

    def test_implicit_tip_for_authored_replacement(self):
        class Authored(m.Brace):
            def init_points(self):self.set_points_as_corners([[-1,0,0],[0,-2,0],[1,0,0]])
        obj=Authored(m.Square())
        np.testing.assert_array_equal(obj.get_tip(),[0,-2,0])

    def test_copy_pickle_and_saved_state_remain_usable(self):
        source=m.Square();original=m.Brace(source);original.add(m.Dot())
        before=original.get_points().copy()
        for duplicate in (original.copy(),copy.deepcopy(original),pickle.loads(pickle.dumps(original))):
            with self.subTest(kind=type(duplicate)):
                self.assertIsNot(duplicate.submobjects[0],original.submobjects[0])
                duplicate.init_points();duplicate.shift(m.RIGHT)
                np.testing.assert_array_equal(original.get_points(),before)
        original.save_state();original.shift(m.RIGHT);original.init_points();original.restore()
        np.testing.assert_array_equal(original.get_points(),before)

    def test_failure_is_not_retried_or_followed_by_later_hooks(self):
        events=[];error=RuntimeError('authored points failed')
        class Authored(m.Brace):
            def init_points(self):events.append('points');raise error
            def init_uniforms(self):events.append('uniforms')
        with self.assertRaises(RuntimeError) as caught:Authored(m.Square())
        self.assertIs(caught.exception,error);self.assertEqual(events,['points'])
        self.assertTrue(m.Brace(m.Square()).has_points())

    def test_invalid_constructor_input_refuses_before_hooks(self):
        calls=[]
        class Authored(m.Brace):
            def init_points(self):calls.append(1);super().init_points()
        for kwargs in ({'buff':float('nan')},{'direction':[0,float('inf'),0]},
                       {'font_size':0},{'unknown':1}):
            with self.subTest(kwargs=kwargs),self.assertRaises((TypeError,ValueError)):
                Authored(m.Square(),**kwargs)
        self.assertEqual(calls,[])
        with self.assertRaises(TypeError):Authored(object())

    def test_failed_live_recipe_leaves_previous_geometry_and_tip(self):
        line=m.Line();obj=m.LineBrace(line);before=obj.data.copy();tip=obj.tip_point_index
        error=RuntimeError('endpoints failed')
        def fail():raise error
        line.get_start=fail
        with self.assertRaises(RuntimeError) as caught:obj.init_points()
        self.assertIs(caught.exception,error)
        np.testing.assert_array_equal(obj.data,before);self.assertEqual(obj.tip_point_index,tip)
        line.get_start=lambda:np.array([0.,0.,0.]);obj.init_points()

    def test_reentrant_and_bound_constructors_refuse(self):
        target=m.Square();obj=m.Brace(target);scene=m.Scene();scene.add(obj)
        before=obj.get_points().copy()
        with self.assertRaises(RuntimeError):obj.__init__(target)
        np.testing.assert_array_equal(obj.get_points(),before)
        class Reentrant(m.Brace):
            def init_points(self):self.__init__(target)
        with self.assertRaisesRegex(RuntimeError,'already in progress'):Reentrant(target)

    def test_animation_lock_and_invalid_endpoint_do_not_publish(self):
        line=m.Line();obj=m.LineBrace(line);before=obj.data.copy()
        obj.locked_data_keys.add('point')
        with self.assertRaisesRegex(RuntimeError,'active animation'):obj.init_points()
        obj.locked_data_keys.clear()
        line.get_end=lambda:np.array([float('nan'),0.,0.])
        with self.assertRaises(ValueError):obj.init_points()
        np.testing.assert_array_equal(obj.data,before)

    def test_endpoint_callback_cannot_silently_replace_live_edits(self):
        line=m.Line();obj=m.LineBrace(line);tip=obj.tip_point_index
        def endpoint():
            obj.shift(m.RIGHT)
            return np.array([0.,0.,0.])
        line.get_start=endpoint
        before=obj.get_points().copy()
        with self.assertRaisesRegex(RuntimeError,'changed during sampling'):obj.init_points()
        # Authored effects remain, but the sampled candidate was not installed.
        np.testing.assert_array_equal(obj.get_points(),(before+m.RIGHT).astype(np.float32))
        self.assertEqual(obj.tip_point_index,tip)

    def test_builder_regeneration_retains_attached_children(self):
        source=m.Square();obj=m.Brace(source);marker=m.Dot().shift(m.UP);obj.add(marker)
        scene=m.Scene();scene.add(obj)
        source.shift(m.RIGHT)
        target,_=native_brace(source)
        scene.play(obj.animate.init_points(),run_time=1/30,rate_func=m.linear)
        self.assertIs(obj.submobjects[0],marker)
        np.testing.assert_array_equal(obj.get_points(),target.get_points())

    def test_label_helpers_follow_transformed_tip(self):
        class Authored(m.Brace):
            def init_points(self):super().init_points();self.shift(2*m.RIGHT)
        brace=Authored(m.Square());text=brace.get_text('x');tex=brace.get_tex('x')
        self.assertAlmostEqual(text.get_center()[0],brace.get_tip()[0],places=6)
        self.assertAlmostEqual(tex.get_center()[0],brace.get_tip()[0],places=6)

    def test_rendered_hooks_match_independent_native_geometry_at_all_frames(self):
        class Authored(m.Brace):
            def init_points(self):super().init_points();self.shift(.25*m.UP)
        def render(path,custom,threads):
            scene=m.Scene()
            with scene.render_session(path,format='png_sequence',resolution=(96,54),fps=4,threads=threads):
                target=m.Rectangle(width=3,height=1)
                obj=Authored(target) if custom else native_brace(target)[0].shift(.25*m.UP)
                scene.add(obj);scene.play(obj.animate.shift(m.RIGHT),run_time=.5,rate_func=m.linear)
            return [p.read_bytes() for p in sorted(path.glob('*.png'))]
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);first=render(root/'one',True,1)
            self.assertEqual(first,render(root/'four',True,4))
            self.assertEqual(first,render(root/'control',False,1))
            self.assertEqual(len(first),2);self.assertNotEqual(first[0],first[1])


if __name__=='__main__':
    unittest.main()
