"""Zero and signed-value transitions on the ordinary native animation path."""
from manimlib import *


class LiveBarChart(Scene):
    def construct(self):
        chart = BarChart([0., .5, .25], height=2., width=5.,
                         label_y_axis=False, bar_stroke_width=0)
        self.add(chart)
        self.play(chart.animate.change_bar_values([.75, 0., -.5]), run_time=1)
        self.play(chart.animate.change_bar_values([-.25, .75, .5]), run_time=1)
        self.play(chart.animate.change_bar_values([.5, .25, 0.]), run_time=1)
