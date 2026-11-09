"""Idle hook ownership and scheduling; scene work is explicitly doubled here."""
from __future__ import annotations

from contextlib import redirect_stdout
import io
import sys
import threading
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from fmn_python.project_autorebuild import _watch_options
from fmn_python.project_prompt import prepare_prompt


class Shell:
    def __init__(self):
        self.simple_prompt = False
        self.pt_app = object()
        self._inputhook = None
        self._use_asyncio_inputhook = False
        self.active_eventloop = None
        self.errors = []

    def showtraceback(self):
        self.errors.append((type(sys.exception()), str(sys.exception())))


class PromptHookTests(unittest.TestCase):
    def setUp(self):
        self.shell = Shell()
        self.calls = []
        self.watch = SimpleNamespace(idle_mode=None, poll_interval=.1, poll=self.poll,
                                     project=SimpleNamespace(scene_name='Demo', generation=2))
        self.active = True
        self.hook = prepare_prompt(self.shell, self.watch, lambda: self.active)
        self.addCleanup(self.hook.close)
        self.now = 1.0
        self.addCleanup(patch.stopall)
        patch('fmn_python.project_prompt.time.monotonic', lambda: self.now).start()
        patch('fmn_python.project_prompt.time.sleep', self.sleep).start()

    def sleep(self, delay):
        self.assertGreater(delay, 0)
        self.assertLessEqual(delay, .02)
        self.now += delay

    def poll(self):
        self.calls.append(self.now)
        return False

    def context(self, limit=1):
        return SimpleNamespace(input_is_ready=lambda: len(self.calls) >= limit)

    def test_uses_only_owner_shell_and_restores_original_fields(self):
        self.hook.install()
        self.assertIs(self.shell._inputhook, self.hook.callback)
        self.hook.run(self.context(3))
        self.assertEqual(len(self.calls), 3)
        self.assertTrue(all(b-a >= .1-1e-12 for a,b in zip(self.calls,self.calls[1:])))
        retained = self.hook.callback
        self.hook.close()
        self.assertIsNone(self.shell._inputhook)
        self.assertIsNone(self.shell.active_eventloop)
        retained(self.context(100))
        self.assertEqual(len(self.calls), 3)
        self.assertIsNone(self.hook.watch)

    def test_ready_input_never_runs_authored_work(self):
        self.hook.install()
        self.hook.run(SimpleNamespace(input_is_ready=lambda: True))
        self.assertEqual(self.calls, [])

    def test_stop_during_poll_returns_without_touching_released_watch(self):
        def poll():
            self.calls.append(self.now)
            self.hook.close()
            return False
        self.watch.poll = poll
        self.hook.install()
        self.hook.run(self.context(100))
        self.assertEqual(len(self.calls), 1)

    def test_nested_prompt_hook_cannot_reenter_scene_work(self):
        def poll():
            self.calls.append(self.now)
            self.hook.run(self.context(100))
            return False
        self.watch.poll = poll
        self.hook.install()
        self.hook.run(self.context())
        self.assertEqual(len(self.calls), 1)

    def test_foreign_thread_cannot_poll_or_release_native_owners(self):
        self.hook.install()
        errors = []
        def foreign():
            for operation in (lambda: self.hook.run(self.context()), self.hook.close):
                try:
                    operation()
                except RuntimeError as error:
                    errors.append(str(error))
        worker = threading.Thread(target=foreign)
        worker.start(); worker.join(2)
        self.assertFalse(worker.is_alive())
        self.assertEqual(len(errors), 2)
        self.assertEqual(self.calls, [])
        self.assertFalse(self.hook.closed)

    def test_does_not_clobber_new_gui_selected_by_user(self):
        self.hook.install()
        other = lambda context: None
        self.shell._inputhook, self.shell.active_eventloop = other, 'other'
        self.hook.run(self.context(100))
        self.hook.close()
        self.assertIs(self.shell._inputhook, other)
        self.assertEqual(self.shell.active_eventloop, 'other')
        self.assertEqual(self.calls, [])

    def test_reports_repeated_ownership_error_once_without_retaining_exception(self):
        def poll():
            self.calls.append(self.now)
            raise ValueError('bad scene')
        self.watch.poll = poll
        self.hook.install()
        self.hook.run(self.context(3))
        self.assertEqual(self.shell.errors, [(ValueError, 'bad scene')])
        self.assertEqual(self.hook.reported, (ValueError, 'bad scene'))
        self.assertFalse(self.hook.busy)

    def test_interrupts_are_not_reported_as_ordinary_source_failures(self):
        for error in (KeyboardInterrupt(), SystemExit(17)):
            with self.subTest(error=type(error).__name__):
                def poll():
                    raise error
                self.watch.poll = poll
                if not self.hook.installed:
                    self.hook.install()
                with self.assertRaises(type(error)) as caught:
                    self.hook.run(self.context(100))
                self.assertIs(caught.exception, error)
                self.assertFalse(self.hook.busy)
                self.assertEqual(self.shell.errors, [])

    def test_success_is_reported_once_and_next_invocation_obeys_poll_interval(self):
        def poll():
            self.calls.append(self.now)
            return True
        self.watch.poll = poll
        self.hook.install()
        output = io.StringIO()
        with redirect_stdout(output):
            self.hook.run(self.context())
        self.assertIn('Rebuilt Demo (generation 2)', output.getvalue())
        first = self.calls[0]
        self.hook.run(self.context(2))
        self.assertGreaterEqual(self.calls[1]-first, .1-1e-12)

    def test_stale_owner_and_replaced_hook_are_inert(self):
        self.hook.install()
        self.active = False
        self.hook.run(self.context(100))
        self.assertEqual(self.calls, [])

    def test_prepare_refuses_explicit_unsupported_mode_without_mutating_shell(self):
        for attr, value in [('simple_prompt', True), ('pt_app', None),
                            ('_use_asyncio_inputhook', True), ('_inputhook', lambda context: None)]:
            with self.subTest(attribute=attr):
                shell = Shell(); setattr(shell, attr, value)
                self.watch.idle_mode = None
                self.assertIsNone(prepare_prompt(shell, self.watch, lambda: True))
                self.watch.idle_mode = True
                before = vars(shell).copy()
                with self.assertRaisesRegex(RuntimeError, 'idle=False'):
                    prepare_prompt(shell, self.watch, lambda: True)
                self.assertEqual(vars(shell), before)

    def test_explicit_cell_mode_and_replacement_preflight(self):
        self.watch.idle_mode = False
        self.assertIsNone(prepare_prompt(self.shell, self.watch, lambda: True))
        self.watch.idle_mode = True
        self.hook.install()
        candidate = prepare_prompt(self.shell, self.watch, lambda: True, replacing=self.hook)
        with self.assertRaises(RuntimeError):
            candidate.install()
        self.hook.close()
        candidate.install()
        candidate.close()
        self.assertIsNone(self.shell._inputhook)

    def test_idle_options_are_frozen_and_bounded(self):
        self.assertEqual(_watch_options()['poll_interval'], .25)
        for idle in (1, 'yes', [], object()):
            with self.assertRaises(TypeError):
                _watch_options(idle=idle)
        for interval in (0, -.01, .009, 61, float('nan'), float('inf'), True):
            with self.assertRaises((TypeError, ValueError)):
                _watch_options(poll_interval=interval)


if __name__ == '__main__':
    unittest.main()
