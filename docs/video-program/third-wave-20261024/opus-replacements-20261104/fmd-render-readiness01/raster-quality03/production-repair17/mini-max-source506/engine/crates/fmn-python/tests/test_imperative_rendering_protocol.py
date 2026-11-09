"""Exercise Scene.render_session's actual binding and generation ownership.

The native boundary double cannot certify pixels; svg_output.py exercises this
same method using native geometry, animation, SVG bytes and publication sinks.
"""
import pathlib
import sys
import types
import unittest
from unittest.mock import patch

from test_render_session_protocol import Camera, EndScene, Scene as BaseScene
from fmn_python.rendering import RenderSession, install_scene_rendering


class Writer:
    write_to_movie = False
    save_last_frame = True
    subdivide_output = False
    open_file_upon_completion = False
    show_file_location_upon_completion = False
    saturation = gamma = 1.0
    png_mode = "RGBA"

    def get_image_file_path(self):
        return "configured/still.png"

    def get_movie_file_path(self):
        return "configured/movie.mov"

    def get_output_file_rootname(self):
        return "configured/scene"


def make_native():
    class Scene(BaseScene):
        def __init__(self):
            super().__init__()
            self.file_writer = Writer()
    module = types.ModuleType("manimlib")
    module.Scene, module.EndScene = Scene, EndScene
    install_scene_rendering(module)
    return module


class ImperativeRenderingTests(unittest.TestCase):
    def setUp(self):
        self.native = make_native()
        self.scene = self.native.Scene()

    def test_context_creation_does_not_start_or_run_the_scene(self):
        session = self.scene.render_session()
        self.assertIsInstance(session, RenderSession)
        self.assertEqual(session.destination, pathlib.Path("configured/still.png"))
        self.assertEqual(self.scene.events, [])
        with session:
            self.scene.events.append(("authored",))
        self.assertEqual([event[0] for event in self.scene.events], ["begin", "authored", "finish"])
        self.assertEqual(session.result.format, "png")
        self.assertNotIn("_fmn_owned_render_session", vars(self.scene))

    def test_all_output_options_reach_the_existing_native_generation(self):
        self.scene.random_seed = 41
        frame = self.scene.camera.frame
        with self.scene.render_session("animation.y4m", resolution=(128, 72), fps=12, threads=3) as session:
            self.assertEqual(self.scene.request, ("animation.y4m", "y4m", 128, 72, 12, 3, 41, False))
            self.assertIs(self.scene.camera.frame, frame)
        self.assertEqual(session.result.seed, 41)
        self.assertEqual(session.result.threads, 3)
        self.assertIs(session.finish(), session.result)
        self.assertEqual(self.scene.events.count(("finish",)), 1)

    def test_explicit_path_needs_no_writer_and_infers_sequence(self):
        del self.scene.file_writer
        with self.scene.render_session("frames") as session:
            pass
        self.assertEqual(session.result.format, "png_sequence")

    def test_explicit_format_uses_writer_root_without_movie_ambiguity(self):
        self.scene.file_writer.write_to_movie = True
        with self.scene.render_session(format="gif") as session:
            pass
        self.assertEqual(session.result.destination, pathlib.Path("configured/scene.gif"))
        self.assertEqual(session.result.format, "gif")

    def test_ambiguous_outputs_fail_before_acquiring_generation(self):
        self.scene.file_writer.write_to_movie = True
        with self.assertRaisesRegex(RuntimeError, "movie and a last-frame"):
            self.scene.render_session()
        self.assertEqual(self.scene.events, [])

    def test_binding_keeps_its_native_identity_despite_module_replacement(self):
        foreign = make_native()
        with patch.dict(sys.modules, {"manimlib": foreign}):
            with self.scene.render_session("first.png") as first:
                pass
            with foreign.Scene().render_session("second.png") as second:
                pass
        self.assertIs(first._native, self.native)
        self.assertIs(second._native, foreign)

    def test_nested_owner_refusal_does_not_cancel_outer(self):
        with self.scene.render_session("outer.png") as outer:
            with self.assertRaisesRegex(RuntimeError, "already has"):
                with self.scene.render_session("inner.png"):
                    self.fail("nested generation must not open")
            self.assertTrue(self.scene.active)
        self.assertIsNotNone(outer.result)
        self.assertNotIn(("abort",), self.scene.events)

    def test_exception_and_keyboard_interrupt_cancel_without_publication(self):
        for error in (RuntimeError("authored failure"), KeyboardInterrupt()):
            with self.subTest(error=type(error)):
                scene = self.native.Scene()
                session = scene.render_session("failed.png")
                with self.assertRaises(type(error)) as raised:
                    with session:
                        raise error
                self.assertIs(raised.exception, error)
                self.assertIsNone(session.result)
                self.assertEqual([event[0] for event in scene.events], ["begin", "abort"])
                self.assertNotIn("_fmn_owned_render_session", vars(scene))

    def test_foreign_generation_survives_failed_context_entry(self):
        self.scene.active = True
        with self.assertRaisesRegex(RuntimeError, "external generation"):
            with self.scene.render_session("unowned.png"):
                pass
        self.assertTrue(self.scene.active)
        self.assertNotIn(("abort",), self.scene.events)

    def test_native_publication_failure_retains_no_success_receipt(self):
        self.scene.finish_error = OSError("publication failed")
        session = self.scene.render_session("failed.png")
        with self.assertRaisesRegex(OSError, "publication failed"):
            with session:
                pass
        self.assertIsNone(session.result)
        self.assertEqual(self.scene.events[-1], ("abort",))

    def test_binding_has_method_provenance_and_reinstall_preserves_overrides(self):
        method = self.native.Scene.render_session
        self.assertEqual(method.__name__, "render_session")
        self.assertEqual(method.__qualname__, self.native.Scene.__qualname__ + ".render_session")
        self.assertEqual(method.__module__, self.native.Scene.__module__)
        install_scene_rendering(self.native)
        self.assertIs(self.native.Scene.render_session, method)
        self.native.Scene.render_session = lambda self: "authored"
        install_scene_rendering(self.native)
        self.assertEqual(self.scene.render_session(), "authored")

    def test_subclass_overrides_are_never_replaced(self):
        class Custom(self.native.Scene):
            def render_session(self):
                return "custom"
        install_scene_rendering(self.native)
        self.assertEqual(Custom().render_session(), "custom")


if __name__ == "__main__":
    unittest.main()
