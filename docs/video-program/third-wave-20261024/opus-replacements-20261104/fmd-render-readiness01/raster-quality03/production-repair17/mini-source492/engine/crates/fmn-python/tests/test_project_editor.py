"""Real IPython with production project/import/console/editor adapters.

Native Scene, checkpoints and capture are explicit lifecycle doubles. Only
terminal interaction is replaced; IPython's actual activation and cells run.
"""
from __future__ import annotations

from contextlib import nullcontext, redirect_stdout, redirect_stderr
import io
from pathlib import Path
import sys
import threading
from types import ModuleType, SimpleNamespace
import unittest
from unittest.mock import patch

try:
    from IPython.terminal.embed import InteractiveShellEmbed
    HAVE_IPYTHON = True
except ImportError:
    InteractiveShellEmbed = None
    HAVE_IPYTHON = False

import test_scene_project as fixtures
from fmn_python.scene_project import SceneProject
from fmn_python.scene_console import _OWNER, _STUDIO_WORKER
from fmn_python.embedded_shell import install_embedded_shell, _SESSION
from fmn_python.source_autoreload import install_source_autoreload
from fmn_python.project_editor import install_scene_project_editor, try_edit_cli
from fmn_python.source_reload import active_source


def editor_fixture():
    native = fixtures.native_fixture()
    class InteractiveScene(native.Scene):
        pass
    class CheckpointManager:
        pass
    class Embedded:
        def get_shortcuts(self):
            return dict(add=self.scene.add, wait=self.scene.wait, reload=self.reload_scene,
                        save_state=self.scene.get_state)
        def reload_scene(self, embed_line=None):
            raise RuntimeError("no reconstructible project recipe")
    native.InteractiveScene = InteractiveScene
    native.InteractiveSceneEmbed = Embedded
    native.CheckpointManager = CheckpointManager
    native._CapabilityError = RuntimeError
    native.Scene.is_window_closing = lambda self: False
    native.Scene.update_frame = lambda self, **kwargs: None
    native.Scene.temp_config_change = lambda self, *args: nullcontext()
    old_init = native.Scene.__init__
    def initialize(self, **kwargs):
        old_init(self, **kwargs)
        camera = self.camera
        camera.capture = lambda *objects: camera.capture_snapshot(*objects)
    native.Scene.__init__ = initialize
    install_embedded_shell(native)
    install_source_autoreload(native)
    install_scene_project_editor(native)
    native.Scene.embed = native.InteractiveScene.embed
    return native


