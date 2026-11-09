"""Real filesystem polling; explicit doubles for scene reconstruction only."""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from fmn_python.project_autorebuild import _RebuildWatch


class RebuildWatchTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.path = self.root / 'scene.py'
        self.path.write_text('VALUE = 1\n')
        self.project = SimpleNamespace(generation=1, _source=SimpleNamespace(
            path=self.path, root=self.root,
            source_digests={self.path: self.digest()},
        ))
        self.attempts = []
        self.failure = None
        self.after_build = None
        self.checked = 0

    def digest(self):
        return hashlib.sha256(self.path.read_bytes()).hexdigest()

    def check(self):
        self.checked += 1

    def rebuild(self, *, if_changed):
        self.attempts.append(if_changed)
        if self.failure is not None:
            raise self.failure
        digest = self.digest()
        if if_changed and self.project._source.source_digests[self.path] == digest:
            return
        compile(self.path.read_bytes(), str(self.path), 'exec')
        self.project._source.source_digests[self.path] = digest
        self.project.generation += 1
        if self.after_build is not None:
            self.after_build()

    def watch(self, **kwargs):
        return _RebuildWatch(self.project, self.check, self.rebuild, **kwargs)

    def test_initial_unchanged_source_keeps_interactive_generation(self):
        watch = self.watch()
        self.assertFalse(watch.poll())
        self.assertFalse(watch.poll())
        self.assertEqual(self.attempts, [True])
        self.assertEqual(self.project.generation, 1)
        self.assertEqual(self.checked, 2)

    def test_edits_before_enabling_are_not_lost(self):
        self.path.write_text('VALUE = 2\n')
        self.assertTrue(self.watch().poll())
        self.assertEqual(self.attempts, [True])

    def test_equal_size_equal_timestamp_edit_rebuilds_once(self):
        watch = self.watch()
        watch.poll()
        before = self.path.stat()
        self.path.write_text('VALUE = 2\n')
        os.utime(self.path, ns=(before.st_atime_ns, before.st_mtime_ns))
        self.assertTrue(watch.poll())
        self.assertFalse(watch.poll())
        self.assertEqual(self.attempts, [True, False])

    def test_new_python_helper_forces_reconstruction(self):
        watch = self.watch()
        watch.poll()
        (self.root / 'new_helper.py').write_text('VALUE = 3\n')
        self.assertTrue(watch.poll())
        self.assertEqual(self.project.generation, 2)

    def test_explicit_asset_bytes_trigger_rebuild_without_source_edits(self):
        asset = self.root / 'data.txt'
        asset.write_text('one')
        watch = self.watch(paths=[asset])
        watch.poll()
        asset.write_text('two')
        self.assertTrue(watch.poll())
        self.assertFalse(watch.poll())

    def test_asset_edit_between_activation_and_first_cell_is_detected(self):
        asset = self.root / 'data.txt'
        asset.write_text('one')
        watch = self.watch(paths=[asset])
        watch.arm()
        asset.write_text('two')
        self.assertTrue(watch.poll())
        self.assertEqual(self.attempts, [False])

    def test_failed_generation_is_not_reexecuted_until_content_changes(self):
        watch = self.watch()
        watch.poll()
        self.path.write_text('VALUE = 2\n')
        self.failure = ValueError('broken constructor')
        with self.assertRaisesRegex(ValueError, 'broken constructor'):
            watch.poll()
        self.assertFalse(watch.poll())
        self.assertEqual(self.project.generation, 1)
        self.failure = None
        (self.root / 'fixed_helper.py').write_text('VALUE = 3\n')
        self.assertTrue(watch.poll())
        self.assertEqual(self.attempts, [True, False, False])

    def test_debounce_coalesces_edits_without_sleeping_or_running_threads(self):
        watch = self.watch(debounce=0.5)
        with patch('fmn_python.project_autorebuild.time.monotonic') as clock:
            clock.return_value = 0
            self.assertFalse(watch.poll())
            self.path.write_text('VALUE = 2\n')
            clock.return_value = 0.4
            self.assertFalse(watch.poll())
            clock.return_value = 0.8
            self.assertFalse(watch.poll())
            clock.return_value = 1.0
            self.assertTrue(watch.poll())
        self.assertEqual(self.attempts, [False])

    def test_edits_during_build_remain_pending(self):
        watch = self.watch()
        watch.poll()
        self.path.write_text('VALUE = 2\n')
        self.after_build = lambda: self.path.write_text('VALUE = 3\n')
        self.assertTrue(watch.poll())
        self.after_build = None
        self.assertTrue(watch.poll())
        self.assertFalse(watch.poll())
        self.assertEqual(self.project.generation, 3)

    def test_scan_failure_reports_once_and_recovers(self):
        watch = self.watch()
        with patch('fmn_python.project_autorebuild._snapshot', side_effect=OSError('unreadable')):
            with self.assertRaisesRegex(OSError, 'unreadable'):
                watch.poll()
            self.assertFalse(watch.poll())
        self.assertTrue(watch.poll())
        self.assertEqual(self.attempts, [False])

    def test_scan_outage_resets_debounce(self):
        watch = self.watch(debounce=1)
        with patch('fmn_python.project_autorebuild.time.monotonic') as clock:
            clock.return_value = 0
            watch.poll()
            with patch('fmn_python.project_autorebuild._snapshot', side_effect=OSError('offline')):
                with self.assertRaises(OSError):
                    watch.poll()
            clock.return_value = 2
            self.assertFalse(watch.poll())
            clock.return_value = 3
            self.assertTrue(watch.poll())

    def test_ownership_failure_does_not_consume_pending_edit(self):
        watch = self.watch()
        self.path.write_text('VALUE = 2\n')
        watch.check = lambda: (_ for _ in ()).throw(RuntimeError('foreign thread'))
        with self.assertRaisesRegex(RuntimeError, 'foreign thread'):
            watch.poll()
        self.assertEqual(self.attempts, [])
        watch.check = self.check
        self.assertTrue(watch.poll())

    def test_nested_poll_is_refused_and_busy_flag_recovers(self):
        watch = self.watch()
        self.path.write_text('VALUE = 2\n')
        self.after_build = watch.poll
        with self.assertRaisesRegex(RuntimeError, 'already polling'):
            watch.poll()
        self.assertFalse(watch.busy)
        self.assertFalse(watch.poll())

    def test_keyboard_interrupt_is_not_downgraded_or_repeated(self):
        watch = self.watch()
        self.failure = KeyboardInterrupt()
        with self.assertRaises(KeyboardInterrupt):
            watch.poll()
        self.assertFalse(watch.poll())
        self.assertFalse(watch.busy)

    def test_close_releases_project_and_rejects_poll(self):
        watch = self.watch()
        watch.close()
        watch.close()
        self.assertIsNone(watch.project)
        self.assertIsNone(watch.check)
        self.assertIsNone(watch.rebuild)
        with self.assertRaisesRegex(RuntimeError, 'closed'):
            watch.poll()

    def test_invalid_options_are_rejected(self):
        for value in (True, '1', None):
            with self.subTest(value=value), self.assertRaises(TypeError):
                self.watch(debounce=value)
        for value in (-1, float('nan'), float('inf'), 10**1000):
            with self.subTest(value=str(value)[:20]), self.assertRaises(ValueError):
                self.watch(debounce=value)
        for value in ('asset.txt', b'asset.txt', Path('asset.txt')):
            with self.subTest(value=value), self.assertRaises(TypeError):
                self.watch(paths=value)
        with patch('fmn_python.project_autorebuild._MAX_FILES', 2), self.assertRaises(ValueError):
            self.watch(paths=['one', 'two', 'three'])

    @unittest.skipUnless(hasattr(os, 'mkfifo'), 'FIFO requires POSIX')
    def test_non_regular_asset_is_rejected_without_blocking(self):
        fifo = self.root / 'pipe'
        os.mkfifo(fifo)
        watch = self.watch(paths=[fifo])
        with self.assertRaisesRegex(ValueError, 'regular file'):
            watch.poll()
        self.assertFalse(watch.poll())
        self.assertEqual(self.attempts, [])


if __name__ == '__main__':
    unittest.main()
