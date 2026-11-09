"""Live native typography, authored numeric formats, and existing matrix cells.

Render with the separately installed fmn-python portal:
  fmn-python demo/python/native_numeric_authoring.py NativeNumericAuthoring \
      --format png_sequence --video_dir out/native-numeric-authoring
"""
from manimlib import *


class ScientificReadout(DecimalNumber):
    def get_formatter(self, **kwargs):
        return "{:.2e}"


class NativeNumericAuthoring(Scene):
    default_camera_config = dict(resolution=(640, 360), fps=24)

    def construct(self):
        title = Text("Native numeric authoring", font="IBM Plex Sans", font_size=32)
        title.to_edge(UP)
        # Native preamble expansion keeps the original argument's span: a live
        # readout can replace 1.00 without touching the generated z and equals.
        formula = Tex(
            r"\coefficient{1.00}", color=RED, template="empty",
            additional_preamble=r"\newcommand{\coefficient}[1]{z = #1}",
        ).scale(1.7).shift(1.3 * UP)
        value = formula.make_number_changeable("1.00")
        value.set_value(1 + 2j)
        coefficient = DecimalNumber(
            1 + 2j, color=BLUE, font_size=32,
            text_config={"font": "IBM Plex Sans", "weight": "BOLD"},
        )
        scientific = ScientificReadout(0.125, font_size=32, color=YELLOW)
        circle = Circle(radius=.25, color=BLUE)
        square = Square(side_length=.5, color=YELLOW)
        # These are already live scene objects. Matrix preserves their native
        # identities and scene.add(matrix) replaces the redundant drawing roots.
        self.add(circle, square)
        matrix = Matrix([[circle, coefficient], [square, scientific]])
        matrix.shift(.6 * DOWN)
        self.add(title, formula, matrix)
        self.wait(.25)
        self.play(
            ChangeDecimalToValue(value, 2 - 3j),
            ChangeDecimalToValue(coefficient, 2 - 3j),
            ChangeDecimalToValue(scientific, 12500),
            circle.animate.shift(.3 * UP),
            run_time=1,
        )
        self.wait(.25)
