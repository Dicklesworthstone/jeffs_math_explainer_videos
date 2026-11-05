"""Regenerate this review's corrections, bindings and check outputs. No Cargo, render or TTS.

Writes only under this folder and under SCRATCH. Canonical, teaching02 and root's staging are read,
never changed. SCRATCH must not exist yet.

Usage: python3 -I -B checks/run_checks.py REPO FMN_CRATES SCRATCH
(It starts c3_synthetic.py with `python3 -B`, because that one needs PIL.)
"""

import difflib
import hashlib
import importlib.util
import itertools
import json
import re
import shutil
import subprocess
import sys
from collections import Counter
from pathlib import Path

REPO, FMN, SCRATCH = (Path(a).resolve() for a in sys.argv[1:4])
HERE = Path(__file__).resolve().parent.parent
OUT, FIX, BIND = HERE / "checks/outputs", HERE / "corrections", HERE / "bindings"
OPUS = HERE.parent
TEACH = OPUS / "complex11-counting-teaching02"
ROOT_STAGE = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/counting-wording-repair01"
CRATE_REL = "explainer_corpus/complex_analysis_visual"
CRATE = REPO / CRATE_REL
COUNTING_REL = f"{CRATE_REL}/src/chapters/counting.rs"
MOD_REL = f"{CRATE_REL}/src/chapters/mod.rs"
STAGED = TEACH / COUNTING_REL
TOOLS = OPUS / "complex/checks"
BINDER = OPUS / "complex/reading-repair02/bind_oliver_plan.py"
PLATE_A = OPUS / "complex-color-repair03/patches/A-plate-core.patch"
GLYPH = OPUS / "complex-glyph-spacing-repair01/complex-glyph-spacing.patch"
FRAMES = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/11_counting"
sys.dont_write_bytecode = True
failures = []


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def digest(text):
    return hashlib.sha256(text.encode("utf8")).hexdigest()


def clean(text):
    for path, name in ((REPO, "<repo>"), (FMN, "<fmn>"), (SCRATCH, "<scratch>"), (Path.home(), "<home>")):
        text = text.replace(str(path), name)
    return text


def save(name, text):
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / name).write_text(clean(text), encoding="utf8")


def run(cmd, cwd=None):
    p = subprocess.run([str(c) for c in cmd], cwd=cwd, capture_output=True, text=True, timeout=500)
    return p.returncode, p.stdout + p.stderr


def expect(name, ok, report):
    report.append(f"{'PASS' if ok else 'FAIL'} {name}")
    if not ok:
        failures.append(name)


if SCRATCH.exists():
    print("STOP: give a fresh SCRATCH")
    sys.exit(2)
SCRATCH.mkdir(parents=True)

# ------------------------------------------------------------- what is under review, still as sealed
report = []
rows = (TEACH / "SHA256SUMS").read_text(encoding="utf8").splitlines()
bad = [r.split(maxsplit=1)[1] for r in rows if sha(TEACH / r.split(maxsplit=1)[1].lstrip("*").strip()) != r.split()[0]]
expect(f"teaching02's seal holds: {len(rows)} files", not bad, report)
moved = [r.split(maxsplit=1)[1].strip() for r in (TEACH / "checks/baseline.sha256").read_text(encoding="utf8").splitlines() if r.strip() and sha(REPO / r.split(maxsplit=1)[1].strip()) != r.split()[0]]
report.append(f"baseline paths moved since teaching02 pinned them: {moved}")
expect("canonical and root's staging are as teaching02 pinned them", not moved, report)
report.append(f"teaching02 HANDOFF.md {sha(TEACH / 'HANDOFF.md')}")
report.append(f"staged counting.rs {sha(STAGED)}")

# ------------------------------------------------------------- patch round trips, from the sources
trial = SCRATCH / "round-trip"
for rel, source in (
    (COUNTING_REL, ROOT_STAGE / "counting.rs"),
    (MOD_REL, CRATE / "src/chapters/mod.rs"),
    (f"{CRATE_REL}/docs/storyboard.md", CRATE / "docs/storyboard.md"),
    (f"{CRATE_REL}/narration_units.json", CRATE / "narration_units.json"),
):
    (trial / rel).parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, trial / rel)
