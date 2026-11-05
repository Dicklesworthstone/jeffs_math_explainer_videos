"""Compile and test the follow-on against stand-in crates, with mutation controls and a run with a stand-in mark.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-opening-oliver-timing01/checks/stand_in_run.py SCRATCH_DIR

Writes only under SCRATCH_DIR. Bare rustc and clippy-driver on the repo's pinned toolchain, no Cargo, no fmn, no real
Kit. The chapters, facts and model are root's stage07 candidate, the exact baseline; the stand-ins are those of
logarithm-repair01-04/checks/stub, adapted as in the sealed logarithm-opening-timing-repair01 harness. phrase_marks in
the stand-in returns one stand-in mark only when STANDIN_FORGETS is set.

This finds type, borrow, panic and bookkeeping errors. It proves nothing about a real signature, the real loader, a
real take, TeX, a glyph or a frame. The 2.2 s mark is a stand-in, not a measurement.
"""
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path.cwd()
OPUS = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HERE = OPUS / "logarithm-opening-oliver-timing01"
SEALED = OPUS / "logarithm-opening-timing-repair01"
STAGE07 = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/candidate"
OLD = OPUS / "logarithm-repair01-04/checks"
CRATE = "explainer_corpus/logarithm_mysteries"
CHANGED = ("folded_decade",)
ALL = ("ratio_ruler", "building_ruler", "slide_rule", "folded_decade", "undoing_growth")
out = Path(sys.argv[1]).resolve()
fails = 0


def clean(text):
    return text.replace(str(out), "SCRATCH").replace(str(REPO), "REPO")


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def tree(name, overrides):
    dest = out / name / CRATE
    (dest / "src/chapters").mkdir(parents=True, exist_ok=True)
    shutil.copyfile(STAGE07 / CRATE / "script.tsv", dest / "script.tsv")
    for f in ("facts.rs", "model.rs"):
        shutil.copyfile(STAGE07 / CRATE / "src" / f, dest / "src" / f)
    for f in ALL + ("draw",):
        shutil.copyfile(STAGE07 / CRATE / "src/chapters" / f"{f}.rs", dest / "src/chapters" / f"{f}.rs")
    draw = dest / "src/chapters/draw.rs"
    lines = draw.read_text(encoding="utf-8").split("#[cfg(test)]")[0].rstrip("\n").split("\n")
    while lines and (lines[-1].strip().startswith(("///", "#[")) or not lines[-1].strip()):
        lines.pop()
    draw.write_text("\n".join(lines) + "\n", encoding="utf-8")
    for f, data in overrides.items():
        (dest / "src/chapters" / f"{f}.rs").write_bytes(data)


