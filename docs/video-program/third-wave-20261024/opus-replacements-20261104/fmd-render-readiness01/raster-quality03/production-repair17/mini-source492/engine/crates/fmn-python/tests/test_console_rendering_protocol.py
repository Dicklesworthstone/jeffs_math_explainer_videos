"""Run the actual CLI parser/session/batch owners against a fake native sink.

The sink below proves calls, ownership and failure ordering, not rasterization
or codec correctness. Real-extension output acceptance is a separate suite.
"""
from __future__ import annotations

import ast
import contextlib
import hashlib
import importlib
import io
import json
import os
from pathlib import Path
import sys
import tempfile
from types import ModuleType, SimpleNamespace
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = "_fmn_console_protocol"
package = ModuleType(PACKAGE)
package.__path__ = [str(ROOT / "python/fmn_python")]
sys.modules[PACKAGE] = package
console = importlib.import_module(PACKAGE + ".console_rendering")


def native_fixture():
    native = ModuleType("manimlib")
    native.instances, native.events = [], []

    class EndScene(Exception):
        pass

    class CapabilityError(RuntimeError):
        pass

    class Scene:
        random_seed = 0
        start_error = finish_error = abort_error = None
        writer_options = {}
        fail_provenance = False

        def __init__(self):
            native.events.append(("construct", type(self).__name__))
            native.instances.append(self)
            self.events = []
            self.frame = object()
            self.dimensions = (12, 8)
            self.camera = SimpleNamespace(
                frame=self.frame, fps=30,
                _core=SimpleNamespace(set_pixel_shape=self.set_pixel_shape),
                get_pixel_shape=lambda: self.dimensions,
            )
            self.file_writer = SimpleNamespace(**self.writer_options)
            self.active = self.published = False
            self._render_audio_inputs = []

        def set_pixel_shape(self, width, height):
            self.dimensions = width, height

        def _begin_native_output(self, *arguments):
            self.events.append("begin")
            if self.start_error is not None:
                raise self.start_error
            if self.active:
                raise RuntimeError("external generation already exists")
            self.arguments = arguments
            self.destination = Path(arguments[0])
            if self.destination.exists():
                raise FileExistsError(str(self.destination))
            self.active = True

        def _abort_render(self):
            self.events.append("abort")
            self.active = False
            if self.abort_error is not None:
                raise self.abort_error

        def _finish_render(self):
            self.events.append("finish")
            if self.finish_error is not None:
                raise self.finish_error
            assert self.camera.fps == self.arguments[4]
            assert self.dimensions == self.arguments[2:4]
            assert self.camera.frame is self.frame
            payload = b"fixture sink, NOT native rendered pixels"
            path = self.destination
            if self.arguments[1] == "png_sequence":
                path.mkdir(parents=True)
                path = path / "fixture.bin"
            else:
                path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(payload)
            self.active, self.published = False, True
            native.events.append(("published", type(self).__name__))
            return str(self.destination), 2, len(payload), hashlib.sha256(payload).hexdigest(), "fixture-only", self.arguments[5]

        @property
        def _render_invocations(self):
            if self.fail_provenance:
                raise RuntimeError("provenance unavailable")
            return [{"fixture": True}]

        def run(self):
            self.events.append("run")
            try:
                self.construct()
            finally:
                self.tear_down()

        def construct(self):
            pass

        def tear_down(self):
            self.events.append("tear_down")

    native.Scene, native.EndScene, native._CapabilityError = Scene, EndScene, CapabilityError
    # Extract exact production definitions, never a second flag parser.
    source = ROOT / "python/manimlib_bootstrap.py"
    wanted = {"_portal_cli_render_arguments", "_portal_cli_scene_types", "_portal_cli_emit"}
    nodes = [node for node in ast.parse(source.read_text()).body
             if isinstance(node, ast.FunctionDef) and node.name in wanted]
    if {node.name for node in nodes} != wanted:
        raise RuntimeError("production CLI definitions missing")
    g = vars(native)
    g["_sys"] = sys
    exec(compile(ast.Module(body=nodes, type_ignores=[]), str(source), "exec"), g)
    native._portal_cli_help = lambda: "native help"
    return native