for name, rel, want in (
    ("counting-from-root.patch", COUNTING_REL, "dba35730d15b3c5e4ff7fb0fe7444265424b920d87861562b11d8468f9d290e4"),
    ("mod-registry.patch", MOD_REL, "2414a6bd8c1ae26af60385d6e5ee80bfc70a1d7dae1ae6aeccdcc97c75b60549"),
    ("storyboard.patch", f"{CRATE_REL}/docs/storyboard.md", "6deb9f01cbcc505a2a08e5d8bc673bfe3f035c499824091058e86e903acee1c4"),
    ("narration-units.patch", f"{CRATE_REL}/narration_units.json", "3989e1855f617734c38caae8d0113d4915b5c44fbde600f674e5779ed4977f28"),
):
    start = sha(trial / rel)
    rc, text = run(["patch", "-p1", "-s", "-i", TEACH / "patches" / name], cwd=trial)
    expect(f"{name} applies and gives {want[:16]}", rc == 0 and sha(trial / rel) == want, report)
    rc, text = run(["patch", "-p1", "-s", "-R", "-i", TEACH / "patches" / name], cwd=trial)
    expect(f"{name} reverses to its source", rc == 0 and sha(trial / rel) == start, report)
    run(["patch", "-p1", "-s", "-i", TEACH / "patches" / name], cwd=trial)
expect("the staged counting.rs is the patch's result", sha(trial / COUNTING_REL) == sha(STAGED), report)

# ------------------------------------------------------------- the script's words
def script_of(path):
    return dict(row.rstrip("\n").split("\t", 1) for row in Path(path).read_text(encoding="utf8").splitlines() if "\t" in row)


old_script, new_script = script_of(CRATE / "script.tsv"), script_of(ROOT_STAGE / "script.tsv")
changed = [c for c in new_script if old_script.get(c) != new_script[c]]
expect(f"only 11_7's words differ between canonical and root's script: {changed}", changed == ["11_7"] and set(old_script) == set(new_script), report)
units = json.loads((trial / f"{CRATE_REL}/narration_units.json").read_text(encoding="utf8"))
unit = next(u for u in units["units"] if u["id"] == "oliver_11_7")
expect("the units plan carries root's 11_7 word for word", unit["text"] == new_script["11_7"] and unit["line_ids"] == ["11_7"], report)
others = {u["id"]: u for u in units["units"]}
expect("11_6 and 11_8 stay their own units", others["oliver_11_6"]["line_ids"] == ["11_6"] and others["oliver_11_8"]["line_ids"] == ["11_8"], report)
requests = json.loads((ROOT_STAGE.parent / "narration-request-preparation01/requests01.json").read_text(encoding="utf8"))
context = [k for r in requests["requests"] for k in r if "context" in k.lower() or "previous" in k.lower() or k.startswith("next")]
expect(f"root's prepared requests carry no neighbour context ({requests['provider']}, {requests['voice']})", not context, report)
save("baseline.txt", "\n".join(report) + "\n")
print("\n".join(report))

# ------------------------------------------------------------- the corrections, by exact replacement
S = STAGED.read_text(encoding="utf8")

C1 = [("let (cols, rows) = picture_cells(&inp);", "let (cols, rows) = picture_cells(&inp, &super::draw::PLATE);")]

C2 = [
    (
        "        fade_in_all(stage, &result, 0.3)?;\n        line.wait_sentence(stage, 2)?;\n",
        '        fade_in_all(stage, &result, 0.3)?;\n        line.wait_for(stage, "with z")?;\n',
    ),
    ('            (2, "Now it starts with z to the n", "let degree_n ="),\n', ""),
    (
        "        // the walk lands on the count\n",
        "        // sentence 2 opens on the result row, which stays until the voice reaches what replaces it\n"
        '        assert!(sentences[2].starts_with("Now it starts with z to the n"));\n'
        "        // the walk lands on the count\n",
    ),
    (
        '        for (k, phrase) in [(0, "complex polynomial"), (1, "which leaves")] {\n',
        '        for (k, phrase) in [(0, "complex polynomial"), (1, "which leaves"), (2, "with z")] {\n',
    ),
    (
        '            "line.wait_sentence(stage, 2)?;",\n            "fade_out_all(stage, &[after], 0.3)?;",\n',
        '            "line.wait_for(stage, \\"with z\\")?;",\n            "fade_out_all(stage, &[after], 0.3)?;",\n',
    ),
]

