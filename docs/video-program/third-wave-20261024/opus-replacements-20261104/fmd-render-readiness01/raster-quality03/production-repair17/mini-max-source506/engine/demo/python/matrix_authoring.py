"""Mixed, customized and live-edited native matrix cells.

    fmn-python demo/python/matrix_authoring.py MatrixAuthoring --format gif

All shapes, glyphs, animation and output use the ordinary native portal.
"""
from manimlib import *


class IconMatrix(Matrix):
    def element_to_mobject(self, value, **config):
        if isinstance(value, str) and value == "circle":
            return Circle(radius=.2, color=YELLOW).set_fill(YELLOW, opacity=1)
        return super().element_to_mobject(value, **config)


class MatrixAuthoring(Scene):
    def construct(self):
        square = Square(side_length=.4, color=BLUE).set_fill(BLUE, opacity=1)
        matrix = IconMatrix(
            [[square, 1.25, "x"], [Tex("y"), 2.5, "circle"]],
            element_config={"font_size": 36}, h_buff=.65, v_buff=.5,
        )
        self.play(Write(matrix))
        self.play(matrix.get_column(1).animate.set_color(YELLOW))
        # The original square is still the first cell, not a rebuilt proxy.
        self.play(square.animate.rotate(PI / 4))
        self.wait(.5)
        # Existing rows/columns keep pointing to the replaced native cells.
        matrix.swap_entries_for_ellipses(row_index=1, col_index=2)
        self.wait(.5)
        self.play(matrix.get_row(0)[0].animate.shift(.25 * UP))
        self.wait()
