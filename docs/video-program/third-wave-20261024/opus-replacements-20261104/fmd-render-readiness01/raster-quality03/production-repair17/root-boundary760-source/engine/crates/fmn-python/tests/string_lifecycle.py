"""Native text/TeX subclass hooks, persistent schemas and live source spans."""
from pathlib import Path
import tempfile
import inspect
import copy
import pickle
import unittest

import numpy as np
import manimlib as m


class StringLifecycleTests(unittest.TestCase):
    def test_all_string_front_doors_dispatch_hooks_once(self):
        for base, text in ((m.Text,'abc'),(m.MarkupText,'<b>abc</b>'),(m.Code,'x = 1'),
                           (m.Tex,'x^2'),(m.TexText,'abc')):
            with self.subTest(base=base.__name__):
                class Custom(base):
                    def init_data(self):
                        self.events=['data'];self.source_at_init=self.string
                        super().init_data()
                    def init_points(self):self.events.append('points');super().init_points()
                    def init_uniforms(self):self.events.append('uniforms');super().init_uniforms()
                    def init_colors(self):self.events.append('colors');super().init_colors()
                obj=Custom(text)
                self.assertEqual(obj.events,['data','points','uniforms','colors'])
                self.assertEqual(obj.source_at_init,text)
                self.assertGreater(len(obj.family_members_with_points()),0)

    def test_custom_schema_decoration_and_span_paths_survive(self):
        for base, text in ((m.Text,'é + é'),(m.Tex,'x + x')):
            with self.subTest(base=base.__name__):
                class Custom(base):
                    data_dtype=m.VMobject.data_dtype+[('custom_lane',1)]
                    def init_data(self):
                        super().init_data();self.decoration=m.Dot().shift(m.DOWN)
                        self.add(self.decoration)
                    def init_points(self):
                        super().init_points()
                        self.set_points_as_corners([m.LEFT,m.RIGHT])
                        self.data['custom_lane'][:]=9
                    def init_uniforms(self):
                        super().init_uniforms();self.uniforms['is_fixed_in_frame']=1
                obj=Custom(text)
                self.assertIn(obj.decoration,obj.submobjects)
                np.testing.assert_array_equal(obj.data['custom_lane'],9)
                self.assertEqual(obj.uniforms['is_fixed_in_frame'],1)
                self.assertTrue(all(path[0]>0 for path in obj._string_sub_paths))
                matches=obj.select_parts('é' if base is m.Text else 'x')
                self.assertEqual(len(matches),2)
                self.assertNotIn(obj.decoration,matches.get_family())
                copied=obj.copy()
                self.assertIsNot(copied.decoration,obj.decoration)
                self.assertIn(copied.decoration,copied.submobjects)
                self.assertEqual(len(copied.select_parts('é' if base is m.Text else 'x')),2)

    def test_custom_init_points_controls_rendered_glyph_geometry(self):
        for base in (m.Text,m.Tex):
            class Raised(base):
                def init_points(self):
                    super().init_points();self.stretch(2,1,about_point=m.ORIGIN)
            plain=base('abc',should_center=False)
            custom=Raised('abc',should_center=False)
            self.assertAlmostEqual(custom.get_height(),2*plain.get_height(),places=6)
            self.assertAlmostEqual(custom.get_width(),plain.get_width(),places=6)

    def test_custom_init_colors_not_overwritten_by_default_style(self):
        for base in (m.Text,m.Tex):
            class Custom(base):
                def init_colors(self):super().init_colors();self.set_color(m.GREEN)
            obj=Custom('ab')
            self.assertTrue(all(part.get_fill_color()==m.GREEN for part in obj.family_members_with_points()))

    def test_native_markup_colors_and_code_highlighting_survive(self):
        text=m.MarkupText('<span foreground="#FC6255">A</span>B')
        self.assertEqual(text.select_part('A').family_members_with_points()[0].get_fill_color(),m.RED)
        self.assertNotEqual(text.select_part('B').family_members_with_points()[0].get_fill_color(),m.RED)
        code=m.Code('def f(x):\n    return 42')
        colors={tuple(part.data['fill_rgba'][0]) for part in code.family_members_with_points()}
        self.assertGreater(len(colors),1)

    def test_explicit_style_and_source_maps_win_over_native_defaults(self):
        text=m.Text('ab',fill_color=m.GREEN,t2c={'b':m.RED},stroke_width=2,stroke_color=m.BLUE)
        self.assertEqual(text.select_part('a').family_members_with_points()[0].get_fill_color(),m.GREEN)
        self.assertEqual(text.select_part('b').family_members_with_points()[0].get_fill_color(),m.RED)
        self.assertAlmostEqual(text.select_part('a').family_members_with_points()[0].get_stroke_width(),2)
        self.assertEqual(text.select_part('a').family_members_with_points()[0].get_stroke_color(),m.BLUE)
        tex=m.Tex('x+y',tex_to_color_map={'x':m.RED},fill_color=m.GREEN)
        self.assertEqual(tex.select_part('x').family_members_with_points()[0].get_fill_color(),m.RED)
        self.assertEqual(tex.select_part('y').family_members_with_points()[0].get_fill_color(),m.GREEN)

    def test_flat_tex_span_paths_skip_hook_decorations(self):
        class Custom(m.Tex):
            def init_data(self):super().init_data();self.dot=m.Dot();self.add(self.dot)
        tex=Custom('x','+','y',tex_to_color_map={'x':m.RED,'y':m.GREEN})
        self.assertIn(tex.dot,tex.submobjects)
        self.assertEqual(tex.select_part('x').family_members_with_points()[0].get_fill_color(),m.RED)
        self.assertEqual(tex.select_part('y').family_members_with_points()[0].get_fill_color(),m.GREEN)
        self.assertNotIn(tex.dot,tex.select_parts('x').get_family())
        self.assertTrue(all(len(path)==1 and path[0]>0 for path in tex._string_sub_paths))
        self.assertTrue(all(child.has_points() for child in tex._fmn_string_children))

    def test_reinitialization_replaces_only_native_glyph_children(self):
        class Custom(m.Text):
            def init_data(self):super().init_data();self.dot=m.Dot();self.add(self.dot)
        text=Custom('ab');old=tuple(text._fmn_string_children)
        decoration=m.Square();text.add(decoration)
        text.text='xyz';text.init_points()
        self.assertIn(text.dot,text.submobjects);self.assertIn(decoration,text.submobjects)
        self.assertFalse(any(child in text.submobjects for child in old))
        self.assertEqual(len(text.select_parts('x')),1)
        self.assertTrue(all(path[0]>=2 for path in text._string_sub_paths))
        scene=m.Scene();scene.add(text);text.init_points()
        self.assertIn(text,scene.mobjects)
        self.assertEqual(len(text.select_parts('x')),1)

    def test_tex_signature_and_code_regeneration_keep_public_contracts(self):
        signature=inspect.signature(m.Tex)
        self.assertEqual(signature.parameters['tex_strings'].annotation,'str')
        self.assertEqual(signature.parameters['font_size'].default,48)
        code=m.Code('return 1')
        code.init_points()
        self.assertGreater(len(code.family_members_with_points()),0)
        self.assertEqual(code.font,'Consolas')

    def test_custom_hook_failure_propagates_once(self):
        for hook in ('init_data','init_points','init_uniforms','init_colors'):
            calls=[];failure=RuntimeError(hook)
            def broken(self):calls.append(hook);raise failure
            cls=type('Broken',(m.Text,),{hook:broken})
            with self.subTest(hook=hook),self.assertRaises(RuntimeError) as caught:cls('ab')
            self.assertIs(caught.exception,failure);self.assertEqual(calls,[hook])

    def test_native_typesetter_failure_preserves_existing_family(self):
        text=m.Text('ab');old=list(text.submobjects);spans=list(text._string_sub_spans)
        text.font='not-a-bundled-font-at-all'
        with self.assertRaises(Exception):text.init_points()
        self.assertEqual(list(text.submobjects),old);self.assertEqual(text._string_sub_spans,spans)

    def test_full_override_can_supply_geometry_without_a_span_map(self):
        class Custom(m.Text):
            def init_points(self):self.add(m.Square(fill_opacity=1,stroke_width=0))
        obj=Custom('not typeset')
        self.assertEqual(obj._string_sub_paths,[])
        self.assertAlmostEqual(obj.get_width(),2)
        self.assertEqual(len(obj.select_parts('not')),0)

    def test_matching_transform_uses_real_custom_text(self):
        class Custom(m.Text):
            def init_points(self):super().init_points();self.stretch(2,1)
        source=Custom('x+y');target=Custom('y+x').shift(m.RIGHT)
        scene=m.Scene();scene.add(source)
        scene.play(m.TransformMatchingStrings(source,target),run_time=2/30)
        self.assertIn(target,scene.mobjects);self.assertNotIn(source,scene.mobjects)
        self.assertEqual(len(target.select_parts('x')),1)

    def test_native_glyph_frames_match_literal_posttransform_and_threads(self):
        def render(path,base,custom,threads):
            scene=m.Scene()
            with scene.render_session(path,format='png_sequence',resolution=(120,68),fps=4,threads=threads):
                if custom:
                    class Custom(base):
                        def init_points(self):super().init_points();self.stretch(1.5,1,about_point=m.ORIGIN)
                    text=Custom('x+y',should_center=False)
                else:
                    text=base('x+y',should_center=False).stretch(1.5,1,about_point=m.ORIGIN)
                scene.add(text);scene.play(text.animate.shift(m.RIGHT),run_time=.5,rate_func=m.linear)
            return [p.read_bytes() for p in sorted(path.glob('*.png'))]
        root=Path(tempfile.mkdtemp(prefix='fmn-string-lifecycle-'))
        for base in (m.Text,m.Tex):
            with self.subTest(base=base.__name__):
                first=render(root/(base.__name__+'one'),base,True,1)
                self.assertEqual(first,render(root/(base.__name__+'four'),base,True,4))
                self.assertEqual(first,render(root/(base.__name__+'expected'),base,False,1))
                self.assertEqual(len(first),2);self.assertNotEqual(first[0],first[1])


