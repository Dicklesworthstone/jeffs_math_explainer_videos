"""Production parser/owners with a native sink double (no pixel claims)."""
import unittest
import test_console_rendering_protocol as fixtures


class VideoConsoleOptions(unittest.TestCase):
    invoke = fixtures.ConsoleRenderingProtocol.invoke
    render = fixtures.ConsoleRenderingProtocol.render

    def setUp(self):
        fixtures.ConsoleRenderingProtocol.setUp(self)
        original = self.native.Scene.__init__

        def construct(scene):
            original(scene)
            scene.camera.background_rgba = [0.2, 0.3, 0.4, 1.0]
            scene.file_writer.video_codec = "scene_codec"
            scene.file_writer.pixel_format = "scene_wire"
            scene.file_writer.ffmpeg_bin = "scene_ffmpeg"
            scene.file_writer.quiet = True
        self.native.Scene.__init__ = construct

    def test_transparent_png_preserves_rgb_and_camera_identity(self):
        code, report, _, _ = self.render("--format", "png", "--transparent")
        self.assertEqual(code, 0, report)
        scene, = self.native.instances
        self.assertEqual(scene.camera.background_rgba, [0.2, 0.3, 0.4, 0.0])
        self.assertIs(scene.camera.frame, scene.frame)
        self.assertEqual(scene.file_writer.video_codec, "scene_codec")

    def test_custom_video_settings_and_space_path_reach_the_same_scene(self):
        code, report, _, _ = self.render("--format=mov", "-t", "--vcodec=qtrle",
                                        "--pix_fmt", "bgra", "--ffmpeg_bin", "/encoder with spaces/ffmpeg")
        self.assertEqual(code, 0, report)
        scene, = self.native.instances
        self.assertEqual(scene.file_writer.video_codec, "qtrle")
        self.assertEqual(scene.file_writer.pixel_format, "bgra")
        self.assertEqual(scene.file_writer.ffmpeg_bin, "/encoder with spaces/ffmpeg")
        self.assertTrue(scene.file_writer.quiet)
        self.assertEqual(scene.camera.background_rgba[3], 0.0)

    def test_both_batch_jobs_receive_overrides_after_no_argument_construction(self):
        self.source.write_text("from manimlib import Scene\n"
                               "class First(Scene):\n"
                               "    def __init__(self):\n        super().__init__()\n"
                               "class Second(First):\n    pass\n")
        code, report, _, _ = self.render("Second", "First", "--format", "mov",
                                        "--transparent", "--vcodec", "qtrle", "--pix_fmt", "rgba")
        self.assertEqual(code, 0, report)
        self.assertEqual([type(scene).__name__ for scene in self.native.instances], ["Second", "First"])
        for scene in self.native.instances:
            self.assertEqual(scene.file_writer.video_codec, "qtrle")
            self.assertEqual(scene.file_writer.pixel_format, "rgba")
            self.assertEqual(scene.camera.background_rgba[3], 0.0)

    def test_write_all_and_still_selection_use_final_png_format(self):
        self.source.write_text("from manimlib import Scene\nclass A(Scene):\n    pass\nclass B(Scene):\n    pass\n")
        code, report, _, _ = self.render("--write_all", "-s", "-t")
        self.assertEqual(code, 0, report)
        self.assertEqual(len(self.native.instances), 2)
        for scene in self.native.instances:
            self.assertEqual(scene.arguments[1], "png")
            self.assertEqual(scene.camera.background_rgba[3], 0.0)

    def test_invalid_combinations_fail_before_loading_authored_source(self):
        self.source.write_text("raise AssertionError('source must not execute')\n")
        for arguments in (("--format", "mp4", "--transparent"),
                          ("--format", "y4m", "-t"),
                          ("--format", "gif", "-t"),
                          ("--format", "wav", "-t"),
                          ("--format", "png", "--vcodec", "qtrle"),
                          ("--format", "mov", "-t", "--pix_fmt", "nv12"),
                          ("--format", "mov", "-t", "--vcodec", "libx264"),
                          ("--format", "mov", "--vcodec="),
                          ("--format", "mov", "--ffmpeg_bin", ""),
                          ("--format", "mov", "--pix_fmt", "rgba\0"),
                          ("--transparent", "-t"),
                          ("--format", "mov", "--pix_fmt")):
            with self.subTest(arguments=arguments):
                code, report, _, _ = self.render(*arguments)
                self.assertEqual(code, 2, report)
                self.assertEqual(self.native.instances, [])

    def test_flag_like_values_are_not_scene_selectors_or_batch_switches(self):
        code, report, _, _ = self.render("--format", "mov", "--ffmpeg_bin", "--write_all")
        self.assertEqual(code, 0, report)
        self.assertEqual(len(self.native.instances), 1)
        self.assertEqual(self.native.instances[0].file_writer.ffmpeg_bin, "--write_all")

    def test_omitted_output_flags_leave_scene_defaults_unchanged(self):
        code, report, _, _ = self.render("--format", "mov")
        self.assertEqual(code, 0, report)
        scene, = self.native.instances
        self.assertEqual(scene.camera.background_rgba[3], 1.0)
        self.assertEqual(scene.file_writer.video_codec, "scene_codec")
        self.assertEqual(scene.file_writer.pixel_format, "scene_wire")
        self.assertEqual(scene.file_writer.ffmpeg_bin, "scene_ffmpeg")

    def test_relative_executable_is_frozen_before_source_changes_cwd(self):
        other = self.root / "other"
        other.mkdir()
        self.source.write_text("import os\nos.chdir(" + repr(str(other)) + ")\n"
                               "from manimlib import Scene\nclass Hello(Scene):\n    pass\n")
        code, report, _, _ = self.render("--format", "mov", "--ffmpeg_bin", "./tools/ffmpeg")
        self.assertEqual(code, 0, report)
        self.assertEqual(self.native.instances[0].file_writer.ffmpeg_bin,
                         str(self.root / "tools" / "ffmpeg"))

    def test_help_explains_alpha_and_wire_limitations(self):
        code, report, _, _ = self.invoke("--robot", "--help")
        self.assertEqual(code, 0)
        for text in ("--transparent", "--vcodec", "--pix_fmt", "--ffmpeg_bin", "HDR"):
            self.assertIn(text, report["help"])


if __name__ == "__main__":
    unittest.main()
