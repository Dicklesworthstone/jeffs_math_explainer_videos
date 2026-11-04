"""Source-level checks on the staged repair. Standard library plus the `patch` command.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex06-09-repair03/checks/source_checks.py

It proves what a text comparison can: the baselines, the patches' round trip, no timed call
changed, no on-screen string or cue word changed, the question windows' source unchanged, and
the patches' fit beside the glyph patch. It proves nothing about the pictures. Temporary
copies go in a new temporary directory.
"""
import hashlib
import os
import re
import shutil
import subprocess
import tempfile

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex06-09-repair03/"
CHAPTERS_DIR = "explainer_corpus/complex_analysis_visual/src/chapters/"
STAGED = HERE + CHAPTERS_DIR
FILES = {"06": "one_derivative", "07": "one_point", "08": "loop_sums", "09": "boundary"}
PATCHES = {"06": "06-one-derivative.patch", "07": "07-one-point.patch", "08": "08-loop-sums.patch", "09": "09-boundary.patch"}
GLYPH = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-glyph-spacing-repair01/complex-glyph-spacing.patch"

# calls that spend film time; their arguments are kept, except a fade's list of mobs
TIMED = [
    "fade_in_all", "fade_out_all", "stage.play", "stage.play_prepared", "stage.wait", "wait_until",
    "line.wait_sentence", "line.wait_for", "line.wait_last", "line.until", "kit.hold", "kit.music_breath",
    "kit.listen", "kit.finish", "reading_window", "predict", "precisely_now", "precisely_over", "names_now",
    "lay_down", "lay_in_step", "pause", "product_diagram", "sample_round", "sample_round_hidden",
    "shift_all", "say",
]
# from a question's cue through the first answer's sentence; the source must not change
WINDOWS = [
    ("06", '"06_8")?;', 'say(&kit, stage, "06_9")?;', "line.wait_sentence(stage, 1)?;"),
    ("07", '"07_3")?;', 'say(&kit, stage, "07_4")?;', "line.wait_sentence(stage, 1)?;"),
    ("09", '"09_4")?;', 'say(&kit, stage, "09_5")?;', "line.wait_sentence(stage, 1)?;"),
    ("09", '"09_8")?;', 'say(&kit, stage, "09_9")?;', "kit.finish(stage)"),
]

fails = 0


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def read(path):
    return open(path, encoding="utf-8").read()


def scene(text):
    """The film code: everything before the test module, with comments removed."""
    code = text[: text.index("#[cfg(test)]\nmod tests")] if "#[cfg(test)]\nmod tests" in text else text
    out, i, in_string = [], 0, False
    while i < len(code):
        c = code[i]
        if in_string:
            out.append(c)
            if c == "\\" and raw_depth == 0:
                out.append(code[i + 1])
                i += 1
            elif c == '"':
                in_string = False
            i += 1
        elif c == '"':
            raw_depth = 1 if i > 0 and code[i - 1] == "r" else 0
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
    """Every timed call in film order, with whitespace gone and a fade's mob list reduced to its time."""
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


def apply(folder, patch):
    return subprocess.run(["patch", "-p1", "--no-backup-if-mismatch", "-i", os.path.abspath(patch)], cwd=folder, capture_output=True, text=True, check=False)


print("== baselines still match the tree the repair was cut from")
for line in read(HERE + "checks/baseline.sha256").splitlines():
    digest, path = line.split("  ")
    check(sha(path) == digest, f"{path.rsplit('/', 1)[-1]} {digest[:16]}")

print("== scope: exactly the four chapter files are staged")
staged_files = sorted(n for _, _, names in os.walk(HERE + "explainer_corpus") for n in names)
check(staged_files == sorted(n + ".rs" for n in FILES.values()), str(staged_files))

