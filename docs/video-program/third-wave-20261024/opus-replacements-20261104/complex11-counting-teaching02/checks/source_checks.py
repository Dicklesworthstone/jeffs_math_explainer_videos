"""Source-level checks on the teaching02 staging against root's counting candidate and the canonical crate.

Standard library plus Pillow, and the `patch` and `rustc` commands. Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-teaching02/checks/source_checks.py

It proves what a text comparison can: the baselines, every patch's round trip, the exact timeline delta,
the exact string delta, the sentence anchors, the storyboard and unit bindings, the reading-window
arithmetic, widths in the film's own faces, the overrun budget on estimated sentence times, the relocated
holds under root's own rules and the repo's real direction_contract, and the fit beside the glyph patch.
It proves nothing about pictures or voice. Temporary copies go in a new temporary directory.
"""
import difflib
import glob
import hashlib
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

from PIL import ImageFont

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-teaching02/"
FROZEN = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-review-repair01/"
ROOT = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/counting-wording-repair01/"
TOPIC = "explainer_corpus/complex_analysis_visual/"
CHAPTERS = TOPIC + "src/chapters/"
COUNTING = CHAPTERS + "counting.rs"
MOD = CHAPTERS + "mod.rs"
BOARD = TOPIC + "docs/storyboard.md"
OLD_MAP = TOPIC + "docs/narration-direction-map-oliver10.json"
NEW_MAP = TOPIC + "docs/narration-direction-map-oliver11.json"
UNITS = TOPIC + "narration_units.json"
STAGED = HERE + COUNTING
STAGED_MAP = HERE + "holds/narration-direction-map-oliver11.json"
P = HERE + "patches/"
GLYPH = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-glyph-spacing-repair01/complex-glyph-spacing.patch"
CM = glob.glob(os.path.expanduser("~/.cargo/git/checkouts/franken_markdown-*/*/fmd-font/fonts/computer-modern/"))[0]
TIMED = [
    "fade_in_all", "fade_out_all", "stage.play", "stage.play_prepared", "stage.wait", "wait_until",
    "line.wait_sentence", "line.wait_for", "line.wait_last", "line.until", "line.sentence", "kit.hold", "kit.music_breath",
    "kit.listen", "kit.finish", "reading_window", "predict", "precisely_now", "names_now", "pause", "say",
]
SAY = re.compile(r'\bsay\(&kit, stage, "(\d\d_\d+)"\)')

fails = 0


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def read(path):
    return open(path, encoding="utf-8").read()


def cue(path, id_):
    for line in open(path, encoding="utf-8"):
        parts = line.rstrip("\n").split("\t")
        if parts[0] == id_:
            return parts[1]
    raise KeyError(id_)


def scene(text):
    """The film code: everything before the test module, with comments removed."""
    code = text[: text.index("#[cfg(test)]\nmod tests")]
    out, i, in_string, raw = [], 0, False, 0
    while i < len(code):
        c = code[i]
        if in_string:
            out.append(c)
            if c == "\\" and raw == 0:
                out.append(code[i + 1])
                i += 1
            elif c == '"':
                in_string = False
            i += 1
        elif c == '"':
            raw = 1 if i > 0 and code[i - 1] == "r" else 0
            in_string = True
            out.append(c)
            i += 1
        elif code.startswith("//", i):
            while i < len(code) and code[i] != "\n":
                i += 1
        else:
            out.append(c)
            i += 1
    return "".join(out)


def strings(text):
    return sorted(re.findall(r'"(?:[^"\\]|\\.)*"', scene(text)))


def closing_paren(text, start):
    depth, at, in_string = 0, start, False
    while True:
        c = text[at]
        if in_string:
            if c == '"':
                in_string = False
        elif c == '"':
            in_string = True
        elif c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return at
        at += 1


def timeline(text):
    """Every timed call in film order, whitespace gone, a fade's mob list reduced to its time."""
    code = scene(text)
    pattern = re.compile(r"(?<![\w.])(" + "|".join(re.escape(t) for t in TIMED) + r")\(")
    out = []
    for m in pattern.finditer(code):
        name = m.group(1)
        if code[max(0, m.start() - 3) : m.start()] == "fn ":
            continue
        end = closing_paren(code, m.end() - 1)
        args = re.sub(r"\s+", "", code[m.end() : end]).rstrip(",")
        if name in ("fade_in_all", "fade_out_all"):
            args = args.rsplit(",", 1)[-1]
        out.append(f"{name}({args})")
    return out


