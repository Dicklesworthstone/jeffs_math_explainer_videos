"""Look Gallery README and example scenes (fm-5wq.50).

Ordinary manimlib scenes of the kind the README and the examples show. Each
panel is the final still after construct, so animations matter only through
the state they leave behind.
"""
# ruff: noqa: F403, F405 — ordinary manim scene style: names come from `from manimlib import *`.
from manimlib import *


class SquareToCircle(Scene):
    def construct(self):
        square = Square(side_length=2.0, color=BLUE)
        label = Tex(r"\int_0^\infty e^{-x^2}\,dx = \frac{\sqrt{\pi}}{2}")
        label.next_to(square, UP)
        self.play(ShowCreation(square), Write(label))
        self.play(square.animate.rotate(PI / 4).set_fill(BLUE, opacity=0.5))
        self.play(Transform(square, Circle(radius=1.0, color=YELLOW)))


class Hello(Scene):
    def construct(self):
        title = Text("FrankenManim", font_size=72)
        formula = Tex(r"e^{i\pi} + 1 = 0")
        formula.next_to(title, DOWN)
        self.play(Write(title), FadeIn(formula, shift=UP))
        self.play(formula.animate.set_color_by_tex("i", YELLOW))


class GraphOnAxes(Scene):
    def construct(self):
        axes = Axes((-3, 3), (-2, 2), width=10, height=6)
        axes.add_coordinate_labels()
        graph = axes.get_graph(lambda x: np.sin(2 * x), color=BLUE)
        label = axes.get_graph_label(graph, Tex(r"\sin(2x)"))
        self.add(axes, graph, label)


class PlaneAndVectors(Scene):
    def construct(self):
        plane = NumberPlane((-6, 6), (-4, 4))
        vector = Vector([2, 1, 0], color=YELLOW)
        other = Vector([-1, 2, 0], color=RED)
        label = Tex(r"\vec{v}", color=YELLOW).next_to(vector.get_end(), RIGHT)
        self.add(plane, vector, other, label)


class BracedMatrix(Scene):
    def construct(self):
        matrix = Matrix([[1, 2], [3, 4]])
        brace = Brace(matrix, DOWN)
        label = brace.get_tex("A")
        self.add(matrix, brace, label)


class PiecewiseCases(Scene):
    def construct(self):
        self.add(Tex(r"f(x) = \begin{cases} x^2 & x \ge 0 \\ -x & x < 0 \end{cases}", font_size=60))


class TrackerTrace(Scene):
    def construct(self):
        tracker = ValueTracker(0)
        dot = Dot(color=YELLOW)
        dot.add_updater(lambda d: d.move_to([np.cos(tracker.get_value()) * 2,
                                             np.sin(2 * tracker.get_value()) * 1.5, 0]))
        trace = TracedPath(dot.get_center, stroke_color=YELLOW, stroke_width=3)
        self.add(trace, dot)
        self.play(tracker.animate.set_value(TAU), run_time=2, rate_func=linear)


class SurfaceWithAxes(ThreeDScene):
    def construct(self):
        self.frame.reorient(-30, 70)
        axes = ThreeDAxes((-3, 3), (-3, 3), (-2, 2))
        surface = ParametricSurface(lambda u, v: [u, v, 0.5 * np.sin(u) * np.cos(v)],
                                    u_range=(-3, 3), v_range=(-3, 3), resolution=(32, 32))
        surface.set_color(BLUE_D)
        self.add(axes, surface)


class Hexagon(RegularPolygon):
    def __init__(self, **kwargs):
        super().__init__(n=6, **kwargs)
        self.set_fill(TEAL, opacity=0.6)


class SubclassedPolygon(Scene):
    def construct(self):
        hexagons = VGroup(*(Hexagon().scale(0.8) for _ in range(3))).arrange(RIGHT, buff=0.5)
        self.add(hexagons)


