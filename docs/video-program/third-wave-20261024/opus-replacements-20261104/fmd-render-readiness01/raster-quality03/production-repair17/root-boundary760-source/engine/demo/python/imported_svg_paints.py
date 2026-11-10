"""A source-unedited SVG scene: holes, true-distance dashes and normal animation."""
from manimlib import *


DIAGRAM = '''<svg viewBox="0 0 240 120">
  <path fill="#58C4DD" fill-rule="evenodd"
        d="M10 10H110V110H10Z M30 30H90V90H30Z"/>
  <path fill="none" stroke="#FFFF00" stroke-width="8"
        stroke-dasharray="9 5 3 5" stroke-dashoffset="4"
        d="M130 95Q175 5 225 85"/>
</svg>'''


class ImportedSvgPaints(Scene):
    def construct(self):
        title = Text("Imported paths remain editable", font_size=28).to_edge(UP)
        diagram = SVGMobject(svg_string=DIAGRAM, width=9, stroke_width=None)
        self.add(title)
        self.play(FadeIn(diagram), run_time=.75)
        self.play(diagram.animate.rotate(.15), run_time=.75)
        diagram.save_state()
        self.play(diagram.animate.fade(.55), run_time=.5)
        self.play(Restore(diagram), run_time=.5)
        self.wait(.5)
