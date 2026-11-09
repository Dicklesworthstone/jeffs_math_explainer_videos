"""Real Transform geometry and native materials on the shared scene clock.

No scene, image, color sampler, animation or output encoder is doubled. Expected
solid-color pixels and manual geometry provide independent frame witnesses.
"""
from __future__ import annotations

import copy
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import numpy as np
import manimlib as m
from fmn_python.raster_animation import RasterTransition
from fmn_python import raster_transform as rt


RED = (255, 0, 0, 255)
BLUE = (0, 0, 255, 255)
GREEN = (0, 255, 0, 255)
PURPLE = (188, 0, 188, 255)
CYAN = (0, 188, 188, 255)


def pixels(color, shape=(2, 3)):
    return np.full((*shape, 4), color, dtype=np.uint8)


def image(color=RED, *, shape=(2, 3)):
    return m.ImageMobject(pixels(color, shape), height=2)


def bare_like(mob):
    """A plain Mobject with `mob`'s record schema, not its class. The
    Reference's Mobject takes no data_dtype keyword; a subclass declares it
    (fm-5wq.20)."""
    class Bare(m.Mobject):
        data_dtype = mob.data_dtype
    return Bare().become(mob)


def capture(obj):
    return m.Camera(resolution=(48, 32), background_opacity=0).capture_snapshot(obj)


def mesh():
    return SimpleNamespace(vertices=np.array([[-1,-1,0],[1,-1,0],[1,1,0],[-1,1,0]]),
                           faces=np.array([[0,1,2],[0,2,3]]),
                           visual=SimpleNamespace(uv=np.array([[0,0],[1,0],[1,1],[0,1]])))