def tests(text):
    return re.findall(r"#\[test\]\s*\n\s*fn (\w+)", text)


def apply(folder, patch, *flags):
    return subprocess.run(["patch", *flags, "-p1", "--no-backup-if-mismatch", "-i", os.path.abspath(patch)], cwd=folder, capture_output=True, text=True, check=False)


def clean(done):
    return done.returncode == 0 and "offset" not in done.stdout and "fuzz" not in done.stdout


def offsets(text):
    """draw.rs's sentence_offsets, again."""
    return [0] + [i + 2 for i in range(len(text) - 2) if text[i] in ".?!" and text[i + 1] == " "]


print("== baselines still match")
for line in read(HERE + "checks/baseline.sha256").splitlines():
    digest, path = line.split("  ")
    check(sha(path) == digest, f"{path.rsplit('/', 2)[-2]}/{path.rsplit('/', 1)[-1]} {digest[:16]}")

print("== scope: one staged source file; everything else is a patch or the candidate map")
staged_files = sorted(os.path.join(d, n) for d, _, names in os.walk(HERE + "explainer_corpus") for n in names)
check(staged_files == [STAGED], str([f.rsplit("/", 1)[-1] for f in staged_files]))

work = tempfile.mkdtemp(prefix="complex11-teaching-source-")


def tree(name, *files):
    folder = os.path.join(work, name)
    for src, rel in files:
        os.makedirs(os.path.dirname(os.path.join(folder, rel)), exist_ok=True)
        shutil.copy(src, os.path.join(folder, rel))
    return folder


print("== patches apply clean, equal what they claim, and reverse")
base, staged = read(ROOT + "counting.rs"), read(STAGED)
folder = tree("root", (ROOT + "counting.rs", COUNTING))
target = os.path.join(folder, COUNTING)
check(clean(apply(folder, P + "counting-from-root.patch")), "counting-from-root.patch on root's candidate")
check(sha(target) == sha(STAGED), f"gives the staged file {sha(STAGED)}")
check(apply(folder, P + "counting-from-root.patch", "-R").returncode == 0 and sha(target) == sha(ROOT + "counting.rs"), "reverses to root's candidate")
# the frozen 01 + 03 chain, then the delta
folder = tree("chain", (ROOT + "counting.rs", COUNTING))
target = os.path.join(folder, COUNTING)
check(clean(apply(folder, FROZEN + "patches/01-reach-label.patch")), "the frozen 01 on root's candidate")
check(clean(apply(folder, FROZEN + "patches/03-optional-a-name.patch")), "the frozen 03 on top")
check(sha(target) == "20ad375dcce1c1172a976a546507a108ce12e746c8fea939cefbc0f2bc6fecdb", "01 + 03 is the file the frozen handoff named, 20ad375d")
base13 = read(target)
check(clean(apply(folder, P + "counting-delta-on-01-03.patch")), "counting-delta-on-01-03.patch on that file")
check(sha(target) == sha(STAGED), "and gives the same staged file")
# mod.rs
folder = tree("mod", (MOD, MOD))
target = os.path.join(folder, MOD)
check(clean(apply(folder, P + "mod-registry.patch")), "mod-registry.patch on canonical mod.rs")
mod_after, mod_sha = read(target), sha(target)
print(f"     mod.rs after {mod_sha}")
added = [l[1:].strip() for l in read(P + "mod-registry.patch").splitlines() if l.startswith("+") and not l.startswith("+++")]
removed = [l for l in read(P + "mod-registry.patch").splitlines() if l.startswith("-") and not l.startswith("---")]
check(not removed and added == ["counting::EVERY_OUTPUT,", "counting::ZERO_KEPT,", "counting::SAME_ROOTS,", "counting::BEFORE_DIVIDING_TEX,", "counting::AFTER_DIVIDING_TEX,"], f"it adds exactly {len(added)} rows and removes none")
text_at, tex_at, mono_at = mod_after.index("pub const TEXT:"), mod_after.index("pub const TEX:"), mod_after.index("pub const MONO:") if "pub const MONO:" in mod_after else len(mod_after)
check(text_at < mod_after.index("counting::EVERY_OUTPUT") < tex_at and tex_at < mod_after.index("counting::BEFORE_DIVIDING_TEX") < mono_at, "three in TEXT and two in TEX")
check(apply(folder, P + "mod-registry.patch", "-R").returncode == 0 and sha(target) == sha(MOD), "reverses")
# storyboard
folder = tree("board", (BOARD, BOARD))
target = os.path.join(folder, BOARD)
check(clean(apply(folder, P + "storyboard.patch")), "storyboard.patch on the canonical storyboard")
patched_board = read(target)
print(f"     storyboard.md after {sha(target)}")
check(cue(ROOT + "script.tsv", "11_7") in patched_board and cue(ROOT + "before-script.tsv", "11_7") not in patched_board, "root's exact 11_7 in the row, the old one out")
check(apply(folder, P + "storyboard.patch", "-R").returncode == 0 and sha(target) == sha(BOARD), "reverses")
# units
folder = tree("units", (UNITS, UNITS))
target = os.path.join(folder, UNITS)
check(clean(apply(folder, P + "narration-units.patch")), "narration-units.patch on the canonical plan")
units_after_path = target
units_after = json.loads(read(target))
print(f"     narration_units.json after {sha(target)}")
# map
folder = tree("map", (OLD_MAP, NEW_MAP))
target = os.path.join(folder, NEW_MAP)
check(clean(apply(folder, P + "direction-map-from-oliver10.patch")), "direction-map-from-oliver10.patch on a copy of oliver10")
check(sha(target) == sha(STAGED_MAP), f"gives holds/narration-direction-map-oliver11.json {sha(STAGED_MAP)}")