FRONT_DOORS = ((m.Text, 'xy'), (m.MarkupText, '<b>xy</b>'),
               (m.Code, 'x = 1'), (m.Tex, 'x+y'), (m.TexText, 'xy'))


def assert_paints(test, obj, fill, stroke):
    members = obj.family_members_with_points()
    test.assertTrue(members, 'the paint assertion requires real native ink')
    for member in members:
        test.assertEqual(member.get_fill_color(), fill)
        test.assertEqual(member.get_stroke_color(), stroke)


class StringRootStyle(unittest.TestCase):
    def test_point_less_roots_report_the_reference_root_style(self):
        # fm-78b6: the Reference's StringMobject root takes no fill opacity
        # and a zero stroke width; the glyphs carry the paint.
        for base, text in FRONT_DOORS:
            with self.subTest(base=base.__name__):
                obj = base(text)
                self.assertFalse(obj.has_points())
                self.assertEqual(obj.get_fill_opacity(), 0.0)
                self.assertEqual(obj.get_stroke_width(), 0.0)
                self.assertEqual(obj.family_members_with_points()[0].get_fill_opacity(), 1.0)

    def test_caller_styles_still_reach_the_root(self):
        self.assertEqual(m.Tex('ab', fill_opacity=0.5).get_fill_opacity(), 0.5)
        self.assertEqual(m.Text('ab', stroke_width=2).get_stroke_width(), 2.0)


