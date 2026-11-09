"""Run: fmn-python edit demo/python/rebuild_scene.py RebuildScene

At the prompt, change OFFSET or construct(), save, and call reload(). The same
IPython editor now controls a freshly constructed native scene. preview()
returns its native image; reload_source() changes definitions without rebuilding.
"""
from manimlib import BLUE, RIGHT, Scene, Square
from fmn_python import embed_scene

OFFSET = 1.0


class RebuildScene(Scene):
    def construct(self):
        square = Square(side_length=1.0, color=BLUE, fill_opacity=1.0)
        self.add(square)
        self.play(square.animate.shift(OFFSET * RIGHT), run_time=0.5)
        # In edit mode this ends construction and exposes square at the prompt.
        # A rebuild creates a new square; separately saved aliases remain old.
        embed_scene(self, locals())
