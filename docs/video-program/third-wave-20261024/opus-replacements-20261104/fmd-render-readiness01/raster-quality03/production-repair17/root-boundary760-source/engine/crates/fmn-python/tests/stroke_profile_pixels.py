"""Retained per-record strokes through real native camera captures."""
import gc
import unittest
import numpy as np
import manimlib as m


class StrokeProfilePixels(unittest.TestCase):
    def camera(self):
        camera = m.Camera()
        camera.reset_pixel_shape(160, 90)
        return camera

    def path(self):
        path = m.VMobject(stroke_color=m.RED, stroke_width=60)
        path.set_points_as_corners([[-3.,0.,0.],[0.,0.,0.],[3.,0.,0.]])
        return path

    def pixels(self, camera, path):
        return np.frombuffer(camera.capture_snapshot(path).pixels(), dtype=np.uint8).reshape((90,160,4))

    def test_zero_ended_width_taper_is_visible_and_repeatable(self):
        path, camera = self.path(), self.camera()
        path.data['stroke_width'][:,0] = [0.,30.,60.,30.,0.]
        first = self.pixels(camera, path)
        self.assertGreater(int(first[:,:,:3].sum()), 0)
        np.testing.assert_array_equal(first, self.pixels(camera,path))

    def test_interior_color_is_not_flattened_to_equal_endpoints(self):
        path, camera = self.path(), self.camera()
        path.data['stroke_rgba'][:,:3] = [[1,0,0],[1,0,0],[0,0,1],[1,0,0],[1,0,0]]
        pixels = self.pixels(camera,path)
        self.assertGreater(int(pixels[:,:,2].sum()), 0)
        self.assertGreater(int(pixels[:,:,0].sum()), 0)
        path.data['stroke_rgba'][2,:3] = [0,1,0]
        other = self.pixels(camera,path)
        self.assertGreater(int(other[:,:,1].sum()), 0)
        self.assertFalse(np.array_equal(pixels,other))

    def test_interior_opacity_survives_transparent_ends(self):
        path, camera = self.path(), self.camera()
        path.data['stroke_rgba'][:,3] = [0.,0.,1.,0.,0.]
        self.assertGreater(int(self.pixels(camera,path)[:,:,:3].sum()), 0)

    def test_actual_streamline_taper_and_rebuild_produce_native_pixels(self):
        axes=m.Axes(x_range=(-1,1,1),y_range=(-1,1,1),width=2,height=2)
        lines=m.StreamLines(lambda xs: np.tile([1.,0.],(len(xs),1)),axes,
            noise_factor=0,solution_time=.4,dt=.05,n_samples_per_line=5,
            stroke_width=30,taper_stroke_width=True)
        camera=self.camera()
        first=self.pixels(camera,lines)
        self.assertGreater(int(first[:,:,:3].sum()),0)
        lines.func=lambda xs: np.tile([0.,1.],(len(xs),1))
        lines.draw_lines()
        second=self.pixels(camera,lines)
        self.assertGreater(int(second[:,:,:3].sum()),0)
        self.assertFalse(np.array_equal(first,second))
        np.testing.assert_array_equal(second,self.pixels(camera,lines))

    def test_passing_flash_retains_interior_widths_and_restores_original(self):
        path,camera=self.path(),self.camera()
        original=path.data.copy()
        animation=m.VShowPassingFlash(path,time_width=.4)
        animation.begin()
        try:
            animation.interpolate(.5)
            widths=path.get_stroke_widths()
            self.assertGreater(float(widths.max()),0)
            self.assertGreater(int(self.pixels(camera,path)[:,:,:3].sum()),0)
        finally:
            animation.finish()
        np.testing.assert_array_equal(path.data,original)

    def test_writable_record_views_do_not_leave_cached_pixels_stale(self):
        path,camera=self.path(),self.camera()
        widths=path.data['stroke_width']
        widths[:,0]=[0.,0.,60.,0.,0.]
        before=self.pixels(camera,path)
        widths[:,0]=0
        after=self.pixels(camera,path)
        self.assertGreater(int(before[:,:,:3].sum()),0)
        self.assertEqual(int(after[:,:,:3].sum()),0)
        widths[:,0]=[0.,0.,60.,0.,0.]
        np.testing.assert_array_equal(before,self.pixels(camera,path))


suite=unittest.defaultTestLoader.loadTestsFromTestCase(StrokeProfilePixels)
assert suite.countTestCases()==6
result=unittest.TextTestRunner(verbosity=2).run(suite)
gc.collect()
if not result.wasSuccessful():
    raise AssertionError('native stroke-profile pixel acceptance failed')