print("== the timeline: what is new, and that nothing else moved")
a, b = timeline(base), timeline(staged)
matcher = difflib.SequenceMatcher(a=a, b=b, autojunk=False)
ops = [op for op in matcher.get_opcodes() if op[0] != "equal"]
inserted, removed_calls = [], []
for tag, i1, i2, j1, j2 in ops:
    removed_calls.extend(a[i1:i2])
    inserted.extend(b[j1:j2])
expected_in = [
    'line.wait_for(stage,"complexpolynomial")',
    "fade_in_all(0.3)",
    "line.sentence(1)",
    "stage.play(tween(turning.turned,1.0,rt)?)",
    "line.wait_sentence(stage,1)",
    "fade_in_all(0.3)",
    'line.until(stage,"whichleaves",1.2)',
    "stage.play(tween(turning.turned,0.0,rt)?)",
    "fade_out_all(0.2)",
    "fade_in_all(0.3)",
    "fade_out_all(0.3)",
]
print(f"     {len(a)} timed calls -> {len(b)}; inserted, in film order:")
for call in inserted:
    print("       + " + call)
check(sorted(inserted) == sorted(expected_in), f"exactly these {len(expected_in)} calls are new")
check(not removed_calls, "no timed call of root's is gone or changed")
start = b.index('say(&kit,stage,"11_7")')
end = b.index("line.wait_sentence(stage,3)", start)
first = next(j1 for tag, i1, i2, j1, j2 in ops)
last = max(j2 for tag, i1, i2, j1, j2 in ops)
check(start < first and last <= end, "every new call falls between 11_7's say and the wait for sentence 3, so nothing after that anchor moves on its account")
region = b[start : end + 1]
check(region.index('line.wait_for(stage,"complexpolynomial")') < region.index("line.wait_sentence(stage,1)") < region.index("line.wait_sentence(stage,2)"), "the order of the anchors holds")
check("say" not in "".join(inserted) and not any("wait(" in c for c in inserted), "no new cue and no new fixed wait: every new call is a fade, a tween or an anchor")
check(timeline(staged.replace("fade_in_all(stage, &[every_output], 0.3)?;", "fade_in_all(stage, &[every_output], 0.5)?;", 1)) != b, "control: a changed fade time is detected")
check(timeline(staged.replace("fade_in_all(stage, &[every_output], 0.3)?;", "fade_in_all(stage, &[every_output], 0.3)?;\n    stage.wait(0.5)?;", 1)) != b, "control: an added wait is detected")

print("== the strings and the tests")
old_strings, new_strings = strings(base), strings(staged)
gained = sorted((set(new_strings) - set(old_strings)))
lost = sorted(set(old_strings) - set(new_strings))
for s in gained:
    print(f"     + {s}")
for s in lost:
    print(f"     - {s}")
