"""Labels follow their live axes without rebuilding grids or glyph geometry."""
from manimlib import *


class LiveCoordinateLabels(Scene):
    def construct(self):
        line = NumberLine((-2, 2, 1)).rotate(PI / 2).shift(4 * LEFT)
        line.add_numbers([-2, -1, 0, 1, 2], direction=RIGHT, buff=0.2)

        plane = ComplexPlane((-2, 2, 1), (-2, 2, 1), faded_line_ratio=1)
        plane.apply_matrix([[1, 0.4, 0], [0.3, 1, 0], [0, 0, 1]])
        plane.shift(RIGHT)
        plane.add_coordinate_labels([1j, 1, -1j, -1, 2j, 2, -2j, -2], font_size=24)
        self.add(line, plane)
        self.wait(0.25)
        self.play(plane.animate.rotate(PI / 6), line.animate.shift(0.5 * UP), run_time=1)
        self.wait(0.25)