work = tempfile.mkdtemp(prefix="complex0609-source-")
print("== each patch applies to its baseline, equals the staged file, and reverses")
for key, name in FILES.items():
    folder = os.path.join(work, "round", key)
    os.makedirs(os.path.join(folder, CHAPTERS_DIR))
    target = os.path.join(folder, CHAPTERS_DIR, name + ".rs")
    shutil.copy(CHAPTERS_DIR + name + ".rs", target)
    done = apply(folder, HERE + "patches/" + PATCHES[key])
    check(done.returncode == 0 and "offset" not in done.stdout and "fuzz" not in done.stdout, f"{PATCHES[key]} applies clean")
    check(sha(target) == sha(STAGED + name + ".rs"), f"{name}.rs equals staged {sha(target)[:16]}")
    back = subprocess.run(["patch", "-R", "-p1", "--no-backup-if-mismatch", "-i", os.path.abspath(HERE + "patches/" + PATCHES[key])], cwd=folder, capture_output=True, text=True, check=False)
    check(back.returncode == 0 and sha(target) == sha(CHAPTERS_DIR + name + ".rs"), f"{PATCHES[key]} reverses to baseline")

print("== no timed call changed: the source-bound hold and phrase delta is 0 at every call")
for key, name in FILES.items():
    a, b = timeline(read(CHAPTERS_DIR + name + ".rs")), timeline(read(STAGED + name + ".rs"))
    check(a == b, f"{name}: {len(a)} timed calls, identical in order and arguments")
    if a != b:
        for i, (x, y) in enumerate(zip(a, b)):
            if x != y:
                print(f"     first difference at {i}: {x} / {y}")
                break
control = read(STAGED + "one_derivative.rs").replace("fade_out_all(stage, &reflection, 0.3)?;", "fade_out_all(stage, &reflection, 0.4)?;", 1)
check(timeline(control) != timeline(read(STAGED + "one_derivative.rs")), "control: a changed fade time is detected")
control = read(STAGED + "loop_sums.rs").replace("line.wait_sentence(stage, 2)?;\n        let mut gone = product.clone();", "line.wait_sentence(stage, 2)?;\n        stage.wait(0.5)?;\n        let mut gone = product.clone();", 1)
check(timeline(control) != timeline(read(STAGED + "loop_sums.rs")), "control: an added wait is detected")

print("== no on-screen string changed, and the script is untouched")
for key, name in FILES.items():
    a, b = strings(read(CHAPTERS_DIR + name + ".rs")), strings(read(STAGED + name + ".rs"))
    check(a == b, f"{name}: {len(a)} string literals in the film code, the same")
check(sha("explainer_corpus/complex_analysis_visual/script.tsv").startswith("2968b0fd76540cbc"), "script.tsv 2968b0fd76540cbc")

print("== no test removed")
for key, name in FILES.items():
    a, b = tests(read(CHAPTERS_DIR + name + ".rs")), tests(read(STAGED + name + ".rs"))
    check(set(a) <= set(b), f"{name}: {len(a)} kept, new {sorted(set(b) - set(a))}")

print("== question windows: source unchanged from the question's cue through the first answer")
for key, question, answer, first in WINDOWS:
    spans = []
    for path in (CHAPTERS_DIR, STAGED):
        text = scene(read(path + FILES[key] + ".rs"))
        start = text.index(question)
        at = text.index(answer, start)
        spans.append(text[start : text.index(first, at) + len(first)])
    check(spans[0] == spans[1], f"{question[1:5]} to {answer.split(chr(34))[1]}: {len(spans[0])} characters, identical")

print("== beside the glyph patch, in either order, with no line of it lost")
results = []
for order in ("glyph-first", "repair-first"):
    folder = os.path.join(work, order)
    shutil.copytree(CHAPTERS_DIR, os.path.join(folder, CHAPTERS_DIR))
    sequence = [GLYPH] + [HERE + "patches/" + p for p in PATCHES.values()]
    if order == "repair-first":
        sequence = sequence[1:] + sequence[:1]
    clean = True
    for patch in sequence:
        done = apply(folder, patch)
        clean = clean and done.returncode == 0
    results.append(tuple(sha(os.path.join(folder, CHAPTERS_DIR, n + ".rs")) for n in FILES.values()))
    check(clean, f"{order}: every hunk applies")
check(results[0] == results[1], "both orders give the same four files")

print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