OLD_ZERO_TEST_HEAD = (
    "    /// 11_8's fixed 0: the distance line reaches it at some bearing and the reach's rim passes it,\n"
    "    /// so its name goes over both on a plate that covers neither axis nor the unit circle, is on\n"
    "    /// empty ground when it is promoted, and carries the digit's whole extent.\n"
    "    #[test]\n"
    "    fn the_zero_keeps_its_name_while_the_line_and_the_rim_go_by() {\n"
)
start_at = S.index(OLD_ZERO_TEST_HEAD)
end_at = S.index("        // the film's code only: these lines are quoted here\n", start_at)
C3 = [
    (
        "/// The plate stays between the axes and inside the unit circle.\n"
        "const ZERO_NAME_AT: (f64, f64) = (-0.25, -0.3);\n",
        "/// On zero itself, the one place the reach can never hold: anywhere else in the disk's sweep\n"
        "/// the digit would be seen inside the disk the picture says misses zero.\n"
        "const ZERO_NAME_AT: (f64, f64) = (0.0, 0.0);\n",
    ),
    (
        "    fade_in_all(stage, &[gap], 0.2)?;\n"
        "    left.push(gap);\n"
        "    // the distance line and the disk's rim both pass the fixed 0: its name goes over them\n"
        "    stage.add_to_scene(zero_plate)?;\n"
        "    stage.add_to_scene(zero_name)?;\n",
        "    // the distance line starts at the fixed 0: it goes under the plate and the name before its\n"
        "    // first frame, or its end would show over them and then be cut. The rim never gets there.\n"
        "    stage.add_to_scene(gap)?;\n"
        "    stage.add_to_scene(zero_plate)?;\n"
        "    stage.add_to_scene(zero_name)?;\n"
        "    fade_in_all(stage, &[gap], 0.2)?;\n"
        "    left.push(gap);\n",
    ),
    (
        '        assert!(find("fade_in_all(stage, &[gap], 0.2)?;") < find("stage.add_to_scene(zero_plate)?;"));\n',
        '        assert!(find("stage.add_to_scene(gap)?;") < find("stage.add_to_scene(zero_plate)?;"));\n',
    ),
    (
        '        assert!(find("stage.add_to_scene(zero_name)?;") < find("let carried = stage.add_value_tracker(0.0);"));\n',
        '        assert!(find("stage.add_to_scene(zero_name)?;") < find("fade_in_all(stage, &[gap], 0.2)?;"));\n'
        '        assert!(find("fade_in_all(stage, &[gap], 0.2)?;") < find("let carried = stage.add_value_tracker(0.0);"));\n',
    ),
    ("        let zero_name = C::new(-0.25, -0.3);\n", "        let zero_name = C::new(ZERO_NAME_AT.0, ZERO_NAME_AT.1);\n"),
    (
        S[start_at:end_at],
        "    /// 11_8's fixed 0 sits on zero. The picture's point is that the reach never holds zero, so at\n"
        "    /// every bearing the disk stays off the whole plate, rim and all. The plate carries the digit's\n"
        "    /// whole extent, and goes over the distance line's end before the sweep.\n"
        "    #[test]\n"
        "    fn the_zero_keeps_its_name_outside_the_reach_at_every_bearing() {\n"
        "        let (_, out) = coarse_planes();\n"
        "        let unit = out.unit();\n"
        "        let at = C::new(ZERO_NAME_AT.0, ZERO_NAME_AT.1);\n"
        "        assert!(at.abs() == 0.0);\n"
        "        let plate = ZERO_PLATE / unit;\n"
        "        // half a 20 pt digit: 0.05 × 0.07 in scene units\n"
        "        assert!(C::new(0.05, 0.07).abs() < ZERO_PLATE);\n"
        "        let reach = f64::from(model::COARSE_POWER) * TAU / model::COARSE_STEPS[1] as f64;\n"
        "        // half the rim's stroke, in window units: 3.5 stroke units of 0.01 in the scene\n"
        "        let half_rim = 0.0175 / unit;\n"
        "        for k in 0..720 {\n"
        "            let c = C::polar(1.0, TAU * f64::from(k) / 720.0);\n"
        '            assert!((at - c).abs() - reach > plate + half_rim, "{k}");\n'
        "        }\n",
    ),
]
CORRECTIONS = {"C1-after-plate-core": C1, "C2-row-swap-on-its-words": C2, "C3-digit-on-zero": C3}