check(lost == ['"size compressed; direction and turns around zero unchanged"'], "the only string gone is the old card")
check('"output size compressed; direction and turns around zero unchanged"' in gained, "the card now says output")
check(set(tests(base)) <= set(tests(staged)), f"{len(tests(base))} of root's tests kept, new: {sorted(set(tests(staged)) - set(tests(base)))}")
check(staged.count("Some(Func::Quintic)") == 3, "the think-time test's count of Some(Func::Quintic) is still 3")
check(base.count("the_roots_argument_waits_on_the_right_sentences") == 0, "the frozen sentence test is this folder's, not root's")

print("== root's script is the script: no word moves")
old, new = cue(ROOT + "before-script.tsv", "11_7"), cue(ROOT + "script.tsv", "11_7")
at = offsets(new)
check(len(at) == 7, "11_7 has seven sentences by draw.rs's rule")
check(new.count("complex polynomial") == 1 and new.count("which leaves") == 1, "the two phrase anchors are each in the script once")
check(new.index("complex polynomial") < at[1] and at[1] < new.index("which leaves") < at[2], "'complex polynomial' is in sentence 0, 'which leaves' in sentence 1")
check(not (set(strings(staged)) & {f'"{new}"'}), "no spoken sentence pasted into the code")

print("== phrase marks: none is needed")
draw = read(CHAPTERS + "draw.rs")
check("if let Some(time) = self.phrase_marks.get(&byte)" in draw and "self.text[..byte].chars().count()" in draw, "without a mark, line.at interpolates inside the measured sentence, so both phrase times are estimates until a word-aligned take; with one, it is used")

print("== the overrun budget, on character-share sentence times (estimates, not Oliver)")
rate_old = 31.938 / len(old)
for name, factor in (("scratch rate", 1.0), ("15% faster", 0.85), ("30% faster", 0.7)):
    rate = rate_old * factor
    s = [x * rate for x in at] + [len(new) * rate]
    phrase = lambda p, k: s[k] + (s[k + 1] - s[k]) * (new.index(p) - at[k]) / (at[k + 1] - at[k])
    now = 0.3 + 0.4 + 0.4
    now = max(now, phrase("complex polynomial", 0)) + 0.3
    rt0 = min(1.4, max(0.7, s[1] - now))
    now += rt0
    late0 = now - s[1]
    now = max(now, s[1]) + 0.3
    rt1 = max(1.2, phrase("which leaves", 1) - now)
    now += rt1
    landed = now - phrase("which leaves", 1)
    now += 0.2 + 0.3
    late1 = now - s[2]
    now = max(now, s[2]) + 0.3 + 0.4
    late2 = now - s[3]
    print(f"     {name}: sentence starts {s[0]:.1f} {s[1]:.1f} {s[2]:.1f} {s[3]:.1f}; turn out {rt0:.2f} s, turn back {rt1:.2f} s")
    check(late0 <= 1e-9 and landed <= 1e-9 and late1 < 0 and late2 < 0, f"{name}: no act runs past its sentence ({late0:+.2f}, {landed:+.2f}, {late1:+.2f}, {late2:+.2f} s)")

print("== storyboard: the real check_video and Script::parse, built with rustc alone")


def balanced(text, start):
    depth, at_, = 0, text.index("{", start)
    while True:
        depth += {"{": 1, "}": -1}.get(text[at_], 0)
        if text[at_] == "}" and depth == 0:
            return text[start : at_ + 1]
        at_ += 1


narration = read("crates/explainer_kit/src/narration.rs")
built_tree = os.path.join(work, "board-test")
os.makedirs(built_tree)
shutil.copy("crates/explainer_kit/src/storyboard.rs", os.path.join(built_tree, "storyboard.rs"))
with open(os.path.join(built_tree, "main.rs"), "w", encoding="utf-8") as out:
    out.write(
        "#![allow(dead_code, unused_imports)]\nmod storyboard;\nmod narration {\nuse std::collections::HashSet;\n"
        "#[derive(Clone, Debug, Default, PartialEq, Eq)]\npub enum LineGaps { #[default] Constant }\n"
        "#[derive(Clone, Debug, Default, PartialEq, Eq)]\n"
        "pub struct Script { lines: Vec<(String, String)>, all: Vec<(String, String)>, line_gaps: LineGaps }\n"
        "impl Script {\n" + balanced(narration, narration.index("pub fn parse(text: &str) -> Result<Self, String>")) + "\n"
        "pub fn lines(&self) -> &[(String, String)] { &self.lines }\n}\n}\n"
        "fn main() {\n"
        "    let args: Vec<String> = std::env::args().collect();\n"
        "    let board = std::fs::read_to_string(&args[1]).unwrap();\n"
        "    let script = std::fs::read_to_string(&args[2]).unwrap();\n"
        "    let chapters = storyboard::parse(&board).unwrap();\n"
        "    let keys: Vec<&str> = chapters.iter().map(|c| c.key.as_str()).collect();\n"
        "    match storyboard::check_video(&board, &script, &keys) {\n"
        '        Ok(()) => println!("agree"),\n'
        '        Err(e) => { println!("{}", &e[..e.len().min(90)]); std::process::exit(1); }\n'
        "    }\n"
        "}\n"
    )
