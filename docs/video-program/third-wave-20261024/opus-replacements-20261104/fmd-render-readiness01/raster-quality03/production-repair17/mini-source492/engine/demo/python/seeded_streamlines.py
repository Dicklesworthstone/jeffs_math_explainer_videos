"""Custom seed hooks and live RK45 redraws, without rebuilding the Scene.

Run: python -m fmn_python demo/python/seeded_streamlines.py SeededFlow --format mp4
"""
import numpy as np
from manimlib import *


class RingSeeds(StreamLines):
    def get_sample_coords(self):
        angles = np.linspace(0, TAU, 8, endpoint=False)
        return 1.25 * np.column_stack([np.cos(angles), np.sin(angles)])


class SeededFlow(Scene):
    def construct(self):
        axes = Axes(x_range=(-3, 3, 1), y_range=(-3, 3, 1))
        expansion = ValueTracker(-.4)

        def field(points):
            x, y = points.T
            speed = expansion.get_value()
            return np.column_stack([-y + speed * x, x + speed * y])

        lines = RingSeeds(field, axes, solution_time=1.5, dt=.125,
                          n_samples_per_line=17, arc_len=8, color_by_magnitude=False,
                          stroke_color=BLUE, stroke_width=4)
        self.frame.set_width(8)
        self.add(lines)
        # Choreo advances the tracker before the normal updater pass. The
        # same container is redrawn from its authored seed hook each frame.
        lines.add_updater(lambda current, dt: current.draw_lines() if dt else None, call=False)
        self.play(expansion.animate.set_value(.4), run_time=2, rate_func=linear)
        lines.clear_updaters()
        self.wait(.5)
