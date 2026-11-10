"""Actual live-pixel updates through native checkpoints and ordered frame jobs."""
from __future__ import annotations

from pathlib import Path
import tempfile
import unittest
import numpy as np
import manimlib as m


def pixels(index):
    data=np.zeros((4,6,4),dtype=np.uint8)
    data[...,index%3]=255
    data[...,3]=255
    data[index%4,:, :3]=255
    return data


def image():
    return m.ImageMobject(pixels(0),height=3)


def rgba(png):
    frozen=m._RasterImage.decode(png)
    w,h=frozen.size
    return np.frombuffer(frozen.pixels(),dtype=np.uint8).reshape(h,w,4)


class RasterFrameTests(unittest.TestCase):
    def test_pixel_only_edits_are_visible_to_checkpoint_equality_and_history(self):
        scene=m.Scene(); obj=image(); scene.add(obj)
        scene.save_state(); first=scene.undo_stack[-1]
        records=obj.data.copy()
        clone=obj.copy()
        self.assertTrue(obj.looks_identical(clone))
        obj.set_pixel_array(pixels(1))
        np.testing.assert_array_equal(obj.data,records)
        self.assertFalse(obj.looks_identical(clone))
        self.assertFalse(m.Group(obj).looks_identical(m.Group(clone)))
        self.assertGreater(first.n_changes(first),0)
        second=scene.get_state()
        self.assertFalse(first.mobjects_match(second))
        self.assertNotEqual(first,second)
        self.assertIsNot(first.mobjects_to_copies[obj],second.mobjects_to_copies[obj])
        scene.save_state()
        self.assertEqual(len(scene.undo_stack),2)
        obj.set_pixel_array(pixels(2))
        scene.undo()
        np.testing.assert_array_equal(obj.get_pixel_array(),pixels(1))
        scene.redo()
        np.testing.assert_array_equal(obj.get_pixel_array(),pixels(2))

    def test_same_pixels_reuse_checkpoint_mirrors_but_dark_only_edits_do_not(self):
        src=m.Surface(resolution=(3,3))
        obj=m.TexturedSurface(src,pixels(0),pixels(1))
        scene=m.Scene(); scene.add(obj); scene.save_state()
        old=scene.undo_stack[-1]
        obj.set_pixel_array(pixels(0),pixels(1))
        same=scene.get_state()
        self.assertEqual(old,same)
        self.assertIs(old.mobjects_to_copies[obj],same.mobjects_to_copies[obj])
        obj.set_pixel_array(pixels(0),pixels(2))
        current=scene.get_state()
        self.assertNotEqual(old,current)
        self.assertGreater(old.n_changes(current),0)
        old.restore_scene(scene)
        np.testing.assert_array_equal(obj.get_pixel_array(dark=True),pixels(1))

    def test_changing_only_raster_dimensions_is_not_a_false_equal_resource(self):
        a=m.ImageMobject(np.arange(24,dtype=np.uint8).reshape(2,3,4))
        b=m.ImageMobject(np.arange(24,dtype=np.uint8).reshape(3,2,4))
        self.assertFalse(m._raster_images_equal(a,b))
        self.assertTrue(m._raster_images_equal(a,a.copy()))
        self.assertTrue(m._raster_images_equal(m.Point(),m.Point()))
        self.assertFalse(m._raster_images_equal(a,m.Point()))

    def test_frame_jobs_freeze_updater_pixels_instead_of_reading_latest_material(self):
        delivered=[]
        with tempfile.TemporaryDirectory(prefix='fmn-raster-frames-') as directory:
            for threads in (1,4,16):
                scene=m.Scene(); expected=[]; count=[0]
                obj=image()
                def update(mob,dt):
                    if dt <= 0:
                        return
                    count[0]+=1
                    mob.set_pixel_array(pixels(count[0]))
                    frame=scene.camera.capture_snapshot(*scene.mobjects)
                    expected.append(np.frombuffer(frame.pixels(),dtype=np.uint8).reshape(64,96,4).copy())
                with scene.render_session(Path(directory)/str(threads),format='png_sequence',
                                          resolution=(96,64),fps=8,threads=threads) as session:
                    scene.add(obj)
                    obj.add_updater(update)
                    scene.wait(6/8)
                    obj.clear_updaters()
                    obj.set_pixel_array(pixels(99))
                files=sorted(session.result.destination.glob('*.png'))
                self.assertEqual(session.result.frame_count,6)
                self.assertEqual(len(expected),6)
                outputs=[path.read_bytes() for path in files]
                self.assertGreater(len(set(outputs)),3)
                for data,reference in zip(outputs,expected):
                    np.testing.assert_array_equal(rgba(data),reference)
                delivered.append(outputs)
                self.assertEqual(scene.time,6/8)
        self.assertEqual(delivered[0],delivered[1])
        self.assertEqual(delivered[1],delivered[2])

    def test_cancelled_raster_animation_publishes_nothing_and_keeps_owned_state(self):
        scene=m.Scene(); obj=image()
        failure=RuntimeError('authored pixel animation failure')
        with tempfile.TemporaryDirectory() as directory:
            target=Path(directory)/'cancelled'
            with self.assertRaises(RuntimeError) as caught:
                with scene.render_session(target,format='png_sequence',resolution=(80,64),fps=8,threads=4):
                    scene.add(obj)
                    scene.wait(1/8)
                    obj.set_pixel_array(pixels(1))
                    scene.wait(1/8)
                    raise failure
            self.assertIs(caught.exception,failure)
            self.assertFalse(target.exists())
            np.testing.assert_array_equal(obj.get_pixel_array(),pixels(1))


result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(RasterFrameTests))
if not result.wasSuccessful():
    raise AssertionError('native raster frame/checkpoint acceptance failed')
