"""Live pointwise paint over native records, the shared clock and renderer.

Run: python -m fmn_python demo/python/pointwise_paint.py PointwisePaint --format gif
"""
import numpy as np
from manimlib import *


class PointwisePaint(Scene):
    default_camera_config = dict(resolution=(320, 180), fps=12)

    def construct(self):
        phase = ValueTracker(0)
        xs = np.linspace(-3, 3, 17)
        curve = VMobject(stroke_width=30)
        curve.set_points_as_corners(np.column_stack((xs, .5 * np.cos(xs) + 1, np.zeros(len(xs)))))
        square = Square(side_length=1.2, fill_opacity=1, stroke_width=0).shift(DOWN)

        def colors(points):
            t = np.clip((points[:, 0] + 3) / 6, 0, 1)
            alpha = phase.get_value()
            red = (1 - t) * (1 - alpha) + t * alpha
            return np.column_stack((red, np.zeros(len(points)), 1 - red))

        curve.add_updater(lambda mob: mob.set_color_by_rgb_func(colors))
        # The same field drives every interior fill record as well as strokes.
        square.add_updater(lambda mob: mob.set_color_by_rgb_func(colors))
        self.add(phase, curve, square)
        self.play(phase.animate.set_value(1), run_time=1.5, rate_func=linear)
        curve.clear_updaters()
        square.clear_updaters()
        self.wait(1 / 6)
