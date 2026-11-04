"""Independent exact check of the tail bound, the recurrence and the 03_7 ladder, written before reading math_exact.py.

Usage from the repo root:  python3 -I -B math_exact.py SCRATCH_DIR
Exact fractions throughout. The model functions under test are cut verbatim from the candidates and compiled with
rustc (std only) into SCRATCH_DIR. Nothing is written in the repo. Arithmetic only: no picture or listening claim.
"""
import math
import os
import re
import subprocess
import sys
from fractions import Fraction as Fr

R_ = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
C = R_ + "exponential-abc-teaching-repair01/candidate/"
S = os.path.abspath(sys.argv[1])
os.makedirs(S, exist_ok=True)
fails = 0


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def t(r, k):
    return Fr(r) ** k / math.factorial(k)


def enclosure(r, n, extra=80):
    """Lower and upper bounds on the sum of t(r, k) for k > n. The upper end uses the halving step only at index
    n + extra, far past 2r, so it does not lean on the claim being tested at n."""
    m = n + extra
    assert m + 2 >= 2 * r
    low = sum(t(r, k) for k in range(n + 1, m + 1))
    return low, low + 2 * t(r, m + 1)


def claim(r, n):
    return 2 * t(r, n + 1)


def oracle(path):
    out = subprocess.run(["python3", "-I", "-B", path], capture_output=True, text=True, check=True).stdout
    return dict(line.split("\t", 1) for line in out.splitlines())


def slices():
    rows = {}
    for name in ("series01", "product01"):
        for line in open(C + name + "/script-slice.tsv", encoding="utf-8").read().splitlines():
            key, text = line.split("\t", 1)
            rows[key] = text
    return rows


def cut(source, signature):
    at = source.index(signature)
    start = source.rfind("\n", 0, at) + 1
    depth = 0
    for i in range(at, len(source)):
        if source[i] == "{":
            depth += 1
        elif source[i] == "}":
            depth -= 1
            if depth == 0:
                return source[start : i + 1]
    raise ValueError(signature)


R = Fr(3, 2)
series = oracle(C + "series01/oracle.py")
product = oracle(C + "product01/oracle.py")
script = slices()

print("--- A. the printed bounds at R = 3/2")
for n, printed in ((4, "0.12657"), (8, "0.00022")):
    bound = claim(R, n)
    low, high = enclosure(R, n)
    first = t(R, n + 1)
    print(f"  degree {n}: first omitted {float(first):.9f}; 2 x that {float(bound):.9f}; true tail in "
          f"[{float(low):.9f}, {float(high):.9f}]; math.exp route {math.exp(1.5) - sum(float(t(R, k)) for k in range(n + 1)):.9f}")
    ck(series[f"factorials.tail.{n}.bound"] == printed, f"oracle row prints {printed}")
    ck(Fr(printed) >= bound, f"degree {n}: the printed number is at or above 2 t(N+1)")
    ck(Fr(printed) - bound < Fr(1, 10**5), f"degree {n}: rounded up by less than one in the fifth place")
    ck(high <= bound, f"degree {n}: the true tail is under the bound")
    ck(n + 2 >= 2 * R, f"degree {n}: N+2 >= 2R ({n + 2} >= {2 * R})")
    ck(all(t(R, k + 1) / t(R, k) <= Fr(1, 2) for k in range(n + 1, 60)), f"degree {n}: every later ratio is at most 1/2")
    ck(all(t(R, k) <= first / 2 ** (k - n - 1) for k in range(n + 1, 60)), f"degree {n}: every later term is under its ceiling")
print(f"  old printed bound 0.08438 = t5/(1 - 1/4) = {float(t(R, 5) * Fr(4, 3)):.7f}; also true, on a different hypothesis")

print("--- B. every |x| <= R, not only x = R")
for n in (4, 8):
    bound = claim(R, n)
    ok = True
    for j in range(-12, 13):
        x = R * Fr(j, 12)
        total = sum(t(abs(x), k) for k in range(n + 1, n + 60))
        ok = ok and total <= bound
    ck(ok, f"degree {n}: the absolute tail stays under the bound at 25 inputs across the interval")

print("--- C. the hypothesis, swept and at its edges")
grid = [Fr(a, 2) for a in (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 16, 20)]
under, broke_guard, broke_old, broke_none = 0, [], [], []
for r in grid:
    for n in range(0, 19):
        low, high = enclosure(r, n)
        bound = claim(r, n)
        if n + 2 >= 2 * r:
            under += 1
            if high > bound:
                broke_guard.append((str(r), n))
        elif low > bound:
            broke_none.append((str(r), n))
            if n + 2 > r:
                broke_old.append((str(r), n))