class StringLeafClasses(unittest.TestCase):
    # fm-15k9: the Reference builds strings through SVGMobject, so each glyph
    # is a VMobjectFromSVGPath and each TeX rule is dvisvgm's <rect>, a
    # Rectangle. Native storage is unchanged; only the leaf class moves.
    def test_glyphs_are_svg_paths_at_every_front_door(self):
        for base, text in FRONT_DOORS:
            with self.subTest(base=base.__name__):
                leaves = base(text).family_members_with_points()
                self.assertTrue(leaves)
                for leaf in leaves:
                    self.assertIs(type(leaf), m.VMobjectFromSVGPath)
                    self.assertEqual(type(leaf).__mro__[1:3], (m.VMobject, m.Mobject))

    def test_a_fraction_bar_is_a_rectangle_with_the_reference_vertices(self):
        tex = m.Tex(r'\frac{1}{2}')
        self.assertEqual([type(leaf).__name__ for leaf in tex],
                         ['VMobjectFromSVGPath', 'Rectangle', 'VMobjectFromSVGPath'])
        bar = tex[1]
        (x0, y0, _), (x1, y1, _) = bar.get_bounding_box()[[0, 2]]
        # dvisvgm's <rect> as the Reference's Rectangle reports it: BR, TR, TL, BL.
        np.testing.assert_allclose(bar.get_vertices(), [[x1, y0, 0], [x1, y1, 0],
                                                        [x0, y1, 0], [x0, y0, 0]], atol=1e-6)
        self.assertAlmostEqual(bar.width, x1 - x0)
        self.assertAlmostEqual(bar.height, y1 - y0)

    def test_classes_survive_copy_pickle_and_regeneration(self):
        tex = m.Tex(r'\sqrt{x} = \frac{a}{b}')
        expected = [type(leaf) for leaf in tex]
        self.assertIn(m.Rectangle, expected)
        for twin in (tex.copy(), copy.deepcopy(tex), pickle.loads(pickle.dumps(tex))):
            self.assertEqual([type(leaf) for leaf in twin], expected)
        text = m.Text('ab')
        text.become(m.Text('cd'))
        self.assertTrue(all(type(leaf) is m.VMobjectFromSVGPath for leaf in text))


