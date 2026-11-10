"""Native prose, equations, source selection and source morphing: no assets."""
from manimlib import *
from fmn_python.markdown import MarkdownMobject


class MathematicalMarkdown(Scene):
    def construct(self):
        source = "# The power rule\n\nFor $x>0$, take $f(x)=x^2$.\n\n```math\nf'(x)=2x\n```\n\n- Native text and mathematics\n- One scene clock\n"
        document = MarkdownMobject(source, math_mode=True, line_width=7.0, font_size=30)
        document.scale(2.2)
        self.add(document)
        self.wait(.25)
        self.play(document.select_text("f'(x)")[0].animate.set_color(YELLOW), run_time=.5)
        changed = source.replace('x^2', 'x^3').replace('2x', '3x^2')
        self.play(document.animate_source(changed), run_time=1)
        self.wait(.25)
