"""A live tracking camera and an authored arc, on the ordinary Scene clock.

Run with: python -m fmn_python demo/python/callback_camera.py CallbackCamera --format mp4
"""
from manimlib import *


class CallbackCamera(Scene):
    def construct(self):
        square = Square(fill_color=BLUE, fill_opacity=1).shift(3 * LEFT)
        self.add(square, Line(4 * LEFT, 4 * RIGHT))
        # The drawable is interpolated first; the camera follows its current
        # position in the same frame, including inside an AnimationGroup.
        self.play(AnimationGroup(
            square.animate(rate_func=linear).shift(6 * RIGHT),
            UpdateFromFunc(self.frame, lambda frame: frame.move_to(square)),
        ), run_time=2)

        def arc(start, target, alpha):
            return (1 - alpha) * start + alpha * target + 2 * alpha * (1 - alpha) * UP

        # Full builder options use the actual Transform constructor, not a
        # fixed list of native-only options. The live path is called per frame.
        self.play(self.frame.animate(path_func=arc,
                                     suspend_mobject_updating=True,
                                     name="return camera", rate_func=linear).move_to(ORIGIN),
                  run_time=1)
        self.wait(.5)
