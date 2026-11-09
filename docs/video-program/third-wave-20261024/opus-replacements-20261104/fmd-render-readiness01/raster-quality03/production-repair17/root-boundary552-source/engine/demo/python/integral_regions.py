"""A moving integral boundary over the live native graph, without resampling it."""
import math

import manimlib as m


class IntegralRegions(m.Scene):
    def construct(self):
        axes = m.Axes(x_range=(-3, 3, 1), y_range=(-2, 2, 1), width=9, height=5)
        graph = axes.get_graph(lambda x: math.sin(2 * x) + .4, (-3, 3, .2), color=m.WHITE)
        stop = m.ValueTracker(-2.75)
        region = m.always_redraw(lambda: axes.get_area_under_graph(
            graph, (-3, stop.get_value()), fill_color=m.BLUE, fill_opacity=.65))
        self.add(axes, region, graph)
        self.play(stop.animate.set_value(3), run_time=3, rate_func=m.linear)
        rectangles = axes.get_riemann_rectangles(
            graph, (-3, 3), dx=.25, input_sample_type='center',
            fill_opacity=.4, stroke_width=1, show_signed_area=True)
        self.play(m.FadeIn(rectangles), run_time=1)
        self.wait(.5)
