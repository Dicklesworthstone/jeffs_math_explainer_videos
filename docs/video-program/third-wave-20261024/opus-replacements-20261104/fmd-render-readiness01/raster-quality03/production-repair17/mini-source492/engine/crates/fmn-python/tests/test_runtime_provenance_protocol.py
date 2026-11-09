"""Runtime guards execute real RenderSession code with an explicit native sink double.

The production class and source freezer are AST-selected from rendering.py.
Hashes use actual temporary file bytes. This is not native rendering evidence.
"""
from __future__ import annotations

import ast
from collections.abc import Mapping
import copy
import importlib.util
import os
from pathlib import Path
import sys
import tempfile
import threading
import types
import unittest
from unittest.mock import patch

_ROOT = Path(__file__).resolve().parents[1] / "python/fmn_python"
_PACKAGE = "_fmn_runtime_provenance_tests"
package = types.ModuleType(_PACKAGE)
package.__path__ = [str(_ROOT)]
sys.modules[_PACKAGE] = package


def load(name):
    spec = importlib.util.spec_from_file_location(_PACKAGE + "." + name, _ROOT / (name + ".py"))
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


identity = load("runtime_identity")
adapter = load("runtime_provenance")
_TEMP = Path(tempfile.mkdtemp(prefix="fmn-runtime-provenance-tests-"))
_LEGACY = {"cpython": "CPython 3.13", "abi": "cpython-3.13-full-abi", "wheel": "franken-manim test", "numpy": "NumPy test"}


def production_session():
    source = ast.parse((_ROOT / "rendering.py").read_text())
    nodes = [node for node in source.body if isinstance(node, (ast.ClassDef, ast.FunctionDef))
             and node.name in {"RenderSession", "_source_snapshot",
                               "_configured_scene_session", "_run_owned_scene_render"}]
    if len(nodes) != 4:
        raise AssertionError("expected the production session, freezer and Scene routing helpers")
    module = types.ModuleType(_PACKAGE + ".rendering")
    module.__dict__.update(
        Mapping=Mapping, Path=Path, os=os, sys=sys, threading=threading, copy=copy,
        _MAX_PROVENANCE_INPUTS=4096, _MAX_PROVENANCE_BYTES=64 * 1024 * 1024,
        _FORMATS={"png", "png_sequence", "gif", "mp4", "wav"}, _VIDEO_FORMATS={"mp4", "mov"},
        _positive_integer=lambda value, name: value, _seed=lambda value: value,
        _animation_range=lambda value: value, apply_animation_range=lambda *args: None,
        _validate_writer_options=lambda *args, **kwargs: None, _cue_assets=lambda values: None,
        _runtime_identities=lambda native: dict(_LEGACY), RenderResult=types.SimpleNamespace,
        _configured_destination=lambda scene, destination, format, native: (destination, format),
    )
    nodes.insert(0, ast.ImportFrom(module="__future__", names=[ast.alias(name="annotations")], level=0))
    exec(compile(ast.fix_missing_locations(ast.Module(body=nodes, type_ignores=[])),
                 str(_ROOT / "rendering.py"), "exec"), vars(module))
    sys.modules[module.__name__] = module
    package.rendering = module
    return module


class RuntimeGuardTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(dir=_TEMP))
        self.payload = self.root / "runtime.bin"
        self.payload.write_bytes(b"initial runtime")
        groups = {role: (role + " test", [("payload", self.payload)])
                  for role in ("cpython", "wheel", "numpy")}
        self.capture_count = 0
        def capture(native):
            self.capture_count += 1
            return identity.RuntimeSnapshot(lambda: (groups, {"version": 1}))
        self.capture = patch.object(adapter, "capture_runtime", side_effect=capture)
        self.capture.start()
        self.addCleanup(self.capture.stop)
        self.rendering = production_session()
        events = self.events = []
        class EndScene(Exception):
            pass
        class Scene:
            random_seed = 0
            def __init__(self):
                self.camera = types.SimpleNamespace(
                    fps=4, get_pixel_shape=lambda: (32, 18),
                    _core=types.SimpleNamespace(set_pixel_shape=lambda *args: None),
                )
                self._render_audio_inputs, self._render_invocations = (), ()
                self.finalize_hook = lambda: None
                self.run_hook = lambda: None
            def _begin_native_output(self, destination, *options):
                events.append("begin")
                self.destination = Path(destination)
            def _finish_render(self):
                events.append("finish")
                self.finalize_hook()
                with self.destination.open("xb") as stream:
                    stream.write(b"explicit native-sink fixture, not rendered pixels")
                return str(self.destination), 1, 48, "digest", "certified-cpu", 1
            def _abort_render(self):
                events.append("abort")
            def run(self):
                events.append("run")
                self.run_hook()
        def manifest(*args):
            events.append(("manifest", copy.deepcopy(args[8])))
            return str(self.root / "manifest.fmnp"), "closure"
        self.native = types.SimpleNamespace(Scene=Scene, EndScene=EndScene, _portal_publish_manifest=manifest,
                                            _CapabilityError=RuntimeError)
        adapter.install_runtime_provenance(self.native)
        self.scene = Scene()

    def session(self, **kwargs):
        options = {"format": "png", "reproducible": True, "sources": {"scene.py": b"pass"}}
        options.update(kwargs)
        return self.rendering.RenderSession(self.scene, self.root / "output.png", _native=self.native, **options)

    def test_configured_subdivision_cannot_bypass_certified_runtime_admission(self):
        self.scene.file_writer = types.SimpleNamespace(subdivide_output=True)
        with self.assertRaisesRegex(RuntimeError, "subdivided.*certify"):
            self.scene.render_session(self.root / "clips", format="gif", reproducible=True,
                                      sources={"scene.py": b"pass"})
        self.assertEqual(self.capture_count, 0)
        self.assertEqual(self.events, [])
        self.assertFalse((self.root / "clips").exists())

    def test_standard_render_has_no_hash_cost_or_runtime_requirement(self):
        with self.session(reproducible=False) as session:
            pass
        self.assertEqual(self.capture_count, 0)
        self.assertFalse(session.result.certified)
        self.assertEqual(self.events, ["begin", "finish"])

    def test_legacy_labels_are_replaced_by_measured_inputs(self):
        with self.session(runtime_identities=_LEGACY) as session:
            expected = session._fmn_runtime_snapshot.identities
        self.assertEqual(self.events[-1], ("manifest", expected))
        self.assertNotEqual(expected, _LEGACY)
        self.assertTrue(session.result.certified)
        self.assertEqual(session.result.closure_digest, "closure")

    def test_arbitrary_caller_labels_refuse_before_acquisition(self):
        with self.assertRaisesRegex(RuntimeError, "supplied runtime identities"):
            self.session(runtime_identities={"wheel": "pretend"}).__enter__()
        self.assertEqual(self.events, [])

    def test_unknown_runtime_refuses_before_acquisition(self):
        with patch.object(adapter, "capture_runtime", side_effect=identity.RuntimeIdentityError("missing wheel")):
            with self.assertRaisesRegex(RuntimeError, "CAPABILITY.*missing wheel"):
                self.session().__enter__()
        self.assertEqual(self.events, [])

    def test_runtime_change_aborts_without_artifact_or_sidecar(self):
        session = self.session()
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "changed during rendering"):
            with session:
                self.payload.write_bytes(b"changed runtime")
        self.assertEqual(self.events, ["begin", "abort"])
        self.assertFalse(session.artifact_published)
        self.assertIsNone(session.result)
        self.assertFalse((self.root / "output.png").exists())
        self.assertNotIn("_fmn_owned_render_session", vars(self.scene))

    def test_source_provider_is_evaluated_once_before_verification(self):
        calls = []
        def sources():
            calls.append("source")
            self.payload.write_bytes(b"provider changed runtime")
            return {"scene.py": b"pass"}
        with self.assertRaises(identity.RuntimeIdentityError):
            with self.session(sources=sources):
                pass
        self.assertEqual(calls, ["source"])
        self.assertEqual(self.events, ["begin", "abort"])

    def test_mapping_side_effect_is_checked_after_freezing(self):
        class Evil(dict):
            def items(inner):
                self.payload.write_bytes(b"mapping changed runtime")
                return super().items()
        with self.assertRaises(identity.RuntimeIdentityError):
            with self.session(sources=lambda: Evil({"scene.py": b"pass"})):
                pass
        self.assertEqual(self.events, ["begin", "abort"])

    def test_late_final_frame_change_retains_honest_partial_publication(self):
        self.scene.finalize_hook = lambda: self.payload.write_bytes(b"late camera hook mutation")
        session = self.session()
        with self.assertRaises(identity.RuntimeIdentityError) as caught:
            with session:
                pass
        self.assertTrue(session.artifact_published)
        self.assertTrue((self.root / "output.png").exists())
        self.assertIsNone(session.result)
        self.assertEqual(self.events, ["begin", "finish"])
        self.assertTrue(any("already published" in note for note in caught.exception.__notes__))
        self.assertNotIn("_fmn_owned_render_session", vars(self.scene))

    def test_late_provenance_replacement_is_not_published(self):
        session = self.session()
        self.scene.finalize_hook = lambda: session.runtime_identities.update(wheel="forged")
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "provenance was changed"):
            with session:
                pass
        self.assertTrue(session.artifact_published)
        self.assertEqual(self.events, ["begin", "finish"])

    def test_original_callback_failure_is_preserved(self):
        failure = ValueError("authored scene failure")
        session = self.session()
        with self.assertRaises(ValueError) as caught:
            with session:
                raise failure
        self.assertIs(caught.exception, failure)
        self.assertEqual(self.events, ["begin", "abort"])

    def test_reentrant_finish_and_provider_cancellation_do_not_publish(self):
        session = self.session()
        session.sources = lambda: session.finish()
        with self.assertRaisesRegex(RuntimeError, "already in progress"):
            with session:
                pass
        self.assertEqual(self.events, ["begin", "abort"])
        self.events.clear()
        session = self.session()
        def sources():
            session.abort()
            return {"scene.py": b"pass"}
        session.sources = sources
        with self.assertRaisesRegex(RuntimeError, "cancelled by its source provider"):
            with session:
                pass
        self.assertEqual(self.events, ["begin", "abort"])

    def test_finished_result_is_idempotent_without_rehash(self):
        with self.session() as session:
            pass
        result, events = session.result, list(self.events)
        self.payload.write_bytes(b"changed after completed publication")
        self.assertIs(session.finish(), result)
        self.assertEqual(self.events, events)

    def test_wrong_thread_is_rejected_before_hashing(self):
        session = self.session()
        errors = []
        def work():
            try:
                session.__enter__()
            except RuntimeError as error:
                errors.append(str(error))
        thread = threading.Thread(target=work)
        thread.start(); thread.join()
        self.assertTrue(errors and "creating thread" in errors[0])
        self.assertEqual(self.capture_count, 0)

    def test_existing_generation_is_not_rehashed_or_cancelled(self):
        with self.session() as first:
            with self.assertRaisesRegex(RuntimeError, "already has an owned"):
                self.session().__enter__()
            self.assertEqual(self.capture_count, 1)
            self.assertNotIn("abort", self.events)
        self.assertIsNotNone(first.result)

    def test_scene_front_doors_forward_provenance_and_keep_aliases(self):
        alias = self.native.Scene
        result = self.scene.render(self.root / "render.png", reproducible=True,
                                   sources={"scene.py": b"pass"}, runtime_identities=_LEGACY)
        self.assertTrue(result.certified)
        self.assertIs(self.scene.render_result, result)
        self.assertIs(alias, self.native.Scene)
        self.assertEqual(self.events[:3], ["begin", "run", "finish"])
        self.events.clear()
        with self.scene.render_session(self.root / "session.png", reproducible=True,
                                       sources={"scene.py": b"pass"}) as session:
            pass
        self.assertTrue(session.result.certified)
        self.assertEqual(self.events[:2], ["begin", "finish"])

    def test_end_scene_is_success_and_install_is_idempotent(self):
        method, enter = self.native.Scene.render, self.rendering.RenderSession.__enter__
        adapter.install_runtime_provenance(self.native)
        self.assertIs(method, self.native.Scene.render)
        self.assertIs(enter, self.rendering.RenderSession.__enter__)
        def end():
            raise self.native.EndScene()
        self.scene.run_hook = end
        result = self.scene.render(self.root / "end.png", reproducible=True, sources={"scene.py": b"pass"})
        self.assertTrue(result.certified)


if __name__ == "__main__":
    print("retaining native-sink protocol fixture inputs:", _TEMP)
    unittest.main()
