"""Morph live sampled surfaces without flattening their UV topology."""
from manimlib import *


class SurfaceMorph(Scene):
    def construct(self):
        first = ParametricSurface(lambda u, v: [u, v, 0.5 * u * v],
                                  u_range=(-2, 2), v_range=(-2, 2),
                                  resolution=(7, 11), color=BLUE)
        second = ParametricSurface(lambda u, v: [u, v, 0.25 * (u*u - v*v)],
                                   u_range=(-2, 2), v_range=(-2, 2),
                                   resolution=(13, 9), color=RED)
        self.add(first)
        self.play(Transform(first, second), run_time=2)
        self.play(first.animate.rotate(PI / 6, axis=RIGHT), run_time=1)
