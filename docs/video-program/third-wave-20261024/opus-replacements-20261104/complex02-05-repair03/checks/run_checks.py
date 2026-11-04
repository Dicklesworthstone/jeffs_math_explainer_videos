"""Regenerate this folder's patch, binding deltas and check outputs. No Cargo, render or TTS.

Writes only under this folder and under SCRATCH. Canonical is read, never changed; the run
stops first if canonical no longer matches checks/baseline.sha256. The desk checks and the binder
are the frozen copies in ../complex, read in place. `python3 -B`, so they leave no bytecode.

Usage: python3 -I -B run_checks.py REPO FMN_CRATES SCRATCH
"""

import difflib
import hashlib
import json
import re
import shutil
import subprocess
import sys
from collections import Counter
from pathlib import Path

REPO = Path(sys.argv[1]).resolve()
FMN = Path(sys.argv[2]).resolve()
SCRATCH = Path(sys.argv[3]).resolve()
HERE = Path(__file__).resolve().parent.parent
OUT = HERE / "checks/outputs"
BIND = HERE / "bindings"
CRATE_REL = "explainer_corpus/complex_analysis_visual"
CRATE = REPO / CRATE_REL
STAGED = HERE / CRATE_REL / "src/chapters"
OPUS = HERE.parent
TOOLS = OPUS / "complex/checks"
BINDER = OPUS / "complex/reading-repair02/bind_oliver_plan.py"
CUES = OPUS / "complex02-05-picture-review01/cues.txt"
NAMES = ("turning", "two_planes", "reading_color", "gallery")
PY = [sys.executable, "-I", "-B"]
MAP_REL = "docs/narration-direction-map-oliver10.json"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def clean(text):
    for path, name in ((REPO, "<repo>"), (FMN, "<fmn>"), (SCRATCH, "<scratch>"), (Path.home(), "<home>")):
        text = text.replace(str(path), name)
    return text


def save(name, text):
    OUT.mkdir(exist_ok=True)
    (OUT / name).write_text(clean(text), encoding="utf8")


def run(cmd, cwd=None):
    p = subprocess.run([str(c) for c in cmd], cwd=cwd, capture_output=True, text=True, timeout=500)
    return p.returncode, p.stdout + p.stderr


# ------------------------------------------------------------- baseline still true
stale = []
for row in (HERE / "checks/baseline.sha256").read_text(encoding="utf8").splitlines():
    if row.strip() and not row.startswith("#"):
        digest, rel = row.split(maxsplit=1)
        if sha(REPO / rel) != digest:
            stale.append(rel)
if stale:
    print("STOP: canonical moved since the baseline:", stale)
    sys.exit(2)
print("canonical equals baseline")

# ------------------------------------------------------------- format, then overlay
report = []
for name in NAMES:
    rc, text = run(["rustfmt", "--edition", "2024", "--config-path", REPO / "rustfmt.toml", "--check", STAGED / f"{name}.rs"])
    report.append(f"rustfmt --check {name}.rs: exit {rc}")
    if rc:
        report.append(text)
overlay = SCRATCH / "run-overlay"
canon = SCRATCH / "run-canon"
for root in (overlay, canon):
    if not root.exists():
        (root / "docs").mkdir(parents=True)
        shutil.copytree(CRATE / "src", root / "src")
        for rel in ("script.tsv", "Cargo.toml", "docs/storyboard.md", MAP_REL):
            shutil.copyfile(CRATE / rel, root / rel)
for name in NAMES:
    shutil.copyfile(STAGED / f"{name}.rs", overlay / "src/chapters" / f"{name}.rs")
rc, text = run(["rustfmt", "--edition", "2024", "--config-path", REPO / "rustfmt.toml", "--check", overlay / "src/main.rs"])
report.append(f"rustfmt --check main.rs of canonical + repair (parses every module): exit {rc}")
if rc:
    report.append(text)
save("rustfmt.txt", "\n".join(report) + "\n")
print(report[-1] if not rc else text[:800])

