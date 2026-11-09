"""A tracker-driven wireframe, rebuilt by the existing native SurfaceMesh owner."""
from manimlib import *


class LiveWireframe(Scene):
    def construct(self):
        bend = ValueTracker(0)
        surface = ParametricSurface(
            lambda u, v: [u, v + bend.get_value() * (u*u - 1), 0],
            u_range=(-2, 2), v_range=(-1, 1), resolution=(17, 13),
        )
        mesh = SurfaceMesh(surface, resolution=(9, 7), normal_nudge=0,
                           stroke_color=BLUE, stroke_width=2)

        def refresh(wireframe):
            surface.init_points()
            wireframe.init_points()

        mesh.add_updater(refresh, call=False)
        self.add(mesh)
        self.play(bend.animate.set_value(.7), run_time=2)
        self.play(bend.animate.set_value(-.4), run_time=2)