class TextChannelColors(unittest.TestCase):
    def test_explicit_channels_win_independently_at_every_text_front_door(self):
        cases = ((dict(fill_color=m.BLUE), m.BLUE, m.RED),
                 (dict(stroke_color=m.GREEN), m.RED, m.GREEN),
                 (dict(fill_color=m.BLUE, stroke_color=m.GREEN), m.BLUE, m.GREEN),
                 (dict(fill_color=None, stroke_color=m.GREEN), m.RED, m.GREEN),
                 (dict(fill_color=m.BLUE, stroke_color=None), m.BLUE, m.RED))
        for cls, text in FRONT_DOORS:
            for channels, fill, stroke in cases:
                with self.subTest(cls=cls.__name__, channels=channels):
                    obj = cls(text, color=m.RED, stroke_width=2, **channels)
                    self.assertEqual(obj.fill_color, fill)
                    self.assertEqual(obj.stroke_color, stroke)
                    assert_paints(self, obj, fill, stroke)

    def test_color_shorthand_still_overrides_native_and_base_defaults(self):
        for cls, text in FRONT_DOORS:
            with self.subTest(cls=cls.__name__):
                obj = cls(text, color=m.RED, base_color=m.GREEN, stroke_width=2)
                assert_paints(self, obj, m.RED, m.RED)

    def test_none_shorthand_does_not_erase_explicit_channels(self):
        for cls, text in FRONT_DOORS:
            for extra in ({}, {'color': None}):
                with self.subTest(cls=cls.__name__, extra=extra):
                    obj = cls(text, fill_color=m.BLUE, stroke_color=m.GREEN,
                              stroke_width=2, **extra)
                    assert_paints(self, obj, m.BLUE, m.GREEN)

    def test_source_color_maps_still_win_after_channel_resolution(self):
        for cls in (m.Text, m.Tex):
            with self.subTest(cls=cls.__name__):
                obj = cls('xy', color=m.RED, fill_color=m.BLUE,
                          stroke_color=m.GREEN, stroke_width=2, t2c={'x': m.YELLOW})
                assert_paints(self, obj.select_part('x'), m.YELLOW, m.YELLOW)
                assert_paints(self, obj.select_part('y'), m.BLUE, m.GREEN)

    def test_unstyled_markup_and_highlighting_are_not_repainted(self):
        markup = m.MarkupText('<span foreground="#FC6255">x</span>y')
        self.assertEqual(markup.select_part('x').family_members_with_points()[0].get_fill_color(), m.RED)
        self.assertNotEqual(markup.select_part('y').family_members_with_points()[0].get_fill_color(), m.RED)
        code = m.Code('def f(x):\n    return 42')
        colors = {part.get_fill_color() for part in code.family_members_with_points()}
        self.assertGreater(len(colors), 1)

    def test_hooks_observe_resolved_channels_and_can_override_them(self):
        for base in (m.Text, m.Tex):
            events = []
            class Authored(base):
                def init_data(self):
                    events.append(('data', self.fill_color, self.stroke_color))
                    super().init_data()
                def init_colors(self):
                    events.append('colors')
                    super().init_colors()
                    self.set_stroke(color=m.YELLOW)
            with self.subTest(base=base.__name__):
                obj = Authored('xy', color=m.RED, fill_color=m.BLUE,
                               stroke_color=m.GREEN, stroke_width=2)
                self.assertEqual(events, [('data', m.BLUE, m.GREEN), 'colors'])
                assert_paints(self, obj, m.BLUE, m.YELLOW)

    def test_caller_options_and_rgb_arrays_are_not_mutated(self):
        fill = np.array([[.1, .2, .8]])
        shorthand = np.array([[.8, .1, .2]])
        kwargs = dict(color=shorthand, fill_color=fill, stroke_color=m.GREEN, stroke_width=2)
        before = {key: value.copy() if isinstance(value, np.ndarray) else value
                  for key, value in kwargs.items()}
        obj = m.Text('x', **kwargs)
        # VMobject itself already implements channel > shorthand. It is an
        # independent native paint oracle, not the changed string initializer.
        expected = m.VMobject(**kwargs)
        expected.set_points_as_corners([m.LEFT, m.RIGHT])
        assert_paints(self, obj, expected.get_fill_color(), expected.get_stroke_color())
        for key, value in before.items():
            np.testing.assert_array_equal(kwargs[key], value)

    def test_empty_text_retains_resolved_defaults_for_later_points(self):
        obj = m.Text('', color=m.RED, fill_color=m.BLUE,
                     stroke_color=m.GREEN, stroke_width=2)
        self.assertFalse(obj.has_points())
        obj.set_points_as_corners([m.LEFT, m.RIGHT])
        assert_paints(self, obj, m.BLUE, m.GREEN)

    def test_copies_and_replayed_color_hooks_keep_the_resolved_recipe(self):
        for cls in (m.Text, m.Tex):
            original = cls('xy', color=m.RED, fill_color=m.BLUE,
                           stroke_color=m.GREEN, stroke_width=2)
            for duplicate in (original.copy(), copy.deepcopy(original)):
                with self.subTest(cls=cls.__name__, copy=id(duplicate)):
                    duplicate.set_color(m.YELLOW)
                    duplicate.init_colors()
                    assert_paints(self, duplicate, m.BLUE, m.GREEN)
                    assert_paints(self, original, m.BLUE, m.GREEN)
                    self.assertIsNot(duplicate.submobjects[0], original.submobjects[0])

    def test_live_rebuild_then_color_hook_preserves_custom_records_and_children(self):
        class Authored(m.Text):
            data_dtype = m.VMobject.data_dtype + [('weight', 1)]
            def init_data(self):
                super().init_data()
                self.marker = m.Dot().shift(m.DOWN)
                self.add(self.marker)
        obj = Authored('xy', color=m.RED, fill_color=m.BLUE,
                       stroke_color=m.GREEN, stroke_width=2)
        scene = m.Scene().add(obj)
        marker = obj.marker
        obj.text = 'xyz'
        obj.init_points(); obj.init_colors()
        self.assertIs(obj.marker, marker)
        self.assertIn(marker, obj.submobjects)
        self.assertIn('weight', obj.data.dtype.names)
        self.assertIn(obj, scene.mobjects)
        assert_paints(self, obj, m.BLUE, m.GREEN)

    def test_transform_uses_actual_native_channel_endpoints(self):
        source = m.Text('xy', color=m.RED, fill_color=m.BLUE,
                        stroke_color=m.GREEN, stroke_width=2)
        target = m.Text('xy', color=m.RED, fill_color=m.GREEN,
                        stroke_color=m.YELLOW, stroke_width=2).shift(m.RIGHT)
        scene = m.Scene().add(source)
        scene.play(m.Transform(source, target), run_time=2/30, rate_func=m.linear)
        assert_paints(self, source, m.GREEN, m.YELLOW)

    def test_rendered_channel_styles_match_independent_postconstruction_paints(self):
        def render(path, cls, kind, workers):
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(96, 54),
                                      fps=8, threads=workers):
                if kind == 'authored':
                    obj = cls('xy', color=m.RED, fill_color=m.BLUE, stroke_color=m.GREEN,
                              stroke_width=3, fill_opacity=.7, stroke_opacity=.8)
                else:
                    obj = cls('xy', color=m.RED, stroke_width=3,
                              fill_opacity=.7, stroke_opacity=.8)
                    if kind == 'control':
                        obj.set_fill(color=m.BLUE)
                        obj.set_stroke(color=m.GREEN)
                scene.add(obj)
                scene.play(obj.animate.shift(m.RIGHT), run_time=.375, rate_func=m.linear)
            return [p.read_bytes() for p in sorted(path.glob('*.png'))]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for cls in (m.Text, m.Tex):
                control = render(root/(cls.__name__+'-control'), cls, 'control', 1)
                self.assertEqual(len(control), 3)
                self.assertNotEqual(control[0], control[-1])
                negative = render(root/(cls.__name__+'-negative'), cls, 'negative', 1)
                self.assertNotEqual(control, negative)
                for workers in (1, 4, 16):
                    actual = render(root/(cls.__name__+str(workers)), cls, 'authored', workers)
                    self.assertEqual(actual, control)


