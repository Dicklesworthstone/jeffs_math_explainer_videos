"""Change the sampling density of one live surface and its wireframe."""
from manimlib import *


class LiveSurfaceResolution(Scene):
    def construct(self):
        value = ValueTracker(0)
        surface = ParametricSurface(
            lambda u, v: [u, v, (1 - value.get_value()) * (u*u - v*v) / 3],
            u_range=(-2, 2), v_range=(-1.5, 1.5), resolution=(5, 7),
            color=BLUE, shading=(0, 0, 0))
        wire = SurfaceMesh(surface, resolution=(5, 7), stroke_width=1)
        self.add(surface, wire)
        surface.add_updater(lambda obj: obj.set_resolution(
            (17, 19) if .25 < value.get_value() < .75 else (5, 7)))
        wire.add_updater(lambda obj: obj.init_points())
        self.play(value.animate.set_value(1), run_time=2, rate_func=linear)
