"""Regenerate this folder's patch, binding deltas and check outputs. No Cargo, render or TTS.

Writes only under this folder and under SCRATCH. Canonical is read, never changed; the run stops first
if canonical no longer matches checks/baseline.sha256. The desk checks and binder are the frozen copies
in ../../complex, read in place. SCRATCH must not exist yet.

Usage: python3 -B run_checks.py REPO FMN_CRATES SCRATCH     (the two picture checks need PIL, so not -I)
"""

import difflib
import hashlib
import importlib.util
import json
import re
import shutil
import subprocess
import sys
from collections import Counter
from pathlib import Path

REPO, FMN, SCRATCH = (Path(a).resolve() for a in sys.argv[1:4])
HERE = Path(__file__).resolve().parent.parent
OUT, BIND = HERE / "checks/outputs", HERE / "bindings"
CRATE_REL = "explainer_corpus/complex_analysis_visual"
CRATE = REPO / CRATE_REL
NAME = "closing.rs"
FILE_REL = f"{CRATE_REL}/src/chapters/{NAME}"
STAGED = HERE / FILE_REL
OPUS = HERE.parent.parent
TOOLS = OPUS / "complex/checks"
BINDER = OPUS / "complex/reading-repair02/bind_oliver_plan.py"
PREVIEW = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-preview02/render/15_closing.mp4"
MAP_REL = "docs/narration-direction-map-oliver10.json"
PATCH_NAME = "complex15-closing-sweep.patch"
sys.dont_write_bytecode = True


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


stale = [r.split(maxsplit=1)[1] for r in (HERE / "checks/baseline.sha256").read_text(encoding="utf8").splitlines() if sha(REPO / r.split(maxsplit=1)[1]) != r.split()[0]]
if stale:
    print("STOP: canonical moved since the baseline:", stale)
    sys.exit(2)
print("canonical equals baseline")
if SCRATCH.exists():
    print("STOP: give a fresh SCRATCH")
    sys.exit(2)

# ------------------------------------------------------------- format, on the file and on the crate
overlay, canon = SCRATCH / "overlay", SCRATCH / "canon"
for root in (overlay, canon):
    (root / "docs").mkdir(parents=True)
    shutil.copytree(CRATE / "src", root / "src")
    for rel in ("script.tsv", "Cargo.toml", "docs/storyboard.md", MAP_REL):
        shutil.copyfile(CRATE / rel, root / rel)
shutil.copyfile(STAGED, overlay / f"src/chapters/{NAME}")
fmt = REPO / "rustfmt.toml"
report = []
for name, path in ((NAME, STAGED), ("main.rs of canonical + repair (parses every module)", overlay / "src/main.rs")):
    rc, text = run(["rustfmt", "--edition", "2024", "--config-path", fmt, "--check", path])
    report.append(f"rustfmt --check {name}: exit {rc}" + (f"\n{text}" if rc else ""))
save("rustfmt.txt", "\n".join(report) + "\n")
print("\n".join(report))

# ------------------------------------------------------------- patch, there and back
a = (REPO / FILE_REL).read_text(encoding="utf8").splitlines(keepends=True)
b = STAGED.read_text(encoding="utf8").splitlines(keepends=True)
parts = list(difflib.unified_diff(a, b, f"a/{FILE_REL}", f"b/{FILE_REL}", n=3))
patch = HERE / PATCH_NAME
patch.write_text("".join(parts), encoding="utf8")
trial = SCRATCH / "patch-trial"
target = trial / FILE_REL
target.parent.mkdir(parents=True)
shutil.copyfile(REPO / FILE_REL, target)
lines = [f"patch sha256 {sha(patch)}"]
for label, cmd in (("--check", ["git", "apply", "--check", patch]), ("apply", ["git", "apply", patch])):
    rc, text = run(cmd, cwd=trial)
    lines.append(f"git apply {label}: exit {rc} {text.strip()}")
same = sha(target) == sha(STAGED)
lines.append(f"applied equals staged: {same}")
rc, text = run(["git", "apply", "-R", patch], cwd=trial)
back = sha(target) == sha(REPO / FILE_REL)
lines.append(f"git apply -R: exit {rc} {text.strip()}; reversed equals canonical: {back}")
lines.append(f"+{sum(1 for p in parts if p[0] == '+' and not p.startswith('+++'))} -{sum(1 for p in parts if p[0] == '-' and not p.startswith('---'))} lines")
save("patch.txt", "\n".join(lines) + "\n")
print("\n".join(lines))

