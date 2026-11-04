"""Patch round trips, the phrase test's Python mirror, and what the proposal leaves alone.

Run from the repo root: python3 -I -B <this file> EMPTY_DIR
EMPTY_DIR must be a new empty directory outside any git repository. Only it is
written. This doesn't compile anything; root's build is the real test.
"""
import hashlib
import re
import shutil
import subprocess
import sys
from pathlib import Path

O = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
CANDIDATE_DIR = O / "algebra-chapter10-picture-repair02"
REVIEW = O / "algebra-chapter10-repair02-review"
CANONICAL = Path("explainer_corpus/abstract_algebra_expanded/src/cyclic.rs")
SCRIPT = Path("explainer_corpus/abstract_algebra_expanded/script.tsv")
LABEL = "explainer_corpus/abstract_algebra_expanded/src/cyclic.rs"
BEFORE_SHA = "946f1da44bd20e674320a26c3b69876092e7f5a46679565151893645f538b05e"
CANDIDATE_SHA = "105f5cf8be10bcc25579a57912fcc8d1a08eaad275a855854b418c9c7bcf0d0c"

scratch = Path(sys.argv[1]).resolve()
if not scratch.is_dir() or any(scratch.iterdir()):
    raise SystemExit("give a new empty directory")

failures = []


def check(ok, text):
    print(("ok   " if ok else "FAIL ") + text)
    if not ok:
        failures.append(text)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def git(tree, *args):
    return subprocess.run(["git", *args], cwd=tree, capture_output=True, text=True).returncode


check(sha(CANDIDATE_DIR / "before-cyclic.rs") == BEFORE_SHA, "before-cyclic.rs is 946f1da4")
check(sha(CANDIDATE_DIR / "cyclic.rs") == CANDIDATE_SHA, "candidate cyclic.rs is 105f5cf8")
print(f"info canonical cyclic.rs is {sha(CANONICAL)[:8]} (baseline {BEFORE_SHA[:8]} when reviewed)")


def tree(name, source):
    root = scratch / name
    (root / LABEL).parent.mkdir(parents=True)
    shutil.copy(source, root / LABEL)
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    return root


t = tree("root-patch", CANDIDATE_DIR / "before-cyclic.rs")
patch = (CANDIDATE_DIR / "cyclic.patch").resolve()
check(git(t, "apply", str(patch)) == 0 and sha(t / LABEL) == CANDIDATE_SHA, "root's cyclic.patch takes before-cyclic.rs to the candidate, byte for byte")

t = tree("proposed", CANDIDATE_DIR / "cyclic.rs")
patch = (REVIEW / "proposed.patch").resolve()
check(git(t, "apply", str(patch)) == 0 and sha(t / LABEL) == sha(REVIEW / "proposed-cyclic.rs"), "proposed.patch takes the candidate to proposed-cyclic.rs, byte for byte")
check(git(t, "apply", "-R", str(patch)) == 0 and sha(t / LABEL) == CANDIDATE_SHA, "and reverses to the candidate")

r = subprocess.run(["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", str(REVIEW / "proposed-cyclic.rs")], capture_output=True, text=True)
check(r.returncode == 0, "rustfmt --check on proposed-cyclic.rs (syntax and format only, not compilation)")

proposed = (REVIEW / "proposed-cyclic.rs").read_text(encoding="utf-8")
changed = [l for l in (REVIEW / "proposed.patch").read_text(encoding="utf-8").splitlines() if l[:1] in "+-" and l[:3] not in ("+++", "---")]
check(not any("k.say(" in l for l in changed), "no changed line is a k.say")
print(f"info script.tsv sha256 {sha(SCRIPT)}: not in the patch, so no spoken word changes")

# Python mirror of the test every_waited_phrase_is_in_its_sentence.
lines = {l.split("\t")[0]: l.split("\t", 1)[1] for l in SCRIPT.read_text(encoding="utf-8").splitlines() if l and not l.startswith("#")}


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start : i + 1].strip())
            start = i + 1
    return out


consts = dict(re.findall(r'const ([A-Z_]+_PHRASE): &str = "([^"]+)";', proposed))
size = int(re.search(r"const PHRASES: \[\(&str, usize, &str\); (\d+)\]", proposed).group(1))
block = proposed[proposed.index("const PHRASES:") :]
block = block[: block.index("];")]
rows = re.findall(r'\("(\d+_\d+)", (\d+), ([A-Z_]+_PHRASE)\)', block)
check(len(rows) == size == 12, f"PHRASES declares {size} rows and lists {len(rows)}")
check(set(r[2] for r in rows) == set(consts), "every phrase constant has a row and every row a constant")
for cue, j, name in rows:
    said = sentences(lines[cue])
    check(int(j) < len(said) and consts[name] in said[int(j)], f"{cue} sentence {j} contains \"{consts[name]}\"")
counts = re.findall(r'\("(10_\d+)", (\d+)\),', proposed[proposed.index("for (cue, sentences) in") :])
for cue, count in counts:
    check(len(sentences(lines[cue])) == int(count), f"{cue} has {count} sentences")
check(("10_11", "4") in counts, "the test now counts 10_11")

# Hooks in the source for the changed cues stay inside each cue's sentences and go forward.
for cue in ("10_7", "10_10", "10_11"):
    start = proposed.index(f'k.say(stage, "{cue}")')
    end = proposed.find("k.say(stage,", start + 10)
    body = proposed[start : end if end > 0 else proposed.index("k.finish(stage)")]
    hooks = [int(x) for x in re.findall(rf'"{cue}", (\d+)', body)]
    check(hooks == sorted(hooks) and max(hooks) < len(sentences(lines[cue])), f"{cue} hooks in source order, inside the cue: {hooks}")

# No absolute home paths in this bundle. The pattern is split so this file doesn't match itself.
home = "/" + "Users/"
for path in sorted(REVIEW.rglob("*")):
    if path.is_file() and path.name != "bundle.sha256":
        check(home not in path.read_text(encoding="utf-8"), f"no absolute home path in {path.relative_to(REVIEW)}")

print()
print("PATCH CHECK OK" if not failures else f"FAILURES: {len(failures)}")
raise SystemExit(1 if failures else 0)
