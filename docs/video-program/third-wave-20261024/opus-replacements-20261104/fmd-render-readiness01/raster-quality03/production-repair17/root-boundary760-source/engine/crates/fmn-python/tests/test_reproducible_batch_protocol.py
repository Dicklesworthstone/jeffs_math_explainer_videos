"""Production batch orchestration with explicit native-render/runtime doubles.

RenderResult, input validation and source freezing are AST-selected from the
actual rendering module. The sink below writes witness files, NOT PNG/WAV or
native manifests; these tests make no renderer/certification claim. Installed
native acceptance is separately registered in reproducible_batch.py.
"""
from __future__ import annotations

import ast
from dataclasses import replace
import hashlib
import importlib.util
from pathlib import Path
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

_PYTHON = Path(__file__).resolve().parents[1] / "python/fmn_python"


def _load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def load_batch():
    """Isolate module globals; do not replace any installed native classes."""
    name = "_fmn_batch_protocol"
    package = types.ModuleType(name)
    package.__path__ = [str(_PYTHON)]
    sys.modules[name] = package
    rendering = types.ModuleType(name + ".rendering")
    sys.modules[rendering.__name__] = rendering
    package.rendering = rendering
    selected = {"RenderResult", "_source_snapshot", "_positive_integer", "_apply_output_options"}
    constants = {"_FORMATS", "SourceInputs", "_MAX_PROVENANCE_INPUTS", "_MAX_PROVENANCE_BYTES"}
    path = _PYTHON / "rendering.py"
    nodes = []
    for node in ast.parse(path.read_text()).body:
        if isinstance(node, (ast.Import, ast.ImportFrom)):
            if not isinstance(node, ast.ImportFrom) or node.level == 0:
                nodes.append(node)
        elif isinstance(node, (ast.FunctionDef, ast.ClassDef)) and node.name in selected:
            nodes.append(node)
        elif isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id in constants for t in node.targets):
            nodes.append(node)
    exec(compile(ast.Module(body=nodes, type_ignores=[]), str(path), "exec"), vars(rendering))
    rendering.render_scene = None  # Explicit native-output boundary double.
    rendering.RenderSession = type("RenderSession", (), {})
    rendering._runtime_identities = lambda native: {"wheel": "legacy version"}
    checkpoint = types.ModuleType(name + ".batch_checkpoint")
    def no_checkpoint(*args, **kwargs):
        raise AssertionError("certified tests must not acquire a checkpoint lock")
    checkpoint.BatchCheckpoint = no_checkpoint
    sys.modules[checkpoint.__name__] = checkpoint
    runtime = types.ModuleType(name + ".runtime_identity")
    class RuntimeIdentityError(RuntimeError):
        pass
    runtime.RuntimeIdentityError = RuntimeIdentityError
    sys.modules[runtime.__name__] = runtime
    # Range validation is unrelated to native storage; load the real module.
    _load(name + ".render_selection", _PYTHON / "render_selection.py")
    provenance = _load(name + ".batch_provenance", _PYTHON / "batch_provenance.py")
    batch = _load(name + ".batch_rendering", _PYTHON / "batch_rendering.py")
    return batch, provenance, rendering, runtime


