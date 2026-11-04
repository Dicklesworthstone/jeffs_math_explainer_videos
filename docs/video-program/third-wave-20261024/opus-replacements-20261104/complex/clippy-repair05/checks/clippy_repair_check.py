#!/usr/bin/env python3
"""The complex strict-Clippy repair, checked without compiling anything.

Run from the repository root:
  python3 -I -B <stage>/checks/clippy_repair_check.py <new empty scratch folder> [--write] [--candidate-is-base]

--write regenerates clippy-repair05.patch and MANIFEST.json.
--candidate-is-base is the negative control: it must fail.

This proves the edit is the enumerated one, that its logic and numbers hold, and
that the patch round-trips. Only root's guarded build can prove Clippy is clean.
"""

import hashlib
import json
import math
import re
import shutil
import struct
import subprocess
import sys
from pathlib import Path

STAGE = Path(__file__).resolve().parent.parent
REPO = Path.cwd()
BASE = STAGE / "base"
CAND = BASE if "--candidate-is-base" in sys.argv else STAGE / "candidate"
PATCH = STAGE / "clippy-repair05.patch"
MANIFEST = STAGE / "MANIFEST.json"
CRATE = "explainer_corpus/complex_analysis_visual"
SRC = f"{CRATE}/src"
LOG = "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-opus-rebuild-build04/command.log"
THEIRS = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex/reading-repair02"

scratch = Path(sys.argv[1])
write = "--write" in sys.argv[2:]
scratch.mkdir(parents=True, exist_ok=False)
failures = []


def check(ok, what):
    print(("  ok   " if ok else "  FAIL ") + what)
    if not ok:
        failures.append(what)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def text(root, rel):
    return (root / SRC / rel).read_text()


def changes(rel):
    """(removed lines, added lines) from base to candidate, git's own reading."""
    out = subprocess.run(
        ["git", "diff", "--no-index", "--no-color", "--no-ext-diff", "-U0", "--",
         str(BASE / SRC / rel), str(CAND / SRC / rel)],
        capture_output=True, text=True,
    ).stdout
    removed, added = [], []
    for line in out.splitlines():
        if line.startswith(("--- ", "+++ ")):
            continue
        if line.startswith("-"):
            removed.append(line[1:].strip())
        elif line.startswith("+"):
            added.append(line[1:].strip())
    return removed, added


# ---------------------------------------------------------------- the log's 12
print("1. every finding in root's log, at its logged line in the base")
log = (REPO / LOG).read_text()
FINDINGS = [
    ("chapters/residues.rs", 8, "unused import: `TAU`", "use std::f64::consts::{FRAC_PI_4, TAU};"),
    ("chapters/draw.rs", 415, "method `contains` is never used", "pub fn contains(&self, z: C) -> bool {"),
    ("chapters/draw.rs", 807, "method `leave` is never used", "pub fn leave(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {"),
    ("chapters/real_integral.rs", 66, "function `antiderivative` is never used", "pub fn antiderivative(x: f64) -> f64 {"),
    ("model.rs", 86, "method `sqrt` is never used", "pub fn sqrt(self) -> Self {"),
    ("model.rs", 384, "variant `Quartic` is never constructed", "Quartic,"),
    ("chapters/counting.rs", 846, "this assertion has a constant value", "assert!(COARSE_ROW_X >= -6.96);"),
    ("chapters/one_point.rs", 804, "this assertion has a constant value", "assert!(MOSAIC_CORE < model::SQUEEZE_RADII[1]);"),
    ("chapters/residues.rs", 358, "this assertion has a constant value", "assert!(BIG_R < 1.6);"),
    ("model.rs", 147, "suspicious use of `*` in `Div` impl", "self * o.recip()"),
    ("model.rs", 349, "float has excessive precision", "25.010_857_580_145_689,"),
    ("model.rs", 649, "negated comparison operators on partially ordered types", "if values.iter().any(|w| !(reach < w.abs())) {"),
]
for rel, n, message, source in FINDINGS:
    at = f"--> src/{rel}:{n}:"
    i = log.find(at)
    near = log[max(0, i - 400):i] if i >= 0 else ""
    line = text(BASE, rel).splitlines()[n - 1].strip()
    check(message in near and line == source, f"{rel}:{n} {message}")
