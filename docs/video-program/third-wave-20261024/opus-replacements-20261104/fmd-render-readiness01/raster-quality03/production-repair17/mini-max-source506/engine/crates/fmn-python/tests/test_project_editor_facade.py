"""Public package forwarding into the production IPython scene editor.

Reuse explicit native-storage doubles from test_project_editor; IPython's
activation, project reconstruction and production adapters execute normally.
"""
from types import ModuleType
import unittest
from unittest.mock import patch

import test_project_editor as fixture
from fmn_python.project_editor import install_scene_project_editor
from fmn_python.scene_project import SceneProject


class ProjectEditorFacadeTests(unittest.TestCase):
    setUp = fixture.ProjectEditorTests.setUp
    write = fixture.ProjectEditorTests.write
    source = fixture.ProjectEditorTests.source
    edit = fixture.ProjectEditorTests.edit
    cell = fixture.ProjectEditorTests.cell

    def facade(self):
        native = self.native
        facade = ModuleType("forwarding_manimlib")
        facade.__getattr__ = lambda name: getattr(native, name)
        return facade

    @unittest.skipUnless(getattr(fixture, "HAVE_IPYTHON", False), "IPython is not installed")
    def test_public_package_enters_and_reloads_the_editor(self):
        path, facade = self.source(), self.facade()
        self.assertNotIn("_FMN_SCENE_PROJECT_EDITOR_INSTALLED", vars(facade))
        with SceneProject(path, "Demo", _native=facade) as project:
            old = project.scene
            def interact(shell):
                self.source(9)
                self.cell(shell, "reload()")
                self.assertIsNot(project.scene, old)
                self.assertEqual(project.scene.mobjects, [9])
                self.assertIs(shell.user_ns["self"], project.scene)
            self.edit(project, interact)
            self.assertIsNone(project._editor)

    def test_installing_through_public_package_is_idempotent(self):
        cls = self.native.InteractiveSceneEmbed
        before = (cls.launch, cls.reload_scene, cls.get_shortcuts)
        facade = self.facade()
        install_scene_project_editor(facade)
        after = (cls.launch, cls.reload_scene, cls.get_shortcuts)
        self.assertTrue(all(a is b for a, b in zip(before, after)))
        self.assertNotIn("_FMN_SCENE_PROJECT_EDITOR_INSTALLED", vars(facade))

    def test_failed_initialization_still_refuses_before_launch(self):
        with SceneProject(self.source(), "Demo", _native=self.facade()) as project:
            self.native._FMN_SCENE_PROJECT_EDITOR_INSTALLED = False
            with patch.object(self.native.InteractiveSceneEmbed, "launch",
                              side_effect=AssertionError("must not launch")):
                with self.assertRaisesRegex(ImportError, "not installed"):
                    project.edit()
            self.assertIsNone(project._editor)


if __name__ == "__main__":
    unittest.main()
