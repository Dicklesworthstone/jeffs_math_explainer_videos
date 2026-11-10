"""Animate a sampled 3D curve in place without replacing its scene object."""
from manimlib import *


class LiveCurve(Scene):
    def construct(self):
        phase = ValueTracker(0.)
        curve = ParametricCurve(
            lambda t: (2 * np.cos(t), np.sin(t + phase.get_value()), .25 * t),
            t_range=(0., TAU, .08), color=BLUE, stroke_width=4)
        curve.add_updater(lambda current: current.init_points(), call=False)
        self.add(curve)
        self.play(phase.animate.set_value(TAU), run_time=3, rate_func=linear)
