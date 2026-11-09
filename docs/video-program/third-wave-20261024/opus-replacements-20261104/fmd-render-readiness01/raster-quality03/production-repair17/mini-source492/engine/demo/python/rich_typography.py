"""Bundled font faces, original-span style maps and native syntax highlighting.

Run: python -m fmn_python demo/python/rich_typography.py RichTypography --format mp4
"""
from manimlib import *


class RichTypography(Scene):
    def construct(self):
        title = Text("Native typography", font="IBM Plex Sans", weight="BOLD", font_size=42)
        title.shift(2 * UP)
        label = Text("Fonts, styles and live source", font_size=26,
                     t2w={"Fonts": "BOLD"}, t2s={"styles": "ITALIC"},
                     t2f={"source": "CM Typewriter"}, gradient=[BLUE, GREEN])
        label.next_to(title, DOWN, buff=.3)
        code = Code("def wave(t):\n    return sin(t) + 1", language="python", font_size=34)
        code.shift(.8 * DOWN)
        self.add(title, label)
        self.play(FadeIn(code), run_time=.5)
        self.play(code.animate.shift(.5 * RIGHT), run_time=.5)
        self.play(code.get_part_by_text("wave").animate.set_color(YELLOW), run_time=.5)
        self.wait(.5)