errors = re.findall(r"^error: (?!unclosed table|could not compile)(.*)$", log, re.M)
check(len(errors) == 13, f"the strict run prints {len(errors)} errors: these 12, with `leave` shown for both targets")
check(log.count("95 passed; 0 failed") == 1, "the same run passed 95 tests before Clippy")

# ---------------------------------------------------------------- the exact edit
print("2. the candidate is exactly the enumerated edit")
EDITS = {
    "chapters/counting.rs": (
        ["assert!(COARSE_ROW_X >= -6.96);"],
        ["const { assert!(COARSE_ROW_X >= -6.96) };"],
    ),
    "chapters/one_point.rs": (
        ["assert!(MOSAIC_CORE < model::SQUEEZE_RADII[1]);"],
        ["const { assert!(MOSAIC_CORE < model::SQUEEZE_RADII[1]) };"],
    ),
    "chapters/draw.rs": (
        ["", "/// Everything the walker drew, for a fade (updater cleared first).",
         "pub fn leave(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {",
         "stage.clear_updaters(self.anchor, true);", "fade_out_all(stage, &self.mobs, rt)", "}"],
        ["#[cfg(test)]"],
    ),
    "chapters/real_integral.rs": (
        ["/// The antiderivative of 13_4, in two lines (it is [`antiderivative`])."],
        ["/// The antiderivative of 13_4, in two lines (the tests check it as `antiderivative`).", "#[cfg(test)]"],
    ),
    "chapters/residues.rs": (
        ["use std::f64::consts::{FRAC_PI_4, TAU};", "assert!(BIG_R < 1.6);"],
        ["use std::f64::consts::FRAC_PI_4;", "use std::f64::consts::TAU;", "", "const { assert!(BIG_R < 1.6) };"],
    ),
}
for rel, (want_removed, want_added) in EDITS.items():
    removed, added = changes(rel)
    check(removed == want_removed and added == want_added, f"{rel}: -{len(removed)} +{len(added)}, as enumerated")
removed, added = changes("model.rs")
check(removed == [
    "self * o.recip()",
    "25.010_857_580_145_689,",
    "/// 1/(1 + z⁴).",
    "Quartic,",
    "Func::Quartic => (ONE + z.powi(4)).recip(),",
    "Func::Quartic => quartic_roots().iter().map(|&p| (p, 1)).collect(),",
    "if values.iter().any(|w| !(reach < w.abs())) {",
], f"model.rs: exactly {len(removed)} lines removed")
for line in ["use std::cmp::Ordering;", "#[cfg(test)]", "Mul::mul(self, o.recip())", "25.010_857_580_145_69,",
             ".any(|w| reach.partial_cmp(&w.abs()) != Some(Ordering::Less))",
             "fn a_tie_or_a_nan_never_certifies_a_winding() {",
             "fn division_is_the_product_with_the_reciprocal() {"]:
    check(line in added, f"model.rs adds: {line}")

print("3. no gate weakened, no coverage lost")
ALL = list(EDITS) + ["model.rs"]
for rel in ALL:
    b, c = text(BASE, rel), text(CAND, rel)
    check(len(re.findall(r"#\s*!?\[\s*(?:allow|expect)\b", c)) == len(re.findall(r"#\s*!?\[\s*(?:allow|expect)\b", b)),
          f"{rel}: no allow or expect added")
    tests_b, tests_c = b.count("#[test]"), c.count("#[test]")
    want_tests = tests_b + (2 if rel == "model.rs" else 0)
    check(tests_c == want_tests, f"{rel}: tests {tests_b} -> {tests_c}")
    a_b = len(re.findall(r"\bassert(?:_eq)?!\(", b))
    a_c = len(re.findall(r"\bassert(?:_eq)?!\(", c))
    check(a_c >= a_b, f"{rel}: assertions {a_b} -> {a_c}")
model = text(CAND, "model.rs")
draw = text(CAND, "chapters/draw.rs")
real = text(CAND, "chapters/real_integral.rs")
resid = text(CAND, "chapters/residues.rs")
check("Quartic" not in model, "the variant nothing constructs is gone, arms included")
for rel in ("facts.rs", "chapters/real_integral.rs", "chapters/closing.rs"):
    source = (REPO / SRC / rel).read_text().partition("#[cfg(test)]\nmod tests")[0]
    check("quartic_roots()" in source, f"quartic_roots stays live: {rel} calls it outside tests")
