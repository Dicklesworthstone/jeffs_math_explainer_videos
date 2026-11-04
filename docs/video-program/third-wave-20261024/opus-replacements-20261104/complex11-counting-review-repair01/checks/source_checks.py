"""Source-level checks on root's counting candidate and on this folder's patches.

Standard library plus the `patch` and `rustc` commands. Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-review-repair01/checks/source_checks.py

It proves what a text comparison can: the baselines, the patches' round trip, no timed call and no
on-screen string changed by the label move, the sentence anchors, the storyboard and unit bindings,
the reading-window arithmetic, the width estimates and the fit beside the glyph patch. It proves
nothing about pictures or voice. Temporary copies go in a new temporary directory.
"""
import difflib
import hashlib
import os
import re
import shutil
import subprocess
import tempfile

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-review-repair01/"
ROOT = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/counting-wording-repair01/"
CHAPTERS = "explainer_corpus/complex_analysis_visual/src/chapters/"
COUNTING = CHAPTERS + "counting.rs"
BOARD = "explainer_corpus/complex_analysis_visual/docs/storyboard.md"
TOPIC = "explainer_corpus/complex_analysis_visual/"
STAGED = HERE + COUNTING
PATCH1, PATCH2, PATCH3 = HERE + "patches/01-reach-label.patch", HERE + "patches/02-storyboard-11_7.patch", HERE + "patches/03-optional-a-name.patch"
GLYPH = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-glyph-spacing-repair01/complex-glyph-spacing.patch"
FRAMES = "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/11_counting/"
TIMED = [
    "fade_in_all", "fade_out_all", "stage.play", "stage.play_prepared", "stage.wait", "wait_until",
    "line.wait_sentence", "line.wait_for", "line.wait_last", "line.until", "line.sentence", "kit.hold", "kit.music_breath",
    "kit.listen", "kit.finish", "reading_window", "predict", "precisely_now", "names_now", "pause", "say",
]
# TeX cmr10 advance widths in ems, the film's text face
EMS = dict(a=.5, b=.5556, c=.4444, d=.5556, e=.4444, f=.3056, g=.5, h=.5556, i=.2778, j=.3056, k=.5278, l=.2778, m=.8333, n=.5556, o=.5, p=.5556, q=.5278, r=.3917, s=.3944, t=.3889, u=.5556, v=.5278, w=.7222, x=.5278, y=.5278, z=.4444)
EMS.update({" ": .3333, ";": .2778, "L": .625, "×": .7778})

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


def offsets(text):
    """draw.rs's sentence_offsets, again."""
    return [0] + [i + 2 for i in range(len(text) - 2) if text[i] in ".?!" and text[i + 1] == " "]


print("== baselines still match")
for line in read(HERE + "checks/baseline.sha256").splitlines():
    digest, path = line.split("  ")
    check(sha(path) == digest, f"{path.rsplit('/', 2)[-2]}/{path.rsplit('/', 1)[-1]} {digest[:16]}")

print("== scope: one staged source file, and the storyboard as a patch only")
staged_files = sorted(os.path.join(d, n) for d, _, names in os.walk(HERE + "explainer_corpus") for n in names)
check(staged_files == [STAGED], str([f.rsplit("/", 1)[-1] for f in staged_files]))

