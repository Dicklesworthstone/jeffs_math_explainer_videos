"""Wheel-level Scene.render and writer routing, with native boundary doubles."""
import ast
import pathlib
import sys
import types
import unittest
from unittest.mock import patch

from test_render_session_protocol import Scene as BaseScene, EndScene
from fmn_python.rendering import install_scene_rendering, render_scene


class Writer:
    write_to_movie = False
    save_last_frame = False
    subdivide_output = False
    open_file_upon_completion = False
    show_file_location_upon_completion = False
    png_mode = "RGBA"
    saturation = gamma = 1.0
    ffmpeg_bin = "ffmpeg"
    video_codec = "libx264"
    pixel_format = "yuv420p"

    def get_image_file_path(self):
        return "chosen/name.png"

    def get_movie_file_path(self):
        return "chosen/name.mov"

    def get_output_file_rootname(self):
        return "chosen/name"


class SceneRenderTests(unittest.TestCase):
    def setUp(self):
        class Scene(BaseScene):
            def __init__(self, **kwargs):
                super().__init__(**kwargs)
                self.file_writer = Writer()
            render = BaseScene.run
        self.native = types.SimpleNamespace(Scene=Scene, EndScene=EndScene, _CapabilityError=RuntimeError)
        self.old_run = Scene.run
        self.old_render = Scene.render
        install_scene_rendering(self.native)
        self.patch = patch.dict(sys.modules, {"manimlib": self.native})
        self.patch.start()
        self.addCleanup(self.patch.stop)
        self.scene = Scene()

    def test_default_render_publishes_png_sequence(self):
        result = self.scene.render(threads=1)
        self.assertEqual(result.destination, pathlib.Path("chosen/name/frames"))
        self.assertEqual(result.format, "png_sequence")
        self.assertIs(self.scene.render_result, result)

    def test_configured_movie_uses_writer_path_and_extension(self):
        self.scene.file_writer.write_to_movie = True
        result = self.scene.render()
        self.assertEqual(result.format, "mov")
        self.assertEqual(result.destination, pathlib.Path("chosen/name.mov"))
        self.assertTrue(result.ffmpeg_invocations)

    def test_configured_last_frame_uses_native_png(self):
        self.scene.file_writer.save_last_frame = True
        result = self.scene.render()
        self.assertEqual(result.format, "png")
        self.assertEqual(result.destination, pathlib.Path("chosen/name.png"))

    def test_explicit_format_overrides_writer_preference(self):
        self.scene.file_writer.write_to_movie = self.scene.file_writer.save_last_frame = True
        result = self.scene.render(format="gif")
        self.assertEqual(result.format, "gif")
        self.assertEqual(result.destination, pathlib.Path("chosen/name.gif"))

    def test_explicit_destination_selects_single_output(self):
        self.scene.file_writer.write_to_movie = self.scene.file_writer.save_last_frame = True
        result = self.scene.render("elsewhere.y4m", fps=4, resolution=(80, 60))
        self.assertEqual(result.destination, pathlib.Path("elsewhere.y4m"))
        self.assertEqual(result.fps, 4)
        self.assertEqual(result.resolution, (80, 60))

    def test_ambiguous_two_outputs_refuse_without_native_start(self):
        self.scene.file_writer.write_to_movie = self.scene.file_writer.save_last_frame = True
        with self.assertRaisesRegex(RuntimeError, "movie and a last-frame"):
            self.scene.render()
        self.assertEqual(self.scene.events, [])

    def test_run_remains_lifecycle_only_for_cli_and_construct_only(self):
        self.scene.file_writer.write_to_movie = True
        self.scene.run()
        self.assertIs(self.native.Scene.run, self.old_run)
        self.assertEqual(self.scene.events, [("run",)])

    def test_existing_cli_generation_is_not_opened_or_finalized_twice(self):
        self.scene._begin_native_output("cli.y4m", "y4m", 96, 54, 8, 1, 0)
        self.scene.run()
        self.scene._finish_render()
        self.assertEqual([x[0] for x in self.scene.events], ["begin", "run", "finish"])

    def test_render_during_external_generation_does_not_cancel_owner(self):
        self.scene._begin_native_output("cli.y4m", "y4m", 96, 54, 8, 1, 0)
        with self.assertRaisesRegex(RuntimeError, "external generation"):
            self.scene.render("nested.png")
        self.assertTrue(self.scene.active)
        self.assertNotIn(("abort",), self.scene.events)

    def test_installer_keeps_class_identity_and_user_subclass_overrides(self):
        original = self.native.Scene
        class Custom(original):
            def render(self):
                return "authored"
        install_scene_rendering(self.native)
        self.assertIs(self.native.Scene, original)
        self.assertEqual(Custom().render(), "authored")

    def test_reinstall_does_not_discard_later_authored_method(self):
        self.native.Scene.render = lambda self: "replacement"
        install_scene_rendering(self.native)
        self.assertEqual(self.scene.render(), "replacement")

    def test_render_calls_authored_run_once(self):
        calls = []
        class Custom(self.native.Scene):
            def run(self):
                calls.append(self)
                super().run()
        scene = Custom()
        scene.render("out.png")
        self.assertEqual(calls, [scene])

    def test_end_scene_finishes_but_keyboard_interrupt_cancels(self):
        self.scene.run_error = EndScene()
        self.assertIsNotNone(self.scene.render("out.png"))
        scene = self.native.Scene()
        scene.run_error = KeyboardInterrupt()
        with self.assertRaises(KeyboardInterrupt):
            scene.render("other.png")
        self.assertNotIn(("finish",), scene.events)
        self.assertFalse(hasattr(scene, "render_result"))

    def test_explicit_cancel_is_not_reported_as_success(self):
        class Custom(self.native.Scene):
            def run(self):
                self._fmn_owned_render_session.abort()
        for direct in (True, False):
            scene = Custom()
            with self.assertRaisesRegex(RuntimeError, "without publishing"):
                scene.render("cancel.png") if direct else render_scene(scene, "cancel.png")
            self.assertFalse(hasattr(scene, "render_result"))

    def test_unrouted_writer_controls_are_not_silently_ignored(self):
        for name, value, format in (
            ("subdivide_output", True, "mp4"),
            ("open_file_upon_completion", True, "png"),
            ("show_file_location_upon_completion", True, "gif"),
            ("png_mode", "RGB", "png"),
            ("gamma", 1.2, "y4m"), ("saturation", .5, "gif"),
        ):
            scene = self.native.Scene()
            setattr(scene.file_writer, name, value)
            expected = r"subdivide_output|audio subdivision" if name == "subdivide_output" else name
            with self.assertRaisesRegex(RuntimeError, expected):
                scene.render("output", format=format)
            self.assertEqual(scene.events, [])

    def test_color_transform_knobs_do_not_block_audio(self):
        self.scene.file_writer.gamma = 2
        result = self.scene.render("audio.wav")
        self.assertEqual(result.format, "wav")

    def test_public_render_scene_also_validates_writer_configuration(self):
        self.scene.file_writer.gamma = 2.0
        with self.assertRaisesRegex(RuntimeError, "gamma"):
            render_scene(self.scene, "out.mp4")
        self.assertEqual(self.scene.events, [])

    def test_negotiated_writer_options_reach_native_generation(self):
        self.scene.file_writer.video_codec = "qtrle"
        self.scene.file_writer.pixel_format = "bgra"
        self.scene.file_writer.ffmpeg_bin = "/custom/ffmpeg"
        result = render_scene(self.scene, "out.mov")
        self.assertEqual(result.format, "mov")
        self.assertEqual(self.scene.file_writer.pixel_format, "bgra")
        self.assertIn(("finish",), self.scene.events)

    def test_explicit_path_does_not_require_file_writer(self):
        del self.scene.file_writer
        with self.assertRaisesRegex(ValueError, "requires a destination"):
            self.scene.render()
        self.assertIsNotNone(self.scene.render("out.png"))

    def test_invalid_format_with_default_destination_is_refused(self):
        with self.assertRaises(ValueError):
            self.scene.render(format="jpeg")
        self.assertEqual(self.scene.events, [])

    def test_original_lifecycle_only_render_is_negative_control(self):
        self.old_render(self.scene)
        self.assertEqual(self.scene.events, [("run",)])
        self.assertFalse(hasattr(self.scene, "render_result"))

    def test_actual_wheel_initializer_preserves_exports_and_installs_render(self):
        path = pathlib.Path(__file__).resolve().parents[1] / "python/manimlib/__init__.py"
        source = path.read_text()
        tree = ast.parse(source)
        aliases = next(ast.literal_eval(node.value) for node in tree.body
                       if isinstance(node, ast.Assign)
                       and any(isinstance(target, ast.Name) and target.id == "_REFERENCE_CLASS_BY_RUST_HELPER"
                               for target in node.targets))
        package = types.ModuleType("manimlib")
        package.__path__ = []
        extension = types.ModuleType("manimlib.manimlib")
        extension.Scene, extension.EndScene = self.native.Scene, EndScene
        shapes = types.ModuleType("manimlib.protocol_shapes")
        for name in aliases.values():
            value = type(name, (), {})
            setattr(extension, name, value)
            setattr(shapes, name, value)
        for name in ("__version__", "__distribution__", "__franken_manim__", "__abi_policy__",
                     "__engine__", "__thread_policy__", "__reference_commit__"):
            setattr(extension, name, "test-native-boundary")
        extension._FMN_PORTAL_RUNTIME_STATE = "ready"
        extension._FMN_ANIMATION_SEMANTICS_INSTALLED = True
        authority = types.ModuleType("fmn_python.library_constructor_authority")
        authority.REFERENCE_CLASS_BY_RUST_HELPER = aliases
        authority.REFERENCE_MODULE_BY_RUST_HELPER = {key: shapes.__name__ for key in aliases}
        provenance = types.ModuleType("fmn_python.schema_provenance")
        provenance.SchemaProvenanceError = ValueError
        provenance.apply_schema_placeholder_provenance = lambda module: None
        modules = {module.__name__: module for module in (package, extension, shapes, authority, provenance)}
        # Playback has its own complete native-table fixture in test_animation_builder_playback.
        with patch.dict(sys.modules, modules), patch("fmn_python._ensure_exclusive_manimlib_namespace"), \
                patch("fmn_python.playback.install_scene_playback"), \
                patch("fmn_python.fading.install_fading"):
            exec(compile(source, str(path), "exec"), vars(package))
            self.assertIs(package.Scene, extension.Scene)
            self.assertIs(extension.Scene.run, self.old_run)
            result = package.Scene().render("installed.png")
            self.assertEqual(result.format, "png")
            self.assertEqual({key for key in vars(package) if not key.startswith("_")},
                             {key for key in vars(extension) if not key.startswith("_")})

    def test_wheel_initializer_installs_into_native_class_table(self):
        path = pathlib.Path(__file__).resolve().parents[1] / "python/manimlib/__init__.py"
        tree = ast.parse(path.read_text())
        calls = [node for node in ast.walk(tree) if isinstance(node, ast.Call)
                 and isinstance(node.func, ast.Name) and node.func.id == "_initialize_portal"]
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0].args[0].id, "_native")


if __name__ == "__main__":
    unittest.main()
