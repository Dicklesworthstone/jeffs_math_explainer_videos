"""Scalar and discontinuous graphs on the ordinary native updater clock.

    fmn-python demo/python/live_graphs.py LiveGraphs --format gif
"""
import math
import numpy as np

from manimlib import Axes, BLUE, Dot, GREY, RIGHT, Scene, ValueTracker, WHITE, YELLOW


class LiveGraphs(Scene):
    def construct(self):
        axes = Axes(x_range=(-3, 3, 1), y_range=(-2, 2, 1), width=10, height=5)
        amplitude = ValueTracker(.5)
        wave = axes.get_graph(lambda x: amplitude.get_value() * math.sin(2 * x),
                              bind=True, color=BLUE, stroke_width=5)
        cut = ValueTracker(-1.)
        step = axes.get_graph(lambda x: x, color=YELLOW, stroke_width=5)
        step.epsilon = .02
        axes.bind_graph_to_func(
            step, lambda xs: np.where(xs < cut.get_value(), -.75, .75),
            jagged=True, get_discontinuities=lambda: [cut.get_value()],
        )
        self.add(axes, wave, step)
        self.wait(.25)
        self.play(amplitude.animate.set_value(1.5), cut.animate.set_value(1.), run_time=2)
        self.play(axes.animate.shift(.4 * RIGHT), run_time=1)
        # A copied, unbound graph keeps the last native geometry. Its queries
        # no longer read the changing tracker, and the live original is intact.
        frozen = wave.copy().set_stroke(GREY, width=3)
        axes.unbind_graph_from_func(frozen)
        marker = Dot(axes.i2gp(1., frozen), color=WHITE)
        self.add(frozen, marker)
        self.play(amplitude.animate.set_value(.25), cut.animate.set_value(-1.), run_time=2)
        self.wait()