def corrected(names, base=S):
    text = base
    for name in names:
        for old, new in CORRECTIONS[name]:
            if text.count(old) != 1:
                raise SystemExit(f"{name}: expected once, found {text.count(old)}: {old[:80]!r}")
            text = text.replace(old, new)
    return text


FIX.mkdir(exist_ok=True)
report = []
for name in CORRECTIONS:
    result = corrected([name])
    parts = list(difflib.unified_diff(S.splitlines(keepends=True), result.splitlines(keepends=True), f"a/{COUNTING_REL}", f"b/{COUNTING_REL}", n=3))
    (FIX / f"{name}.patch").write_text("".join(parts), encoding="utf8")
    plus = sum(1 for p in parts if p[0] == "+" and not p.startswith("+++"))
    minus = sum(1 for p in parts if p[0] == "-" and not p.startswith("---"))
    report.append(f"{name}.patch sha256 {sha(FIX / f'{name}.patch')} (+{plus} -{minus})")

# every order of every subset gives the same bytes; apply and reverse with git's own tool
states = {}
for size in range(4):
    for subset in itertools.combinations(CORRECTIONS, size):
        finals = set()
        for order in itertools.permutations(subset):
            work = SCRATCH / "orders" / digest("/".join(order))[:12]
            (work / COUNTING_REL).parent.mkdir(parents=True)
            shutil.copyfile(STAGED, work / COUNTING_REL)
            for name in order:
                rc, text = run(["git", "apply", FIX / f"{name}.patch"], cwd=work)
                if rc:
                    failures.append(f"{order}: {name}: {text.strip()}")
            finals.add(sha(work / COUNTING_REL))
            for name in reversed(order):
                run(["git", "apply", "-R", FIX / f"{name}.patch"], cwd=work)
            if sha(work / COUNTING_REL) != sha(STAGED):
                failures.append(f"{order}: does not reverse to staged")
        want = digest(corrected(subset))
        expect(f"{'+'.join(s[:2] for s in subset) or 'staged'}: every order gives {want[:16]}, and reverses", finals == {want}, report)
        states["+".join(s[:2] for s in subset) or "S"] = want
report.append("state hashes of counting.rs:")
report += [f"  {k} {v}" for k, v in states.items()]

# with the glyph patch on the same file
for label, names in (("S", ()), ("C3", ("C3-digit-on-zero",)), ("C2+C3", ("C2-row-swap-on-its-words", "C3-digit-on-zero")), ("C1+C2+C3", tuple(CORRECTIONS))):
    work = SCRATCH / "glyph" / label.replace("+", "-")
    shutil.copytree(CRATE, work / CRATE_REL, ignore=shutil.ignore_patterns("target", "renders", "assets", "*.mp4", "*.wav"))
    (work / COUNTING_REL).write_text(corrected(names), encoding="utf8")
    rc, text = run(["patch", "-p1", "-s", "-i", GLYPH], cwd=work)
    states[f"{label} + glyph patch"] = sha(work / COUNTING_REL)
    expect(f"the glyph patch applies on {label}: {sha(work / COUNTING_REL)[:16]}", rc == 0 and "|{-3z}|" in (work / COUNTING_REL).read_text(encoding="utf8"), report)
expect("S + glyph equals teaching02's stated 138b5d84", states["S + glyph patch"].startswith("138b5d84d0c9240d"), report)

