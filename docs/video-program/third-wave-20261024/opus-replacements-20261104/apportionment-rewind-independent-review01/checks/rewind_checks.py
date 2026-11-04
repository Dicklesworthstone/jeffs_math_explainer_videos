"""Independent exact checks for the apportionment 12_8 rewind and grey key. Reads files, runs no crate.

    python3 -I -B rewind_checks.py        (from the repo root)

Part 1 derives the run from the tile rule alone (the entered, unheld tile that ends soonest), and
separately from Balinski and Young's wording (Jefferson's list, skipping a town above its upper quota).
Part 2 walks the patched scene's fill and marker order. Part 3 counts frames as the pinned engine does:
n = ceil(run_time x fps) on the exact rational value of the f64 (fmn-anim/src/clock.rs). Part 4 places
phrases as Spoken::at does with no measured marks. Times are the scratch rule scaled, never Oliver.
"""
import math
import pathlib
import re
from fractions import Fraction as Fr

POP = {"Ash": 4000, "Birch": 9000, "Cedar": 26000}
NAMES = list(POP)
TOTAL = sum(POP.values())
THEN, NOW = 6, 13
STAGE = pathlib.Path("narration_lab/third-wave-20261024/root/elevenlabs-apportionment-rebuild-20261104/combined-repair-stage03/explainer_corpus/apportionment")
SRC = (STAGE / "src/chapters/both_promises.rs").read_text()


def end(t, k):
    return Fr(k * TOTAL, POP[t])


def start(t, k):
    return end(t, k - 1)


def run_by_tiles(n):
    seats, out = {t: 0 for t in POP}, []
    for h in range(1, n + 1):
        entered = [(end(t, seats[t] + 1), t) for t in NAMES if start(t, seats[t] + 1) < h]
        assert entered, h
        e, t = min(entered)
        assert sum(1 for x, _ in entered if x == e) == 1, ("tie", h)
        seats[t] += 1
        out.append((h, t, seats[t], dict(seats)))
    return out


def run_by_quota_method(n):
    seats, out = {t: 0 for t in POP}, []
    for h in range(1, n + 1):
        order = sorted(NAMES, key=lambda t: Fr(POP[t], seats[t] + 1), reverse=True)
        for t in order:
            if seats[t] + 1 <= math.ceil(Fr(h * POP[t], TOTAL)):
                seats[t] += 1
                out.append((h, t, seats[t], dict(seats)))
                break
    return out


a, b = run_by_tiles(16), run_by_quota_method(16)
assert [x[:3] for x in a] == [x[:3] for x in b]
print("== 1. the run, two derivations agree through 16")
for h, t, k, s in a:
    print(f"  h{h:2d} {t:5s} seat {k:2d}  tile {start(t, k)} to {end(t, k)}  seats {[s[x] for x in NAMES]}")

hist = {0: {t: 0 for t in POP}} | {h: s for h, _, _, s in a}
later = next((t, k) for h, t, k, _ in a if h == THEN)
waiting = [(t, hist[THEN][t] + 1) for t in NAMES]
print("  later (handed out at then):", later, "ends", end(*later))
missing = [x for x in waiting if start(*x) >= THEN and end(*x) < end(*later) and end(*x) <= NOW]
print("  candidates for missing:", missing, "-> ends", [str(end(*x)) for x in missing], "starts", [str(start(*x)) for x in missing])
assert missing == [("Cedar", 5)]
M = missing[0]
for t, k in waiting:
    print(f"  at then, next {t} seat {k}: {start(t, k)} to {end(t, k)}; entered at 6: {start(t, k) < THEN}")

