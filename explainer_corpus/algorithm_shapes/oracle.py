"""Project retained independently authored research enumeration onto current native facts.
This checks every emitted native key, not unimplemented later-film facts or audiovisual quality.
The retained script's own research assertions also run unchanged.
"""
from pathlib import Path
import contextlib
import hashlib
import io
import runpy
import sys

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "docs/video-program/research-20261016/evidence/algorithm-shapes-20261114/full-script02/tools/facts02.py"
assert hashlib.sha256(SOURCE.read_bytes()).hexdigest() == "742f85c96832f80e54bc8639d9c594084f20ee15ff5b155d6a5dbd173a0bcb67", "Research oracle changed; review before rebinding"
with contextlib.redirect_stdout(io.StringIO()):
    research = runpy.run_path(str(SOURCE))
roads = research["ROADS"]
places = research["P4"]
routes = research["routes"]
source = research["S"]
facts = {}
def add(key, value):
    assert key not in facts
    facts[key] = str(value)
def minutes(value):
    return "unreachable" if value is None else str(value)
add("map.roads", len(roads))
add("map.routes", len(routes(roads, source, places[-1])))
add("map.order", ",".join(research["topo"](places, roads)))
add("copies10.roads", len(research["rd10"]))
add("copies10.routes", len(routes(roads, source, places[-1])) ** 10)
for i, road in enumerate(roads):
    add(f"road.{i}.value", road[2])
for place in places:
    paths = routes(roads, source, place)
    times = [sum(road[2] for road in path) for path in paths]
    best = min(times) if times else None
    add(f"quick.{place}", minutes(best))
    add(f"count.{place}", len(paths))
    # The empty route has no bottleneck; it is the unlimited identity.
    widths = [min((road[2] for road in path), default=float("inf")) for path in paths]
    width = max(widths) if widths else None
    add(f"wide.{place}", "unlimited" if width == float("inf") else minutes(width))
    add(f"quick_count.{place}", f"{minutes(best)},{times.count(best) if best is not None else 0}")
    for k in range(4):
        exact = [sum(road[2] for road in path) for path in paths if len(path) == k]
        add(f"exactly.{k}.{place}", minutes(min(exact) if exact else None))
for mask in range(1 << len(roads)):
    open_roads = [road for i, road in enumerate(roads) if not mask & (1 << i)]
    add(f"closure.{mask}.reachable", ",".join("yes" if routes(open_roads, source, place) else "no" for place in places))
# Exhaustive monotone alignments, distinct from the native minimum-cost table.
# Root-authored supplemental check; independent peer review remains open.
def alignment_costs(a, b):
    stack = [(0, 0, 0)]
    completed = []
    while stack:
        i, j, cost = stack.pop()
        if i == len(a) and j == len(b):
            completed.append(cost)
            continue
        if i < len(a):
            stack.append((i + 1, j, cost + 1))
        if j < len(b):
            stack.append((i, j + 1, cost + 1))
        if i < len(a) and j < len(b):
            stack.append((i + 1, j + 1, cost + (a[i] != b[j])))
    return completed

for word in ["", "c", "ca", "cat", "cu", "cut"]:
    assert min(alignment_costs(word, word)) == 0
    assert min(alignment_costs(word, "")) == len(word)
    assert min(alignment_costs("", word)) == len(word)
for n, answer in enumerate(research["ways"]):
    add(f"history.ways.{n}", answer)
    add(f"history.asks.{n}", research["asks"][n])
add("history.calls", research["stair_calls"](10))
add("history.distinct", len(research["ways"]))
add("history.grid.cells", len(research["grid"]))
for i in range(4):
    for j in range(4):
        a, b = "cat"[:i], "cut"[:j]
        answer = min(alignment_costs(a, b))
        assert answer == min(alignment_costs(b, a))
        add(f"history.edit.{i}.{j}", answer)
for name, values in [
    ("a", [1, 2, 3]), ("b", [10]), ("merged", [1, 2, 3, 10]),
    ("empty", []), ("zero", [0]), ("two", [2]), ("one_three", [1, 3]),
    ("zero_two", [0, 2]), ("zero_one_three", [0, 1, 3]),
    ("tree_left", [1, 2]), ("tree_right", [3, 10]),
]:
    add(f"chunk.{name}.pair", research["sc_"](values))
    add(f"chunk.{name}.mean", research["mean"](values) if values else "undefined")
add("chunk.wrong_mean", (research["mean"](research["A"]) + research["mean"](research["B"])) / 2)
import struct
for name, value in [("left", (0.1 + 0.2) + 0.3), ("right", 0.1 + (0.2 + 0.3))]:
    add(f"chunk.float.{name}.bits", struct.unpack(">Q", struct.pack(">d", value))[0])