# C2 changes one phrase of the storyboard's device column, on teaching02's result
board_rel = f"{CRATE_REL}/docs/storyboard.md"
board = (trial / board_rel).read_text(encoding="utf8")
old_words, new_words = 'On "Now it starts" the row gives way', 'On "with z to the n" the row gives way'
expect("the storyboard's sentence for the row is there once", board.count(old_words) == 1, report)
board_new = board.replace(old_words, new_words)
parts = list(difflib.unified_diff(board.splitlines(keepends=True), board_new.splitlines(keepends=True), f"a/{board_rel}", f"b/{board_rel}", n=3))
(FIX / "C2-storyboard.patch").write_text("".join(parts), encoding="utf8")
report.append(f"C2-storyboard.patch sha256 {sha(FIX / 'C2-storyboard.patch')}: storyboard.md {digest(board_new)}")
save("corrections.txt", "\n".join(report) + "\n")
print("\n".join(report))


# what root's scene put on screen against what the staged scene does, by string
def strings(source):
    code = source[: source.index("#[cfg(test)]\nmod tests")]
    code = "\n".join(ln for ln in code.splitlines() if not ln.lstrip().startswith("//"))
    return Counter(m.group(1) for m in re.finditer(r'"((?:[^"\\]|\\.)*)"', code))


root_text = (ROOT_STAGE / "counting.rs").read_text(encoding="utf8")
root_strings, staged_strings = strings(root_text), strings(S)
rows = [f"NEW {s!r}" for s in sorted(staged_strings - root_strings)] + [f"GONE {s!r}" for s in sorted(root_strings - staged_strings)]
save("strings.txt", "\n".join(rows) + "\n")
root_tests = re.findall(r"fn (\w+)\(\) \{", root_text[root_text.index("mod tests") :])
staged_tests = re.findall(r"fn (\w+)\(\) \{", S[S.index("mod tests") :])
report = [f"root's tests: {len(root_tests)}; staged tests: {len(staged_tests)}; root's tests gone: {sorted(set(root_tests) - set(staged_tests))}"]
removed = [ln[1:] for ln in (TEACH / "patches/counting-from-root.patch").read_text(encoding="utf8").splitlines() if ln.startswith("-") and not ln.startswith("---")]
report.append(f"lines the patch takes out of root's file: {len(removed)}")
report += ["  - " + r.strip() for r in removed]
expect("no test of root's is removed", not (set(root_tests) - set(staged_tests)), report)
save("preserved.txt", "\n".join(report) + "\n")
print(report[0])

# ------------------------------------------------------------- overlays for format, desk checks, binder
FINAL = tuple(CORRECTIONS)
NO_PLATE = ("C2-row-swap-on-its-words", "C3-digit-on-zero")


def overlay(name, counting_text, plate=False):
    root = SCRATCH / "overlays" / name
    (root / "docs").mkdir(parents=True)
    shutil.copytree(CRATE / "src", root / "src")
    shutil.copyfile(CRATE / "Cargo.toml", root / "Cargo.toml")
    shutil.copyfile(ROOT_STAGE / "script.tsv", root / "script.tsv")
    shutil.copyfile(trial / f"{CRATE_REL}/docs/storyboard.md", root / "docs/storyboard.md")
    for rel in ("docs/narration-direction-map-oliver10.json",):
        shutil.copyfile(CRATE / rel, root / rel)
    shutil.copyfile(TEACH / "holds/narration-direction-map-oliver11.json", root / "docs/narration-direction-map-oliver11.json")
    shutil.copyfile(trial / MOD_REL, root / "src/chapters/mod.rs")
    (root / "src/chapters/counting.rs").write_text(counting_text, encoding="utf8")
    if plate:
        rc, text = run(["patch", "-p3", "-s", "-i", PLATE_A], cwd=root)
        if rc:
            failures.append(f"plate core on {name}: {text.strip()}")
    return root