check("for p in quartic_roots() {" in model.partition("#[cfg(test)]\nmod tests")[2], "and model.rs's test still covers the roots")
check("z.sqrt() * z.sqrt()" in model.partition("#[cfg(test)]\nmod tests")[2], "C::sqrt keeps its test")
check("the_antiderivative_shown_is_right" in real.partition("#[cfg(test)]\nmod tests")[2], "antiderivative keeps its test")
check(".contains(z)" in (REPO / SRC / "chapters/boundary.rs").read_text().partition("#[cfg(test)]\nmod tests")[2],
      "Plane::contains keeps its tests")
check(re.search(r"\bfade_out_all\b", draw.partition("#[cfg(test)]\nmod tests")[0]) is not None,
      "draw.rs still has fade_out_all: removing `leave` strands no import")
check(bool(re.search(r"pub fn fade_out_all", draw)) or "fade_out_all" in draw.split("impl ")[0],
      "fade_out_all is defined or imported there for the chapters")
leave = re.search(r"impl Walker \{.*?\n\}\n", draw, re.S).group(0)
check("fn leave" not in leave and "self.anchor" in leave and "self.mobs" in leave,
      "Walker's anchor and mobs are still read, so no field goes dead")
check("TAU" not in resid.partition("#[cfg(test)]\nmod tests")[0].replace("use std::f64::consts::FRAC_PI_4;", ""),
      "residues.rs uses TAU only in tests")
check("use std::f64::consts::TAU;" in resid.partition("#[cfg(test)]\nmod tests")[2], "and imports it there")
check("[`antiderivative`]" not in real, "no doc link points at the cfg(test) item")
# dead-code analysis cannot vouch for what only the removed arms kept alive
others = [f.name for f in (REPO / SRC).rglob("*.rs") if f.name != "model.rs" and "Quartic" in f.read_text()]
check(not others, f"no other source names the removed variant, not even in a pattern {others}")
live = model.partition("#[cfg(test)]\nmod tests")[0]
check("Func::Quintic => z.powi(5) - z.scale(3.0) + ONE," in live, "the removed arm strands nothing: C::powi and ONE keep a live caller")
check("Func::Bump => (ONE + z * z).recip()," in live, "and so does C::recip")

# ---------------------------------------------------------------- the numbers
print("4. what stays numerically the same")
old, new = float("25.010857580145689"), float("25.01085758014569")
check(struct.pack("<d", old) == struct.pack("<d", new), f"the shorter literal is the same double, {struct.pack('>d', new).hex()}")
check("25.010_857_580_145_69," in model, "and it is the literal in the candidate")
oracle = (REPO / CRATE / "oracle.py").read_text()
check("25.010857580145689" in oracle, "the oracle keeps the published digits, an equal double")

GRID = [float("nan"), float("-inf"), -1.0, -0.0, 0.0, 5e-324, 0.5, 1.0, 1.0000000000000002, float("inf")]


def partial_cmp_is_less(a, b):
    """f64::partial_cmp(a, b) == Some(Less)."""
    return not (math.isnan(a) or math.isnan(b)) and a < b


bad = [(a, b) for a in GRID for b in GRID if (not (a < b)) != (not partial_cmp_is_less(a, b))]
check(not bad and "Some(Ordering::Less)" in model, f"!(a < b) equals cmp != Some(Less) on all {len(GRID) ** 2} pairs, NaN and ties included")

# the new test's cases, by the certificate's own arithmetic
r, l, n = 1.0, 3.0, 64
reach = l * math.tau * r / n
passes = lambda a, b: partial_cmp_is_less(a, b)  # noqa: E731
check(passes(reach, math.hypot(2 * reach, 0.0)), "new test: a constant at twice the reach passes")
check(math.hypot(reach, 0.0) == reach and not passes(reach, math.hypot(reach, 0.0)), "new test: a constant at the reach is a tie, refused")
check(not passes(reach, math.hypot(float("nan"), 0.0)), "new test: a NaN value is refused")
check(not passes(l * math.nan * r / n, 1.0), "new test: a NaN bound is refused")
check(not passes(math.inf * math.tau * r / n, 1.0), "new test: an infinite bound is refused")

