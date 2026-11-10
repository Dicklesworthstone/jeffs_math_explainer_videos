"""Real native subset selection, playback, cancellation and rendered frames.

Run against the installed wheel or the embedded production test module. There
is deliberately no mock fallback for manimlib, geometry, child lists or sinks.
"""
from pathlib import Path
import copy
import math
import os
import tempfile
import unittest

import numpy as np
import manimlib as m
from fmn_python import render_scene


def children():
    return [m.Square(side_length=.65, fill_opacity=1, stroke_width=0)
            .set_color(m.WHITE).shift((2 * index - 3) * m.RIGHT)
            for index in range(4)]


def assert_released(test, group, members):
    for node in [group, *members]:
        test.assertFalse(node._is_updating_suspended())
        test.assertFalse(getattr(node, "_is_animating", False))


class NativeSubsetTests(unittest.TestCase):
    def test_public_qualified_identity_and_live_callback_selection(self):
        from manimlib.animation.creation import ShowIncreasingSubsets
        self.assertIs(ShowIncreasingSubsets, m.ShowIncreasingSubsets)
        members = children()
        group = m.VGroup(*members)
        calls = []
        animation = ShowIncreasingSubsets(group, rate_func=m.linear,
                                         int_func=lambda value: calls.append(value) or math.floor(value))
        self.assertEqual(calls, [])
        animation.interpolate(.625)
        self.assertEqual(calls, [2.5])
        self.assertEqual(list(group.submobjects), members[:2])
        animation.int_func = lambda value: 3
        animation.interpolate(.25)
        self.assertEqual(list(group.submobjects), members[:3])

    def test_empty_native_family_and_one_by_one_reference_endpoint(self):
        for cls in (m.ShowIncreasingSubsets, m.ShowSubmobjectsOneByOne):
            empty = m.VGroup()
            animation = cls(empty, int_func=math.floor)
            animation.begin()
            animation.finish()
            self.assertEqual(list(empty.submobjects), [])
        members = children()
        group = m.VGroup(*members)
        animation = m.ShowSubmobjectsOneByOne(group, suspend_mobject_updating=True)
        animation.begin()
        animation.finish()
        self.assertEqual(list(group.submobjects), [members[-2]])
        assert_released(self, group, members)

    def test_native_scene_play_and_composition_preserve_source_identities(self):
        members = children()
        group = m.VGroup(*members)
        scene = m.Scene(camera_config=dict(resolution=(192, 108), fps=8))
        scene.add(group)
        seen = []
        class Authored(m.ShowIncreasingSubsets):
            def update_submobject_list(self, index):
                super().update_submobject_list(index)
                seen.append(tuple(self.mobject.submobjects))
        animation = Authored(group, int_func=math.floor, rate_func=m.linear,
                             run_time=.5, suspend_mobject_updating=True)
        composition = m.AnimationGroup(animation)
        scene.play(composition)
        self.assertEqual(list(group.submobjects), members)
        self.assertTrue(any(0 < len(row) < len(members) for row in seen))
        self.assertTrue(all(list(row) == members[:len(row)] for row in seen))
        # A stock composition delegates root allocation to Choreo; the
        # authoring descriptor does not own that generated native proxy.
        self.assertEqual(len(scene.mobjects), 1)
        native_root = scene.mobjects[0]
        self.assertIsInstance(native_root, m.Group)
        self.assertEqual(list(native_root.submobjects), [group])
        self.assertTrue(all(node.parents == [group] for node in members))
        assert_released(self, group, members)

    def test_partial_finish_abort_replay_and_prior_suspension(self):
        members = children()
        group = m.VGroup(*members)
        members[-1].suspend_updating()
        animation = m.ShowIncreasingSubsets(group, int_func=math.floor,
                                            rate_func=m.linear, final_alpha_value=.5,
                                            suspend_mobject_updating=True)
        animation.begin()
        animation.finish()
        self.assertEqual(list(group.submobjects), members[:2])
        self.assertTrue(members[-1]._is_updating_suspended())
        assert_released(self, group, members[:-1])
        animation.begin()
        animation.interpolate(.75)
        animation.abort()
        self.assertEqual(list(group.submobjects), members[:2])
        animation.final_alpha_value = 1
        animation.begin()
        animation.finish()
        self.assertEqual(list(group.submobjects), members)
        self.assertTrue(members[-1]._is_updating_suspended())
        assert_released(self, group, members[:-1])

    def test_native_copy_does_not_reuse_live_candidates(self):
        members = children()
        group = m.VGroup(*members)
        animation = m.ShowIncreasingSubsets(group, int_func=math.floor, rate_func=m.linear)
        duplicate = animation.copy()
        self.assertIsNot(duplicate.mobject, group)
        self.assertTrue(all(a is not b for a, b in zip(members, duplicate.all_submobs)))
        self.assertEqual(list(duplicate.mobject.submobjects), duplicate.all_submobs)
        duplicate.begin()
        duplicate.interpolate(.5)
        self.assertEqual(list(group.submobjects), members)
        duplicate.abort()
        self.assertEqual(list(duplicate.mobject.submobjects), duplicate.all_submobs)

    def test_native_deepcopy_preserves_previsited_and_cross_root_aliases(self):
        for bound in (False, True):
            for child_first in (False, True):
                member = children()[0]
                left, right = m.VGroup(member), m.VGroup(member)
                left.label = {"child": member}
                if bound:
                    scene = m.Scene().add(left, right)
                    original_roots = tuple(scene.mobjects)
                source = [member, left, right] if child_first else [left, right, member]
                duplicate = copy.deepcopy(source)
                if child_first:
                    copied_member, copied_left, copied_right = duplicate
                else:
                    copied_left, copied_right, copied_member = duplicate
                self.assertIs(copied_left.submobjects[0], copied_member)
                self.assertIs(copied_right.submobjects[0], copied_member)
                self.assertIs(copied_left.label["child"], copied_member)
                self.assertIsNot(copied_member, member)
                original_points = member.get_points().copy()
                copied_member.shift(m.RIGHT)
                np.testing.assert_array_equal(member.get_points(), original_points)
                self.assertFalse(np.array_equal(copied_member.get_points(), original_points))
                if bound:
                    self.assertEqual(tuple(scene.mobjects), original_roots)

    def test_native_setter_failure_after_splice_rolls_back(self):
        members = children()
        group = m.VGroup(*members)
        scene = m.Scene().add(group)
        animation = m.ShowIncreasingSubsets(group, rate_func=m.linear,
                                            suspend_mobject_updating=True)
        animation.begin()
        original = group.set_submobjects
        failure, calls = RuntimeError("authored child splice"), []
        def fail_once(values):
            original(values)
            calls.append(tuple(values))
            if len(calls) == 1:
                raise failure
        group.set_submobjects = fail_once
        try:
            with self.assertRaises(RuntimeError) as caught:
                animation.interpolate(.5)
            self.assertIs(caught.exception, failure)
            self.assertEqual(list(group.submobjects), members)
            self.assertEqual(list(scene.mobjects), [group])
            assert_released(self, group, members)
        finally:
            group.__dict__.pop("set_submobjects", None)

    def test_native_word_selector_obeys_source_spans_and_authored_hooks(self):
        text = m.Text("alpha beta", font_size=36)
        paths = list(text._string_sub_paths)
        spans = list(text._string_sub_spans)
        inner = m.VGroup(*list(text.submobjects))
        text.set_submobjects([inner])
        text._string_sub_paths = [[0, *path] for path in paths]
        leaves = []
        for path in text._string_sub_paths:
            node = text
            for index in path:
                node = node.submobjects[index]
            leaves.append(node)
        expected = [node for node, span in zip(leaves, spans) if span[0] < 5]
        calls, indices = [], []
        class Authored(m.AddTextWordByWord):
            def update_submobject_list(self, index):
                indices.append(index)
                super().update_submobject_list(index)
        animation = Authored(text, int_func=lambda value: calls.append(value) or math.floor(value),
                             run_time=1, rate_func=m.linear, suspend_mobject_updating=True)
        self.assertEqual(calls, [])
        animation.begin()
        animation.interpolate(.4)
        visible = text.get_family()
        self.assertFalse(any(node in visible for node in leaves))
        animation.interpolate(.6)
        visible = text.get_family()
        self.assertEqual([node for node in leaves if node in visible], expected)
        self.assertEqual(calls, [0., .8, 1.2])
        self.assertEqual(indices, [0, 0, 1])
        animation.int_func = lambda _: -1
        animation.interpolate(.9)
        visible = text.get_family()
        self.assertEqual([node for node in leaves if node in visible], expected)
        animation.abort()
        self.assertEqual([node for node in leaves if node in text.get_family()], leaves)
        self.assertEqual(list(text.submobjects), [inner])
        assert_released(self, text, leaves)
        animation.int_func = math.floor
        scene = m.Scene(camera_config=dict(resolution=(192, 108), fps=8))
        scene.play(m.AnimationGroup(animation))
        self.assertEqual([node for node in leaves if node in text.get_family()], leaves)
        assert_released(self, text, leaves)

    def test_native_letter_selector_and_failed_authored_override_restore_glyphs(self):
        from manimlib.animation.creation import AddTextLetterByLetter
        text = m.Text("abcd", font_size=36)
        rows = [(node, tuple(node.submobjects)) for node in text.get_family()]
        calls = []
        animation = AddTextLetterByLetter(text, int_func=lambda value: calls.append(value) or math.floor(value),
                                          rate_func=m.linear, suspend_mobject_updating=True)
        animation.begin()
        animation.interpolate(.6)
        self.assertEqual(calls, [0., 2.4])
        self.assertEqual(len(text.submobjects), 2)
        animation.abort()
        failure = RuntimeError("authored native text interpolation")
        class Broken(m.AddTextWordByWord):
            def interpolate_mobject(self, alpha):
                super().interpolate_mobject(alpha)
                if alpha > 0:
                    raise failure
        broken = Broken(text, suspend_mobject_updating=True)
        broken.begin()
        with self.assertRaises(RuntimeError) as caught:
            broken.interpolate(.5)
        self.assertIs(caught.exception, failure)
        for node, original in rows:
            self.assertEqual(list(node.submobjects), list(original))
            self.assertFalse(node._is_updating_suspended())
            self.assertFalse(getattr(node, "_is_animating", False))

    def test_native_output_contains_progressive_pixels_and_is_thread_reproducible(self):
        outputs = []
        for threads in (1, 4, 16):
            destination = evidence / f"subset-{threads}.y4m"
            result = render_scene(SubsetScene, destination, threads=threads)
            self.assertEqual(result.frame_count, 8)
            self.assertFalse(result.certified)
            data = destination.read_bytes()
            header, payload = data.split(b"\n", 1)
            self.assertEqual(header, b"YUV4MPEG2 W192 H108 F8:1 Ip A1:1 C420mpeg2")
            size = 192 * 108 * 3 // 2
            self.assertEqual(len(payload), 8 * (size + 6))
            counts = []
            for offset in range(0, len(payload), size + 6):
                self.assertEqual(payload[offset:offset + 6], b"FRAME\n")
                luma = np.frombuffer(payload[offset + 6:offset + 6 + 192 * 108], dtype=np.uint8)
                counts.append(int(np.count_nonzero(luma > 180)))
            self.assertEqual(counts, sorted(counts))
            self.assertGreater(counts[-1], counts[0])
            self.assertGreater(len(set(counts)), 2)
            outputs.append(data)
        self.assertEqual(outputs[0], outputs[1])
        self.assertEqual(outputs[0], outputs[2])

    def test_render_failure_after_capture_restores_children_and_does_not_publish(self):
        scene = BrokenSubsetScene()
        destination = evidence / "not-published.y4m"
        with self.assertRaises(SelectorFailure) as caught:
            render_scene(scene, destination, threads=1)
        self.assertIs(caught.exception, scene.failure)
        self.assertFalse(destination.exists())
        self.assertEqual(list(scene.group.submobjects), scene.members)
        assert_released(self, scene.group, scene.members)
        scene.play(m.ShowIncreasingSubsets(scene.group, int_func=math.floor, run_time=.5))
        self.assertEqual(list(scene.group.submobjects), scene.members)


