"""Actual IPython embedding/mainloop/event/singleton behavior, without a TTY.

Only terminal input is replaced with authored test cells. The native Scene
boundary is explicitly the protocol fixture, not claimed native rendering.
"""
from contextlib import redirect_stdout, redirect_stderr
import io
import sys
from types import ModuleType
import unittest
from unittest.mock import patch

try:
    from IPython.core.interactiveshell import InteractiveShell
    from IPython.terminal.embed import InteractiveShellEmbed
    HAVE_IPYTHON = True
except ImportError:
    HAVE_IPYTHON = False

from fmn_python import embedded_shell as adapter
from test_embedded_shell_protocol import native_classes


@unittest.skipUnless(HAVE_IPYTHON, "IPython is not installed")
class EmbeddedIPythonTests(unittest.TestCase):
    def setUp(self):
        self.native = native_classes()
        self.scene = self.native.InteractiveScene()
        self.box = self.scene.mobjects[0]
        self.embed = self.native.InteractiveSceneEmbed(self.scene)
        self.embed._fmn_namespace = {"box": self.box, "sentinel": object()}
        # This is an explicit external existing IPython instance, not one owned
        # or replaceable by the scene embedder.
        self.host = InteractiveShell()
        owners = tuple(InteractiveShellEmbed._walk_mro())
        previous = [(cls, vars(cls).get("_instance", adapter._MISSING)) for cls in owners]
        hook = sys.excepthook
        for cls in owners:
            cls._instance = self.host
        def restore():
            for cls, value in previous:
                if value is adapter._MISSING:
                    if "_instance" in vars(cls):
                        delattr(cls, "_instance")
                else:
                    cls._instance = value
            sys.excepthook = hook
        self.addCleanup(restore)
        self.output = io.StringIO()
        self.outer_hook = sys.excepthook
    def make_shell(self):
        with redirect_stdout(self.output), redirect_stderr(self.output):
            self.embed.shell = self.embed.get_ipython_shell_for_embedded_scene()
        self.embed.shell.confirm_exit = False
        self.embed.shell.exit_msg = ""
        return self.embed.shell
    def launch(self, cells):
        shell = self.embed.shell or self.make_shell()
        with redirect_stdout(self.output), redirect_stderr(self.output), patch.object(shell, "interact", lambda: cells(shell)):
            return self.embed.launch()
    def assert_host_unchanged(self):
        self.assertIs(InteractiveShell.instance(), self.host)
        self.assertIs(sys.excepthook, self.outer_hook)
        self.assertIsNone(adapter._ACTIVE.get())
        self.assertNotIn("_fmn_scene_console", vars(self.scene))
    def test_shell_creation_does_not_replace_notebook_instance_or_exception_hook(self):
        shell = self.make_shell()
        self.assertIsInstance(shell, InteractiveShellEmbed)
        self.assertIs(shell.user_ns["box"], self.box)
        self.assertIs(shell.user_ns["self"], self.scene)
        self.assertEqual(shell.user_ns["checkpoint_paste"], self.embed.checkpoint_paste)
        self.assert_host_unchanged()
    def test_real_mainloop_keyed_paste_rewinds_and_keeps_authored_namespace(self):
        source = iter(("# native\nbox.x += 2\nanswer = 42", "# native\nbox.x += 5"))
        self.embed.clipboard = lambda: next(source)
        def cells(shell):
            self.assertIs(InteractiveShell.instance(), shell)
            # __call__/mainloop creates another namespace; the console must
            # bind it rather than resetting globals from a stale snapshot.
            shell.run_cell("checkpoint_paste()")
            self.assertEqual(self.box.x, 2)
            shell.run_cell("checkpoint_paste()")
            self.assertEqual(self.box.x, 5)
            self.assertEqual(shell.user_ns["answer"], 42)
        shell = self.launch(cells)
        self.assertEqual(shell.user_ns["answer"], 42)
        self.assertTrue(self.scene.camera.captures)
        self.assert_host_unchanged()
    def test_real_cells_use_current_ipython_namespace_not_stale_saved_bindings(self):
        self.embed.clipboard = lambda: "# a\nassert 'sentinel' not in locals()\nbox.x += current"
        def cells(shell):
            shell.run_cell("del sentinel\ncurrent = 7")
            result = shell.run_cell("checkpoint_paste()")
            self.assertIsNone(result.error_in_exec)
            self.assertEqual(self.box.x, 7)
        self.launch(cells)
        self.assert_host_unchanged()
    def test_magics_are_run_by_actual_ipython_inside_checkpointed_paste(self):
        self.embed.clipboard = lambda: "# a\n%time box.x += 2"
        def cells(shell):
            self.embed.checkpoint_paste()
            self.embed.checkpoint_paste()
            self.assertEqual(self.box.x, 2)
        self.launch(cells)
        self.assertIn("Wall time:", self.output.getvalue())
        self.assert_host_unchanged()
    def test_actual_shell_error_does_not_hide_partial_state_or_prevent_retry(self):
        primary = RuntimeError("interactive failure")
        source = iter(("# a\nbox.x = 99\nraise primary", "# a\nbox.x += 1"))
        self.embed.clipboard = lambda: next(source)
        def cells(shell):
            shell.user_ns["primary"] = primary
            with self.assertRaises(RuntimeError) as caught:
                self.embed.checkpoint_paste()
            self.assertIs(caught.exception, primary)
            self.assertEqual(self.box.x, 99)
            self.embed.checkpoint_paste()
            self.assertEqual(self.box.x, 1)
        self.launch(cells)
        self.assert_host_unchanged()
    def test_abrupt_terminal_exit_restores_global_shell_state_and_removes_hooks(self):
        primary = KeyboardInterrupt("terminal closed")
        shell = self.make_shell()
        callbacks = list(shell.events.callbacks.get("post_run_cell", ()))
        def cells(shell):
            shell.run_cell("box.x = 3")
            raise primary
        with self.assertRaises(KeyboardInterrupt) as caught:
            self.launch(cells)
        self.assertIs(caught.exception, primary)
        self.assertEqual(shell.events.callbacks.get("post_run_cell", []), callbacks)
        self.assert_host_unchanged()
    def test_no_authored_source_module_reimport_or_namespace_mutation_on_shell_creation(self):
        namespace = {"box": self.box, "secret": "host-bound", "__file__": "/never/reimport/lesson.py"}
        self.embed._fmn_namespace = namespace
        shell = self.make_shell()
        self.assertNotIn("checkpoint_paste", namespace)
        self.assertEqual(shell.user_ns["secret"], "host-bound")
        self.assertIsInstance(shell.user_module, ModuleType)
        self.assert_host_unchanged()


if __name__ == "__main__":
    unittest.main()
