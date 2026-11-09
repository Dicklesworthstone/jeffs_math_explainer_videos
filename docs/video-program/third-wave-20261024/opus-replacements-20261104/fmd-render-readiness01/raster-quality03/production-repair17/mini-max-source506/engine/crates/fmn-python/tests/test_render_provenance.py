"""Render/CLI ownership tests with an explicit fake native publication boundary.

Python source loading, RenderSession and console dispatch are production code.
The native renderer/parser/publisher are test doubles: these are not pixel,
Rust, installed-wheel or certification tests.
"""
from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
from types import ModuleType, SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python import rendering
from fmn_python.rendering import RenderSession
from fmn_python.scene_loading import SceneSource


class CapabilityError(RuntimeError):
    pass


class EndScene(Exception):
    pass


class FakeScene:
    def __init__(self):
        self.events = []
        self.random_seed = 0
        self.camera = SimpleNamespace(
            fps=30, get_pixel_shape=lambda: (8, 8),
            _core=SimpleNamespace(set_pixel_shape=lambda *args: None),
        )
        self._render_invocations = []
        self._render_audio_inputs = []
        self.finish_hook = lambda: None
        self.fail_finish = None
        self.fail_abort = None

    def _begin_native_output(self, destination, *args):
        self.destination = Path(destination)
        self.events.append("begin")

    def _abort_render(self):
        self.events.append("abort")
        if self.fail_abort:
            raise self.fail_abort

    def _finish_render(self):
        self.events.append("finish")
        if self.fail_finish:
            raise self.fail_finish
        data = b"explicit native boundary test double, not an image"
        with self.destination.open("xb") as stream:
            stream.write(data)
        self.finish_hook()
        return str(self.destination), 1, len(data), hashlib.sha256(data).hexdigest(), "test-double", 1

    def construct(self):
        pass

    def tear_down(self):
        pass

    def run(self):
        try:
            self.construct()
        finally:
            self.tear_down()


class RenderProvenanceTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.native = ModuleType("manimlib")
        self.native.Scene = FakeScene
        self.native.EndScene = EndScene
        self.native._CapabilityError = CapabilityError
        self.native._portal_publish_manifest = self.publish
        self.published_sources = []
        self.published_cues = []
        self.identities = {"cpython": "test", "abi": "test", "wheel": "test", "numpy": "test"}
        self.publish_error = None
        self.publish_hook = lambda: None

    def publish(self, destination, fmt, resolution, fps, threads, seed, report, sources, identities, cues):
        self.publish_hook()
        if self.publish_error:
            raise self.publish_error
        if not isinstance(sources, dict):
            raise TypeError("test boundary expects materialized source bytes")
        self.published_sources.append(copy.deepcopy(sources))
        self.published_cues.append(copy.deepcopy(cues))
        self.observed_identities = dict(identities)
        sidecar = Path(str(destination) + ".manifest")
        with sidecar.open("x") as stream:
            json.dump({"sources": sorted(sources)}, stream)
        return str(sidecar), "f" * 64

    def session(self, *, scene=None, **kwargs):
        options = dict(reproducible=True, sources={"scene.py": b"VALUE = 1\n"},
                       runtime_identities=self.identities, _native=self.native, threads=1)
        options.update(kwargs)
        return RenderSession(scene or FakeScene(), self.root / ("out." + options.get("format", "png")), **options)

    def write(self, name, data):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return path

    def test_success_is_idempotent_and_reports_artifact(self):
        session = self.session()
        with session:
            self.assertFalse(session.artifact_published)
        self.assertTrue(session.artifact_published)
        self.assertIs(session.finish(), session.result)
        self.assertEqual(session.scene.events, ["begin", "finish"])
        self.assertTrue(session.result.manifest.is_file())
        self.assertIsNone(getattr(session.scene, "_fmn_owned_render_session", None))

    def test_manifest_failure_is_terminal_not_success_or_cancellation(self):
        error = OSError("manifest disk full")
        self.publish_error = error
        session = self.session()
        with self.assertRaises(OSError) as raised:
            with session:
                pass
        self.assertIs(raised.exception, error)
        self.assertTrue(session.artifact_published)
        self.assertIsNone(session.result)
        self.assertTrue(session.destination.is_file())
        self.assertIn("already published", " ".join(error.__notes__))
        with self.assertRaisesRegex(RuntimeError, "active"):
            session.finish()
        session.abort()
        self.assertEqual(session.scene.events, ["begin", "finish"])
        self.assertIsNone(getattr(session.scene, "_fmn_owned_render_session", None))

    def test_manifest_interrupt_preserves_published_status(self):
        self.publish_error = KeyboardInterrupt()
        session = self.session()
        with self.assertRaises(KeyboardInterrupt):
            with session:
                pass
        self.assertTrue(session.artifact_published)
        self.assertIsNone(session.result)
        self.assertNotIn("abort", session.scene.events)

    def test_missing_publisher_refuses_before_native_begin(self):
        self.native._portal_publish_manifest = None
        session = self.session()
        with self.assertRaises(CapabilityError):
            with session:
                self.fail("unreachable")
        self.assertEqual(session.scene.events, [])
        self.assertFalse(session.destination.exists())

    def test_missing_sources_refuses_before_native_begin(self):
        session = self.session(sources=None)
        with self.assertRaisesRegex(ValueError, "executed sources"):
            session.__enter__()
        self.assertEqual(session.scene.events, [])

    def test_direct_sources_and_identities_are_detached_from_caller(self):
        source = {"scene.py": b"VALUE = 1\n"}
        session = self.session(sources=source)
        with session:
            source["scene.py"] = b"VALUE = 9\n"
            self.identities["cpython"] = "changed"
        self.assertEqual(self.published_sources, [{"scene.py": b"VALUE = 1\n"}])
        self.assertEqual(self.observed_identities["cpython"], "test")

    def test_provider_runs_once_after_execution_before_native_finish(self):
        scene = FakeScene()
        def provide():
            self.assertEqual(scene.events, ["begin", "executed"])
            scene.events.append("snapshot")
            return {"scene.py": b"VALUE = 1\n"}
        session = self.session(scene=scene, sources=provide)
        with session:
            scene.events.append("executed")
        session.finish()
        self.assertEqual(scene.events, ["begin", "executed", "snapshot", "finish"])

    def test_provider_failure_cancels_before_publication(self):
        error = RuntimeError("mixed source versions")
        def provide():
            raise error
        session = self.session(sources=provide)
        with self.assertRaises(RuntimeError) as raised:
            with session:
                pass
        self.assertIs(raised.exception, error)
        self.assertFalse(session.artifact_published)
        self.assertFalse(session.destination.exists())
        self.assertEqual(session.scene.events, ["begin", "abort"])
        self.assertIsNone(session.result)

    def test_cleanup_failure_does_not_replace_provider_failure(self):
        session = self.session(sources=lambda: {})
        session.scene.fail_abort = OSError("abort failed")
        with self.assertRaises(ValueError) as raised:
            with session:
                pass
        self.assertIn("abort failed", " ".join(raised.exception.__notes__))
        self.assertIsNone(getattr(session.scene, "_fmn_owned_render_session", None))

    def test_provider_cannot_cancel_then_publish(self):
        def provide():
            session.abort()
            return {"scene.py": b"pass"}
        session = self.session(sources=provide)
        with self.assertRaisesRegex(RuntimeError, "cancelled"):
            with session:
                pass
        self.assertEqual(session.scene.events, ["begin", "abort"])
        self.assertFalse(session.artifact_published)

    def test_provider_cannot_reenter_finish(self):
        def provide():
            session.finish()
        session = self.session(sources=provide)
        with self.assertRaisesRegex(RuntimeError, "already in progress"):
            with session:
                pass
        self.assertEqual(session.scene.events, ["begin", "abort"])

    def test_manifest_callback_cannot_reenter_finish_or_release_owner(self):
        session = self.session()
        def hook():
            self.assertIs(session.scene._fmn_owned_render_session, session)
            with self.assertRaisesRegex(RuntimeError, "already in progress"):
                session.finish()
        self.publish_hook = hook
        with session:
            pass
        self.assertTrue(session.artifact_published)

    def test_native_finish_failure_still_cancels(self):
        session = self.session()
        session.scene.fail_finish = OSError("native finish failed")
        with self.assertRaisesRegex(OSError, "native finish failed"):
            with session:
                pass
        self.assertFalse(session.artifact_published)
        self.assertIsNone(session.result)
        self.assertEqual(session.scene.events, ["begin", "finish", "abort"])

    def test_media_receipt_failure_never_installs_partial_result(self):
        session = self.session(format="mp4", reproducible=False)
        session.scene._render_invocations = None
        with self.assertRaises(TypeError):
            with session:
                pass
        self.assertTrue(session.artifact_published)
        self.assertIsNone(session.result)
        with self.assertRaises(RuntimeError):
            session.finish()
        self.assertEqual(session.scene.events, ["begin", "finish"])

    def test_standard_render_never_evaluates_source_provider(self):
        def provide():
            raise AssertionError("ordinary render must not require provenance")
        self.native._portal_publish_manifest = None
        session = self.session(reproducible=False, sources=provide)
        with session:
            pass
        self.assertFalse(session.result.certified)
        self.assertEqual(self.published_sources, [])

    def test_invalid_source_names_and_values(self):
        for name in ("", "/tmp/scene.py", "../scene.py", "a/../b", "a//b", "./b", "a\\b", "C:scene", "a\0b"):
            with self.subTest(name=name), self.assertRaises(ValueError):
                self.session(sources={name: b"pass"})
        for value in ("pass", bytearray(b"pass"), None):
            with self.subTest(value=value), self.assertRaises(TypeError):
                self.session(sources={"scene.py": value})

    def test_source_budget_and_exact_boundary(self):
        with patch.object(rendering, "_MAX_PROVENANCE_BYTES", 4):
            self.assertEqual(rendering._source_snapshot({"scene.py": b"pass"}), {"scene.py": b"pass"})
            with self.assertRaisesRegex(ValueError, "budget"):
                rendering._source_snapshot({"scene.py": b"pass!"})
        with patch.object(rendering, "_MAX_PROVENANCE_INPUTS", 1):
            with self.assertRaises(ValueError):
                rendering._source_snapshot({"a.py": b"", "b.py": b""})

    def test_lazy_construct_and_teardown_imports_reach_manifest(self):
        data = (b"from manimlib import Scene\nclass CapturedScene(Scene):\n"
                b"    def construct(self):\n        import render_capture_construct\n"
                b"    def tear_down(self):\n        import render_capture_teardown\n")
        path = self.write("scene.py", data)
        helper = self.write("render_capture_construct.py", b"VALUE = 42\n")
        self.write("render_capture_teardown.py", b"VALUE = 7\n")
        with patch.dict(sys.modules, {"manimlib": self.native}):
            with SceneSource(path, FakeScene) as loaded:
                cls = loaded.scenes["CapturedScene"]
                scene = cls()
                scene.finish_hook = lambda: helper.write_bytes(b"VALUE = 99\n")
                result = rendering.render_scene(scene, self.root / "out.png", reproducible=True,
                                                sources=lambda: loaded.sources,
                                                runtime_identities=self.identities)
        self.assertTrue(result.certified)
        self.assertEqual(self.published_sources, [{
            "scene.py": data, "render_capture_construct.py": b"VALUE = 42\n",
            "render_capture_teardown.py": b"VALUE = 7\n",
        }])

    def test_verified_audio_has_collision_free_names_and_deduplication(self):
        a = self.write("a/tone.wav", b"first")
        b = self.write("b/tone.wav", b"second")
        def receipt(path):
            return {"path": str(path), "source_sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
        facts = [receipt(a), receipt(b), receipt(a)]
        session = self.session(format="wav")
        session.scene._render_audio_inputs = facts
        with session:
            pass
        cues = self.published_cues[0]
        self.assertEqual(len(cues), 2)
        self.assertEqual({value for _, value in cues}, {b"first", b"second"})
        self.assertEqual(len({name for name, _ in cues}), 2)
        self.assertEqual(session.result.audio_inputs, tuple(facts))

    def test_changed_or_missing_audio_never_publishes_manifest(self):
        for missing in (False, True):
            with self.subTest(missing=missing):
                path = self.write("tone.wav", b"decoded bytes")
                session = self.session(format="wav")
                session.scene._render_audio_inputs = [{
                    "path": str(path), "source_sha256": hashlib.sha256(b"decoded bytes").hexdigest(),
                }]
                session.destination = self.root / ("missing.wav" if missing else "changed.wav")
                session.scene.finish_hook = path.unlink if missing else lambda: path.write_bytes(b"new bytes")
                with self.assertRaises((RuntimeError, FileNotFoundError)):
                    with session:
                        pass
                self.assertTrue(session.artifact_published)
                self.assertIsNone(session.result)
                self.assertEqual(self.published_sources, [])

    def test_audio_byte_budget_is_bounded(self):
        path = self.write("tone.wav", b"12345")
        facts = [{"path": str(path), "source_sha256": hashlib.sha256(b"12345").hexdigest()}]
        with patch.object(rendering, "_MAX_PROVENANCE_BYTES", 4):
            with self.assertRaisesRegex(ValueError, "byte budget"):
                rendering._cue_assets(facts)

    def test_sidecar_preflight_preserves_existing_generation(self):
        session = self.session()
        sidecar = Path(str(session.destination) + ".manifest")
        sidecar.write_bytes(b"existing")
        with self.assertRaises(FileExistsError):
            session.__enter__()
        self.assertEqual(session.scene.events, [])
        self.assertEqual(sidecar.read_bytes(), b"existing")


class ConsoleProvenanceTests(unittest.TestCase):
    """Exercise real single-scene CLI dispatch; batch-only imports are inert."""

    publish = RenderProvenanceTests.publish
    write = RenderProvenanceTests.write

    def setUp(self):
        RenderProvenanceTests.setUp(self)
        # Deliberately separate parser/publication doubles from the production
        # console, source loader and session under test. No batch is executed.
        batch_cli = ModuleType("fmn_python.batch_cli")
        batch_cli._BATCH_HELP = ""
        batch_cli._VALUE_FLAGS = {"--format", "--video_dir", "--resolution", "--fps", "--threads"}
        batch_cli._emit_result = lambda *args, **kwargs: self.fail("unexpected batch")
        checkpoint = ModuleType("fmn_python.checkpoint_cli")
        checkpoint.CHECKPOINT_HELP = ""
        checkpoint.CHECKPOINT_VALUES = set()
        checkpoint.take_checkpoint_options = lambda options, flags: (options, {})
        batch = ModuleType("fmn_python.batch_rendering")
        batch.BatchRenderError = type("BatchRenderError", (Exception,), {})
        batch.BatchRenderResult = type("BatchRenderResult", (), {})
        batch._error_fields = lambda error: (type(error).__name__, str(error))
        batch._error_notes = lambda error: tuple(getattr(error, "__notes__", ()))
        batch._name_key = str.casefold
        batch.render_scenes = batch_cli._emit_result
        path = Path(rendering.__file__).with_name("console_rendering.py")
        spec = importlib.util.spec_from_file_location("fmn_python._capture_console_test", path)
        self.console = importlib.util.module_from_spec(spec)
        with patch.dict(sys.modules, {
            "fmn_python.batch_cli": batch_cli, "fmn_python.checkpoint_cli": checkpoint,
            "fmn_python.batch_rendering": batch,
        }):
            spec.loader.exec_module(self.console)
        self.emitted = []
        def emit(code, identity, kind, message, robot, **details):
            self.emitted.append((code, identity, kind, details))
            return code
        self.native._portal_cli_emit = emit
        self.native._portal_cli_render_arguments = lambda args: (
            [args[0]], {"format": "png", "video_dir": str(self.root / "out.png"),
                        "reproducible": "--reproducible" in args}, 8, 8, 30, 1,
        )

    def run_cli(self, data, certified=True):
        path = self.write("scene.py", data)
        args = [str(path), "--robot"] + (["--reproducible"] if certified else [])
        with patch.dict(sys.modules, {"manimlib": self.native}):
            return self.console.try_render_cli(self.native, args)

    def test_cli_lazy_imports_reach_manifest(self):
        self.write("console_capture_helper.py", b"VALUE = 8\n")
        data = (b"from manimlib import Scene\nclass Demo(Scene):\n"
                b"    def construct(self):\n        import console_capture_helper\n")
        self.assertEqual(self.run_cli(data), 0)
        self.assertEqual(self.published_sources[0]["console_capture_helper.py"], b"VALUE = 8\n")
        self.assertTrue(self.emitted[-1][3]["certified"])

    def test_cli_reports_published_artifact_on_manifest_failure(self):
        self.publish_error = OSError("manifest publication refused")
        self.assertEqual(self.run_cli(b"from manimlib import Scene\nclass Demo(Scene): pass\n"), 6)
        code, identity, kind, details = self.emitted[-1]
        self.assertEqual(kind, "render-finish-failed")
        self.assertTrue(details["artifact_published"])
        self.assertTrue((self.root / "out.png").is_file())

    def test_cli_source_capture_error_cancels_before_publication(self):
        from fmn_python import scene_loading
        with patch.object(scene_loading, "_MAX_CAPTURED_SOURCES", 0):
            code = self.run_cli(b"from manimlib import Scene\nclass Demo(Scene): pass\n")
        self.assertEqual(code, 6)
        self.assertFalse(self.emitted[-1][3]["artifact_published"])
        self.assertFalse((self.root / "out.png").exists())

    def test_cli_standard_render_does_not_require_source_capture(self):
        from fmn_python import scene_loading
        with patch.object(scene_loading, "_MAX_CAPTURED_SOURCES", 0):
            code = self.run_cli(b"from manimlib import Scene\nclass Demo(Scene): pass\n", certified=False)
        self.assertEqual(code, 0)
        self.assertFalse(self.emitted[-1][3]["certified"])


if __name__ == "__main__":
    unittest.main()
