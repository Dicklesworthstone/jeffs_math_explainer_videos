"""The shipped CLI plus production parser/loader, substituting only native work."""
import ast
import contextlib
import io
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

from test_batch_rendering_protocol import Scene, EndScene
from fmn_python import __main__ as console
from fmn_python.batch_cli import try_batch_cli


ROOT = pathlib.Path(__file__).resolve().parents[1]
HELPERS = {"_portal_cli_emit", "_portal_cli_help", "_portal_cli_scene_types", "_portal_cli_render_arguments"}


def native_helpers():
    source = ROOT / "python/manimlib_bootstrap.py"
    tree = ast.parse(source.read_text())
    definitions = [node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name in HELPERS]
    assert {node.name for node in definitions} == HELPERS
    namespace = {"_sys": sys, "Scene": Scene}
    exec(compile(ast.Module(body=definitions, type_ignores=[]), str(source), "exec"), namespace)
    native = types.SimpleNamespace(Scene=Scene, EndScene=EndScene)
    for name in HELPERS:
        setattr(native, name, namespace[name])
    native._native = native
    native._console_main = lambda: 77
    return native


SOURCE = '''from manimlib import Scene
print("source-loaded")
class Zulu(Scene):
    def run(self):
        print("running-zulu")
        super().run()
class Alpha(Scene):
    def run(self):
        print("running-alpha")
        super().run()
'''


class BatchCliTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.source = self.root / "scenes.py"
        self.source.write_text(SOURCE)
        self.native = native_helpers()
        self.imports = patch.dict(sys.modules, {"manimlib": self.native})
        self.imports.start()
        self.addCleanup(self.imports.stop)
        self.exclusive = patch.object(console, "_ensure_exclusive_manimlib_namespace")
        self.exclusive.start()
        self.addCleanup(self.exclusive.stop)
        Scene.instances, Scene.trace = [], []
    def invoke(self, *arguments):
        output, errors = io.StringIO(), io.StringIO()
        with patch.object(sys, "argv", ["fmn-python", *map(str, arguments)]), contextlib.redirect_stdout(output), contextlib.redirect_stderr(errors):
            code = console.main()
        return code, output.getvalue(), errors.getvalue()
    def batch(self, *extra, source=None, directory=None):
        return self.invoke("--robot", source or self.source, "--write_all", "--format", "png",
                           "--video_dir", directory or self.root / "output", *extra)
    def test_shipped_entrypoint_runs_all_scenes_in_sorted_order_once(self):
        code, output, errors = self.batch()
        self.assertEqual(code, 0)
        data = json.loads(output)
        self.assertEqual(data["kind"], "render-batch")
        self.assertEqual(data["exit"]["code"], 0)
        self.assertTrue(data["all_succeeded"])
        self.assertEqual([x["name"] for x in data["batch"]["outcomes"]], ["Alpha", "Zulu"])
        self.assertEqual(errors.count("source-loaded"), 1)
        self.assertIn("running-alpha", errors)
        self.assertNotIn("running-alpha", output)
        self.assertEqual(len(Scene.instances), 2)
        self.assertTrue((self.root / "output/Alpha.png").exists())
    def test_original_parser_is_negative_control(self):
        with self.assertRaisesRegex(RuntimeError, "not connected"):
            self.native._portal_cli_render_arguments([str(self.source), "--write_all"])
    def test_keep_going_returns_nonzero_with_partial_receipts(self):
        self.source.write_text(SOURCE + '\nclass Middle(Scene):\n    def run(self):\n        raise ValueError("failure")\n')
        code, output, _ = self.batch("--keep-going")
        data = json.loads(output)
        self.assertEqual(code, 5)
        self.assertFalse(data["all_succeeded"])
        self.assertEqual(data["batch"]["counts"], dict(succeeded=2, failed=1, cancelled=0, not_run=0))
        self.assertEqual([x["status"] for x in data["batch"]["outcomes"]], ["succeeded", "failed", "succeeded"])
    def test_fail_fast_stops_without_constructing_later_scene(self):
        self.source.write_text(SOURCE + '\nclass Middle(Scene):\n    def run(self):\n        raise ValueError("failure")\n')
        code, output, _ = self.batch()
        self.assertEqual(code, 5)
        data = json.loads(output)
        self.assertEqual([x["status"] for x in data["batch"]["outcomes"]], ["succeeded", "failed", "not_run"])
        self.assertEqual([type(x).__name__ for x in Scene.instances], ["Alpha", "Middle"])
    def test_keyboard_interrupt_emits_one_receipt_after_native_abort(self):
        self.source.write_text(SOURCE + '\nclass Middle(Scene):\n    def run(self):\n        raise KeyboardInterrupt("cancel")\n')
        code, output, _ = self.batch("--keep-going")
        data = json.loads(output)
        self.assertEqual(code, 130)
        self.assertEqual(data["batch"]["counts"], dict(succeeded=1, failed=0, cancelled=1, not_run=1))
        self.assertEqual(Scene.instances[-1].events[-1], "abort")
    def test_scene_system_exit_zero_is_not_false_batch_success(self):
        self.source.write_text('from manimlib import Scene\nclass Stop(Scene):\n    def run(self):\n        raise SystemExit(0)\n')
        code, output, _ = self.batch()
        self.assertEqual(code, 5)
        self.assertEqual(json.loads(output)["batch"]["counts"]["cancelled"], 1)
    def test_loading_print_then_exception_keeps_robot_stdout_valid(self):
        self.source.write_text('print("loading")\nraise ValueError("load error")\n')
        code, output, errors = self.batch()
        self.assertEqual(code, 5)
        self.assertEqual(json.loads(output)["kind"], "scene-load-failed")
        self.assertIn("loading", errors)
        self.assertEqual(Scene.instances, [])
    def test_source_exit_does_not_report_completed_render(self):
        self.source.write_text('print("exit now")\nraise SystemExit(0)\n')
        code, output, _ = self.batch()
        self.assertEqual(code, 5)
        self.assertEqual(json.loads(output)["kind"], "render-interrupted")
        self.assertEqual(Scene.instances, [])
    def test_imported_scenes_are_not_rendered(self):
        helper = self.root / "batch_test_external.py"
        helper.write_text('from manimlib import Scene\nclass External(Scene): pass\n')
        self.source.write_text('from batch_test_external import External\n' + SOURCE)
        code, output, _ = self.batch()
        self.assertEqual(code, 0)
        self.assertEqual([x["name"] for x in json.loads(output)["batch"]["outcomes"]], ["Alpha", "Zulu"])
    def test_sibling_imports_work_during_construct_and_sys_path_is_restored(self):
        (self.root / "batch_lazy_helper.py").write_text('VALUE = 23\n')
        self.source.write_text('from manimlib import Scene\nclass Lazy(Scene):\n    def run(self):\n        from batch_lazy_helper import VALUE\n        self.random_seed = VALUE\n')
        before = list(sys.path)
        code, output, _ = self.batch()
        self.assertEqual(code, 0)
        self.assertEqual(sys.path, before)
        self.assertEqual(json.loads(output)["batch"]["counts"]["succeeded"], 1)
    def test_paths_freeze_before_source_module_changes_cwd(self):
        before = os.getcwd()
        self.addCleanup(os.chdir, before)
        os.chdir(self.root)
        moved = self.root / "changed"
        moved.mkdir()
        self.source.write_text(f'import os\nos.chdir({str(moved)!r})\n' + SOURCE)
        code, output, _ = self.invoke("--robot", "scenes.py", "--write_all", "--format", "png", "--video_dir", "output")
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(output)["destination"], str(self.root / "output"))
        self.assertTrue((self.root / "output/Alpha.png").exists())
    def test_default_directory_is_source_scoped(self):
        before = os.getcwd()
        self.addCleanup(os.chdir, before)
        os.chdir(self.root)
        code, output, _ = self.invoke("--robot", "scenes.py", "--write_all", "--format", "png")
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(output)["destination"], str(self.root / "media/videos/scenes"))
    def test_native_option_parser_values_are_forwarded(self):
        code, output, _ = self.batch("--fps=12", "--threads=3", "--resolution=16x8", "--format=y4m")
        self.assertEqual(code, 0)
        for item in json.loads(output)["batch"]["outcomes"]:
            self.assertEqual(item["result"]["resolution"], [16, 8])
            self.assertEqual(item["result"]["threads"], 3)
            self.assertEqual(item["result"]["fps"], 12)
            self.assertEqual(item["result"]["format"], "y4m")
    def test_invalid_configuration_never_executes_source(self):
        for extra in (("--fps", "0"), ("--threads", str(1 << 32)), ("--resolution", "bad"), ("--fps",), ("--unknown",)):
            code, output, errors = self.batch(*extra)
            self.assertEqual(code, 2)
            self.assertEqual(json.loads(output)["kind"], "usage-error")
            self.assertNotIn("source-loaded", errors)
        self.assertEqual(Scene.instances, [])
    def test_existing_destination_fails_without_constructing_any_scene(self):
        destination = self.root / "output"
        destination.mkdir()
        (destination / "Zulu.png").write_bytes(b"preserve")
        code, output, _ = self.batch()
        self.assertEqual(code, 6)
        self.assertEqual(json.loads(output)["kind"], "render-start-failed")
        self.assertEqual(Scene.instances, [])
        self.assertEqual((destination / "Zulu.png").read_bytes(), b"preserve")
    def test_capability_flags_remain_precise_refusals(self):
        for extra in (("--reproducible",), ("--autoreload",), ("-o",), ("--format", "webp")):
            code, output, errors = self.batch(*extra)
            self.assertEqual(code, 4)
            self.assertEqual(json.loads(output)["kind"], "render-capability-unavailable")
            self.assertNotIn("source-loaded", errors)
    def test_exclusive_scene_name_and_write_all_are_rejected(self):
        code, output, errors = self.batch("Alpha")
        self.assertEqual(code, 2)
        self.assertIn("cannot be combined with explicit scene names", json.loads(output)["message"])
        self.assertNotIn("source-loaded", errors)
    def test_keep_going_alone_and_repeated_switches_are_rejected(self):
        for args in (("--keep-going",), ("--write_all", "--write_all"), ("--write_all", "--robot", "--robot")):
            code, output, errors = self.invoke("--robot", self.source, *args)
            self.assertEqual(code, 2)
            self.assertEqual(json.loads(output)["kind"], "usage-error")
            self.assertNotIn("source-loaded", errors)
    def test_switch_named_option_values_are_not_treated_as_switches(self):
        for value in ("--write_all", "--keep-going", "--robot"):
            self.assertIsNone(try_batch_cli(self.native, [str(self.source), "--video_dir", value]))
    def test_ordinary_cli_modes_are_delegated_unchanged(self):
        for args in (("--version",), ("--list-scenes", str(self.source)), (str(self.source), "Alpha"), ("studio", str(self.source))):
            self.assertIsNone(try_batch_cli(self.native, list(args)))
    def test_empty_discovery_is_not_batch_success(self):
        self.source.write_text("from manimlib import Scene\n")
        code, output, _ = self.batch()
        self.assertEqual(code, 5)
        self.assertIn("no locally declared", json.loads(output)["message"])
    def test_help_includes_batch_usage_without_stale_refusal(self):
        code, output, errors = self.invoke("--help")
        self.assertEqual(code, 0)
        self.assertIn("--write_all", output)
        self.assertIn("--keep-going", output)
        self.assertNotIn("opener flags, write-all", output)
        code, output, _ = self.invoke("--write_all", "--help", "--robot")
        self.assertEqual(code, 0)
        self.assertIn("--keep-going", json.loads(output)["help"])
        self.assertEqual(Scene.instances, [])
    def test_human_mode_prints_progress_and_final_summary(self):
        code, output, errors = self.invoke(self.source, "--write_all", "--format", "png", "--video_dir", self.root / "output")
        self.assertEqual(code, 0)
        self.assertIn("2 succeeded", output)
        self.assertIn("Alpha: succeeded", errors)
        self.assertIn("source-loaded", output)
    def test_actual_python_module_process_emits_one_json_record(self):
        driver = self.root / "driver.py"
        driver.write_text('import runpy, sys\nfrom test_batch_cli_protocol import native_helpers\n'
                          'import fmn_python\nfmn_python._ensure_exclusive_manimlib_namespace = lambda: None\n'
                          'sys.modules["manimlib"] = native_helpers()\n'
                          'runpy.run_module("fmn_python", run_name="__main__")\n')
        env = dict(os.environ, PYTHONPATH=os.pathsep.join([str(ROOT / "python"), str(ROOT / "tests")]))
        result = subprocess.run([sys.executable, str(driver), "--robot", str(self.source), "--write_all",
                                 "--format", "png", "--video_dir", str(self.root / "output")],
                                capture_output=True, text=True, env=env, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["batch"]["counts"]["succeeded"], 2)
        self.assertEqual(result.stderr.count("source-loaded"), 1)


    def test_namespace_collision_precedes_source_execution(self):
        from fmn_python import _ManimlibNamespaceCollision
        with patch.object(console, "_ensure_exclusive_manimlib_namespace", side_effect=_ManimlibNamespaceCollision(["other-provider"])):
            code, output, errors = self.batch()
        self.assertEqual(code, 4)
        self.assertEqual(json.loads(output)["kind"], "namespace-collision")
        self.assertNotIn("source-loaded", errors)
        self.assertEqual(Scene.instances, [])

    def test_progress_output_failure_retains_the_published_receipt(self):
        class BrokenProgress(io.StringIO):
            def write(self, text):
                if "fmn-python:" in text:
                    raise OSError("progress stream closed")
                return super().write(text)
        output = io.StringIO()
        args = ["fmn-python", "--robot", str(self.source), "--write_all", "--format", "png",
                "--video_dir", str(self.root / "output")]
        with patch.object(sys, "argv", args), contextlib.redirect_stdout(output), contextlib.redirect_stderr(BrokenProgress()):
            code = console.main()
        self.assertEqual(code, 6)
        data = json.loads(output.getvalue())
        self.assertEqual(data["kind"], "render-batch-reporting-failed")
        self.assertEqual([x["status"] for x in data["batch"]["outcomes"]], ["succeeded", "not_run"])
        self.assertTrue(pathlib.Path(data["batch"]["outcomes"][0]["destination"]).exists())


    def test_independent_y4m_reader_with_known_bytes_and_corruption_controls(self):
        source = ROOT / "tests/batch_rendering.py"
        tree = ast.parse(source.read_text())
        definitions = [node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name in {"read_y4m", "centers"}]
        namespace = {"pathlib": pathlib}
        exec(compile(ast.Module(body=definitions, type_ignores=[]), str(source), "exec"), namespace)
        header = b"YUV4MPEG2 W16 H8 F8:1 Ip A1:1 C420mpeg2\n"
        records = bytearray()
        for left in (4, 8):
            frame = bytearray([16] * (16 * 8))
            for y in range(2, 6):
                for x in range(left, left + 4):
                    frame[y * 16 + x] = 235
            records.extend(b"FRAME\n" + frame + bytes([128] * 64))
        path = self.root / "known.y4m"
        path.write_bytes(header + records)
        frames = namespace["read_y4m"](path, width=16, height=8, fps=8)
        self.assertEqual(namespace["centers"](frames, width=16), [5.5, 9.5])
        path.write_bytes(header + records[:-1])
        with self.assertRaises(AssertionError):
            namespace["read_y4m"](path, width=16, height=8, fps=8)
        path.write_bytes(header + records.replace(b"FRAME", b"WRONG", 1))
        with self.assertRaises(AssertionError):
            namespace["read_y4m"](path, width=16, height=8, fps=8)


if __name__ == "__main__":
    unittest.main()
