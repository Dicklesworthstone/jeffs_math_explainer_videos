"""Look Gallery primitive panels (fm-5wq.50).

The calibration scenes of scripts/capture_reference_imagery.py as ordinary scene
classes, so the pinned Reference (`manimlib ... -s -w`) and the installed portal
(`fmn-python ... --format png`) render the same unedited source. Each panel is
the scene's final still.
"""
# ruff: noqa: F403, F405 — ordinary manim scene style: names come from `from manimlib import *`.
from manimlib import *


class GradientFills(Scene):
    def construct(self):
        square = Square(side_length=3.0)
        square.set_fill(color=[BLUE_E, YELLOW], opacity=0.8)
        square.set_stroke(color=[RED, GREEN], width=6.0)
        circle = Circle(radius=1.5)
        circle.set_fill(color=[PURPLE, TEAL], opacity=0.5)
        circle.next_to(square, RIGHT, buff=0.5)
        self.add(VGroup(square, circle).center())


class SelfIntersections(Scene):
    def construct(self):
        # A five-point star drawn edge to edge: the nonzero-winding stress case.
        angles = [PI / 2 + k * 4 * PI / 5 for k in range(5)]
        star = Polygon(*(3.0 * np.array([np.cos(a), np.sin(a), 0.0]) for a in angles))
        star.set_fill(BLUE_D, opacity=0.7)
        star.set_stroke(WHITE, width=4.0)
        self.add(star.center())


class JointsAndCaps(Scene):
    def construct(self):
        rows = []
        for joint in ["auto", "bevel", "miter", "no_joint"]:
            zig = VMobject()
            zig.set_points_as_corners([[-3.0, 0.0, 0.0], [-1.0, 1.2, 0.0], [1.0, -1.2, 0.0], [3.0, 0.0, 0.0]])
            zig.set_stroke(YELLOW, width=20.0)
            zig.set_joint_type(joint)
            label = Text(joint, font_size=24)
            label.next_to(zig, LEFT, buff=0.3)
            rows.append(VGroup(zig, label))
        stack = VGroup(*rows).arrange(DOWN, buff=0.6)
        stack.set_height(FRAME_HEIGHT - 1.0)
        self.add(stack.center())


class Glow(Scene):
    def construct(self):
        self.add(GlowDot(LEFT * 2, radius=1.0, color=BLUE),
                 GlowDot(ORIGIN, radius=1.5, color=YELLOW),
                 GlowDot(RIGHT * 2, radius=0.75, color=RED))


class Lighting3D(ThreeDScene):
    def construct(self):
        sphere = Sphere(radius=2.0)
        sphere.set_color(BLUE_E)
        self.frame.reorient(20, 70)
        self.add(sphere)


class TextSample(Scene):
    def construct(self):
        title = Text("FrankenManim look study", font_size=60)
        body = Text("the same feel, cleaner — measured, then kept", font_size=32, slant=ITALIC)
        body.next_to(title, DOWN, buff=0.5)
        self.add(VGroup(title, body).center())


class MathFormula(Scene):
    def construct(self):
        self.add(Tex(r"e^{i\pi} + 1 = 0", font_size=96).center())