ck(not broke_guard, f"N+2 >= 2R: no failure in {under} cases")
print(f"  without the guard the claim is false in {len(broke_none)} of the other cases, first {broke_none[:4]}")
ck(("3/2", 0) in broke_none, "edge: R = 3/2, N = 0 fails without the guard: 2 t1 = 3 under e^1.5 - 1")
ck(bool(broke_old), f"mutation caught: the old hypothesis N+2 > R paired with the factor 2 is false in {len(broke_old)} cases")
low, high = enclosure(R, 1)
ck(high <= claim(R, 1), "edge: R = 3/2, N = 1 is the first degree the guard allows, and the bound holds there")
low, high = enclosure(Fr(3), 4)
ck(high <= claim(Fr(3), 4) and Fr(6) == 2 * Fr(3), "edge: equality N+2 = 2R (R = 3, N = 4) holds")

print("--- D. mutations that must fail")
ck(enclosure(R, 4)[0] > t(R, 5), "factor 1 instead of 2 is below the true tail at degree 4")
ck(Fr("0.12656") < claim(R, 4), "rounding degree 4 down prints under the bound")
ck(Fr("0.00021") < claim(R, 8), "rounding degree 8 down prints under the bound")

print("--- E. the model functions, cut verbatim from series_model.rs")
model = open(C + "series01/series_model.rs", encoding="utf-8").read()
parts = [cut(model, "fn " + name + "(") for name in ("factorial", "term_size", "halving_from", "tail_bound", "envelope", "tail_label")]
product_src = open(C + "product01/product_model.rs", encoding="utf-8").read()
consts = "\n".join(re.findall(r"pub const LADDER_[A-Z]+: i32 = -?\d+;", product_src))
ladder = [cut(product_src, "pub fn " + name + "(") for name in ("ladder_whole", "ladder_fractions", "ladder_negatives")]
main = (
    "#![allow(dead_code)]\n" + consts + "\n" + "\n".join(parts + ladder) + r"""
fn main() {
    for (r, n) in [(1.5, 0), (1.5, 1), (1.5, 4), (1.5, 8), (1.5, 18), (1.5, 19), (3.0, 3), (3.0, 4), (-1.0, 4), (f64::NAN, 4)] {
        println!("tail {r} {n} {:?}", tail_bound(r, n));
    }
    for n in [4usize, 8] {
        println!("label {n} {}", tail_label(tail_bound(1.5, n).unwrap()));
    }
    println!("halving {}", halving_from(1.5));
    println!("whole {:?}", ladder_whole());
    println!("fractions {:?}", ladder_fractions());
    println!("negatives {:?}", ladder_negatives());
}
"""
)
path = os.path.join(S, "model_cut.rs")
with open(path, "w", encoding="utf-8") as out:
    out.write(main)
binary = os.path.join(S, "model_cut")
r = subprocess.run(["rustc", "--edition", "2024", "-o", binary, path], capture_output=True, text=True)
ck(r.returncode == 0, "the cut functions compile", r.stderr[-400:].replace(S, "<scratch>") if r.returncode else "")
out = subprocess.run([binary], capture_output=True, text=True).stdout.splitlines()
for line in out:
    print("  " + line[:120])
rows = dict((" ".join(line.split()[:3 if line.startswith("tail") else 1]), " ".join(line.split()[3 if line.startswith("tail") else 1:])) for line in out if not line.startswith("label"))
labels = dict((line.split()[1], line.split()[2]) for line in out if line.startswith("label"))
ck(rows["tail 1.5 0"] == "None", "tail_bound refuses R = 1.5, N = 0")
ck(rows["tail 1.5 1"].startswith("Some(2.25"), "tail_bound allows R = 1.5, N = 1, at 2.25")
ck(rows["tail 3 3"] == "None" and rows["tail 3 4"].startswith("Some"), "tail_bound changes over at N + 2 = 2R for R = 3")
ck(rows["tail -1 4"] == "None" and rows["tail NaN 4"] == "None", "tail_bound refuses a negative or non-finite radius")
ck(labels == {"4": "0.12657", "8": "0.00022"}, "tail_label prints the two figures from the native arithmetic")
ck(rows["halving"] == "3", "halving_from(1.5) = 3")

print("--- F. the ratio row, the ceilings and the derivative shift")
for i in range(1, 9):
    ck(series[f"factorials.ratio.{i}"] == f"{(R / i).numerator}/{(R / i).denominator}", f"ratio {i} = R/{i}")
ck(min(i for i in range(1, 20) if R / i <= Fr(1, 2)) == int(series["factorials.halving.from"]), "the first ratio at most 1/2 is index 3")
for k in range(5, 10):
    ck(abs(Fr(series[f"factorials.envelope.{k}"]) - t(R, 5) / 2 ** (k - 5)) <= Fr(5, 10**6), f"ceiling over term {k} matches t5 / 2^(k-5) to print precision")
ck(all(k * Fr(1, math.factorial(k)) == Fr(1, math.factorial(k - 1)) for k in range(1, 30)), "d/dx x^k/k! = x^(k-1)/(k-1)!: the shift is exact")
ck(all(enclosure(R, n - 1)[1] <= 2 * t(R, n) for n in range(2, 19)), "the derivative series after degree N-1 obeys the same bound, one index down, from N = 2")

