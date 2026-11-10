"""A native zero set splits without replacing its scene object or updater."""
from manimlib import *


class SplittingContour(Scene):
    def construct(self):
        separation = ValueTracker(0)

        def field(x, y):
            a = separation.get_value()
            return ((x-a)**2 + y*y) * ((x+a)**2 + y*y) - .5**4

        contour = ImplicitFunction(field, x_range=(-2, 2), y_range=(-1, 1),
                                   min_depth=4, max_quads=256,
                                   use_smoothing=True, color=BLUE, stroke_width=4)
        contour.add_updater(lambda obj: obj.init_points(), call=False)
        self.add(contour)
        self.play(separation.animate.set_value(.9), run_time=2, rate_func=linear)
        self.play(separation.animate.set_value(0), run_time=2, rate_func=linear)
