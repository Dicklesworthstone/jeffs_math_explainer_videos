"""Asset-free native material/geometry Transform: fmn-python raster_transform.py RasterMorph."""
import numpy as np
from manimlib import Group, ImageMobject, Scene, Transform, linear


class RasterMorph(Scene):
    def construct(self):
        self.camera.reset_pixel_shape(640, 360)
        red = np.full((24, 32, 4), (255, 40, 0, 255), dtype=np.uint8)
        blue = np.full((32, 24, 4), (0, 70, 255, 255), dtype=np.uint8)
        red[::4, :, :3] = 255
        blue[:, ::4, :3] = 255
        start = Group(ImageMobject(red, height=2).shift((-2, 1, 0)),
                      ImageMobject(blue, height=2).shift((2, -1, 0)))
        end = Group(ImageMobject(blue, height=3).shift((2, 1, 0)).rotate(.4),
                    ImageMobject(red, height=1.5).shift((-2, -1, 0)).rotate(-.4))
        self.add(start)
        self.play(Transform(start, end, lag_ratio=.25, rate_func=linear), run_time=2)
