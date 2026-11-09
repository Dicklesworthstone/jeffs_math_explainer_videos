"""Real IPython and prompt_toolkit polling with explicit native lifecycle doubles.

A helper performs filesystem/terminal I/O only. Production source import,
reconstruction, console replacement and preview run on the test's owner thread.
"""
from __future__ import annotations

from contextlib import redirect_stdout, redirect_stderr
import io
import threading
import time
import unittest
from unittest.mock import patch

import test_project_editor as fixtures
from fmn_python.project_autorebuild import _STATE

if fixtures.HAVE_IPYTHON:
    from IPython.terminal.interactiveshell import TerminalInteractiveShell
    from prompt_toolkit import PromptSession
    from prompt_toolkit.input import create_pipe_input
    from prompt_toolkit.output import DummyOutput


@unittest.skipUnless(fixtures.HAVE_IPYTHON, 'IPython is not installed')
class PromptEditorTests(unittest.TestCase):
    setUp = fixtures.ProjectEditorTests.setUp
    write = fixtures.ProjectEditorTests.write
    source = fixtures.ProjectEditorTests.source
    project = fixtures.ProjectEditorTests.project
    edit = fixtures.ProjectEditorTests.edit

    def run_prompt(self, shell, project, edits, *, fail=False):
        """Drive real prompt readiness, not a hand-invoked pre-cell callback."""
        ready, rebuilt, failed = (threading.Event() for _ in range(3))
        observations, writer_errors, work_threads = [], [], []
        owner = threading.get_ident()
        with create_pipe_input() as terminal:
            shell.simple_prompt = False
            shell._use_asyncio_inputhook = False
            shell.pt_app = PromptSession(input=terminal, output=DummyOutput(),
                                         key_bindings=shell._merge_shortcuts(user_shortcuts=shell.shortcuts))
            shell.pt_app.app.before_render += lambda app: ready.set()
            shell.user_ns['auto_rebuild'](idle=True, poll_interval=.01)
            watch = vars(project._editor)[_STATE]
            self.assertIs(shell._inputhook, watch.prompt.callback)
            original = watch.rebuild
            def observe(**options):
                work_threads.append(threading.get_ident())
                value = original(**options)
                if project.generation > 1:
                    rebuilt.set()
                return value
            watch.rebuild = observe
            report = shell.showtraceback
            manager = project._editor.checkpoint_manager
            def show_error(*args, **kwargs):
                observations.append((project.generation, project._editor.checkpoint_manager is manager))
                failed.set()
                return report(*args, **kwargs)
            shell.showtraceback = show_error
            def writer():
                try:
                    if not ready.wait(3):
                        raise AssertionError('real prompt did not start')
                    terminal.send_text('kept = ')
                    # Only raw source bytes and terminal input cross threads.
                    # No Scene, project, import or native callback is touched.
                    edits[0][0].write_bytes(edits[0][1])
                    if fail:
                        if not failed.wait(3):
                            raise AssertionError('idle source failure was not reported')
                        time.sleep(.08)  # unchanged failed bytes span multiple polls
                        edits[1][0].write_bytes(edits[1][1])
                    if not rebuilt.wait(3):
                        raise AssertionError('scene did not rebuild before terminal input')
                except BaseException as error:
                    writer_errors.append(error)
                finally:
                    terminal.send_text('99\n')
            worker = threading.Thread(target=writer, name='test-source-editor')
            worker.start()
            count = shell.execution_count
            try:
                # Invoke IPython's actual prompt route: PromptSession.prompt
                # installs a real InputHookSelector, which drives our callback.
                code = TerminalInteractiveShell.prompt_for_code(shell)
            finally:
                worker.join(5)
                shell.showtraceback = report
            self.assertFalse(worker.is_alive())
            self.assertFalse(writer_errors, str(writer_errors) + self.captured.getvalue())
            self.assertEqual(code, 'kept = 99')
            self.assertEqual(shell.execution_count, count)
            self.assertNotIn('kept', shell.user_ns)
            self.assertEqual(set(work_threads), {owner})
        return observations, watch

    def test_file_save_reconstructs_before_any_cell_and_preserves_partial_input(self):
        path = self.source(7)
        edited = path.read_bytes()
        self.source(1)
        retained = []
        with self.project(path) as project:
            old = project.scene
            def interact(shell):
                before = project.preview
                observations, watch = self.run_prompt(shell, project, [(path, edited)])
                retained.append((watch, shell))
                self.assertEqual(observations, [])
                self.assertEqual(project.generation, 2)
                self.assertIsNot(project.scene, old)
                self.assertEqual(project.scene.mobjects, [7])
                self.assertIs(shell.user_ns['scene'], project.scene)
                self.assertIs(shell.user_ns['self'], project.scene)
                self.assertIsNot(project.preview, before)
            self.edit(project, interact)
        watch, shell = retained[0]
        self.assertTrue(watch.closed)
        self.assertIsNone(shell._inputhook)
        self.assertIsNone(shell.active_eventloop)

    def test_broken_edit_preserves_generation_and_recovers_without_a_cell(self):
        path = self.source(8)
        corrected = path.read_bytes()
        self.source(4, "        raise ValueError('broken idle generation')\n")
        broken = path.read_bytes()
        self.source(1)
        with self.project(path) as project:
            def interact(shell):
                observations, watch = self.run_prompt(shell, project,
                                                       [(path, broken), (path, corrected)], fail=True)
                self.assertEqual(observations, [(1, True)])
                self.assertEqual(project.generation, 2)
                self.assertEqual(project.scene.mobjects, [8])
                # Initial, failed, successful: no repeated authored execution
                # despite unchanged failed bytes spanning several idle polls.
                self.assertEqual(self.native.events.count('setup'), 3)
            self.edit(project, interact)

    def test_invalid_idle_reconfiguration_preserves_existing_watch(self):
        with self.project(self.source()) as project:
            def interact(shell):
                shell.user_ns['auto_rebuild'](idle=False)
                original = vars(project._editor)[_STATE]
                self.assertIsNone(original.prompt)
                with self.assertRaisesRegex(RuntimeError, 'terminal prompt_toolkit'):
                    shell.user_ns['auto_rebuild'](idle=True)
                self.assertIs(vars(project._editor)[_STATE], original)
                self.assertFalse(original.closed)
            self.edit(project, interact)


if __name__ == '__main__':
    unittest.main()
