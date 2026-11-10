"""Native legacy TeX initialization, normalized expressions and safe regeneration."""
import copy
import gc
import importlib
import inspect
from pathlib import Path
import pickle
import tempfile
import unittest
import weakref

import manimlib as m
import numpy as np
from manimlib.mobject.svg.old_tex_mobject import SingleStringTex


def native_tex(source, *, math_mode=True, font_size=48, height=None, **style):
    target = m._native_shell_factory()
    specs = target._build_tex(m._native_shell_factory, [source], '', not math_mode,
                             float(font_size), None, False, '', '', 'center')
    m._hang_native_children(target, specs)
    # SingleStringTex's own white fill is a constructor default: a caller's
    # color= beats it (BN-07 C-19, fm-qead).
    defaults = dict(fill_color=m._StyleDefault(m.WHITE), fill_opacity=1., stroke_width=0.)
    defaults.update(style)
    m._apply_vmobject_style_kwargs(target, defaults)
    if height is not None:
        target.set_height(height)
    return target


def assert_family(test, actual, expected):
    left, right = actual.get_family(), expected.get_family()
    test.assertEqual(len(left), len(right))
    for a, b in zip(left, right):
        np.testing.assert_array_equal(a.get_points(), b.get_points())
        for key in ('fill_rgba', 'stroke_rgba', 'stroke_width', 'fill_border_width'):
            np.testing.assert_array_equal(a.data[key], b.data[key])


class Decorated(SingleStringTex):
    data_dtype = m.VMobject.data_dtype + [('weight', 1)]

    def init_data(self):
        super().init_data()
        self.marker = m.Dot().shift(m.UP)
        self.add(self.marker)

    def init_points(self):
        super().init_points()
        self.set_points_as_corners([[-1.,0.,0.], [1.,0.,0.]])
        self.data['weight'][:] = 5


