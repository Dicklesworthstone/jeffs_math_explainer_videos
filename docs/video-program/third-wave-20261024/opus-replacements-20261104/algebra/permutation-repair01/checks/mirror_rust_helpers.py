#!/usr/bin/env python3
"""A no-Cargo stand-in for chapter 11's pure Rust helpers.

Transcribes the helpers in candidate src/permutations.rs line by line (zero-based
image arrays, d4.rs's apply, the PAIRS table), reruns every `repair_tests` and
`route_tests` assertion that does not need Rust's type checker, and compares the
facts `add_facts` would register with the `permutations.*` lines of candidate
oracle.py.

This is a transcription, so it proves the arithmetic and the expected values in
the tests. It does not prove the Rust compiles or that the Rust matches this file.

Run from the repository root: python3 -I -B <this file>
"""
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1] / "candidate" / "explainer_corpus" / "abstract_algebra_expanded"
SWAP_12 = [1, 0, 2]
CYCLE_123 = [1, 2, 0]
SWAP_13_ON_134 = [1, 0, 2]
SWAP_14_ON_134 = [2, 1, 0]
CYCLE_134_ON_134 = [1, 2, 0]
INVERSE_134_ON_134 = [2, 0, 1]
LEGAL_MAP = [1, 2, 0]
BAD_MAP = [0, 0, 2]
BAD_DOUBLED, BAD_MISSING = 0, 1
WITNESS = [1, 2, 4, 8]
PAIRS = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]
ONE_SWAP = [(0, 2)]
THREE_SWAPS = [(0, 1), (1, 2), (0, 1)]
REVERSES_IN_PLACE = 1
TRADE_AND_REVERSE = [0, 3]
TRADE_ONLY = [2, 5]
UNTOUCHED = 4
CORNER_POINTS = [(1, 1), (1, -1), (-1, -1), (-1, 1)]
ALL = [(t, False) for t in range(4)] + [(t, True) for t in range(4)]
R, S = (1, False), (0, True)


def listed(values):
    return ",".join(str(v) for v in values)


def route(origin, first, second):
    return [origin, first[origin], second[first[origin]]]


def cycle_images(n, cycles):
    images = list(range(n))
    for cycle in cycles:
        for index, label in enumerate(cycle):
            images[label - 1] = cycle[(index + 1) % len(cycle)] - 1
    return images


def then(first, second):
    return [second[slot] for slot in first]


def cards_by_slot(at):
    cards = [0] * len(at)
    for card, slot in enumerate(at):
        cards[slot] = card + 1
    return cards


def swaps_needed(p):
    seen = [False] * len(p)
    cycles = 0
    for start in range(len(p)):
        at = start
        if not seen[at]:
            cycles += 1
        while not seen[at]:
            seen[at] = True
            at = p[at]
    return len(p) - cycles


def parity_word(swaps):
    return "even" if swaps % 2 == 0 else "odd"


def cycle_tex(p):
    seen = [False] * len(p)
    written = ""
    for start in range(len(p)):
        at = start
        if p[at] == at:
            continue
        labels = []
        while not seen[at]:
            seen[at] = True
            labels.append(str(at + 1))
            at = p[at]
        if labels:
            written += "(" + r"\;".join(labels) + ")"
    return written


def arrivals(mapping):
    counts = [0, 0, 0]
    for slot in mapping:
        counts[slot] += 1
    return counts


def differences(x):
    return [x[j] - x[i] for i, j in PAIRS]


def after_swaps(x, swaps):
    x = list(x)
    for a, b in swaps:
        x[a], x[b] = x[b], x[a]
    return x


def delta(x):
    total = 1
    for d in differences(x):
        total *= d
    return total


def chip_destinations(before, after):
    return [next(k for k, new in enumerate(after) if abs(new) == abs(old)) for old in before]


def route_arrangements(swaps):
    x = list(WITNESS)
    out = [list(x)]
    for a, b in swaps:
        x[a], x[b] = x[b], x[a]
        out.append(list(x))
    return out


def tally_tex(x):
    factors = r"\cdot".join(f"({n})" if n < 0 else str(n) for n in differences(x))
    return f"\\Delta={factors}={delta(x):+d}"


def d4_apply(point, turns, flip):
    x, y = point
    if flip:
        y = -y
    for _ in range(turns):
        x, y = -y, x
    return (x, y)


def corner_images(turns, flip):
    return [CORNER_POINTS.index(d4_apply(point, turns, flip)) for point in CORNER_POINTS]