class ConsoleRenderingProtocol(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="fmn-console-protocol-"))
        self.native = native_fixture()
        self.modules = patch.dict(sys.modules, {"manimlib": self.native})
        self.modules.start()
        self.addCleanup(self.modules.stop)
        self.cwd = Path.cwd()
        self.addCleanup(os.chdir, self.cwd)
        os.chdir(self.root)
        self.source = self.root / "scene.py"
        self.source.write_text("from manimlib import Scene\nclass Hello(Scene):\n    pass\n")

    def invoke(self, *arguments):
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = console.try_render_cli(self.native, list(arguments))
        text = out.getvalue()
        report = json.loads(text) if text.startswith('{') else None
        return code, report, text, err.getvalue()

    def render(self, *extra):
        return self.invoke("--robot", str(self.source), "--resolution", "96x54", "--fps", "8", "--threads", "4", *extra)

    def test_audio_decoder_path_reaches_wav_writer(self):
        code, report, _, _ = self.render("--format", "wav", "--ffmpeg_bin", "tools/decoder with spaces")
        self.assertEqual(code, 0)
        self.assertEqual(self.native.instances[0].file_writer.ffmpeg_bin,
                         str(self.root / "tools/decoder with spaces"))
        self.assertEqual(report["audio_inputs"], [])
        self.assertEqual(report["ffmpeg_invocations"], [{"fixture": True}])

    def test_video_codec_and_pixel_format_are_not_audio_decode_options(self):
        for option, value in (("--vcodec", "aac"), ("--pix_fmt", "rgba")):
            with self.subTest(option=option):
                code, report, _, _ = self.render("--format", "wav", option, value)
                self.assertEqual(code, 2)
                self.assertIn("require mp4 or mov", report["message"])
        self.assertEqual(self.native.instances, [])

    def test_single_uses_actual_session_and_native_finish_signature(self):
        code, report, _, _ = self.render()
        self.assertEqual(code, 0)
        self.assertEqual(report["kind"], "render")
        self.assertEqual(report["resolution"], [96, 54])
        self.assertEqual(report["fps"], 8)
        self.assertEqual(report["threads"], 4)
        scene = self.native.instances[0]
        self.assertEqual(scene.events, ["begin", "run", "tear_down", "finish"])
        self.assertNotIn("_fmn_owned_render_session", vars(scene))
        self.assertTrue(scene.published)

    def test_robot_stdout_contains_only_terminal_json(self):
        self.source.write_text("from manimlib import Scene\nprint('module output')\nclass Hello(Scene):\n    def construct(self):\n        print('scene output')\n    def tear_down(self):\n        print('teardown output')\n")
        code, report, out, err = self.render()
        self.assertEqual(code, 0)
        self.assertEqual(len(out.splitlines()), 1)
        self.assertTrue(report["rendered"])
        for text in ('module output', 'scene output', 'teardown output'):
            self.assertIn(text, err)
            self.assertNotIn(text, out)

    def test_end_scene_is_normal_completion(self):
        self.source.write_text("from manimlib import Scene, EndScene\nclass Hello(Scene):\n    def construct(self):\n        raise EndScene('done')\n")
        self.assertEqual(self.render()[0], 0)
        self.assertTrue(self.native.instances[0].published)

    def test_keyboard_interrupt_cancels_owned_generation(self):
        self.source.write_text("from manimlib import Scene\nclass Hello(Scene):\n    def construct(self):\n        raise KeyboardInterrupt()\n")
        code, report, _, _ = self.render()
        self.assertEqual(code, 130)
        self.assertEqual(report["phase"], "execute")
        scene = self.native.instances[0]
        self.assertEqual(scene.events.count("abort"), 1)
        self.assertFalse(scene.published)
        self.assertNotIn("_fmn_owned_render_session", vars(scene))

    def test_system_exit_cancels_and_never_reports_success(self):
        self.source.write_text("from manimlib import Scene\nclass Hello(Scene):\n    def construct(self):\n        raise SystemExit(0)\n")
        code, report, _, _ = self.render()
        self.assertEqual(code, 5)
        self.assertEqual(report["kind"], "render-interrupted")
        self.assertEqual(self.native.instances[0].events.count("abort"), 1)

    def test_execution_failure_cancels(self):
        self.source.write_text("from manimlib import Scene\nclass Hello(Scene):\n    def construct(self):\n        raise ValueError('authored failure')\n")
        code, report, _, _ = self.render()
        self.assertEqual(code, 5)
        self.assertIn("authored failure", report["message"])
        self.assertFalse(report["artifact_published"])
        self.assertEqual(self.native.instances[0].events[-1], "abort")

    def test_finish_failure_cancels(self):
        self.native.Scene.finish_error = RuntimeError("reel: finish failed")
        code, report, _, _ = self.render()
        self.assertEqual(code, 6)
        self.assertEqual(report["kind"], "render-finish-failed")
        self.assertEqual(self.native.instances[0].events.count("abort"), 1)

    def test_start_failure_does_not_cancel_unowned_generation(self):
        self.native.Scene.start_error = RuntimeError("other owner")
        code, report, _, _ = self.render()
        self.assertEqual(code, 6)
        self.assertEqual(report["phase"], "start")
        self.assertEqual(self.native.instances[0].events, ["begin"])

    def test_borrowed_session_owner_is_preserved(self):
        self.source.write_text("from manimlib import Scene\nclass Hello(Scene):\n    def __init__(self):\n        super().__init__()\n        self._fmn_owned_render_session = 'external'\n")
        self.assertEqual(self.render()[0], 6)
        scene = self.native.instances[0]
        self.assertEqual(scene._fmn_owned_render_session, 'external')
        self.assertEqual(scene.events, [])

    def test_constructor_failure_never_begins_output(self):
        self.source.write_text("from manimlib import Scene\nclass Hello(Scene):\n    def __init__(self):\n        super().__init__()\n        raise ValueError('constructor failed')\n")
        code, report, _, _ = self.render()
        self.assertEqual(code, 5)
        self.assertEqual(report["phase"], "construct")
        self.assertEqual(self.native.instances[0].events, [])

    def test_sibling_imports_available_at_load_and_lazy_construct(self):
        (self.root / "fmn_load_helper.py").write_text("value = 4\n")
        (self.root / "fmn_lazy_helper.py").write_text("value = 7\n")
        self.source.write_text("from manimlib import Scene\nimport fmn_load_helper\nclass Hello(Scene):\n    def construct(self):\n        import fmn_lazy_helper\n        assert fmn_load_helper.value + fmn_lazy_helper.value == 11\n")
        before = list(sys.path)
        self.assertEqual(self.render()[0], 0)
        self.assertEqual(sys.path, before)

    def test_source_cwd_change_does_not_move_output(self):
        elsewhere = self.root / "elsewhere"
        elsewhere.mkdir()
        self.source.write_text(f"from manimlib import Scene\nimport os\nos.chdir({str(elsewhere)!r})\nclass Hello(Scene):\n    pass\n")
        code, report, _, _ = self.render("--format", "png", "--video_dir", "chosen.png")
        self.assertEqual(code, 0)
        self.assertEqual(Path(report["destination"]), self.root / "chosen.png")

    def test_unknown_writer_options_fail_before_native_start(self):
        self.native.Scene.writer_options = {"gamma": 2.0}
        code, report, _, _ = self.render()
        self.assertEqual(code, 4)
        self.assertIn("gamma", report["message"])
        self.assertEqual(self.native.instances[0].events, [])

    def test_repeated_robot_rejected_before_source_load(self):
        self.source.write_text("raise AssertionError('must not load')\n")
        self.assertEqual(self.render("--robot")[0], 2)
        self.assertEqual(self.native.instances, [])

    def test_invalid_options_refuse_before_source(self):
        self.source.write_text("raise AssertionError('must not load')\n")
        for options, expected in ((["--format", "invalid"], 4), (["--reproducible"], 4),
                                  (["--fps", "0"], 2), (["--fps", str(1 << 32)], 2),
                                  (["--threads"], 2), (["--video_dir=\0bad"], 2)):
            with self.subTest(options=options):
                self.assertEqual(self.render(*options)[0], expected)

    def test_missing_scene_rejected_before_constructor(self):
        code, report, _, _ = self.render("Absent")
        self.assertEqual(code, 5)
        self.assertIn("Absent", report["message"])
        self.assertEqual(self.native.instances, [])

    def test_multiple_discovered_without_selection_still_requires_choice(self):
        self.source.write_text("from manimlib import Scene\nclass A(Scene): pass\nclass B(Scene): pass\n")
        self.assertEqual(self.render()[0], 5)
        self.assertEqual(self.native.instances, [])

    def test_control_commands_delegated_without_loading(self):
        for args in ([], ["--robot"], ["--version"], ["--list-scenes", "a.py"],
                     ["--construct-only", "a.py"], ["studio", "a.py"], ["--audit-parity"]):
            with self.subTest(args=args):
                self.assertIsNone(self.invoke(*args)[0])
        self.assertEqual(self.native.instances, [])

    def test_option_values_are_not_controls_or_robot_switches(self):
        code, report, _, _ = self.render("--video_dir", "--version", "--format", "png")
        self.assertEqual(code, 0)
        self.assertEqual(Path(report["destination"]).name, "--version")
        code, _, out, _ = self.invoke(str(self.source), "--video_dir", "--robot", "--format", "png")
        self.assertEqual(code, 0)
        self.assertFalse(out.startswith('{'))

    def test_wav_receipt_retains_sample_units(self):
        code, report, _, _ = self.render("--format", "wav")
        self.assertEqual(code, 0)
        self.assertEqual(report["sample_frames"], report["frame_count"])
        self.assertEqual(report["sample_rate"], 48000)
        self.assertEqual(report["channels"], 2)
        self.assertFalse(report["certified"])

    def test_postpublication_provenance_failure_reports_existing_artifact(self):
        self.native.Scene.fail_provenance = True
        code, report, _, _ = self.render("--format", "mp4")
        self.assertEqual(code, 6)
        self.assertTrue(report["artifact_published"])
        self.assertTrue(Path(report["destination"]).exists())
        self.assertTrue(report["notes"])
        self.assertNotIn("abort", self.native.instances[0].events)

    def test_secondary_abort_error_does_not_replace_primary(self):
        self.native.Scene.abort_error = RuntimeError("secondary cancellation failure")
        self.source.write_text("from manimlib import Scene\nclass Hello(Scene):\n    def construct(self):\n        raise ValueError('primary failure')\n")
        code, report, _, _ = self.render()
        self.assertEqual(code, 5)
        self.assertIn("primary failure", report["message"])
        self.assertIn("secondary cancellation failure", " ".join(report["notes"]))

    def test_unprintable_exception_has_bounded_receipt(self):
        self.source.write_text("from manimlib import Scene\nclass BadError(Exception):\n    def __str__(self): raise RuntimeError('broken formatter')\nclass Hello(Scene):\n    def construct(self): raise BadError()\n")
        code, report, _, _ = self.render()
        self.assertEqual(code, 5)
        self.assertIn("could not be formatted", report["message"])

    def test_existing_output_is_not_clobbered(self):
        existing = self.root / "kept.png"
        existing.write_bytes(b"keep me")
        self.assertEqual(self.render("--format", "png", "--video_dir", str(existing))[0], 6)
        self.assertEqual(existing.read_bytes(), b"keep me")
        self.assertNotIn("abort", self.native.instances[0].events)

    def test_entrypoint_uses_session_instead_of_legacy_console_render(self):
        package._ensure_exclusive_manimlib_namespace = lambda: None
        package._ManimlibNamespaceCollision = type("Collision", (ImportError,), {})
        main = importlib.import_module(PACKAGE + ".__main__")
        self.native._native = self.native
        self.native._console_main = lambda: self.fail("legacy render path was called")
        with patch.object(sys, "argv", ["fmn-python", "--robot", str(self.source)]), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(main.main(), 0)
        self.assertTrue(self.native.instances[0].published)

    def test_entrypoint_does_not_steal_audit_spelling_from_output_value(self):
        package._ensure_exclusive_manimlib_namespace = lambda: None
        package._ManimlibNamespaceCollision = type("Collision", (ImportError,), {})
        main = importlib.import_module(PACKAGE + ".__main__")
        self.native._native = self.native
        self.native._console_main = lambda: self.fail("legacy path was called")
        argv = ["fmn-python", "--robot", str(self.source), "--video_dir", "--audit-parity", "--format", "png"]
        with patch.object(sys, "argv", argv), patch.object(main, "_emit_parity_audit", side_effect=AssertionError("audit was not requested")), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(main.main(), 0)


if __name__ == "__main__":
    unittest.main()