work = tempfile.mkdtemp(prefix="complex11-source-")
print("== patches apply, equal what they claim, and reverse")
folder = os.path.join(work, "one")
os.makedirs(os.path.join(folder, CHAPTERS))
target = os.path.join(folder, COUNTING)
shutil.copy(ROOT + "counting.rs", target)
done = apply(folder, PATCH1)
check(done.returncode == 0 and "offset" not in done.stdout and "fuzz" not in done.stdout, "01 applies clean to root's candidate")
check(sha(target) == sha(STAGED), f"01 gives the staged file {sha(target)}")
done = apply(folder, PATCH3)
check(done.returncode == 0 and "offset" not in done.stdout and "fuzz" not in done.stdout, "03 applies clean on top of 01")
optional = read(target)
print(f"     01 + 03 = {sha(target)}")
done = apply(folder, PATCH3, "-R")
check(done.returncode == 0 and sha(target) == sha(STAGED), "03 reverses")
done = apply(folder, PATCH1, "-R")
check(done.returncode == 0 and sha(target) == sha(ROOT + "counting.rs"), "01 reverses to root's candidate")
folder = os.path.join(work, "board")
os.makedirs(os.path.join(folder, TOPIC, "docs"))
target = os.path.join(folder, BOARD)
shutil.copy(BOARD, target)
done = apply(folder, PATCH2)
check(done.returncode == 0 and "offset" not in done.stdout and "fuzz" not in done.stdout, "02 applies clean to the canonical storyboard")
patched_board = read(target)
check(cue(ROOT + "script.tsv", "11_7") in patched_board and cue(ROOT + "before-script.tsv", "11_7") not in patched_board, "02 puts root's exact 11_7 in the row and takes the old one out")
print(f"     02 gives storyboard.md {sha(target)}")
done = apply(folder, PATCH2, "-R")
check(done.returncode == 0 and sha(target) == sha(BOARD), "02 reverses")

base, staged = read(ROOT + "counting.rs"), read(STAGED)
print("== the label move: no timed call, no on-screen string, no test lost")
a, b = timeline(base), timeline(staged)
check(a == b, f"{len(a)} timed calls, identical in order and arguments")
check(timeline(staged.replace("fade_in_all(stage, &lit, 0.4)?;", "fade_in_all(stage, &lit, 0.5)?;", 1)) != b, "control: a changed fade time is detected")
check(timeline(staged.replace("fade_in_all(stage, &lit, 0.4)?;", "fade_in_all(stage, &lit, 0.4)?;\n    stage.wait(0.5)?;", 1)) != b, "control: an added wait is detected")
check(strings(base) == strings(staged), f"{len(strings(base))} string literals in the film code, the same")
check(set(tests(base)) <= set(tests(staged)), f"{len(tests(base))} tests kept, new {sorted(set(tests(staged)) - set(tests(base)))}")
changed = [l[2:].strip() for l in difflib.ndiff(scene(base).splitlines(), scene(staged).splitlines()) if l[:2] in ("- ", "+ ") and l[2:].strip()]
expected = [
    "const REACH_NAMES_AT: (f64, f64) = (0.0, 2.2);",
    "let covers = kit.text(stage, REACH_COVERS, 18.0, warn(), out.p(C::new(0.0, 2.2)))?;",
    "let names_at = out.p(C::new(REACH_NAMES_AT.0, REACH_NAMES_AT.1));",
    "let covers = kit.text(stage, REACH_COVERS, 18.0, warn(), names_at)?;",
    "let reach_name = kit.text(",
    "stage,",
    "STEP_SKETCH[2],",
    "18.0,",
    "derivative_teal(),",
    "out.p(C::new(0.95, -1.2)),",
    ")?;",
    "let reach_name = kit.text(stage, STEP_SKETCH[2], 18.0, derivative_teal(), names_at)?;",
]
check(sorted(changed) == sorted(expected), f"film code differs in exactly {len(changed)} lines, the label's")
if sorted(changed) != sorted(expected):
    print("     " + "\n     ".join(changed))

print("== the map's three hold markers keep their lines between root's candidate and the staged file")
marks = [("11_5", "let asked = predict(&kit, stage, &line, &ASK, v(ASK_AT.0, ASK_AT.1))?;", 0, 429), ("11_6", "reading_window(&kit, stage, &line)?;", 0, 492), ("11_8", "reading_window(&kit, stage, &line)?;", 1, 547)]
for id_, marker, which, in_map in marks:
    lines = {}
    for name, text in (("canonical", read(COUNTING)), ("root", base), ("staged", staged)):
        lines[name] = [i + 1 for i, l in enumerate(text.split("\n")) if marker in l][which]
    check(lines["canonical"] == in_map and lines["root"] == lines["staged"], f"{id_} map {in_map}, canonical {lines['canonical']}, root {lines['root']}, staged {lines['staged']}")