tree = research["T"]
fold_tree = research["fold"]
layers = research["all_layers"]
add("fusion.tree.nodes", len(layers))
add("fusion.tree.leaves", len(research["leaves"](tree)))
add("fusion.tree.sum", research["h"](tree))
for name, change in research["CHANGES"].items():
    mapped, constructed = research["tmap"](tree, change)
    add(f"fusion.{name}.mapped", research["h"](mapped))
    add(f"fusion.{name}.fused", fold_tree(tree, change, lambda a, b: a + b))
    add(f"fusion.{name}.after", change(research["h"](tree)))
    add(f"fusion.{name}.constructed", constructed)
add("fusion.plus1.repaired", fold_tree(tree, lambda n: n + 1, lambda a, b: a + b - 1))
for name, evaluator in [("fold", research["h"]), ("three_times_leaves", research["g"])]:
    add(f"square.{name}.layers", len(layers))
    for i, layer in enumerate(layers):
        across, down = research["square"](evaluator, layer)
        add(f"square.{name}.{i}", f"{across},{down}")
add("loop.simple.routes", len(research["simple"]))
for k in range(8):
    quick = research["quick_k"][k]
    add(f"loop.quick.within.{k + 1}", "unreachable" if quick == research["INF"] else quick)
    add(f"loop.count.within.{k + 1}", research["count_k"][k])
    add(f"loop.crowd.within.{k + 1}", research["crowd_k"][k])
# Chapter 8's other loop numbers, by walk enumeration over the research road lists rather than
# from research's typed (2, 3), 1/2 x 1/2 or lap formula.
from fractions import Fraction
loop_roads, neg_roads = research["LOOP"], research["NEG"]
H, L, F = research["H"], research["L"], research["F"]
def cheapest(road_list, place, most):
    return min(sum(r[2] for r in walk) for walk in routes(road_list, source, place, max_roads=most))
add("loop.added.minutes", loop_roads[-1][2])
add("loop.lap.minutes", sum(r[2] for r in loop_roads if {r[0], r[1]} == {H, L}))
hill_best, lake_best = cheapest(loop_roads, H, 8), cheapest(loop_roads, L, 8)
add("loop.eq.quick.Hill", hill_best)
add("loop.eq.quick.Lake", lake_best)
road_minutes = {(r[0], r[1]): r[2] for r in loop_roads}
add("loop.eq.quick.offer.Hill", lake_best + road_minutes[(L, H)])
add("loop.eq.quick.offer.Lake", hill_best + road_minutes[(H, L)])
crowd_shares = [Fraction(1, sum(1 for other in loop_roads if other[0] == r[0])) for r in loop_roads]
for road_index, crowd_part in enumerate(crowd_shares):
    add(f"loop.crowd.share.{road_index}", crowd_part)
share_by_road = {(r[0], r[1]): part for r, part in zip(loop_roads, crowd_shares)}
add("loop.crowd.quarter", share_by_road[(source, H)] * share_by_road[(H, F)])
add("loop.negative.payout", neg_roads[-1][2])
add("loop.negative.lap.total", sum(r[2] for r in neg_roads if {r[0], r[1]} == {H, L}))
for extra in range(5):
    add(f"loop.negative.laps.{extra}", cheapest(neg_roads, F, 3 + 2 * extra))
add("backward.questions", research["calls"](research["F"], research["IN"]))
for place in research["P4"]:
    add(f"backward.asked.{place}", research["tally"][place])
questions = []
def visit(place):
    questions.append(place)
    for upstream, _ in research["IN"].get(place, []):
        visit(upstream)
visit(research["F"])
for i, place in enumerate(questions):
    add(f"backward.node.{i}", f'{place},{research["answer"](place, research["IN"])}')
for copies in range(11):
    # Closed-form tree size, independently of the native iterative recurrence.
    add(f"backward.calls.{copies}", 3 ** (copies + 1) - 2)
for seed in [6, 7]:
    prefix, stopped = research["unfold"](seed, 5)
    add(f"unfold.{seed}.prefix", ",".join(map(str, prefix)))
    add(f"unfold.{seed}.stopped", str(stopped).lower())
tree = research["T"]
display_order = [tree, tree[1], tree[2], tree[1][1], tree[1][2]]
for name, leaf, branch in [
    ("sum", lambda n: n, lambda a, b: a + b),
    ("leaves", lambda n: 1, lambda a, b: a + b),
    ("largest", lambda n: n, max),
]:
    for i, node in enumerate(display_order):
        add(f"tree.{name}.{i}", research["fold"](node, leaf, branch))
add("tree.printed", research["fold"](tree, str, lambda a, b: f"({a} + {b})"))
add("tree.anybig", str(research["fold"](tree, lambda n: n > 4, lambda a, b: a or b)).lower())
def pair_text(pair):
    time, count = pair
    return f'({"∞" if time == research["INF"] else time}, {count})'