class RasterTransformTests(unittest.TestCase):
    def setUp(self):
        self.a = image().shift((-1, 0, 0))
        self.b = image(BLUE).shift((1, 0, 0))

    def clean(self, animation):
        self.assertNotIn(rt._CACHE, vars(animation))
        self.assertNotIn(rt._ACTIVE, vars(animation))
        self.assertNotIn(rt._TRANSIENTS, vars(animation))
        self.assertNotIn(rt._FRAME_BUSY, vars(animation))

    def test_plain_and_unchanged_material_placement_keep_native_dispatch(self):
        self.assertFalse(m._requires_python_animation(m.Transform(m.Square(), m.Circle())))
        self.assertFalse(m._requires_python_animation(m.Transform(self.a, self.a.copy().shift(m.RIGHT))))
        self.assertTrue(m._requires_python_animation(m.Transform(self.a, self.b)))
        scene=m.Scene();scene.add(self.a)
        scene.play(self.a.animate.shift(m.RIGHT),run_time=.1)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(RED))
        np.testing.assert_allclose(self.a.get_center(),(0,0,0),atol=1e-7)

    def test_direct_transform_interpolates_native_pixels_and_geometry_together(self):
        frozen=capture(self.a).png()
        target=self.b.data.copy(),self.b.get_pixel_array().copy()
        t=m.Transform(self.a,self.b,rate_func=m.linear)
        t.begin();t.interpolate(.5)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(PURPLE))
        np.testing.assert_allclose(self.a.get_center(),(0,0,0),atol=1e-7)
        self.assertEqual(capture(self.a).png(),capture(image(PURPLE)).png())
        t.finish();self.clean(t)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(BLUE))
        np.testing.assert_allclose(self.a.get_points(),self.b.get_points(),atol=1e-7)
        np.testing.assert_array_equal(self.b.data,target[0])
        np.testing.assert_array_equal(self.b.get_pixel_array(),target[1])
        self.assertNotEqual(capture(self.a).png(),frozen)

    def test_direct_mobject_interpolation_has_native_resource_not_class_semantics(self):
        a, b = bare_like(self.a), bare_like(self.b)
        result=a.copy()
        self.assertIs(result.interpolate(mobject1=a,mobject2=b,alpha=.5),result)
        self.assertEqual(m._read_raster_image(result).pixels(),pixels(PURPLE).tobytes())
        self.assertEqual(capture(result).png(),capture(image(PURPLE)).png())
        result.interpolate(a,a,.4)  # direct copy of a constant resource still publishes
        self.assertTrue(m._raster_images_equal(result,a))

    def test_scene_play_and_native_record_views_observe_the_final_generation(self):
        scene=m.Scene();scene.add(self.a)
        view=self.a.data
        t=m.Transform(self.a,self.b,rate_func=m.linear,run_time=.5)
        scene.play(t)
        self.assertAlmostEqual(scene.time,.5)
        np.testing.assert_array_equal(view,self.a.data)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(BLUE))
        self.assertIn(self.a,scene.mobjects);self.assertNotIn(self.b,scene.mobjects)
        self.assertFalse(self.a._is_updating_suspended())
        self.clean(t)

    def test_replacement_and_from_copy_preserve_their_scene_identity_contracts(self):
        for kind in (m.ReplacementTransform,m.TransformFromCopy):
            a,b=image(),image(BLUE).shift(m.RIGHT)
            scene=m.Scene();scene.add(a)
            old=capture(a).png()
            t=kind(a,b,rate_func=m.linear,run_time=.1)
            scene.play(t)
            self.assertIn(b,scene.mobjects)
            np.testing.assert_array_equal(b.get_pixel_array(),pixels(BLUE))
            if kind is m.TransformFromCopy:
                self.assertIn(a,scene.mobjects)
                self.assertEqual(capture(a).png(),old)
            else:
                self.assertNotIn(a,scene.mobjects)
            self.clean(t)

    def test_move_to_target_restore_and_builder_become_preserve_pixels(self):
        scene=m.Scene();scene.add(self.a)
        self.a.save_state()
        self.a.generate_target()
        self.a.target.set_pixel_array(pixels(BLUE)).shift((2,0,0))
        scene.play(m.MoveToTarget(self.a),run_time=.1)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(BLUE))
        scene.play(m.Restore(self.a),run_time=.1)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(RED))
        scene.play(self.a.animate.become(self.b).shift(m.UP),run_time=.1)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(BLUE))
        np.testing.assert_allclose(self.a.get_center(),(1,1,0),atol=1e-6)

    def test_family_lag_applies_the_same_eased_alpha_to_records_and_pixels(self):
        left=m.Group(image(),image()).shift((-1,0,0))
        right=m.Group(image(BLUE),image(GREEN)).shift((1,0,0))
        t=m.Transform(left,right,rate_func=m.linear,lag_ratio=.5)
        t.begin();t.interpolate(.5)
        # Root + two leaves => full length 2; sub-alphas .5 and 0 for the leaves.
        np.testing.assert_array_equal(left[0].get_pixel_array(),pixels(PURPLE))
        np.testing.assert_array_equal(left[1].get_pixel_array(),pixels(RED))
        np.testing.assert_allclose(left[0].get_center(),(0,0,0),atol=1e-7)
        np.testing.assert_allclose(left[1].get_center(),(-1,0,0),atol=1e-7)
        t.finish();self.clean(t)
        np.testing.assert_array_equal(left[1].get_pixel_array(),pixels(GREEN))

    def test_unbalanced_image_families_follow_existing_alignment(self):
        a=m.Group(image())
        b=m.Group(image(BLUE),image(GREEN).shift(m.RIGHT))
        t=m.Transform(a,b,rate_func=m.linear)
        t.begin();t.interpolate(.5);t.finish()
        self.assertEqual(len(a),2)
        for actual,expected in zip(a,b):
            self.assertTrue(m._raster_images_equal(actual,expected))
            np.testing.assert_allclose(actual.get_points(),expected.get_points(),atol=1e-7)
        self.clean(t)

    def test_surface_grid_alignment_and_material_transition_share_one_transform(self):
        a=m.TexturedSurface(m.Surface(resolution=(2,3)),pixels(RED),pixels(GREEN))
        b=m.TexturedSurface(m.Surface(resolution=(4,2)),pixels(BLUE)).shift(m.UP)
        t=m.Transform(a,b,rate_func=m.linear)
        t.begin();t.interpolate(.5)
        self.assertEqual(a.resolution,(4,3))
        np.testing.assert_array_equal(a.get_pixel_array(),pixels(PURPLE))
        np.testing.assert_array_equal(a.get_pixel_array(dark=True),pixels(CYAN))
        t.finish()
        np.testing.assert_array_equal(a.get_pixel_array(),pixels(BLUE))
        with self.assertRaises(ValueError):a.get_pixel_array(dark=True)
        self.assertEqual(a.num_textures,1)
        self.clean(t)

    def test_indexed_mesh_materials_interpolate_without_uv_grid_metadata(self):
        a=m.TexturedGeometry(mesh(),pixels(RED))
        b=m.TexturedGeometry(mesh(),pixels(BLUE)).shift(m.RIGHT)
        t=m.Transform(a,b,rate_func=m.linear)
        t.begin();t.interpolate(.5)
        np.testing.assert_array_equal(a.get_pixel_array(),pixels(PURPLE))
        np.testing.assert_allclose(a.get_center(),(.5,0,0),atol=1e-7)
        t.finish();self.clean(t)

    def test_changed_pixel_dimensions_preserve_native_geometry_and_exact_endpoints(self):
        b=image(BLUE,shape=(3,2)).set_width(4,stretch=True)
        t=m.Transform(self.a,b,rate_func=m.linear)
        t.begin();t.interpolate(.5)
        self.assertEqual((self.a.pixel_width,self.a.pixel_height),(3,3))
        expected=(t.starting_mobject.get_points()+t.target_copy.get_points())/2
        np.testing.assert_allclose(self.a.get_points(),expected,atol=1e-7)
        t.finish()
        self.assertEqual((self.a.pixel_width,self.a.pixel_height),(2,3))
        self.assertTrue(m._raster_images_equal(self.a,b))

    def test_succession_captures_previous_result_and_reuse_refreshes_endpoints(self):
        c=image(GREEN)
        first=m.Transform(self.a,self.b,run_time=.1,rate_func=m.linear)
        second=m.Transform(self.a,c,run_time=.1,rate_func=m.linear)
        observed=[]
        begin=second.begin
        def starting():
            observed.append(self.a.get_pixel_array().copy())
            return begin()
        second.begin=starting
        scene=m.Scene();scene.add(self.a)
        scene.play(m.Succession(first,second))
        np.testing.assert_array_equal(observed[0],pixels(BLUE))
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(GREEN))
        self.b.set_pixel_array(pixels(RED))
        scene.play(first)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(RED))
        self.clean(first);self.clean(second)

    def test_live_target_material_updates_refresh_the_cached_plan(self):
        t=m.Transform(self.a,self.b,rate_func=m.linear)
        t.begin();t.interpolate(.5)
        old=tuple(vars(t)[rt._CACHE].values())[0]
        self.b.set_pixel_array(pixels(GREEN))
        t.interpolate(.5)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels((188,188,0,255)))
        self.assertIsNot(tuple(vars(t)[rt._CACHE].values())[0],old)
        self.b.set_pixel_array(pixels(RED))
        t.interpolate(.7)  # now equal; it must publish the original exact resource
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(RED))
        t.finish();self.clean(t)

    def test_target_updater_on_initially_equal_image_enters_callback_path(self):
        self.b.set_pixel_array(pixels(RED))
        self.b.add_updater(lambda obj,dt: obj.set_pixel_array(pixels(BLUE)) if dt>0 else None)
        t=m.Transform(self.a,self.b,rate_func=m.linear,run_time=.1)
        self.assertTrue(m._requires_python_animation(t))
        scene=m.Scene();scene.add(self.a);scene.play(t)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(BLUE))
        self.clean(t)

    def test_time_span_rate_and_there_and_back_apply_exactly_once(self):
        seen=[]
        def rate(alpha):seen.append(alpha);return alpha
        t=m.Transform(self.a,self.b,run_time=2,time_span=(.5,1.5),rate_func=rate)
        t.begin();t.interpolate(.25);t.interpolate(.5)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(PURPLE))
        self.assertEqual(seen,[0,0,.5])
        t.finish();self.clean(t)
        self.a.set_pixel_array(pixels(RED))
        t=m.Transform(self.a,self.b,rate_func=m.there_and_back)
        t.begin();t.interpolate(.5)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(BLUE))
        t.finish()
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(RED))

    def test_final_alpha_and_remover_follow_shared_animation_contract(self):
        t=m.Transform(self.a,self.b,rate_func=m.linear,final_alpha_value=.5,remover=True,run_time=.1)
        scene=m.Scene();scene.add(self.a);scene.play(t)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(PURPLE))
        np.testing.assert_allclose(self.a.get_center(),(0,0,0),atol=1e-7)
        self.assertNotIn(self.a,scene.mobjects);self.clean(t)

    def test_unchanged_placement_cannot_overwrite_a_simultaneous_raster_animation(self):
        scene=m.Scene();scene.add(self.a)
        move=m.Transform(self.a,self.a.copy().shift((2,0,0)),rate_func=m.linear)
        scene.play(RasterTransition(self.a,pixels(BLUE)),move,run_time=.1)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(BLUE))
        np.testing.assert_allclose(self.a.get_center(),(1,0,0),atol=1e-7)

    def test_direct_nonfinite_alpha_refuses_before_geometry_or_image_publication(self):
        before=self.a.data.copy(),self.a.get_pixel_array().copy()
        for value in (float('nan'),float('inf')):
            with self.assertRaises(ValueError):self.a.interpolate(self.a.copy(),self.b,value)
            np.testing.assert_array_equal(self.a.data,before[0])
            np.testing.assert_array_equal(self.a.get_pixel_array(),before[1])

    def test_failed_path_preserves_error_and_releases_animation_flags_and_plans(self):
        failure=KeyboardInterrupt('path cancelled')
        def path(a,b,alpha):
            if alpha>.4:raise failure
            return (1-alpha)*a+alpha*b
        t=m.Transform(self.a,self.b,rate_func=m.linear,path_func=path)
        t.begin()
        with self.assertRaises(KeyboardInterrupt) as caught:t.interpolate(.5)
        self.assertIs(caught.exception,failure)
        self.clean(t)
        self.assertFalse(self.a._is_updating_suspended())
        self.assertFalse(vars(self.a).get('_is_animating',False))
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(RED))
        t.abort()

    def test_failed_family_admission_occurs_before_decoding_or_interpolation(self):
        a=m.Group(image(),image())
        b=m.Group(image(BLUE),image(GREEN))
        t=m.Transform(a,b,rate_func=m.linear)
        before=tuple(obj.get_pixel_array().copy() for obj in a)
        # Exercise aggregate host policy without allocating a 256 MiB test scene.
        # Native per-plan admission is independently tested in the kernel suite.
        with patch.object(rt,'_MAX_TEXELS',12):
            with self.assertRaisesRegex(ValueError,'family.*budget'):t.begin()
        for obj,expected in zip(a,before):
            np.testing.assert_array_equal(obj.get_pixel_array(),expected)
            self.assertFalse(obj._is_updating_suspended())
        self.clean(t)

    def test_preexisting_suspension_survives_failed_begin(self):
        self.a.suspend_updating()
        def rate(alpha):raise ValueError('failed rate')
        t=m.Transform(self.a,self.b,rate_func=rate)
        with self.assertRaisesRegex(ValueError,'failed rate'):t.begin()
        self.assertTrue(self.a._is_updating_suspended());self.clean(t)
        self.a.resume_updating(call_updater=False)

    def test_checkpoint_and_frozen_capture_retain_pretransform_material(self):
        scene=m.Scene();scene.add(self.a)
        saved=m.SceneState(scene)
        frozen=capture(self.a);data=frozen.png()
        t=m.Transform(self.a,self.b,rate_func=m.linear)
        scene.play(t,run_time=.1)
        self.assertGreater(saved.n_changes(saved),0)
        saved.restore_scene(scene)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(RED))
        self.assertEqual(frozen.png(),data)
        self.assertEqual(capture(self.a).png(),data)

    def test_frame_pipeline_matches_independent_solid_color_and_geometry_oracle(self):
        def render(directory,threads,manual):
            class Demo(m.Scene):
                def construct(self):
                    item=image().shift((-1,0,0));self.add(item)
                    if manual:
                        # Independent solid-color gamma encoding and translated
                        # geometry: do not use Mobject.interpolate or RasterTransition.
                        class Oracle(m.Animation):
                            def interpolate_mobject(self,alpha):
                                def srgb(x):
                                    value=12.92*x if x<=.0031308 else 1.055*x**(1/2.4)-.055
                                    return int(np.floor(255*value+.5))
                                self.mobject.set_pixel_array(pixels((srgb(1-alpha),0,srgb(alpha),255)))
                                self.mobject.move_to((-1+2*alpha,0,0))
                        self.play(Oracle(item,rate_func=m.linear),run_time=.5)
                    else:
                        self.play(m.Transform(item,image(BLUE).shift((1,0,0)),rate_func=m.linear),run_time=.5)
            Demo().render(str(directory),format='png_sequence',resolution=(48,32),
                          fps=8,threads=threads)
            return [path.read_bytes() for path in sorted(directory.glob('*.png'))]
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            reference=render(root/'manual',1,True)
            self.assertGreater(len(set(reference)),2)
            for threads in (1,4,16):
                self.assertEqual(render(root/f'actual-{threads}',threads,False),reference)

    def test_cancelled_output_does_not_publish_partial_image_transform_frames(self):
        failure=KeyboardInterrupt('cancelled render')
        item=self.a
        def rate(alpha):
            if alpha>.3:raise failure
            return alpha
        t=m.Transform(item,self.b,rate_func=rate)
        class Demo(m.Scene):
            def construct(self):self.add(item);self.play(t,run_time=.5)
        with tempfile.TemporaryDirectory() as directory:
            destination=Path(directory)/'frames'
            with self.assertRaises(KeyboardInterrupt) as caught:
                Demo().render(str(destination),format='png_sequence',resolution=(32,24),
                              fps=8,threads=4)
            self.assertIs(caught.exception,failure)
            self.assertFalse(destination.exists())
        self.clean(t)
        self.assertFalse(item._is_updating_suspended())

    def test_active_animation_copy_releases_only_its_own_status_and_materials(self):
        t=m.Transform(self.a,self.b,rate_func=m.linear)
        t.begin();t.interpolate(.5)
        clone=copy.deepcopy(t)
        self.assertIsNot(clone.mobject,self.a)
        clone.interpolate(1);clone.abort()
        np.testing.assert_array_equal(clone.mobject.get_pixel_array(),pixels(BLUE))
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(PURPLE))
        self.assertFalse(vars(clone.mobject).get('_is_animating',False))
        self.assertFalse(vars(clone.mobject).get('animating',False))
        self.assertFalse(clone.mobject._is_updating_suspended())
        self.assertTrue(vars(self.a).get('_is_animating',False))
        self.clean(clone)
        t.abort();self.clean(t)
        self.assertFalse(vars(self.a).get('_is_animating',False))

    def test_failed_begin_cleans_alignment_created_family_members_and_can_retry(self):
        a=m.Group(image())
        b=m.Group(image(BLUE),image(GREEN))
        error=ValueError('failed initial alpha')
        def rate(alpha):raise error
        t=m.Transform(a,b,rate_func=rate)
        with self.assertRaises(ValueError) as caught:t.begin()
        self.assertIs(caught.exception,error)
        self.assertEqual(len(a),2)  # authored/alignment geometry is not rolled back
        for member in a.get_family():
            self.assertFalse(vars(member).get('_is_animating',False))
            self.assertFalse(member._is_updating_suspended())
        self.clean(t)
        scene=m.Scene();scene.add(a)
        scene.play(m.Transform(a,b,rate_func=m.linear),run_time=.1)
        for actual,target in zip(a,b):self.assertTrue(m._raster_images_equal(actual,target))

    def test_same_animation_reentry_refuses_and_releases_acquired_state(self):
        t=None
        def rate(alpha):
            if alpha>.2:t.interpolate(.1)
            return alpha
        t=m.Transform(self.a,self.b,rate_func=rate)
        t.begin()
        with self.assertRaisesRegex(RuntimeError,'reenter'):t.interpolate(.5)
        self.clean(t)
        self.assertFalse(self.a._is_updating_suspended())
        self.assertFalse(vars(self.a).get('_is_animating',False))
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(RED))

    def test_nested_different_transform_keeps_each_context_and_material_plan(self):
        other=image(GREEN)
        inner=m.Transform(other,image(BLUE),rate_func=m.linear)
        inner.begin()
        def path(a,b,alpha):
            inner.interpolate(alpha)
            return (1-alpha)*a+alpha*b
        outer=m.Transform(self.a,self.b,rate_func=m.linear,path_func=path)
        outer.begin();outer.interpolate(.5)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(PURPLE))
        np.testing.assert_array_equal(other.get_pixel_array(),pixels(CYAN))
        outer.finish();inner.finish()
        self.clean(outer);self.clean(inner)

    def test_persistent_transform_updater_uses_existing_time_and_completion_boundary(self):
        t=m.Transform(self.a,self.b,rate_func=m.linear,run_time=.5)
        self.assertIs(m.turn_animation_into_updater(t),self.a)
        # Existing updater protocol samples accumulated time before adding dt.
        self.a.update(.25);self.a.update(0)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(PURPLE))
        np.testing.assert_allclose(self.a.get_center(),(0,0,0),atol=1e-7)
        self.a.update(.25);self.a.update(0)
        np.testing.assert_array_equal(self.a.get_pixel_array(),pixels(BLUE))
        self.assertFalse(self.a.updaters)
        self.assertFalse(self.a._is_updating_suspended())
        self.clean(t)

    def test_abort_recovers_ancestor_status_without_touching_unrelated_siblings(self):
        sibling=m.Square()
        parent=m.Group(self.a,sibling)
        ancestor=m.Group(parent)
        sibling.suspend_updating()
        before=tuple((owner,dict(vars(owner))) for owner in (self.a,parent,ancestor))
        t=m.Transform(self.a,self.b,rate_func=m.linear)
        t.begin()
        self.assertTrue(vars(parent).get('_is_animating',False))
        self.assertTrue(vars(ancestor).get('_is_animating',False))
        t.abort();self.clean(t)
        for owner,attrs in before:
            for key in ('_is_animating','animating'):
                self.assertEqual(key in vars(owner),key in attrs)
                if key in attrs:self.assertIs(vars(owner)[key],attrs[key])
        self.assertTrue(sibling._is_updating_suspended())
        self.assertFalse(vars(sibling).get('_is_animating',False))
        sibling.resume_updating(call_updater=False)


def run_raster_transform_acceptance():
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(RasterTransformTests))
    if not result.wasSuccessful():
        raise AssertionError('native Transform material lifecycle failed')


if __name__ == '__main__':
    run_raster_transform_acceptance()
