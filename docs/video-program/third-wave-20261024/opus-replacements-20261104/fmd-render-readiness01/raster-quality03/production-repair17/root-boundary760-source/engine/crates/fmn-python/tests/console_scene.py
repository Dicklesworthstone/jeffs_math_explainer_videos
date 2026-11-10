"""README-style installed-artifact fixture for the ``fmn-python`` console."""

from manimlib import DOWN, RED, UP, YELLOW, Dot, FadeIn, Scene, Tex, Text, Write


class Hello(Scene):
    def construct(self):
        # Permanent output-orientation witness.  It is deliberately above the
        # origin and uniquely red so the bridge can decode the emitted PNG and
        # prove row 0 is the top row without a screenshot or a hash-only test.
        self.add(Dot(2.5 * UP, radius=0.3, color=RED))
        title = Text("FrankenManim", font_size=72)
        formula = Tex(r"e^{i\pi} + 1 = 0")
        formula.next_to(title, DOWN)
        self.play(Write(title), FadeIn(formula, shift=UP))
        self.play(formula.animate.set_color_by_tex("i", YELLOW))
        self.wait()
