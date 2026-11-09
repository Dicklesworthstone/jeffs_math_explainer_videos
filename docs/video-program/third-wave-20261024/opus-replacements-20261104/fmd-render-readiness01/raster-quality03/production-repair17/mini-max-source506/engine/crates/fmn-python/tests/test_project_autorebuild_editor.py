"""Real IPython/source/editor transactions with explicit native lifecycle doubles."""
from __future__ import annotations

from contextlib import redirect_stdout, redirect_stderr
import io
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import test_project_editor as fixtures
from fmn_python.project_autorebuild import _STATE
from fmn_python.source_autoreload import _STATE as _DEFINITIONS


@unittest.skipUnless(fixtures.HAVE_IPYTHON, 'IPython is not installed')
class AutomaticRebuildEditorTests(unittest.TestCase):
    setUp = fixtures.ProjectEditorTests.setUp
    write = fixtures.ProjectEditorTests.write
    source = fixtures.ProjectEditorTests.source
    project = fixtures.ProjectEditorTests.project
    edit = fixtures.ProjectEditorTests.edit
    cell = fixtures.ProjectEditorTests.cell

    def test_enable_preserves_unchanged_console_checkpoints_and_interactive_edits(self):
        with self.project(self.source()) as project:
            def interact(shell):
                console = project._editor._fmn_console
                manager = project._editor.checkpoint_manager
                manager.handle_checkpoint_key(project.scene, '#before')
                self.cell(shell, 'auto_rebuild()\nadd(4)')
                self.cell(shell, 'observed=tuple(scene.mobjects)')
                self.assertEqual(shell.user_ns['observed'], (1, 4))
                self.assertEqual(project.generation, 1)
                self.assertIs(project._editor._fmn_console, console)
                self.assertFalse(console._closed)
                self.assertIn('#before', manager.checkpoint_states)
            self.edit(project, interact)

    def test_source_edit_rebuilds_before_cell_and_keeps_old_aliases(self):
        with self.project(self.source()) as project:
            old = project.scene
            def interact(shell):
                self.cell(shell, 'auto_rebuild()\nold_scene=scene\ncustom=42')
                self.source(7)
                self.cell(shell, 'add(8)\nobserved=VALUE')
                self.assertEqual(project.scene.mobjects, [7, 8])
                self.assertEqual(shell.user_ns['observed'], 7)
                self.assertIs(shell.user_ns['old_scene'], old)
                self.assertEqual(shell.user_ns['custom'], 42)
                self.assertEqual(project.generation, 2)
                self.cell(shell, 'pass')
                self.assertEqual(project.generation, 2)
            self.edit(project, interact)

    def test_failed_candidate_is_retained_reported_once_and_recovers(self):
        with self.project(self.source()) as project:
            old = project.scene
            def interact(shell):
                self.cell(shell, 'auto_rebuild()')
                self.source(2, "        raise ValueError('broken generation')\n")
                self.cell(shell, 'still_working=scene')
                self.assertIs(project.scene, old)
                self.assertIs(shell.user_ns['still_working'], old)
                self.assertEqual(project.generation, 1)
                setups = self.native.events.count("setup")
                self.cell(shell, 'pass')
                # Post-cell preview is expected; authored construction is not.
                self.assertEqual(self.native.events.count("setup"), setups)
                self.source(6)
                self.cell(shell, 'value=VALUE')
                self.assertEqual(project.scene.mobjects, [6])
                self.assertEqual(shell.user_ns['value'], 6)
                self.assertEqual(project.generation, 2)
            self.edit(project, interact)
            self.assertIn('broken generation', self.captured.getvalue())

    def test_native_capture_failure_retains_previous_scene_module_and_preview(self):
        with self.project(self.source()) as project:
            old, module, image = project.scene, project.module, project.preview
            def interact(shell):
                self.cell(shell, 'auto_rebuild()')
                self.source(4)
                self.native.preview_failure = OSError('capture failed')
                self.cell(shell, 'pass')
                self.native.preview_failure = None
                self.assertIs(project.scene, old)
                self.assertIs(project.module, module)
                self.assertIs(project.preview, image)
                self.source(5)
                self.cell(shell, 'pass')
                self.assertEqual(project.scene.mobjects, [5])
            self.edit(project, interact)

    def test_added_helper_and_explicit_asset_changes_force_reconstruction(self):
        asset = self.write('data.txt', 'one')
        with self.project(self.source()) as project:
            def interact(shell):
                shell.user_ns['asset_path'] = str(asset)
                self.cell(shell, 'auto_rebuild(paths=[asset_path])')
                # The source is unchanged, but this edit occurs before the
                # first watch callback; activation bytes must be remembered.
                asset.write_text('two')
                self.cell(shell, 'pass')
                self.assertEqual(project.generation, 2)
                self.write('new_helper.py', 'VALUE = 9\n')
                self.cell(shell, 'pass')
                self.assertEqual(project.generation, 3)
            self.edit(project, interact)

    def test_modes_are_exclusive_and_switching_back_only_updates_definitions(self):
        with self.project(self.source()) as project:
            def interact(shell):
                self.cell(shell, 'auto_reload()')
                self.cell(shell, 'auto_rebuild()')
                self.assertIsNone(vars(project._editor)[_DEFINITIONS].callback)
                self.source(7)
                self.cell(shell, 'pass')
                self.assertEqual(project.scene.mobjects, [7])
                self.assertEqual(project.generation, 2)
                self.cell(shell, 'auto_reload()')
                self.assertNotIn(_STATE, vars(project._editor))
                self.source(9)
                self.cell(shell, 'value=VALUE')
                self.assertEqual(shell.user_ns['value'], 9)
                self.assertEqual(project.scene.mobjects, [7])
                self.assertEqual(project.generation, 2)
            self.edit(project, interact)

    def test_stopped_and_closed_callbacks_are_inert(self):
        retained = []
        with self.project(self.source()) as project:
            def interact(shell):
                self.cell(shell, 'auto_rebuild()')
                state = vars(project._editor)[_STATE]
                callback = state.callback
                self.cell(shell, 'stop_auto_rebuild()')
                self.source(2)
                callback()
                self.assertTrue(state.closed)
                self.assertEqual(project.generation, 1)
                self.cell(shell, 'auto_rebuild()')
                retained.append(vars(project._editor)[_STATE])
            self.edit(project, interact)
            self.assertTrue(retained[0].closed)
            retained[0].callback()
            self.assertIsNone(retained[0].project)
            self.assertEqual(project.generation, 1)

    def test_conditional_manual_reload_is_a_true_noop(self):
        with self.project(self.source()) as project:
            def interact(shell):
                console = project._editor._fmn_console
                self.cell(shell, 'reload(if_changed=True)\nadd(4)')
                self.assertEqual(project.generation, 1)
                self.assertIs(project._editor._fmn_console, console)
                self.assertFalse(console._closed)
                self.assertEqual(project.scene.mobjects, [1, 4])
                result = shell.run_cell('reload(if_changed=1)')
                self.assertIsInstance(result.error_in_exec, TypeError)
            self.edit(project, interact)

    def test_active_output_does_not_consume_pending_rebuild(self):
        with self.project(self.source()) as project:
            def interact(shell):
                self.cell(shell, 'auto_rebuild()')
                state = vars(project._editor)[_STATE]
                self.source(8)
                vars(project.scene)['_fmn_owned_render_session'] = object()
                try:
                    with self.assertRaises(RuntimeError):
                        state.poll()
                finally:
                    vars(project.scene).pop('_fmn_owned_render_session')
                self.assertTrue(state.poll())
                self.assertEqual(project.scene.mobjects, [8])
            self.edit(project, interact)


    def test_public_edit_enables_rebuild_before_first_prompt_cell(self):
        with self.project(self.source()) as project:
            def interact(shell):
                self.assertIn(_STATE, vars(project._editor))
                self.source(8)
                self.cell(shell, 'value=VALUE')
                self.assertEqual(shell.user_ns['value'], 8)
                self.assertEqual(project.scene.mobjects, [8])
                self.assertEqual(project.generation, 2)
            self.edit(project, interact, auto_rebuild=True)

    def test_public_edit_watches_frozen_asset_generator(self):
        asset = self.write('data.txt', 'one')
        with self.project(self.source()) as project:
            def interact(shell):
                asset.write_text('two')
                self.cell(shell, 'pass')
                self.assertEqual(project.generation, 2)
            self.edit(project, interact, auto_rebuild=True, paths=(path for path in [asset]))

    def test_invalid_public_options_refuse_before_opening_shell(self):
        with self.project(self.source()) as project:
            for options, error_type in (({'auto_rebuild': 1}, TypeError),
                                        ({'auto_rebuild': True, 'debounce': -1}, ValueError),
                                        ({'auto_rebuild': True, 'paths': 'data.txt'}, TypeError),
                                        ({'debounce': 0.5}, ValueError),
                                        ({'paths': ['data.txt']}, ValueError)):
                with self.subTest(options=options), patch.object(
                        self.native.InteractiveSceneEmbed, 'launch',
                        side_effect=AssertionError('must not open shell')):
                    with self.assertRaises(error_type):
                        project.edit(**options)
                self.assertIsNone(project._editor)
                self.assertEqual(project.generation, 1)

    def test_configured_definition_autoreload_does_not_execute_a_second_generation(self):
        self.native._pinned_manim_config = lambda: SimpleNamespace(embed=SimpleNamespace(autoreload=True))
        with self.project(self.source()) as project:
            def interact(shell):
                self.assertIsNone(vars(project._editor)[_DEFINITIONS].callback)
                self.source(3)
                self.cell(shell, 'pass')
                self.assertEqual(project.generation, 2)
                self.assertEqual(self.native.constructed, 2)
            self.edit(project, interact, auto_rebuild=True)

    def test_invalid_reconfiguration_keeps_active_watch(self):
        with self.project(self.source()) as project:
            def interact(shell):
                state = vars(project._editor)[_STATE]
                with self.assertRaises(ValueError):
                    project._editor.auto_rebuild(debounce=-1)
                self.assertIs(vars(project._editor)[_STATE], state)
                self.assertFalse(state.closed)
                self.source(4)
                self.cell(shell, 'pass')
                self.assertEqual(project.scene.mobjects, [4])
            self.edit(project, interact, auto_rebuild=True)

    def test_nested_launch_refusal_does_not_stop_outer_watch(self):
        with self.project(self.source()) as project:
            def interact(shell):
                embedded = project._editor
                state = vars(embedded)[_STATE]
                with self.assertRaises(RuntimeError):
                    embedded.launch()
                self.assertIs(vars(embedded)[_STATE], state)
                self.assertFalse(state.closed)
                self.source(5)
                self.cell(shell, 'pass')
                self.assertEqual(project.scene.mobjects, [5])
            self.edit(project, interact, auto_rebuild=True)