# what each const assertion claims must hold, or the crate stops compiling
counting = text(BASE, "chapters/counting.rs")
one_point = text(BASE, "chapters/one_point.rs")
residues = text(BASE, "chapters/residues.rs")
row_x = float(re.search(r"const COARSE_ROW_X: f64 = (-?[0-9.]+);", counting).group(1))
core = float(re.search(r"const MOSAIC_CORE: f64 = ([0-9.]+);", one_point).group(1))
radii = re.search(r"pub const SQUEEZE_RADII: \[f64; 2\] = \[([0-9.]+), ([0-9.]+)\];", model)
big = float(re.search(r"pub const BIG_R: f64 = ([0-9.]+);", residues).group(1))
check(row_x >= -6.96, f"const assert holds: COARSE_ROW_X {row_x} >= -6.96")
check(core < float(radii.group(2)), f"const assert holds: MOSAIC_CORE {core} < SQUEEZE_RADII[1] {radii.group(2)}")
check(big < 1.6, f"const assert holds: BIG_R {big} < 1.6")
check(all(re.search(r"^const |^pub const ", s, re.M) for s in (counting, one_point, residues)), "all three are `const` items, usable in a const block")

# ---------------------------------------------------------------- the patch
print("5. the patch")
names = sorted(p.relative_to(BASE).as_posix() for p in BASE.rglob("*") if p.is_file())
if write and CAND != BASE:
    out = subprocess.run(
        ["git", "diff", "--no-index", "--no-color", "--no-ext-diff", "--no-renames", "--", "base", "candidate"],
        cwd=STAGE, capture_output=True, text=True,
    )
    if out.returncode != 1:
        sys.exit(f"git diff exited {out.returncode}: {out.stderr}")
    lines = []
    for line in out.stdout.splitlines(keepends=True):
        if line.startswith(("diff --git ", "--- a/", "+++ b/")):
            line = line.replace("a/base/", "a/").replace("b/candidate/", "b/")
        lines.append(line)
    PATCH.write_text("".join(lines))