canon = SCRATCH / "overlays/canonical"
(canon / "docs").mkdir(parents=True)
shutil.copytree(CRATE / "src", canon / "src")
shutil.copyfile(CRATE / "Cargo.toml", canon / "Cargo.toml")
shutil.copyfile(CRATE / "script.tsv", canon / "script.tsv")
shutil.copyfile(CRATE / "docs/storyboard.md", canon / "docs/storyboard.md")
shutil.copyfile(CRATE / "docs/narration-direction-map-oliver10.json", canon / "docs/narration-direction-map-oliver10.json")
roots = {
    "staged": overlay("staged", S),
    "staged+plate": overlay("staged-plate", S, plate=True),
    "C2+C3": overlay("c2c3", corrected(NO_PLATE)),
    "C1+C2+C3+plate": overlay("final-plate", corrected(FINAL), plate=True),
}

report = []
fmt = REPO / "rustfmt.toml"
for label, root in roots.items():
    rc, text = run(["rustfmt", "--edition", "2024", "--config-path", fmt, "--check", root / "src/chapters/counting.rs"])
    expect(f"rustfmt --check counting.rs, {label}", rc == 0, report)
    if rc:
        report.append(text[:1500])
    rc, text = run(["rustfmt", "--edition", "2024", "--config-path", fmt, "--check", root / "src/main.rs"])
    report.append(f"rustfmt --check main.rs parses every module, {label}: exit {rc}")
    if rc not in (0, 1):
        failures.append(f"main.rs does not parse on {label}")
save("rustfmt.txt", "\n".join(report) + "\n")
print("\n".join(report))

# the desk checks: rows not on canonical
kit = REPO / "crates/explainer_kit/src"
line_mask = lambda s: re.sub(r":\d+", ":N", s)  # noqa: E731
desk = []
base_rows = {}
for tool, args in (("arity_check", (kit, FMN)), ("scope_check", (kit, FMN)), ("shape_check", (kit, FMN)), ("unused_check", ()), ("desk_check", ()), ("supplements", ())):
    rc, base = run([sys.executable, "-I", "-B", TOOLS / f"{tool}.py", *args], cwd=canon)
    base_rows[tool] = base
    desk.append(f"=== {tool}: canonical exit {rc}")
    for label, root in roots.items():
        rc, text = run([sys.executable, "-I", "-B", TOOLS / f"{tool}.py", *args], cwd=root)
        seen = Counter(line_mask(r) for r in base.splitlines())
        new = []
        for row in text.splitlines():
            if seen[line_mask(row)] > 0:
                seen[line_mask(row)] -= 1
            else:
                new.append(row)
        desk.append(f"--- {label}: exit {rc}; rows not on canonical: {len(new)}")
        desk.extend("   " + r for r in new)
desk_text = "\n".join(desk) + "\n"
save("desk_checks.txt", desk_text)
print("\n".join(d for d in desk if d.startswith(("===", "---"))))
sections = re.split(r"(?m)^--- ", desk_text.split("=== scope_check")[0])
arity = {s.split(":")[0]: s for s in sections[1:]}
report = []
expect("the arity tool flags picture_cells on staged + plate core", "picture_cells" in arity["staged+plate"], report)
expect("the arity tool does not flag it on staged alone", "picture_cells" not in arity["staged"], report)
expect("the arity tool does not flag it with C1 on the plate core", "picture_cells" not in arity["C1+C2+C3+plate"], report)
plate_draw = (roots["staged+plate"] / "src/chapters/draw.rs").read_text(encoding="utf8")
expect("plate core's signature takes the style", "pub fn picture_cells(plane: &Plane, style: &PlateStyle) -> (usize, usize) {" in plate_draw, report)
expect("plate core keeps what Turning uses", all(t in plate_draw for t in ("pub fn portrait_colors(", "pub fn portrait(stage: &mut Stage<'_>, f: Func, plane: &Plane, enhance: f64) -> fmn::Result<Picture>", "pub struct Picture {\n    pub group: Mob,", "pub fn recolor(&self, arena: &mut Arena, colors: &[Vec<Srgb>]) -> bool", "pub const PLATE: PlateStyle")), report)
stage_draw = ROOT_STAGE.parent / "color-native-stage01/crate/src/chapters/draw.rs"
expect("root's colour stage carries the same signature", "pub fn picture_cells(plane: &Plane, style: &PlateStyle)" in stage_draw.read_text(encoding="utf8"), report)
callers = [str(p.relative_to(roots["staged+plate"])) for p in sorted((roots["staged+plate"] / "src").rglob("*.rs")) if p.name != "draw.rs" and "picture_cells(" in p.read_text(encoding="utf8")]
report.append(f"callers outside draw.rs on the overlay: {callers}")
expect("counting.rs is the only caller outside draw.rs", callers == ["src/chapters/counting.rs"], report)
canon_callers = [p.name for p in sorted((CRATE / "src").rglob("*.rs")) if p.name != "draw.rs" and "picture_cells(" in p.read_text(encoding="utf8")]
expect("canonical has none, so the plate lane's own checks could not see this", not canon_callers, report)
save("plate_conflict.txt", "\n".join(report) + "\n")
print("\n".join(report))

