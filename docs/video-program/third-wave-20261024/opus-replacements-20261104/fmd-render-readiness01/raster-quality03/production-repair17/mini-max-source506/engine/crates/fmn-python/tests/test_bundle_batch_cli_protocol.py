"""Shipping console/parser/source-loader; explicitly modeled native bundle sink.

The payload is not FMTL. Native capture and byte comparisons live separately in
bundle_batch_console.py; this suite isolates option, ownership and error routes.
"""
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python import __main__ as console
from fmn_python.bundle_cli import try_bundle_cli
from test_console_rendering_protocol import native_fixture


SOURCE = '''from manimlib import Scene
print("source-loaded")
class Zulu(Scene):
    def construct(self):
        print("running-zulu")
class Alpha(Scene):
    def construct(self):
        print("running-alpha")
'''


class BundleBatchCliProtocol(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="fmn-bundle-batch-cli-protocol-"))
        self.cwd = Path.cwd()
        self.addCleanup(os.chdir, self.cwd)
        os.chdir(self.root)
        self.source = self.root / "scenes.py"
        self.source.write_text(SOURCE)
        self.directory = self.root / "output"
        self.native = native_fixture()
        self.native._native = self.native
        self.native._console_main = lambda: 77
        self.calls = []

        def begin(scene, path, width, height, fps, seed, *limits, camera=False):
            self.calls.append((type(scene).__name__, camera, path, width, height, fps, limits))
            if scene.active:
                raise RuntimeError("an earlier output owner is still active")
            scene.active = True
            scene.destination = path

        def finish(scene):
            destination = Path(scene.destination)
            destination.parent.mkdir(parents=True, exist_ok=True)
            payload = ("modeled-bundle-not-FMTL:" + type(scene).__name__).encode()
            with destination.open("xb") as handle:
                handle.write(payload)
            scene.active = False
            return str(destination), 7, 2, len(payload), hashlib.sha256(payload).hexdigest()

        self.native._portal_begin_bundle = begin
        self.native._portal_begin_camera_bundle = lambda *args: begin(*args, camera=True)
        self.native._portal_bundle_segment = lambda *args: None
        self.native._portal_finish_bundle = finish
        self.modules = patch.dict(sys.modules, {"manimlib": self.native})
        self.modules.start()
        self.addCleanup(self.modules.stop)
        self.exclusive = patch.object(console, "_ensure_exclusive_manimlib_namespace")
        self.exclusive.start()
        self.addCleanup(self.exclusive.stop)

    def invoke(self, *arguments):
        out, err = io.StringIO(), io.StringIO()
        with (patch.object(sys, "argv", ["fmn-python", *map(str, arguments)]),
              contextlib.redirect_stdout(out), contextlib.redirect_stderr(err)):
            code = console.main()
        lines = out.getvalue().splitlines()
        self.assertEqual(len(lines), 1, out.getvalue())
        return code, json.loads(lines[0]), err.getvalue()

    def batch(self, *extra, names=(), directory=None):
        return self.invoke("--robot", self.source, *names, "--format", "fmtl",
                           "--resolution", "128x72", "--fps", "24", "--video_dir",
                           self.directory if directory is None else directory, *extra)

    def test_write_all_and_alias_use_single_loader_and_sorted_independent_bundles(self):
        for flag in ("--write_all", "-a"):
            with self.subTest(flag=flag):
                code, receipt, stderr = self.batch(flag, directory=self.directory / flag)
                self.assertEqual(code, 0)
                self.assertEqual(receipt["kind"], "render-batch")
                self.assertTrue(receipt["all_succeeded"])
                self.assertEqual([r["name"] for r in receipt["batch"]["outcomes"]], ["Alpha", "Zulu"])
                self.assertEqual(stderr.count("source-loaded"), 1)
                self.assertIn("running-alpha", stderr)
                self.assertIn("Alpha: succeeded", stderr)
                for row in receipt["batch"]["outcomes"]:
                    self.assertTrue(Path(row["destination"]).is_file())
                    self.assertEqual(row["result"]["format"], "fmtl")
                    self.assertNotIn("fmtl_minor", row["result"])
        self.assertEqual([row[:2] for row in self.calls], [("Alpha", False), ("Zulu", False)] * 2)

    def test_named_camera_order_and_common_options(self):
        code, receipt, _ = self.batch("--bundle-camera", names=("Zulu", "Alpha"))
        self.assertEqual(code, 0)
        self.assertEqual([r[0:2] for r in self.calls], [("Zulu", True), ("Alpha", True)])
        for call, row in zip(self.calls, receipt["batch"]["outcomes"]):
            self.assertEqual(call[3:6], (128, 72, 24))
            self.assertTrue(row["result"]["camera_track"])
            self.assertEqual(row["result"]["fmtl_minor"], 1)
            self.assertFalse(row["result"]["certified_source"])
            self.assertEqual(Path(row["destination"]).suffix, ".fmtl")

    def test_failed_middle_keeps_previous_and_keep_going_controls_later_constructor(self):
        self.source.write_text(SOURCE + '\nclass Middle(Scene):\n    def construct(self):\n        raise ValueError("authored failure")\n')
        for keep in (False, True):
            self.native.instances.clear()
            code, receipt, _ = self.batch("--write_all", *(("--keep-going",) if keep else ()),
                                          directory=self.directory / str(keep))
            self.assertEqual(code, 5)
            rows = receipt["batch"]["outcomes"]
            self.assertEqual([r["status"] for r in rows], ["succeeded", "failed", "succeeded" if keep else "not_run"])
            self.assertTrue(Path(rows[0]["destination"]).exists())
            self.assertFalse(Path(rows[1]["destination"]).exists())
            self.assertEqual(Path(rows[2]["destination"]).exists(), keep)
            self.assertEqual(len(self.native.instances), 3 if keep else 2)
            self.assertFalse(any(scene.active for scene in self.native.instances))

    def test_interrupt_and_system_exit_never_report_success_or_run_later_scene(self):
        for exception, expected in (("KeyboardInterrupt()", 130), ("SystemExit(0)", 5)):
            self.source.write_text(SOURCE + f'\nclass Middle(Scene):\n    def construct(self):\n        raise {exception}\n')
            self.native.instances.clear()
            code, report, _ = self.batch("--write_all", "--keep-going", directory=self.directory / str(expected))
            self.assertEqual(code, expected)
            self.assertEqual(report["kind"], "render-batch-interrupted")
            self.assertEqual([r["status"] for r in report["batch"]["outcomes"]], ["succeeded", "cancelled", "not_run"])
            self.assertEqual(len(self.native.instances), 2)
            self.assertFalse(any(scene.active for scene in self.native.instances))

    def test_later_output_collision_preflights_all_constructors(self):
        self.directory.mkdir()
        protected = self.directory / "Zulu.fmtl"
        protected.write_bytes(b"existing user artifact")
        code, report, _ = self.batch("--write_all")
        self.assertEqual(code, 6)
        self.assertEqual(report["phase"], "batch")
        self.assertEqual(protected.read_bytes(), b"existing user artifact")
        self.assertEqual(self.native.instances, [])
        self.assertFalse((self.directory / "Alpha.fmtl").exists())

    def test_occupied_non_directory_refuses_before_source(self):
        self.directory.write_bytes(b"keep")
        self.source.write_text('raise AssertionError("source executed")\n')
        code, report, _ = self.batch("--write_all")
        self.assertEqual(code, 6)
        self.assertEqual(report["phase"], "start")
        self.assertEqual(self.calls, [])

    def test_unknown_and_imported_scene_names_refuse_before_any_constructor(self):
        (self.root / "foreign.py").write_text("from manimlib import Scene\nclass Imported(Scene):\n    pass\n")
        self.source.write_text("from foreign import Imported\n" + SOURCE)
        for names in (("Alpha", "Absent"), ("Alpha", "Imported")):
            code, report, _ = self.batch(names=names)
            self.assertEqual(code, 2)
            self.assertEqual(report["phase"], "select")
            self.assertEqual(self.native.instances, [])
        code, report, _ = self.batch("--write_all")
        self.assertEqual(code, 0)
        self.assertEqual([r["name"] for r in report["batch"]["outcomes"]], ["Alpha", "Zulu"])
        self.assertNotIn("foreign", sys.modules)

    def test_selection_errors_are_usage_before_source(self):
        self.source.write_text('raise AssertionError("source executed")\n')
        cases = [(("Alpha", "Alpha"), ()), (("Alpha", "alpha"), ()),
                 (("Alpha", "CON"), ()), (("Alpha",), ("--write_all",)),
                 ((), ("--write_all", "-a")), ((), ("--write_all", "--write_all")),
                 (("Alpha", "Zulu"), ("--keep-going", "--keep-going")),
                 (("Alpha",), ("--keep-going",)), ((), ("--keep-going",)),
                 (tuple("Scene" + str(i) for i in range(1025)), ())]
        for names, flags in cases:
            with self.subTest(names=names[:2], flags=flags):
                code, report, _ = self.batch(*flags, names=names)
                self.assertEqual(code, 2)
                self.assertEqual(report["phase"], "options")
        self.assertEqual(self.calls, [])

    def test_camera_capability_and_unsupported_output_options_refuse_before_source(self):
        self.source.write_text('raise AssertionError("source executed")\n')
        for flags in (("--threads", "4"), ("--reproducible",), ("--subdivide",), ("-n", "1")):
            code, report, _ = self.batch("--write_all", *flags)
            self.assertEqual(code, 4)
            self.assertEqual(report["phase"], "options")
        del self.native._portal_begin_camera_bundle
        code, report, _ = self.batch("--write_all", "--bundle-camera")
        self.assertEqual(code, 4)
        self.assertEqual(report["phase"], "options")
        self.assertEqual(self.calls, [])

    def test_lazy_import_scope_and_default_destinations_survive_authored_chdir(self):
        child = self.root / "changed"
        child.mkdir()
        (self.root / "helper.py").write_text("VALUE = 'retained sibling helper'\n")
        self.source.write_text("import os\n" + SOURCE.replace('print("source-loaded")', f'print("source-loaded")\nos.chdir({str(child)!r})')
                               .replace('print("running-alpha")', 'import helper\n        print(helper.VALUE)'))
        code, report, stderr = self.invoke("--robot", self.source, "--format=fmtl", "--write_all")
        self.assertEqual(code, 0)
        self.assertEqual(Path(report["destination"]), self.root / "media/videos/scenes")
        self.assertIn("retained sibling helper", stderr)
        self.assertNotIn("helper", sys.modules)
        for row in report["batch"]["outcomes"]:
            self.assertTrue(Path(row["destination"]).is_relative_to(self.root / "media/videos/scenes"))

    def test_single_scene_keeps_exact_file_and_receipt_contract(self):
        path = self.root / "exact-output.fmtl"
        code, report, _ = self.batch(names=("Zulu",), directory=path)
        self.assertEqual(code, 0)
        self.assertEqual(report["kind"], "bundle-export")
        self.assertEqual(report["destination"], str(path))
        self.assertNotIn("batch", report)
        self.assertTrue(path.is_file())
        self.assertNotIn("fmtl_minor", report["bundle"])

    def test_write_all_with_one_scene_is_still_a_batch_directory(self):
        self.source.write_text("from manimlib import Scene\nclass Only(Scene):\n    pass\n")
        code, report, _ = self.batch("--write_all")
        self.assertEqual(code, 0)
        self.assertEqual(report["kind"], "render-batch")
        self.assertTrue((self.directory / "Only.fmtl").exists())

    def test_empty_write_all_is_usage_without_output(self):
        self.source.write_text("from manimlib import Scene\n")
        code, report, _ = self.batch("--write_all")
        self.assertEqual(code, 2)
        self.assertEqual(report["phase"], "select")
        self.assertFalse(self.directory.exists())

    def test_flags_used_as_option_values_do_not_select_batch_or_camera(self):
        code, report, _ = self.batch(names=("Alpha",), directory="--bundle-camera")
        self.assertEqual(code, 0)
        self.assertEqual(report["kind"], "bundle-export")
        self.assertFalse(self.calls[0][1])
        self.assertEqual(Path(report["destination"]).name, "--bundle-camera")
        self.assertIsNone(try_bundle_cli(self.native, [str(self.source), "--format", "png", "--video_dir", "fmtl"]))


if __name__ == "__main__":
    unittest.main()