print("== 2. the patched order: fade the newest fill, then step the marker back")
since = [(h, t, k) for h, t, k, _ in a if THEN < h <= NOW]
assert len(since) == NOW - THEN
filled = {(t, k) for h, t, k, _ in a if h <= NOW}
marker = NOW
ring_on_fill = 0
for i in reversed(range(len(since))):
    h, t, k = since[i]
    assert h == THEN + i + 1 and h == marker, "the fill leaving is the hand-out at the marker"
    filled.discard((t, k))
    assert filled == {(x, j) for hh, x, j, _ in a if hh <= h - 1}, "after the fade: the run at h-1"
    assert all(filled >= {(x, j) for j in range(1, n + 1)} for x, n in {y: sum(1 for z, _ in filled if z == y) for y in NAMES}.items()), "a hole"
    marker = THEN + i
    print(f"  fade {t} {k} (h{h}), marker -> {marker}: seats {[sum(1 for z, _ in filled if z == y) for y in NAMES]}"
          f"{'  <- the missing tile' if (t, k) == M else ''}{'  ends later than 7 1/2' if end(t, k) > end(*M) else ''}")
assert marker == THEN and M not in filled
print("  at landing the marker is exactly at then, seats", [hist[THEN][t] for t in NAMES], "and the missing tile is unfilled:", M not in filled)
print("  the missing tile's fill is since_then[0], the last to leave:", (since[0][1], since[0][2]) == M)

# every rooting of `gone` and `ask`, and every call between the last fill's fade and the ring's fade
scene = SRC.split("#[cfg(test)]")[0]
lines = scene.splitlines()
loop = next(i for i, l in enumerate(lines) if "since_then.iter().enumerate().rev()" in l)
ring_fade = next(i for i, l in enumerate(lines) if "fade_in_all(stage, &[gone, ask]" in l)
first_use = [i + 1 for i, l in enumerate(lines) if re.search(r"\bgone\b|\bask\b", l)]
print(f"  loop at line {loop + 1}, ring's only fade at line {ring_fade + 1}, every line naming gone/ask: {first_use}")
assert loop < ring_fade
assert not [i for i in first_use if i - 1 < ring_fade and "let " not in lines[i - 1] and "//" not in lines[i - 1]], "named before its fade"
fills_used = [i + 1 for i, l in enumerate(lines) if re.search(r"\bsince_then\b", l) and "//" not in l]
print("  since_then is used at lines", fills_used, "so no fill is faded a second time")

print("== 3. whole tiles and the partition the veils claim")
tiles = []
for t in NAMES:
    k = 1
    while start(t, k) < Fr(39, 2):
        tiles.append((t, k))
        k += 1
entered_then = [x for x in tiles if start(*x) < THEN and x != later]
beyond = [x for x in tiles if start(*x) >= THEN and end(*x) > NOW]
inside = [x for x in tiles if start(*x) >= THEN and end(*x) <= NOW]
assert len(entered_then) + 1 + len(beyond) + len(inside) == len(tiles)
assert M in inside and M not in entered_then and M not in beyond
print("  inside:", inside, "counts", [sum(1 for x, _ in inside if x == t) for t in NAMES])
for t in NAMES:
    n = sum(1 for x, _ in inside if x == t)
    assert n <= Fr(NOW - THEN) * Fr(POP[t], TOTAL), t
print(f"  {len(inside)} whole tiles, room for {NOW - THEN}, the supposition needs {NOW - THEN + 1}")
print("  no veil and no fill covers the missing tile from landing to the fade of `supposed`: True (it is neither entered at then nor beyond now)")
print("  real hand-outs since then that end later than 7 1/2:", [(t, k) for _, t, k in since if end(t, k) > end(*M)])


def frames(rt, fps):
    return math.ceil(Fr(rt) * fps)


print("== 4. frames, exact rational of the f64")
for rt in (0.12, 0.1, 0.15, 0.2, 0.3, 0.4, 0.5):
    print(f"  {rt}: {frames(rt, 30)} frames at 30, {frames(rt, 60)} at 60; exact f64 {'above' if Fr(rt) > Fr(str(rt)) else 'below'} the decimal")
BEATS = {"marker fade": (0.3,), "rewind": (0.12,) * 14, "then + legend": (0.3,), "ring + ?": (0.3,)}
tot = {}
for fps in (30, 60):
    t = 0
    for name, rts in BEATS.items():
        t += sum(frames(r, fps) for r in rts)
    tot[fps] = Fr(t, fps)
    print(f"  from 'Go back' to the ring fully in, {fps} fps: {t} frames = {float(tot[fps]):.4f} s")