print("== root's sentence anchors against root's script")
old, new = cue(ROOT + "before-script.tsv", "11_7"), cue(ROOT + "script.tsv", "11_7")
at = offsets(new)
said = [new[x:] for x in at]
print(f"     11_7: {len(old.split())} words {len(offsets(old))} sentences -> {len(new.split())} words {len(at)} sentences; {len(old)} -> {len(new)} characters")
check(len(at) == 7, "seven sentences by draw.rs's rule")
region = base[base.index('"11_7")?;') : base.index('"11_8")?;')]
for k, opens, follows in [(2, "Now it starts with z to the n", "degree_n"), (3, "On a huge circle", "wide_all"), (4, "Picture z to the n", "dog.show")]:
    tail = region[region.index(f"line.wait_sentence(stage, {k})?;") :]
    check(said[k].startswith(opens) and follows in tail[: tail.index("\n", 120)], f"wait_sentence {k}: {opens!r} brings {follows}")
check(said[6].startswith("So there are exactly n roots") and "line.sentence(6)?" in region, "the walk ends on sentence 6, 'So there are exactly n roots'")
check(not re.search(r"line\.(at|wait_for|until)\(", region), "11_7 uses no phrase anchor, so no phrase mark needs rebinding for it")
check("line.wait_sentence(stage, 1)" not in region and "line.sentence(1)" not in region, "nothing on screen is keyed to sentence 1, 'Divide by its leading coefficient'")
rate = 31.938 / len(old)
print("     character-share estimate at preview02's scratch rate, not Oliver and not measured:")
print(f"     picture holds on the claim until {offsets(old)[1] * rate:.1f} s of the old cue, {at[2] * rate:.1f} s of the new; the cue goes {len(old) * rate:.1f} -> {len(new) * rate:.1f} s")

print("== storyboard: main.rs's test calls check_video on it and script.tsv; the real function, built with rustc alone")
source = read(CHAPTERS.replace("chapters/", "main.rs"))
check("check_video(STORYBOARD, SCRIPT" in source, "main.rs's the_storyboard_the_registry_and_the_script_agree makes that call at cargo test")


def balanced(text, start):
    depth, at = 0, text.index("{", start)
    while True:
        depth += {"{": 1, "}": -1}.get(text[at], 0)
        if text[at] == "}" and depth == 0:
            return text[start : at + 1]
        at += 1


narration = read("crates/explainer_kit/src/narration.rs")
tree = os.path.join(work, "board-test")
os.makedirs(tree)
shutil.copy("crates/explainer_kit/src/storyboard.rs", os.path.join(tree, "storyboard.rs"))
with open(os.path.join(tree, "main.rs"), "w", encoding="utf-8") as out:
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
built = subprocess.run(["rustc", "--edition", "2024", os.path.join(tree, "main.rs"), "-o", os.path.join(tree, "check.bin"), "--cap-lints", "allow"], capture_output=True, text=True, check=False)
check(built.returncode == 0, "the real check_video and Script::parse build" + ("" if built.returncode == 0 else ": " + built.stderr.replace(tree, "<work>")[:800]))


def board_check(board_path, script_path):
    return subprocess.run([os.path.join(tree, "check.bin"), board_path, script_path], capture_output=True, text=True, check=False)


patched_path = os.path.join(tree, "patched-storyboard.md")
with open(patched_path, "w", encoding="utf-8") as out:
    out.write(patched_board)
done = board_check(BOARD, TOPIC + "script.tsv")
check(done.returncode == 0, "control: canonical storyboard and canonical script agree")
done = board_check(BOARD, ROOT + "script.tsv")
check(done.returncode != 0 and "not the storyboard's words" in done.stdout, f"root's script against the canonical storyboard fails, as root's folder stands: {done.stdout.strip()[:60]}...")
done = board_check(patched_path, ROOT + "script.tsv")
check(done.returncode == 0, "root's script against patch 02's storyboard agrees")

print("== narration unit limits: tools/coherent_narration.py allows 100 words, with any following context")
words = {name: {i: len(cue(ROOT + name, i).split()) for i in ("11_6", "11_7", "11_8")} for name in ("before-script.tsv", "script.tsv")}
check(words["script.tsv"]["11_7"] <= 100, f"11_7 alone {words['script.tsv']['11_7']} words, {100 - words['script.tsv']['11_7']} to spare")
check(words["before-script.tsv"]["11_6"] + words["before-script.tsv"]["11_7"] > 100 and words["before-script.tsv"]["11_7"] + words["before-script.tsv"]["11_8"] > 100, "following context around 11_7 was already over the limit, so the longer cue closes no option")
units = read(TOPIC + "narration_units.json")
check(old in units and new not in units, "narration_units.json still holds the old text: root's rebind")

