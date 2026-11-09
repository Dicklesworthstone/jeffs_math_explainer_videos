"""Scene.render and its imperative context honor native writer subdivision."""
from pathlib import Path
import tempfile
import traceback
import unittest

import manimlib as m
from fmn_python import RenderResult, SubdividedRenderResult, render_scene


class ConfiguredSubdivision(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='fmn configured clips ')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.calls = []
        calls = self.calls

        class Motion(m.Scene):
            default_camera_config = dict(resolution=(96, 54), fps=8)
            def setup(self):
                calls.append('setup')
            def construct(self):
                calls.append('construct')
                self.square = m.Square(fill_color=m.WHITE, fill_opacity=1)
                self.add(self.square)
                self.play(self.square.animate.shift(m.RIGHT), run_time=.25, rate_func=m.linear)
                self.wait(.125)
            def tear_down(self):
                calls.append('tear_down')
        self.Motion = Motion

    def scene(self, **writer):
        config = dict(subdivide_output=True)
        config.update(writer)
        return self.Motion(file_writer_config=config)

    def assert_y4m(self, result, counts=(2, 1)):
        self.assertIsInstance(result, SubdividedRenderResult)
        self.assertTrue(result.completed)
        self.assertEqual([s.render.frame_count for s in result.segments], list(counts))
        for s in result.segments:
            self.assertTrue(s.render.destination.read_bytes().startswith(
                b'YUV4MPEG2 W96 H54 F8:1 Ip A1:1 C420mpeg2\n'))

    def test_scene_render_runs_once_and_returns_actual_collection(self):
        scene = self.scene()
        frame = scene.frame
        result = scene.render(self.root / 'clips', format='y4m', threads=1)
        self.assert_y4m(result)
        self.assertIs(scene.render_result, result)
        self.assertEqual(self.calls, ['setup', 'construct', 'tear_down'])
        self.assertIs(scene.frame, frame)
        self.assertIs(scene.mobjects[0], scene.square)
        self.assertEqual(m._portal_scene_clock(scene), (8, 3))

    def test_writer_movie_path_and_flags_select_the_configured_collection(self):
        scene = self.scene(write_to_movie=True, movie_file_extension='.y4m',
                           output_directory=str(self.root), file_name='lesson')
        result = scene.render(threads=1)
        self.assert_y4m(result)
        self.assertEqual(result.destination, self.root / 'lesson/clips')
        self.assertTrue(scene.file_writer.subdivide_output)
        self.assertTrue(scene.file_writer.write_to_movie)
        self.assertFalse((self.root / 'lesson.y4m').exists())

    def test_imperative_method_does_not_run_lifecycle_or_acquire_before_entry(self):
        scene = self.scene()
        context = scene.render_session(self.root / 'imperative', format='y4m', threads=1)
        self.assertFalse(context.destination.exists())
        self.assertNotIn('_fmn_subdivision_session', vars(scene))
        with context:
            scene.add(m.Square(fill_opacity=1))
            scene.wait(.25)
        self.assert_y4m(context.result, (2,))
        self.assertEqual(self.calls, [])
        self.assertNotIn('_fmn_subdivision_session', vars(scene))

    def test_render_range_keeps_original_segment_indices(self):
        scene = self.scene()
        result = scene.render(self.root / 'selected', format='y4m', threads=1, animation_range=(1, 2))
        self.assert_y4m(result, (1,))
        self.assertEqual([s.play_index for s in result.segments], [1])
        self.assertEqual(self.calls, ['setup', 'construct', 'tear_down'])

    def test_failed_scene_render_preserves_completed_clips_in_primary_exception(self):
        scene = self.scene()
        def fail():
            scene.wait(.125)
            raise ValueError('authored failure')
        scene.construct = fail
        try:
            scene.render(self.root / 'partial', format='y4m', threads=1)
        except ValueError as error:
            self.assertEqual(str(error), 'authored failure')
            result = error.render_subdivision_result
            traceback.clear_frames(error.__traceback__)
            error.__traceback__ = None
        else:
            self.fail('authored failure swallowed')
        self.assertFalse(result.completed)
        self.assertEqual(result.frame_count, 1)
        self.assertEqual(len(result.segments), 1)
        self.assertTrue(result.segments[0].render.destination.exists())
        self.assertNotIn('_fmn_owned_render_session', vars(scene))
        self.assertNotIn('_fmn_subdivision_session', vars(scene))

    def test_default_nonmovie_preference_is_per_segment_png_sequences(self):
        scene = self.scene(output_directory=str(self.root), file_name='frames')
        result = scene.render(threads=1)
        self.assertIsInstance(result, SubdividedRenderResult)
        self.assertEqual(result.destination, self.root / 'frames/clips')
        self.assertEqual([len(list(s.render.destination.glob('*.png'))) for s in result.segments], [2, 1])

    def test_conflicting_still_preference_refuses_before_lifecycle_and_explicit_clip_selects_one(self):
        scene = self.scene(write_to_movie=True, save_last_frame=True)
        with self.assertRaises(m._CapabilityError):
            scene.render(self.root / 'conflict', threads=1)
        self.assertEqual(self.calls, [])
        self.assertFalse((self.root / 'conflict').exists())
        result = scene.render(self.root / 'chosen', format='y4m', threads=1)
        self.assert_y4m(result)

    def test_explicit_suffix_selects_clips_but_cannot_silently_convert_a_still(self):
        scene = self.scene()
        with self.assertRaises(ValueError):
            scene.render(self.root / 'still.png', threads=1)
        self.assertEqual(self.calls, [])
        result = scene.render(self.root / 'suffix.y4m', threads=1)
        self.assert_y4m(result)
        self.assertTrue((self.root / 'suffix.y4m').is_dir())

    def test_configured_subdivision_refuses_certification_without_running_or_publishing(self):
        scene = self.scene()
        for method in (scene.render, scene.render_session):
            with self.assertRaisesRegex(m._CapabilityError, 'subdivided.*certify'):
                method(self.root / 'certified', format='y4m', reproducible=True,
                       sources={'scene.py': b'pass'}, threads=1)
        self.assertEqual(self.calls, [])
        self.assertFalse((self.root / 'certified').exists())
        self.assertEqual(m._portal_scene_clock(scene), (30, 0))

    def test_ordinary_scene_methods_and_explicit_flat_api_keep_their_contract(self):
        ordinary = self.Motion()
        receipt = ordinary.render(self.root / 'ordinary.y4m', threads=1)
        self.assertIsInstance(receipt, RenderResult)
        self.assertTrue(receipt.destination.is_file())
        self.assertEqual(receipt.frame_count, 3)
        with self.assertRaises(m._CapabilityError):
            render_scene(self.scene(), self.root / 'explicit.y4m', threads=1)
        self.assertFalse((self.root / 'explicit.y4m').exists())


suite = unittest.defaultTestLoader.loadTestsFromTestCase(ConfiguredSubdivision)
result = unittest.TextTestRunner(verbosity=2).run(suite)
assert result.wasSuccessful(), 'configured native subdivision acceptance failed'