print("--- G. the recurrence, from coefficient matching alone")
u = [Fr(1)]
for k in range(0, 12):
    # Derivative of sum u_k x^k has coefficient (k+1) u_{k+1} at x^k. Match it to u_k and solve.
    u.append(u[k] / (k + 1))
ck(all(u[k] == Fr(1, math.factorial(k)) for k in range(13)), "matching gives 1/k! through k = 12")
ck(all((k + 1) * u[k + 1] == u[k] for k in range(12)), "(k+1) u(k+1) = u(k): the reason behind the chain")
spoken = {"one": Fr(1), "one half": Fr(1, 2), "one sixth": Fr(1, 6), "one twenty-fourth": Fr(1, 24)}
sentence = re.search(r"We get ([^.]*)\.", script["02_3"]).group(1)
said = [spoken[w.strip()] for w in sentence.split(",")]
ck(said == u[:5], "02_3 s2 says the five values in order", sentence)
for k in range(5):
    shown = series[f"factorials.coefficient.{k}"]
    ck(Fr(shown) == u[k], f"coefficient row {k} = {shown}")
    run = series[f"factorials.running.{k}"]
    product_value = math.prod(int(p) for p in run.split(r"\cdot "))
    ck(product_value == math.factorial(k), f"running product {k}: {run} = {k}!")
ck(all(u[i] == u[i - 1] / i for i in range(1, 5)), "the chain's divisors are 1, 2, 3, 4: each is the next whole number")
print("  voiced anywhere in 02_1 to 03_7: 'u' as a word:", sum(len(re.findall(r"\bu\b", text)) for text in script.values()),
      "| 'zero factorial':", sum(text.count("zero factorial") for text in script.values()))

print("--- H. 03_7: the ladder, in exact interval arithmetic")


def e_interval(q, n=40):
    centre = sum(Fr(q) ** k / math.factorial(k) for k in range(n + 1))
    radius = 2 * t(abs(Fr(q)), n + 1)
    assert n + 2 >= 2 * abs(Fr(q))
    return centre - radius, centre + radius


def mul(a, b):
    corners = [a[0] * b[0], a[0] * b[1], a[1] * b[0], a[1] * b[1]]
    return min(corners), max(corners)


def power(a, m):
    out = (Fr(1), Fr(1))
    for _ in range(m):
        out = mul(out, a)
    return out


def holds_one(a):
    return a[0] <= 1 <= a[1]


parsed = {name: [Fr(v).limit_denominator(4) for v in re.findall(r"-?\d+\.?\d*", rows[name])] for name in ("whole", "fractions", "negatives")}
marks = parsed["whole"] + parsed["fractions"] + parsed["negatives"]
wanted = [Fr(j, 4) for j in range(-8, 13)]
ck(sorted(marks) == wanted and len(marks) == len(set(marks)), f"the three clouds mark each of the {len(wanted)} quarter points from -2 to 3 exactly once")
ck(all(v > 0 for v in parsed["fractions"]) and all(v < 0 for v in parsed["negatives"]), "'fractions' are all positive and 'negatives' all negative")
ck(int(product["product.ladder.fractions"]) == len(parsed["fractions"]) and int(product["product.ladder.negatives"]) == len(parsed["negatives"]), "the fact rows count the same clouds")
e = e_interval(1)
ck(e[0] > Fr("2.71828") and e[1] < Fr("2.71829") and product["product.e"] == "2.71828", "E(1) is in (2.71828, 2.71829), and the fact row prints 2.71828")
ck(holds_one(mul(power(e, 2), (1 / power(e_interval(2), 1)[1], 1 / power(e_interval(2), 1)[0]))), "E(2) = e times e")
ck(holds_one(mul(power(e, 3), (1 / e_interval(3)[1], 1 / e_interval(3)[0]))), "E(3) = e cubed")
half = e_interval(Fr(1, 2))
ck(power(half, 2)[0] <= e[1] and e[0] <= power(half, 2)[1], "E(1/2)^2 = e")
ck(half[0] > 0, "E(1/2) > 0, as 03_6 shows and as the screen line states")
ck(mul((-half[1], -half[0]), (-half[1], -half[0]))[0] <= e[1], "control: -E(1/2) squares to e as well, so 'E > 0' on the screen line is needed")
ok = True
for q in wanted:
    fourth = power(e_interval(q), 4)
    steps = int(4 * q)
    if steps >= 0:
        target = power(e, steps)
        ok = ok and fourth[0] <= target[1] and target[0] <= fourth[1]
    else:
        ok = ok and holds_one(mul(fourth, power(e, -steps)))
ck(ok, "E(q)^4 = e^(4q) at every quarter point, negatives included")
ck(all(holds_one(mul(e_interval(q), e_interval(-q))) for q in parsed["negatives"]), "E(q) E(-q) = 1 at every negative mark: the reciprocal step")
ck(min(parsed["whole"] + parsed["fractions"]) == 0, "without reciprocals the marks stop at 0: continuity alone never reaches a negative input, so the clause is needed")

print("RESULT", "PASS" if not fails else f"{fails} FAIL")
sys.exit(1 if fails else 0)
