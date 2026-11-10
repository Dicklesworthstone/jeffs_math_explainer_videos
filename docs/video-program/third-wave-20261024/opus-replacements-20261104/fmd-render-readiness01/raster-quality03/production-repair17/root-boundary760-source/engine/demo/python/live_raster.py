"""Native raster animation with no temporary PNG inputs.

Run: fmn-python demo/python/live_raster.py LiveRasterScene -w
The output path/format can also be chosen through Scene.render_session.
"""
import numpy as np
from manimlib import ImageMobject, Scene, ValueTracker, linear


class LiveRasterScene(Scene):
    def construct(self):
        self.camera.reset_pixel_shape(640, 360)
        rows, columns = np.indices((48, 96))
        phase = ValueTracker(0)

        def pixels():
            offset = int(phase.get_value())
            rgba = np.empty((48, 96, 4), dtype=np.uint8)
            rgba[..., 0] = ((columns + offset) % 96) * 255 // 95
            rgba[..., 1] = rows * 255 // 47
            rgba[..., 2] = ((columns - rows - offset) % 48) * 255 // 47
            rgba[..., 3] = 255
            return rgba

        image = ImageMobject.from_pixel_array(pixels(), height=4)
        image.add_updater(lambda mob: mob.set_pixel_array(pixels()))
        self.add(image)
        self.play(phase.animate.set_value(95), run_time=3, rate_func=linear)
        image.clear_updaters()
        self.wait(.25)
