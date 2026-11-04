#!/usr/bin/env python3
"""Exact author checks of oriented cubical boundaries; not a general proof."""
from collections import defaultdict
from itertools import product


def boundary(cell):
    """Cell entries -1 mean varying; 0/1 mean fixed on the lower/upper face."""
    result = defaultdict(int)
    varying = [i for i, state in enumerate(cell) if state == -1]
    for position, axis in enumerate(varying):
        sign = -1 if position % 2 else 1
        for endpoint, coefficient in ((0, -sign), (1, sign)):
            face = list(cell)
            face[axis] = endpoint
            result[tuple(face)] += coefficient
    return result


def boundary_twice(cell):
    result = defaultdict(int)
    for face, outer_sign in boundary(cell).items():
        for edge, inner_sign in boundary(face).items():
            result[edge] += outer_sign * inner_sign
    return {edge: coefficient for edge, coefficient in result.items() if coefficient}


counts = {}
for ambient in range(1, 9):
    count = 0
    for cell in product((-1, 0, 1), repeat=ambient):
        if cell.count(-1) < 2:
            continue
        assert boundary_twice(cell) == {}
        count += 1
    counts[ambient] = count

print('oriented cubical cells with boundary-of-boundary zero:', counts)
print('finite author checks only; no test of d², independent review or general proof claimed')
