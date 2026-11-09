"""Real batch/session orchestration with a file-publishing native boundary double.

The files below contain test markers, not rendered pixels. The companion
installed-wheel suite checks real Lumen/Reel output through the same API.
"""
import hashlib
import itertools
import json
import os
import pathlib
import sys
import tempfile
import threading
import types
import unittest
from unittest.mock import patch

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / "python"))
from fmn_python import BatchRenderError, BatchRenderResult, RenderJob, render_scenes


class EndScene(Exception):
    pass


class Camera:
    def __init__(self):
        self.shape, self.fps, self._core = (8, 4), 8, self
    def get_pixel_shape(self):
        return self.shape
    def set_pixel_shape(self, width, height):
        self.shape = (width, height)


class Scene:
    trace = []
    instances = []
    def __init__(self, random_seed=0, **kwargs):
        self.camera, self.random_seed = Camera(), random_seed
        self.active, self.used = False, False
        self.run_error = self.start_error = self.finish_error = self.abort_error = None
        self.events = []
        self._render_invocations = [{"argv": ["ffmpeg", "input"]}]
        self._render_audio_inputs = []
        self.kwargs = kwargs
        Scene.instances.append(self)
        Scene.trace.append(("construct", type(self).__name__, threading.get_ident()))
    def _begin_native_output(self, *request):
        self.events.append("begin")
        if self.active:
            raise RuntimeError("external generation active")
        if self.used:
            raise RuntimeError("scene is not pristine")
        if self.start_error:
            raise self.start_error
        self.active, self.request = True, request
    def run(self):
        self.events.append("run")
        Scene.trace.append(("run", type(self).__name__, threading.get_ident()))
        if self.run_error:
            raise self.run_error
    def _abort_render(self):
        self.events.append("abort")
        self.active = False
        if self.abort_error:
            raise self.abort_error
    def _finish_render(self):
        self.events.append("finish")
        if self.finish_error:
            raise self.finish_error
        path, format, width, height, fps, threads, seed, reproducible = self.request
        if reproducible is not False:
            raise AssertionError("the uncertified protocol double cannot certify a render")
        destination = pathlib.Path(path)
        payload = f"test-native-boundary:{seed}:{format}:{width}x{height}".encode()
        destination.parent.mkdir(parents=True, exist_ok=True)
        if format == "png_sequence":
            destination.mkdir()
            (destination / "frame-test-marker").write_bytes(payload)
        else:
            with destination.open("xb") as stream:
                stream.write(payload)
        self.active, self.used = False, True
        Scene.trace.append(("publish", type(self).__name__, threading.get_ident()))
        return path, 4800 if format == "wav" else 3, len(payload), hashlib.sha256(payload).hexdigest(), "native-boundary-double", threads


class First(Scene):
    pass


class Second(Scene):
    pass


native = types.SimpleNamespace(Scene=Scene, EndScene=EndScene)


class BatchRenderingTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.imports = patch.dict(sys.modules, {"manimlib": native})
        self.imports.start()
        self.addCleanup(self.imports.stop)
        Scene.trace, Scene.instances = [], []
    def render(self, scenes, **options):
        options.setdefault("format", "png")
        options.setdefault("threads", 2)
        return render_scenes(scenes, self.root / "output", **options)
    def test_ordered_mapping_produces_native_receipts(self):
        report = self.render({"second": Second, "first": First})
        self.assertTrue(report.ok)
        self.assertEqual([x.name for x in report.outcomes], ["second", "first"])
        self.assertEqual(report.counts, dict(succeeded=2, failed=0, cancelled=0, not_run=0))
        for outcome in report.outcomes:
            self.assertEqual(outcome.result.digest, hashlib.sha256(outcome.destination.read_bytes()).hexdigest())
            self.assertEqual(outcome.result.bytes, outcome.destination.stat().st_size)
            self.assertFalse(outcome.result.certified)
        self.assertEqual(json.loads(json.dumps(report.as_dict()))["execution"], "sequential")
    def test_classes_construct_just_in_time_on_calling_thread(self):
        self.render(iter([First, Second]))
        self.assertEqual([(kind, name) for kind, name, _ in Scene.trace], [
            ("construct", "First"), ("run", "First"), ("publish", "First"),
            ("construct", "Second"), ("run", "Second"), ("publish", "Second"),
        ])
        self.assertEqual({owner for _, _, owner in Scene.trace}, {threading.get_ident()})
    def test_per_job_class_arguments_and_instances(self):
        marker = object()
        kwargs = {"random_seed": 17, "marker": marker}
        instance = Second(random_seed=21)
        report = self.render([RenderJob("param", First, kwargs), RenderJob("live", instance)])
        self.assertEqual([x.result.seed for x in report.outcomes], [17, 21])
        self.assertIs(Scene.instances[-1].kwargs["marker"], marker)
        self.assertEqual(instance.events, ["begin", "run", "finish"])
        self.assertEqual(kwargs["random_seed"], 17)
    def test_all_formats_and_native_count_units(self):
        for format in ("png", "png_sequence", "gif", "y4m", "wav", "mp4", "mov"):
            with self.subTest(format=format):
                report = render_scenes([First], self.root / format, format=format, threads=2)
                item = report.outcomes[0]
                self.assertEqual(item.result.format, format)
                if format == "png_sequence":
                    self.assertEqual(item.destination, self.root / format / "First" / "frames")
                if format == "wav":
                    self.assertIsNone(item.result.frame_count)
                    self.assertEqual(item.result.sample_frames, 4800)
                    self.assertEqual(item.result.as_dict()["sample_rate"], 48000)
                elif format in {"mp4", "mov"}:
                    self.assertEqual(item.result.ffmpeg_invocations[0]["argv"], ["ffmpeg", "input"])
    def test_fail_fast_keeps_success_and_unattempted_jobs(self):
        failure = ValueError("broken scene")
        class Failing(Scene):
            def run(self):
                raise failure
        with self.assertRaises(BatchRenderError) as caught:
            self.render([First, Failing, Second])
        self.assertIs(caught.exception.__cause__, failure)
        report = caught.exception.result
        self.assertEqual([x.status for x in report.outcomes], ["succeeded", "failed", "not_run"])
        self.assertTrue(report.outcomes[0].destination.exists())
        self.assertFalse(report.outcomes[1].destination.exists())
        self.assertEqual(Scene.instances[-1].events[-1], "abort")
        self.assertEqual(len(Scene.instances), 2)
    def test_keep_going_is_not_false_success(self):
        class Failing(Scene):
            def run(self):
                raise OSError("sink")
        report = self.render([Failing, First, Second], continue_on_error=True)
        self.assertFalse(report.ok)
        self.assertEqual([x.status for x in report.outcomes], ["failed", "succeeded", "succeeded"])
        self.assertIsNone(report.outcomes[0].result)
    def test_constructor_failure_can_continue_without_opening_session(self):
        class Broken(Scene):
            def __init__(self):
                raise TypeError("constructor")
        report = self.render([Broken, First], continue_on_error=True)
        self.assertEqual(report.counts["failed"], 1)
        self.assertEqual(len(Scene.instances), 1)
        self.assertEqual(Scene.instances[0].events, ["begin", "run", "finish"])
    def test_interrupts_preserve_exact_exception_and_progress(self):
        for failure in (KeyboardInterrupt("stop"), SystemExit(13), GeneratorExit()):
            with self.subTest(failure=type(failure).__name__):
                class Interrupted(Scene):
                    def run(self):
                        raise failure
                directory = self.root / type(failure).__name__
                with self.assertRaises(type(failure)) as caught:
                    render_scenes([First, Interrupted, Second], directory, format="png", continue_on_error=True)
                self.assertIs(caught.exception, failure)
                report = failure.render_batch_result
                self.assertEqual([x.status for x in report.outcomes], ["succeeded", "cancelled", "not_run"])
                self.assertEqual(Scene.instances[-1].events[-1], "abort")
                self.assertTrue(report.outcomes[0].destination.exists())
    def test_observer_failure_does_not_relabel_published_artifact(self):
        failure = RuntimeError("observer")
        def observer(outcome):
            self.assertFalse(Scene.instances[-1].active)
            self.assertTrue(outcome.destination.exists())
            raise failure
        with self.assertRaises(RuntimeError) as caught:
            self.render([First, Second], on_result=observer, continue_on_error=True)
        self.assertIs(caught.exception, failure)
        self.assertEqual([x.status for x in failure.render_batch_result.outcomes], ["succeeded", "not_run"])
        self.assertEqual(len(Scene.instances), 1)
        self.assertNotIn("abort", Scene.instances[0].events)
    def test_observer_sees_failed_outcome_after_abort(self):
        instance = First()
        instance.run_error = ValueError("failed")
        seen = []
        def observer(outcome):
            seen.append(outcome.status)
            self.assertFalse(instance.active)
        report = self.render([instance, Second], continue_on_error=True, on_result=observer)
        self.assertEqual(seen, ["failed", "succeeded"])
        self.assertEqual(report.counts["succeeded"], 1)
    def test_external_render_owner_is_never_aborted(self):
        instance = First()
        instance.active = True
        report = self.render([instance, Second], continue_on_error=True)
        self.assertEqual(report.outcomes[0].status, "failed")
        self.assertTrue(instance.active)
        self.assertEqual(instance.events, ["begin"])
    def test_native_finish_failure_has_no_success_receipt(self):
        instance = First()
        instance.finish_error = OSError("publish failed")
        report = self.render([instance, Second], continue_on_error=True)
        self.assertEqual(instance.events, ["begin", "run", "finish", "abort"])
        self.assertIsNone(report.outcomes[0].result)
        self.assertFalse(report.outcomes[0].destination.exists())
    def test_end_scene_is_normal_completion(self):
        instance = First()
        instance.run_error = EndScene("done")
        self.assertTrue(self.render([instance, Second]).ok)
        self.assertNotIn("abort", instance.events)
    def test_late_invalid_job_prevents_all_construction(self):
        with self.assertRaises(TypeError):
            self.render([First, object()])
        self.assertEqual(Scene.instances, [])
        self.assertFalse((self.root / "output").exists())
    def test_portable_name_collisions_prevent_all_construction(self):
        for left, right in [("Name", "name"), ("Café", "Cafe\u0301"), ("K", "\u212a")]:
            with self.assertRaisesRegex(ValueError, "collision"):
                self.render([RenderJob(left, First), RenderJob(right, Second)])
        self.assertEqual(Scene.instances, [])
    def test_unsafe_or_reserved_names_are_rejected(self):
        for name in ("", "..", "../escape", "a/b", "a\\b", "C:escape", "x\0y", "con", "LPT9", "a" * 129):
            with self.subTest(name=name), self.assertRaises(ValueError):
                self.render([RenderJob(name, First)])
        self.assertEqual(Scene.instances, [])
    def test_repeated_scene_instance_is_rejected_before_start(self):
        instance = First()
        with self.assertRaisesRegex(ValueError, "instance"):
            self.render({"one": instance, "two": instance})
        self.assertEqual(instance.events, [])
    def test_same_class_can_render_parameterized_variants(self):
        report = self.render([RenderJob("one", First, {"random_seed": 1}), RenderJob("two", First, {"random_seed": 2})])
        self.assertTrue(report.ok)
        self.assertIsNot(Scene.instances[0], Scene.instances[1])
        self.assertNotEqual(report.outcomes[0].result.digest, report.outcomes[1].result.digest)
    def test_invalid_class_kwargs_are_preflight_errors(self):
        for entry in (RenderJob("one", First(), {}), RenderJob("two", First, []), RenderJob("three", First, {1: 3})):
            with self.assertRaises(TypeError):
                self.render([entry])
        self.assertTrue(all(not scene.events for scene in Scene.instances))
    def test_infinite_job_iterable_is_bounded(self):
        consumed = []
        def entries():
            for index in itertools.count():
                consumed.append(index)
                yield RenderJob(f"Scene{index}", First)
        with self.assertRaisesRegex(ValueError, "max_jobs=3"):
            self.render(entries(), max_jobs=3)
        self.assertEqual(consumed, [0, 1, 2, 3])
        self.assertEqual(Scene.instances, [])
    def test_generator_failure_does_not_publish_a_prefix(self):
        def entries():
            yield First
            raise ValueError("enumeration failed")
        with self.assertRaisesRegex(ValueError, "enumeration failed"):
            self.render(entries())
        self.assertEqual(Scene.instances, [])
    def test_output_root_is_frozen_before_constructor_changes_cwd(self):
        previous = os.getcwd()
        self.addCleanup(os.chdir, previous)
        os.chdir(self.root)
        other = self.root / "elsewhere"
        other.mkdir()
        class Chdir(Scene):
            def __init__(self):
                os.chdir(other)
                super().__init__()
        report = render_scenes([Chdir, Second], "out", format="png")
        self.assertEqual(report.outcomes[1].destination, self.root / "out" / "Second.png")
        self.assertFalse((other / "out").exists())
    def test_existing_later_output_is_rejected_without_any_construction(self):
        directory = self.root / "output"
        directory.mkdir()
        original = directory / "Second.png"
        original.write_bytes(b"preserve")
        with self.assertRaises(FileExistsError):
            self.render([First, Second])
        self.assertEqual(original.read_bytes(), b"preserve")
        self.assertFalse((directory / "First.png").exists())
        self.assertEqual(Scene.instances, [])
    def test_dangling_output_symlink_is_not_overwritten(self):
        directory = self.root / "output"
        directory.mkdir()
        target = directory / "First.png"
        target.symlink_to(directory / "missing")
        with self.assertRaises(FileExistsError):
            self.render([First])
        self.assertTrue(target.is_symlink())
    def test_non_directory_ancestor_is_rejected(self):
        (self.root / "output").write_bytes(b"file")
        with self.assertRaises(NotADirectoryError):
            self.render([First])
        self.assertEqual(Scene.instances, [])
    def test_invalid_common_options_precede_scene_construction(self):
        for options in ({"fps": 0}, {"fps": True}, {"threads": 1.5}, {"resolution": (1,)},
                        {"resolution": (0, 4)}, {"resolution": (4, True)}, {"max_jobs": 65537},
                        {"format": []}, {"format": "webp"}, {"continue_on_error": 1}, {"on_result": 7}):
            with self.subTest(options=options), self.assertRaises((TypeError, ValueError)):
                self.render([First], **options)
        self.assertEqual(Scene.instances, [])
    def test_empty_and_non_batch_inputs_are_not_success(self):
        for scenes in ([], {}, "First", First, First(), None):
            with self.subTest(scenes=scenes), self.assertRaises((TypeError, ValueError)):
                self.render(scenes)
        self.assertFalse(BatchRenderResult(()).ok)
    def test_options_are_forwarded_per_job_without_changing_seed(self):
        report = self.render([RenderJob("one", First, {"random_seed": 7}), RenderJob("two", Second, {"random_seed": 9})],
                             resolution=(16, 8), fps=12, threads=3)
        for index, item in enumerate(report.outcomes):
            self.assertEqual(item.result.resolution, (16, 8))
            self.assertEqual(item.result.fps, 12)
            self.assertEqual(item.result.threads, 3)
            self.assertEqual(item.result.seed, (7, 9)[index])
    def test_planning_snapshots_top_level_kwargs(self):
        arguments = {"random_seed": 42}
        def observer(outcome):
            arguments["random_seed"] = 5
        report = self.render([First, RenderJob("two", Second, arguments)], on_result=observer)
        self.assertEqual(report.outcomes[1].result.seed, 42)
    def test_broken_exception_formatter_does_not_mask_failure(self):
        class BrokenError(Exception):
            def __str__(self):
                raise ValueError("bad formatter")
        class Failing(Scene):
            def run(self):
                raise BrokenError()
        report = self.render([Failing], continue_on_error=True)
        self.assertEqual(report.outcomes[0].error_type, "BrokenError")
        self.assertEqual(report.outcomes[0].message, "exception message could not be formatted")
    def test_error_messages_are_bounded(self):
        instance = First()
        instance.run_error = ValueError("x" * 10000)
        report = self.render([instance], continue_on_error=True)
        self.assertEqual(len(report.outcomes[0].message), 4096)
    def test_json_receipt_does_not_alias_native_provenance(self):
        report = self.render([First], format="mp4")
        Scene.instances[0]._render_invocations[0]["argv"].append("changed")
        dictionary = report.as_dict()
        dictionary["outcomes"][0]["result"]["ffmpeg_invocations"][0]["argv"].append("changed again")
        self.assertEqual(report.outcomes[0].result.ffmpeg_invocations[0]["argv"], ["ffmpeg", "input"])


if __name__ == "__main__":
    unittest.main()