built = subprocess.run(["rustc", "--edition", "2024", os.path.join(built_tree, "main.rs"), "-o", os.path.join(built_tree, "check.bin"), "--cap-lints", "allow"], capture_output=True, text=True, check=False)
check(built.returncode == 0, "check_video builds" + ("" if built.returncode == 0 else ": " + built.stderr.replace(built_tree, "<work>")[:800]))


def board_check(board_path, script_path):
    return subprocess.run([os.path.join(built_tree, "check.bin"), board_path, script_path], capture_output=True, text=True, check=False)


patched_path = os.path.join(built_tree, "patched-storyboard.md")
with open(patched_path, "w", encoding="utf-8") as out:
    out.write(patched_board)
check(board_check(BOARD, TOPIC + "script.tsv").returncode == 0, "control: canonical storyboard and canonical script agree")
check(board_check(BOARD, ROOT + "script.tsv").returncode != 0, "control: root's script against the canonical storyboard fails")
check(board_check(patched_path, ROOT + "script.tsv").returncode == 0, "root's script against this storyboard agrees")
row6 = next(l for l in patched_board.splitlines() if l.startswith("| 11_6 |"))
row7 = next(l for l in patched_board.splitlines() if l.startswith("| 11_7 |"))
check("output size compressed; direction and turns around zero unchanged" in row6 and row6.rstrip().endswith("| read 7 s |"), "11_6's picture quotes the card as drawn, and its hold column is untouched")
for needle in ("complex polynomial", "Divide", "which leaves", "Now it starts", "On a huge circle", "every output ÷ 2i  (2i ≠ 0)", "0 ÷ 2i = 0, and nothing else lands on 0", "same five roots", "argument.quintic.big"):
    check(needle in row7, f"11_7's picture names: {needle}")

print("== narration units: tools/coherent_narration.py allows 100 words")
words = len(new.split())
check(words <= 100, f"11_7 alone {words} words, unchanged by this folder")
row = next(u for u in units_after["units"] if u["id"] == "oliver_11_7")
check(row["text"] == new, "the rebound plan holds root's exact 11_7")
others = [u for u in units_after["units"] if u["id"] != "oliver_11_7"]
before_units = json.loads(read(UNITS))["units"]
check(others == [u for u in before_units if u["id"] != "oliver_11_7"], f"its other {len(others)} units are untouched, so only one cue is new to synthesize")

print("== the holds: root's validate_oliver_plan rules, on the staged file and the candidate map")
direction = json.loads(read(STAGED_MAP))
check(direction["scripts"]["complex_analysis_visual"] == sha(ROOT + "script.tsv"), "the map binds root's script")
check(direction["hold_kinds"] == json.loads(read(OLD_MAP))["hold_kinds"], f"hold_kinds unchanged {direction['hold_kinds']}")
check(direction["overrides"] == json.loads(read(OLD_MAP))["overrides"], f"the {len(direction['overrides'])} overrides unchanged")
lines = staged.split("\n")
says = [(i + 1, SAY.search(l).group(1)) for i, l in enumerate(lines) if SAY.search(l)]
rows = [h for h in direction["visual_holds"] if h["path"].endswith("/counting.rs")]
check([h["cue"] for h in rows] == ["11_5", "11_6", "11_8"], "the three counting holds")
for hold in rows:
    n = hold["line"]
    ok = hold["sha256"] == sha(STAGED) and lines[n - 1].strip() == hold["marker"]
    before_say = [c for i, c in says if i < n]
    after_say = [c for i, c in says if i > n]
    owned = before_say[-1] == hold["cue"]
    follows = (after_say[0] == hold["next_cue"]) if after_say else hold["next_cue"][:2] != hold["cue"][:2]
    check(ok and owned and follows, f"{hold['cue']} -> {hold['next_cue']}: counting.rs:{n} is the marker, owned by {hold['cue']}, followed by {after_say[0] if after_say else 'the chapter end'}")