class ReproducibleBatchTests(unittest.TestCase):
    def setUp(self):
        self.modules = patch.dict(sys.modules)
        self.modules.start()
        self.addCleanup(self.modules.stop)
        self.batch, self.provenance, self.rendering, self.runtime = load_batch()
        # Tests intentionally retain witness files; no repository/user files
        # are deleted, even by test cleanup.
        self.root = Path(tempfile.mkdtemp(prefix="fmn-batch-protocol-"))
        self.payload = self.root / "runtime"
        self.payload.write_bytes(b"runtime generation one")
        self.events, self.calls, self.sources_seen, self.snapshots = [], [], [], []
        outer = self
        class Scene:
            def __init__(self, value=0):
                self.value = value
                outer.events.append(("construct", type(self).__name__))
            def run(self):
                outer.events.append(("run", type(self).__name__))
        self.Scene = Scene
        self.native = types.ModuleType("manimlib")
        self.native.Scene = Scene
        self.native._portal_publish_manifest = lambda *args: None
        sys.modules["manimlib"] = self.native
        class Snapshot:
            def __init__(self):
                self.expected = outer.payload.read_bytes()
                self.identities = {"wheel": hashlib.sha256(self.expected).hexdigest()}
                self.verifications = 0
            def verify(self):
                self.verifications += 1
                if outer.payload.read_bytes() != self.expected:
                    raise outer.runtime.RuntimeIdentityError("runtime changed")
        def capture(native):
            snapshot = Snapshot()
            self.snapshots.append(snapshot)
            return snapshot
        self.runtime.capture_runtime = capture
        self.batch.render_scene = self.sink

    def scenes(self, *names):
        return {name: type(name, (self.Scene,), {}) for name in names or ("One", "Two")}

    def render(self, scenes=None, **kwargs):
        options = dict(format="png", reproducible=True, sources={"scene.py": b"scene"})
        options.update(kwargs)
        return self.batch.render_scenes(self.scenes() if scenes is None else scenes, self.root / "outputs", **options)

    def sink(self, scene, destination, **options):
        self.calls.append((scene, destination, options))
        if isinstance(scene, type):
            scene = scene(**(options["scene_kwargs"] or {}))
        scene.run()
        if options.get("reproducible"):
            provider = options["sources"]
            sources = provider() if callable(provider) else provider
            self.sources_seen.append(self.rendering._source_snapshot(sources))
        destination.parent.mkdir(parents=True, exist_ok=True)
        payload = (type(scene).__name__ + str(scene.value)).encode()
        if options["format"] == "png_sequence":
            destination.mkdir()
            (destination / "witness").write_bytes(payload)
        else:
            with destination.open("xb") as stream:
                stream.write(payload)
        manifest = None
        if options.get("reproducible"):
            sidecar = destination.with_name(destination.name + ".manifest")
            sidecar.mkdir()
            manifest = sidecar / "manifest.fmnp"
            manifest.write_bytes(b"explicit test double, not a native manifest")
        return self.rendering.RenderResult(
            destination=destination, format=options["format"], resolution=options["resolution"] or (32, 18),
            fps=options["fps"] or 30, threads=options["threads"] or 1, engine="test-double",
            bytes=len(payload), digest=hashlib.sha256(payload).hexdigest(),
            frame_count=None if options["format"] == "wav" else 1,
            sample_frames=1 if options["format"] == "wav" else None, seed=scene.value,
            certified=bool(options.get("reproducible")), manifest=manifest,
            closure_digest="a" * 64 if manifest else None,
            animation_range=options.get("animation_range"),
        )

    def test_success_has_separate_manifest_receipts_and_order(self):
        report = self.render(self.scenes("Second", "First"))
        self.assertTrue(report.ok and report.all_scenes_certified)
        data = report.as_dict()
        self.assertFalse(data["certified"], "the aggregate is not a certified artifact")
        self.assertTrue(data["reproducible_requested"] and data["all_scenes_certified"])
        self.assertEqual([item.name for item in report.outcomes], ["Second", "First"])
        self.assertEqual(self.events, [("construct", "Second"), ("run", "Second"), ("construct", "First"), ("run", "First")])
        self.assertNotEqual(report.outcomes[0].result.manifest, report.outcomes[1].result.manifest)
        self.assertEqual(len(self.snapshots), 1)
        self.assertEqual(self.snapshots[0].verifications, 2)

    def test_all_native_certified_formats_keep_destination_policy(self):
        for format in ("png", "png_sequence", "wav"):
            with self.subTest(format=format):
                report = self.render(self.scenes(format), format=format)
                result = report.outcomes[0].result
                self.assertTrue(report.all_scenes_certified)
                expected = (self.root / "outputs" / format / "frames" if format == "png_sequence"
                            else self.root / "outputs" / (format + "." + format))
                self.assertEqual(result.destination, expected)
                self.assertEqual(result.manifest.parent, expected.with_name(expected.name + ".manifest"))

    def test_standard_mode_does_not_hash_or_forward_provenance_options(self):
        self.runtime.capture_runtime = lambda *args: self.fail("standard mode hashed runtime")
        report = self.render(reproducible=False, sources=None)
        self.assertTrue(report.ok)
        self.assertFalse(report.all_scenes_certified or report.reproducible)
        for _, _, options in self.calls:
            self.assertNotIn("sources", options)
            self.assertNotIn("reproducible", options)

    def test_later_sidecar_collision_prevents_every_constructor(self):
        path = self.root / "outputs" / "Two.png.manifest"
        path.parent.mkdir()
        path.write_bytes(b"preserve")
        with self.assertRaises(FileExistsError):
            self.render()
        self.assertEqual(self.events, [])
        self.assertEqual(self.snapshots, [])
        self.assertEqual(path.read_bytes(), b"preserve")

    def test_dangling_sidecar_symlink_is_not_treated_as_absent(self):
        path = self.root / "outputs" / "Two.png.manifest"
        path.parent.mkdir()
        path.symlink_to("does-not-exist")
        with self.assertRaises(FileExistsError):
            self.render()
        self.assertTrue(path.is_symlink())
        self.assertEqual(self.events, [])

    def test_artifact_collision_prevents_every_constructor(self):
        path = self.root / "outputs" / "Two.png"
        path.parent.mkdir()
        path.write_bytes(b"preserve")
        with self.assertRaises(FileExistsError):
            self.render()
        self.assertEqual(self.events, [])
        self.assertEqual(path.read_bytes(), b"preserve")

    def test_missing_and_invalid_sources_fail_before_capture(self):
        for sources in (None, {}, {"../scene.py": b"data"}, {"scene.py": "not bytes"}):
            with self.subTest(sources=sources), self.assertRaises((TypeError, ValueError)):
                self.render(sources=sources)
        self.assertEqual(self.snapshots, [])
        self.assertEqual(self.events, [])

    def test_provider_is_deferred_and_freezes_lazy_imports_per_scene(self):
        values, calls = {"scene.py": b"source"}, []
        outer = self
        class One(self.Scene):
            def run(self):
                values["one.py"] = b"lazy one"
                super().run()
        class Two(self.Scene):
            def run(self):
                values["two.py"] = b"lazy two"
                super().run()
        def sources():
            calls.append(tuple(outer.events))
            return values
        report = self.render({"One": One, "Two": Two}, sources=sources)
        self.assertTrue(report.ok)
        self.assertEqual(len(calls), 2)
        self.assertEqual(calls[0][-1], ("run", "One"))
        self.assertNotIn("two.py", self.sources_seen[0])
        self.assertIn("two.py", self.sources_seen[1])

    def test_static_sources_and_passed_runtime_tables_are_detached(self):
        values = {"scene.py": b"original"}
        def observer(outcome):
            values["scene.py"] = b"edited"
            self.calls[-1][2]["sources"]["scene.py"] = b"also edited"
            self.calls[-1][2]["runtime_identities"].clear()
        report = self.render(sources=values, on_result=observer)
        self.assertTrue(report.ok)
        self.assertEqual(self.sources_seen, [{"scene.py": b"original"}] * 2)
        self.assertEqual(len(self.snapshots[0].identities), 1)

    def test_runtime_change_stops_next_constructor_and_retains_first_result(self):
        def observer(outcome):
            self.payload.write_bytes(b"runtime generation two")
        with self.assertRaises(self.batch.BatchRenderError) as caught:
            self.render(on_result=observer)
        report = caught.exception.result
        self.assertTrue(report.reproducible)
        self.assertFalse(report.all_scenes_certified)
        self.assertEqual([x.status for x in report.outcomes], ["succeeded", "failed"])
        self.assertEqual(len(self.calls), 1)
        self.assertTrue(report.outcomes[0].result.manifest.is_file())

    def test_runtime_change_with_keep_going_never_runs_new_generation(self):
        report = self.render(self.scenes("One", "Two", "Three"), continue_on_error=True,
            on_result=lambda outcome: self.payload.write_bytes(b"new runtime"))
        self.assertEqual([x.status for x in report.outcomes], ["succeeded", "failed", "failed"])
        self.assertEqual(len(self.calls), 1)

    def test_supplied_identity_mismatch_refuses_before_construction(self):
        with self.assertRaisesRegex(RuntimeError, "CAPABILITY.*disagree"):
            self.render(runtime_identities={"wheel": "made-up build"})
        self.assertEqual(self.calls, [])

    def test_legacy_labels_are_replaced_by_measured_identities(self):
        self.render(runtime_identities={"wheel": "legacy version"})
        for _, _, options in self.calls:
            self.assertEqual(options["runtime_identities"], self.snapshots[0].identities)

    def test_invalid_modes_and_checkpoint_refuse_without_locks(self):
        for options in ({"reproducible": 1}, {"format": "mp4"}, {"format": "gif"},
                        {"checkpoint": self.root / "checkpoint", "resume_key": "key"}):
            with self.subTest(options=options), self.assertRaises((TypeError, RuntimeError)):
                self.render(**options)
        self.assertEqual(self.calls, [])
        self.assertFalse((self.root / "checkpoint").exists())
        self.assertFalse((self.root / "checkpoint.lock").exists())

    def test_missing_publisher_and_windows_refuse_early(self):
        with patch.object(self.native, "_portal_publish_manifest", None), self.assertRaisesRegex(RuntimeError, "publisher"):
            self.render()
        with patch.object(self.provenance.sys, "platform", "win32"), self.assertRaisesRegex(RuntimeError, "windows"):
            self.render()
        self.assertEqual(self.calls, [])

    def test_invalid_plan_is_checked_before_runtime_scan(self):
        with self.assertRaisesRegex(ValueError, "collision"):
            self.render(self.scenes("One", "one"))
        self.assertEqual(self.snapshots, [])

    def test_failed_scene_keep_going_retains_successful_manifests(self):
        class Broken(self.Scene):
            def run(self):
                raise ValueError("authored failure")
        report = self.render({"One": self.Scene, "Broken": Broken, "Three": self.Scene}, continue_on_error=True)
        self.assertEqual([x.status for x in report.outcomes], ["succeeded", "failed", "succeeded"])
        self.assertFalse(report.all_scenes_certified)
        self.assertIn("authored failure", report.outcomes[1].message)
        for index in (0, 2):
            self.assertTrue(report.outcomes[index].result.certified)
            self.assertTrue(report.outcomes[index].result.manifest.exists())

    def test_fail_fast_retains_not_run_and_requested_mode(self):
        class Broken(self.Scene):
            def run(self):
                raise ValueError("authored")
        with self.assertRaises(self.batch.BatchRenderError) as caught:
            self.render({"Broken": Broken, "Later": self.Scene})
        self.assertTrue(caught.exception.result.reproducible)
        self.assertEqual([x.status for x in caught.exception.result.outcomes], ["failed", "not_run"])
        self.assertIsInstance(caught.exception.__cause__, ValueError)

    def test_keyboard_interrupt_preserves_identity_and_progress(self):
        interrupt = KeyboardInterrupt("stop")
        class Stop(self.Scene):
            def run(self):
                raise interrupt
        with self.assertRaises(KeyboardInterrupt) as caught:
            self.render({"One": self.Scene, "Stop": Stop, "Later": self.Scene}, continue_on_error=True)
        self.assertIs(caught.exception, interrupt)
        report = interrupt.render_batch_result
        self.assertTrue(report.reproducible)
        self.assertEqual([x.status for x in report.outcomes], ["succeeded", "cancelled", "not_run"])

    def test_observer_failure_does_not_relabel_published_scene(self):
        failure = RuntimeError("observer")
        def observer(outcome):
            raise failure
        with self.assertRaises(RuntimeError) as caught:
            self.render(on_result=observer)
        self.assertIs(caught.exception, failure)
        report = failure.render_batch_result
        self.assertTrue(report.reproducible)
        self.assertEqual([x.status for x in report.outcomes], ["succeeded", "not_run"])

    def test_partial_receipt_is_failure_without_deleting_output(self):
        for index, change in enumerate((dict(certified=False), dict(manifest=None),
                                       dict(closure_digest="bad"), dict(manifest=Path("other")),
                                       dict(destination=Path("other")), dict(format="mp4"))):
            with self.subTest(change=change):
                def sink(scene, destination, **kwargs):
                    return replace(self.sink(scene, destination, **kwargs), **change)
                self.batch.render_scene = sink
                report = self.render(self.scenes("Case" + str(index)), continue_on_error=True)
                self.assertEqual(report.outcomes[0].status, "failed")
                self.assertFalse(report.all_scenes_certified)
                self.assertTrue(report.outcomes[0].destination.exists())
                self.assertTrue(report.outcomes[0].notes)

    def test_observer_creating_later_sidecar_is_detected_before_next_constructor(self):
        def observer(outcome):
            (self.root / "outputs" / "Two.png.manifest").mkdir(exist_ok=True)
        with self.assertRaises(self.batch.BatchRenderError):
            self.render(on_result=observer)
        self.assertEqual(len(self.calls), 1)

    def test_ranges_options_and_constructor_kwargs_preserved(self):
        jobs = [self.batch.RenderJob("Named", self.Scene, {"value": 7})]
        report = self.render(jobs, resolution=(48, 27), fps=8, threads=4, animation_range=(1, 3),
                             _output_options={"transparent": True})
        receipt = report.outcomes[0].result
        self.assertEqual((receipt.resolution, receipt.fps, receipt.threads, receipt.seed, receipt.animation_range),
                         ((48, 27), 8, 4, 7, (1, 3)))
        self.assertEqual(self.calls[0][2]["_output_options"], {"transparent": True})

    def test_empty_or_standard_report_never_claims_certified_scenes(self):
        self.assertFalse(self.batch.BatchRenderResult((), reproducible=True).all_scenes_certified)
        report = self.render()
        self.assertFalse(self.batch.BatchRenderResult(report.outcomes).all_scenes_certified)

    def test_provider_failure_preserves_previous_publication(self):
        calls = []
        def provider():
            calls.append(None)
            if len(calls) == 2:
                raise RuntimeError("lazy source failure")
            return {"scene.py": b"source"}
        report = self.render(sources=provider, continue_on_error=True)
        self.assertEqual([x.status for x in report.outcomes], ["succeeded", "failed"])
        self.assertTrue(report.outcomes[0].result.manifest.exists())
        self.assertFalse(report.outcomes[1].destination.exists())

    def test_runtime_capture_failure_has_named_capability_error(self):
        def capture(native):
            raise self.runtime.RuntimeIdentityError("missing runtime")
        self.runtime.capture_runtime = capture
        with self.assertRaisesRegex(RuntimeError, "CAPABILITY.*missing runtime"):
            self.render()
        self.assertEqual(self.events, [])


if __name__ == "__main__":
    unittest.main()
