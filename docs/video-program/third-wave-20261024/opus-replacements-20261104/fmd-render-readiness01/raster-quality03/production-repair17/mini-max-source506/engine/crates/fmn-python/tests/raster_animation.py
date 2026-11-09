"""Real raster animations through Choreo, native materials and frame publication."""
from __future__ import annotations

import copy
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest

import numpy as np
import manimlib as m
from fmn_python.raster_animation import RasterTransition


def pixels(color, shape=(2, 3)):
    return np.full((*shape, 4), color, dtype=np.uint8)


RED = (255, 0, 0, 255)
GREEN = (0, 255, 0, 255)
BLUE = (0, 0, 255, 255)
PURPLE = (188, 0, 188, 255)
CYAN = (0, 188, 188, 255)


def mesh():
    return SimpleNamespace(vertices=np.array([[-1,-1,0],[1,-1,0],[1,1,0],[-1,1,0]]),
                           faces=np.array([[0,1,2],[0,2,3]]),
                           visual=SimpleNamespace(uv=np.array([[0,0],[1,0],[1,1],[0,1]])))


def capture(obj):
    return m.Camera(resolution=(48, 32), background_opacity=0).capture_snapshot(obj)


class RasterAnimationTests(unittest.TestCase):
    def setUp(self):
        self.obj = m.ImageMobject(pixels(RED), height=2)

    def animation(self, obj=None, end=BLUE, **kwargs):
        return RasterTransition(self.obj if obj is None else obj, pixels(end),
                                rate_func=m.linear, **kwargs)

    def test_public_type_and_builder_preserve_native_class_and_alias_identity(self):
        from manimlib.mobject.types.image_mobject import ImageMobject
        self.assertIs(ImageMobject, m.ImageMobject)
        self.assertTrue(issubclass(RasterTransition, m.Animation))
        self.assertIs(type(self.obj.animate.set_image(pixels(BLUE)).build()), RasterTransition)
        self.assertIs(type(self.obj.animate.set_pixel_array(pixels(BLUE)).build()), RasterTransition)

    def test_direct_begin_middle_finish_has_independent_expected_colors(self):
        before = self.obj.data.copy()
        animation = self.animation()
        animation.begin()
        np.testing.assert_array_equal(self.obj.get_pixel_array(), pixels(RED))
        animation.interpolate(.5)
        np.testing.assert_array_equal(self.obj.get_pixel_array(), pixels(PURPLE))
        animation.finish()
        np.testing.assert_array_equal(self.obj.get_pixel_array(), pixels(BLUE))
        np.testing.assert_array_equal(self.obj.data, before)
        self.assertFalse(self.obj._is_updating_suspended())
        self.assertIsNone(animation._raster_plan)
        animation.finish()  # successful completion is idempotent

    def test_target_is_frozen_at_construction_and_start_is_captured_at_begin(self):
        data = pixels(BLUE)
        animation = RasterTransition(self.obj, data, rate_func=m.linear)
        data[:] = RED
        self.obj.set_pixel_array(pixels(GREEN))
        animation.begin()
        np.testing.assert_array_equal(self.obj.get_pixel_array(), pixels(GREEN))
        animation.interpolate(.5)
        np.testing.assert_array_equal(self.obj.get_pixel_array(), pixels(CYAN))
        animation.finish()
        np.testing.assert_array_equal(self.obj.get_pixel_array(), pixels(BLUE))

    def test_encoded_target_and_missing_source_file_are_not_reopened_during_play(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'end.png'
            encoded = capture(m.ImageMobject(pixels(BLUE))).png()
            path.write_bytes(encoded)
            animation = RasterTransition(self.obj, path, rate_func=m.linear)
            path.write_bytes(b'not an image any more')
        animation.begin(); animation.finish()
        expected = m._RasterImage.decode(encoded)
        self.assertEqual(self.obj.get_pixel_array().tobytes(), expected.pixels())
        self.assertEqual(self.obj.image_path, str(path))  # informational source, not live storage

    def test_builder_forwards_duration_easing_final_alpha_and_remover(self):
        builder = self.obj.animate(run_time=.5, rate_func=m.linear, final_alpha_value=.5,
                                   remover=True).set_pixel_array(pixels(BLUE))
        animation = builder.build()
        self.assertEqual(animation.run_time, .5)
        self.assertIs(animation.rate_func, m.linear)
        scene = m.Scene(); scene.add(self.obj)
        scene.play(builder)
        self.assertAlmostEqual(scene.time, .5)
        np.testing.assert_array_equal(self.obj.get_pixel_array(), pixels(PURPLE))
        self.assertNotIn(self.obj, scene.mobjects)
        self.assertIsNone(animation._raster_plan)

    def test_time_span_and_rate_function_use_the_shared_normalization_once(self):
        observed = []
        def rate(alpha):
            observed.append(float(alpha))
            return alpha
        animation = RasterTransition(self.obj, pixels(BLUE), run_time=2,
                                     time_span=(.5, 1.5), rate_func=rate)
        animation.begin()
        animation.interpolate(.25)
        np.testing.assert_array_equal(self.obj.get_pixel_array(), pixels(RED))
        animation.interpolate(.5)
        np.testing.assert_array_equal(self.obj.get_pixel_array(), pixels(PURPLE))
        animation.interpolate(.75)
        np.testing.assert_array_equal(self.obj.get_pixel_array(), pixels(BLUE))
        self.assertEqual(observed, [0, 0, .5, 1])
        animation.finish()

    def test_there_and_back_finishes_at_the_start_material_not_forced_end(self):
        animation = RasterTransition(self.obj, pixels(BLUE), rate_func=m.there_and_back)
        saved = self.obj.copy()
        animation.begin(); animation.interpolate(.5)
        np.testing.assert_array_equal(self.obj.get_pixel_array(), pixels(BLUE))
        animation.finish()
        self.assertTrue(m._raster_images_equal(self.obj, saved))

    def test_new_dimensions_do_not_reset_geometry_views_or_opacity(self):
        self.obj.rotate(.3).shift((1,.2,0)).set_opacity(.4)
        scene = m.Scene(); scene.add(self.obj)
        view, before = self.obj.data, self.obj.data.copy()
        animation = RasterTransition(self.obj, pixels(BLUE, (3, 2)), rate_func=m.linear)
        animation.begin(); animation.interpolate(.5)
        self.assertEqual((self.obj.pixel_width,self.obj.pixel_height), (3,3))
        animation.finish()
        self.assertEqual((self.obj.pixel_width,self.obj.pixel_height), (2,3))
        np.testing.assert_array_equal(self.obj.data, before)
        np.testing.assert_array_equal(view, before)
        self.assertEqual(scene.time, 0)

    def test_surface_and_mesh_builders_interpolate_complete_light_dark_pairs(self):
        for obj in (m.TexturedSurface(m.Surface(resolution=(2,2)), pixels(RED), pixels(GREEN)),
                    m.TexturedGeometry(mesh(), pixels(RED))):
            obj.set_pixel_array(pixels(RED), pixels(GREEN))
            before = obj.data.copy()
            # Test both method override front doors, with explicit timing.
            for method in ('set_textures','set_pixel_array'):
                obj.set_pixel_array(pixels(RED), pixels(GREEN))
                animation = getattr(obj.animate(rate_func=m.linear), method)(pixels(BLUE)).build()
                animation.begin(); animation.interpolate(.5)
                np.testing.assert_array_equal(obj.get_pixel_array(), pixels(PURPLE))
                np.testing.assert_array_equal(obj.get_pixel_array(dark=True), pixels(CYAN))
                animation.finish()
                np.testing.assert_array_equal(obj.get_pixel_array(), pixels(BLUE))
                with self.assertRaises(ValueError): obj.get_pixel_array(dark=True)
                self.assertEqual(obj.num_textures,1)
                np.testing.assert_array_equal(obj.data,before)

    def test_begin_reads_native_dark_pair_even_if_python_metadata_is_stale(self):
        obj = m.TexturedSurface(m.Surface(resolution=(2,2)),pixels(RED),pixels(GREEN))
        obj.num_textures = 1
        animation = self.animation(obj)
        animation.begin(); animation.interpolate(.5)
        self.assertEqual(obj.num_textures,2)
        np.testing.assert_array_equal(obj.get_pixel_array(dark=True),pixels(CYAN))
        animation.abort()

    def test_succession_captures_the_previous_transition_end_just_in_time(self):
        first = self.animation(run_time=.25)
        second = self.animation(end=GREEN, run_time=.25)
        observations = []
        original = second.begin
        def begin():
            observations.append(self.obj.get_pixel_array().copy())
            return original()
        second.begin = begin
        scene=m.Scene();scene.add(self.obj)
        scene.play(m.Succession(first,second))
        self.assertEqual(len(observations),1)
        np.testing.assert_array_equal(observations[0],pixels(BLUE))
        np.testing.assert_array_equal(self.obj.get_pixel_array(),pixels(GREEN))
        self.assertAlmostEqual(scene.time,.5)
        self.assertIsNone(first._raster_plan);self.assertIsNone(second._raster_plan)

    def test_preexisting_suspension_is_preserved_and_frozen_copies_do_not_tick(self):
        ticks=[]
        self.obj.add_updater(lambda obj,dt:ticks.append(dt))
        self.obj.suspend_updating()
        before=len(ticks)
        animation=self.animation()
        animation.begin();animation.update_mobjects(.2);animation.finish()
        self.assertEqual(len(ticks),before)
        self.assertTrue(self.obj._is_updating_suspended())
        self.obj.resume_updating(call_updater=False)

    def test_live_updaters_observe_post_interpolation_material_on_the_scene_clock(self):
        observed=[]
        self.obj.add_updater(lambda obj,dt: observed.append(obj.get_pixel_array().copy()) if dt>0 else None)
        scene=m.Scene();scene.add(self.obj)
        scene.play(self.animation(run_time=.5,suspend_mobject_updating=False))
        self.assertGreater(len(observed),1)
        self.assertTrue(any(np.any(value!=pixels(RED)) and np.any(value!=pixels(BLUE)) for value in observed))
        self.assertAlmostEqual(scene.time,.5)

    def test_failed_rate_preserves_last_published_image_and_releases_transients(self):
        error=ValueError('rate failure')
        def rate(alpha):
            if alpha>.5: raise error
            return alpha
        animation=RasterTransition(self.obj,pixels(BLUE),rate_func=rate)
        animation.begin();animation.interpolate(.5)
        with self.assertRaises(ValueError) as caught: animation.interpolate(.75)
        self.assertIs(caught.exception,error)
        np.testing.assert_array_equal(self.obj.get_pixel_array(),pixels(PURPLE))
        self.assertFalse(self.obj._is_updating_suspended())
        self.assertIsNone(animation._raster_plan)
        animation.abort()

    def test_family_failure_after_preparation_releases_decoded_plan(self):
        animation = self.animation()
        original = self.obj.get_family
        error = ValueError('authored family failure')
        def fail(): raise error
        self.obj.get_family = fail
        try:
            with self.assertRaises(ValueError) as caught: animation.begin()
            self.assertIs(caught.exception, error)
            self.assertIsNone(animation._raster_plan)
            self.assertFalse(self.obj._is_updating_suspended())
        finally:
            self.obj.get_family = original

    def test_nonfinite_rate_is_rejected_without_publishing_a_candidate(self):
        animation=RasterTransition(self.obj,pixels(BLUE),rate_func=lambda a:float('nan'))
        with self.assertRaisesRegex(ValueError,'finite'):animation.begin()
        self.assertFalse(self.obj._is_updating_suspended())
        self.assertIsNone(animation._raster_plan)
        np.testing.assert_array_equal(self.obj.get_pixel_array(),pixels(RED))

    def test_input_failure_and_reentry_preserve_live_material_before_begin(self):
        error=ValueError('input failure')
        class Bad:
            def __array__(self,*args,**kwargs): raise error
        for method in ('set_image','set_pixel_array'):
            with self.assertRaises(ValueError) as caught: getattr(self.obj.animate,method)(Bad())
            self.assertIs(caught.exception,error)
        obj=self.obj
        class Reenter:
            def __array__(self,*args,**kwargs):
                obj.set_pixel_array(pixels(GREEN))
                return pixels(BLUE)
        with self.assertRaisesRegex(RuntimeError,'reenter'): RasterTransition(obj,Reenter())
        np.testing.assert_array_equal(obj.get_pixel_array(),pixels(RED))

    def test_copy_before_and_during_play_owns_independent_objects_and_frozen_materials(self):
        animation=self.animation()
        detached=animation.copy()
        self.assertIsNot(detached.mobject,self.obj)
        detached.begin();detached.finish()
        np.testing.assert_array_equal(self.obj.get_pixel_array(),pixels(RED))
        animation.begin();animation.interpolate(.5)
        live_copy=copy.deepcopy(animation)
        self.assertIsNot(live_copy.mobject,self.obj)
        live_copy.interpolate(1)
        np.testing.assert_array_equal(live_copy.mobject.get_pixel_array(),pixels(BLUE))
        np.testing.assert_array_equal(self.obj.get_pixel_array(),pixels(PURPLE))
        live_copy.abort();animation.abort()

    def test_checkpoint_and_frozen_png_survive_transition_and_restore_original_pixels(self):
        scene=m.Scene();scene.add(self.obj)
        state=scene.get_state();old=self.obj.copy();png=capture(self.obj).png()
        scene.play(self.animation(run_time=.2))
        self.assertFalse(state.mobjects_match(scene.get_state()))
        self.assertNotEqual(capture(self.obj).png(),png)
        self.assertEqual(capture(old).png(),png)
        state.restore_scene(scene)
        np.testing.assert_array_equal(self.obj.get_pixel_array(),pixels(RED))
        self.assertEqual(scene.time,0)

    def test_cancelled_native_output_never_publishes_a_partial_movie(self):
        error=RuntimeError('cancel transition')
        def rate(alpha):
            if alpha>.5:raise error
            return alpha
        scene=m.Scene()
        animation=RasterTransition(self.obj,pixels(BLUE),rate_func=rate,run_time=1)
        with tempfile.TemporaryDirectory() as directory:
            target=Path(directory)/'frames'
            with self.assertRaises(RuntimeError) as caught:
                with scene.render_session(target,format='png_sequence',resolution=(48,32),fps=8,threads=4):
                    scene.add(self.obj);scene.play(animation)
            self.assertIs(caught.exception,error)
            self.assertFalse(target.exists())
        self.assertIsNone(animation._raster_plan)
        self.assertFalse(self.obj._is_updating_suspended())
        np.testing.assert_array_equal(self.obj.get_pixel_array(),pixels(PURPLE))

    def test_raster_transition_can_share_a_play_with_native_placement(self):
        before = self.obj.get_center().copy()
        scene = m.Scene(); scene.add(self.obj)
        scene.play(self.animation(), self.obj.animate.shift((1,0,0)), run_time=.5, rate_func=m.linear)
        np.testing.assert_allclose(self.obj.get_center(), before+(1,0,0), atol=1e-6)
        np.testing.assert_array_equal(self.obj.get_pixel_array(), pixels(BLUE))
        self.assertFalse(self.obj._is_updating_suspended())
        self.assertEqual(scene.time, .5)

    def test_other_animation_overrides_do_not_receive_new_builder_argument_behavior(self):
        # Local subclass prevents a test-only hook escaping to another suite.
        class Custom(m.ImageMobject):
            def custom(self): return self
        owned = m.Animation(self.obj, run_time=7)
        Custom.custom._override_animate = lambda obj: owned
        instance = Custom(pixels(RED))
        self.assertIs(instance.animate(run_time=.5).custom().build(), owned)
        self.assertEqual(owned.run_time, 7)
        with self.assertRaises(NotImplementedError):
            self.obj.animate.set_pixel_array(pixels(BLUE)).shift((1,0,0))

    def test_pixel_frames_match_frozen_captures_at_one_four_and_sixteen_threads(self):
        all_frames=[]
        with tempfile.TemporaryDirectory() as directory:
            for threads in (1,4,16):
                obj=m.ImageMobject(pixels(RED),height=2);scene=m.Scene();expected=[]
                obj.add_updater(lambda mob,dt: expected.append(scene.camera.capture_snapshot(mob).pixels()) if dt>0 else None)
                with scene.render_session(Path(directory)/str(threads),format='png_sequence',
                                          resolution=(48,32),fps=8,threads=threads) as session:
                    scene.add(obj)
                    scene.play(RasterTransition(obj,pixels(BLUE),run_time=.5,rate_func=m.linear,
                                                suspend_mobject_updating=False))
                    obj.clear_updaters();obj.set_pixel_array(pixels(GREEN))
                frames=[path.read_bytes() for path in sorted(session.result.destination.glob('*.png'))]
                self.assertEqual(len(frames),4)
                self.assertEqual(len(set(frames)),4)
                self.assertEqual([m._RasterImage.decode(data).pixels() for data in frames],expected)
                all_frames.append(frames)
        self.assertEqual(all_frames[0],all_frames[1]);self.assertEqual(all_frames[1],all_frames[2])


result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(RasterAnimationTests))
if not result.wasSuccessful():
    raise AssertionError('native raster animation acceptance failed')