# fm-4qmn: Reference Tex/StringMobject expose SVG glyphs directly. The
# existing native builder's one-source form is an independent layout oracle;
# it does not execute Tex.__init__ or this adapter's init_points.
def native_tex_control(source, *, text_mode=False, font_size=48, preamble=''):
    obj = m._native_shell_factory()
    specs = obj._build_tex(m._native_shell_factory, [source], '', text_mode,
                          font_size, None, False, '', preamble,
                          'center' if text_mode else 'left')
    m._hang_native_children(obj, specs)
    obj.center()
    return obj


class ModernTexFamilies(unittest.TestCase):
    def assert_flat_layout(self, obj, control):
        self.assertFalse(obj.has_points())
        self.assertEqual(len(obj), len(control.submobjects))
        self.assertEqual(list(obj._string_sub_spans), list(control._string_sub_spans))
        self.assertEqual(obj._string_sub_paths, control._string_sub_paths)
        for actual, expected in zip(obj.submobjects, control.submobjects):
            self.assertTrue(actual.has_points(), 'an argument wrapper escaped into the public family')
            self.assertEqual(list(actual.submobjects), [])
            np.testing.assert_array_equal(actual.get_points(), expected.get_points())

    def test_multiargument_tex_exposes_native_glyphs_not_argument_containers(self):
        for parts in [('a', '+', 'b'), ('ab', '+', 'cd'), ('', 'x', '', '+y', ''),
                      (r'\frac{x}{2}', '=', 'y'), ('x^2', '+', 'y_1')]:
            with self.subTest(parts=parts):
                obj = m.Tex(*parts)
                self.assert_flat_layout(obj, native_tex_control(obj.string))
                self.assertEqual(obj.tex_strings, [p.lstrip() if i == 0 else p.rstrip()
                                                  if i == len(parts)-1 else p
                                                  for i, p in enumerate(parts)])

    def test_text_mode_keeps_words_math_islands_and_utf8_spans(self):
        for parts in [('é', 'plus', '$x^2$'), ('alpha', '+', 'beta'),
                      ('sum:', r'$\frac{1}{2}$', 'end')]:
            with self.subTest(parts=parts):
                obj = m.TexText(*parts)
                self.assert_flat_layout(obj, native_tex_control(obj.string, text_mode=True))
                payload = obj.string.encode('utf-8')
                for start, end in obj._string_sub_spans:
                    payload[:start].decode('utf-8'); payload[:end].decode('utf-8')

    def test_isolate_is_source_selection_not_family_partition(self):
        import re
        source = 'x^2 + y^2'
        for selector in (['x', 'y'], 'x', re.compile('[xy]'), (0, 3), ['x^2', 'y^2']):
            with self.subTest(selector=selector):
                obj = m.Tex(source, isolate=selector)
                self.assert_flat_layout(obj, native_tex_control(source))
                self.assertEqual(len(obj), 5)
                self.assertIs(obj.select_part('x')[0], obj[0])
                self.assertIs(obj.select_part('y')[0], obj[3])
                obj[1].set_color(m.YELLOW)
                self.assertEqual(obj.select_part('2')[0].get_fill_color(), m.YELLOW)
                self.assertNotEqual(obj[4].get_fill_color(), m.YELLOW)

    def test_logical_parts_and_slices_reference_existing_glyphs(self):
        import re
        obj = m.Tex('ab', '+', 'ab')
        self.assertEqual(len(obj), 5)
        for selector in ('ab', re.compile('ab')):
            parts = obj.select_parts(selector)
            self.assertEqual(len(parts), 2)
            self.assertEqual(list(parts[0]), [obj[0], obj[1]])
            self.assertEqual(list(parts[1]), [obj[3], obj[4]])
        self.assertEqual(list(obj.select_part((1, 4))), [obj[1], obj[2]])
        self.assertEqual(list(obj[1:4]), [obj[1], obj[2], obj[3]])
        obj.get_part_by_tex('ab', index=1).shift(m.UP)
        self.assertGreater(obj[3].get_y(), obj[0].get_y())

    def test_source_color_maps_style_the_right_flat_members(self):
        obj = m.Tex('x', '+', 'y', color=m.BLUE,
                    stroke_color=m.GREEN, stroke_width=2, t2c={'y': m.RED})
        self.assertEqual(len(obj), 3)
        self.assertEqual(obj[0].get_fill_color(), m.BLUE)
        self.assertEqual(obj[1].get_stroke_color(), m.GREEN)
        self.assertEqual(obj[2].get_fill_color(), m.RED)
        self.assertEqual(obj[2].get_stroke_color(), m.RED)

    def test_custom_separator_and_macro_arguments_keep_original_source(self):
        class Joined(m.Tex):
            _tex_arg_separator = ''
        preamble = r'\newcommand{\sq}[1]{#1^2}'
        obj = Joined(r'\sq{', 'x', '}+', 'y', additional_preamble=preamble)
        self.assertEqual(obj.string, r'\sq{x}+y')
        self.assert_flat_layout(obj, native_tex_control(obj.string, preamble=preamble))
        self.assertEqual(len(obj.select_part('x')), 1)
        self.assertIs(obj.select_part('x')[0], obj[0])

    def test_legacy_argument_groups_remain_intentionally_distinct(self):
        from manimlib.mobject.svg.old_tex_mobject import OldTex, OldTexText, SingleStringTex
        for cls in (OldTex, OldTexText):
            with self.subTest(cls=cls.__name__):
                obj = cls('ab', '+', 'cd')
                self.assertEqual(len(obj), 3)
                self.assertTrue(all(not part.has_points() for part in obj))
                self.assertEqual([len(part) for part in obj], [2, 1, 2])
                self.assertTrue(all(len(path) == 2 for path in obj._string_sub_paths))
                duplicate = obj.copy()
                self.assertEqual([len(part) for part in duplicate], [2, 1, 2])
                self.assertIsNot(duplicate[0][0], obj[0][0])
        single = SingleStringTex('ab')
        self.assertEqual(len(single), 2)
        self.assertTrue(all(child.has_points() for child in single))

    def test_infix_fraction_indices_survive_separate_arguments(self):
        obj = m.Tex('12', r'\over', '34')
        self.assertEqual(len(obj), 5)
        numerator = obj.make_number_changeable('12')
        denominator = obj.make_number_changeable('34')
        self.assertEqual(len(obj), 3)
        self.assertIs(obj[0], numerator)
        self.assertIs(obj[2], denominator)
        duplicate = obj.copy()
        duplicate[0].set_value(7); duplicate[2].set_value(9)
        self.assertEqual((obj[0].get_value(), obj[2].get_value()), (12, 34))
        self.assertEqual((duplicate[0].get_value(), duplicate[2].get_value()), (7, 9))

    def test_copies_pickle_and_targets_keep_flat_family_and_span_identity(self):
        import pickle
        original = m.Tex('ab', '+', 'cd')
        for duplicate in (original.copy(), copy.deepcopy(original),
                          pickle.loads(pickle.dumps(original)), original.generate_target()):
            with self.subTest(kind=type(duplicate).__name__):
                self.assert_flat_layout(duplicate, native_tex_control(original.string))
                self.assertIs(duplicate.select_part('cd')[0], duplicate[3])
                self.assertIsNot(duplicate[3], original[3])
                duplicate[3].shift(m.UP)
                self.assertNotEqual(duplicate[3].get_y(), original[3].get_y())

    def test_authored_hooks_observe_flat_glyphs_and_keep_custom_data(self):
        class Authored(m.Tex):
            data_dtype = m.VMobject.data_dtype + [('weight', 1)]
            def init_data(self):
                super().init_data()
                self.marker = m.Dot().shift(m.DOWN)
                self.add(self.marker)
            def init_points(self):
                super().init_points()
                self.observed = [child.has_points() for child in self._fmn_string_children]
                self.set_points_as_corners([m.LEFT, m.RIGHT])
                self.data['weight'][:] = 7
        obj = Authored('ab', '+', 'cd')
        self.assertEqual(obj.observed, [True] * 5)
        self.assertEqual(len(obj.submobjects), 6)
        self.assertIs(obj.select_part('ab')[0], obj[1])
        np.testing.assert_array_equal(obj.data['weight'], 7)
        scene = m.Scene().add(obj)
        view = obj.data['weight']
        obj[2].shift(m.UP)
        np.testing.assert_array_equal(view, 7)
        self.assertIn(obj, scene.mobjects)
        self.assertIn(obj.marker, obj.submobjects)

    def test_regeneration_replaces_only_generated_members_in_either_owner_state(self):
        for bound in (False, True):
            with self.subTest(bound=bound):
                obj = m.Tex('ab', '+', 'cd')
                marker = m.Dot().shift(m.DOWN)
                obj.add(marker)
                if bound:
                    scene = m.Scene().add(obj)
                for _ in range(2):
                    old = tuple(obj._fmn_string_children)
                    obj.init_points()
                    self.assertEqual(len(obj.submobjects), 6)
                    self.assertEqual(len(obj._fmn_string_children), 5)
                    self.assertTrue(all(child.has_points() for child in obj._fmn_string_children))
                    self.assertFalse(any(child in obj.submobjects for child in old))
                    self.assertIn(marker, obj.submobjects)
                    self.assertNotIn(marker, obj.select_parts('ab').get_family())
                    self.assertIs(obj.select_part('cd')[0], obj[4])

    def test_native_failure_does_not_publish_half_a_new_family(self):
        obj = m.Tex('x', '+', 'y')
        children = tuple(obj.submobjects)
        paths = [list(path) for path in obj._string_sub_paths]
        obj.tex_strings = [r'\unknownUnimplementedCommand{x}']
        with self.assertRaises(Exception):
            obj.init_points()
        self.assertEqual(tuple(obj.submobjects), children)
        self.assertEqual(obj._string_sub_paths, paths)

    def test_indexed_glyph_animation_matches_native_controls_and_workers(self):
        def render(path, workers, control):
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(96, 54),
                                      fps=8, threads=workers):
                obj = native_tex_control('ab + cd') if control else m.Tex('ab', '+', 'cd')
                obj.scale(2)
                scene.add(obj)
                scene.play(obj[1].animate.shift(m.UP), run_time=.25, rate_func=m.linear)
                scene.play(m.FadeOut(obj[3]), run_time=.25, rate_func=m.linear)
            return [p.read_bytes() for p in sorted(path.glob('*.png'))]
        root = Path(tempfile.mkdtemp(prefix='fmn-flat-tex-'))
        control = render(root/'control', 1, True)
        self.assertEqual(len(control), 4)
        self.assertNotEqual(control[0], control[-1])
        for workers in (1, 4, 16):
            self.assertEqual(render(root/str(workers), workers, False), control)

    def test_matching_transform_reaches_flat_target_and_keeps_selectors_live(self):
        source = m.Tex('ab', '+', 'cd')
        target = m.Tex('cd', '-', 'ab').shift(m.UP)
        scene = m.Scene().add(source)
        scene.play(m.TransformMatchingTex(source, target), run_time=.125)
        self.assertIn(target, scene.mobjects)
        self.assertNotIn(source, scene.mobjects)
        self.assertEqual(len(target), 5)
        self.assertIs(target.select_part('ab')[1], target[4])
        self.assertTrue(all(child.has_points() for child in target))