def replace_once(path, old, new):
    text = path.read_text(encoding="utf-8")
    if old not in text:
        sys.exit(f"stand-in adaptation: not found in {path.name}: {old}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


shutil.copytree(OLD / "stub", out / "checks/stub", dirs_exist_ok=True)
shutil.copyfile(OLD / "run_against_stub.py", out / "checks/run_against_stub.py")
kit = out / "checks/stub/kit_stub.rs"
text = kit.read_text(encoding="utf-8")
text = re.sub(r'#\[path = "[^"]*crates/explainer_kit/src/([a-z_]+\.rs)"\]', lambda m: f'#[path = "{REPO}/crates/explainer_kit/src/{m.group(1)}"]', text)
text = text.replace("    /// The prediction hold: music alone", "    pub fn mark(&self, _: &Stage<'_>, _: &str) {}\n\n    /// The prediction hold: music alone", 1)
text = text.replace(
    "    pub fn phrase_marks(&self, _: &str) -> HashMap<usize, f64> {\n        HashMap::new()\n    }",
    """    pub fn phrase_marks(&self, id: &str) -> HashMap<usize, f64> {
        let mut marks = HashMap::new();
        if id == "03_3" {
            if let Ok(value) = std::env::var("STANDIN_FORGETS") {
                marks.insert(35, value.parse::<f64>().expect("a number"));
            }
        }
        marks
    }""",
    1,
)
if "STANDIN_FORGETS" not in text:
    sys.exit("stand-in adaptation: phrase_marks not found")
kit.write_text(text, encoding="utf-8")
replace_once(out / "checks/stub/fmn_stub.rs", "    pub fn get_center(&self, mob: Mob) -> Vec3 {", "    pub fn get_width(&self, _: Mob) -> f64 {\n        1.0\n    }\n\n    pub fn get_center(&self, mob: Mob) -> Vec3 {")
# chapter 00's calls, which chapters 01 to 04 never needed
with open(out / "checks/stub/fmn_stub.rs", "a", encoding="utf-8") as stub:
    stub.write("""
impl Shape {
    pub fn moved_to(mut self, to: Vec3) -> Self {
        let n = self.pts.len().max(1) as f64;
        let (cx, cy) = (self.pts.iter().map(|q| q[0]).sum::<f64>() / n, self.pts.iter().map(|q| q[1]).sum::<f64>() / n);
        for q in &mut self.pts {
            q[0] += to[0] - cx;
            q[1] += to[1] - cy;
        }
        self
    }
}

impl Scene {
    pub fn mobjects(&self) -> Vec<Mob> {
        stub::with(|book| {
            let shown = |r: &stub::Record| r.life == stub::Life::Shown && r.parent.is_none();
            book.mobs.iter().enumerate().filter(|(_, r)| shown(r)).map(|(k, _)| Mob(k)).collect()
        })
    }
}
""")
with open(kit, "a", encoding="utf-8") as stub:
    stub.write("""
pub const BACKGROUND: &str = "background";

pub fn hex(_: &str) -> Srgb {
    Srgb("background")
}

impl Kit {
    pub fn allow_crossing(&self, _: Mob) {}
}
""")
harness = out / "checks/stub/harness_main.rs"
replace_once(harness, "use chapters::{building_ruler, folded_decade, slide_rule, undoing_growth};", "use chapters::{building_ruler, folded_decade, ratio_ruler, slide_rule, undoing_growth};")
replace_once(harness, '        "01_" => {\n', '        "00_" => {\n            let mut scene = ratio_ruler::RatioRuler { kit: kit.clone() };\n            scene.construct(&mut stage)?;\n            scene.name().to_owned()\n        }\n        "01_" => {\n')
replace_once(harness, 'for prefix in ["01_", "02_", "03_", "04_"]', 'for prefix in ["00_", "01_", "02_", "03_", "04_"]')
replace_once(out / "checks/run_against_stub.py", 'CHAPTERS = ("building_ruler", "slide_rule", "folded_decade", "undoing_growth")', f"CHAPTERS = {ALL!r}")

candidates = {f: (HERE / CRATE / "src/chapters" / f"{f}.rs").read_bytes() for f in CHANGED}
tree("base", {})
tree("candidate", candidates)
build = out / "build"
build.mkdir(exist_ok=True)
done = subprocess.run([sys.executable, "-I", "-B", str(out / "checks/run_against_stub.py"), str(build), "base", "candidate"], cwd=REPO, capture_output=True, text=True)
report = clean(done.stdout + done.stderr)
keep = re.compile(r"^(######|-- |test result|test .*FAILED|error|\(clippy clean\)|== |chapter ends|\d+ object-ledger|\d+ estimated|.*failed:|  +[0-9.]+  |  \(in motion\))")
for line in report.split("\n"):
    if keep.match(line):
        print(line)
if done.returncode:
    print(report[-3000:])
check(done.returncode == 0, "stand-in run finished for both trees")
if done.returncode:
    print("RESULT FAIL")
    sys.exit(1)
results = re.findall(r"test result: ok\. (\d+) passed; 0 failed", report)
check(len(results) == 2 and int(results[1]) == int(results[0]) + 1, f"tests passed {results}: base, then candidate with one more")
check("(clippy clean)" in report, "clippy-driver -D warnings clean on the candidate, with and without tests")

print("\n-- the voice and the picture, cue by cue: id, start, voice, picture over the voice, latest beat, in sentence")
base_part, cand_part = report.split("######## candidate ########")
row = re.compile(r"^(\d\d_\d+)\t.*$", re.M)
for rate in ("2.68", "2.3", "3.1"):
    def rows(part):
        seg = part[part.index(f"-- scene run at {rate}") :]
        seg = seg[: seg.index("-- scene run", 5)] if "-- scene run" in seg[5:] else seg
        return {m.group(1): m.group(0) for m in row.finditer(seg)}
    b, c = rows(base_part), rows(cand_part)
    differs = [k for k in b if b[k] != c.get(k)]
    print(f"at {rate} words per second, {len(b)} cues, rows that differ: {differs}")
    for k in differs:
        print("  base", b[k].replace("\t", " "))
        print("  cand", c[k].replace("\t", " "))
    starts = lambda d: {k: v.split("\t")[1] for k, v in d.items()}
    check(starts(b) == starts(c), f"at {rate}: every cue starts at the same time in both trees")
ends = lambda part: re.findall(r"== (\S+) at ([0-9.]+) words per second.*?chapter ends at ([0-9.]+) s", part, re.S)
check(ends(base_part) == ends(cand_part) and len(ends(base_part)) == 15, f"all {len(ends(base_part))} scene runs end at the same time in both trees")
check(re.findall(r"^(\d+) object-ledger problems", cand_part, re.M) == ["0"] * 15, "no fade or removal of an object that is not on screen, in any candidate run")


def overlaps(part):
    return sorted(set(re.findall(r"^ +[0-9.]+  (.*)$", part, re.M)))


new = [x for x in overlaps(cand_part) if x not in overlaps(base_part)]
gone = [x for x in overlaps(base_part) if x not in overlaps(cand_part)]
print("estimated overlaps at rest, new in the candidate:", new)
print("estimated overlaps at rest, gone from the candidate:", gone)

print("\n-- the candidate's scene again, with a stand-in mark of 2.2 s on byte 35 of 03_3")
scene = build / "candidate_scene"
for rate in ("2.68", "2.3", "3.1"):
    plain = subprocess.run([str(scene), rate, str(build / f"plain-{rate}.tsv")], capture_output=True, text=True, cwd=REPO)
    marked = subprocess.run([str(scene), rate, str(build / f"marked-{rate}.tsv")], capture_output=True, text=True, cwd=REPO, env=dict(os.environ, STANDIN_FORGETS="2.2"))
    check(plain.returncode == 0 and marked.returncode == 0, f"at {rate}: both runs finish")
    plain_rows = (build / f"plain-{rate}.tsv").read_text(encoding="utf-8").split("\n")
    marked_rows = (build / f"marked-{rate}.tsv").read_text(encoding="utf-8").split("\n")
    changed_rows = [(x, y) for x, y in zip(plain_rows, marked_rows) if x != y]
    print(f"  at {rate}: {len(plain_rows)} snapshot rows, {len(changed_rows)} differ")
    for x, y in changed_rows[:2]:
        print("    plain ", x[:150])
        print("    marked", y[:150])
    check(not changed_rows and len(plain_rows) == len(marked_rows), f"at {rate}: the same picture at every sentence start, because the lay still ends on sentence 1")
    chapter = lambda o: re.findall(r"chapter ends at ([0-9.]+) s", o)
    check(chapter(plain.stdout) == chapter(marked.stdout), f"at {rate}: chapter ends unchanged {chapter(marked.stdout)}")
    check(re.findall(r"^(\d+) object-ledger problems", marked.stdout, re.M) == re.findall(r"^(\d+) object-ledger problems", plain.stdout, re.M), f"at {rate}: object-ledger counts unchanged")

print("\n-- patch round trip, on a scratch copy")
trip = out / "trip"
target = trip / CRATE / "src/chapters/folded_decade.rs"
target.parent.mkdir(parents=True, exist_ok=True)
base_bytes = (SEALED / CRATE / "src/chapters/folded_decade.rs").read_bytes()
target.write_bytes(base_bytes)
subprocess.run(["git", "init", "-q"], cwd=trip, check=True)
patch = HERE / "digits-boundary.patch"
done_apply = subprocess.run(["git", "apply", "--check", str(patch)], cwd=trip, capture_output=True, text=True)
check(done_apply.returncode == 0, f"patch checks against the sealed baseline {done_apply.stderr[:120]}")
subprocess.run(["git", "apply", str(patch)], cwd=trip, check=True)
check(target.read_bytes() == candidates["folded_decade"], "applied, it gives the candidate byte for byte")
subprocess.run(["git", "apply", "-R", str(patch)], cwd=trip, check=True)
check(target.read_bytes() == base_bytes, "reversed, it gives the baseline byte for byte")

print("\n-- mutation controls: each must fail as named")
links = ["-L", str(build), "--extern", f"fmn={build}/libfmn.rlib", "--extern", f"explainer_kit={build}/libexplainer_kit.rlib"]
NEW = "folded_decade::tests::a_mark_on_forgets_ends_the_digits_fade_and_a_mark_outside_the_sentence_is_ignored"
OLD = "folded_decade::tests::the_digits_stay_until_they_are_named_and_the_lay_ends_on_the_next_sentence"
controls = (
    ("no mutation (positive control)", None, None, None, "pass"),
    ("the scene ignores the mark", "folded_decade", b"spoken, forgets_mark(&kit));", b"spoken, None);", NEW),
    ("a mark outside the sentence is trusted", "folded_decade", b".filter(|at| (s0..s1).contains(at))", b"", NEW),
    ("the fade starts on the mark", "folded_decade", b"|at| at - DIGITS_FADE)", b"|at| at)", NEW),
    ("the mark is read as scene time", "folded_decade", b"forgets.map(|m| s0 + m)", b"forgets", NEW),
    ("the phrase is not in the script", "folded_decade", b'const FORGETS_PHRASE: &str = "forgets";', b'const FORGETS_PHRASE: &str = "forgot";', NEW),
    ("the phrase occurs more than once", "folded_decade", b'const FORGETS_PHRASE: &str = "forgets";', b'const FORGETS_PHRASE: &str = "laps";', NEW),
    ("the share is dropped when there is no mark", "folded_decade", b"mark.map_or(s0 + (s1 - s0) * DIGITS_SHARE, |at| at - DIGITS_FADE)", b"mark.map_or(s0, |at| at - DIGITS_FADE)", OLD),
)
support = out / "base" / CRATE / "src"
chapters = out / "mutant" / CRATE / "src/chapters"
rows_ = ["#![allow(dead_code, clippy::should_implement_trait)]", f'#[path = "{support}/facts.rs"]', "pub mod facts;", f'#[path = "{support}/model.rs"]', "pub mod model;", "pub mod chapters {", f'    #[path = "{support}/chapters/draw.rs"]', "    pub mod draw;"]
for name in ALL:
    rows_ += [f'    #[path = "{chapters}/{name}.rs"]', f"    pub mod {name};"]
rows_ += ["}", f'include!("{out}/checks/stub/harness_main.rs");']
root = build / "mutant_root.rs"
root.write_text("\n".join(rows_) + "\n", encoding="utf-8")
binary = build / "mutant_tests"
env = dict(os.environ, TREE_ROOT=str(out / "mutant" / CRATE))
for what, file, old, new, expect in controls:
    files = dict(candidates)
    if file is not None:
        parts = files[file].split(b"#[cfg(test)]")
        if parts[0].count(old) != 1:
            sys.exit(f"control {what}: the text is not once in the scene code")
        # the change is in the scene code and constants, never in the test's own text
        parts[0] = parts[0].replace(old, new)
        files[file] = b"#[cfg(test)]".join(parts)
    tree("mutant", files)
    comp = subprocess.run(["rustc", "--edition", "2024", *links, "--crate-name", "chapter", "--test", "-o", str(binary), str(root)], capture_output=True, text=True, env=env, cwd=REPO)
    if comp.returncode:
        first = clean(comp.stderr).split("\n")[0][:140]
        check(False, f"{what}: did not compile: {first}")
        continue
    run = subprocess.run([str(binary), "--test-threads=1"], capture_output=True, text=True)
    failed = [x.replace("chapters::", "") for x in re.findall(r"^test (\S+) \.\.\. FAILED", run.stdout, re.M)]
    if expect == "pass":
        check(run.returncode == 0, f"{what}: compiles, and {re.findall(r'test result: (.*?);', run.stdout)} tests")
    elif expect == "several":
        check(len(failed) >= 2, f"{what}: compiles, fails {failed}")
    else:
        check(failed == [expect], f"{what}: compiles, fails {failed}")

print("\n-- probe: a scratch copy of the candidate prints the two beats it passes on, without and with the stand-in mark")
files = dict(candidates)
line = b"        fade_out_all(stage, &lap_things, laps_run)?;"
if files["folded_decade"].count(line) != 1:
    sys.exit("probe: the fade line is not once in the source")
files["folded_decade"] = files["folded_decade"].replace(line, b'        eprintln!("BEATS {laps_run:.3} {lay:.3}");\n' + line)
tree("mutant", files)
probe = build / "probe_scene"
comp = subprocess.run(["rustc", "--edition", "2024", *links, "--crate-name", "chapter", "-o", str(probe), str(root)], capture_output=True, text=True, cwd=REPO, env=env)
check(comp.returncode == 0, "the probe compiles")
for rate in ("2.68", "2.3", "3.1"):
    beats = lambda extra: re.findall(r"BEATS ([0-9.]+) ([0-9.]+)", subprocess.run([str(probe), rate, str(build / "probe.tsv")], capture_output=True, text=True, cwd=REPO, env=dict(os.environ, **extra)).stderr)
    plain, marked = beats({}), beats({"STANDIN_FORGETS": "2.2"})
    print(f"  at {rate}: share laps, lay {plain}; with the mark {marked}")
    check(len(plain) == len(marked) == 1 and plain != marked, f"at {rate}: the mark changes the beats the scene uses")
    check(abs(float(marked[0][0]) - 1.4) < 0.05, f"at {rate}: laps run 2.2 - 0.3 - 0.5 = 1.4 s with the mark, within a frame or so")

print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
