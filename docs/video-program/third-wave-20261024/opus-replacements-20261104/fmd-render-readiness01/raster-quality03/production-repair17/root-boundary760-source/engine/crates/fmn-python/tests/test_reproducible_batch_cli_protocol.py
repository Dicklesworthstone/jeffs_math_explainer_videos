"""Real console/batch adapters with explicit parser, loader and sink doubles.

No native parsing/rendering claim: the unchanged native parser and SceneSource
have their own acceptance suites. This tests forwarding, preflight, output
redirection and progress receipts through the production Python call chain.
"""
from contextlib import redirect_stdout, redirect_stderr
import io
import json
from pathlib import Path
import sys
import types
import unittest
from unittest.mock import patch

import test_reproducible_batch_protocol as core


class BatchCliTests(unittest.TestCase):
    scenes = core.ReproducibleBatchTests.scenes
    sink = core.ReproducibleBatchTests.sink

    def setUp(self):
        core.ReproducibleBatchTests.setUp(self)
        self.source = self.root / "scene.py"
        self.source.write_bytes(b"explicit source-loader double")
        self.classes = self.scenes("Two", "One")
        self.loaded = False
        self.load_count = 0
        self.loaded_sources = {"scene.py": self.source.read_bytes()}
        outer = self
        class Source:
            def __init__(self, path, Scene):
                self.scenes = outer.classes
            def __enter__(self):
                outer.loaded = True
                outer.load_count += 1
                print("authored import stdout")
                return self
            def __exit__(self, *args):
                outer.loaded = False
            @property
            def sources(self):
                if not outer.loaded:
                    raise AssertionError("provider ran after source loader closed")
                return outer.loaded_sources
        loader = types.ModuleType("_fmn_batch_protocol.scene_loading")
        loader.SceneSource = Source
        sys.modules[loader.__name__] = loader
        self.rendering.RenderSession = lambda *a, **k: self.fail("batch entered single-scene CLI path")
        self.rendering._apply_output_options = lambda *a: None
        self.legacy = core._load("_fmn_batch_protocol.batch_cli", core._PYTHON / "batch_cli.py")
        core._load("_fmn_batch_protocol.checkpoint_cli", core._PYTHON / "checkpoint_cli.py")
        self.console = core._load("_fmn_batch_protocol.console_rendering", core._PYTHON / "console_rendering.py")
        self.native._portal_cli_render_arguments = self.parse
        self.native._portal_cli_emit = self.emit
        self.native._portal_cli_help = lambda: "native help"
        self.parsed, self.emitted = [], []

    def parse(self, args):
        # Native parser double, deliberately small; do not use as production parsing.
        self.parsed.append(list(args))
        options = dict(format="png_sequence", video_dir=None, reproducible=False)
        positionals, index = [], 0
        while index < len(args):
            arg = args[index]
            if arg == "--reproducible":
                options["reproducible"] = True
            elif arg == "--transparent":
                options["transparent"] = True
            elif arg in self.legacy._VALUE_FLAGS:
                if index + 1 == len(args):
                    raise ValueError("missing native option value")
                options[arg[2:]] = args[index + 1]
                index += 1
            elif arg.startswith("-"):
                raise ValueError("unsupported native option " + arg)
            else:
                positionals.append(arg)
            index += 1
        if not positionals:
            raise ValueError("source required")
        return positionals, options, 48, 27, 8, 4

    def emit(self, code, identity, kind, message, robot, **details):
        record = dict(schema="fmn-python.cli", version=1, kind=kind,
                      exit={"code": code, "identity": identity}, message=message, **details)
        self.emitted.append(record)
        print(json.dumps(record))
        return code

    def invoke(self, selectors, *, options=(), legacy=False, reproducible=True):
        args = ["--robot", str(self.source), *selectors, "--video_dir", str(self.root / "outputs")]
        if reproducible:
            args.append("--reproducible")
        args.extend(options)
        out, err = io.StringIO(), io.StringIO()
        handler = self.legacy.try_batch_cli if legacy else self.console.try_render_cli
        with redirect_stdout(out), redirect_stderr(err):
            code = handler(self.native, args)
        lines = out.getvalue().splitlines()
        self.assertEqual(len(lines), 1, out.getvalue())
        return code, json.loads(lines[0]), err.getvalue()

    def test_named_reproducible_batch_order_and_provider_scope(self):
        code, receipt, err = self.invoke(["Two", "One"], options=["--format", "png"])
        self.assertEqual(code, 0)
        report = receipt["batch"]
        self.assertEqual([x["name"] for x in report["outcomes"]], ["Two", "One"])
        self.assertTrue(report["reproducible_requested"] and report["all_scenes_certified"])
        self.assertFalse(report["certified"])
        self.assertEqual(self.sources_seen, [self.loaded_sources] * 2)
        self.assertFalse(self.loaded)
        self.assertIn("authored import stdout", err)
        self.assertIn("Two: succeeded", err)
        self.assertNotIn("Two", self.parsed[0], "native parser receives one source, not extra selectors")

    def test_write_all_uses_sorted_names(self):
        code, receipt, _ = self.invoke(["--write_all"])
        self.assertEqual(code, 0)
        self.assertEqual([x["name"] for x in receipt["batch"]["outcomes"]], ["One", "Two"])
        self.assertTrue(receipt["batch"]["all_scenes_certified"])

    def test_short_write_all_alias(self):
        code, receipt, _ = self.invoke(["-a"])
        self.assertEqual(code, 0)
        self.assertTrue(receipt["batch"]["all_scenes_certified"])

    def test_legacy_write_all_delegates_same_provenance_path(self):
        code, receipt, _ = self.invoke(["--write_all"], legacy=True)
        self.assertEqual(code, 0)
        self.assertTrue(receipt["batch"]["all_scenes_certified"])
        self.assertEqual(len(self.snapshots), 1)

    def test_noncertified_formats_refuse_before_source_import(self):
        for format in ("mp4", "mov", "gif", "y4m", "svg"):
            with self.subTest(format=format):
                code, receipt, _ = self.invoke(["One", "Two"], options=["--format", format])
                self.assertEqual((code, receipt["exit"]["identity"]), (4, "capability"))
        self.assertEqual(self.load_count, 0)
        self.assertEqual(self.calls, [])

    def test_legacy_unsupported_format_refuses_before_import(self):
        code, _, _ = self.invoke(["--write_all"], legacy=True, options=["--format", "mp4"])
        self.assertEqual(code, 4)
        self.assertEqual(self.load_count, 0)

    def test_checkpoint_combination_refuses_before_import_or_lock(self):
        path = self.root / "progress.json"
        code, receipt, _ = self.invoke(["One", "Two"], options=["--checkpoint", str(path), "--resume-key", "inputs"])
        self.assertEqual(code, 4)
        self.assertIn("checkpoint", receipt["message"])
        self.assertFalse(path.exists())
        self.assertEqual(self.load_count, 0)

    def test_batch_does_not_treat_root_as_single_artifact(self):
        (self.root / "outputs.manifest").write_bytes(b"unrelated directory-sidecar name")
        code, receipt, _ = self.invoke(["One", "Two"])
        self.assertEqual(code, 0)
        self.assertTrue(receipt["batch"]["all_scenes_certified"])

    def test_late_sidecar_collision_reports_preflight_failure_without_any_scene(self):
        path = self.root / "outputs" / "Two" / "frames.manifest"
        path.parent.mkdir(parents=True)
        path.write_bytes(b"preserve")
        code, receipt, _ = self.invoke(["One", "Two"])
        self.assertEqual(code, 6)
        self.assertEqual(receipt["kind"], "render-start-failed")
        self.assertEqual(self.events, [])
        self.assertEqual(path.read_bytes(), b"preserve")

    def test_runtime_preflight_unavailable_is_capability_not_usage(self):
        def capture(native):
            raise self.runtime.RuntimeIdentityError("runtime payload missing")
        self.runtime.capture_runtime = capture
        code, receipt, _ = self.invoke(["One", "Two"])
        self.assertEqual((code, receipt["exit"]["identity"]), (4, "capability"))
        self.assertEqual(self.events, [])

    def test_standard_batch_still_has_no_runtime_requirement(self):
        self.runtime.capture_runtime = lambda *a: self.fail("ordinary batch hashed runtime")
        code, receipt, _ = self.invoke(["One", "Two"], reproducible=False)
        self.assertEqual(code, 0)
        self.assertFalse(receipt["batch"]["reproducible_requested"])

    def test_partial_failure_and_keep_going_preserve_per_scene_manifests(self):
        class Broken(self.Scene):
            def run(self):
                print("authored failing scene")
                raise ValueError("broken")
        self.classes["Broken"] = Broken
        code, receipt, err = self.invoke(["One", "Broken", "Two"], options=["--keep-going"])
        self.assertEqual(code, 5)
        report = receipt["batch"]
        self.assertEqual([x["status"] for x in report["outcomes"]], ["succeeded", "failed", "succeeded"])
        self.assertTrue(report["reproducible_requested"])
        self.assertFalse(report["all_scenes_certified"])
        self.assertIn("manifest", report["outcomes"][0]["result"])
        self.assertIn("authored failing scene", err)

    def test_interrupt_emits_one_partial_receipt_with_requested_mode(self):
        class Stop(self.Scene):
            def run(self):
                raise KeyboardInterrupt("stop")
        self.classes["Stop"] = Stop
        code, receipt, _ = self.invoke(["One", "Stop", "Two"])
        self.assertEqual(code, 130)
        report = receipt["batch"]
        self.assertTrue(report["reproducible_requested"])
        self.assertEqual([x["status"] for x in report["outcomes"]], ["succeeded", "cancelled", "not_run"])

    def test_skip_and_range_forward_to_reproducible_scenes(self):
        code, receipt, _ = self.invoke(["One", "Two"], options=["-s", "-n", "1,3", "--transparent"])
        self.assertEqual(code, 0)
        for row in receipt["batch"]["outcomes"]:
            self.assertEqual(row["result"]["format"], "png")
            self.assertEqual(row["result"]["animation_range"], [1, 3])
        self.assertEqual(self.calls[0][2]["_output_options"], {"transparent": True})

    def test_unknown_scene_never_runs_a_partial_selection(self):
        code, receipt, _ = self.invoke(["One", "Absent"])
        self.assertEqual(code, 5)
        self.assertEqual(receipt["kind"], "scene-load-failed")
        self.assertEqual(self.calls, [])

    def test_missing_native_publisher_refuses_before_loading(self):
        self.native._portal_publish_manifest = None
        code, _, _ = self.invoke(["One", "Two"])
        self.assertEqual(code, 4)
        self.assertEqual(self.load_count, 0)


if __name__ == "__main__":
    unittest.main()
