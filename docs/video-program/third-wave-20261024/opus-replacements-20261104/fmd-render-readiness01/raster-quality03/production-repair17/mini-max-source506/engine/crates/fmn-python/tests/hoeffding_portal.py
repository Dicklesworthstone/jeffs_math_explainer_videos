"""Two segments of the Hoeffding's D explainer (the native Rust original is
crates/fmn/examples/hoeffding_d), written as ordinary manimlib scene code, to
exercise franken_manim's *second* front door: the fmn-python portal
(source-compatible `manimlib` on a host CPython).

    fmn-python hoeffding_portal.py RingHook ShuffleLiveD --format mp4 \
        --resolution 1920x1080 --fps 60 --video_dir out.mp4

The Gauntlet (`run_portal_gauntlet_hoeffding`) executes this file unedited
and renders both scenes through `fmn_python.render_scene`.
"""

from manimlib import *
import numpy as np


def hoeffding_d(x, y):
    """Hoeffding's D with average ranks and the tie-aware Q (N >= 5)."""
    def ranks(v):
        order = np.argsort(v, kind="mergesort")
        r = np.empty(len(v))
        sv = np.asarray(v)[order]
        i = 0
        while i < len(v):
            j = i
            while j + 1 < len(v) and sv[j + 1] == sv[i]:
                j += 1
            r[order[i:j + 1]] = (i + j) / 2 + 1
            i = j + 1
        return r

    R, S, N = ranks(x), ranks(y), len(x)
    Q = np.empty(N)
    for i in range(N):
        Q[i] = 1 + np.sum((R < R[i]) & (S < S[i]))
        Q[i] += 0.25 * (np.sum((R == R[i]) & (S == S[i])) - 1)
        Q[i] += 0.5 * np.sum((R == R[i]) & (S < S[i]))
        Q[i] += 0.5 * np.sum((R < R[i]) & (S == S[i]))
    D1 = np.sum((Q - 1) * (Q - 2))
    D2 = np.sum((R - 1) * (R - 2) * (S - 1) * (S - 2))
    D3 = np.sum((R - 2) * (S - 2) * (Q - 1))
    return 30 * ((N - 2) * (N - 3) * D1 + D2 - 2 * (N - 2) * D3) / (
        N * (N - 1) * (N - 2) * (N - 3) * (N - 4))


# The article's worked example must hold before anything is drawn.
assert abs(hoeffding_d([55, 62, 68, 70, 72, 65, 67, 78, 78, 78],
                       [125, 145, 160, 156, 190, 150, 165, 250, 250, 250])
           - 0.4107142857142857) < 1e-12


class RingHook(Scene):
    def construct(self):
        rng = np.random.default_rng(41)
        n = 150
        t = (np.arange(n) + 0.5) / n
        xs = 0.9 * np.cos(TAU * t) + 0.05 * rng.standard_normal(n)
        ys = 0.9 * np.sin(TAU * t) + 0.05 * rng.standard_normal(n)

        axes = Axes(x_range=(-1.2, 1.2, 0.4), y_range=(-1.2, 1.2, 0.4),
                    width=6.0, height=6.0)
        axes.to_edge(LEFT, buff=0.8)
        colors = color_gradient([BLUE_C, TEAL_C, GREEN_C, GOLD_C], n)
        dots = VGroup(*(Dot(axes.c2p(x, y), radius=0.05, color=c)
                        for x, y, c in zip(xs, ys, colors)))

        question = Text("Are X and Y related?", font_size=46)
        question.to_corner(UR, buff=0.8)
        self.play(ShowCreation(axes), Write(question))
        self.play(LaggedStartMap(FadeIn, dots, lag_ratio=0.04, run_time=2.4))

        pearson = ValueTracker(0.0)
        d_value = ValueTracker(0.0)
        r_label = Text("Pearson r =", font_size=40, color=PURPLE_B)
        d_label = Tex(r"\text{Hoeffding's } D =", font_size=44, color=YELLOW)
        labels = VGroup(r_label, d_label).arrange(DOWN, buff=0.6, aligned_edge=LEFT)
        labels.next_to(question, DOWN, buff=0.9).align_to(question, LEFT)
        r_num = always_redraw(lambda: DecimalNumber(
            pearson.get_value(), num_decimal_places=3, include_sign=True,
            font_size=40, color=PURPLE_B).next_to(r_label, RIGHT))
        d_num = always_redraw(lambda: DecimalNumber(
            d_value.get_value(), num_decimal_places=3, include_sign=True,
            font_size=44, color=YELLOW).next_to(d_label, RIGHT))
        self.play(FadeIn(labels, shift=RIGHT * 0.3))
        self.add(r_num, d_num)
        self.play(
            pearson.animate.set_value(float(np.corrcoef(xs, ys)[0, 1])),
            d_value.animate.set_value(float(hoeffding_d(xs, ys))),
            run_time=2.0,
        )
        verdict = VGroup(
            Text("Pearson sees nothing.", font_size=34, color=GREY_B),
            Text("D sees the ring.", font_size=34, color=GREEN_B),
        ).arrange(DOWN, buff=0.3, aligned_edge=LEFT)
        verdict.next_to(labels, DOWN, buff=0.9).align_to(labels, LEFT)
        ring = Circle(radius=axes.x_axis.get_unit_size() * 0.9,
                      color=YELLOW, stroke_width=6).move_to(axes.c2p(0, 0))
        self.play(ShowPassingFlash(ring, time_width=0.6, run_time=2.0),
                  Write(verdict))
        self.wait(2)


class ShuffleLiveD(Scene):
    def construct(self):
        n = 48
        rng = np.random.default_rng(77)
        x = np.linspace(-1, 1, n) + 0.05 * rng.standard_normal(n)
        y = 1.7 * x ** 2 - 0.85 + 0.05 * rng.standard_normal(n)
        rx = np.argsort(np.argsort(x)) + 1
        ry = np.argsort(np.argsort(y)) + 1

        frame = Square(side_length=5.6, stroke_color=GREY_B).to_edge(LEFT, buff=1.0)
        origin = frame.get_corner(DL)
        unit = 5.6 / (n + 1)

        def to_scene(a, b):
            return origin + RIGHT * a * unit + UP * b * unit

        dots = VGroup(*(Dot(to_scene(a, b), radius=0.075, color=c)
                        for a, b, c in zip(rx, ry, color_gradient([BLUE_C, GREEN_C, GOLD_C], n))))
        title = Text("Shuffle Y: the marginals stay, the pairing goes", font_size=38)
        title.to_edge(UP)
        self.play(Write(title), ShowCreation(frame))
        self.play(LaggedStartMap(GrowFromCenter, dots, lag_ratio=0.05, run_time=1.6))

        def live_d():
            pts = np.array([d.get_center() for d in dots])
            return hoeffding_d(pts[:, 0], pts[:, 1])

        label = Tex("D =", font_size=56, color=YELLOW).move_to(RIGHT * 2.6)
        number = always_redraw(lambda: DecimalNumber(
            live_d(), num_decimal_places=3, include_sign=True,
            font_size=56, color=YELLOW).next_to(label, RIGHT))
        self.play(FadeIn(label))
        self.add(number)
        self.wait()

        perm = rng.permutation(n)
        self.play(*(d.animate.move_to(to_scene(rx[k], ry[perm[k]]))
                    for k, d in enumerate(dots)), run_time=3.5)
        self.wait(1.5)
        self.play(*(d.animate.move_to(to_scene(rx[k], ry[k]))
                    for k, d in enumerate(dots)), run_time=2.5)
        self.wait(2)