print("== the card's reading window, as mod.rs's reading_windows_leave_time_for_what_the_voice_leaves_out computes it")
card = re.search(r'pub const SQUASHED: &str = "([^"]*)";', base).group(1)
old_card = re.search(r'pub const SQUASHED: &str =\s*"([^"]*)";', read(COUNTING)).group(1)
clean = 0.6 - 0.4 + 7.0
for name, text in (("canonical", old_card), ("root", card), ("suggested", "output " + card)):
    rule = len(text) / 15.0
    check(rule <= clean <= 2 * rule, f"{name}: {len(text)} characters, floor {rule:.2f} s <= {clean:.1f} s clean <= ceiling {2 * rule:.2f} s")
check(clean > 2 * 53 / 15.0, "a card of 53 characters or fewer would fail the ceiling, with the 7 s window kept")

print("== width estimates, in pixels at 1280 x 720, cmr10 ems x size x 0.01 x 90")


def px(text, size):
    return sum(EMS[c] for c in text) * size * 0.9


claim = re.search(r'pub const ROOTS_CLAIM: &str = "([^"]*)";', base).group(1)
measured_old = 320.0  # frame 360: every polynomial has roots, ink x 170 to 489
scale = measured_old / px("every polynomial has roots", 30)
width = px(claim, 30) * scale
left, right = 329.5 - width / 2, 329.5 + width / 2
print(f"     claim: estimate calibrated by {scale:.3f} on frame 360; {width:.0f} px wide, x {left:.0f} to {right:.0f}")
check(left > 40 and right < 707 - 40, "claim clear of the frame's edge and of the output window at x 707")
line = px(card, 26)
check(len(card) <= 92 and 168 + line < 1244, f"card one line ({len(card)} of 92 characters), ending near x {168 + line:.0f}, band ends at 1244")

print("== glyphs: every character of the two new strings is already in a string the film shows")
others = read(CHAPTERS + "real_integral.rs") + read(COUNTING)
for name, text in (("SQUASHED", card), ("ROOTS_CLAIM", claim)):
    missing = sorted(set(text) - set(others))
    check(not missing, f"{name}: {missing or 'none missing'}")
honest = re.search(r'pub const HONEST: &str = "([^"]*)";', read(CHAPTERS + "real_integral.rs")).group(1)
check(";" in honest, "the semicolon appears in real_integral::HONEST, drawn by the same lower third")

print("== beside the glyph patch, in either order")
results = []
for order, sequence in (("glyph-first", [GLYPH, PATCH1, PATCH3]), ("repair-first", [PATCH1, PATCH3, GLYPH])):
    folder = os.path.join(work, order)
    os.makedirs(os.path.join(folder, CHAPTERS))
    shutil.copy(ROOT + "counting.rs", os.path.join(folder, COUNTING))
    # the glyph patch also touches files this folder does not; take them from the tree
    for name in ("mod.rs", "the_bump.rs", "rigidity.rs", "real_integral.rs", "gallery.rs", "boundary.rs", "continuing.rs", "loop_sums.rs", "one_point.rs", "one_derivative.rs", "reading_color.rs", "turning.rs", "two_planes.rs", "residues.rs", "closing.rs"):
        if os.path.exists(CHAPTERS + name):
            shutil.copy(CHAPTERS + name, os.path.join(folder, CHAPTERS, name))
    clean_apply = True
    for patch in sequence:
        done = apply(folder, patch, "--forward")
        clean_apply = clean_apply and done.returncode == 0
        if done.returncode != 0:
            print("     " + done.stdout[:300])
    results.append(sha(os.path.join(folder, COUNTING)))
    check(clean_apply, f"{order}: every hunk applies")
check(results[0] == results[1], "both orders give the same counting.rs")

print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