# ------------------------------------------------------------- the frozen binder, read-only
spec = importlib.util.spec_from_file_location("bind_oliver_plan", BINDER)
binder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(binder)


def build(root):
    plan, _outputs, _audit, _canonical, map_text = binder.build(str(REPO), str(root), "complex11-cross")
    return plan.problems, json.loads(map_text)


mask = lambda s: re.sub(r"\.rs:\d+", ".rs:N", s)  # noqa: E731
before, map_before = build(canon)
report = [f"canonical: {len(before)} problems"]
teach_map = json.loads((TEACH / "holds/narration-direction-map-oliver11.json").read_text(encoding="utf8"))
teach_rows = {r["cue"]: r for r in teach_map["visual_holds"]}
binder_rows = {}
for label in ("staged", "C2+C3"):
    problems, derived = build(roots[label])
    added = sorted(set(map(mask, problems)) - set(map(mask, before)))
    gone = sorted(set(map(mask, before)) - set(map(mask, problems)))
    report.append(f"{label}: {len(problems)} problems; added {len(added)}, gone {len(gone)}")
    report += [f"  ADDED {t}" for t in added] + [f"  GONE  {t}" for t in gone]
    expect(f"{label} adds nothing to the frozen binder's problems", not added, report)
    binder_rows[label] = {r["cue"]: r for r in derived["visual_holds"]}
agree = []
for cue, row in teach_rows.items():
    new = binder_rows["staged"].get(cue)
    if not new or any(row[k] != new[k] for k in ("sha256", "line", "marker", "path")):
        agree.append(cue)
report.append(f"rows of teaching02's map the binder derives differently on staged: {agree}")
expect("every hold row of teaching02's map is what the frozen binder derives on staged", not agree, report)
rows_a = {r["cue"]: r for r in map_before["visual_holds"]}
moved_rows = sorted(c for c in teach_rows if rows_a.get(c) != teach_rows[c])
report.append(f"rows that differ from canonical's derivation: {moved_rows}")
for name in ("mod.rs",):
    report.append("mod.rs helper rows take staged mod.rs: " + str(sorted({r["sha256"][:12] for r in teach_rows.values() if Path(r["path"]).name == name} | {sha(trial / MOD_REL)[:12]})))
save("binder_compare.txt", "\n".join(report) + "\n")
print("\n".join(report))

# hold rows in every state, by marker, and against the binder where it ran
markers = {r["cue"]: r for r in teach_rows.values() if Path(r["path"]).name == "counting.rs"}


def rows_in(text):
    lines = text.splitlines()
    out = {}
    for cue, row in markers.items():
        hits = [i + 1 for i, ln in enumerate(lines) if ln.strip() == row["marker"]]
        # the marker can repeat: take the occurrence that keeps the staged row's place in the order
        rank = [i + 1 for i, ln in enumerate(S.splitlines()) if ln.strip() == row["marker"]].index(row["line"])
        out[cue] = {"line": hits[rank], "marker": row["marker"]}
    return out


report = []
state_rows = {}
for label, names in (("S", ()), ("C1", ("C1-after-plate-core",)), ("C3", ("C3-digit-on-zero",)), ("C1+C3", ("C1-after-plate-core", "C3-digit-on-zero")), ("C2+C3", NO_PLATE), ("C1+C2+C3", FINAL)):
    text = corrected(names)
    state_rows[label] = {"counting_sha256": digest(text), "rows": rows_in(text)}
