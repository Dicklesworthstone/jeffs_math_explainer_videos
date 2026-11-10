"""Source-unedited SVGMobject rendering, ownership and paint-role acceptance."""
from __future__ import annotations
import copy
import importlib
import inspect
from pathlib import Path
import pickle
import tempfile
import unittest

import numpy as np
import manimlib as m

NESTED = '<svg><path fill="red" fill-rule="evenodd" d="M0 0H10V10H0Z M2 2H8V8H2Z"/></svg>'
DASHED = '<svg><path fill="none" stroke="red" stroke-width="6" stroke-dasharray="2 2" d="M0 0H10"/></svg>'
CLOSED = '<svg><path fill="blue" stroke="red" stroke-width="6" stroke-dasharray="3 2" d="M0 0H10V10H0Z"/></svg>'


def pixels(mob):
    return np.frombuffer(m.Camera(resolution=(128,72)).capture_snapshot(mob).pixels(), dtype=np.uint8).reshape(72,128,4)


def geometry(mob):
    return [np.asarray(obj.get_points()).copy() for obj in mob.family_members_with_points()]


class SvgPaintIngressTests(unittest.TestCase):
    def test_evenodd_and_nonzero_have_distinct_native_pixels(self):
        evenodd=m.SVGMobject(svg_string=NESTED,height=4)
        nonzero=m.SVGMobject(svg_string=NESTED.replace('evenodd','nonzero'),height=4)
        a,b=pixels(evenodd),pixels(nonzero)
        np.testing.assert_array_equal(a[36,64,:3],[0,0,0])
        self.assertGreater(b[36,64,0],200)
        self.assertFalse(np.array_equal(a,b))

    def test_crossing_contours_have_symmetric_difference_not_union(self):
        src='<svg><path fill="white" fill-rule="evenodd" d="M0 0H6V4H0Z M3 0H9V4H3Z"/></svg>'
        actual=m.SVGMobject(svg_string=src,height=4)
        # Analytic result consists of two disjoint 3x4 rectangles.
        expected=m.VGroup(m.Rectangle(width=3,height=4).move_to((-3,0,0)),
                          m.Rectangle(width=3,height=4).move_to((3,0,0)))
        expected.set_fill(m.WHITE,1).set_stroke(width=0)
        np.testing.assert_array_equal(pixels(actual),pixels(expected))

    def test_dashes_render_as_independently_authored_native_lines(self):
        actual=m.SVGMobject(svg_string=DASHED,width=6,stroke_width=None)
        expected=m.VGroup(*(m.Line((a,0,0),(b,0,0),color="#ff0000",stroke_width=6)
                           for a,b in [(-3,-1.8),(-.6,.6),(1.8,3)]))
        np.testing.assert_array_equal(pixels(actual),pixels(expected))

    def test_constructor_channel_overrides_do_not_fill_stroke_fragments(self):
        actual=m.SVGMobject(svg_string=CLOSED,fill_color=m.GREEN,fill_opacity=.3,
                           stroke_opacity=.6,stroke_width=5)
        fill,*strokes=actual[0].submobjects
        self.assertAlmostEqual(fill.get_fill_opacity(),.3,places=6)
        self.assertEqual(fill.get_stroke_opacity(),0)
        self.assertTrue(strokes)
        for stroke in strokes:
            self.assertEqual(stroke.get_fill_opacity(),0)
            self.assertAlmostEqual(stroke.get_stroke_opacity(),.6,places=6)

    def test_later_styling_retains_hidden_fill_and_dash_geometry(self):
        a=m.SVGMobject(svg_string=CLOSED,stroke_width=0)
        a.set_stroke(width=6)
        b=m.SVGMobject(svg_string=CLOSED,stroke_width=6)
        np.testing.assert_array_equal(pixels(a),pixels(b))
        hidden=NESTED.replace('fill="red"','fill="red" fill-opacity="0"')
        a=m.SVGMobject(svg_string=hidden,height=4)
        a.set_fill(opacity=1)
        np.testing.assert_array_equal(pixels(a),pixels(m.SVGMobject(svg_string=NESTED,height=4)))

    def test_styling_through_parent_group_respects_each_native_paint_role(self):
        svg=m.SVGMobject(svg_string=CLOSED,stroke_width=None)
        ordinary=m.Square()
        group=m.VGroup(svg,ordinary)
        group.set_color(m.YELLOW,opacity=.5)
        group.set_rgba_array([[.2,.4,.6,.7]],name='fill_rgba',recurse=True)
        group.set_style(fill_rgba=[[.8,.4,.2,.8]],stroke_rgba=[[.2,.4,.8,.6]])
        fill,*strokes=svg[0].submobjects
        self.assertEqual(fill.get_stroke_opacity(),0)
        self.assertAlmostEqual(fill.get_fill_opacity(),.8,places=6)
        self.assertTrue(all(s.get_fill_opacity()==0 for s in strokes))
        self.assertAlmostEqual(ordinary.get_fill_opacity(),.8,places=6)
        self.assertAlmostEqual(ordinary.get_stroke_opacity(),.6,places=6)

    def test_parts_api_is_detached_and_does_not_mutate_a_live_receiver(self):
        obj=m.SVGMobject(svg_string=NESTED);scene=m.Scene();scene.add(obj)
        before=geometry(obj);children=list(obj.submobjects)
        parts=obj.mobjects_from_svg_string(DASHED)
        self.assertEqual(list(obj.submobjects),children)
        for a,b in zip(before,geometry(obj)):np.testing.assert_array_equal(a,b)
        self.assertEqual(len(parts),1)
        self.assertEqual(len(parts[0]),4)
        self.assertFalse(parts[0]._is_bound())
        self.assertEqual(parts[0].parents,[])
        self.assertEqual(scene.get_time(),0)

    def test_single_file_read_and_authored_resolver_dispatch(self):
        calls=[]
        case=self
        class Resolved(m.SVGMobject):
            file_name='authored.svg'
            def file_name_to_svg_string(self,name):
                case.assertIsInstance(self.svg_default,dict)
                case.assertEqual(self.path_string_config,{})
                self.get_points()  # Authored resolver sees initialized live state.
                calls.append(name)
                return NESTED if len(calls)==1 else '<svg><rect width="3" height="4"/></svg>'
        svg=Resolved()
        self.assertEqual(calls,['authored.svg'])
        self.assertEqual(svg.svg_string,NESTED)
        self.assertEqual(len(svg[0]),2)
        with tempfile.TemporaryDirectory() as temp:
            path=Path(temp)/'shape.svg';path.write_text(NESTED)
            actual=m.SVGMobject(path)
            np.testing.assert_array_equal(pixels(actual),pixels(m.SVGMobject(svg_string=NESTED)))

    def test_copy_pickle_and_checkpoint_retain_independent_paint_roles(self):
        source=m.SVGMobject(svg_string=CLOSED,stroke_width=None)
        before=pixels(source)
        for clone in (source.copy(),copy.deepcopy(source),pickle.loads(pickle.dumps(source))):
            clone.set_opacity(.3)
            self.assertTrue(all(s.get_fill_opacity()==0 for s in clone[0].submobjects[1:]))
            self.assertIsNot(clone[0],source[0])
            np.testing.assert_array_equal(pixels(source),before)
        scene=m.Scene();scene.add(source);state=scene.get_state()
        source.set_color(m.YELLOW).shift(m.RIGHT)
        scene.restore_state(state)
        np.testing.assert_array_equal(pixels(source),before)
        source.set_opacity(.3)
        self.assertTrue(all(s.get_fill_opacity()==0 for s in source[0].submobjects[1:]))

    def test_fade_and_builder_animation_preserve_holes_and_roles(self):
        obj=m.SVGMobject(svg_string=NESTED,height=4)
        expected=pixels(obj)
        scene=m.Scene();scene.play(m.FadeIn(obj),run_time=.125)
        np.testing.assert_array_equal(pixels(obj),expected)
        scene.play(obj.animate.set_opacity(.5),run_time=.125)
        self.assertEqual(obj[0][0].get_stroke_opacity(),0)
        self.assertEqual(obj[0][1].get_fill_opacity(),0)
        np.testing.assert_array_equal(pixels(obj)[36,64,:3],[0,0,0])
        self.assertGreater(scene.get_time(),0)

    def test_transform_and_svg_export_keep_native_paint_families(self):
        class Exported(m.Scene):
            def construct(self):
                self.obj=m.SVGMobject(svg_string=CLOSED,stroke_width=None)
                self.add(self.obj)
                self.play(self.obj.animate.rotate(.3).shift(m.RIGHT),run_time=.125)
        with tempfile.TemporaryDirectory() as temp:
            path=Path(temp)/'result.svg'
            scene=Exported()
            scene.render(path,format='svg',resolution=(128,72))
            self.assertTrue(all(s.get_fill_opacity()==0 for s in scene.obj[0].submobjects[1:]))
            text=path.read_text()
            self.assertIn('<svg',text)
            self.assertIn('<path',text)
            self.assertNotIn('stroke-dasharray',text)
            imported=m.SVGMobject(svg_string=text,stroke_width=None)
            self.assertGreater(len(imported),1)

    def test_refusals_leave_parts_receiver_unchanged(self):
        obj=m.SVGMobject(svg_string=NESTED);before=pixels(obj)
        for source in ('<svg><script/></svg>', '<!DOCTYPE svg><svg/>',
                       DASHED.replace('2 2','0 1'), DASHED.replace('2 2','1e-300 1e-300')):
            with self.assertRaises(ValueError):obj.mobjects_from_svg_string(source)
            np.testing.assert_array_equal(pixels(obj),before)
        with self.assertRaisesRegex(NotImplementedError,'svg_default'):m.SVGMobject(svg_string=NESTED,svg_default={'color':m.RED})

    def test_rebuild_keeps_scene_owner_and_rejects_bad_source_before_publication(self):
        obj=m.SVGMobject(svg_string=NESTED);scene=m.Scene();scene.add(obj)
        calls=[];obj.add_updater(lambda mob,dt:calls.append(dt),call=False)
        obj.svg_string=DASHED
        # Reference semantics (Ledger `same`): the rebuilt family is appended.
        kept=tuple(obj.submobjects);added=len(obj.mobjects_from_svg_string(DASHED))
        self.assertIsNone(obj.init_svg_mobject())
        self.assertTrue(obj._is_bound())
        self.assertEqual(tuple(obj.submobjects[:len(kept)]),kept)
        self.assertEqual(len(obj.submobjects),len(kept)+added)
        self.assertEqual(len(obj[len(kept)]),4)
        before=geometry(obj);children=tuple(obj.submobjects)
        obj.svg_string='<svg><script/></svg>'
        with self.assertRaises(ValueError):obj.init_svg_mobject()
        self.assertEqual(tuple(obj.submobjects),children)
        for a,b in zip(before,geometry(obj)):np.testing.assert_array_equal(a,b)
        scene.wait(.125);self.assertTrue(calls)

    def test_plain_svg_geometry_and_original_signature_remain_unchanged(self):
        src='<svg><rect width="4" height="3" fill="red"/><circle cx="8" cy="2" r="1" fill="blue"/></svg>'
        actual=m.SVGMobject(svg_string=src)
        raw=m.VMobject();specs=raw._build_svg_mobject(m._native_shell_factory,'',src)
        m._hang_native_children(raw,specs)
        raw.flip(m.RIGHT).set_stroke(width=0).center().set_height(2)
        np.testing.assert_array_equal(pixels(actual),pixels(raw))
        self.assertIs(importlib.import_module('manimlib.mobject.svg.svg_mobject').SVGMobject,m.SVGMobject)
        self.assertIn('svg_default',inspect.signature(m.SVGMobject).parameters)

    def test_hole_pixels_are_identical_at_one_four_threads(self):
        class Imported(m.Scene):
            def construct(self):
                self.add(m.SVGMobject(svg_string=NESTED,height=4))
                self.wait(.25)
        class Analytic(m.Scene):
            def construct(self):
                # Explicit reversed inner contour, no SVG parser/boolean used.
                path=m.VMobject().set_points_as_corners([(-2,-2,0),(2,-2,0),(2,2,0),(-2,2,0),(-2,-2,0)])
                inner=m.VMobject().set_points_as_corners([(-1.2,-1.2,0),(-1.2,1.2,0),(1.2,1.2,0),(1.2,-1.2,0),(-1.2,-1.2,0)])
                path.append_vectorized_mobject(inner)
                path.set_fill("#ff0000",1).set_stroke(width=0)
                self.add(path);self.wait(.25)
        outputs=[]
        with tempfile.TemporaryDirectory() as temp:
            for threads in (1,4):
                pair=[]
                for cls in (Imported,Analytic):
                    path=Path(temp)/f'{cls.__name__}-{threads}.y4m'
                    result=cls().render(path,format='y4m',resolution=(128,72),fps=8,threads=threads)
                    self.assertEqual(result.frame_count,2);pair.append(path.read_bytes())
                self.assertEqual(*pair);outputs.append(pair[0])
        self.assertEqual(*outputs)

if __name__=='__main__':unittest.main(verbosity=2)
