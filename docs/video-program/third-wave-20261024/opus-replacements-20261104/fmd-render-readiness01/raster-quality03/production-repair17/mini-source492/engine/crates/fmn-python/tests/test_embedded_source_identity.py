"""Production shell construction with real IPython and scoped source imports.

Only native class/SceneConsole interfaces are minimal doubles. The actual
embedded-shell adapter constructs the real IPython shell; no terminal is run
and no native geometry or rendering is claimed.
"""
from __future__ import annotations

from contextvars import ContextVar
import importlib.util
import os
from pathlib import Path
import sys
import tempfile
from types import ModuleType
import unittest
from unittest.mock import patch

try:
    from IPython.terminal.embed import InteractiveShellEmbed
    HAVE_IPYTHON = True
except ImportError:
    HAVE_IPYTHON = False

from fmn_python.scene_loading import SceneSource
from fmn_python.source_autoreload import SourceNamespace


@unittest.skipUnless(HAVE_IPYTHON, "IPython is not installed")
class EmbeddedSourceIdentityTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="fmn-shell-identity-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        console = ModuleType("fmn_python.scene_console")
        console.SceneConsole = type("UnavailableNativeConsole", (), {})
        console._MAX_CHECKPOINTS = 32
        console._OWNER = "_test_console_owner"
        console._STUDIO_WORKER = ContextVar("identity_worker", default=False)
        def uncalled(*args, **kwargs):
            raise AssertionError("native checkpoint/console operation was not requested")
        console._note = console.checkpoint = uncalled
        modules = patch.dict(sys.modules, {console.__name__: console})
        modules.start()
        self.addCleanup(modules.stop)
        path = Path(os.environ.get("FMN_EMBEDDED_SHELL_SOURCE",
                        str(Path(__file__).resolve().parents[1] / "python/fmn_python/embedded_shell.py")))
        spec = importlib.util.spec_from_file_location("fmn_python._identity_shell_test", path)
        adapter = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(adapter)
        self.native = ModuleType("native_class_identity_fixture")
        class Scene:
            pass
        class InteractiveScene(Scene):
            pass
        class CheckpointManager:
            pass
        class InteractiveSceneEmbed:
            def get_shortcuts(self):
                return {}
        self.native.Scene = Scene
        self.native.InteractiveScene = InteractiveScene
        self.native.CheckpointManager = CheckpointManager
        self.native.InteractiveSceneEmbed = InteractiveSceneEmbed
        self.native._CapabilityError = RuntimeError
        adapter.install_embedded_shell(self.native)

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        return path

    def shell(self, namespace):
        embedded = self.native.InteractiveSceneEmbed(self.native.Scene())
        embedded._fmn_namespace = dict(namespace)
        shell = embedded.get_ipython_shell_for_embedded_scene()
        self.assertIsInstance(shell, InteractiveShellEmbed)
        self.addCleanup(shell.history_manager.end_session)
        return shell

    def test_constructing_real_shell_does_not_replace_active_source_module(self):
        path = self.write("scene.py", "VALUE=1\n")
        with SceneSource(path, self.native.Scene) as loaded:
            previous = loaded.module
            shell = self.shell(vars(previous))
            self.assertIs(sys.modules[loaded.name], previous)
            self.assertIs(sys.modules["scene"], previous)
            self.assertEqual(shell.user_module.__name__, "_fmn_scene_embed")
            self.assertEqual(shell.user_module.__file__, str(path))
            self.assertEqual(shell.user_ns["VALUE"], 1)
            self.assertIsNot(shell.user_module, previous)

    def test_package_relative_imports_retain_the_authored_package_context(self):
        self.write("reload_package/__init__.py", "")
        self.write("reload_package/helper.py", "VALUE=42\n")
        path = self.write("reload_package/scene.py", "VALUE=1\n")
        with SceneSource(path, self.native.Scene) as loaded:
            shell = self.shell(vars(loaded.module))
            result = shell.run_cell("from .helper import VALUE\nobserved=VALUE")
            self.assertTrue(result.success, result.error_in_exec)
            self.assertEqual(shell.user_ns["observed"], 42)
            self.assertEqual(shell.user_module.__package__, "reload_package")
            self.assertIs(sys.modules[loaded.name], loaded.module)

    def test_real_factory_and_source_reload_compose_without_module_collision(self):
        path = self.write("scene.py", "def value(): return 1\n")
        with SceneSource(path, self.native.Scene) as loaded:
            shell = self.shell(vars(loaded.module))
            bindings = SourceNamespace(loaded, shell.user_ns)
            path.write_text("def value(): return 7\n")
            bindings.refresh()
            result = shell.run_cell("observed=value()")
            self.assertTrue(result.success, result.error_in_exec)
            self.assertEqual(shell.user_ns["observed"], 7)
            self.assertIs(sys.modules[loaded.name], loaded.module)

    def test_copied_namespace_cannot_rebind_engine_module_identity(self):
        engine = ModuleType("manimlib")
        sys.modules["manimlib"] = engine
        self.shell({"__name__": "manimlib", "__spec__": object(), "__loader__": object()})
        self.assertIs(sys.modules["manimlib"], engine)

    def test_real_factory_restores_ipython_singletons_and_exception_hook(self):
        classes = tuple(InteractiveShellEmbed._walk_mro())
        absent = object()
        before = [(cls, vars(cls).get("_instance", absent)) for cls in classes]
        exception_hook = sys.excepthook
        self.shell({"VALUE": 1})
        self.assertIs(sys.excepthook, exception_hook)
        for cls, instance in before:
            self.assertIs(vars(cls).get("_instance", absent), instance)


if __name__ == "__main__":
    unittest.main()