for cue, row in rows_in(corrected(NO_PLATE)).items():
    derived = binder_rows["C2+C3"][cue]
    expect(f"{cue} on C2+C3: marker scan {row['line']} equals the binder's {derived['line']}", derived["line"] == row["line"] and derived["sha256"] == digest(corrected(NO_PLATE)), report)
BIND.mkdir(exist_ok=True)
(BIND / "holds-by-state.json").write_text(
    json.dumps(
        {
            "note": "Against teaching02's candidate map narration-direction-map-oliver11.json. In the rows listed replace only `sha256` (with counting_sha256) and `line`. Lines in draw.rs and mod.rs do not move for C2 or C3. If the plate core goes in, draw.rs moves and root rebinds the map last anyway.",
            "map_sha256": sha(TEACH / "holds/narration-direction-map-oliver11.json"),
            "states": state_rows,
        },
        indent=1,
        ensure_ascii=False,
    )
    + "\n",
    encoding="utf8",
)
save("holds.txt", "\n".join(report) + "\n" + json.dumps(state_rows, indent=1, ensure_ascii=False) + "\n")
print("\n".join(report))

# ------------------------------------------------------------- the sub-checks
model = CRATE / "src/model.rs"
final_path = roots["C1+C2+C3+plate"] / "src/chapters/counting.rs"
c2c3_path = roots["C2+C3"] / "src/chapters/counting.rs"
for name, cmd in (
    ("math_mirror.txt", [sys.executable, "-I", "-B", HERE / "checks/math_mirror.py", STAGED, model]),
    ("order_walk.txt", [sys.executable, "-I", "-B", HERE / "checks/order_walk.py", STAGED]),
    ("order_walk-final.txt", [sys.executable, "-I", "-B", HERE / "checks/order_walk.py", final_path]),
    ("plate_budget.txt", [sys.executable, "-I", "-B", HERE / "checks/plate_budget.py"]),
):
    rc, text = run(cmd)
    save(name, text + f"\nexit {rc}\n")
    print(name, "exit", rc, "|", text.strip().splitlines()[-1][:150])
    if rc:
        failures.append(name)
rc, text = run([sys.executable, "-I", "-B", HERE / "checks/oliver_rate.py", REPO])
save("oliver_rate.txt", text)
rates = text.strip().splitlines()[-1].split()
rc, text = run([sys.executable, "-I", "-B", HERE / "checks/reading_mirror.py", ROOT_STAGE / "script.tsv", STAGED, c2c3_path, trial / MOD_REL, *rates])
save("reading_mirror.txt", text + f"\nexit {rc}\n")
print("reading_mirror exit", rc, "|", text.strip().splitlines()[-1][:150])
if rc:
    failures.append("reading_mirror")
rc, text = run(["python3", "-B", HERE / "checks/c3_synthetic.py", FRAMES, OUT / "c3-digit-on-zero-NOT-A-RENDER.png"])
print("c3_synthetic exit", rc, text.strip()[:120])
if rc:
    failures.append("c3_synthetic")

# ------------------------------------------------------------- who else stages this file
report = []
for patch in sorted(OPUS.rglob("*.patch")):
    if patch.is_relative_to(HERE) or patch.is_relative_to(TEACH):
        continue
    text = patch.read_text(encoding="utf8", errors="replace")
    touched = [n for n in ("chapters/counting.rs", "chapters/mod.rs", "chapters/draw.rs") if f"+++ b/{CRATE_REL}/src/{n}" in text]
    if touched:
        report.append(f"{patch.relative_to(OPUS)} {sha(patch)[:12]} touches {[t.split('/')[1] for t in touched]}")
save("overlap.txt", "\n".join(report) + "\n")
print(f"{len(report)} other staged patches touch counting.rs, mod.rs or draw.rs")

print("\nFAILURES:" if failures else "\nall driver expectations hold")
for fault in failures:
    print(" ", fault)
sys.exit(1 if failures else 0)
