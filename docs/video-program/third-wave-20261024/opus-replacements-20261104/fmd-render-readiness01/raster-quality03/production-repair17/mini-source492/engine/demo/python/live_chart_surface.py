"""Native live plotting and wireframes in a rotated coordinate system."""
from manimlib import *


class LiveChartSurface(Scene):
    def construct(self):
        axes = ThreeDAxes(x_range=(-2, 2), y_range=(-2, 2), z_range=(-2, 2))
        axes.rotate(PI / 6, axis=RIGHT)
        height = ValueTracker(0)
        surface = axes.get_graph(lambda u, v: height.get_value() + u*v/4,
            u_range=(-2, 2), v_range=(-2, 2), resolution=(9, 7),
            color=BLUE, opacity=.8)
        surface.add_updater(lambda obj: obj.init_points(), call=False)
        mesh = SurfaceMesh(surface, resolution=(5, 5), stroke_width=2)
        mesh.add_updater(lambda obj: obj.init_points(), call=False)
        self.camera.frame.set_euler_angles(phi=.7)
        self.add(axes, surface, mesh)
        self.play(height.animate.set_value(1), run_time=1, rate_func=linear)
