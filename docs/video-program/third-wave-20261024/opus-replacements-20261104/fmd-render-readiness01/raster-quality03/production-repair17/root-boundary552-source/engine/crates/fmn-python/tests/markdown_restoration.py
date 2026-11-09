"""Real native saved-document restoration across nonisomorphic glyph families."""
from __future__ import annotations

import unittest

import numpy as np
import manimlib as m
from fmn_python.markdown import MarkdownMobject


def capture(document):
    return m.Camera(resolution=(160, 90)).capture_snapshot(document).png()


def records(document):
    result = []
    for member in document.get_family():
        member.get_points()
        result.append((member.data.dtype.descr, member.data.copy()))
    return result


def equal_records(case, actual, expected):
    left, right = records(actual), records(expected)
    case.assertEqual(len(left), len(right))
    for (dtype_a, a), (dtype_b, b) in zip(left, right):
        case.assertEqual(dtype_a, dtype_b)
        np.testing.assert_array_equal(a, b)


class SavedMarkdownTests(unittest.TestCase):
    def document(self, math_mode=True):
        return MarkdownMobject('# Result\n\n$x_i+y_j$\n', math_mode=math_mode,
                               font_size=30, **({'line_width': 3, 'body_color': m.GREEN}
                                                if math_mode else {}))

    def test_structural_restore_is_exact_repeatable_and_never_changes_saved_owner(self):
        for bound in (False, True):
            for math_mode in (False, True):
                with self.subTest(bound=bound, math_mode=math_mode):
                    doc = self.document(math_mode)
                    doc.rotate(.2).stretch(1.3, 0).shift((.4, -.3, 0))
                    doc.get_block(0).set_color(m.YELLOW)
                    scene = m.Scene()
                    if bound:
                        scene.add(doc)
                    doc.save_state()
                    saved = doc.saved_state
                    expected = capture(saved)
                    saved_members = tuple(saved.get_family())
                    saved_records = [(d, a.copy()) for d, a in records(saved)]
                    for source in ('short', '# Other\n\nA longer paragraph\n\n- one\n- two\n', ''):
                        doc.set_source(source).shift((1, 0, 0))
                        self.assertIs(doc.restore(), doc)
                        self.assertIs(doc.saved_state, saved)
                        self.assertEqual(doc.source, saved.source)
                        self.assertEqual(doc.block_ranges, saved.block_ranges)
                        self.assertEqual(doc.block_kinds, saved.block_kinds)
                        self.assertEqual(doc.math_mode, saved.math_mode)
                        self.assertEqual(doc.line_width, saved.line_width)
                        equal_records(self, doc, saved)
                        self.assertEqual(capture(doc), expected)
                        self.assertEqual(tuple(saved.get_family()), saved_members)
                        for (_, current), (_, original) in zip(records(saved), saved_records):
                            np.testing.assert_array_equal(current, original)
                    self.assertEqual(scene.get_time(), 0)
                    self.assertEqual(tuple(scene.mobjects), (doc,) if bound else ())

    def test_bound_restore_retains_root_parent_callbacks_and_remaps_saved_annotations(self):
        doc = self.document()
        doc.note = m.Dot((2, 1, 0), radius=.08)
        doc.add(doc.note)
        scene = m.Scene(); parent = m.VGroup(doc); scene.add(parent)
        calls = []
        callback = lambda obj, dt: calls.append((obj, dt))
        doc.add_updater(callback, call=False)
        doc.save_state(); saved_note = doc.saved_state.note
        doc.set_source('changed')
        doc.note.shift((3, 0, 0))
        self.assertIs(doc.restore(), doc)
        self.assertIs(parent[0], doc)
        self.assertEqual(tuple(scene.mobjects), (parent,))
        self.assertIs(doc._scene, scene)
        self.assertIs(doc.updaters[0], callback)
        self.assertEqual(calls, [])
        self.assertEqual(scene.get_time(), 0)
        self.assertIsNot(doc.note, saved_note)
        self.assertIn(doc.note, doc.get_family())
        np.testing.assert_array_equal(doc.note.get_points(), saved_note.get_points())
        doc.note.shift((1, 0, 0))
        self.assertNotEqual(doc.note.get_center()[0], saved_note.get_center()[0])
        scene.wait(.125)
        self.assertTrue(calls)
        self.assertTrue(all(obj is doc for obj, _ in calls))

    def test_restored_descendants_are_owned_copies_not_saved_state_aliases(self):
        doc = self.document(); doc.save_state()
        saved = doc.saved_state
        doc.set_source('# Changed\n\n$x^5$ and text\n\nEnd')
        doc.restore()
        self.assertTrue({id(x) for x in doc.get_family()}.isdisjoint(
            {id(x) for x in saved.get_family()}))
        doc.get_block(1).set_color(m.RED)
        self.assertNotEqual(capture(doc), capture(saved))
        doc.restore()
        self.assertEqual(capture(doc), capture(saved))
        doc.set_source('# Result\n\nNow $x^2$\n')
        self.assertTrue(doc.math_mode)
        self.assertIn('x_i', saved.source)

    def test_saved_restore_recovers_a_partial_source_morph_and_permits_further_edits(self):
        doc = self.document(); scene = m.Scene(); scene.add(doc); doc.save_state()
        animation = doc.animate_source('# Changed\n\n$\\frac{1}{x}$\n',
                                       rate_func=m.linear, final_alpha_value=.5)
        animation.begin(); animation.finish()
        self.assertTrue(vars(doc).get('_markdown_partial', False))
        doc.restore()
        self.assertFalse(vars(doc).get('_markdown_partial', False))
        equal_records(self, doc, doc.saved_state)
        doc.set_source('# Ready\n\n$x^3$\n')
        self.assertEqual(doc.source, '# Ready\n\n$x^3$\n')

    def test_empty_saved_document_restores_and_can_gain_mathematics_again(self):
        doc = MarkdownMobject('', math_mode=True, line_width=2.5)
        scene = m.Scene(); scene.add(doc); doc.save_state()
        doc.set_source('# Full\n\n$\\sqrt{x}$\n'); doc.restore()
        self.assertEqual(doc.source, '')
        self.assertEqual(doc.block_ranges, ())
        self.assertTrue(doc.math_mode)
        self.assertEqual(doc.line_width, 2.5)
        self.assertEqual(tuple(scene.mobjects), (doc,))
        doc.set_source('```math\nx^2\n```')
        self.assertEqual(doc.block_kinds, ('math',))

    def test_active_restore_refuses_and_foreign_snapshots_are_independently_copied(self):
        doc = self.document(); scene = m.Scene(); scene.add(doc); doc.save_state()
        initial = doc.source
        anim = doc.animate_source('# Changed\n\n$x^4$\n', rate_func=m.linear)
        anim.begin(); anim.interpolate(.4)
        before = capture(doc)
        with self.assertRaisesRegex(RuntimeError, 'active'):
            doc.restore()
        self.assertEqual(capture(doc), before)
        anim.abort()
        self.assertEqual(doc.source, initial)
        other = self.document(); other_scene = m.Scene(); other_scene.add(other)
        saved = doc.saved_state; doc.saved_state = other
        before = capture(doc)
        # The document API supports snapshots from another Scene by detached
        # native copying; the source owner must not be adopted or mutated.
        other_members = tuple(other.get_family())
        doc.restore()
        equal_records(self, doc, other)
        self.assertEqual(tuple(other.get_family()), other_members)
        self.assertIs(other._scene, other_scene)
        self.assertTrue({id(member) for member in doc.get_family()}.isdisjoint(
            {id(member) for member in other.get_family()}))
        self.assertEqual(tuple(scene.mobjects), (doc,))
        self.assertEqual(tuple(other_scene.mobjects), (other,))
        doc.saved_state = saved
        doc.restore()
        with self.assertRaises(m._CapabilityError):
            doc.animate.restore()

    def test_scene_checkpoint_still_restores_original_block_identity_after_saved_restore(self):
        doc = self.document(); scene = m.Scene(); scene.add(doc)
        block = doc.get_block(1); scene_state = scene.get_state()
        doc.save_state(); doc.set_source('different'); doc.restore()
        self.assertIsNot(doc.get_block(1), block)
        scene.restore_state(scene_state)
        self.assertIs(doc.get_block(1), block)
        self.assertTrue(doc.math_mode)
        doc.set_source('# Edited\n\n$x^9$\n')


if __name__ == '__main__':
    unittest.main(verbosity=2)