# ------------------------------------------------------------- patch, there and back
parts = []
for name in NAMES:
    rel = f"{CRATE_REL}/src/chapters/{name}.rs"
    a = (REPO / rel).read_text(encoding="utf8").splitlines(keepends=True)
    b = (STAGED / f"{name}.rs").read_text(encoding="utf8").splitlines(keepends=True)
    parts.extend(difflib.unified_diff(a, b, f"a/{rel}", f"b/{rel}", n=3))
patch = HERE / "complex02-05-repair03.patch"
patch.write_text("".join(parts), encoding="utf8")
trial = SCRATCH / "patch-trial"
if trial.exists():
    print("STOP: patch trial folder exists; give a fresh SCRATCH")
    sys.exit(2)
for name in NAMES:
    target = trial / CRATE_REL / "src/chapters" / f"{name}.rs"
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(CRATE / "src/chapters" / f"{name}.rs", target)
lines = [f"patch sha256 {sha(patch)}"]
rc, text = run(["git", "apply", "--check", patch], cwd=trial)
lines.append(f"git apply --check: exit {rc} {text.strip()}")
rc, text = run(["git", "apply", patch], cwd=trial)
lines.append(f"git apply: exit {rc} {text.strip()}")
same = all(sha(trial / CRATE_REL / "src/chapters" / f"{n}.rs") == sha(STAGED / f"{n}.rs") for n in NAMES)
lines.append(f"applied equals staged: {same}")
rc, text = run(["git", "apply", "-R", patch], cwd=trial)
lines.append(f"git apply -R: exit {rc} {text.strip()}")
back = all(sha(trial / CRATE_REL / "src/chapters" / f"{n}.rs") == sha(CRATE / "src/chapters" / f"{n}.rs") for n in NAMES)
lines.append(f"reversed equals canonical: {back}")
added = sum(1 for p in parts if p.startswith("+") and not p.startswith("+++"))
removed = sum(1 for p in parts if p.startswith("-") and not p.startswith("---"))
lines.append(f"+{added} -{removed} lines")
save("patch.txt", "\n".join(lines) + "\n")
print("\n".join(lines))

# ------------------------------------------------------------- mirrors, binder, timing
rc, text = run(PY + [HERE / "checks/mirror_tests.py", REPO, STAGED])
save("mirror_tests.txt", text)
print("mirror_tests exit", rc, text.splitlines()[1] if len(text.splitlines()) > 1 else text, text.strip().splitlines()[-1])
rows_json = SCRATCH / "binder-rows.json"
rc, text = run(PY + [HERE / "checks/binder_compare.py", REPO, BINDER, STAGED, overlay, rows_json])
save("binder_compare.txt", text)
print("binder_compare exit", rc, "|", " ".join(text.splitlines()[:2]))
rc, text = run(PY + [HERE / "checks/timing_budget.py", CRATE / "script.tsv", CUES])
save("timing_budget.txt", text)
print("timing_budget exit", rc)

# ------------------------------------------------------------- desk checks, new findings
mask = lambda s: re.sub(r":\d+", ":N", s)  # noqa: E731
kit = REPO / "crates/explainer_kit/src"
desk = []
for tool, args in (
    ("arity_check", (kit, FMN)),
    ("scope_check", (kit, FMN)),
    ("shape_check", (kit, FMN)),
    ("unused_check", ()),
    ("desk_check", ()),
):
    # the tools find their helpers beside themselves, and read the crate from the cwd
    rc_a, a = run(PY + [TOOLS / f"{tool}.py", *args], cwd=canon)
    rc_b, b = run(PY + [TOOLS / f"{tool}.py", *args], cwd=overlay)
    seen = Counter(mask(x) for x in a.splitlines())
    new = []
    for row in b.splitlines():
        if seen[mask(row)] > 0:
            seen[mask(row)] -= 1
        else:
            new.append(row)
    desk.append(f"=== {tool}: canonical exit {rc_a}; with repair exit {rc_b}; rows not on canonical: {len(new)}")
    desk.extend("   " + row for row in new)
save("desk_checks.txt", "\n".join(desk) + "\n")
print("\n".join(d for d in desk if d.startswith("===")))

