"""Installed native SceneProject acceptance; no renderer or scene doubles."""
from __future__ import annotations

from pathlib import Path
import sys
import tempfile
import unittest

import manimlib as m
import numpy as np

from fmn_python import SceneProject
from fmn_python.scene_loading import SceneSource


class SceneProjectAcceptance(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="fmn-project-native-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.path = self.root / "scene.py"
        self.helper = self.root / "project_native_helper.py"
        self.helper.write_text("OFFSET = -1\n")

    def write_scene(self, tail=""):
        self.path.write_text(
            "from manimlib import *\n"
            "class Demo(Scene):\n"
            "    def setup(self):\n"
            "        self.events = ['setup']\n"
            "        self.camera.reset_pixel_shape(160, 90)\n"
            "    def construct(self):\n"
            "        from project_native_helper import OFFSET\n"
            "        self.item = Square(side_length=1, fill_opacity=1, stroke_width=0, color=BLUE)\n"
            "        self.add(self.item)\n"
            "        self.play(self.item.animate.shift(OFFSET * RIGHT), run_time=.125)\n"
            "        self.events.append('construct')\n" + tail +
            "    def tear_down(self):\n        self.events.append('tear_down')\n"
        )

    def test_rebuild_runs_real_lifecycle_and_changes_native_preview(self):
        self.write_scene()
        with SceneProject(self.path, "Demo") as project:
            old, image = project.scene, project.preview
            points = old.item.get_points().copy()
            before = image.pixels()
            self.assertEqual(old.events, ["setup", "construct", "tear_down"])
            self.helper.write_text("OFFSET = 1\n")
            new = project.rebuild(if_changed=True)
            self.assertIsNot(new, old)
            self.assertEqual(new.events, old.events)
            self.assertEqual(new.time, old.time)
            self.assertEqual(project.generation, 2)
            np.testing.assert_allclose(new.item.get_center(), m.RIGHT, atol=2e-5)
            np.testing.assert_array_equal(old.item.get_points(), points)
            self.assertEqual(image.pixels(), before)
            self.assertNotEqual(project.preview.pixels(), before)
            pixels = np.frombuffer(project.preview.pixels(), dtype=np.uint8).reshape(90, 160, 4)
            self.assertTrue(np.any(pixels[:, :, :3]))
            self.assertTrue(project._repr_png_().startswith(b"\x89PNG\r\n\x1a\n"))

    def test_failed_construction_preserves_native_handles_clock_checkpoint_and_imports(self):
        self.write_scene()
        with SceneProject(self.path, "Demo") as project:
            old, module, preview = project.scene, project.module, project.preview
            state, points, clock = old.get_state(), old.item.get_points().copy(), old.time
            helper = sys.modules["project_native_helper"]
            self.helper.write_text("OFFSET = 2\n")
            self.write_scene("        raise ValueError('candidate failed after animation')\n")
            with self.assertRaisesRegex(ValueError, "candidate failed"):
                project.rebuild()
            self.assertIs(project.scene, old)
            self.assertIs(project.module, module)
            self.assertIs(project.preview, preview)
            self.assertIs(sys.modules["project_native_helper"], helper)
            self.assertEqual(old.time, clock)
            np.testing.assert_array_equal(old.item.get_points(), points)
            old.item.shift(m.UP)
            old.restore_state(state)
            np.testing.assert_array_equal(old.item.get_points(), points)
            self.write_scene()
            project.rebuild()
            np.testing.assert_allclose(project.scene.item.get_center(), 2 * m.RIGHT, atol=2e-5)

    def test_unchanged_rebuild_and_force_reset_have_distinct_meanings(self):
        self.write_scene()
        with SceneProject(self.path, "Demo") as project:
            scene, preview = project.scene, project.preview
            scene.item.shift(m.UP)
            self.assertIs(project.rebuild(if_changed=True), scene)
            self.assertIs(project.preview, preview)
            new = project.rebuild()
            self.assertIsNot(new, scene)
            np.testing.assert_allclose(new.item.get_center(), m.LEFT, atol=2e-5)
            np.testing.assert_allclose(scene.item.get_center(), m.LEFT + m.UP, atol=2e-5)

    def test_existing_native_source_scope_and_no_capture(self):
        self.write_scene()
        with SceneSource(self.path, m.Scene) as source:
            with SceneProject(source, "Demo", capture=False) as project:
                self.assertIsNone(project.preview)
                self.helper.write_text("OFFSET = 0\n")
                project.rebuild()
                np.testing.assert_allclose(project.scene.item.get_center(), m.ORIGIN, atol=2e-5)
            self.assertTrue(source._active)
            self.assertIsNotNone(source.module.Demo)

    def test_initial_error_releases_native_and_import_ownership_for_retry(self):
        self.write_scene("        raise ValueError('bad first build')\n")
        with self.assertRaisesRegex(ValueError, "bad first build"):
            with SceneProject(self.path, "Demo"):
                pass
        self.assertNotIn("project_native_helper", sys.modules)
        self.write_scene()
        with SceneProject(self.path, "Demo") as project:
            self.assertEqual(project.generation, 1)
            self.assertGreater(project.scene.time, 0)


suite = unittest.defaultTestLoader.loadTestsFromTestCase(SceneProjectAcceptance)
if suite.countTestCases() != 5:
    raise AssertionError("scene project acceptance selected the wrong test count")
result = unittest.TextTestRunner(verbosity=2).run(suite)
if not result.wasSuccessful():
    raise AssertionError("native SceneProject acceptance failed")
