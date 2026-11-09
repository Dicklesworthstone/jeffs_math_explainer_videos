"""Look Gallery math sheets (fm-5wq.50).

Fifty authored tier-1 formulas, the first fifty rows of the box oracle's fixture
(crates/fmn-library/tests/fixtures/tex_reference_boxes.v1.tsv), set as display
`Tex` at one fixed font size in a 5x5 grid per sheet. Nothing is scaled to fit,
so a formula that is too wide or too tall shows as such. The corpus formulas
stay private (plan §15.3), so the sheet uses this committed list instead.
"""
# ruff: noqa: F403, F405 — ordinary manim scene style: names come from `from manimlib import *`.
from pathlib import Path

from manimlib import *

FIXTURE = (Path(__file__).resolve().parents[2]
           / "crates" / "fmn-library" / "tests" / "fixtures" / "tex_reference_boxes.v1.tsv")


def formulas():
    rows = [line.split("\t", 2)[2] for line in FIXTURE.read_text(encoding="utf-8").splitlines()
            if line.strip() and not line.startswith("#")]
    return rows[:50]


class MathSheet(Scene):
    first = 0

    def construct(self):
        cells = VGroup()
        for index, source in enumerate(formulas()[self.first:self.first + 25]):
            tex = Tex(source, font_size=28)
            column, row = index % 5, index // 5
            tex.move_to([(column - 2) * 2.8, (2 - row) * 1.5, 0])
            cells.add(tex)
        self.add(cells)


class MathSheet1(MathSheet):
    first = 0


class MathSheet2(MathSheet):
    first = 25
