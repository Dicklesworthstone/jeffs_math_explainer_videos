"""Real host IPython cell execution; native scene storage is a named test double."""
from contextlib import redirect_stderr, redirect_stdout
import io
import unittest

try:
    from IPython.core.interactiveshell import InteractiveShell
    HAVE_IPYTHON = True
except ImportError:
    HAVE_IPYTHON = False

from test_scene_console_protocol import Scene, NATIVE
from fmn_python import SceneConsole


@unittest.skipUnless(HAVE_IPYTHON, "IPython is not installed")
class IPythonConsoleTests(unittest.TestCase):
    def setUp(self):
        self.scene = Scene()
        self.shell = InteractiveShell()
        self.console = SceneConsole(self.scene, {"box": self.scene.mobjects[0]}, shell=self.shell, _native=NATIVE)
        self.addCleanup(self.console.close)
    def test_real_ipython_cells_bind_scene_namespace_and_restore_checkpoint(self):
        with redirect_stdout(io.StringIO()):
            self.console.run_cell("# move\nbox.x += 2\nanswer = 42")
            self.console.run_cell("# move\nbox.x += 7")
        self.assertEqual(self.scene.mobjects[0].x, 7)
        self.assertEqual(self.console.namespace["answer"], 42)
        self.assertIs(self.console.namespace, self.shell.user_ns)
        self.assertIs(self.shell.user_ns["scene"], self.scene)
    def test_real_ipython_runtime_error_preserves_identity_and_retry(self):
        error = RuntimeError("cell-failure")
        self.console.namespace["error"] = error
        with redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            with self.assertRaises(RuntimeError) as caught:
                self.console.run_cell("# move\nbox.x = 8\nraise error")
            self.assertIs(caught.exception, error)
            self.console.run_cell("# move\nbox.x += 1")
        self.assertEqual(self.scene.mobjects[0].x, 1)
    def test_ipython_magic_is_validated_and_executed_by_ipython_not_python_exec(self):
        with redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            self.console.run_cell("# magic\n%time box.x += 3")
            self.console.run_cell("# magic\n%time box.x += 4")
        self.assertEqual(self.scene.mobjects[0].x, 4)
    def test_ipython_syntax_failure_does_not_rewind_scene(self):
        with redirect_stdout(io.StringIO()):
            self.console.run_cell("# a\nbox.x = 8")
        with self.assertRaises(SyntaxError):
            self.console.run_cell("# a\ninvalid(")
        self.assertEqual(self.scene.mobjects[0].x, 8)
    def test_top_level_await_remains_on_the_host_ipython_runner(self):
        with redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            self.console.run_cell("import asyncio\nawait asyncio.sleep(0)\nbox.x = 5")
        self.assertEqual(self.scene.mobjects[0].x, 5)
    def test_host_shell_namespace_is_not_erased_on_close(self):
        self.console.run_cell("host_value = 9")
        self.console.close()
        self.assertEqual(self.shell.user_ns["host_value"], 9)


if __name__ == "__main__":
    unittest.main()
