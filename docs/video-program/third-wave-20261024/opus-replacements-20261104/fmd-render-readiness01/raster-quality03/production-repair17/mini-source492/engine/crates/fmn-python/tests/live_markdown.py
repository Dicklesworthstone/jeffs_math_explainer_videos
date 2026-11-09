"""Native Markdown document selection, edits and real scene-clock animation."""
from __future__ import annotations

import copy
import io
from pathlib import Path
import pickle
import tempfile
import unittest
from unittest.mock import patch

import numpy as np
import manimlib as m
from fmn_python.markdown import MarkdownMobject

SOURCE = '# Native scenes\n\nA **bold** word & `Vec<T>`.\n\n```rust\nfn main() {}\n```\n'


def capture(obj):
    return m.Camera(resolution=(192, 108)).capture_snapshot(obj).png()


def records(obj):
    return tuple((id(member), member.data.tobytes()) for member in obj.get_family())


class LiveMarkdownTests(unittest.TestCase):
    def test_document_catalog_selection_and_unicode_byte_provenance(self):
        doc = MarkdownMobject('# αβ\n\nBody Ω\n\nAnother Ω')
        self.assertEqual(doc.block_kinds, ('heading', 'paragraph', 'paragraph'))
        self.assertEqual(doc.get_block_source(-1), 'Another Ω')
        self.assertIs(doc.select_text('Ω', occurrence=1)[0], doc.get_block(2))
        start = doc.source.index('Body')
        self.assertIs(doc.select_source(start, start+len('Body Ω'))[0], doc.get_block(1))
        self.assertEqual(len(doc.select_source(0, 0)), 0)
        self.assertEqual(len(doc.select_source(0, len(doc.source))), 3)
        with self.assertRaises(ValueError): doc.select_source(-1, 2)
        with self.assertRaises(ValueError): doc.select_text('absent')
        with self.assertRaises(ValueError): doc.select_text('')
        with self.assertRaises(IndexError): doc.get_block(3)
        with self.assertRaises(AttributeError): doc.source = 'invalid'

    def test_initial_visible_bounds_are_centered_and_blocks_are_real(self):
        doc = MarkdownMobject(SOURCE)
        self.assertIsInstance(doc, m.VGroup)
        np.testing.assert_allclose(doc.get_center(), 0, atol=1e-6)
        self.assertEqual(doc.block_kinds, ('heading', 'paragraph', 'code'))
        self.assertTrue(all(block.family_members_with_points() for block in doc.get_blocks()))
        self.assertGreater(len(set(tuple(row) for member in doc.get_block(2).family_members_with_points()
                                   for row in member.data['fill_rgba'])), 1)

    def test_insert_and_remove_reuse_exact_blocks_and_live_glyph_views(self):
        doc = MarkdownMobject('one\n\ntwo\n\nthree')
        original = tuple(doc.get_blocks())
        glyph = original[1]._markdown_content.family_members_with_points()[0]
        view = glyph.get_points()
        self.assertIs(doc.set_source('new\n\none\n\ntwo\n\nthree'), doc)
        self.assertEqual(tuple(doc.get_blocks())[1:], original)
        np.testing.assert_array_equal(view, glyph.get_points())
        doc.set_source('two\n\nthree')
        self.assertEqual(tuple(doc.get_blocks()), original[1:])
        np.testing.assert_array_equal(view, glyph.get_points())
        self.assertEqual(len(doc._markdown_blocks), 2)

    def test_duplicate_blocks_match_in_original_occurrence_order(self):
        doc = MarkdownMobject('same\n\nsame\n\nlast')
        a, b, c = doc.get_blocks()
        doc.set_source('same\n\nnew\n\nsame\n\nlast')
        self.assertIs(doc.get_block(0), a)
        self.assertIs(doc.get_block(2), b)
        self.assertIs(doc.get_block(3), c)

    def test_edited_blocks_keep_wrapper_annotations_and_updaters(self):
        doc = MarkdownMobject('one\n\ntwo')
        block = doc.get_block(1)
        annotation = m.Dot(block.get_right() + m.RIGHT)
        block.add(annotation)
        before = annotation.get_center() - block._markdown_anchor.get_center()
        calls=[]
        callback=lambda obj, dt: calls.append(dt)
        block.add_updater(callback, call=False)
        scene=m.Scene(); scene.add(doc)
        doc.set_source('# Heading\n\nchanged')
        self.assertIs(doc.get_block(1), block)
        self.assertIn(annotation, block.submobjects)
        self.assertIs(block.updaters[0], callback)
        np.testing.assert_allclose(annotation.get_center()-block._markdown_anchor.get_center(), before, atol=1e-6)
        self.assertEqual(calls, [])
        self.assertEqual(scene.get_time(), 0)
        self.assertEqual(tuple(scene.mobjects), (doc,))

    def test_unchanged_paint_survives_reflow_and_noop_is_truly_noop(self):
        doc=MarkdownMobject('first\n\nkeep')
        keep=doc.get_block(1).set_color(m.RED)
        before=records(doc)
        doc.set_source(doc.source)
        self.assertEqual(records(doc), before)
        doc.set_source('new\n\nfirst\n\nkeep')
        self.assertIs(doc.get_block(2), keep)
        for member in keep._markdown_content.family_members_with_points():
            np.testing.assert_allclose(member.data['fill_rgba'][:, :3], np.tile(m.color_to_rgb(m.RED), (len(member.data),1)), atol=1e-6)

    def test_affine_reflow_preserves_upper_left_and_world_geometry(self):
        transform=np.array([[-1.2,.35,.1],[.25,.8,0],[.15,.2,1]])
        doc=MarkdownMobject('old\n\nbody')
        origin=doc._markdown_anchors[0].get_center().copy()
        doc.apply_matrix(transform, about_point=m.ORIGIN).shift((1,-.5,.2))
        target='longer words\n\nnew body\n\nend'
        reference=MarkdownMobject(target)
        reference.shift(origin-reference._markdown_anchors[0].get_center())
        reference.apply_matrix(transform, about_point=m.ORIGIN).shift((1,-.5,.2))
        expected_origin=doc._markdown_anchors[0].get_center().copy()
        doc.set_source(target)
        np.testing.assert_allclose(doc._markdown_anchors[0].get_center(), expected_origin, atol=1e-6)
        np.testing.assert_allclose(doc.get_all_points(), reference.get_all_points(), atol=2e-6)
        self.assertEqual(capture(doc), capture(reference))

    def test_empty_document_can_grow_shrink_and_recover(self):
        doc=MarkdownMobject('')
        doc.shift((1,2,0)).rotate(.2)
        origin=doc._markdown_anchors[0].get_center().copy()
        doc.set_source('hello')
        np.testing.assert_allclose(doc._markdown_anchors[0].get_center(), origin, atol=2e-6)
        doc.set_source('')
        self.assertEqual(doc.block_ranges, ())
        doc.set_source('# Back')
        self.assertEqual(doc.block_kinds, ('heading',))

    def test_copy_deepcopy_pickle_and_saved_state_are_independent(self):
        original=MarkdownMobject(SOURCE)
        original.save_state()
        for clone in (original.copy(), copy.deepcopy(original), pickle.loads(pickle.dumps(original))):
            self.assertIsNot(clone.get_block(0), original.get_block(0))
            clone.set_source('changed')
            self.assertEqual(original.source, SOURCE)
            self.assertEqual(len(original.block_ranges),3)
        original.set_source('new')
        original.restore()
        self.assertEqual(original.source,SOURCE)
        original.set_source('restored and editable')

    def test_invalid_input_and_native_failure_preserve_live_state(self):
        doc=MarkdownMobject('valid')
        scene=m.Scene(); scene.add(doc)
        before=records(doc)
        for source in (None,b'bytes','x'*32769,'π'*16385):
            with self.assertRaises((TypeError,ValueError)): doc.set_source(source)
            self.assertEqual(records(doc),before)
        error=KeyboardInterrupt('cancel native shaping')
        with patch.object(m,'_build_math_markdown',side_effect=error):
            with self.assertRaises(KeyboardInterrupt) as caught: doc.set_source('new')
        self.assertIs(caught.exception,error)
        self.assertEqual(records(doc),before)
        self.assertEqual(doc.source,'valid')
        doc.set_source('recovery')

    def test_reentry_and_callback_edits_do_not_poison_copies(self):
        doc=MarkdownMobject('first')
        builder=m._build_math_markdown
        clones=[]
        def hooked(*args,**kwargs):
            clones.extend((doc.copy(),copy.deepcopy(doc)))
            with self.assertRaises(RuntimeError): doc.set_source('recursive')
            return builder(*args,**kwargs)
        with patch.object(m,'_build_math_markdown',side_effect=hooked): doc.set_source('second')
        for clone in clones: clone.set_source('independent')
        def mutation(*args,**kwargs):
            doc.shift(m.RIGHT)
            return builder(*args,**kwargs)
        old=doc._markdown_anchors[0].get_center().copy()
        with patch.object(m,'_build_math_markdown',side_effect=mutation):
            with self.assertRaisesRegex(RuntimeError,'changed during'): doc.set_source('third')
        self.assertEqual(doc.source,'second')
        np.testing.assert_allclose(doc._markdown_anchors[0].get_center(),old+m.RIGHT,atol=1e-6)

    def test_edits_refuse_broken_structure_singular_chart_and_animate_spelling(self):
        doc=MarkdownMobject('x')
        with self.assertRaises(m._CapabilityError): doc.animate.set_source('y')
        doc.stretch(0,0)
        with self.assertRaisesRegex(ValueError,'nondegenerate'): doc.set_source('y')
        other=MarkdownMobject('x')
        other.remove(other._markdown_blocks)
        with self.assertRaisesRegex(RuntimeError,'structure'): other.set_source('y')

    def test_native_blocks_can_be_independently_animated(self):
        doc=MarkdownMobject('one\n\ntwo')
        scene=m.Scene(); scene.add(doc)
        block=doc.get_block(0)
        before=block.get_center().copy()
        scene.play(block.animate.shift(m.RIGHT),run_time=.25,rate_func=m.linear)
        np.testing.assert_allclose(block.get_center(),before+m.RIGHT,atol=1e-6)
        self.assertEqual(doc.source,'one\n\ntwo')
        doc.set_source('one\n\ntwo\n\nthree')

    def test_source_animation_commits_only_complete_endpoints(self):
        doc=MarkdownMobject('one\n\ntwo')
        blocks=tuple(doc.get_blocks())
        animation=doc.animate_source('three\n\nfour',rate_func=m.linear)
        animation.begin(); animation.interpolate(.5)
        self.assertEqual(doc.source,'one\n\ntwo')
        with self.assertRaises(RuntimeError): doc.set_source('conflict')
        animation.finish()
        self.assertEqual(doc.source,'three\n\nfour')
        self.assertEqual(tuple(doc.get_blocks()),blocks)
        self.assertFalse(vars(doc).get('_is_animating',False))
        doc.set_source('finished')

    def test_succession_prepares_against_latest_source_and_restores_on_abort(self):
        doc=MarkdownMobject('one')
        scene=m.Scene(); scene.add(doc)
        scene.play(m.Succession(doc.animate_source('two',run_time=.25),
                                doc.animate_source('three',run_time=.25)))
        self.assertEqual(doc.source,'three')
        before=capture(doc)
        animation=doc.animate_source('four',rate_func=m.linear)
        animation.begin(); animation.interpolate(.5); animation.abort()
        self.assertEqual(doc.source,'three')
        self.assertEqual(capture(doc),before)
        self.assertFalse(vars(doc).get('_is_animating',False))

    def test_partial_and_reversed_endpoints_are_not_mislabelled(self):
        doc=MarkdownMobject('first')
        doc.save_state()
        animation=doc.animate_source('second',rate_func=m.linear,final_alpha_value=.5)
        animation.begin(); animation.finish()
        self.assertEqual(doc.source,'first')
        with self.assertRaisesRegex(RuntimeError,'partial'): doc.set_source('next')
        doc.restore()
        animation=doc.animate_source('second',rate_func=m.linear,final_alpha_value=0)
        animation.begin(); animation.finish()
        self.assertEqual(doc.source,'first')
        doc.set_source('next')

    def test_structural_animation_refuses_before_live_alignment(self):
        doc=MarkdownMobject('one')
        before=records(doc)
        with self.assertRaisesRegex(ValueError,'equal block counts'):
            doc.animate_source('one\n\ntwo').begin()
        self.assertEqual(records(doc),before)
        self.assertFalse(vars(doc).get('_is_animating',False))

    def test_reinitialization_refuses_without_changing_scene_membership(self):
        doc=MarkdownMobject('original')
        scene=m.Scene(); scene.add(doc)
        before=records(doc)
        with self.assertRaisesRegex(RuntimeError,'set_source'):
            doc.__init__('replacement')
        self.assertEqual(doc.source,'original')
        self.assertEqual(records(doc),before)
        self.assertEqual(tuple(scene.mobjects),(doc,))
        doc.set_source('supported edit')

    def test_reflow_overflow_refuses_before_publishing_new_geometry(self):
        doc=MarkdownMobject('M').scale(1e38,about_point=m.ORIGIN)
        before=records(doc)
        with np.errstate(over='ignore',invalid='ignore'):
            with self.assertRaises((ValueError,m._CapabilityError)):
                doc.set_source('W'*100)
        self.assertEqual(doc.source,'M')
        self.assertEqual(records(doc),before)
        self.assertTrue(np.isfinite(doc.get_all_points()).all())

    def test_bound_saved_document_restores_full_family_and_frozen_source(self):
        doc=MarkdownMobject('first\n\nsecond')
        doc.add(m.Dot((2,1,0)))
        scene=m.Scene(); scene.add(doc)
        doc.save_state()
        saved=doc.saved_state
        saved_records=records(saved)
        before=capture(doc)
        doc.shift(m.RIGHT).set_color(m.RED).set_source('only one new block')
        self.assertIs(doc.restore(),doc)
        self.assertEqual(doc.source,'first\n\nsecond')
        self.assertEqual(tuple(scene.mobjects),(doc,))
        self.assertEqual(capture(doc),before)
        self.assertEqual(records(saved),saved_records)
        self.assertEqual(len(doc.submobjects),3)
        doc.set_source('different\n\nblocks\n\nagain')
        doc.restore()
        self.assertEqual(capture(doc),before)

    def test_scene_checkpoint_restores_source_catalog_and_original_block_identities(self):
        doc=MarkdownMobject('first\n\nsecond')
        scene=m.Scene();scene.add(doc)
        blocks=tuple(doc.get_blocks())
        state=scene.get_state()
        before=capture(doc)
        doc.set_source('new\n\nfirst\n\nsecond').shift(m.RIGHT)
        scene.restore_state(state)
        self.assertEqual(doc.source,'first\n\nsecond')
        self.assertEqual(tuple(doc.get_blocks()),blocks)
        self.assertEqual(capture(doc),before)
        self.assertIs(doc.select_text('second')[0],blocks[1])
        doc.set_source('restored document remains editable')

    def test_direct_interpolation_failure_restores_source_geometry_and_can_retry(self):
        doc=MarkdownMobject('initial')
        before=capture(doc)
        error=KeyboardInterrupt('cancel authored path')
        def path(a,b,alpha):
            if alpha > 0:
                raise error
            return a.copy()
        animation=doc.animate_source('replacement',rate_func=m.linear,path_func=path)
        animation.begin()
        with self.assertRaises(KeyboardInterrupt) as caught:
            animation.interpolate(.5)
        self.assertIs(caught.exception,error)
        self.assertEqual(doc.source,'initial')
        self.assertEqual(capture(doc),before)
        self.assertFalse(vars(doc).get('_is_animating',False))
        self.assertFalse(animation._source_owned)
        animation.abort()  # Cleanup is idempotent, including after path failure.
        animation.path_func=m.straight_path
        animation.begin(); animation.finish()
        self.assertEqual(doc.source,'replacement')

    def test_scene_updater_cancellation_restores_document_and_independent_annotations(self):
        doc=MarkdownMobject('before')
        annotation=m.Dot((2,1,0))
        doc.add(annotation)
        before=capture(doc)
        error=RuntimeError('stop at frame boundary')
        def stop(obj,dt):
            if dt > 0:
                raise error
        sentinel=m.VectorizedPoint()
        sentinel.add_updater(stop,call=False)
        scene=m.Scene(); scene.add(doc,sentinel)
        with self.assertRaises(RuntimeError) as caught:
            scene.play(doc.animate_source('after'),run_time=.25,rate_func=m.linear)
        self.assertIs(caught.exception,error)
        self.assertEqual(doc.source,'before')
        self.assertEqual(capture(doc),before)
        self.assertIn(annotation,doc.submobjects)
        self.assertFalse(vars(doc).get('_is_animating',False))
        sentinel.remove_updater(stop)
        scene.play(doc.animate_source('recovered'),run_time=.25)
        self.assertEqual(doc.source,'recovered')

    def test_literal_paragraph_matches_independent_native_text_pixels(self):
        doc=MarkdownMobject('Hello **world** & `x < y`')
        reference=m.MarkupText('Hello <b>world</b> &amp; <tt>x &lt; y</tt>',font_size=24)
        reference.shift(doc._markdown_anchors[0].get_center()-reference.get_corner(m.UL))
        self.assertEqual(capture(doc),capture(reference))

    def test_owned_block_group_animation_keeps_future_edits_in_the_scene(self):
        doc=MarkdownMobject('one\n\ntwo')
        scene=m.Scene();scene.add(doc)
        self.assertIs(doc.get_blocks(),doc.get_blocks())
        scene.play(doc.get_blocks().animate.shift(m.RIGHT),run_time=.25)
        self.assertEqual(tuple(scene.mobjects),(doc,))
        doc.set_source('one\n\ntwo\n\nnewly inserted')
        self.assertEqual(len(doc.block_ranges),3)
        actual=m.Camera(resolution=(192,108)).capture_snapshot(*scene.mobjects).png()
        self.assertEqual(actual,capture(doc))

    def test_source_selected_block_animation_keeps_later_table_visible(self):
        doc=MarkdownMobject('paragraph\n\n```rust\nlet x = 1;\n```')
        scene=m.Scene();scene.add(doc)
        scene.play(doc.select_text('let x')[0].animate.shift(.3*m.RIGHT),run_time=.25)
        self.assertEqual(tuple(scene.mobjects),(doc,))
        before=capture(doc)
        doc.set_source(doc.source+'\n\n| Name | Value |\n| --- | --- |\n| ready | 1 |\n')
        self.assertEqual(doc.block_kinds[-1],'table')
        actual=m.Camera(resolution=(192,108)).capture_snapshot(*scene.mobjects).png()
        self.assertEqual(actual,capture(doc))
        self.assertNotEqual(actual,before)

    def test_tables_render_visible_rules_and_match_independent_native_table(self):
        from fmn_python.table import TableMobject
        doc=MarkdownMobject('| Name | Value |\n| --- | --- |\n| ready | 8 |\n',font_size=24)
        reference=TableMobject(['Name','Value'],[['ready','8']],font_size=24,
                              color=m.WHITE,header_color=m.WHITE,rule_color=m.GREY_B)
        reference.shift(doc._markdown_anchors[0].get_center()-reference.get_corner(m.UL))
        self.assertEqual(capture(doc),capture(reference))
        outline=doc.get_block(0)._markdown_content[0]
        expected=m.color_to_rgb(m.GREY_B)
        np.testing.assert_allclose(outline.data['stroke_rgba'][:,:3],
                                   np.tile(expected,(len(outline.data),1)),atol=1e-6)
        before=capture(doc)
        outline.set_stroke(opacity=0)
        self.assertNotEqual(capture(doc),before)

    def test_morph_frame_bytes_match_independent_text_at_one_and_four_threads(self):
        class Document(m.Scene):
            def construct(self):
                doc=MarkdownMobject('A',font_size=48)
                self.add(doc)
                self.play(doc.animate_source('B'),run_time=.5,rate_func=m.linear)
        class Reference(m.Scene):
            def construct(self):
                a=m.MarkupText('A',font_size=48)
                a.shift(-a.get_center())
                b=m.MarkupText('B',font_size=48)
                b.shift(a.get_corner(m.UL)-b.get_corner(m.UL))
                self.add(a)
                self.play(m.Transform(a,b),run_time=.5,rate_func=m.linear)
        with tempfile.TemporaryDirectory(prefix='fmn-markdown-') as folder:
            outputs=[]
            for cls in (Document,Reference):
                for threads in (1,4):
                    path=Path(folder)/f'{cls.__name__}-{threads}.y4m'
                    receipt=cls().render(path,format='y4m',resolution=(96,54),fps=8,threads=threads)
                    self.assertEqual(receipt.frame_count,4)
                    outputs.append(path.read_bytes())
            self.assertTrue(all(data==outputs[0] for data in outputs[1:]))


def run_live_markdown_acceptance():
    stream=io.StringIO()
    result=unittest.TextTestRunner(stream=stream,verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(LiveMarkdownTests))
    print(stream.getvalue())
    if not result.wasSuccessful(): raise AssertionError(stream.getvalue())


if __name__ == '__main__':
    run_live_markdown_acceptance()