print("== 5. phrase times, Spoken::at with no marks, scratch length scaled")
script = dict(l.split("\t", 1) for l in (STAGE / "script.tsv").read_text().splitlines() if l and not l.startswith("#"))
s = script["12_8"]
assert s.isascii()
offs = [0] + [i + 2 for i in range(len(s) - 1) if s[i] in ".?!" and s[i + 1] == " " and i + 2 < len(s)]
print("  sentences:", len(offs))


def at(phrase, length):
    i = s.index(phrase)
    j = max(n for n, o in enumerate(offs) if o <= i)
    frm, t0 = offs[j], length * Fr(offs[j], len(s))
    to = offs[j + 1] if j + 1 < len(offs) else len(s)
    t1 = length * Fr(to, len(s))
    return t0 + (t1 - t0) * Fr(i - frm, max(to - frm, 1))


WORDS = len(s.split())
ORDER = ["Could a passed tile ever be left out", "Suppose one were", "Go back to the last moment",
         "a tile that ends later than the missing one", "the missing one", "or to the very start if it never did",
         "Every tile handed out since then", "and the missing tile too", "had not been entered at that moment",
         "or the rule would have picked a tile ending sooner", "And every one of them ends by now"]
assert [s.index(p) for p in ORDER] == sorted(s.index(p) for p in ORDER)
print("  every phrase once, in this order:", all(s.count(p) == 1 for p in ORDER if p != "the missing one"), f"('the missing one' x{s.count('the missing one')})")
for wps in (2.0, 2.6, 3.0, 3.25, 3.5, 3.75):
    length = Fr(WORDS, 1) / Fr(str(wps)) + Fr(2, 5)
    go, later_p, miss, zero = (at(p, length) for p in ("Go back to the last moment", "a tile that ends later than the missing one", "the missing one", "or to the very start if it never did"))
    done = go + tot[30]
    print(f"  {wps:4.2f} words/s: 'Go back' {float(go):6.2f}; ring in {float(done):6.2f}; 'a tile' {float(later_p):6.2f} (spare {float(later_p - done):5.2f});"
          f" 'the missing one' {float(miss):6.2f} (spare {float(miss - done):5.2f}); 'or to the very start' {float(zero):6.2f} (spare after the 0.4 s later-ring fade {float(zero - max(done, later_p) - Fr(13, 30)):5.2f})")

print("== 6. grey key, by the crate's test rule and by CMR10 ems")
LEG = "solid: real up to then, hollow: only supposed"
KEY = "grey: ruled out"
OLD = "solid: real up to then, hollow: only supposed, grey: ruled out"
wide = lambda t, size: Fr(len(t)) * size * Fr("0.0048")
print(f"  characters {len(LEG)}, {len(KEY)}, {len(OLD)}")
print(f"  test rule: legend half {float(wide(LEG, 18) / 2):.3f} < key x 2.0; key end {float(2 + wide(KEY, 18)):.3f} < 6.96")
EM = {" ": .333, ":": .278, ",": .278}
EM.update(a=.5, b=.556, c=.444, d=.556, e=.444, f=.306, g=.5, h=.556, i=.278, j=.306, k=.528, l=.278, m=.833, n=.556, o=.5, p=.556, q=.528, r=.392, s=.394, t=.389, u=.556, v=.528, w=.722, x=.528, y=.528, z=.444)
em = lambda t, size: sum(EM[c] for c in t) * size * 0.01
print(f"  CMR10: legend half {em(LEG, 18) / 2:.3f}, gap to key {2 - em(LEG, 18) / 2:.3f}; key 2.000 .. {2 + em(KEY, 18):.3f}; old 63-char line half {em(OLD, 18) / 2:.3f}")
hx = lambda h: -5.2 + h * (6.6 - -5.2) / 19.5
print(f"  'now' centred at x {hx(13):.3f}, y 2.50; key at y 2.95 (test: {2.95 - 18 * .005:.2f} > {2.5 + 22 * .005:.2f})")
print(f"  'then' at x {hx(6):.3f}; the missing tile {hx(6):.3f} .. {hx(7.5):.3f}, so the ring's left edge lies on the then marker")
