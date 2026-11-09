"""Real batch/journal/receipt logic and filesystem I/O; no renderer or encoder.

Only native rendering is replaced. Receipt classes and scalar admission helpers
are loaded from production definitions, while the actual batch loop, pair
adapter, locking, hashing and checkpoint persistence run unchanged. Execute the
existing batch_checkpoint_acceptance.py against an installed wheel for render proof.
"""
from __future__ import annotations

import ast
import copy
from dataclasses import dataclass
import hashlib
import importlib
import operator
from pathlib import Path
import os
import sys
import tempfile
import types
from typing import Any
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1] / "python" / "fmn_python"
PACKAGE = "_fmn_paired_checkpoint_test"


def definitions(name, selected, **globals):
    """Load named production declarations, not a parallel implementation."""
    source = ast.parse((ROOT / (name + ".py")).read_text())
    body = [ast.ImportFrom(module="__future__", names=[ast.alias(name="annotations")], level=0)]
    for node in source.body:
        names = ({node.name} if isinstance(node, (ast.FunctionDef, ast.ClassDef)) else
                 {target.id for target in node.targets if isinstance(target, ast.Name)}
                 if isinstance(node, ast.Assign) else set())
        if names.intersection(selected):
            body.append(node)
    module = types.ModuleType(PACKAGE + "." + name)
    module.__dict__.update(globals)
    sys.modules[module.__name__] = module
    exec(compile(ast.fix_missing_locations(ast.Module(body=body, type_ignores=[])),
                 str(ROOT / (name + ".py")), "exec"), module.__dict__)
    return module


def unavailable(*args, **kwargs):
    raise AssertionError("unrelated native/provenance path entered a checkpoint unit test")


package = types.ModuleType(PACKAGE)
package.__path__ = [str(ROOT)]
sys.modules[PACKAGE] = package
rendering = definitions("rendering", {"RenderResult", "_positive_integer", "_FORMATS"},
                        dataclass=dataclass, Path=Path, Any=Any, copy=copy, operator=operator)
rendering.SourceInputs = Any
rendering.render_scene = unavailable
provenance = definitions("batch_provenance", {"validate_batch_mode"}, sys=sys)
provenance.BatchProvenance = unavailable
provenance.has_provenance = lambda value: False
selection = definitions("render_selection", {"animation_range", "_MAX_INDEX"}, operator=operator, Any=Any)
subdivision = types.ModuleType(PACKAGE + ".subdivision")
subdivision.SubdividedRenderResult = type("SubdividedRenderResult", (), {})
sys.modules[subdivision.__name__] = subdivision
subdivision_rendering = definitions("subdivision_rendering", {"validate_subdivided_mode"},
                                    _CLIP_FORMATS={"gif", "y4m", "png_sequence", "mp4", "mov", "wav"})
subdivision_rendering.render_subdivided_scene = unavailable
bundle = types.ModuleType(PACKAGE + ".bundle_export")
bundle.BundleExportResult = type("BundleExportResult", (), {})
bundle._validated_limits = bundle.export_bundle = bundle.require_bundle_capability = unavailable
sys.modules[bundle.__name__] = bundle
paired = importlib.import_module(PACKAGE + ".paired_output")
journal = importlib.import_module(PACKAGE + ".batch_checkpoint")
batch = importlib.import_module(PACKAGE + ".batch_rendering")
adapter = importlib.import_module(PACKAGE + ".paired_checkpoint")


class Scene:
    pass


native = types.SimpleNamespace(Scene=Scene, _CapabilityError=RuntimeError,
                               _CameraCapture=types.SimpleNamespace(prepare_png=lambda: None))


class PairedCheckpointTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="fmn-paired-checkpoint-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.checkpoint = self.root / "progress.json"
        self.calls = []
        self.fail = set()
        self.partial = set()
        calls = self.calls

        class First(Scene):
            def __init__(self, **kwargs):
                calls.append((type(self).__name__, kwargs))

        class Second(First):
            pass

        class Third(First):
            pass

        self.scenes = {"First": First, "Second": Second, "Third": Third}
        self.addCleanup(patch.stopall)
        patch.dict(sys.modules, {"manimlib": native}).start()
        patch.object(batch, "render_scene_with_still", self.render_pair).start()
        patch.object(batch, "render_scene", self.render_single).start()

    def receipt(self, path, format, **options):
        resolution = options.get("resolution") or (32, 24)
        fps = options.get("fps") or 8
        content = ("fixture-" + format + "-" + str(path)).encode()
        path.parent.mkdir(parents=True, exist_ok=True)
        if format == "png_sequence":
            path.mkdir()
            for index in range(2):
                with (path / f"{index:010d}.png").open("xb") as stream:
                    stream.write(content)
            size = len(content) * 2
        else:
            with path.open("xb") as stream:
                stream.write(content)
            size = len(content)
        return rendering.RenderResult(
            path, format, resolution, fps, options.get("threads") or 1,
            "fixture-not-native", size, hashlib.sha256(content).hexdigest(),
            None if format == "wav" else 1 if format == "png" else 2,
            0 if format == "wav" else None, 17,
            animation_range=options.get("animation_range"),
        )

    def render_single(self, scene, path, *, format, scene_kwargs=None, **options):
        scene(**(scene_kwargs or {}))
        if scene.__name__ in self.fail:
            raise RuntimeError("transient renderer failure")
        return self.receipt(path, format, **options)

    def render_pair(self, scene, path, *, still_destination, format, scene_kwargs=None, **options):
        primary = self.render_single(scene, path, format=format, scene_kwargs=scene_kwargs, **options)
        if scene.__name__ in self.partial:
            error = RuntimeError("PNG publication failed after primary publication")
            error.render_pair_result = paired.PairedRenderResult(path, still_destination, primary, None, False)
            raise error
        still = self.receipt(still_destination, "png", **options)
        return paired.PairedRenderResult(path, still_destination, primary, still, True)

    def run_batch(self, *, scenes=None, **changes):
        options = dict(format="gif", save_last_frame=True, checkpoint=self.checkpoint,
                       resume_key="authored-inputs-v1", resolution=(32, 24), fps=8, threads=1)
        options.update(changes)
        return batch.render_scenes(self.scenes if scenes is None else scenes,
                                   self.root / "out", **options)

    def saved(self):
        return journal._read_document(self.checkpoint)

    def store(self, document):
        self.checkpoint.write_bytes(journal._json(document))

    def test_failure_then_resume_skips_complete_pairs(self):
        self.fail.add("Second")
        with self.assertRaises(batch.BatchRenderError) as failure:
            self.run_batch()
        self.assertEqual([row.status for row in failure.exception.result.outcomes],
                         ["succeeded", "failed", "not_run"])
        document = self.saved()
        self.assertEqual(set(document["artifacts"]["First"]), {"primary", "still"})
        self.assertTrue(document["plan"]["options"]["save_last_frame"])
        self.fail.clear()
        result = self.run_batch(resume=True)
        self.assertTrue(result.ok)
        self.assertEqual([name for name, _ in self.calls], ["First", "Second", "Second", "Third"])
        self.assertIsInstance(result.outcomes[0].result, paired.PairedRenderResult)
        self.assertEqual(result.outcomes[0].result.as_dict(), document["outcomes"][0]["result"])
        self.assertFalse(result.all_scenes_certified)

    def test_observer_failure_does_not_lose_completed_pair(self):
        def observer(outcome):
            raise KeyboardInterrupt()
        with self.assertRaises(KeyboardInterrupt):
            self.run_batch(on_result=observer)
        self.assertEqual([name for name, _ in self.calls], ["First"])
        self.assertTrue(self.run_batch(resume=True).ok)
        self.assertEqual([name for name, _ in self.calls], ["First", "Second", "Third"])

    def test_all_completed_pairs_are_reused_without_construction(self):
        before = self.run_batch()
        self.calls.clear()
        after = self.run_batch(resume=True)
        self.assertEqual(self.calls, [])
        self.assertEqual(before.as_dict(), after.as_dict())

    def test_native_file_formats_and_wav_receipts_round_trip(self):
        for format in ("mp4", "mov", "gif", "y4m", "wav", "png_sequence"):
            with self.subTest(format=format):
                self.checkpoint = self.root / (format + ".json")
                # Different scene names prevent cross-format PNG collisions.
                jobs = {"Scene_" + format: self.scenes["First"]}
                result = self.run_batch(scenes=jobs, format=format)
                self.calls.clear()
                resumed = self.run_batch(scenes=jobs, format=format, resume=True)
                self.assertEqual(self.calls, [])
                self.assertEqual(result.as_dict(), resumed.as_dict())

    def test_corruption_of_either_member_fails_before_any_scene(self):
        self.run_batch()
        saved = self.saved()
        for member in ("primary", "still"):
            with self.subTest(member=member):
                path = Path(saved["outcomes"][0]["result"][member]["destination"])
                original, info = path.read_bytes(), path.stat()
                path.write_bytes(b"x" * len(original))
                os.utime(path, ns=(info.st_atime_ns, info.st_mtime_ns))
                self.calls.clear()
                with self.assertRaises(ValueError):
                    self.run_batch(resume=True)
                self.assertEqual(self.calls, [])
                self.assertEqual(path.read_bytes(), b"x" * len(original))
                path.write_bytes(original)

    def test_missing_png_never_reuses_primary_alone(self):
        result = self.run_batch()
        png = result.outcomes[0].result.still_destination
        png.rename(png.with_suffix(".saved"))
        self.calls.clear()
        with self.assertRaises(FileNotFoundError):
            self.run_batch(resume=True)
        self.assertEqual(self.calls, [])
        self.assertTrue(result.outcomes[0].destination.exists())

    def test_partial_pair_remains_no_clobber_and_is_not_reusable(self):
        self.partial.add("Second")
        with self.assertRaises(batch.BatchRenderError) as failure:
            self.run_batch()
        partial = failure.exception.result.outcomes[1].result
        self.assertFalse(partial.completed)
        self.assertTrue(partial.destination.exists())
        self.assertNotIn("Second", self.saved()["artifacts"])
        self.partial.clear()
        self.calls.clear()
        with self.assertRaises(FileExistsError):
            self.run_batch(resume=True)
        self.assertEqual(self.calls, [])
        self.assertTrue(partial.destination.exists())

    def test_unjournaled_complete_pair_cannot_be_adopted(self):
        self.fail.add("Second")
        with self.assertRaises(batch.BatchRenderError):
            self.run_batch()
        self.fail.clear()
        path = self.root / "out/Second.gif"
        self.render_pair(self.scenes["Second"], path, still_destination=path.with_suffix(".png"), format="gif")
        self.calls.clear()
        with self.assertRaises(FileExistsError):
            self.run_batch(resume=True)
        self.assertEqual(self.calls, [])

    def test_malformed_pair_receipts_are_not_successes(self):
        self.run_batch()
        saved = self.saved()
        mutations = [
            lambda r: r.update(completed=False),
            lambda r: r.update(version=True),
            lambda r: r.update(still=None),
            lambda r: r.update(unreceipted_artifacts=[r["destination"]]),
            lambda r: r.update(still_destination=str(self.root / "other.png")),
            lambda r: r["still"].update(destination=str(self.root / "other.png")),
            lambda r: r["still"].update(frame_count=2),
            lambda r: r["still"].update(certified=True),
            lambda r: r["still"].update(bytes=True),
            lambda r: r["still"].update(manifest=None),
            lambda r: r["still"].update(resolution=[34, 24]),
            lambda r: r["still"].update(fps=9),
            lambda r: r["still"].update(seed=True),
            lambda r: r["primary"].update(frame_count=None),
            lambda r: r["primary"].update(format="fmtl"),
            lambda r: r["primary"].update(digest="0" * 64),
        ]
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                document = copy.deepcopy(saved)
                mutate(document["outcomes"][0]["result"])
                self.store(document)
                self.calls.clear()
                with self.assertRaises((ValueError, TypeError)):
                    self.run_batch(resume=True)
                self.assertEqual(self.calls, [])
        self.store(saved)

    def test_inventory_roles_are_required(self):
        self.run_batch()
        saved = self.saved()
        for replacement in (saved["artifacts"]["First"]["primary"], {"primary": saved["artifacts"]["First"]["primary"]}):
            document = copy.deepcopy(saved)
            document["artifacts"]["First"] = replacement
            self.store(document)
            self.calls.clear()
            with self.assertRaises(ValueError):
                self.run_batch(resume=True)
            self.assertEqual(self.calls, [])

    def test_changed_plan_and_single_output_mode_refuse(self):
        self.run_batch()
        for changes in ({"save_last_frame": False}, {"fps": 12}, {"resolution": (34, 24)},
                        {"animation_range": (0, 2)}, {"_output_options": {"video_crf": 16}},
                        {"resume_key": "different-inputs"}, {"scenes": dict(reversed(list(self.scenes.items())))}):
            with self.subTest(changes=changes):
                self.calls.clear()
                with self.assertRaises(ValueError):
                    self.run_batch(resume=True, **changes)
                self.assertEqual(self.calls, [])

    def test_checkpoint_cannot_overwrite_the_companion_png(self):
        self.checkpoint = self.root / "out/First.png"
        with self.assertRaisesRegex(ValueError, "outside published artifacts"):
            self.run_batch()
        self.assertEqual(self.calls, [])
        self.assertFalse(self.checkpoint.exists())

    def test_checkpoint_lock_cannot_be_inside_sequence(self):
        self.checkpoint = self.root / "out/First/frames/progress.json"
        with self.assertRaisesRegex(ValueError, "outside published artifacts"):
            self.run_batch(format="png_sequence")
        self.assertEqual(self.calls, [])
        self.assertFalse(self.checkpoint.exists())

    @unittest.skipUnless(hasattr(os, "symlink"), "symlinks unavailable")
    def test_symlink_member_is_rejected_without_touching_target(self):
        result = self.run_batch()
        path = result.outcomes[0].result.still_destination
        actual = path.with_suffix(".original")
        path.rename(actual)
        path.symlink_to(actual)
        before = actual.read_bytes()
        self.calls.clear()
        with self.assertRaises(ValueError):
            self.run_batch(resume=True)
        self.assertEqual(self.calls, [])
        self.assertEqual(actual.read_bytes(), before)

    @unittest.skipUnless(hasattr(os, "mkfifo"), "FIFOs unavailable")
    def test_fifo_member_refuses_without_blocking(self):
        result = self.run_batch()
        path = result.outcomes[0].result.still_destination
        path.rename(path.with_suffix(".original"))
        os.mkfifo(path)
        self.calls.clear()
        with self.assertRaises(ValueError):
            self.run_batch(resume=True)
        self.assertEqual(self.calls, [])

    def test_combined_file_budget_includes_the_png(self):
        with patch.object(journal, "_MAX_FILES", 2):
            with self.assertRaisesRegex(ValueError, "combined file budget"):
                self.run_batch(format="png_sequence")
        self.assertEqual(self.saved()["outcomes"][0]["status"], "not_run")
        self.assertTrue((self.root / "out/First/frames.png").exists())

    def test_prior_single_artifact_v2_plans_still_round_trip(self):
        before = self.run_batch(save_last_frame=False)
        self.assertNotIn("save_last_frame", self.saved()["plan"]["options"])
        self.calls.clear()
        self.assertEqual(before.as_dict(), self.run_batch(save_last_frame=False, resume=True).as_dict())
        self.assertEqual(self.calls, [])

    def test_capability_and_subdivision_exclusions_still_precede_construction(self):
        for changes in ({"reproducible": True}, {"subdivide": True}, {"format": "png"}):
            with self.subTest(changes=changes):
                with self.assertRaises((ValueError, RuntimeError)):
                    self.run_batch(**changes)
                self.assertEqual(self.calls, [])
                self.assertFalse(self.checkpoint.exists())

    def test_checkpoint_remains_exclusive_during_observers(self):
        seen = []
        def observer(outcome):
            with self.assertRaises((BlockingIOError, OSError)):
                with journal.BatchCheckpoint(self.checkpoint, resume=True, key="authored-inputs-v1"):
                    pass
            seen.append(outcome.name)
        self.run_batch(on_result=observer)
        self.assertEqual(seen, list(self.scenes))


if __name__ == "__main__":
    unittest.main(verbosity=2)