# ------------------------------------------------------------- the sweep and draw order, the frames, the look
rc_sweep, text = run([sys.executable, "-I", "-B", HERE / "checks/sweep_check.py", REPO / FILE_REL, STAGED])
save("sweep_check.txt", text)
print("sweep_check exit", rc_sweep, "|", text.strip().splitlines()[-1])
rc, text = run([sys.executable, "-B", HERE / "checks/preview_times.py", PREVIEW, SCRATCH / "preview-times"])
save("preview_times.txt", text)
print("preview_times exit", rc, "|", len(text.splitlines()), "rows")
rc, text = run([sys.executable, "-B", HERE / "checks/synthetic_sweep.py", PREVIEW, SCRATCH / "preview-times", OUT / "synthetic-sweep-NOT-A-RENDER.png"])
print("synthetic_sweep exit", rc, text.strip()[:120])


# ------------------------------------------------------------- strings on screen, new and gone
def strings(source):
    code = source[: source.index("#[cfg(test)]\nmod tests")]
    code = "\n".join(ln for ln in code.splitlines() if not ln.lstrip().startswith("//"))
    return Counter(m.group(1) for m in re.finditer(r'"((?:[^"\\]|\\.)*)"', code))


old_strings = strings((REPO / FILE_REL).read_text(encoding="utf8"))
new_strings = strings(STAGED.read_text(encoding="utf8"))
string_rows = [f"NEW {s!r}" for s in sorted(new_strings - old_strings)] + [f"GONE {s!r}" for s in sorted(old_strings - new_strings)]
save("strings.txt", "\n".join(string_rows) + "\n")
print("strings:", string_rows)

# ------------------------------------------------------------- the frozen binder, read-only
spec = importlib.util.spec_from_file_location("bind_oliver_plan", BINDER)
binder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(binder)


def build(root):
    plan, _outputs, _audit, _canonical, map_text = binder.build(str(REPO), str(root), "complex15-compare")
    return plan.problems, json.loads(map_text)


mask = lambda s: re.sub(r"\.rs:\d+", ".rs:N", s)  # noqa: E731
before, map_before = build(CRATE)
after, map_after = build(overlay)
added = sorted(set(map(mask, after)) - set(map(mask, before)))
gone = sorted(set(map(mask, before)) - set(map(mask, after)))
out = [f"canonical problems: {len(before)}; with this repair: {len(after)}", f"added by the repair: {len(added)}; gone with the repair: {len(gone)}"]
out += [f"  ADDED {t}" for t in added] + [f"  GONE  {t}" for t in gone]
out += ["problems on canonical (not this repair's):"] + [f"   {t}" for t in before]
rows_a, rows_b = ({r["cue"]: r for r in m["visual_holds"]} for m in (map_before, map_after))
changed = sorted(c for c in set(rows_a) | set(rows_b) if rows_a.get(c) != rows_b.get(c))
out.append(f"hold rows the binder derives differently: {changed}")
out.append(f"other sections unchanged: { {k: map_before.get(k) == map_after.get(k) for k in map_after if k != 'visual_holds'} }")
save("binder_compare.txt", "\n".join(out) + "\n")
print("\n".join(out[:2]), "| rows", changed)

# ------------------------------------------------------------- the desk checks, new findings only
kit = REPO / "crates/explainer_kit/src"
line_mask = lambda s: re.sub(r":\d+", ":N", s)  # noqa: E731
desk = []
for tool, args in (("arity_check", (kit, FMN)), ("scope_check", (kit, FMN)), ("shape_check", (kit, FMN)), ("unused_check", ()), ("desk_check", ())):
    rc_a, x = run([sys.executable, "-I", "-B", TOOLS / f"{tool}.py", *args], cwd=canon)
    rc_b, y = run([sys.executable, "-I", "-B", TOOLS / f"{tool}.py", *args], cwd=overlay)
    seen = Counter(line_mask(r) for r in x.splitlines())
    new = []
    for row in y.splitlines():
        if seen[line_mask(row)] > 0:
            seen[line_mask(row)] -= 1
        else:
            new.append(row)
    desk.append(f"=== {tool}: canonical exit {rc_a}; with repair exit {rc_b}; rows not on canonical: {len(new)}")
    desk.extend("   " + r for r in new)
save("desk_checks.txt", "\n".join(desk) + "\n")
print("\n".join(d for d in desk if d.startswith("===")))
print("\n".join(d for d in desk if not d.startswith("===")))