class LegacyTexLifecycleTests(unittest.TestCase):
    def test_qualified_class_identity_and_signature(self):
        qualified = importlib.import_module('manimlib.mobject.svg.old_tex_mobject')
        self.assertIs(qualified.SingleStringTex, SingleStringTex)
        self.assertEqual(SingleStringTex.__bases__, (m.SVGMobject,))
        self.assertFalse(hasattr(m, 'SingleStringTex'))
        signature = str(inspect.signature(SingleStringTex))
        self.assertEqual(signature,
            "(tex_string, height=None, fill_color='#FFFFFF', fill_opacity=1.0, "
            "stroke_width=0, svg_default={'fill_color': '#FFFFFF'}, "
            "path_string_config={}, font_size=48, alignment='\\\\centering', "
            "math_mode=True, organize_left_to_right=False, template='', "
            "additional_preamble='', **kwargs)")

    def test_stock_native_layout_and_paint_are_unchanged(self):
        cases = [(r'x^2+y', True), (r'\frac{x}{y}', True), ('native', False),
                 ('é + λ', False), (r'\sqrt{x}', True)]
        for source, mode in cases:
            with self.subTest(source=source):
                options = dict(math_mode=mode, font_size=36, fill_color=m.GREEN,
                               fill_opacity=.6, stroke_width=1.25)
                actual = SingleStringTex(source, **options)
                expected = native_tex(source, **options)
                assert_family(self, actual, expected)

    def test_none_styles_retain_native_defaults_and_opacity_fallback(self):
        for opacity in (None, .25):
            options = dict(fill_color=None, fill_opacity=None, stroke_color=None,
                           stroke_width=None, fill_border_width=None, opacity=opacity)
            obj = SingleStringTex('x', **options)
            assert_family(self, obj, native_tex('x', **options))

    def test_live_glyph_rebuilds_render_each_frame(self):
        def render(path, rebuilding, threads):
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(96,54),
                                      fps=4, threads=threads):
                obj = SingleStringTex('x', fill_opacity=.5)
                scene.add(obj)
                step = [0]
                def update(current, dt):
                    if not dt:
                        return
                    step[0] += 1
                    text = 'x' if step[0] % 2 else 'yy'
                    if rebuilding:
                        current.tex_string = text
                        current.init_points()
                    else:
                        expected = native_tex(text, fill_opacity=.5)
                        current.set_submobjects(list(expected.submobjects))
                    current.shift(step[0] * .25 * m.RIGHT)
                obj.add_updater(update)
                scene.wait(.75)
            return [p.read_bytes() for p in sorted(path.glob('*.png'))]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = render(root/'one', True, 1)
            self.assertEqual(first, render(root/'four', True, 4))
            self.assertEqual(first, render(root/'native', False, 1))
            self.assertEqual(len(first), 3)
            self.assertEqual(len(set(first)), 3)

    def test_hooks_run_once_with_normalization_inside_points(self):
        events = []
        class Authored(SingleStringTex):
            def init_data(self):
                events.append(('data', self.tex_string, self.font_size))
                super().init_data()
            def init_points(self):
                events.append('points'); super().init_points(); self.shift(m.RIGHT)
            def get_modified_expression(self, text):
                events.append(('expression', text))
                return super().get_modified_expression(text)
            def init_uniforms(self):
                events.append('uniforms'); super().init_uniforms()
            def init_colors(self):
                events.append('colors'); super().init_colors(); self.set_color(m.RED)
        actual = Authored('x', font_size=24, fill_color=m.GREEN)
        self.assertEqual(events, [('data','x',24.), 'points', ('expression','x'), 'uniforms','colors'])
        expected = native_tex('x',font_size=24,color=m.RED).shift(m.RIGHT)
        assert_family(self,actual,expected)

    def test_custom_records_root_geometry_and_decoration_survive_rebuild(self):
        obj = Decorated('xy'); marker = obj.marker
        fields = obj.data.dtype
        self.assertIn(marker, obj.submobjects)
        self.assertEqual(obj.get_num_points(),3)
        np.testing.assert_array_equal(obj.data['weight'],5)
        obj.tex_string='z';obj.init_points()
        self.assertEqual(obj.data.dtype,fields)
        self.assertEqual(len(obj.submobjects),2)
        self.assertIs(obj.submobjects[0],marker)
        np.testing.assert_array_equal(obj.data['weight'],5)
        self.assertEqual(obj._string_sub_paths,[[1]])

    def test_replace_point_hook_can_avoid_typesetting(self):
        class Authored(SingleStringTex):
            def get_modified_expression(self, text):
                raise AssertionError('replacement hook must not typeset')
            def init_points(self):
                self.set_points_as_corners([m.LEFT,m.UP,m.RIGHT])
        actual=Authored('not evaluated')
        self.assertEqual(actual.get_num_points(),5)
        self.assertEqual(len(actual.submobjects),0)

    def test_live_rebuild_uses_the_current_public_normalizer(self):
        obj=SingleStringTex('x');calls=[]
        def normalize(text):calls.append(text);return 'y^2'
        obj.get_modified_expression=normalize
        scene=m.Scene();scene.add(obj)
        obj.tex_string='recipe';obj.set_color(m.BLUE)
        self.assertIs(obj.init_points(),obj)
        expected=native_tex('y^2',color=m.BLUE)
        assert_family(self,obj,expected)
        self.assertEqual(calls,['recipe'])
        self.assertEqual(obj.get_tex(),'recipe')
        self.assertIn(obj,scene.mobjects)

    def test_copy_deepcopy_pickle_replace_owned_glyphs_once(self):
        original=Decorated('abc');before=original.get_all_points().copy()
        for obj in (original.copy(),copy.deepcopy(original),pickle.loads(pickle.dumps(original))):
            with self.subTest(copy=obj):
                marker=obj.marker
                for source,count in [('x',1),('yz',2),('x',1)]:
                    obj.tex_string=source;obj.init_points()
                    self.assertEqual(len(obj),count+1)
                    self.assertIs(obj.submobjects[0],marker)
                    self.assertEqual(tuple(obj._fmn_string_children),tuple(obj.submobjects)[1:])
                np.testing.assert_array_equal(original.get_all_points(),before)

    def test_failed_normalizer_preserves_live_family_and_span_map(self):
        obj=SingleStringTex('abc');before=list(obj.submobjects)
        spans=list(obj._string_sub_spans);points=obj.get_all_points().copy()
        error=RuntimeError('authored expression failed')
        def normalize(text):raise error
        obj.get_modified_expression=normalize
        with self.assertRaises(RuntimeError) as caught:obj.init_points()
        self.assertIs(caught.exception,error)
        self.assertEqual(list(obj.submobjects),before)
        self.assertEqual(obj._string_sub_spans,spans)
        np.testing.assert_array_equal(obj.get_all_points(),points)
        del obj.get_modified_expression
        obj.tex_string='x';obj.init_points();self.assertEqual(len(obj),1)

    def test_typesetter_failure_keeps_prior_family(self):
        obj=SingleStringTex('x');children=list(obj.submobjects)
        obj.get_modified_expression=lambda text:r'\definitelyUnsupportedNativeCommand'
        with self.assertRaises(m._TexError):obj.init_points()
        self.assertEqual(list(obj.submobjects),children)
        del obj.get_modified_expression;obj.init_points();self.assertEqual(len(obj),1)

    def test_mutated_recipe_in_callback_is_not_silently_consumed(self):
        obj=SingleStringTex('x');children=list(obj.submobjects)
        def normalize(text):obj.font_size=72;return 'y'
        obj.get_modified_expression=normalize
        with self.assertRaisesRegex(RuntimeError,'changed during typesetting'):obj.init_points()
        self.assertEqual(obj.font_size,72)
        self.assertEqual(list(obj.submobjects),children)

    def test_height_and_organization_dispatch_after_colors(self):
        events=[]
        class Authored(SingleStringTex):
            def init_colors(self):events.append('colors');return super().init_colors()
            def set_height(self, height, *args, **kwargs):
                events.append(('height',height));return super().set_height(height,*args,**kwargs)
            def organize_submobjects_left_to_right(self):
                events.append('organize');return super().organize_submobjects_left_to_right()
        obj=Authored('xyz',height=.75,organize_left_to_right=True)
        self.assertEqual(events,['colors',('height',.75),'organize'])
        self.assertAlmostEqual(obj.get_height(),.75,places=6)

    def test_failures_stop_later_hooks_and_release_guard(self):
        error=RuntimeError('points');events=[]
        class Authored(SingleStringTex):
            def init_points(self):events.append('points');raise error
            def init_uniforms(self):events.append('uniforms')
        with self.assertRaises(RuntimeError) as caught:Authored('x')
        self.assertIs(caught.exception,error);self.assertEqual(events,['points'])
        self.assertEqual(len(SingleStringTex('x')),1)

    def test_invalid_options_and_budget_refuse_before_hooks(self):
        calls=[]
        class Authored(SingleStringTex):
            def init_points(self):calls.append(1);super().init_points()
        for options in ({'font_size':0},{'height':float('nan')},{'fill_opacity':float('inf')},
                        {'template':'external-latex-template'},{'unknown':True}):
            with self.subTest(options=options),self.assertRaises((ValueError,TypeError)):
                Authored('x',**options)
        with self.assertRaises(ValueError):Authored('é'*131073)
        self.assertEqual(calls,[])

    def test_reentry_and_animation_locks_refuse(self):
        obj=SingleStringTex('x');children=list(obj.submobjects)
        obj.get_modified_expression=lambda text:obj.init_points()
        with self.assertRaisesRegex(RuntimeError,'already in progress'):obj.init_points()
        del obj.get_modified_expression
        obj.locked_data_keys.add('point')
        with self.assertRaisesRegex(RuntimeError,'active animation'):obj.init_points()
        obj.locked_data_keys.clear()
        self.assertEqual(list(obj.submobjects),children)
        scene=m.Scene();scene.add(obj)
        with self.assertRaisesRegex(RuntimeError,'detached target'):obj.__init__('y')

    def test_invalid_normalized_expression_refuses_before_publication(self):
        obj=SingleStringTex('x');children=list(obj.submobjects)
        for answer in (None,'a'*262145):
            obj.get_modified_expression=lambda text:answer
            with self.subTest(answer=type(answer)),self.assertRaises((TypeError,ValueError)):obj.init_points()
            self.assertEqual(list(obj.submobjects),children)

    def test_existing_expression_helpers_and_external_pipeline_refusal(self):
        obj=SingleStringTex('x')
        self.assertEqual(obj.get_modified_expression(''),r'\quad')
        self.assertEqual(obj.get_modified_expression('x_{'),'x_{}')
        self.assertIn('x',obj.get_tex_file_body('x'))
        with self.assertRaisesRegex(NotImplementedError,'latex_to_svg'):
            obj.get_svg_string_by_content('x')

    def test_collectable_generated_family_and_copy(self):
        def construct():
            obj=SingleStringTex('xy');dup=obj.copy();dup.init_points()
            return weakref.ref(obj),weakref.ref(dup),weakref.ref(dup.submobjects[0])
        refs=construct();gc.collect()
        self.assertTrue(all(ref() is None for ref in refs))

    def test_builder_and_changing_frames_match_independent_native_glyphs(self):
        class Authored(SingleStringTex):
            def init_points(self):super().init_points();self.shift(.25*m.UP)
        def render(path,custom,threads):
            scene=m.Scene()
            with scene.render_session(path,format='png_sequence',resolution=(96,54),fps=4,threads=threads):
                obj=Authored('xy',fill_opacity=.5) if custom else native_tex('xy',fill_opacity=.5).shift(.25*m.UP)
                scene.add(obj);scene.play(obj.animate.shift(m.RIGHT),run_time=.5,rate_func=m.linear)
            return [p.read_bytes() for p in sorted(path.glob('*.png'))]
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);first=render(root/'one',True,1)
            self.assertEqual(first,render(root/'four',True,4))
            self.assertEqual(first,render(root/'native',False,1))
            self.assertEqual(len(first),2);self.assertNotEqual(first[0],first[1])


if __name__=='__main__':
    unittest.main()
