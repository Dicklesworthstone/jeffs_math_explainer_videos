"""Real Python scene -> native render-only FMTL -> native pixels.

Run by portal_studio::bundle::tests with the production reader/renderer helper.
The compiled portal and actual Scene lifecycle are required, never doubled.
"""
from pathlib import Path
import hashlib
import json
import tempfile
import unittest

import manimlib as m
import numpy as np
from fmn_python.bundle_export import export_bundle


class Visible(m.Scene):
    history_count = 0

    def construct(self):
        self.kept = []
        for index in range(self.history_count):
            discarded = m.Square(side_length=0.5, fill_opacity=1)
            self.add(discarded)
            self.remove(discarded)
            # Keep every proxy alive throughout recording. Passing because
            # Python happens to collect an unused object is not this contract.
            self.kept.append(discarded)
        self.subject = m.Square(side_length=0.5, fill_opacity=0.75, stroke_width=0)
        self.subject.set_color(m.BLUE).shift(m.LEFT)
        if self.history_count:
            self.subject.save_state()
            self.subject.generate_target()
            self.kept.extend([self.subject.copy() for _ in range(3)])
        self.add(self.subject)
        self.play(self.subject.animate(run_time=0.5, rate_func=m.linear).shift(m.RIGHT))
        self.wait(0.25)


class WithHistory(Visible):
    history_count = 128


class CompactBundleExport(unittest.TestCase):
    def setUp(self):
        # Retain artifacts on failure for inspection; no corpus assets required.
        self.root = Path(tempfile.mkdtemp(prefix="fmn-compact-bundle-"))

    def export(self, scene, name, **options):
        path = self.root / (name + ".fmtl")
        result = export_bundle(scene, path, fps=8, resolution=(96, 64), **options)
        data = path.read_bytes()
        self.assertEqual(result.digest, hashlib.sha256(data).hexdigest())
        self.assertEqual(result.bytes, len(data))
        self.assertEqual(result.frame_count, 6)
        return result, data

    def test_export_bytes_ignore_history_pins_saved_states_and_target_copies(self):
        _, clean = self.export(Visible, "clean")
        for run in range(3):
            instance = WithHistory()
            _, history = self.export(instance, "history-" + str(run))
            self.assertEqual(history, clean)
            self.assertEqual(len(instance.kept), 131)
            # The serializer did not reclaim or invalidate any retained proxy.
            self.assertTrue(np.isfinite(instance.kept[0].get_points()).all())
            self.assertAlmostEqual(instance.kept[0].get_width(), 0.5, places=5)
            self.assertTrue(np.isfinite(instance.subject.saved_state.get_points()).all())
        print(json.dumps({"scenario": "compact-bundle-history", "history": 128,
                          "bytes": len(clean), "sha256": hashlib.sha256(clean).hexdigest()},
                         sort_keys=True, separators=(",", ":")))

    def test_replay_preserves_all_frames_and_thread_independence(self):
        result, data = self.export(WithHistory, "replay")
        indices = list(range(result.frame_count))
        one = _test_bundle_frames(data, indices, 96, 64, 1)
        self.assertGreater(len(set(bytes(frame) for frame in one)), 2)
        for threads in (4, 16):
            self.assertEqual(one, _test_bundle_frames(data, indices, 96, 64, threads))
        self.assertEqual(one[::-1], _test_bundle_frames(data, indices[::-1], 96, 64, 1))
        _, control = self.export(Visible, "control")
        self.assertEqual(one, _test_bundle_frames(control, indices, 96, 64, 1))

    def test_exact_budget_and_no_clobber_publication_survive_projection(self):
        _, first = self.export(Visible, "budget-reference")
        _, bounded = self.export(WithHistory, "budget-exact", max_output_bytes=len(first))
        self.assertEqual(first, bounded)
        failed = self.root / "too-small.fmtl"
        with self.assertRaisesRegex(RuntimeError, "output budget"):
            export_bundle(WithHistory, failed, fps=8, resolution=(96, 64),
                          max_output_bytes=len(first) - 1)
        self.assertFalse(failed.exists())
        destination = self.root / "budget-reference.fmtl"
        with self.assertRaises(FileExistsError):
            export_bundle(WithHistory, destination, fps=8, resolution=(96, 64))
        self.assertEqual(destination.read_bytes(), first)

    def test_removed_proxy_can_return_with_changed_state_later_in_the_bundle(self):
        class Reappear(m.Scene):
            def construct(self):
                self.original = m.Square(side_length=0.5, fill_opacity=1, stroke_width=0)
                self.original.set_color(m.RED).shift(m.LEFT)
                self.add(self.original)
                self.wait(0.25)
                self.remove(self.original)
                self.wait(0.25)
                self.original.shift(2 * m.RIGHT).set_color(m.BLUE)
                self.add(self.original)
                self.wait(0.25)
        instance = Reappear()
        result, data = self.export(instance, "reappear")
        frames = _test_bundle_frames(data, list(range(result.frame_count)), 96, 64, 1)
        self.assertEqual(frames[0], frames[1])
        self.assertEqual(frames[2], frames[3])
        self.assertEqual(frames[4], frames[5])
        self.assertNotEqual(frames[0], frames[2])
        self.assertNotEqual(frames[0], frames[4])
        np.testing.assert_allclose(instance.original.get_center(), m.RIGHT, atol=1e-5)


_result = unittest.TextTestRunner(verbosity=2).run(
    unittest.defaultTestLoader.loadTestsFromTestCase(CompactBundleExport))
if not _result.wasSuccessful():
    raise AssertionError("compact bundle export acceptance failed")