@unittest.skipUnless(HAVE_IPYTHON, "IPython is not installed")
class ProjectEditorTests(unittest.TestCase):
    setUp = fixtures.SceneProjectTests.setUp
    write = fixtures.SceneProjectTests.write
    source = fixtures.SceneProjectTests.source
    project = fixtures.SceneProjectTests.project

    def setUp(self):
        fixtures.SceneProjectTests.setUp(self)
        self.native = editor_fixture()
        sys.modules['manimlib'] = self.native
        self.captured = io.StringIO()
        self.calls = 0

    def edit(self, project, interact, **kwargs):
        self.calls += 1
        def mainloop(shell, *args, **options):
            self.addCleanup(shell.history_manager.end_session)
            self.assertIsInstance(shell, InteractiveShellEmbed)
            interact(shell)
        with patch.object(InteractiveShellEmbed, 'interact', mainloop), \
                redirect_stdout(self.captured), redirect_stderr(self.captured):
            return project.edit(**kwargs)

    def cell(self, shell, source):
        result = shell.run_cell(source)
        self.assertTrue(result.success, str(result.error_in_exec) + self.captured.getvalue())
        return result

    def test_reload_switches_real_active_namespace_shortcuts_and_module_globals(self):
        path = self.source()
        with self.project(path) as project:
            old = project.scene
            def interact(shell):
                module = shell.user_module
                namespace = shell.user_ns
                self.cell(shell, 'def read_value(): return VALUE\ncustom=42\nold_scene=scene')
                self.source(7)
                self.cell(shell, 'reload()\nadd(8)\nobserved=read_value()')
                self.assertIs(shell.user_module, module)
                self.assertIs(shell.user_ns, namespace)
                self.assertIs(shell.user_ns['self'], project.scene)
                self.assertIs(shell.user_ns['scene'], project.scene)
                self.assertIs(shell.user_ns['project'], project)
                self.assertEqual(project.scene.mobjects, [7, 8])
                self.assertEqual(shell.user_ns['observed'], 7)
                self.assertEqual(shell.user_ns['custom'], 42)
                self.assertIs(shell.user_ns['old_scene'], old)
                self.assertEqual(old.mobjects, [1])
                self.assertEqual(project.generation, 2)
                self.assertEqual(self.cell(shell, 'image=preview()').error_in_exec, None)
                self.assertEqual(shell.user_ns['image'].values, (7, 8))
            self.edit(project, interact)
            self.assertNotIn(_SESSION, vars(project.scene))
            self.assertNotIn(_OWNER, vars(project.scene))
            self.assertNotIn(_OWNER, vars(old))
            self.assertIsNone(project._editor)

    def test_failed_rebuild_keeps_shell_scene_manager_and_checkpoint_history(self):
        path = self.source()
        with self.project(path) as project:
            old = project.scene
            def interact(shell):
                embedded = project._editor
                manager, console = embedded.checkpoint_manager, embedded._fmn_console
                manager.handle_checkpoint_key(old, '#before')
                self.cell(shell, 'add(4)')
                self.source(2, "        raise ValueError('bad candidate')\n")
                result = shell.run_cell('reload()')
                self.assertIsInstance(result.error_in_exec, ValueError)
                self.assertIs(project.scene, old)
                self.assertIs(embedded.scene, old)
                self.assertIs(embedded._fmn_console, console)
                self.assertIs(embedded.checkpoint_manager, manager)
                self.assertIn('#before', manager.checkpoint_states)
                self.assertEqual(shell.user_ns['VALUE'], 1)
                self.assertEqual(project.generation, 1)
                self.source(6)
                self.cell(shell, 'reload()')
                self.assertEqual(project.scene.mobjects, [6])
                self.assertTrue(console._closed)
                self.assertEqual(manager.checkpoint_states, {})
                self.assertIsNot(embedded.checkpoint_manager, manager)
                self.assertEqual(embedded.checkpoint_manager.checkpoint_states, {})
                with self.assertRaises(RuntimeError):
                    console.run_cell('scene.add(99)')
            self.edit(project, interact)

    def test_native_preview_failure_rolls_back_imports_and_namespaces(self):
        path = self.source()
        with self.project(path) as project:
            old, module = project.scene, project.module
            def interact(shell):
                self.source(3)
                self.native.preview_failure = OSError('bad preview')
                result = shell.run_cell('reload()')
                self.native.preview_failure = None
                self.assertIsInstance(result.error_in_exec, OSError)
                self.assertIs(shell.user_ns['scene'], old)
                self.assertIs(project.module, module)
                self.assertEqual(shell.user_ns['VALUE'], 1)
                self.cell(shell, 'reload()')
                self.assertEqual(project.scene.mobjects, [3])
            self.edit(project, interact)

    def test_interactive_overrides_survive_and_removed_owned_names_disappear(self):
        path = self.source()
        with self.project(path) as project:
            def interact(shell):
                self.cell(shell, 'VALUE=99\nadd=lambda n: n+1\ncustom=42')
                path.write_text('from manimlib import Scene\nclass Demo(Scene):\n'
                                '    def construct(self): self.add(8)\n')
                self.cell(shell, 'reload()')
                self.assertEqual(shell.user_ns['VALUE'], 99)
                self.assertEqual(shell.user_ns['add'](3), 4)
                self.assertEqual(shell.user_ns['custom'], 42)
                self.assertIs(shell.user_ns['Demo'], type(project.scene))
                self.assertEqual(project.scene.mobjects, [8])
            self.edit(project, interact)

    def test_embed_inside_construct_stops_at_checkpoint_without_starting_terminal(self):
        path = self.write('scene.py', 'from manimlib import Scene\nclass Demo(Scene):\n'
                          '    def construct(self):\n        item=object()\n        self.add(item)\n'
                          '        self.embed()\n        self.add("never")\n')
        with patch.object(InteractiveShellEmbed, '__call__', side_effect=AssertionError('unexpected terminal')):
            with self.project(path) as project:
                self.assertEqual(len(project.scene.mobjects), 1)
                self.assertIs(project.namespace['item'], project.scene.mobjects[0])
                self.assertIn('tear_down', self.native.events)
                project.rebuild()
                self.assertEqual(project.generation, 2)
        self.assertIsNone(active_source())

    def test_rebuild_refreshes_authored_embed_locals_in_same_editor(self):
        path = self.write('scene.py', 'from manimlib import Scene\nclass Demo(Scene):\n'
                          '    def construct(self):\n        item=[1]\n        self.add(item)\n        self.embed()\n')
        with self.project(path) as project:
            def interact(shell):
                item = shell.user_ns['item']
                self.cell(shell, 'saved=item')
                path.write_text(path.read_text().replace('item=[1]', 'item=[9]'))
                self.cell(shell, 'reload()')
                self.assertIsNot(shell.user_ns['item'], item)
                self.assertEqual(shell.user_ns['item'], [9])
                self.assertIs(shell.user_ns['item'], project.scene.mobjects[0])
                self.assertIs(shell.user_ns['saved'], item)
            self.edit(project, interact)

    def test_source_autoreload_and_full_reload_remain_distinct_and_compose(self):
        path = self.source(1)
        with self.project(path) as project:
            old = project.scene
            def interact(shell):
                self.cell(shell, 'auto_reload()')
                self.source(2)
                self.cell(shell, 'observed=VALUE')
                self.assertEqual(shell.user_ns['observed'], 2)
                self.assertIs(project.scene, old)
                self.assertEqual(old.mobjects, [1])
                self.cell(shell, 'reload()')
                self.assertEqual(project.scene.mobjects, [2])
                self.source(3)
                self.cell(shell, 'observed=VALUE')
                self.assertEqual(shell.user_ns['observed'], 3)
                self.assertEqual(project.scene.mobjects, [2])
                self.cell(shell, 'reload()')
                self.assertEqual(project.scene.mobjects, [3])
            self.edit(project, interact)

    def test_functions_refresh_after_auto_definition_reload_then_full_rebuild(self):
        path = self.write('scene.py', 'from manimlib import Scene\ndef value(): return 1\n'
                          'class Demo(Scene):\n    def construct(self): self.add(value())\n')
        with self.project(path) as project:
            def interact(shell):
                self.cell(shell, 'auto_reload()\ndef read_value(): return value()')
                path.write_text(path.read_text().replace('return 1', 'return 2'))
                self.cell(shell, 'reload()')
                path.write_text(path.read_text().replace('return 2', 'return 3'))
                self.cell(shell, 'observed=read_value()')
                self.assertEqual(shell.user_ns['observed'], 3)
            self.edit(project, interact)

    def test_active_checkpoint_cell_cannot_swap_its_scene_owner(self):
        with self.project(self.source()) as project:
            def interact(shell):
                console = project._editor._fmn_console
                with self.assertRaisesRegex(RuntimeError, 'console'):
                    console.run_cell('#point\nreload()')
                self.assertEqual(project.generation, 1)
                self.assertIs(console.scene, project.scene)
            self.edit(project, interact)

    def test_active_output_refusal_and_invalid_embed_line_leave_old_generation(self):
        with self.project(self.source()) as project:
            def interact(shell):
                self.cell(shell, 'self._fmn_owned_render_session=object()')
                result = shell.run_cell('reload()')
                self.assertIsInstance(result.error_in_exec, RuntimeError)
                vars(project.scene).pop('_fmn_owned_render_session')
                with self.assertRaisesRegex(RuntimeError, 'insert source lines'):
                    project._editor.reload_scene(100)
                self.assertEqual(project.generation, 1)
            self.edit(project, interact)

    def test_rebuild_and_close_from_external_reference_refuse_while_editing(self):
        with self.project(self.source()) as project:
            def interact(shell):
                for call in (project.close, project.rebuild, project.edit):
                    with self.assertRaises(RuntimeError):
                        call()
            self.edit(project, interact)
            project.rebuild()
            self.assertEqual(project.generation, 2)

    def test_saved_reload_after_editor_exit_is_inert(self):
        with self.project(self.source()) as project:
            saved = []
            self.edit(project, lambda shell: saved.append(shell.user_ns['reload']))
            with self.assertRaises(RuntimeError):
                saved[0]()
            self.assertEqual(project.generation, 1)

    def test_clipboard_checkpointing_routes_to_new_scene_after_reload(self):
        text = ['#point\nadd(8)']
        path = self.source()
        with self.project(path) as project:
            def interact(shell):
                self.cell(shell, 'checkpoint_paste()')
                self.assertEqual(project.scene.mobjects, [1, 8])
                self.source(2)
                self.cell(shell, 'reload()\ncheckpoint_paste()')
                self.assertEqual(project.scene.mobjects, [2, 8])
                text[0] = '#point\nadd(9)'
                self.cell(shell, 'checkpoint_paste()')
                self.assertEqual(project.scene.mobjects, [2, 9])
            self.edit(project, interact, clipboard=lambda: text[0])

    def test_preview_and_reload_refuse_cross_thread_without_touching_native(self):
        with self.project(self.source()) as project:
            def interact(shell):
                failures = []
                def worker():
                    for call in (shell.user_ns['reload'], shell.user_ns['preview']):
                        try:
                            call()
                        except RuntimeError as error:
                            failures.append(str(error))
                thread = threading.Thread(target=worker)
                thread.start(); thread.join()
                self.assertEqual(len(failures), 2)
                self.assertTrue(all('creating thread' in error for error in failures))
                self.assertEqual(project.generation, 1)
            self.edit(project, interact)

    def test_module_or_constructor_embed_cannot_start_an_unowned_terminal(self):
        for source in ('from manimlib import Scene\nScene().embed()\nclass Demo(Scene): pass\n',
                       'from manimlib import Scene\nclass Demo(Scene):\n'
                       '    def __init__(self):\n        super().__init__()\n        self.embed()\n'):
            path = self.write('scene.py', source)
            with patch.object(InteractiveShellEmbed, '__call__', side_effect=AssertionError('terminal')):
                with self.assertRaisesRegex(RuntimeError, 'currently under construction'):
                    with self.project(path):
                        pass
            self.assertIsNone(active_source())

    def test_repeat_editor_sessions_are_owned_independently(self):
        path = self.source()
        with self.project(path) as project:
            self.edit(project, lambda shell: self.cell(shell, 'reload()'))
            self.edit(project, lambda shell: self.cell(shell, 'reload()'))
            self.assertEqual(project.generation, 3)
            self.assertIsNone(project._editor)

    def test_missing_ipython_and_worker_stdio_refuse_without_losing_generation(self):
        with self.project(self.source()) as project:
            with patch('fmn_python.embedded_shell.importlib.import_module', side_effect=ImportError('no IPython')):
                with self.assertRaisesRegex(RuntimeError, 'IPython is not installed'):
                    project.edit()
            self.assertNotIn(_OWNER, vars(project.scene))
            token = _STUDIO_WORKER.set(True)
            try:
                with self.assertRaises(RuntimeError):
                    project.edit()
            finally:
                _STUDIO_WORKER.reset(token)
            self.assertIsNone(project._editor)
            self.assertEqual(project.generation, 1)


    def test_shortcut_failure_rolls_back_new_console_and_namespace(self):
        path = self.source()
        with self.project(path) as project:
            old = project.scene
            def interact(shell):
                embedded = project._editor
                console, manager = embedded._fmn_console, embedded.checkpoint_manager
                original = embedded.get_shortcuts
                candidates = []
                def failing_shortcuts():
                    if embedded.scene is not old:
                        candidates.append(embedded.scene)
                        raise OSError('authored shortcuts failed')
                    return original()
                embedded.get_shortcuts = failing_shortcuts
                self.source(2)
                with self.assertRaisesRegex(OSError, 'shortcuts failed'):
                    embedded.reload_scene()
                self.assertIs(embedded.scene, old)
                self.assertIs(embedded._fmn_console, console)
                self.assertIs(embedded.checkpoint_manager, manager)
                self.assertEqual(shell.user_ns['VALUE'], 1)
                self.assertIs(project.module, active_source(path).module)
                self.assertEqual(project.module.VALUE, 1)
                self.assertNotIn(_OWNER, vars(candidates[0]))
                self.assertNotIn(_SESSION, vars(candidates[0]))
                embedded.get_shortcuts = original
                self.cell(shell, 'reload()')
                self.assertEqual(project.scene.mobjects, [2])
            self.edit(project, interact)

    def test_cleanup_failure_after_commit_reports_new_active_generation(self):
        path = self.source()
        with self.project(path) as project:
            old = project.scene
            def interact(shell):
                embedded = project._editor
                manager = embedded.checkpoint_manager
                original = manager.clear_checkpoints
                def failing_cleanup():
                    raise OSError('checkpoint cleanup failed')
                manager.clear_checkpoints = failing_cleanup
                self.source(2)
                try:
                    with self.assertRaises(OSError) as caught:
                        embedded.reload_scene()
                    self.assertIn('new scene generation is active', ' '.join(caught.exception.__notes__))
                    self.assertIsNot(project.scene, old)
                    self.assertEqual(project.scene.mobjects, [2])
                    self.assertIs(shell.user_ns['scene'], project.scene)
                    self.assertIs(embedded.scene, project.scene)
                    self.assertIsNot(embedded.checkpoint_manager, manager)
                    self.assertNotIn(_OWNER, vars(old))
                finally:
                    manager.clear_checkpoints = original
                self.cell(shell, 'reload()')
                self.assertEqual(project.generation, 3)
            self.edit(project, interact)

    def test_authored_interrupt_during_candidate_does_not_discard_working_scene(self):
        path = self.source()
        with self.project(path) as project:
            old = project.scene
            def interact(shell):
                self.source(2, "        raise KeyboardInterrupt()\n")
                with self.assertRaises(KeyboardInterrupt):
                    project._editor.reload_scene()
                self.assertIs(project.scene, old)
                self.assertEqual(project.module.VALUE, 1)
                self.assertIs(shell.user_ns['scene'], old)
                self.source(3)
                self.cell(shell, 'reload()')
                self.assertEqual(project.scene.mobjects, [3])
            self.edit(project, interact)

    def test_reentrant_old_checkpoint_cleanup_cannot_start_a_second_rebuild(self):
        with self.project(self.source()) as project:
            def interact(shell):
                embedded = project._editor
                manager = embedded.checkpoint_manager
                original = manager.clear_checkpoints
                observations = []
                def cleanup():
                    with self.assertRaisesRegex(RuntimeError, 'already in progress'):
                        embedded.reload_scene()
                    observations.append(project.generation)
                    original()
                manager.clear_checkpoints = cleanup
                self.cell(shell, 'reload()')
                self.assertEqual(observations, [2])
                self.assertEqual(project.generation, 2)
            self.edit(project, interact)

    def test_default_embed_outside_project_keeps_existing_reload_refusal(self):
        embedded = self.native.InteractiveSceneEmbed(self.native.Scene())
        with self.assertRaisesRegex(RuntimeError, 'no reconstructible project recipe'):
            embedded.reload_scene()


    def test_switched_shell_module_is_rejected_without_mutating_foreign_globals(self):
        with self.project(self.source()) as project:
            def interact(shell):
                previous = shell.user_module
                foreign = ModuleType('foreign_host_module')
                foreign.VALUE = 99
                try:
                    shell.user_module = foreign
                    with self.assertRaisesRegex(RuntimeError, 'owning shell module'):
                        project._editor.reload_scene()
                    self.assertEqual(foreign.VALUE, 99)
                    self.assertNotIn('scene', vars(foreign))
                    self.assertEqual(project.generation, 1)
                finally:
                    shell.user_module = previous
            self.edit(project, interact)

    def test_installer_is_idempotent_and_preserves_native_class_identity(self):
        cls = self.native.InteractiveSceneEmbed
        methods = [cls.launch, cls.reload_scene, cls.ensure_frame_update_post_cell]
        install_scene_project_editor(self.native)
        self.assertIs(self.native.InteractiveSceneEmbed, cls)
        self.assertEqual(methods, [cls.launch, cls.reload_scene, cls.ensure_frame_update_post_cell])


