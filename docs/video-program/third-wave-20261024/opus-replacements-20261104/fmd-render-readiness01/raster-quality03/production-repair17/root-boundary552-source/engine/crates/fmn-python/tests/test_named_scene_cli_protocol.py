"""Named scene selection over real CLI/session/batch code and a fake sink."""
from pathlib import Path
import unittest

from test_console_rendering_protocol import ConsoleRenderingProtocol


class NamedSceneCliProtocol(unittest.TestCase):
    invoke = ConsoleRenderingProtocol.invoke
    render = ConsoleRenderingProtocol.render

    def setUp(self):
        ConsoleRenderingProtocol.setUp(self)
        self.source.write_text(
            "from manimlib import Scene, events\nevents.append(('imported',))\n"
            "class C(Scene): pass\nclass A(Scene): pass\nclass B(Scene): pass\n"
        )

    def names(self):
        return [type(scene).__name__ for scene in self.native.instances]

    def test_explicit_names_render_in_requested_order(self):
        code, report, _, _ = self.render("C", "A")
        self.assertEqual(code, 0)
        self.assertEqual(self.names(), ["C", "A"])
        self.assertEqual([row["name"] for row in report["batch"]["outcomes"]], ["C", "A"])
        self.assertEqual(self.native.events.count(("imported",)), 1)
        self.assertEqual(report["batch"]["counts"]["succeeded"], 2)

    def test_short_write_all_alias_sorts_local_scenes(self):
        code, report, _, _ = self.render("-a")
        self.assertEqual(code, 0)
        self.assertEqual(self.names(), ["A", "B", "C"])
        self.assertEqual(report["batch"]["counts"]["succeeded"], 3)

    def test_long_write_all_keeps_existing_contract(self):
        self.assertEqual(self.render("--write_all")[0], 0)
        self.assertEqual(self.names(), ["A", "B", "C"])

    def test_all_unknown_names_are_checked_before_any_constructor(self):
        code, report, _, _ = self.render("A", "Missing", "OtherMissing")
        self.assertEqual(code, 5)
        self.assertIn("Missing", report["message"])
        self.assertIn("OtherMissing", report["message"])
        self.assertEqual(self.names(), [])
        self.assertFalse((self.root / "media").exists())

    def test_duplicate_and_filesystem_aliases_refuse_before_source(self):
        self.source.write_text("raise AssertionError('must not load')\n")
        for names in (("A", "A"), ("A", "a"), ("con", "A"), ("A", "../B")):
            with self.subTest(names=names):
                code, report, _, _ = self.render(*names)
                self.assertEqual(code, 2)
                self.assertEqual(report["kind"], "usage-error")
        self.assertEqual(self.names(), [])

    def test_named_keep_going_retains_successes_and_reports_failure(self):
        self.source.write_text("from manimlib import Scene\nclass A(Scene): pass\nclass B(Scene):\n    def construct(self): raise ValueError('B failed')\nclass C(Scene): pass\n")
        code, report, _, _ = self.render("A", "B", "C", "--keep-going", "--format", "png")
        self.assertEqual(code, 5)
        self.assertEqual(self.names(), ["A", "B", "C"])
        self.assertEqual(report["batch"]["counts"], dict(succeeded=2, failed=1, cancelled=0, not_run=0))
        outcomes = report["batch"]["outcomes"]
        self.assertTrue(Path(outcomes[0]["destination"]).exists())
        self.assertFalse(Path(outcomes[1]["destination"]).exists())
        self.assertTrue(Path(outcomes[2]["destination"]).exists())

    def test_named_fail_fast_never_starts_later_scenes(self):
        self.source.write_text("from manimlib import Scene\nclass A(Scene): pass\nclass B(Scene):\n    def construct(self): raise ValueError('B failed')\nclass C(Scene): pass\n")
        code, report, _, _ = self.render("A", "B", "C")
        self.assertEqual(code, 5)
        self.assertEqual(self.names(), ["A", "B"])
        self.assertEqual(report["batch"]["counts"], dict(succeeded=1, failed=1, cancelled=0, not_run=1))

    def test_cancellation_stops_even_with_keep_going_and_preserves_receipts(self):
        self.source.write_text("from manimlib import Scene\nclass A(Scene): pass\nclass B(Scene):\n    def construct(self): raise KeyboardInterrupt()\nclass C(Scene): pass\n")
        code, report, _, _ = self.render("A", "B", "C", "--keep-going")
        self.assertEqual(code, 130)
        self.assertEqual(report["kind"], "render-batch-interrupted")
        self.assertEqual(report["batch"]["counts"], dict(succeeded=1, failed=0, cancelled=1, not_run=1))
        self.assertTrue(self.native.instances[0].published)
        self.assertEqual(self.native.instances[1].events[-1], "abort")

    def test_system_exit_is_interruption_not_batch_success(self):
        self.source.write_text("from manimlib import Scene\nclass A(Scene):\n    def construct(self): raise SystemExit(0)\nclass B(Scene): pass\n")
        code, report, _, _ = self.render("A", "B", "--keep-going")
        self.assertEqual(code, 5)
        self.assertEqual(report["batch"]["counts"]["cancelled"], 1)
        self.assertEqual(self.names(), ["A"])

    def test_output_root_and_independent_sessions(self):
        code, report, _, _ = self.render("B", "A", "--format", "gif", "--video_dir", "batch")
        self.assertEqual(code, 0)
        rows = report["batch"]["outcomes"]
        self.assertEqual([Path(row["destination"]) for row in rows], [self.root / "batch/B.gif", self.root / "batch/A.gif"])
        self.assertIsNot(self.native.instances[0].frame, self.native.instances[1].frame)
        for scene in self.native.instances:
            self.assertEqual(scene.arguments[2:6], (96, 54, 8, 4))
            self.assertNotIn("_fmn_owned_render_session", vars(scene))

    def test_single_scene_explicit_destination_is_not_reinterpreted_as_root(self):
        code, report, _, _ = self.render("A", "--format", "png", "--video_dir", "single.png")
        self.assertEqual(code, 0)
        self.assertEqual(report["kind"], "render")
        self.assertEqual(Path(report["destination"]), self.root / "single.png")

    def test_write_all_and_named_selection_are_mutually_exclusive(self):
        self.source.write_text("raise AssertionError('must not load')\n")
        for args in (("-a", "A"), ("--write_all", "A", "B"), ("-a", "--write_all")):
            self.assertEqual(self.render(*args)[0], 2)

    def test_keep_going_needs_multiple_names_or_write_all(self):
        self.assertEqual(self.render("A", "--keep-going")[0], 2)
        self.assertEqual(self.names(), [])

    def test_missing_option_value_does_not_consume_source_during_normalization(self):
        code, report, _, _ = self.render("A", "B", "--threads")
        self.assertEqual(code, 2)
        self.assertIn("requires a value", report["message"])
        self.assertEqual(self.native.events, [])

    def test_native_capability_refusals_still_precede_source(self):
        self.source.write_text("raise AssertionError('must not load')\n")
        for flag in ("--reproducible", "--autoreload", "-o"):
            self.assertEqual(self.render("A", "B", flag)[0], 4)

    def test_switch_looking_option_value_remains_data(self):
        code, report, _, _ = self.render("A", "B", "--video_dir", "--write_all")
        self.assertEqual(code, 0)
        self.assertEqual(self.names(), ["A", "B"])
        self.assertEqual(Path(report["destination"]), self.root / "--write_all")

    def test_batch_paths_are_frozen_before_source_chdir(self):
        elsewhere = self.root / "elsewhere"
        elsewhere.mkdir()
        self.source.write_text(f"from manimlib import Scene\nimport os\nos.chdir({str(elsewhere)!r})\nclass A(Scene): pass\nclass B(Scene): pass\n")
        code, report, _, _ = self.render("B", "A", "--video_dir", "batch", "--format", "png")
        self.assertEqual(code, 0)
        self.assertEqual(Path(report["destination"]), self.root / "batch")

    def test_existing_later_destination_refuses_before_first_scene(self):
        output = self.root / "batch"
        output.mkdir()
        (output / "B.png").write_bytes(b"preserved")
        code, report, _, _ = self.render("A", "B", "--video_dir", str(output), "--format", "png")
        self.assertEqual(code, 6)
        self.assertEqual(self.names(), [])
        self.assertFalse((output / "A.png").exists())
        self.assertEqual((output / "B.png").read_bytes(), b"preserved")

    def test_help_explains_named_order_and_batch_destination(self):
        code, report, _, _ = self.invoke("--help", "--robot")
        self.assertEqual(code, 0)
        self.assertIn("SCENE [SCENE ...]", report["help"])
        self.assertIn("command-line order", report["help"])

    def test_selection_budget_refuses_before_module_execution(self):
        self.source.write_text("raise AssertionError('must not load')\n")
        code, report, _, _ = self.render(*(f"Scene{i}" for i in range(1025)))
        self.assertEqual(code, 2)
        self.assertIn("1024", report["message"])
        self.assertEqual(self.names(), [])

    def test_robot_batch_stdout_is_one_receipt(self):
        self.source.write_text("from manimlib import Scene\nprint('module chatter')\nclass A(Scene):\n    def construct(self): print('A chatter')\nclass B(Scene):\n    def construct(self): print('B chatter')\n")
        code, report, out, err = self.render("B", "A")
        self.assertEqual(code, 0)
        self.assertEqual(len(out.splitlines()), 1)
        self.assertEqual(report["kind"], "render-batch")
        self.assertIn("module chatter", err)
        self.assertIn("A chatter", err)
        self.assertIn("B chatter", err)


if __name__ == "__main__":
    unittest.main()
