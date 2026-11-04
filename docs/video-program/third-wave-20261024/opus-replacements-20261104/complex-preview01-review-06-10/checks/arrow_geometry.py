#!/usr/bin/env python3
"""What Arrow::new draws for each arrow in chapters 06-08, from the crate's constants.

Engine pin a399d626: Arrow::new has buff = MED_SMALL_BUFF = 0.25 per end, so an arrow
asked to be L long is drawn from start + 0.25 to end - 0.25. Below L = 0.5 the two ends
pass each other and the arrow points backwards.

Reads nothing. Prints scene units and pixels at 720p (90 px per unit).
"""
import math

BUFF = 0.25
PX = 90.0
TIP_RATIO = 5.0  # tip width / thickness, engine default
THICK = 0.015  # scene units per thickness unit


def row(name, anchor, length, thickness):
    drawn = length - 2 * BUFF
    state = "reversed" if drawn < 0 else "forward"
    # with buff 0 the shaft width is capped at 0.1 of the length, as in the engine
    width0 = min(thickness * THICK, 0.1 * length)
    print(f"{name:34} {anchor:28} L={length:5.3f} now {drawn:+.3f} {state:8} ({abs(drawn) * PX:5.1f} px)  with buff 0: {length * PX:5.1f} px, shaft {width0 * PX:4.1f} px")


# 06: input plane 4.6 wide over 2.4, output 4.6 over 3.2; h = 0.45 e^{0.6i}, z0 = 1 + i
h = complex(0.45 * math.cos(0.6), 0.45 * math.sin(0.6))
z0 = complex(1, 1)
row("06_1 step", "one_derivative.rs:138", abs(h) * 4.6 / 2.4, 4.0)
row("06_1 change", "one_derivative.rs:144", abs((z0 + h) ** 2 - z0**2) * 4.6 / 3.2, 4.0)
# 06_8: orientation head on a square of side 0.6 on the mirror plane, 5.4 wide over 3.2
row("06_9 orientation head", "one_derivative.rs:429", 2 * 0.6 * 0.28 * math.sin(0.2) * 5.4 / 3.2, 3.0)
# 08: 24 steps on the unit circle
step = 2 * math.sin(math.pi / 24)
row("08_1/08_3 dz_k, product", "loop_sums.rs:151", step * 2.4, 3.0)
# 08_2: f = z. The chain is a 12-gon walked twice, radius 0.5, side = step
scale = min(3.4, 4.8, 3.0)
row("08_2 chain", "draw.rs:967 via :315", step * scale, 3.0)
# 08_3: f = 1/z. Column of height 6.27 fitted to 4.8
row("08_3 column", "draw.rs:967 via :368", step * 4.8 / 6.27, 3.0)
# 08_4: small row
row("08_4 row", "draw.rs:967 via :466", step * 0.45, 3.0)
# 08_5: circulation tip: chord of an arc of radius 0.26 between angles a1 - 0.3 and a1 + 0.25
row("08_5 circulation tip", "loop_sums.rs:207", 2 * 0.26 * math.sin(0.55 / 2), 3.0)
row("08_5 walk_on", "loop_sums.rs:521", 0.8, 3.0)
row("08_5 shared walks", "loop_sums.rs:559", 0.8, 3.0)
# 07: painted plane is 5.4 wide over [-2, 2], so 1.35 scene units per plane unit
row("07_5 steps", "one_point.rs:480,495", 0.6 * 1.35, 4.0)
row("07_1 map arrow", "one_point.rs:234", 1.2, 4.0)
