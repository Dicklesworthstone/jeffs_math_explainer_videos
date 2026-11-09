"""Host display policy and editor transactions; native images are explicit doubles."""
from __future__ import annotations

from contextlib import redirect_stdout
import io
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import test_project_editor as fixtures
from fmn_python.terminal_preview import _show_snapshot, _validate_project_preview
from fmn_python.project_autorebuild import _STATE, _watch_options


class Tty(io.StringIO):
    def isatty(self):
        return True


class Image:
    def __init__(self, payload=b'\x1b_Gnative-payload\x1b\\'):
        self.payload = payload
        self.calls = []
        self.error = None

    def terminal_bytes(self, protocol, *, max_bytes):
        self.calls.append((protocol, max_bytes))
        if self.error:
            raise self.error
        return self.payload


class TerminalTransportTests(unittest.TestCase):
    def setUp(self):
        self.native = SimpleNamespace(_CameraCapture=Image)
        self.image = Image()

    def show(self, stream=None, **kwargs):
        return _show_snapshot(self.image, kwargs.get('protocol', 'kitty'), stream,
                              kwargs.get('max_bytes', 16777216), self.native)

    def test_explicit_stream_receives_only_native_encoded_image(self):
        output = io.StringIO()
        self.assertIs(self.show(output), self.image)
        self.assertEqual(output.getvalue().encode('ascii'), self.image.payload)
        self.assertEqual(self.image.calls, [('kitty', 16777216)])

    def test_default_output_requires_terminal_and_uses_current_stdout(self):
        with redirect_stdout(io.StringIO()):
            with self.assertRaisesRegex(RuntimeError, 'terminal stdout'):
                self.show()
        self.assertEqual(self.image.calls, [])
        output = Tty()
        with redirect_stdout(output):
            self.show()
        self.assertEqual(output.getvalue().encode('ascii'), self.image.payload)

    def test_short_writes_are_completed_and_bad_writers_refuse(self):
        class Short(Tty):
            def write(self, text):
                return super().write(text[:3])
        output = Short()
        self.show(output)
        self.assertEqual(output.getvalue().encode('ascii'), self.image.payload)
        for value in (None, 0, -1, True, 9999):
            with self.subTest(value=value):
                bad = SimpleNamespace(write=lambda text: value, flush=lambda: None)
                with self.assertRaisesRegex(OSError, 'progress'):
                    self.show(bad)

    def test_encoding_failure_happens_before_any_terminal_bytes(self):
        self.image.error = ValueError('budget exceeded')
        output = io.StringIO()
        with self.assertRaisesRegex(ValueError, 'budget exceeded'):
            self.show(output)
        self.assertEqual(output.getvalue(), '')

    def test_protocol_native_capability_and_budget_are_validated(self):
        output = io.StringIO()
        for protocol in (True, 'auto', '\x1b_G', 'Kitty', None):
            with self.assertRaises((ValueError, TypeError)):
                self.show(output, protocol=protocol)
        for value in (True, 0, -1, 134217729, '1'):
            with self.assertRaises((ValueError, TypeError)):
                self.show(output, max_bytes=value)
        self.assertEqual(self.image.calls, [])
        self.native._CameraCapture = type('OldCapture', (), {})
        with self.assertRaisesRegex(RuntimeError, 'unavailable'):
            self.show(output)
        self.assertEqual(output.getvalue(), '')

    def test_preview_cannot_accept_a_foreign_image_or_capture_disabled_project(self):
        with self.assertRaises(TypeError):
            _show_snapshot(object(), 'kitty', io.StringIO(), 1000, self.native)
        project = SimpleNamespace(capture=False, preview=self.image, _native=self.native)
        with self.assertRaisesRegex(ValueError, 'capture=True'):
            _validate_project_preview(project, 'kitty')
        with self.assertRaises(ValueError):
            _watch_options(preview_protocol='not-an-image-protocol')

    def test_io_failure_preserves_the_exact_primary_exception(self):
        failure = BrokenPipeError('terminal closed')
        def write(text):
            raise failure
        with self.assertRaises(BrokenPipeError) as caught:
            self.show(SimpleNamespace(write=write, flush=lambda: None))
        self.assertIs(caught.exception, failure)