if CAND != BASE:
    forward = scratch / "forward"
    shutil.copytree(BASE, forward)
    done = subprocess.run(["git", "apply", str(PATCH.resolve())], cwd=forward, capture_output=True, text=True)
    check(done.returncode == 0, f"it applies to a copy of base {done.stderr.strip()}")
    same = [n_ for n_ in names if (forward / n_).read_bytes() == (CAND / n_).read_bytes()]
    check(len(same) == len(names), f"and gives candidate byte for byte ({len(same)} of {len(names)})")
    reverse = scratch / "reverse"
    shutil.copytree(CAND, reverse)
    done = subprocess.run(["git", "apply", "-R", str(PATCH.resolve())], cwd=reverse, capture_output=True, text=True)
    check(done.returncode == 0, f"reversed, it applies to a copy of candidate {done.stderr.strip()}")
    same = [n_ for n_ in names if (reverse / n_).read_bytes() == (BASE / n_).read_bytes()]
    check(len(same) == len(names), f"and gives base byte for byte ({len(same)} of {len(names)})")
    drift = [n_ for n_ in names if sha(REPO / n_) != sha(BASE / n_)]
    check(not drift, f"base equals the canonical crate's current bytes (drift: {drift})")
    done = subprocess.run(["git", "apply", "--check", str(PATCH.resolve())], cwd=REPO, capture_output=True, text=True)
    check(done.returncode == 0, f"git apply --check accepts it from the repository root {done.stderr.strip()}")
    whole = scratch / "whole" / CRATE
    shutil.copytree(REPO / SRC, whole / "src")
    for n_ in names:
        shutil.copy2(CAND / n_, scratch / "whole" / n_)
    done = subprocess.run(["rustfmt", "--check", "--edition", "2024", "--config-path", str(REPO / "rustfmt.toml"), "src/main.rs"],
                          cwd=whole, capture_output=True, text=True)
    version = subprocess.run(["rustfmt", "--version"], cwd=REPO, capture_output=True, text=True).stdout.strip()
    check(done.returncode == 0, f"the whole crate with the candidate parses and is formatted ({version}; parse only, not a compile)")

    # for root's merge only: not a failure if the other stage moves or conflicts
    print("6. against reading-repair02 as it stands now, on scratch copies (information for the merge)")
    theirs = REPO / THEIRS
    per_file = subprocess.run(["git", "apply", "--numstat", str(PATCH.resolve())], cwd=REPO, capture_output=True, text=True)
    print("  " + per_file.stdout.strip().replace("\n", "\n  ").replace(SRC + "/", ""))
    overlay = scratch / "overlay"
    for n_ in names:
        staged = theirs / n_
        out_path = overlay / n_
        out_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(staged if staged.exists() else BASE / n_, out_path)
    for n_ in names:
        staged = (theirs / n_).exists()
        done = subprocess.run(["git", "apply", "--check", f"--include={n_}", str(PATCH.resolve())], cwd=overlay, capture_output=True, text=True)
        verdict = "applies" if done.returncode == 0 else "CONFLICT: " + done.stderr.strip().splitlines()[0][:120]
        print(f"  {n_.replace(SRC + '/', '')}: {'staged by them' if staged else 'not staged by them'}; {verdict}")
    their_patch = theirs / "reading-repair02.patch"
    print(f"  their patch now: sha256 {sha(their_patch)}")
    their_added = [x[1:] for x in their_patch.read_text().splitlines() if x.startswith("+") and not x.startswith("+++")]
    hits = [x.strip() for x in their_added if re.search(r"\.sqrt\(|antiderivative|Quartic|\.leave\(|\bTAU\b", x)]
    print(f"  their added lines naming sqrt, antiderivative, Quartic, leave or TAU: {hits}")
    receivers = sorted({m for x in their_added for m in re.findall(r"([\w\[\]+. ]*?)\.contains\(", x)})
    print(f"  their added .contains( receivers (all text, none a Plane): {receivers}")
    merged = scratch / "merged"
    shutil.copytree(REPO / SRC, merged / SRC)
    for f in (theirs / SRC).rglob("*"):
        if f.is_file():
            shutil.copy2(f, merged / f.relative_to(theirs))
    conflicts = 0
    for n_ in names:
        done = subprocess.run(["git", "merge-file", "-q", str(merged / n_), str(BASE / n_), str(CAND / n_)],
                              capture_output=True, text=True)
        conflicts += done.returncode
        print(f"  git merge-file {n_.replace(SRC + '/', '')}: {done.returncode} conflicts")
    merged_real = (merged / SRC / "chapters/real_integral.rs").read_text()
    print("  merged real_integral.rs gates the right item:", "#[cfg(test)]\npub fn antiderivative(" in merged_real)
    if conflicts == 0:
        done = subprocess.run(["rustfmt", "--check", "--edition", "2024", "--config-path", str(REPO / "rustfmt.toml"), "src/main.rs"],
                              cwd=merged / CRATE, capture_output=True, text=True)
        print(f"  rustfmt --check through main.rs on the merged crate: exit {done.returncode}")
    for rel in ("chapters/draw.rs", "chapters/real_integral.rs", "chapters/counting.rs", "chapters/one_point.rs"):
        staged = theirs / SRC / rel
        if staged.exists():
            code = staged.read_text().partition("#[cfg(test)]\nmod tests")[0]
            base_code = text(BASE, rel).partition("#[cfg(test)]\nmod tests")[0]
            same_calls = sorted(re.findall(r"(\w+(?:\[\d\])?)\.leave\(", code)) == sorted(re.findall(r"(\w+(?:\[\d\])?)\.leave\(", base_code))
            uses_anti = len(re.findall(r"\bantiderivative\(", code)) > (1 if "pub fn antiderivative" in code else 0)
            print(f"  {rel.replace('chapters/', '')}: same .leave( callers as base: {same_calls}; non-test antiderivative call: {uses_anti}")

if write and CAND != BASE and not failures:
    MANIFEST.write_text(json.dumps({
        "stage": "clippy-repair05",
        "grant": "Agent Mail 1361",
        "log": LOG,
        "patch": {"path": PATCH.name, "sha256": sha(PATCH), "strip": 1, "apply_from": "repository root"},
        "files": [
            {"path": n_, "base_sha256": sha(BASE / n_), "candidate_sha256": sha(CAND / n_)} for n_ in names
        ],
        "not_done": "no compile, Clippy, Cargo test, render, listening, fine sync or AV acceptance",
    }, indent=2) + "\n")
    print("  wrote", PATCH.name, "and", MANIFEST.name)

if PATCH.exists():
    print("patch sha256", sha(PATCH))
for n_ in names:
    print(sha(BASE / n_)[:16], sha(CAND / n_)[:16], n_.replace(SRC + "/", ""))
print()
if failures:
    print(f"FAILED {len(failures)}")
    sys.exit(1)
print("PASS (source text, exact logic and round trip; no compile, no Clippy, no test run)")