for mask in range(32):
    closed = {(r[0], r[1]) for i, r in enumerate(research["ROADS"]) if mask & (1 << i)}
    good = research["tc_sweep"](research["ROADS"], closed)[0]
    bad = research["tc_sweep"](research["ROADS"], closed, naive=True)[0]
    for place in research["P4"]:
        add(f"pairs.closed.{mask}.good.{place}", pair_text(good[place]))
        add(f"pairs.closed.{mask}.bad.{place}", pair_text(bad[place]))
    add(f"pairs.closed.{mask}.listed", pair_text(research["tc_list"](research["ROADS"], closed, naive=True)))
for place in research["P4"]:
    add(f"pairs.tied.{place}", pair_text(research["t4"][place]))
add("pairs.tied.replay", pair_text(research["dplus"](research["t4"][research["F"]], (6, 1))))
plus, times = research["dplus"], research["dtimes"]
a, b, closed = (3, 1), (5, 1), (research["INF"], 1)
add("pairs.law.left", pair_text(times(plus(a, b), closed)))
add("pairs.law.right", pair_text(plus(times(a, closed), times(b, closed))))
def words_text(words):
    return "{ " + ", ".join(w or "ε" for w in sorted(words)) + " }"
for place in research["P4"]:
    add(f"words.forward.{place}", words_text(research["wr"][place]))
    add(f"words.reversed.{place}", words_text(research["wl"][place]))
add("words.listed", words_text("".join(r[3] for r in path) for path in research["full"]))
add("words.ten.min_length", min(map(len, research["w10"]["S10"])))
add("words.ten.max_length", max(map(len, research["w10"]["S10"])))
for k, matrix in enumerate(research["pmin"]):
    for i, row in enumerate(matrix):
        for j, value in enumerate(row):
            add(f"matrix.quick.{k + 1}.{i}.{j}", "·" if value == research["INF"] else value)
            add(f"matrix.count.{k + 1}.{i}.{j}", research["pcnt"][k][i][j])
for place in places:
    add(f"project.{place}", research["pj"][place])
    add(f"rel.sets.{place}", ",".join(map(str, sorted(research["wb"][place]))))
    add(f"rel.patched.{place}", f'{float(research["rs"][place]):.6f}')
for i, job in enumerate(research["ORDER"]):
    add(f"project.job.{i}", ",".join(map(str, research["spans"][job])))
add("project.max.overlap", research["overlap"])
add("project.one.at.a.time", sum(research["dur"].values()))
add("rel.truth.numerator", research["truth"].numerator)
add("rel.truth.denominator", research["truth"].denominator)
add("rel.successful.patterns", len(research["patterns"]))
add("rel.ten.copies.sets", len(research["wb"][places[-1]]) ** 10)
road_sets = sorted(research["wb"][places[-1]])
for subset in range(1, 1 << len(road_sets)):
    edges = {i for j, mask in enumerate(road_sets) if subset & (1 << j)
             for i in range(len(roads)) if mask & (1 << i)}
    mask = sum(1 << i for i in edges)
    sign = "+" if subset.bit_count() % 2 else "-"
    add(f"rel.overlap.term.{subset}", f'{sign},{mask},{float(research["p"] ** len(edges)):.5f}')
# Exact chances and the two-at-a-time schedule that chapter 15 now shows on screen.
road_chance, por = research["p"], research["por"]
add("rel.sum.of.routes", sum((road_chance ** len(rt) for rt in research["full"]), type(road_chance)(0)))
add("rel.law", f"{road_chance * por(road_chance, road_chance)},{por(road_chance ** 2, road_chance ** 2)}")
add("rel.list.patched", research["listed"])
add("rel.sweep", research["rs"][places[-1]])
add("project.capacity.best.2", research["best_by_c"][2])
add("project.capacity.soup.ends.2", ",".join(map(str, research["soup_ends"][2])))
expected = "".join(f"{key}\t{value}\n" for key, value in facts.items())
if len(sys.argv) == 1:
    print(expected, end="")
elif len(sys.argv) == 2:
    actual = {}
    for line in Path(sys.argv[1]).read_text().splitlines():
        key, value = line.split("\t")
        assert key not in actual, f"duplicate native key: {key}"
        actual[key] = value
    assert actual == facts, {key: (actual.get(key), facts.get(key)) for key in actual.keys() | facts.keys() if actual.get(key) != facts.get(key)}
    print(f"PASS: all {len(facts)} exported native facts match retained research and supplemental checks; script/scene coverage audit and independent review remain open")
else:
    raise SystemExit("Usage: oracle.py [native-facts.tsv]")
