"""Checks for algebra-fixes that need no Cargo: the 11_17 text and the coset link routes.

Mirrors the Rust: the kit's sentence split and the 100-word limit, home_runs on (1 3), and
link_route with the rack geometry from cosets.rs and d4_group.rs. Prints the clearances.

    python3 -I -B checks/check_fixes.py <algebra-fixes folder> <current script.tsv>
"""
import sys
from pathlib import Path

HERE, SCRIPT = Path(sys.argv[1]), Path(sys.argv[2])
FAILED = []


def check(ok, what):
    print('ok  ' if ok else 'FAIL', what)
    if not ok:
        FAILED.append(what)


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in '.?!' and (i + 1 == len(text) or text[i + 1] == ' '):
            out.append(text[start:i + 1].strip())
            start = i + 1
    if text[start:].strip():
        out.append(text[start:].strip())
    return out


# 11_17: same sentence count and order as the recorded cue, only sentence 2 changes.
new = (HERE / '11_17-frederick.txt').read_text().strip()
old = next(line.split('\t', 1)[1] for line in SCRIPT.read_text().splitlines() if line.startswith('11_17\t'))
a, b = sentences(old), sentences(new)
check(len(a) == len(b) == 5, f'11_17 has 5 sentences (old {len(a)}, new {len(b)}), as repair_tests pins')
check([x for x, y in zip(a, b) if x != y] == [a[2]], 'only sentence 2 differs')
check(len(new.split()) <= 100, f'11_17 is {len(new.split())} words, limit 100')
check(not any(ch.isdigit() for ch in new), 'no digits in the spoken text')
check('not six' not in new and 'first returns after two runs' in new, 'no denial of the return at six')
print('   old sentence 2:', a[2])
print('   new sentence 2:', b[2])

# (1 2 3)(1 2) = (1 3): home after runs 2, 4, 6 within six runs.
p = [2, 1, 0]
at, home = [0, 1, 2], []
for runs in range(1, 7):
    at = [p[slot] for slot in at]
    if at == [0, 1, 2]:
        home.append(runs)
check(home == [2, 4, 6], f'(1 3) is home after runs {home}: first return 2, six is the third')

# Coset links: rack geometry from cosets.rs (RACK_X0, RACK_DX, ROW_Y) and d4_group::thumb (±0.48).
X0, DX, ROW = -5.4, 1.45, (1.0, -0.75)
HALF, CAP, OFFSET, LANES = 0.48, 0.03, 0.25, (0.1, -0.1)
NAME_W, NAME_H = 0.15, 0.1


def rack(turns, flip):
    return X0 + DX * turns, ROW[flip]


def route(top, bottom):
    (tx, ty), (bx, by) = rack(top, 0), rack(bottom, 1)
    leave, arrive_y = (tx + OFFSET, ty - HALF - CAP), by + HALF + CAP
    if top == bottom:
        return [leave, (leave[0], arrive_y)]
    lane = LANES[int(bx < tx)]
    return [leave, (leave[0], lane), (bx - OFFSET, lane), (bx - OFFSET, arrive_y)]


# Right cosets of {e, s}: {r^k, s r^k} = {r^k, r^(4-k) s}.
links = {f'r^{k} -> r^{(4 - k) % 4}s': route(k, (4 - k) % 4) for k in range(4)}
worst_tile, worst_name = 9.0, 9.0
for name, corners in links.items():
    for (x0, y0), (x1, y1) in zip(corners, corners[1:]):
        box = (min(x0, x1) - CAP, max(x0, x1) + CAP, min(y0, y1) - CAP, max(y0, y1) + CAP)
        for t in range(4):
            for f in (0, 1):
                x, y = rack(t, f)
                tile = (x - HALF, x + HALF, y - HALF, y + HALF)
                label = (x - NAME_W, x + NAME_W, y - 0.68 - NAME_H, y - 0.68 + NAME_H)
                for target, kind in ((tile, 'tile'), (label, 'name')):
                    gap = max(target[0] - box[1], box[0] - target[1], target[2] - box[3], box[2] - target[3])
                    if kind == 'tile':
                        worst_tile = min(worst_tile, gap)
                    else:
                        worst_name = min(worst_name, gap)
    print(f'   {name}: {[(round(x, 2), round(y, 2)) for x, y in corners]}')
check(worst_tile > -1e-9, f'no link enters a tile (closest {worst_tile:+.3f}; 0 = the cap meets its own tile edge)')
check(worst_name > 0.05, f'no link comes within 0.05 of a name box (closest {worst_name:.3f})')
print('RESULT', 'PASS' if not FAILED else f'FAIL {FAILED}')