class ProjectEditCliTests(unittest.TestCase):
    def setUp(self):
        import tempfile
        temporary = tempfile.TemporaryDirectory(prefix='fmn-edit-cli-')
        self.addCleanup(temporary.cleanup)
        self.path = Path(temporary.name) / 'scene.py'
        self.path.write_text('from manimlib import Scene\nclass Demo(Scene):\n'
                             '    def construct(self): self.add(1)\n')
        self.native = editor_fixture()
        self.native._native = self.native
        modules = patch.dict(sys.modules, {'manimlib': self.native})
        modules.start(); self.addCleanup(modules.stop)
        self.reports = []
        def emit(code, identity, kind, message, robot, **details):
            self.reports.append(dict(code=code, identity=identity, kind=kind,
                                     message=message, robot=robot, **details))
            return code
        self.native._portal_cli_emit = emit

    def run_cli(self, *args, terminal=True):
        with patch.object(sys, 'stdin', SimpleNamespace(isatty=lambda: terminal, fileno=lambda: 0,
                                                        encoding="utf-8", errors="replace")):
            return try_edit_cli(self.native, list(args))

    def test_help_does_not_import_authored_source_or_open_shell(self):
        with patch('fmn_python.project_editor.SceneProject', side_effect=AssertionError('must not load')):
            self.assertEqual(self.run_cli('edit', '--help', '--robot'), 0)
            self.assertIn('reload()', self.reports[-1]['help'])
            self.assertTrue(self.reports[-1]['robot'])
            with redirect_stdout(io.StringIO()) as output:
                self.assertEqual(self.run_cli('edit', '-h'), 0)
            self.assertIn('Rebuildable', output.getvalue())

    def test_other_commands_are_not_reinterpreted(self):
        for args in ([], ['--help'], ['studio', 'x.py'], ['scene.py', 'Demo'], ['--audit-parity']):
            self.assertIsNone(self.run_cli(*args))
        self.assertEqual(self.reports, [])

    def test_bad_usage_is_rejected_before_source_execution(self):
        for args in (['edit'], ['edit', str(self.path)], ['edit', str(self.path), 'Demo', 'extra'],
                     ['edit', '--bad', 'Demo']):
            self.assertEqual(self.run_cli(*args), 2)
        self.assertEqual(self.native.constructed, 0)

    def test_robot_and_non_terminal_refuse_before_authored_effects(self):
        self.path.write_text("raise AssertionError('must not execute')\n")
        self.assertEqual(self.run_cli('--robot', 'edit', str(self.path), 'Demo'), 4)
        self.assertTrue(self.reports[-1]['robot'])
        self.assertEqual(self.run_cli('edit', str(self.path), 'Demo', terminal=False), 4)
        with patch('sys.stdin', None):
            self.assertEqual(try_edit_cli(self.native, ['edit', str(self.path), 'Demo']), 4)
        self.assertEqual(self.native.constructed, 0)

    def test_missing_ipython_refuses_before_authored_effects(self):
        with patch('fmn_python.project_editor.importlib.import_module', side_effect=ImportError()):
            self.assertEqual(self.run_cli('edit', str(self.path), 'Demo'), 4)
        self.assertEqual(self.native.constructed, 0)

    @unittest.skipUnless(HAVE_IPYTHON, "IPython is not installed")
    def test_real_project_and_ipython_mainloop_run_through_cli(self):
        scenes = []
        def interact(shell, *args, **kwargs):
            self.addCleanup(shell.history_manager.end_session)
            scenes.append(shell.user_ns['scene'])
            self.path.write_text(self.path.read_text().replace('self.add(1)', 'self.add(9)'))
            result = shell.run_cell('reload()')
            self.assertTrue(result.success)
            scenes.append(shell.user_ns['scene'])
        with patch.object(InteractiveShellEmbed, 'interact', interact), \
                redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            self.assertEqual(self.run_cli('edit', str(self.path), 'Demo'), 0)
        self.assertEqual(self.reports[-1]['generations'], 2)
        self.assertEqual([scene.mobjects for scene in scenes], [[1], [9]])
        self.assertIsNone(active_source())
        self.assertTrue(all(_OWNER not in vars(scene) for scene in scenes))

    @unittest.skipUnless(HAVE_IPYTHON, "IPython is not installed")
    def test_initial_scene_failure_returns_nonzero_with_import_cleanup(self):
        self.path.write_text(self.path.read_text().replace('self.add(1)', "raise ValueError('failed scene')"))
        self.assertEqual(self.run_cli('edit', str(self.path), 'Demo'), 5)
        self.assertIn('failed scene', self.reports[-1]['message'])
        self.assertIsNone(active_source())

    @unittest.skipUnless(HAVE_IPYTHON, "IPython is not installed")
    def test_keyboard_interrupt_is_not_success(self):
        with patch.object(SceneProject, 'edit', side_effect=KeyboardInterrupt()):
            self.assertEqual(self.run_cli('edit', str(self.path), 'Demo'), 130)
        self.assertIsNone(active_source())

    @unittest.skipUnless(HAVE_IPYTHON, "IPython is not installed")
    def test_authored_system_exit_zero_is_not_reported_as_editor_success(self):
        self.path.write_text('raise SystemExit(0)\n')
        self.assertEqual(self.run_cli('edit', str(self.path), 'Demo'), 5)
        self.assertEqual(self.reports[-1]['kind'], 'edit-interrupted')
        self.assertIsNone(active_source())

    def test_production_dispatcher_routes_edit_before_other_commands(self):
        import fmn_python.__main__ as main
        modules = {}
        for name in ('console_rendering', 'scene_controls', 'studio', 'bundle_cli'):
            module = ModuleType('fmn_python.' + name)
            modules[module.__name__] = module
        def uncalled(*args):
            raise AssertionError('edit reached the wrong dispatcher')
        modules['fmn_python.console_rendering']._tokens = uncalled
        modules['fmn_python.console_rendering'].try_render_cli = uncalled
        modules['fmn_python.scene_controls'].try_scene_cli = uncalled
        modules['fmn_python.studio'].try_studio_cli = uncalled
        modules['fmn_python.bundle_cli'].try_bundle_cli = uncalled
        with patch.dict(sys.modules, modules), \
                patch.object(main, '_ensure_exclusive_manimlib_namespace'), \
                patch('sys.argv', ['fmn-python', 'edit', '--help', '--robot']):
            self.assertEqual(main.main(), 0)
        self.assertEqual(self.reports[-1]['kind'], 'help')


if __name__ == '__main__':
    unittest.main()