for hold in direction["visual_holds"]:
    if not hold["path"].endswith("/counting.rs"):
        check(sha(hold["path"]) == hold["sha256"], f"{hold['cue']} in {hold['path'].rsplit('/', 1)[-1]} unchanged") if hold["cue"].startswith("11") else None
unchanged = [h for h in direction["visual_holds"] if not h["path"].endswith("/counting.rs")]
check(all(sha(h["path"]) == h["sha256"] for h in unchanged), f"the other {len(unchanged)} holds still bind their files")
for helper in direction["helpers"]:
    path = helper["path"]
    if path.endswith("/mod.rs"):
        ok = helper["sha256"] == mod_sha and mod_after.split("\n")[helper["line"] - 1].strip() == helper["marker"]
    else:
        ok = sha(path) == helper["sha256"] and read(path).split("\n")[helper["line"] - 1].strip() == helper["marker"]
    check(ok, f"helper {path.rsplit('/', 1)[-1]}:{helper['line']} {helper['marker'][:50]}")
hold_11_6 = next(h for h in rows if h["cue"] == "11_6")
tail = staged[staged.index(lines[hold_11_6["line"] - 1], staged.index('"11_6")?;')) :]
check(tail[: tail.index('say(&kit, stage, "11_7")')].count("\n") == 1, "nothing stands between 11_6's reading window and 11_7's say")
check("with squashed on screen and nothing drawn or cleared" in hold_11_6["basis"], "the basis still describes it")
contract = units_after["direction_contract"]
check(contract["map"]["path"] == NEW_MAP and contract["map"]["sha256"] == sha(STAGED_MAP) and contract["script_sha256"] == sha(ROOT + "script.tsv"), "the plan binds the candidate map and root's script")
spec = importlib.util.spec_from_file_location("coherent_narration", "tools/coherent_narration.py")
coherent = importlib.util.module_from_spec(spec)
sys.path.insert(0, "tools")
sys.dont_write_bytecode = True
spec.loader.exec_module(coherent)
from pathlib import Path

real_path = coherent.direction_map_path
coherent.direction_map_path = lambda value, topic: Path(STAGED_MAP) if value == NEW_MAP else real_path(value, topic)
resolved = coherent.direction_contract(ROOT + "script.tsv", units_after)
coherent.direction_map_path = real_path
check(len(resolved["resolved"]) == len(direction["overrides"]), f"tools/coherent_narration.py direction_contract accepts the rebound plan, with the map path pointed here: {len(resolved['resolved'])} directions resolved")
for bad_plan, what in (({**units_after, "script_sha256": "0" * 64, "direction_contract": {**contract, "script_sha256": "0" * 64}}, "a wrong script hash"),):
    coherent.direction_map_path = lambda value, topic: Path(STAGED_MAP) if value == NEW_MAP else real_path(value, topic)
    try:
        coherent.direction_contract(ROOT + "script.tsv", bad_plan)
        caught = False
    except ValueError:
        caught = True
    coherent.direction_map_path = real_path
    check(caught, f"control: the real function refuses {what}")
print(f"     visual_holds relocated: " + ", ".join(f"{h['cue']} line {h['line']}" for h in rows))

print("== the card's reading window, as mod.rs's reading_windows_leave_time_for_what_the_voice_leaves_out computes it")
card = re.search(r'pub const SQUASHED: &str = "([^"]*)";', staged).group(1)
clean_s = 0.6 - 0.4 + 7.0
rule = len(card) / 15.0
check(rule <= clean_s <= 2 * rule, f"{len(card)} characters: floor {rule:.2f} s <= {clean_s:.1f} s clean <= ceiling {2 * rule:.2f} s")
check(len(card) <= 92, f"one line on the lower third, {len(card)} of 92 characters")

print("== widths, in the film's own faces, in native pixels at 1280 x 720")


def width(text, pt):
    font = ImageFont.truetype(CM + "cmunrm.ttf", 1000)
    total = 0.0
    for part in re.split("([−≠])", text):
        total += 777.8 if part in ("−", "≠") else font.getlength(part)
    return total / 1000 * pt * 0.9


