"""Native fixed-topology surface animation driven by one ValueTracker."""
from manimlib import BLUE, ParametricSurface, Scene, ValueTracker, linear


class LiveSurfaceGeometry(Scene):
    def construct(self):
        amplitude = ValueTracker(0)
        surface = ParametricSurface(
            lambda u, v: (u, v, amplitude.get_value() * u * v),
            u_range=(-2, 2), v_range=(-2, 2), resolution=(21, 21),
            color=BLUE,
        )
        surface.add_updater(lambda current: current.init_points(), call=False)
        self.add(surface)
        self.play(amplitude.animate.set_value(.8), run_time=2, rate_func=linear)
        self.play(amplitude.animate.set_value(-.8), run_time=2, rate_func=linear)
        surface.clear_updaters()
        self.wait(.25)