@unittest.skipUnless(fixtures.HAVE_IPYTHON, 'IPython is not installed')
class TerminalEditorTests(unittest.TestCase):
    setUp = fixtures.ProjectEditorTests.setUp
    write = fixtures.ProjectEditorTests.write
    source = fixtures.ProjectEditorTests.source
    project = fixtures.ProjectEditorTests.project
    edit = fixtures.ProjectEditorTests.edit
    cell = fixtures.ProjectEditorTests.cell

    def native_image(self, project):
        self.captured = Tty()
        self.native._CameraCapture = type(project.preview)
        self.encoded = []
        def encode(image, protocol, *, max_bytes):
            self.encoded.append((image, protocol, max_bytes))
            return ('\x1b_Gframe=' + repr(image.values) + '\x1b\\').encode('ascii')
        context = patch.object(self.native._CameraCapture, 'terminal_bytes', encode, create=True)
        context.start(); self.addCleanup(context.stop)

    def test_host_options_display_initial_and_rebuilt_native_snapshots_only(self):
        with self.project(self.source()) as project:
            self.native_image(project)
            initial = project.preview
            def interact(shell):
                self.assertEqual(len(self.encoded), 1)
                self.assertIs(self.encoded[0][0], initial)
                state = vars(project._editor)[_STATE]
                self.assertEqual(state.poll_interval, .1)
                self.assertIsNone(state.prompt)
                self.assertFalse(state.poll())
                self.assertEqual(len(self.encoded), 1)
                self.source(9)
                self.assertTrue(state.poll())
                self.assertEqual(project.scene.mobjects, [9])
                self.assertIs(self.encoded[-1][0], project.preview)
                self.assertEqual(len(self.encoded), 2)
            self.edit(project, interact, auto_rebuild=True, idle=False,
                      poll_interval=.1, preview_protocol='kitty')

    def test_terminal_failure_reports_new_generation_active_without_rebuilding_it_again(self):
        with self.project(self.source()) as project:
            self.native_image(project)
            def interact(shell):
                state = vars(project._editor)[_STATE]
                self.source(6)
                failure = BrokenPipeError('no terminal')
                with patch.object(self.captured, 'write', side_effect=failure):
                    with self.assertRaises(BrokenPipeError) as caught:
                        state.poll()
                self.assertIs(caught.exception, failure)
                self.assertTrue(any('new scene generation is active' in note for note in failure.__notes__))
                self.assertEqual(project.generation, 2)
                self.assertEqual(project.scene.mobjects, [6])
                count = self.native.constructed
                self.assertFalse(state.poll())
                self.assertEqual(self.native.constructed, count)
            self.edit(project, interact, auto_rebuild=True, preview_protocol='sixel')

    def test_preview_shortcut_is_opt_in_and_uses_current_scene_image(self):
        with self.project(self.source()) as project:
            self.native_image(project)
            def interact(shell):
                image = shell.user_ns['preview']()
                self.assertEqual(self.encoded, [])
                before = project.scene.clock
                shown = shell.user_ns['preview'](protocol='kitty')
                self.assertIs(self.encoded[-1][0], shown)
                self.assertEqual(project.scene.clock, before)
                self.assertEqual(image.values, shown.values)
            self.edit(project, interact)

    def test_invalid_preview_mode_preserves_working_watch_and_console(self):
        with self.project(self.source()) as project:
            self.native_image(project)
            def interact(shell):
                state = vars(project._editor)[_STATE]
                console = project._editor._fmn_console
                with self.assertRaises(ValueError):
                    project._editor.auto_rebuild(preview_protocol='auto')
                self.assertIs(vars(project._editor)[_STATE], state)
                self.assertIs(project._editor._fmn_console, console)
                self.assertFalse(state.closed)
                self.source(2)
                self.assertTrue(state.poll())
            self.edit(project, interact, auto_rebuild=True)


class TerminalCliTests(unittest.TestCase):
    setUp = fixtures.ProjectEditCliTests.setUp
    run_cli = fixtures.ProjectEditCliTests.run_cli

    def test_missing_native_terminal_capability_refuses_before_loading_scene(self):
        self.path.write_text("raise AssertionError('must not load')\n")
        self.assertEqual(self.run_cli('edit', '--watch', '--kitty', str(self.path), 'Demo'), 4)
        self.assertEqual(self.native.constructed, 0)

    def test_terminal_options_help_is_nonexecuting_and_invalid_combinations_refuse(self):
        self.path.write_text("raise AssertionError('must not load')\n")
        self.assertEqual(self.run_cli('edit', '--kitty', '--help', '--robot'), 0)
        self.assertIn('--sixel', self.reports[-1]['help'])
        for arguments in (['--kitty'], ['--kitty', '--sixel'], ['--watch', '--kitty', '--kitty']):
            self.assertEqual(self.run_cli('edit', *arguments, str(self.path), 'Demo', '--robot'), 2)
        self.assertEqual(self.native.constructed, 0)


if __name__ == '__main__':
    unittest.main()