class SubsetScene(m.Scene):
    default_camera_config = dict(resolution=(192, 108), fps=8)
    def construct(self):
        self.members = children()
        self.group = m.VGroup(*self.members)
        self.play(m.ShowIncreasingSubsets(self.group, int_func=math.floor,
                                         rate_func=m.linear, run_time=1,
                                         suspend_mobject_updating=True))


class SelectorFailure(RuntimeError):
    pass


class BrokenSubsetScene(SubsetScene):
    def construct(self):
        self.members = children()
        self.group = m.VGroup(*self.members)
        self.failure = SelectorFailure("selector failed after native capture")
        def select(count):
            if count > 2:
                raise self.failure
            return math.floor(count)
        self.play(m.ShowIncreasingSubsets(self.group, int_func=select,
                                         rate_func=m.linear, run_time=1,
                                         suspend_mobject_updating=True))


evidence = Path(tempfile.mkdtemp(prefix="fmn-native-subset-", dir=os.environ.get("FMN_SUBSET_EVIDENCE_DIR")))
suite = unittest.defaultTestLoader.loadTestsFromTestCase(NativeSubsetTests)
result = unittest.TextTestRunner(verbosity=2).run(suite)
assert result.wasSuccessful(), "native subset reveal acceptance failed"
print("native subset reveal output evidence:", evidence)