# ------------------------------------------------------------- hold rows, checked against the file
the_map = json.loads((CRATE / MAP_REL).read_text(encoding="utf8"))
staged_lines = STAGED.read_text(encoding="utf8").splitlines()
delta, problems = [], []
for row in the_map["visual_holds"]:
    if Path(row["path"]).name != NAME:
        continue
    new = rows_b.get(row["cue"])
    if rows_a.get(row["cue"]) == new:
        problems.append(f"{row['cue']}: the binder derives no change")
        continue
    if staged_lines[new["line"] - 1].strip() != new["marker"]:
        problems.append(f"{row['cue']}: line {new['line']} is not the marker")
    if new["sha256"] != sha(STAGED):
        problems.append(f"{row['cue']}: hash is not the staged file's")
    delta.append(
        {
            "cue": row["cue"],
            "next_cue": row["next_cue"],
            "kind": row["kind"],
            "path": row["path"],
            "old": {k: row[k] for k in ("sha256", "line", "marker")},
            "new": {k: new[k] for k in ("sha256", "line", "marker")},
        }
    )
BIND.mkdir(exist_ok=True)
(BIND / "visual-holds-delta.json").write_text(
    json.dumps(
        {
            "map": f"{CRATE_REL}/{MAP_REL}",
            "map_sha256": sha(CRATE / MAP_REL),
            "script_sha256": sha(CRATE / "script.tsv"),
            "note": f"Replace only these fields. Rows and sections not listed do not change. Derived by the frozen binder's build() on canonical + this patch, and each row re-read here. Valid only for this exact file hash; a later edit to {NAME} moves them again.",
            "rows": delta,
        },
        indent=1,
        ensure_ascii=False,
    )
    + "\n",
    encoding="utf8",
)
print("hold rows:", [(r["cue"], r["old"]["line"], r["new"]["line"], r["old"]["marker"] == r["new"]["marker"]) for r in delta], "problems:", problems)


# ------------------------------------------------------------- every literal hook
def hooks(source):
    code = source[: source.index("#[cfg(test)]\nmod tests")]
    says = [m.start() for m in re.finditer(re.escape('let line = say(&kit, stage, "'), code)]
    found = Counter()
    for k, at in enumerate(says):
        body = code[at : says[k + 1] if k + 1 < len(says) else len(code)]
        cue = body.split('"')[1]
        for m in re.finditer(r'line\.(wait_for|until)\(stage, "([^"]*)"', body):
            found[(cue, m.group(1), m.group(2))] += 1
        for m in re.finditer(r"line\.wait_sentence\(stage, (\d+)\)", body):
            found[(cue, "wait_sentence", m.group(1))] += 1
    return found


old, new = hooks((REPO / FILE_REL).read_text(encoding="utf8")), hooks(STAGED.read_text(encoding="utf8"))
table = ["cue\tcall\thook\tstatus"]
for row in sorted(set(old) | set(new), key=lambda r: (int(r[0][3:]), r[1], r[2])):
    table.append("\t".join(row) + "\t" + ("kept" if old[row] == new[row] else "CHANGED"))
(BIND / "phrase-hooks.tsv").write_text("\n".join(table) + "\n", encoding="utf8")
print("hooks:", Counter(r.rsplit("\t", 1)[1] for r in table[1:]))

# ------------------------------------------------------------- the neighbours this must stay clear of
overlap = []
neighbours = sorted((OPUS / "complex-color-repair03/patches").glob("*.patch"))
neighbours += [OPUS / "complex02-05-repair03/complex02-05-repair03.patch", HERE.parent / "complex13-radius-repair01.patch", HERE.parent / "continuation14/complex14-pole-order.patch"]
neighbours += sorted((OPUS / "complex11-counting-teaching02/patches").glob("*.patch"))
neighbours += sorted((OPUS / "complex11-counting-review-repair01/patches").glob("*.patch"))
for patch_path in neighbours:
    text = patch_path.read_text(encoding="utf8")
    touched = [ln for ln in text.splitlines() if ln.startswith("+++ ") and NAME in ln]
    words = "a loop counts" in text
    overlap.append(f"{patch_path.relative_to(OPUS)}: {'TOUCHES ' + NAME if touched else 'does not touch ' + NAME}; mentions the old caption: {words}")
save("overlap.txt", "\n".join(overlap) + "\n")
print("\n".join(overlap))
sys.exit(1 if problems or not same or not back or rc_sweep else 0)