class AutomaticRebuildCliTests(unittest.TestCase):
    setUp = fixtures.ProjectEditCliTests.setUp
    run_cli = fixtures.ProjectEditCliTests.run_cli

    def test_watch_help_is_nonexecuting_and_describes_safe_cell_boundary(self):
        self.path.write_text("raise AssertionError('must not load')\n")
        self.assertEqual(self.run_cli('edit', '--watch', '--help', '--robot'), 0)
        self.assertIn('before the next cell', self.reports[-1]['help'])
        self.assertEqual(self.native.constructed, 0)

    def test_watch_refuses_invalid_options_and_robot_without_authored_effects(self):
        self.path.write_text("raise AssertionError('must not load')\n")
        for args in (('edit', '--watch', '--watch', str(self.path), 'Demo'),
                     ('edit', '--watch=invalid', str(self.path), 'Demo'),
                     ('edit', '--watch', str(self.path))):
            self.assertEqual(self.run_cli(*args), 2)
        self.assertEqual(self.run_cli('edit', '--watch', str(self.path), 'Demo', '--robot'), 4)
        self.assertEqual(self.run_cli('edit', '--watch', str(self.path), 'Demo', terminal=False), 4)
        self.assertEqual(self.native.constructed, 0)

    @unittest.skipUnless(fixtures.HAVE_IPYTHON, 'IPython is not installed')
    def test_real_cli_watch_reconstructs_without_manual_reload(self):
        scenes = []
        def interact(shell, *args, **kwargs):
            self.addCleanup(shell.history_manager.end_session)
            scenes.append(shell.user_ns['scene'])
            self.path.write_text(self.path.read_text().replace('self.add(1)', 'self.add(9)'))
            result = shell.run_cell('observed=scene')
            self.assertTrue(result.success)
            scenes.append(shell.user_ns['observed'])
        with patch.object(fixtures.InteractiveShellEmbed, 'interact', interact), \
                redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            self.assertEqual(self.run_cli('edit', str(self.path), 'Demo', '--watch'), 0)
        self.assertEqual([scene.mobjects for scene in scenes], [[1], [9]])
        self.assertEqual(self.reports[-1]['generations'], 2)
        self.assertIsNone(fixtures.active_source())


if __name__ == '__main__':
    unittest.main()
