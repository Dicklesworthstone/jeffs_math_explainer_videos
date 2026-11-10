"""Real IPython events/cells and project imports with a minimal Scene boundary.

No terminal is blocked and no native rendering is claimed. The fake Embedded
lifecycle supplies ownership flags; the production autoreload adapter, Python
loader, namespaces and IPython execution/event dispatch run unchanged.
"""
from __future__ import annotations

import contextlib
from contextvars import ContextVar
import io
from pathlib import Path
import sys
import tempfile
import threading
from types import ModuleType, SimpleNamespace
import unittest
from unittest.mock import patch

try:
    from IPython.core.interactiveshell import InteractiveShell
    HAVE_IPYTHON = True
except ImportError:
    HAVE_IPYTHON = False

from fmn_python.scene_loading import SceneSource
from fmn_python.source_autoreload import SourceNamespace, install_source_autoreload, _STATE
from fmn_python.source_reload import active_source


@unittest.skipUnless(HAVE_IPYTHON, "IPython is not installed")
class SourceAutoreloadTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="fmn-autoreload-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.worker = ContextVar("test_studio_worker", default=False)
        console = ModuleType("fmn_python.scene_console")
        console._STUDIO_WORKER = self.worker
        support = ModuleType("fmn_autoreload_support")
        support.events = []
        self.events = support.events
        modules = patch.dict(sys.modules, {console.__name__: console, support.__name__: support})
        modules.start()
        self.addCleanup(modules.stop)
        self.native = ModuleType("native_ownership_fixture")
        class Scene:
            def play(self):
                pass
        self.native.Scene = Scene
        self.native._CapabilityError = type("CapabilityError", (RuntimeError,), {})
        self.config = SimpleNamespace(embed=SimpleNamespace(autoreload=False))
        self.native._pinned_manim_config = lambda: self.config
        class Embedded:
            def __init__(self, scene, shell, script):
                self.scene, self.shell, self.script = scene, shell, script
                self._fmn_thread = threading.get_ident()
                self._fmn_launching = False
                self._fmn_hooks = []
                self.preview_events = []
            def get_shortcuts(self):
                return {"play": self.scene.play, "reload": self.reload_scene}
            def reload_scene(self):
                raise RuntimeError("full scene reconstruction unavailable")
            def ensure_frame_update_post_cell(self):
                def preview(*_args):
                    if self._fmn_launching:
                        self.preview_events.append("preview")
                self.shell.events.register("post_run_cell", preview)
                self._fmn_hooks.append((self.shell.events, "post_run_cell", preview))
            def launch(self):
                if self._fmn_launching:
                    raise RuntimeError("nested shell")
                self._fmn_launching = True
                try:
                    self.shell.user_ns.update(self.get_shortcuts())
                    self.ensure_frame_update_post_cell()
                    self.script(self)
                    return self.shell
                finally:
                    self._fmn_launching = False
                    for events, name, function in reversed(self._fmn_hooks):
                        events.unregister(name, function)
                    self._fmn_hooks.clear()
        self.native.InteractiveSceneEmbed = Embedded
        install_source_autoreload(self.native)

    def write(self, name, data):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(data)
        return path

    def shell(self, filename=None, namespace=None):
        module = ModuleType("_fmn_reload_cell_test")
        if namespace:
            module.__dict__.update({key: value for key, value in namespace.items()
                                    if key not in {"__name__", "__spec__", "__loader__"}})
        if filename is not None:
            module.__file__ = str(filename)
        # IPython's real event manager and run_cell execute every authored cell.
        shell = InteractiveShell(user_module=module, user_ns=module.__dict__)
        return shell

    def launch(self, path, script, *, source=None, scene=None):
        shell = self.shell(path, None if source is None else vars(source.module))
        scene = self.native.Scene() if scene is None else scene
        shell.user_ns.update(self=scene, scene=scene)
        embedded = self.native.InteractiveSceneEmbed(scene, shell, script)
        result = embedded.launch()
        self.assertIs(result, shell)
        return embedded

    def run_cell(self, shell, text):
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            result = shell.run_cell(text)
        self.assertTrue(result.success, result.error_in_exec or result.error_before_exec)
        return result

    def test_auto_reload_updates_helper_before_real_ipython_cell(self):
        helper = self.write("autoreload_helper.py", "def value():\n    return 1\n")
        path = self.write("scene.py", "from autoreload_helper import value\n")
        with SceneSource(path, self.native.Scene) as source:
            def script(embedded):
                embedded.auto_reload()
                helper.write_text("def value():\n    return 8\n")
                self.run_cell(embedded.shell, "observed = value()")
                self.assertEqual(embedded.shell.user_ns["observed"], 8)
                self.assertIs(embedded.shell.user_ns["scene"], embedded.scene)
                self.assertIs(embedded.shell.user_ns["self"], embedded.scene)
            embedded = self.launch(path, script, source=source)
            self.assertTrue(source._active)
            self.assertEqual(embedded.preview_events, ["preview"])

    def test_unchanged_source_cannot_hide_an_outside_module_replacement(self):
        path = self.write("scene.py", "VALUE=1\n")
        with SceneSource(path, self.native.Scene) as source:
            replacement = ModuleType(source.name)
            sys.modules[source.name] = replacement
            with self.assertRaisesRegex(ImportError, "outside its source owner"):
                source.reload(if_changed=True)
            self.assertIs(sys.modules[source.name], replacement)

    def test_configured_autoreload_uses_existing_owner_without_nested_source(self):
        self.config.embed.autoreload = True
        path = self.write("scene.py", "VALUE=1\n")
        with SceneSource(path, self.native.Scene) as source:
            def script(embedded):
                path.write_text("VALUE=2\n")
                self.run_cell(embedded.shell, "observed=VALUE")
                self.assertEqual(embedded.shell.user_ns["observed"], 2)
                self.assertIs(vars(embedded)[_STATE].source, source)
            self.launch(path, script, source=source)

    def test_disabled_configuration_never_opens_source(self):
        path = self.root / "absent.py"
        def script(embedded):
            self.run_cell(embedded.shell, "value=3")
            self.assertNotIn(_STATE, vars(embedded))
        self.launch(path, script)
        self.assertIsNone(active_source())

    def test_explicit_reload_source_shortcut_refreshes_without_autoreload(self):
        path = self.write("scene.py", "VALUE=1\n")
        with SceneSource(path, self.native.Scene) as source:
            def script(embedded):
                path.write_text("VALUE=6\n")
                self.run_cell(embedded.shell, "reload_source(); observed=VALUE")
                self.assertEqual(embedded.shell.user_ns["observed"], 6)
                self.assertIsNone(vars(embedded)[_STATE].callback)
                with self.assertRaisesRegex(RuntimeError, "reconstruction"):
                    embedded.shell.user_ns["reload"]()
            self.launch(path, script, source=source)

    def test_owned_source_executes_once_then_closes_with_shell(self):
        path = self.write("scene.py", "from fmn_autoreload_support import events\nevents.append('load')\nVALUE=1\n")
        def script(embedded):
            self.run_cell(embedded.shell, "reload_source(); observed=VALUE")
            self.assertEqual(self.events, ["load"])
            self.assertEqual(embedded.shell.user_ns["observed"], 1)
            self.assertIsNotNone(active_source(path))
        embedded = self.launch(path, script)
        self.assertIsNone(active_source())
        self.assertNotIn(_STATE, vars(embedded))
        self.assertEqual(embedded._fmn_hooks, [])
        with SceneSource(path, self.native.Scene):
            pass

    def test_local_overrides_shortcuts_and_native_scene_identity_survive(self):
        path = self.write("scene.py", "VALUE=1\nREMOVE=object()\ndef helper(): return 1\n")
        with SceneSource(path, self.native.Scene) as source:
            def script(embedded):
                embedded.auto_reload()
                original_scene = embedded.scene
                original_play = embedded.shell.user_ns["play"]
                self.run_cell(embedded.shell, "VALUE='local'; local_only=object()")
                local_only = embedded.shell.user_ns["local_only"]
                path.write_text("VALUE=2\nNEW=3\nplay=99\nscene=99\nself=99\ndef helper(): return 7\n")
                self.run_cell(embedded.shell, "answer=helper()")
                ns = embedded.shell.user_ns
                self.assertEqual(ns["VALUE"], "local")
                self.assertEqual(ns["NEW"], 3)
                self.assertEqual(ns["answer"], 7)
                self.assertNotIn("REMOVE", ns)
                self.assertIs(ns["local_only"], local_only)
                self.assertIs(ns["scene"], original_scene)
                self.assertIs(ns["self"], original_scene)
                self.assertIs(ns["play"], original_play)
            self.launch(path, script, source=source)

    def test_deleted_source_name_with_local_override_is_not_deleted(self):
        path = self.write("scene.py", "VALUE=object()\n")
        with SceneSource(path, self.native.Scene) as source:
            def script(embedded):
                embedded.auto_reload()
                self.run_cell(embedded.shell, "VALUE='interactive'")
                path.write_text("OTHER=2\n")
                self.run_cell(embedded.shell, "observed=VALUE")
                self.assertEqual(embedded.shell.user_ns["observed"], "interactive")
            self.launch(path, script, source=source)

    def test_failed_edit_preserves_previous_callable_and_next_edit_recovers(self):
        helper = self.write("autoreload_helper.py", "def value(): return 1\n")
        path = self.write("scene.py", "from autoreload_helper import value\n")
        with SceneSource(path, self.native.Scene) as source:
            def script(embedded):
                embedded.auto_reload()
                previous = source.module
                helper.write_text("def value(\n")
                # IPython reports event errors and still permits the cell to use
                # the last working definitions. The module graph remains old.
                self.run_cell(embedded.shell, "observed=value()")
                self.assertEqual(embedded.shell.user_ns["observed"], 1)
                self.assertIs(source.module, previous)
                helper.write_text("def value(): return 9\n")
                self.run_cell(embedded.shell, "observed=value()")
                self.assertEqual(embedded.shell.user_ns["observed"], 9)
            self.launch(path, script, source=source)

    def test_source_side_effects_do_not_repeat_for_unchanged_cells(self):
        path = self.write("scene.py", "from fmn_autoreload_support import events\nevents.append(1)\n")
        with SceneSource(path, self.native.Scene) as source:
            def script(embedded):
                embedded.auto_reload()
                for _ in range(3):
                    self.run_cell(embedded.shell, "value=1")
                self.assertEqual(self.events, [1])
                embedded.reload_source()
                self.assertEqual(self.events, [1, 1])
            self.launch(path, script, source=source)

    def test_registration_is_idempotent_and_stale_callback_is_inert(self):
        path = self.write("scene.py", "VALUE=1\n")
        callbacks = []
        with SceneSource(path, self.native.Scene) as source:
            def script(embedded):
                embedded.auto_reload()
                state = vars(embedded)[_STATE]
                callbacks.append(state.callback)
                embedded.auto_reload()
                self.assertIs(state.callback, callbacks[0])
                self.assertEqual(sum(name == "pre_run_cell" for _, name, _ in embedded._fmn_hooks), 1)
                self.run_cell(embedded.shell, "observed=VALUE")
            embedded = self.launch(path, script, source=source)
            module = source.module
            path.write_text("VALUE=4\n")
            callbacks[0]()
            self.assertIs(source.module, module)
            self.assertEqual(embedded.shell.user_ns["observed"], 1)
            self.assertEqual(embedded._fmn_hooks, [])

    def test_nested_refused_launch_keeps_outer_controller(self):
        path = self.write("scene.py", "VALUE=1\n")
        with SceneSource(path, self.native.Scene) as source:
            def script(embedded):
                embedded.auto_reload()
                state = vars(embedded)[_STATE]
                with self.assertRaisesRegex(RuntimeError, "nested"):
                    embedded.launch()
                self.assertIs(vars(embedded)[_STATE], state)
                self.assertFalse(state.closed)
                path.write_text("VALUE=3\n")
                self.run_cell(embedded.shell, "observed=VALUE")
                self.assertEqual(embedded.shell.user_ns["observed"], 3)
            self.launch(path, script, source=source)

    def test_terminal_exception_closes_owned_import_scope(self):
        path = self.write("scene.py", "VALUE=1\n")
        failure = LookupError("terminal failure")
        def script(embedded):
            embedded.reload_source()
            raise failure
        with self.assertRaises(LookupError) as raised:
            self.launch(path, script)
        self.assertIs(raised.exception, failure)
        self.assertIsNone(active_source())

    def test_studio_worker_refuses_before_module_execution(self):
        path = self.write("scene.py", "from fmn_autoreload_support import events\nevents.append(1)\n")
        def script(embedded):
            token = self.worker.set(True)
            try:
                with self.assertRaisesRegex(RuntimeError, "supervisor"):
                    embedded.reload_source()
            finally:
                self.worker.reset(token)
        self.launch(path, script)
        self.assertEqual(self.events, [])

    def test_certified_generation_and_active_animation_refuse_source_changes(self):
        path = self.write("scene.py", "VALUE=1\n")
        def script(embedded):
            embedded.scene._fmn_owned_render_session = SimpleNamespace(reproducible=True)
            with self.assertRaisesRegex(RuntimeError, "certified"):
                embedded.reload_source()
            embedded.scene._fmn_owned_render_session = None
            embedded.scene._fmn_scene_execution = object()
            with self.assertRaisesRegex(RuntimeError, "executing"):
                embedded.reload_source()
        self.launch(path, script)
        self.assertIsNone(active_source())

    def test_fileless_notebook_session_refuses_only_requested_reload(self):
        shell = self.shell()
        scene = self.native.Scene()
        def script(embedded):
            with self.assertRaisesRegex(RuntimeError, "file-backed"):
                embedded.auto_reload()
            self.run_cell(embedded.shell, "working=3")
        embedded = self.native.InteractiveSceneEmbed(scene, shell, script)
        embedded.launch()
        self.assertEqual(shell.user_ns["working"], 3)

    def test_other_thread_cannot_use_live_reload_callback(self):
        path = self.write("scene.py", "VALUE=1\n")
        def script(embedded):
            errors = []
            def worker():
                try:
                    embedded.reload_source()
                except RuntimeError as error:
                    errors.append(str(error))
            thread = threading.Thread(target=worker)
            thread.start()
            thread.join()
            self.assertEqual(len(errors), 1)
            self.assertIn("thread", errors[0])
        self.launch(path, script)

    def test_installer_does_not_replace_native_class_or_stack_wrappers(self):
        cls = self.native.InteractiveSceneEmbed
        methods = cls.launch, cls.auto_reload, cls.get_shortcuts
        install_source_autoreload(self.native)
        self.assertIs(self.native.InteractiveSceneEmbed, cls)
        self.assertEqual(methods, (cls.launch, cls.auto_reload, cls.get_shortcuts))

    def test_configured_autoreload_survives_real_embedded_mainloop_namespace_activation(self):
        from IPython.terminal.embed import InteractiveShellEmbed
        path = self.write("scene.py", "VALUE=1\n")
        self.config.embed.autoreload = True
        with SceneSource(path, self.native.Scene) as source:
            module = ModuleType("_fmn_reload_activation_test")
            module.__dict__.update({key: value for key, value in vars(source.module).items()
                                    if key not in {"__name__", "__spec__", "__loader__"}})
            shell = InteractiveShellEmbed(user_module=module, user_ns=module.__dict__, display_banner=False)
            initial_namespace = shell.user_ns
            observed = []
            def script(embedded):
                def interact():
                    self.assertIsNot(shell.user_ns, initial_namespace)
                    path.write_text("VALUE=12\n")
                    self.run_cell(shell, "observed=VALUE")
                    observed.append(shell.user_ns["observed"])
                shell.interact = interact
                shell(local_ns=shell.user_ns, module=shell.user_module)
            self.native.InteractiveSceneEmbed(self.native.Scene(), shell, script).launch()
            self.assertEqual(observed, [12])
            self.assertIs(shell.user_ns, initial_namespace)

    def test_reloaded_source_helpers_are_visible_to_functions_defined_in_embedded_cells(self):
        from IPython.terminal.embed import InteractiveShellEmbed
        path = self.write("scene.py", "def value(): return 1\n")
        with SceneSource(path, self.native.Scene) as source:
            module = ModuleType("_fmn_reload_globals_test")
            module.__dict__.update({key: value for key, value in vars(source.module).items()
                                    if key not in {"__name__", "__spec__", "__loader__"}})
            shell = InteractiveShellEmbed(user_module=module, user_ns=module.__dict__, display_banner=False)
            observed = []
            def script(embedded):
                def interact():
                    embedded.auto_reload()
                    self.run_cell(shell, "def from_cell(): return value()")
                    path.write_text("def value(): return 15\n")
                    self.run_cell(shell, "observed=from_cell()")
                    observed.append(shell.user_ns["observed"])
                shell.interact = interact
                shell(local_ns=shell.user_ns, module=shell.user_module)
            self.native.InteractiveSceneEmbed(self.native.Scene(), shell, script).launch()
            self.assertEqual(observed, [15])

    def test_namespace_adapter_preserves_ipython_internal_names(self):
        path = self.write("scene.py", "VALUE=1\n")
        with SceneSource(path, self.native.Scene) as source:
            marker = object()
            ns = {"__name__": "shell", "In": marker, "_": marker, "VALUE": 1}
            bindings = SourceNamespace(source, ns)
            path.write_text("In='source'\n_='source'\nVALUE=2\n")
            bindings.refresh()
            self.assertEqual(ns["__name__"], "shell")
            self.assertIs(ns["In"], marker)
            self.assertIs(ns["_"], marker)
            self.assertEqual(ns["VALUE"], 2)


if __name__ == "__main__":
    unittest.main()