claim = re.search(r'pub const ROOTS_CLAIM: &str = "([^"]*)";', staged).group(1)
every = re.search(r'pub const EVERY_OUTPUT: &str = "([^"]*)";', staged).group(1)
kept = re.search(r'pub const ZERO_KEPT: &str = "([^"]*)";', staged).group(1)
same = re.search(r'pub const SAME_ROOTS: &str = "([^"]*)";', staged).group(1)
for name, text, pt, centre, low, high in (
    ("claim", claim, 30, 329.5, 40, 707 - 40),
    ("every output", every, 24, 932.5, 707, 1157),
    ("zero kept", kept, 24, 932.5, 707, 1157),
    ("same roots card", same, 24, 329.5, 78, 582),
):
    w = width(text, pt) + (45 if name == "same roots card" else 0)
    check(centre - w / 2 > low and centre + w / 2 < high, f"{name}: {w:.0f} px wide, x {centre - w / 2:.0f} to {centre + w / 2:.0f}, inside {low} to {high}")
print("     the rows' widths are from the desk composites' hand setting: 270 and 384 px against the field's 504; fmn's TeX did not run")
line = width(card, 26)
check(168 + line < 1244, f"the card's text ends near x {168 + line:.0f}, its band ends at 1244")

print("== glyphs: every new text character is already drawn by a string in the registry's lists")
others_text = "".join(read(p) for p in glob.glob(CHAPTERS + "*.rs") if not p.endswith("counting.rs")) + read(ROOT + "counting.rs")
for name, text in (("EVERY_OUTPUT", every), ("ZERO_KEPT", kept), ("SAME_ROOTS", same), ("SQUASHED", card)):
    missing = sorted(set(text) - set(others_text))
    check(not missing, f"{name}: {missing or 'none missing'}")
check(re.search(r'KEPT: &str = "derivative ≠ 0', read(CHAPTERS + "one_point.rs")) is not None, "≠ is in one_point::KEPT, and ÷ in TEXT[0] and one_derivative::GAP, both drawn by kit.text")
tex_elsewhere = read("explainer_corpus/eigen_engineering/src/chapters/can_it_feed_itself.rs")
check(r"\div" in tex_elsewhere, r"\div is typeset by another crate on this pin (eigen_engineering), though not yet by this film; the probe page will show it")
before_tex = re.search(r'pub const BEFORE_DIVIDING_TEX: &str = r"([^"]*)";', staged).group(1)
check("2{i}" in before_tex and before_tex.count("2i") == 1, "fmd_math::find_occurrences is plain text matching, so the key finds only the leading 2i")

print("== beside the glyph patch, in either order")
results, mods = [], []
for order, sequence in (("glyph-first", [GLYPH, P + "counting-from-root.patch", P + "mod-registry.patch"]), ("repair-first", [P + "counting-from-root.patch", P + "mod-registry.patch", GLYPH])):
    folder = os.path.join(work, order)
    os.makedirs(os.path.join(folder, CHAPTERS))
    for name in os.listdir(CHAPTERS):
        shutil.copy(CHAPTERS + name, os.path.join(folder, CHAPTERS, name))
    shutil.copy(ROOT + "counting.rs", os.path.join(folder, COUNTING))
    clean_apply = True
    for patch in sequence:
        done = apply(folder, patch, "--forward")
        clean_apply = clean_apply and done.returncode == 0
        if done.returncode != 0:
            print("     " + done.stdout[:300])
    results.append(sha(os.path.join(folder, COUNTING)))
    mods.append(sha(os.path.join(folder, MOD)))
    check(clean_apply, f"{order}: every hunk applies")
check(results[0] == results[1] and mods[0] == mods[1], "both orders give the same counting.rs and mod.rs")
print(f"     with the glyph patch too: counting.rs {results[0]}, mod.rs {mods[0]}")
glyphed = read(os.path.join(work, "glyph-first", COUNTING)).split("\n")
glyphed_mod = read(os.path.join(work, "glyph-first", MOD)).split("\n")
check(all(glyphed[h["line"] - 1].strip() == h["marker"] for h in rows), "the glyph patch moves no counting hold: only the map's sha256 fields would change")
check(all(glyphed_mod[h["line"] - 1].strip() == h["marker"] for h in direction["helpers"] if h["path"].endswith("/mod.rs")), "and no mod.rs helper row")

print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