# ------------------------------------------------------------- the five hold rows
the_map = json.loads((CRATE / MAP_REL).read_text(encoding="utf8"))
derived = json.loads(rows_json.read_text(encoding="utf8"))
delta, problems = [], []
for row in the_map["visual_holds"]:
    if not row["cue"][:3] in ("02_", "03_", "04_", "05_"):
        continue
    new = derived.get(row["cue"], {}).get("after")
    if not new:
        problems.append(f"{row['cue']}: the binder derives no change")
        continue
    name = Path(row["path"]).name
    text = (STAGED / name).read_text(encoding="utf8").splitlines()
    if text[new["line"] - 1].strip() != new["marker"]:
        problems.append(f"{row['cue']}: line {new['line']} is not the marker")
    if new["sha256"] != sha(STAGED / name):
        problems.append(f"{row['cue']}: hash is not the staged file's")
    delta.append(
        {
            "cue": row["cue"],
            "next_cue": row["next_cue"],
            "kind": row["kind"],
            "path": row["path"],
            "old": {k: row[k] for k in ("sha256", "line", "marker")},
            "new": {k: new[k] for k in ("sha256", "line", "marker")},
            "basis_changed": row.get("basis") != new.get("basis"),
        }
    )
BIND.mkdir(exist_ok=True)
(BIND / "visual-holds-delta.json").write_text(
    json.dumps(
        {
            "map": f"{CRATE_REL}/{MAP_REL}",
            "map_sha256": sha(CRATE / MAP_REL),
            "script_sha256": sha(CRATE / "script.tsv"),
            "note": "Replace only these fields. Rows and sections not listed do not change. Derived by the frozen binder's build() on canonical + this patch, and each row re-read here. Valid only for these exact file hashes; a later edit to any of the four files moves them again.",
            "rows": delta,
        },
        indent=1,
        ensure_ascii=False,
    )
    + "\n",
    encoding="utf8",
)
print("hold rows:", [(r["cue"], r["old"]["line"], r["new"]["line"]) for r in delta], "problems:", problems)


# ------------------------------------------------------------- every literal hook
def hooks(source, script):
    code = source[: source.index("#[cfg(test)]\nmod tests")]
    says = [m.start() for m in re.finditer(re.escape('let line = say(&kit, stage, "'), code)]
    out = []
    for k, at in enumerate(says):
        end = says[k + 1] if k + 1 < len(says) else len(code)
        body = code[at:end]
        cue = body.split('"')[1]
        raw = script[cue].encode("utf8")
        for m in re.finditer(r'line\.(wait_for|until)\(stage, "([^"]*)"', body):
            out.append((cue, m.group(1), m.group(2), raw.find(m.group(2).encode("utf8"))))
        for m in re.finditer(r"line\.wait_sentence\(stage, (\d+)\)", body):
            out.append((cue, "wait_sentence", m.group(1), ""))
    return out


script = {}
for row in (CRATE / "script.tsv").read_text(encoding="utf8").splitlines():
    cells = row.split("\t")
    if len(cells) >= 2 and re.fullmatch(r"\d\d_\d+", cells[0]):
        script[cells[0]] = cells[1]
table = ["cue\tfile\tcall\thook\tbyte_in_cue\tstatus\tphrase_mark_needed"]
counts = Counter()
for name in NAMES:
    old = Counter(hooks((CRATE / "src/chapters" / f"{name}.rs").read_text(encoding="utf8"), script))
    new = Counter(hooks((STAGED / f"{name}.rs").read_text(encoding="utf8"), script))
    for row in sorted(set(old) | set(new), key=lambda r: (r[0][:2], int(r[0][3:]), str(r[3]) if r[3] == "" else f"{r[3]:06}", r[2])):
        cue, call, hook, byte = row
        if new[row] > old[row]:
            status = "added"
        elif new[row] < old[row]:
            status = "removed"
        else:
            status = "kept"
        needed = "yes" if status == "added" and call != "wait_sentence" else "no"
        counts[status] += 1
        table.append(f"{cue}\t{name}.rs\t{call}\t{hook}\t{byte}\t{status}\t{needed}")
(BIND / "phrase-hooks.tsv").write_text("\n".join(table) + "\n", encoding="utf8")
print("hooks:", dict(counts))
sys.exit(1 if problems or not same or not back else 0)