class NumberReadouts(Scene):
    def construct(self):
        decimal = DecimalNumber(3.14159, num_decimal_places=3, font_size=72)
        integer = Integer(42, font_size=72).next_to(decimal, DOWN, buff=0.8)
        self.add(decimal, integer)


class TitleAndList(Scene):
    def construct(self):
        title = Title("Native text")
        items = BulletedList("bundled faces", "owned shaping", "no Pango")
        items.next_to(title, DOWN, buff=0.8)
        self.add(title, items)


class ArrowsAndLabels(Scene):
    def construct(self):
        left, right = Dot(LEFT * 3), Dot(RIGHT * 3)
        arrow = Arrow(left.get_center(), right.get_center(), buff=0.2)
        double = CurvedDoubleArrow(LEFT * 3 + DOWN * 1.5, RIGHT * 3 + DOWN * 1.5, angle=-PI / 6)
        curved = CurvedArrow(LEFT * 2 + UP * 1.5, RIGHT * 2 + UP * 1.5)
        label = Text("flow", font_size=36).next_to(arrow, UP)
        self.add(left, right, arrow, double, curved, label)


class ColoredWords(Scene):
    def construct(self):
        words = Text("red green blue", t2c={"red": RED, "green": GREEN, "blue": BLUE}, font_size=72)
        formula = Tex(r"a^2 + b^2 = c^2", font_size=72).next_to(words, DOWN, buff=0.8)
        formula.set_color_by_gradient(YELLOW, ORANGE)
        self.add(words, formula)


class ShapeGrid(Scene):
    def construct(self):
        shapes = VGroup(Square(), Circle(), Triangle(), RegularPolygon(7), RegularPolygon(5), Annulus(),
                        Ellipse(width=2, height=1), RoundedRectangle(width=2, height=1.2))
        for shape, color in zip(shapes, [BLUE, RED, GREEN, YELLOW, PURPLE, TEAL, ORANGE, PINK]):
            shape.set_fill(color, opacity=0.6).set_stroke(WHITE, 2)
        shapes.arrange_in_grid(2, 4, buff=0.6)
        self.add(shapes)


class NumberLineMarks(Scene):
    def construct(self):
        line = NumberLine((-5, 5, 1), width=12, include_numbers=True)
        mark = Triangle(fill_opacity=1, color=YELLOW).scale(0.15).rotate(PI)
        mark.next_to(line.n2p(2.5), UP, buff=0.1)
        self.add(line, mark)


class CodeSnippet(Scene):
    def construct(self):
        self.add(Code("def area(r):\n    return PI * r ** 2", language="python", font_size=28))


class DashesAndDots(Scene):
    def construct(self):
        dashed = DashedLine(LEFT * 4, RIGHT * 4, dash_length=0.2)
        circle = DashedVMobject(Circle(radius=1.5, color=BLUE), num_dashes=24).shift(DOWN * 0.5)
        dots = VGroup(*(Dot(radius=0.06).move_to([x, 2, 0]) for x in np.linspace(-4, 4, 17)))
        self.add(dashed, circle, dots)


class Highlights(Scene):
    def construct(self):
        formula = Tex(r"E = mc^2", font_size=96)
        box = SurroundingRectangle(formula, color=YELLOW, buff=0.25)
        underline = Underline(formula, color=RED).shift(DOWN * 0.3)
        self.add(formula, box, underline)


class GroupsArranged(Scene):
    def construct(self):
        rows = VGroup(*(VGroup(*(Square(side_length=0.6).set_fill(c, 0.8) for c in colors)).arrange(RIGHT)
                        for colors in ([RED, ORANGE, YELLOW], [GREEN, TEAL, BLUE], [PURPLE, PINK, GREY])))
        rows.arrange(DOWN, buff=0.4)
        self.add(rows)


class FadeTransformResult(Scene):
    def construct(self):
        fraction = Tex(r"\frac{1}{6}", font_size=72)
        decimal = DecimalNumber(1 / 6, num_decimal_places=4, font_size=72)
        self.add(fraction)
        self.play(FadeTransform(fraction, decimal))
