"""Native captured-frame publication, including the prepare/commit boundary."""
import gc
import hashlib
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m


class CaptureOutput(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="fmn-capture-output-"))
        self.scene = m.Scene(camera_config=dict(resolution=(96, 54), fps=8))
        self.mob = m.Square(side_length=1, fill_opacity=1, stroke_width=0)
        self.scene.add(self.mob)
        self.snapshot = self.scene.camera.capture_snapshot(*self.scene.mobjects)

    def verify(self, receipt):
        path, size, digest = receipt
        data = Path(path).read_bytes()
        self.assertEqual(data[:8], b"\x89PNG\r\n\x1a\n")
        self.assertEqual(len(data), size)
        self.assertEqual(hashlib.sha256(data).hexdigest(), digest)
        return data

    def test_native_png_is_repeatable_across_threads(self):
        left = self.verify(self.snapshot.save_png(self.root / "one.png", threads=1))
        right = self.verify(self.snapshot.save_png(self.root / "four.png", threads=4))
        self.assertEqual(left, right)
        self.assertEqual(self.scene.time, 0.)

    def test_prepared_file_is_not_visible_until_commit(self):
        path = self.root / "nested" / "final.png"
        pending = self.snapshot.prepare_png(path)
        self.assertFalse(path.exists())
        receipt = pending.commit()
        self.verify(receipt)
        self.assertEqual(pending.commit(), receipt)
        pending.abort()
        self.assertTrue(path.exists())

    def test_abort_cancels_only_the_prepared_file(self):
        path = self.root / "aborted.png"
        pending = self.snapshot.prepare_png(path)
        pending.abort()
        pending.abort()
        with self.assertRaises(RuntimeError):
            pending.commit()
        self.assertFalse(path.exists())
        self.assertEqual(list(self.root.iterdir()), [])

    def test_drop_cleans_prepared_file(self):
        pending = self.snapshot.prepare_png(self.root / "dropped.png")
        del pending
        gc.collect()
        self.assertEqual(list(self.root.iterdir()), [])

    def test_collision_during_commit_never_overwrites(self):
        path = self.root / "collision.png"
        pending = self.snapshot.prepare_png(path)
        path.write_bytes(b"another publisher")
        with self.assertRaises(Exception):
            pending.commit()
        self.assertEqual(path.read_bytes(), b"another publisher")
        self.assertEqual(list(self.root.iterdir()), [path])

    def test_existing_and_dangling_destinations_refuse(self):
        path = self.root / "existing.png"
        path.write_bytes(b"keep")
        with self.assertRaises(FileExistsError):
            self.snapshot.save_png(path)
        self.assertEqual(path.read_bytes(), b"keep")
        link = self.root / "link.png"
        link.symlink_to(self.root / "missing")
        with self.assertRaises(FileExistsError):
            self.snapshot.prepare_png(link)
        self.assertTrue(link.is_symlink())

    def test_snapshot_is_frozen_and_camera_save_does_not_recapture(self):
        before = self.snapshot.pixels()
        self.mob.shift(m.RIGHT)
        self.mob.add_updater(lambda mob, dt: self.fail("saving must not run an updater"), call=False)
        saved = self.verify(self.scene.camera.save_png(self.root / "last.png"))
        original = self.verify(self.snapshot.save_png(self.root / "frozen.png"))
        self.assertEqual(saved, original)
        self.assertEqual(self.snapshot.pixels(), before)
        self.assertIsNot(self.scene.camera.capture_snapshot(*self.scene.mobjects), self.snapshot)

    def test_scene_file_writer_uses_the_snapshot_and_its_configured_path(self):
        writer = self.scene.file_writer
        writer.output_directory = str(self.root / "writer")
        writer.file_name = "final"
        receipt = writer.save_final_image(self.snapshot)
        self.assertEqual(Path(receipt[0]), self.root / "writer" / "final.png")
        self.verify(receipt)
        with self.assertRaises(TypeError):
            writer.save_final_image(np.zeros((54, 96, 4), np.uint8))

    def test_invalid_worker_budget_has_no_filesystem_effect(self):
        for threads in (0, 97):
            with self.assertRaises(ValueError):
                self.snapshot.prepare_png(self.root / "invalid.png", threads=threads)
        self.assertEqual(list(self.root.iterdir()), [])


suite = unittest.defaultTestLoader.loadTestsFromTestCase(CaptureOutput)
assert suite.countTestCases() == 9
if not unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful():
    raise AssertionError("native capture output acceptance failed")
