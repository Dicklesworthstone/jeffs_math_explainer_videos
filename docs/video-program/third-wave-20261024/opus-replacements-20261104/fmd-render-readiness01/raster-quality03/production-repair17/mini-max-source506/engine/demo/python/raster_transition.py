"""Native linear-light pixel transitions, with simultaneous scene placement."""
import numpy as np
from manimlib import ImageMobject, Scene, Succession, linear
from fmn_python.raster_animation import RasterTransition


class RasterTransitions(Scene):
    def construct(self):
        self.camera.reset_pixel_shape(640, 360)
        red = np.full((24, 40, 4), (255, 0, 0, 255), dtype=np.uint8)
        blue = np.full((40, 24, 4), (0, 0, 255, 255), dtype=np.uint8)
        blue[::4, :, :3] = 255
        image = ImageMobject(red, height=3).shift((-2, 0, 0))
        self.add(image)
        self.play(RasterTransition(image, blue, rate_func=linear),
                  image.animate.shift((4, 0, 0)), run_time=1)
        self.play(Succession(RasterTransition(image, red, run_time=.5),
                             RasterTransition(image, blue, run_time=.5)))
        self.play(image.animate(run_time=.5, rate_func=linear).set_pixel_array(red))
