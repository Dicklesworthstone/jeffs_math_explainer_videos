"""Export with fmn-python --format fmtl; replay with the standalone fmn binary."""
from manimlib import LEFT, RIGHT, UP, Scene, Square


class PortableScene(Scene):
    def construct(self):
        moving = Square(side_length=1, fill_opacity=1, stroke_width=0)
        moving.shift(2 * LEFT)
        follower = Square(side_length=0.4, fill_opacity=1, stroke_width=0)
        follower.add_updater(lambda mob, dt: mob.move_to(moving.get_center() + UP))
        self.add(moving, follower)
        self.play(moving.animate.shift(4 * RIGHT), run_time=1)
        self.wait(0.5)
