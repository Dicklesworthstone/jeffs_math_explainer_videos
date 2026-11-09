"""Fail-closed checkpoint I/O checks using the existing render protocol fixture."""
import pathlib
import unittest
from unittest.mock import patch

import test_batch_checkpoint_helpers as fixtures

checkpoint_module = fixtures.checkpoint_module


class CheckpointIoTests(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.CheckpointTests()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)

    def test_unreadable_sequence_subdirectory_cannot_be_ignored(self):
        self.fixture.run_batch(format="png_sequence")
        blocked = self.fixture.output / "first/frames/unreadable"
        blocked.mkdir()
        (blocked / "extra.png").write_bytes(b"unknown frame")
        original = checkpoint_module.os.scandir
        def deny(path):
            if pathlib.Path(path) == blocked:
                raise PermissionError("inaccessible frame directory")
            return original(path)
        self.fixture.calls.clear()
        before = self.fixture.journal.read_bytes()
        with patch.object(checkpoint_module.os, "scandir", deny):
            with self.assertRaisesRegex(PermissionError, "inaccessible frame"):
                self.fixture.run_batch(format="png_sequence", resume=True)
        self.assertEqual(self.fixture.calls, [])
        self.assertEqual(self.fixture.journal.read_bytes(), before)

    @unittest.skipUnless(hasattr(checkpoint_module.os, "mkfifo"), "requires POSIX FIFOs")
    def test_fifo_checkpoint_is_rejected_without_waiting_for_writer(self):
        checkpoint_module.os.mkfifo(self.fixture.journal)
        with self.assertRaisesRegex(ValueError, "regular file"):
            self.fixture.run_batch(resume=True)
        self.assertEqual(self.fixture.calls, [])

    def test_directory_checkpoint_is_rejected_before_execution(self):
        self.fixture.journal.mkdir()
        with self.assertRaisesRegex(ValueError, "regular file"):
            self.fixture.run_batch(resume=True)
        self.assertEqual(self.fixture.calls, [])

    def test_oversized_journal_fails_before_parsing(self):
        with self.fixture.journal.open("wb") as stream:
            stream.truncate(checkpoint_module._MAX_BYTES + 1)
        with patch.object(checkpoint_module.json, "loads", side_effect=AssertionError("must not parse")):
            with self.assertRaisesRegex(ValueError, "byte budget"):
                self.fixture.run_batch(resume=True)
        self.assertEqual(self.fixture.calls, [])

    def test_replaced_input_after_lstat_is_rejected(self):
        self.fixture.run_batch()
        replacement = self.fixture.root / "replacement.json"
        replacement.write_bytes(self.fixture.journal.read_bytes())
        original = checkpoint_module.os.open
        replaced = False
        def swap(path, flags, *args, **kwargs):
            nonlocal replaced
            if not replaced and pathlib.Path(path) == self.fixture.journal:
                replaced = True
                checkpoint_module.os.replace(replacement, self.fixture.journal)
            return original(path, flags, *args, **kwargs)
        self.fixture.calls.clear()
        with patch.object(checkpoint_module.os, "open", swap):
            with self.assertRaisesRegex(ValueError, "changed while opening"):
                self.fixture.run_batch(resume=True)
        self.assertTrue(replaced)
        self.assertEqual(self.fixture.calls, [])
        self.assertTrue(self.fixture.run_batch(resume=True).ok)

    def test_checkpoint_and_lock_cannot_obstruct_output_ancestors(self):
        for checkpoint, output in (
            (self.fixture.output, self.fixture.output),
            (self.fixture.journal, self.fixture.journal.with_name(self.fixture.journal.name + ".lock")),
        ):
            with self.subTest(checkpoint=checkpoint, output=output):
                with self.assertRaisesRegex(ValueError, "outside published artifacts"):
                    fixtures.batch.render_scenes(
                        self.fixture.jobs, output, format="png", resolution=(96, 54),
                        fps=8, threads=1, checkpoint=checkpoint, resume_key="inputs-v1")
                self.assertFalse(checkpoint.exists())
                self.assertFalse(output.exists())
        self.assertEqual(self.fixture.calls, [])


if __name__ == "__main__":
    unittest.main()
