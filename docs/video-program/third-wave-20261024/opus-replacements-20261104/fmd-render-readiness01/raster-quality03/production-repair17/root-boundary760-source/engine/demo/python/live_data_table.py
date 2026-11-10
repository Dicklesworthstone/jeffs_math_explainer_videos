"""No-assets native CSV table with value morphing and retained cell annotations."""
from manimlib import *
from fmn_python.table import TableMobject


class LiveDataTable(Scene):
    def construct(self):
        title = Text("Error decreases as the grid improves", font_size=28).to_edge(UP)
        table = TableMobject.from_csv("method,error\nEuler,0.1\nRK4,0.001\n", font_size=32)
        table.get_cell(1, "error").set_color(GREEN)
        self.add(title, table)
        self.play(table.animate_data([["Euler", "0.05"], ["RK4", "0.0001"]]), run_time=1)
        self.play(table.animate.rotate(0.08), run_time=0.5)
        table.set_data([["Euler", "0.05"], ["RK4", "0.0001"], ["Exact", "0"]])
        self.wait(0.5)
