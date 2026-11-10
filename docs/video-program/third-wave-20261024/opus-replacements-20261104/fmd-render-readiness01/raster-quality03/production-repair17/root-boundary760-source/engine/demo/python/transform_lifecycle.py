"""Sequential targets and an authored restoration path on native group geometry.

Run: python -m fmn_python demo/python/transform_lifecycle.py TransformLifecycle --format mp4
"""
from manimlib import *


class TransformLifecycle(Scene):
    def construct(self):
        shapes = VGroup(
            Square(fill_color=BLUE, fill_opacity=1, stroke_width=0).scale(.4).shift(.6 * LEFT),
            Circle(fill_color=ORANGE, fill_opacity=1, stroke_width=0).scale(.4).shift(.6 * RIGHT),
        ).shift(2 * LEFT)
        shapes.save_state()
        self.add(shapes)
        # Both targets are sampled when their leaf begins, so the shifts
        # accumulate. The second shift must not reuse the first destination.
        self.play(Succession(
            ApplyMethod(shapes.shift, 2 * RIGHT, run_time=1, rate_func=linear),
            ApplyMethod(shapes.shift, 2 * RIGHT, run_time=1, rate_func=linear),
        ))

        def arc(start, end, alpha):
            return (1 - alpha) * start + alpha * end + 6 * alpha * (1 - alpha) * UP

        # The point-free VGroup root and its point-bearing children follow
        # the same Transform lifecycle without replacing their identities.
        self.play(Restore(shapes, path_func=arc, rate_func=linear), run_time=1)
        self.wait(.5)
