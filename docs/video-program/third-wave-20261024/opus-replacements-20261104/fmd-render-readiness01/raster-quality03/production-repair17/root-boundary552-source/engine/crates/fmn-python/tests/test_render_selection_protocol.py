"""Real selection/CLI/session owners; protocol sink, not a pixel oracle.

Native playback_selection.py separately checks exact decoded frames and WAV
samples. Here constructors, option parsing, ownership and receipts are tested.
"""
from __future__ import annotations

import contextlib
import importlib
import io
import json
from types import SimpleNamespace
import unittest

import test_console_rendering_protocol as protocol

PACKAGE, console = protocol.PACKAGE, protocol.console

selection = importlib.import_module(PACKAGE + ".render_selection")
rendering = importlib.import_module(PACKAGE + ".rendering")
batch = importlib.import_module(PACKAGE + ".batch_rendering")
batch_cli = importlib.import_module(PACKAGE + ".batch_cli")


class SelectionTests(unittest.TestCase):
    setUp = protocol.ConsoleRenderingProtocol.setUp
    invoke = protocol.ConsoleRenderingProtocol.invoke
    render = protocol.ConsoleRenderingProtocol.render

    def test_range_spellings_reach_run_without_constructor_kwargs(self):
        self.source.write_text("""from manimlib import Scene
class Hello(Scene):
    def __init__(self):
        super().__init__()
        self.original_skipping_status = False
    def construct(self):
        assert self.start_at_animation_number == 2
        assert self.end_at_animation_number == 4
        assert self.skip_animations
""")
        for i, flags in enumerate((("-n", "2,4"), ("--start_at_animation_number", "2,4"),
                                   ("--start_at_animation_number=2,4",), ("-n2,4",))):
            with self.subTest(flags=flags):
                code, report, _, _ = self.render(*flags, "--video_dir", str(self.root / str(i)))
                self.assertEqual(code, 0, report)
                self.assertEqual(report["animation_range"], [2, 4])
        self.assertEqual(len(self.native.instances), 4)

    def test_start_only_has_no_upper_limit(self):
        code, report, _, _ = self.render("-n", "3")
        self.assertEqual(code, 0, report)
        self.assertEqual(report["animation_range"], [3, None])
        self.assertIsNone(self.native.instances[0].end_at_animation_number)

    def test_invalid_and_repeated_ranges_fail_before_source_execution(self):
        self.source.write_text("raise AssertionError('must not execute')\n")
        invalid = ("", "-1", "1,-2", "4,4", "4,3", "1,2,3", "1,", ",2", "1.5", "1e3",
                   "true", "1_000", " 1", "１", "9" * 1000, str(1 << 63))
        for value in invalid:
            with self.subTest(value=value[:30]):
                code, report, _, _ = self.render("-n", value)
                self.assertEqual(code, 2, report)
                self.assertEqual(report["kind"], "usage-error")
        for flags in (("-n",), ("-n", "0", "-n", "1"),
                      ("-n0", "--start_at_animation_number=1")):
            self.assertEqual(self.render(*flags)[0], 2)
        self.assertEqual(self.native.instances, [])

    def test_still_alias_chooses_final_png(self):
        for flag in ("-s", "--skip_animations"):
            code, report, _, _ = self.render(flag, "--video_dir", str(self.root / flag[1:]))
            self.assertEqual(code, 0, report)
            self.assertEqual(report["format"], "png")
            self.assertEqual(self.native.instances[-1].arguments[1], "png")

    def test_still_and_range_combine_without_ignoring_explicit_format(self):
        code, report, _, _ = self.render("-s", "-n", "1,2", "--format=png")
        self.assertEqual(code, 0, report)
        self.assertEqual(report["format"], "png")
        self.assertEqual(report["animation_range"], [1, 2])
        self.source.write_text("raise AssertionError('must not execute')\n")
        for flags in (("-s", "--format", "y4m"), ("--format=gif", "-s"),
                      ("-s", "--skip_animations")):
            self.assertEqual(self.render(*flags)[0], 2)

    def test_native_option_values_are_never_stolen_as_playback_flags(self):
        for value in ("-n", "-s", "--start_at_animation_number=4,8"):
            code, report, _, _ = self.render("--video_dir", value)
            self.assertEqual(code, 0, report)
            self.assertNotIn("animation_range", report)
            self.assertEqual(report["format"], "png_sequence")
        code, report, _, _ = self.render("--video_dir", "--format", "-s")
        self.assertEqual(code, 0, report)
        self.assertEqual(report["format"], "png")

    def test_named_batches_apply_selection_to_each_original_scene_class(self):
        self.source.write_text("""from manimlib import Scene
class First(Scene):
    def __init__(self):
        super().__init__()
    def construct(self):
        assert self.start_at_animation_number == 1
        assert self.end_at_animation_number == 3
class Second(First):
    pass
""")
        code, report, _, _ = self.render("Second", "First", "-n", "1,3")
        self.assertEqual(code, 0, report)
        outcomes = report["batch"]["outcomes"]
        self.assertEqual([o["name"] for o in outcomes], ["Second", "First"])
        self.assertEqual([o["result"]["animation_range"] for o in outcomes], [[1, 3], [1, 3]])
        self.assertEqual([type(s).__name__ for s in self.native.instances], ["Second", "First"])

    def test_write_all_and_legacy_batch_owner_share_selection(self):
        code, report, _, _ = self.render("-a", "-s", "-n0,1")
        self.assertEqual(code, 0, report)
        self.assertEqual(report["batch"]["outcomes"][0]["result"]["animation_range"], [0, 1])
        out = io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(io.StringIO()):
            code = batch_cli.try_batch_cli(self.native, ["--robot", str(self.source), "--write_all",
                                                        "-n", "1,2", "--video_dir", str(self.root / "batch")])
        self.assertEqual(code, 0, out.getvalue())
        self.assertEqual(json.loads(out.getvalue())["batch"]["outcomes"][0]["result"]["animation_range"], [1, 2])

    def test_certification_refusal_is_not_bypassed_by_selection(self):
        self.assertEqual(self.render("-n", "1,2", "--reproducible")[0], 4)
        self.assertEqual(self.native.instances, [])

    def test_failed_start_never_changes_someone_elses_playback(self):
        scene = self.native.Scene()
        scene.skip_animations = False
        scene.start_at_animation_number = 7
        scene.start_error = RuntimeError("external generation")
        session = rendering.RenderSession(scene, self.root / "x.y4m", animation_range=(1, 3), _native=self.native)
        with self.assertRaisesRegex(RuntimeError, "external generation"):
            session.__enter__()
        self.assertEqual(scene.start_at_animation_number, 7)
        self.assertFalse(scene.skip_animations)
        self.assertEqual(scene.events, ["begin"])

    def test_owned_session_freezes_range_before_run_and_reports_it(self):
        scene = self.native.Scene()
        values = [2, 5]
        session = rendering.RenderSession(scene, self.root / "x.y4m", animation_range=values, _native=self.native)
        values[0] = 100
        with session:
            self.assertEqual(scene.start_at_animation_number, 2)
            self.assertEqual(scene.end_at_animation_number, 5)
            self.assertTrue(scene.skip_animations)
        self.assertEqual(session.result.as_dict()["animation_range"], [2, 5])

    def test_nested_owner_cannot_reconfigure_range(self):
        scene = self.native.Scene()
        with rendering.RenderSession(scene, self.root / "x.y4m", animation_range=(2, 5), _native=self.native):
            with self.assertRaisesRegex(RuntimeError, "already has"):
                with rendering.RenderSession(scene, self.root / "y.y4m", animation_range=(0, 1), _native=self.native):
                    self.fail("nested body must not execute")
            self.assertEqual(scene.start_at_animation_number, 2)
            self.assertTrue(scene.skip_animations)

    def test_range_validation_happens_before_programmatic_constructors(self):
        for invalid in ((True, 4), (1, False), (1., 3), (2, 2), (-1, None), (0, 1 << 64), "0,2"):
            with self.subTest(invalid=invalid):
                with self.assertRaises((TypeError, ValueError)):
                    rendering.render_scene(self.native.Scene, self.root / "x.y4m", animation_range=invalid)
                with self.assertRaises((TypeError, ValueError)):
                    batch.render_scenes({"Hello": self.native.Scene}, self.root / "jobs", animation_range=invalid)
        self.assertEqual(self.native.instances, [])

    def test_existing_skip_all_is_preserved_and_old_preroll_is_replaced(self):
        for original in (True, False):
            scene = SimpleNamespace(num_plays=0, original_skipping_status=original,
                                    skip_animations=True, start_at_animation_number=10)
            selection.apply_animation_range(scene, (0, None))
            self.assertEqual(scene.skip_animations, original)
            self.assertEqual(scene.start_at_animation_number, 0)
            self.assertIsNone(scene.end_at_animation_number)

    def test_failed_range_configuration_aborts_owned_output(self):
        scene = self.native.Scene()
        scene.num_plays = 1
        session = rendering.RenderSession(scene, self.root / "x.y4m", animation_range=(0, 2), _native=self.native)
        with self.assertRaisesRegex(ValueError, "no completed"):
            session.__enter__()
        self.assertEqual(scene.events, ["begin", "abort"])
        self.assertFalse(scene.active)
        self.assertIsNone(session.result)

    def test_programmatic_scene_render_and_imperative_session_both_select(self):
        rendering.install_scene_rendering(self.native)
        scene = self.native.Scene()
        receipt = scene.render(self.root / "a.y4m", animation_range=(3, None))
        self.assertEqual(receipt.animation_range, (3, None))
        scene = self.native.Scene()
        with rendering.render_session(scene, self.root / "b.y4m", animation_range=(0, 4)) as session:
            self.assertFalse(scene.skip_animations)
            self.assertEqual(scene.end_at_animation_number, 4)
        self.assertEqual(session.result.animation_range, (0, 4))

    def test_help_documents_exclusive_end_and_still_format(self):
        code, report, _, _ = self.invoke("--robot", "--help")
        self.assertEqual(code, 0)
        self.assertIn("exclusive END", report["help"])
        self.assertIn("final-state PNG", report["help"])


if __name__ == "__main__":
    unittest.main()
