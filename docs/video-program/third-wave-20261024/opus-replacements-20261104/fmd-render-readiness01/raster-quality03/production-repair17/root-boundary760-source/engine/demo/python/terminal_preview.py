"""Run: fmn-python edit --watch --kitty demo/python/terminal_preview.py TerminalScene.

Change RADIUS or COLOR, then save while the prompt is idle. Use --sixel instead
only in a sixel-capable terminal. The local `circle`/`label` bindings are editable.
"""
from manimlib import BLUE, DOWN, Circle, Scene, Tex

RADIUS = 1.4
COLOR = BLUE


class TerminalScene(Scene):
    def construct(self):
        self.camera.reset_pixel_shape(640, 360)
        circle = Circle(radius=RADIUS, fill_color=COLOR, fill_opacity=.5)
        label = Tex(r"x^2+y^2=r^2")
        label.next_to(circle, DOWN)
        self.add(circle, label)
        self.embed()
