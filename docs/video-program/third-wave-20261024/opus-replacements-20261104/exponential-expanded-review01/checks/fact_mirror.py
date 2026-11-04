#!/usr/bin/env python3
"""Mirror the Rust fact exporters in IEEE doubles and compare them with the oracles, key by key.

Python floats and Rust f64 share IEEE 754 arithmetic. Both formatters round the exact
binary value and break exact ties to even, so a mismatch here predicts a facts-check
mismatch. This is a mirror, not native execution. Run from the repo root.
"""
import subprocess
import sys

ROOT = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
FACTS = ROOT + "exponential-rate-facts01/"
SERIES = ROOT + "exponential-expanded-series01/"
failed = False


def oracle(path):
    run = subprocess.run([sys.executable, "-I", "-B", path], capture_output=True, text=True)
    assert run.returncode == 0, (path, run.stderr)
    return run.stdout


def rows(text):
    return dict(line.split("\t", 1) for line in text.splitlines())


def compare(name, ours, theirs):
    global failed
    only_a = sorted(set(ours) - set(theirs))
    only_b = sorted(set(theirs) - set(ours))
    wrong = [(k, ours[k], theirs[k]) for k in ours if k in theirs and ours[k] != theirs[k]]
    print(
        f"{name}: {len(ours)} mirrored rows, {len(theirs)} oracle rows, "
        f"{len(only_a)} only in the mirror, {len(only_b)} only in the oracle, {len(wrong)} differ"
    )
    for row in wrong:
        print("  DIFFERS", *row)
    failed |= bool(only_a or only_b or wrong)


# ---- rate_model.rs at exponential-rate-facts01/candidate/src
START, DURATION, COUNTS = 1.0, 1.0, (4, 8, 16)


def proportional(n):
    h = DURATION / float(n)
    y = START
    points = [(0.0, y)]
    for k in range(1, n + 1):
        y += h * y
        points.append((float(k) * h, y))
    return points


def constant(n):
    return [(float(k) * DURATION / float(n), START + float(k) * DURATION / float(n)) for k in range(n + 1)]


def reading(height, rate):
    return f"height {height:.2f}: slope {rate:.2f}"


def endpoints(counts):
    ends = ", ".join(f"{proportional(n)[n][1]:.3f}" for n in counts)
    return ", ".join(map(str, counts)) + " steps: " + ends


rate = {"rate.start": f"{START:.6f}", "rate.duration": f"{DURATION:.6f}"}
for n in COUNTS:
    p, c = proportional(n), constant(n)
    rate[f"rate.{n}.steps"] = str(n)
    for k, ((t, y), (_, linear)) in enumerate(zip(p, c)):
        rate[f"rate.{n}.{k}.t"] = f"{t:.6f}"
        rate[f"rate.{n}.{k}.proportional"] = f"{y:.6f}"
        rate[f"rate.{n}.{k}.constant"] = f"{linear:.6f}"
        rate[f"rate.{n}.{k}.right_readout"] = reading(y, y)
        rate[f"rate.{n}.{k}.left_readout"] = reading(linear, START)
    rate[f"rate.{n}.last_step"] = f"last step: slope {p[n - 1][1]:.2f}, height now {p[n][1]:.2f}"
for k in range(1, 4):
    rate[f"rate.refinement.{k}"] = endpoints(COUNTS[:k])

stored = open(FACTS + "oracle.tsv", encoding="utf-8").read()
fresh = oracle(FACTS + "candidate/oracle.py")
print("rate oracle: fresh run equals stored oracle.tsv:", fresh == stored, "| rows", len(rows(fresh)))
failed |= fresh != stored
compare("rate", rate, {k: v for k, v in rows(fresh).items() if k.startswith("rate.")})

# ---- series_model.rs at exponential-expanded-series01
RADIUS, TOP, SEGMENTS, TAIL = 1.5, 5.0, 60, 4


def plot_x(j):
    return RADIUS * float(j) / float(SEGMENTS)


def partial(x, n):
    term = 1.0
    total = term
    for k in range(1, n + 1):
        term *= x / float(k)
        total += term
    return total


def factorial(n):
    out = 1
    for k in range(1, n + 1):
        out *= k
    return out


def tail_bound(radius, n):
    ratio = radius / float(n + 2)
    return radius ** (n + 1) / float(factorial(n + 1)) / (1.0 - ratio)


series = {
    "factorials.plot.radius": f"{RADIUS:.6f}",
    "factorials.plot.top": f"{TOP:.6f}",
    "factorials.plot.segments": str(SEGMENTS),
}
for n in range(5):
    d = factorial(n)
    series[f"factorials.coefficient.{n}"] = "1" if d == 1 else f"1/{d}"
    series[f"factorials.denominator.{n}"] = str(d)
    for j in range(SEGMENTS + 1):
        x = plot_x(j)
        series[f"factorials.curve.{n}.{j}.x"] = f"{x:.6f}"
        series[f"factorials.curve.{n}.{j}.y"] = f"{partial(x, n):.6f}"
series["factorials.tail.order"] = str(TAIL)
series["factorials.tail.bound"] = f"{tail_bound(RADIUS, TAIL):.5f}"

stored = open(SERIES + "oracle01.tsv", encoding="utf-8").read()
fresh = oracle(SERIES + "oracle.py")
print("series oracle: fresh run equals stored oracle01.tsv:", fresh == stored, "| rows", len(rows(fresh)))
failed |= fresh != stored
compare("series", series, rows(fresh))
value = tail_bound(RADIUS, TAIL)
print(f"tail bound: the Rust path gives {value!r}; exact value 27/320 = 0.084375; nearest double {27 / 320!r}")
print(f"  Rust prints {value:.5f}; the oracle prints {27 / 320:.5f}")
print("RESULT", "MISMATCH (see DIFFERS rows)" if failed else "PASS")
sys.exit(1 if failed else 0)