class StringMobjectAbstractness(unittest.TestCase):
    # The Reference's StringMobject(SVGMobject, ABC) at 6199a00d: the class
    # sweep records StringMobject() -> "TypeError: Can't instantiate abstract
    # class StringMobject without an implementation for abstract methods
    # 'get_attr_dict_from_command_pair', 'get_command_flag', ...".
    STEPS = frozenset((
        "get_attr_dict_from_command_pair", "get_command_flag", "get_command_matches",
        "get_command_string", "get_configured_items", "get_content_prefix_and_suffix",
        "get_svg_string_by_content", "replace_for_content", "replace_for_matching",
    ))

    def test_the_base_refuses_instantiation_like_the_reference(self):
        for args in ((), ("x",)):
            with self.assertRaises(TypeError) as caught:
                m.StringMobject(*args)
            self.assertTrue(str(caught.exception).startswith(
                "Can't instantiate abstract class StringMobject without an "
                "implementation for abstract methods 'get_attr_dict_from_command_pair', "
                "'get_command_flag', 'get_command_matches', 'get_command_string', 'get_c"),
                caught.exception)
        self.assertEqual(m.StringMobject.__abstractmethods__, self.STEPS)

    def test_a_direct_subclass_must_implement_every_step(self):
        class Partial(m.StringMobject):
            @staticmethod
            def get_command_flag(match_obj):
                return 0
        self.assertEqual(Partial.__abstractmethods__, self.STEPS - {"get_command_flag"})
        with self.assertRaises(TypeError):
            Partial("x")

    def test_concrete_string_classes_still_construct(self):
        for build in (lambda: m.Text("x"), lambda: m.MarkupText("x"),
                      lambda: m.Code("x = 1"), lambda: m.Tex("x"), lambda: m.TexText("x")):
            mob = build()
            self.assertIsInstance(mob, m.StringMobject)
            self.assertFalse(type(mob).__abstractmethods__)
            self.assertGreater(len(mob.get_all_points()), 0)
        # The steps keep the portal's by-name render-twice refusal.
        with self.assertRaises(Exception) as caught:
            m.Text("x").get_command_flag(None)
        self.assertIn("get_command_flag", str(caught.exception))


if __name__=='__main__':unittest.main()
