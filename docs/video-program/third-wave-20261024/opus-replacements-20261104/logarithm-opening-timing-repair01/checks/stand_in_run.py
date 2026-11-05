"""Compile and test the three candidates against stand-in crates, with mutation controls for the new tests.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-opening-timing-repair01/checks/stand_in_run.py SCRATCH_DIR

Writes only under SCRATCH_DIR. Bare rustc and clippy-driver on the repo's pinned toolchain, no Cargo, no fmn, no real
Kit. It uses the stand-ins of logarithm-repair01-04/checks/stub, copied and brought up to canonical: Stage::get_width,
Kit::mark, draw.rs's test module dropped because it needs the real engine, and chapter 00 added to the harness with
the four calls it alone makes (Shape::moved_to, Scene::mobjects, Kit::allow_crossing, explainer_kit::hex).

This finds type, borrow, panic and bookkeeping errors, and it compares estimated overlaps on estimated text boxes. It
proves nothing about a real signature, TeX, a glyph or a frame.
"""
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path.cwd()
OPUS = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HERE = OPUS / "logarithm-opening-timing-repair01"
OLD = OPUS / "logarithm-repair01-04/checks"
CRATE = "explainer_corpus/logarithm_mysteries"
CHANGED = ("ratio_ruler", "folded_decade", "slide_rule")
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
    shutil.copyfile(REPO / CRATE / "script.tsv", dest / "script.tsv")
    for f in ("facts.rs", "model.rs"):
        shutil.copyfile(REPO / CRATE / "src" / f, dest / "src" / f)
    for f in ALL + ("draw",):
        shutil.copyfile(REPO / CRATE / "src/chapters" / f"{f}.rs", dest / "src/chapters" / f"{f}.rs")
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
check(len(results) == 2 and int(results[1]) == int(results[0]) + 5, f"tests passed {results}: base, then candidate with five more")
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

print("\n-- mutation controls: each must fail as named")
links = ["-L", str(build), "--extern", f"fmn={build}/libfmn.rlib", "--extern", f"explainer_kit={build}/libexplainer_kit.rlib"]
controls = (
    ("no mutation (positive control)", None, None, None, "pass"),
    ("F1 axis before the numbers", "ratio_ruler", b"read_out(stage, &kit, &labels)?;", b"fade_in_lagged(stage, &ticks, 1.0, 0.15)?;\n        read_out(stage, &kit, &labels)?;", "ratio_ruler::tests::the_ruler_waits_for_the_words_that_ask_for_it"),
    ("F1 say moved after the dots", "ratio_ruler", b'        kit.say(stage, "00_2")?;\n        fade_in_lagged(stage, &ticks, 1.0, 0.15)?;', b'        fade_in_lagged(stage, &ticks, 1.0, 0.15)?;\n        kit.say(stage, "00_2")?;', "ratio_ruler::tests::the_ruler_waits_for_the_words_that_ask_for_it"),
    ("F1 phrases out of order", "ratio_ruler", b'    "eight hundred and eighty",\n    "seventeen sixty",', b'    "seventeen sixty",\n    "eight hundred and eighty",', "ratio_ruler::tests::the_numbers_arrive_in_the_order_the_voice_reads_them"),
    ("F1 a phrase not in the script", "ratio_ruler", b'    "their frequencies",\n', b'    "their frequency",\n', "ratio_ruler::tests::the_numbers_arrive_in_the_order_the_voice_reads_them"),
    ("F2 digits leave before the laps", "folded_decade", b"        fade_out_all(stage, &lap_things, laps_run)?;\n        let mut digits = vec![land, land_word];\n        digits.extend(labels.iter().copied());\n        fade_out_all(stage, &digits, DIGITS_FADE)?;\n", b"        let mut digits = vec![land, land_word];\n        digits.extend(labels.iter().copied());\n        fade_out_all(stage, &digits, DIGITS_FADE)?;\n        fade_out_all(stage, &lap_things, laps_run)?;\n", "folded_decade::tests::the_digits_outlast_the_laps_and_leave_before_the_dial_lies_flat"),
    ("F2 share not the script's", "folded_decade", b"const DIGITS_SHARE: f64 = 0.58;", b"const DIGITS_SHARE: f64 = 0.4;", "folded_decade::tests::the_digits_stay_until_they_are_named_and_the_lay_ends_on_the_next_sentence"),
    ("F2 lay fixed at 2.0 again", "folded_decade", b"tween(p_lay, 1.0, lay)", b"tween(p_lay, 1.0, 2.0)", "several"),
    ("F2 lay misses the sentence", "folded_decade", b"(laps, (s1 - now - laps - DIGITS_FADE).max(LAY_SHORTEST))", b"(laps, LAY_UNTIMED)", "folded_decade::tests::the_digits_stay_until_they_are_named_and_the_lay_ends_on_the_next_sentence"),
    ("F3 first label loses its mark", "slide_rule", b'&format!("{}: {} of a full length", fact("slide.a")?, fact("slide.pos_a")?)', b'&format!("{} of a full length", fact("slide.pos_a")?)', "slide_rule::tests::the_printed_lengths_are_fractions_of_one_full_length"),
    ("F3 second label bare above the bar", "slide_rule", b'let c_len_word = kit.text(\n            stage,\n            &format!("{}: {}", fact("slide.b")?, fact("slide.pos_b")?),', b'let c_len_word = kit.text(\n            stage,\n            fact("slide.pos_b")?,', "slide_rule::tests::the_second_length_names_its_mark_each_time_it_is_printed"),
    ("F3 second label bare on the D row", "slide_rule", b'let c_word_fix = kit.text(\n            stage,\n            &format!("{}: {}", fact("slide.b")?, fact("slide.pos_b")?),', b'let c_word_fix = kit.text(\n            stage,\n            fact("slide.pos_b")?,', "slide_rule::tests::the_second_length_names_its_mark_each_time_it_is_printed"),
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
print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
