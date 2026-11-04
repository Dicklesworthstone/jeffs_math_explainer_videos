"""Source-only invariants of this repair. Run from the repo root:

    python3 -I -B <this folder>/checks/invariants.py

Compares baseline/ with candidate/ and with the working tree. It reads text and
runs nothing: it cannot show that the Rust compiles, or how a frame looks.
"""
import hashlib
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent.parent
CRATE = "explainer_corpus/logarithm_mysteries"
FILES = ["complex_turns", "lifting_loops", "branch_cut"]
EXPECT = {
    "complex_turns": "d1f2d72555ee7f10",
    "lifting_loops": "b409ace64e13fafb",
    "branch_cut": "c3fd774e9172cdbe",
}
SCRIPT = "3e4e34af7090e448"
ANCHOR = re.compile(r'kit\.(say|until_sentence|until_share|hold)\(\s*stage(?:,\s*"([0-9_]+)"(?:,\s*(\d+))?)?')
PREDICT = re.compile(r"\bpredict\(")
THINK = re.compile(r"THINK")
HOME = re.compile(r"/(?:Users|home)/[A-Za-z0-9_.-]+/")


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def scene(text):
    """The scene code, without the tests."""
    return text.split("#[cfg(test)]")[0]


failures = []


def check(ok, what):
    print(("ok   " if ok else "FAIL ") + what)
    if not ok:
        failures.append(what)


repo = pathlib.Path(".")
check(sha(repo / CRATE / "script.tsv").startswith(SCRIPT), f"script.tsv is still {SCRIPT}")
for name in FILES:
    rel = f"{CRATE}/src/chapters/{name}.rs"
    base, cand = HERE / "baseline" / rel, HERE / "candidate" / rel
    check(sha(base).startswith(EXPECT[name]), f"{name}: baseline hash {EXPECT[name]}")
    base_s, cand_s = scene(base.read_text()), scene(cand.read_text())
    a, b = ANCHOR.findall(base_s), ANCHOR.findall(cand_s)
    check(a == b, f"{name}: {len(b)} say/until_sentence/hold anchors unchanged, in order")
    a, b = len(PREDICT.findall(base_s)), len(PREDICT.findall(cand_s))
    check(a == b, f"{name}: {b} predict call(s) unchanged")
    check(not HOME.search(cand.read_text()), f"{name}: no absolute home path")
    plays = cand_s.count("play!(")
    print(f"     {name}: {plays} play! calls, {cand_s.count('fade_in_all(')} fade_in_all, {cand_s.count('swap(')} swap")

live = {
    name: sha(repo / CRATE / f"src/chapters/{name}.rs").startswith(EXPECT[name])
    for name in FILES
}
print("working tree still equals baseline:", live)

# the words on screen that the voice depends on stay in the source
spoken = {
    "complex_turns": ['"05_1"', '"05_5"'],
    "lifting_loops": ['"06_2"', '"06_4"'],
    "branch_cut": ['"07_3"'],
}
for name, cues in spoken.items():
    text = (HERE / "candidate" / f"{CRATE}/src/chapters/{name}.rs").read_text()
    for cue in cues:
        check(cue in text, f"{name}: prediction cue {cue} still said")

print("RESULT:", "FAIL" if failures else "PASS")
sys.exit(1 if failures else 0)
