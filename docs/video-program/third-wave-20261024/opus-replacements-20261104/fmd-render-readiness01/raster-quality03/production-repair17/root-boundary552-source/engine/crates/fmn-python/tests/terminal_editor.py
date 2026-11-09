"""Real native scene edits and terminal frames before an IPython cell executes.

Only keyboard/source-file I/O is scripted. Scene, import transactions, Lumen,
Kitty encoder, IPython, prompt_toolkit and stdout integration are production.
"""
from __future__ import annotations

import base64
from contextlib import redirect_stderr, redirect_stdout
import io
from pathlib import Path
import re
import tempfile
import threading
import unittest
from unittest.mock import patch

from IPython.terminal.embed import InteractiveShellEmbed
from IPython.terminal.interactiveshell import TerminalInteractiveShell
from prompt_toolkit import PromptSession
from prompt_toolkit.application import create_app_session
from prompt_toolkit.data_structures import Size
from prompt_toolkit.input import create_pipe_input
from prompt_toolkit.output.vt100 import Vt100_Output

import manimlib as m
from fmn_python import SceneProject
from fmn_python.project_autorebuild import _STATE
from fmn_python.terminal_preview import show_snapshot


class Terminal(io.StringIO):
    def isatty(self):
        return True


def images(text):
    frames, pending = [], []
    for controls, payload in re.findall(r'\x1b_G([^;]*);([^\x1b]*)\x1b\\', text):
        if 'a=T' in controls:
            assert not pending
        pending.append(payload)
        if controls.endswith('m=0'):
            frames.append(base64.b64decode(''.join(pending), validate=True))
            pending = []
    assert not pending
    return frames


class NativeTerminalEditorTests(unittest.TestCase):
    def source(self, x, color='RED', *, broken=False):
        failure = "        raise ValueError('broken saved scene')\n" if broken else ''
        return ("from manimlib import *\nclass Demo(Scene):\n    def construct(self):\n"
                "        self.camera.reset_pixel_shape(96, 64)\n"
                f"        square = Square(side_length=1, fill_color={color}, fill_opacity=1, stroke_width=0)\n"
                f"        square.shift(({x}, 0, 0))\n"
                "        self.add(square)\n        self.wait(.1)\n" + failure + "        self.embed()\n"
                "        raise AssertionError('code after Scene.embed must not run in a project')\n").encode()

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='fmn-native-terminal-')
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / 'scene.py'
        self.path.write_bytes(self.source(-2))
        self.output, self.errors = Terminal(), io.StringIO()

    def drive(self, *, recover):
        with SceneProject(self.path, 'Demo') as project:
            initial = project.preview.png()
            old_scene = project.scene
            native_threads, failures, writer_errors = [], [], []
            owner = threading.get_ident()
            ready, rebuilt, failed = threading.Event(), threading.Event(), threading.Event()
            with create_pipe_input() as keyboard:
                output = Vt100_Output(self.output, lambda: Size(rows=40, columns=120),
                                     term='xterm-kitty', enable_cpr=False)
                with create_app_session(input=keyboard, output=output):
                    def interact(shell, *args, **kwargs):
                        self.addCleanup(shell.history_manager.end_session)
                        shell.simple_prompt = False
                        shell._use_asyncio_inputhook = False
                        shell.pt_app = PromptSession(input=keyboard, output=output,
                            key_bindings=shell._merge_shortcuts(user_shortcuts=shell.shortcuts))
                        shell.pt_app.app.before_render += lambda app: ready.set()
                        shell.user_ns['auto_rebuild'](idle=True, poll_interval=.02,
                                                       preview_protocol='kitty')
                        state = vars(project._editor)[_STATE]
                        original = state.rebuild
                        def rebuild(**options):
                            native_threads.append(threading.get_ident())
                            result = original(**options)
                            if project.generation > 1:
                                rebuilt.set()
                            return result
                        state.rebuild = rebuild
                        traceback = shell.showtraceback
                        def report(*args, **kwargs):
                            failures.append((project.generation, project.preview.png()))
                            failed.set()
                            return traceback(*args, **kwargs)
                        shell.showtraceback = report
                        def edit_source():
                            try:
                                if not ready.wait(5):
                                    raise AssertionError('prompt did not become ready')
                                keyboard.send_text('saved = ')
                                if recover:
                                    self.path.write_bytes(self.source(1, broken=True))
                                    if not failed.wait(5):
                                        raise AssertionError('failed saved scene was not reported')
                                self.path.write_bytes(self.source(2, 'BLUE'))
                                if not rebuilt.wait(5):
                                    raise AssertionError('saved scene did not rebuild while idle')
                            except BaseException as error:
                                writer_errors.append(error)
                            finally:
                                keyboard.send_text('42\n')
                        worker = threading.Thread(target=edit_source, name='test-terminal-io')
                        worker.start()
                        before = shell.execution_count
                        try:
                            code = TerminalInteractiveShell.prompt_for_code(shell)
                        finally:
                            worker.join(6)
                            shell.showtraceback = traceback
                        self.assertFalse(worker.is_alive())
                        self.assertFalse(writer_errors, str(writer_errors) + self.errors.getvalue())
                        self.assertEqual(code, 'saved = 42')
                        self.assertEqual(shell.execution_count, before)
                        self.assertNotIn('saved', shell.user_ns)
                        self.assertEqual(project.generation, 2)
                        self.assertIs(shell.user_ns['scene'], project.scene)
                        self.assertAlmostEqual(shell.user_ns['square'].get_center()[0], 2)
                        self.assertAlmostEqual(old_scene.mobjects[0].get_center()[0], -2)
                        self.assertNotEqual(project.preview.png(), initial)
                    with patch.object(InteractiveShellEmbed, 'interact', interact), \
                            redirect_stdout(self.output), redirect_stderr(self.errors):
                        shell = project.edit(auto_rebuild=True, preview_protocol='kitty')
            self.assertEqual(set(native_threads), {owner})
            self.assertIsNone(shell._inputhook)
            frames = images(self.output.getvalue())
            self.assertGreaterEqual(len(frames), 2)
            self.assertIn(initial, frames)
            self.assertIn(project.preview.png(), frames)
            self.assertEqual(failures, [(1, initial)] if recover else [])

    def test_save_rebuilds_and_displays_changed_native_pixels_while_prompt_is_idle(self):
        self.drive(recover=False)

    def test_bad_save_retains_old_pixels_and_recovers_without_entering_a_cell(self):
        self.drive(recover=True)

    def test_base_scene_embed_remains_a_headless_noop_outside_a_project(self):
        scene = m.Scene()
        clock = scene.time
        with patch.object(InteractiveShellEmbed, '__call__', side_effect=AssertionError('unexpected terminal')):
            self.assertIsNone(scene.embed(close_scene_on_exit=False, show_animation_progress=True))
        self.assertEqual(scene.time, clock)

    def test_explicit_snapshot_display_has_no_scene_or_clock_effects(self):
        with SceneProject(self.path, 'Demo') as project:
            snapshot = project.preview
            clock = project.scene.time
            png = snapshot.png()
            stream = io.StringIO()
            self.assertIs(show_snapshot(snapshot, protocol='kitty', stream=stream), snapshot)
            self.assertEqual(images(stream.getvalue()), [png])
            self.assertEqual(project.scene.time, clock)
            self.assertEqual(project.generation, 1)
            project.scene.mobjects[0].shift((1, 0, 0))
            self.assertEqual(snapshot.png(), png)


if __name__ == '__main__':
    unittest.main()