def elem_mul(a, b):
    """d4_group::Elem::mul: s r^b = r^(-b) s."""
    carried = (4 - b[0]) % 4 if a[1] else b[0]
    return ((a[0] + carried) % 4, a[1] != b[1])


def symmetries():
    return [corner_images(t, f) for t, f in ALL]


def arrangements():
    found = []
    for a in range(4):
        for b in range(4):
            for c in range(4):
                for d in range(4):
                    p = [a, b, c, d]
                    if all(card in p for card in range(4)):
                        found.append(p)
    return found


def broken_edges(at):
    return [(i, (i + 1) % 4) for i in range(4) if abs(at[i] - at[(i + 1) % 4]) % 2 == 0]


def way_home(at):
    back = [0] * 4
    for card, slot in enumerate(at):
        back[slot] = card
    return back


def one_based(images):
    return listed(i + 1 for i in images)


def cell_text(p):
    return "".join(str(card + 1) for card in p)


def line_text(cue, script):
    for line in script.splitlines():
        if line.startswith(cue + "\t"):
            return line[len(cue) + 1 :]
    return None


def sentence_ranges(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            piece = text[start : i + 1]
            lead = len(piece) - len(piece.lstrip())
            out.append((start + lead, i + 1))
            start = i + 1
    return out


def facts():
    out = {}
    out["permutations.bad_map_arrivals"] = listed(arrivals(BAD_MAP))
    every = arrangements()
    symmetric = symmetries()
    out["permutations.corner_count"] = len(every)
    kept = [cell_text(p) for p in every if p in symmetric]
    out["permutations.edge_keeping"] = len(kept)
    out["permutations.edge_keeping_arrangements"] = ",".join(kept)
    trade = cycle_images(4, [[1, 2]])
    out["permutations.trade_broken_edges"] = ",".join(f"{i + 1}-{j + 1}" for i, j in broken_edges(trade))
    swapped = after_swaps(WITNESS, ONE_SWAP)
    before, after = differences(WITNESS), differences(swapped)
    out["permutations.witness"] = listed(WITNESS)
    out["permutations.witness_swapped"] = listed(swapped)
    out["permutations.chips_start"] = listed(before)
    out["permutations.chips_swapped"] = listed(after)
    out["permutations.chip_destinations"] = one_based(chip_destinations(before, after))
    out["permutations.delta_start"] = delta(WITNESS)
    out["permutations.delta_swapped"] = delta(swapped)
    steps = route_arrangements(THREE_SWAPS)
    out["permutations.route_three_arrangements"] = ";".join(listed(x) for x in steps[1:])
    out["permutations.route_three_deltas"] = listed(delta(x) for x in steps)
    r, s = corner_images(*R), corner_images(*S)
    out["permutations.quarter_images"] = one_based(r)
    out["permutations.flip_images"] = one_based(s)
    out["permutations.half_images"] = one_based(then(r, r))
    out["permutations.rs_images"] = one_based(then(s, r))
    out["permutations.sr_images"] = one_based(then(r, s))
    out["permutations.quarter_swaps"] = swaps_needed(r)
    out["permutations.half_swaps"] = swaps_needed(then(r, r))
    out["permutations.inverse_images"] = one_based(cycle_images(4, [[1, 4, 3]]))
    tri, swap = cycle_images(6, [[1, 3, 5]]), cycle_images(6, [[2, 6]])
    out["permutations.disjoint_final"] = listed(cards_by_slot(then(tri, swap)))
    return out


def tests(script):
    done = []

    def check(name, condition):
        assert condition, name
        done.append(name)

    # route_tests
    check("two_swap_routes", [[1, 3, 4][route(i, [1, 0, 2], [2, 1, 0])[2]] for i in range(3)] == [3, 4, 1])
    forward = [route(i, SWAP_12, CYCLE_123) for i in range(3)]
    reverse = [route(i, CYCLE_123, SWAP_12) for i in range(3)]
    check("forward_routes", forward == [[0, 1, 2], [1, 0, 1], [2, 2, 0]])
    check("reverse_routes", reverse == [[0, 1, 0], [1, 2, 2], [2, 0, 1]])
    check("inverse_board", all(route(o, CYCLE_134_ON_134, INVERSE_134_ON_134)[2] == o for o in range(3)))
    # repair_tests
    counts = {"11_1": 4, "11_2": 5, "11_3": 8, "11_4": 7, "11_5": 5, "11_6": 3, "11_7": 3, "11_11": 5, "11_8": 4, "11_9": 5, "11_10": 4}
    for cue, count in counts.items():
        check(f"sentences {cue}", len(sentence_ranges(line_text(cue, script))) == count)
        check(f"at most 100 words {cue}", len(line_text(cue, script).split()) <= 100)
    check("legal_map", arrivals(LEGAL_MAP) == [1, 1, 1])
    check("bad_map", arrivals(BAD_MAP)[BAD_DOUBLED] == 2 and arrivals(BAD_MAP)[BAD_MISSING] == 0)
    cycle = cycle_images(4, [[1, 3, 4]])
    inverse = cycle_images(4, [[1, 4, 3]])
    check("cycle_reads_back", cycle == [2, 1, 3, 0] and cycle == cycle_images(4, [[3, 4, 1]]))
    check("cycle_tex", cycle_tex(cycle) == r"(1\;3\;4)")
    check("reverse_differs", cycle != inverse)
    check("swap_is_own_reverse", cycle_images(4, [[1, 2]]) == cycle_images(4, [[2, 1]]))
    check("undo", then(cycle, inverse) == [0, 1, 2, 3] == then(inverse, cycle) and way_home(cycle) == inverse)
    tri, swap = cycle_images(6, [[1, 3, 5]]), cycle_images(6, [[2, 6]])
    both = then(tri, swap)
    check("commute", both == then(swap, tri))
    check("row", cards_by_slot(both) == [5, 6, 1, 4, 3, 2])
    check("restated", both == cycle_images(6, [[2, 6], [3, 5, 1]]))
    check("disjoint_tex", cycle_tex(both) == r"(1\;3\;5)(2\;6)")
    check("four_stays", both[3] == 3)
    c3, s3 = cycle_images(3, [[1, 2, 3]]), cycle_images(3, [[1, 2]])
    check("overlap", then(s3, c3) == cycle_images(3, [[1, 3]]) and then(c3, s3) == cycle_images(3, [[2, 3]]))

    def sw(a, b):
        return cycle_images(4, [[a, b]])

    check("descriptions", then(sw(1, 3), sw(1, 4)) == cycle == then(sw(1, 4), sw(3, 4)))
    before, after = differences(WITNESS), differences(after_swaps(WITNESS, ONE_SWAP))
    check("chips", before == [1, 3, 7, 2, 6, 4] and after == [-2, -3, 4, -1, 6, 7])
    check("deltas", delta(WITNESS) == 1008 and delta(after_swaps(WITNESS, ONE_SWAP)) == -1008)
    to = chip_destinations(before, after)
    check("destinations", to == [3, 1, 5, 0, 4, 2])
    check("in_place", to[REVERSES_IN_PLACE] == REVERSES_IN_PLACE and after[REVERSES_IN_PLACE] == -before[REVERSES_IN_PLACE])
    check("untouched", to[UNTOUCHED] == UNTOUCHED and after[UNTOUCHED] == before[UNTOUCHED])
    for pair, reversed_ in ((TRADE_AND_REVERSE, True), (TRADE_ONLY, False)):
        a, b = pair
        check(f"trade {pair}", (to[a], to[b]) == (b, a) and before[a] * before[b] == after[a] * after[b])
        check(f"reversal {pair}", (after[b] == -before[a]) == reversed_ and (after[a] == -before[b]) == reversed_)
    check("tally_start", tally_tex(WITNESS) == r"\Delta=1\cdot3\cdot7\cdot2\cdot6\cdot4=+1008")
    check("tally_end", tally_tex(after_swaps(WITNESS, ONE_SWAP)) == r"\Delta=(-2)\cdot(-3)\cdot4\cdot(-1)\cdot6\cdot7=-1008")
    words = [list(WITNESS)]
    for length in range(1, 7):
        words = [after_swaps(x, [pair]) for x in words for pair in PAIRS]
        expected = 1008 if length % 2 == 0 else -1008
        check(f"words {length}", all(delta(x) == expected for x in words))
    one, three = route_arrangements(ONE_SWAP), route_arrangements(THREE_SWAPS)
    check("route_three", three[1:] == [[2, 1, 4, 8], [2, 4, 1, 8], [4, 2, 1, 8]])
    check("same_end", one[-1] == three[-1])
    check("route_deltas", [delta(x) for x in three] == [1008, -1008, 1008, -1008])
    at = [0, 1, 2, 3]
    for a, b in THREE_SWAPS:
        at = [b if slot == a else a if slot == b else slot for slot in at]
    read = [0] * 4
    for value, slot in zip(WITNESS, at):
        read[slot] = value
    check("row_bookkeeping", read == after_swaps(WITNESS, THREE_SWAPS))
    r, s = corner_images(1, False), corner_images(0, True)
    check("r", r == cycle_images(4, [[1, 4, 3, 2]]) and cycle_tex(r) == r"(1\;4\;3\;2)")
    check("s", s == cycle_images(4, [[1, 2], [3, 4]]))
    check("r2", cycle_tex(then(r, r)) == r"(1\;3)(2\;4)")
    check("rs", cycle_tex(then(s, r)) == r"(2\;4)" and then(s, r) == corner_images(1, True))
    check("sr", cycle_tex(then(r, s)) == r"(1\;3)")
    check("thumb_rs", then(s, r) == corner_images(*elem_mul(R, S)))
    check("thumb_sr", then(r, s) == corner_images(*elem_mul(S, R)))
    check("noncommuting", then(s, r) != then(r, s))
    check("r_swaps", then(then(sw(1, 4), sw(1, 3)), sw(1, 2)) == r)
    check("parity", (swaps_needed(r), parity_word(3)) == (3, "odd") and (swaps_needed(then(r, r)), parity_word(2)) == (2, "even"))
    check("rings", (s[1], r[s[1]]) == (0, 3) and (r[0], s[r[0]]) == (3, 2))
    every = arrangements()
    check("24", len(every) == 24)
    kept = [p for p in every if not broken_edges(p)]
    symmetric = symmetries()
    check("8", len(kept) == 8 and all(p in symmetric for p in kept))
    check("inverse_closed", all((p in symmetric) == (way_home(p) in symmetric) for p in every))
    check("closed", all(then(a, b) in symmetric for a in symmetric for b in symmetric))
    check("square_unbroken", not broken_edges([0, 1, 2, 3]))
    check("torn", broken_edges(cycle_images(4, [[1, 2]])) == [(1, 2), (3, 0)])
    check("way_home", all(then(p, way_home(p)) == [0, 1, 2, 3] for p in every))
    return done


def main():
    script = (ROOT / "script.tsv").read_text(encoding="utf-8")
    source = (ROOT / "src" / "permutations.rs").read_text(encoding="utf-8")
    done = tests(script)
    print(f"mirrored assertions passed: {len(done)}")

    anchors = re.findall(r'const \w+: Anchor = \("(11_\d+)", (\d+), "([^"]+)"\);', source)
    for cue, j, phrase in anchors:
        text = line_text(cue, script)
        start, end = sentence_ranges(text)[int(j)]
        assert phrase in text[start:end], (cue, j, phrase)
    print(f"anchors in their sentences: {len(anchors)}")

    waits = re.findall(r'until_sentence\(stage, "(11_\d+)", (\d+)\)', source)
    for cue, j in waits:
        assert int(j) < len(sentence_ranges(line_text(cue, script))), (cue, j)
    print(f"until_sentence indexes in range: {len(waits)}")

    # Playback order: walk construct, stepping into witness_proof at its call.
    say = r'k\.say\(stage, "(11_\d+)"\)'
    inside = re.findall(say, source[source.index("fn witness_proof("):source.index("fn two_routes(")])
    assert inside == ["11_11"], inside
    said = []
    for cue, call in re.findall(say + r"|(witness_proof)\(stage", source[source.index("fn construct("):]):
        said += inside if call else [cue]
    assert len(said) == len(re.findall(say, source)), "a k.say outside construct and witness_proof"
    rows =[line.split("\t", 1)[0] for line in script.splitlines() if line.startswith("11_")]
    expected = ["11_1", "11_2", "11_3", "11_4", "11_5", "11_6", "11_7", "11_11", "11_8", "11_9", "11_10"]
    assert said == expected == rows, (said, rows)
    print("k.say order equals script row order:", " ".join(said))

    oracle = subprocess.run([sys.executable, "-I", "-B", str(ROOT / "oracle.py")], capture_output=True, text=True, check=True).stdout
    independent = dict(line.split("\t", 1) for line in oracle.splitlines() if line.startswith("permutations."))
    mine = {key: str(value) for key, value in facts().items()}
    assert set(independent) == set(mine), set(independent) ^ set(mine)
    wrong = {key: (mine[key], independent[key]) for key in mine if mine[key] != independent[key]}
    assert not wrong, wrong
    print(f"facts agree with oracle.py: {len(mine)} permutations.* keys; oracle prints {len(oracle.splitlines())} lines")
    keys = re.findall(r'"(permutations\.\w+)"', source)
    assert set(keys) == set(mine), set(keys) ^ set(mine)
    print(f"add_facts registers the same {len(set(keys))} keys, read from the Rust source")
    for key in sorted(mine):
        print(f"  {key}\t{mine[key]}")


if __name__ == "__main__":
    main()
