"""Check the staged patch without building it.

Run from the repo root: python3 -I -B <this file> EMPTY_SCRATCH_DIR
Writes only inside EMPTY_SCRATCH_DIR, which must exist and be empty.

It checks text, not behaviour. Nothing here compiles the Rust or renders a frame:
- the baseline is the reviewed source, and the patch applies and reverses byte-exactly
- the proposed file passes rustfmt with the repo's config
- script.tsv is byte-identical to the one the frames were rendered from, and every
  k.say line of the proposed file matches the baseline in order
- every until_sentence and until_phrase index exists in its cue, hooks inside one cue
  run in source order, and every waited phrase sits in its sentence
- no handle faded out by a list is faded again by a later list in the patched regions
  that were changed (a text scan with the limits stated at that step)
- no absolute home path in any bundle file
"""
import hashlib
import re
import subprocess
import sys
from pathlib import Path

R = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra-picture-review-17_applications")
A = Path("explainer_corpus/abstract_algebra_expanded")
LABEL = "explainer_corpus/abstract_algebra_expanded/src/applications.rs"
BASE = "7cc3f0d08b61087206144a6f187b2ce1a21059632c4aec8aa95a1a9a46f90bef"
SCRIPT = "83619d48676b32d5d93bf7527477aa752a902dd16927c548e4b3052cc2d689c0"

scratch = Path(sys.argv[1]).resolve()
assert scratch.is_dir() and not any(scratch.iterdir()), "scratch must be an empty directory"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(cmd, cwd=None):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)


before = R / "before-applications.rs"
proposed = R / "proposed-applications.rs"
patch = (R / "proposed.patch").resolve()
assert sha(before) == BASE
print("baseline is the reviewed source", BASE[:12])
print("canonical source now:", "same as baseline" if sha(A / "src/applications.rs") == BASE else "CHANGED SINCE REVIEW")
assert sha(A / "script.tsv") == SCRIPT
print("script.tsv is the rendered one", SCRIPT[:12])

tree = scratch / "tree"
target = tree / LABEL
target.parent.mkdir(parents=True)
assert run(["git", "init", "-q"], cwd=tree).returncode == 0
target.write_bytes(before.read_bytes())
applied = run(["git", "apply", "--whitespace=nowarn", str(patch)], cwd=tree)
assert applied.returncode == 0, applied.stderr
assert target.read_bytes() == proposed.read_bytes()
print("git apply forward: byte-exact")
reversed_ = run(["git", "apply", "-R", "--whitespace=nowarn", str(patch)], cwd=tree)
assert reversed_.returncode == 0, reversed_.stderr
assert target.read_bytes() == before.read_bytes()
print("git apply -R: back to the baseline, byte-exact")

fmt = run(["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", str(proposed)])
assert fmt.returncode == 0, fmt.stdout + fmt.stderr
print("rustfmt --check: clean")

old_text = before.read_text(encoding="utf-8")
new_text = proposed.read_text(encoding="utf-8")
says = lambda t: re.findall(r"k\.say\(stage, \"(17_\d+)\"\)", t)
assert says(old_text) == says(new_text)
print("k.say order unchanged:", " ".join(says(new_text)))


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start : i + 1].strip())
            start = i + 1
    return out


lines = {}
for raw in (A / "script.tsv").read_text(encoding="utf-8").splitlines():
    if raw.startswith("17_"):
        cue, text = raw.split("\t", 1)
        lines[cue] = sentences(text)

constants = dict(re.findall(r'const (\w+_PHRASE): &str = "([^"]+)";', new_text))
assert set(constants) == {"CONTRIBUTIONS_PHRASE", "SWAP_CHANGE_PHRASE"}

hooks = re.compile(
    r'k\.say\(stage, "(17_\d+)"\)'
    r'|until_sentence\(stage, "(17_\d+)", ([^)]+)\)'
    r'|until_phrase\(stage, &k, "(17_\d+)", (\d+), (\w+)\)'
)
# The two hooks with computed indices, with the indices they take.
COMPUTED = {("17_7", "j"): [2, 3, 4], ("17_12", "i + 1"): [1, 2, 3]}
last = {}
count = 0
for m in hooks.finditer(new_text):
    if m.group(1):
        last[m.group(1)] = -1
        continue
    cue = m.group(2) or m.group(4)
    if m.group(2):
        raw = m.group(3)
        indices = [int(raw)] if raw.isdigit() else COMPUTED[(cue, raw)]
    else:
        indices = [int(m.group(5))]
        said = lines[cue][indices[0]]
        phrase = constants[m.group(6)]
        assert phrase in said, (cue, phrase)
    for j in indices:
        assert 0 <= j < len(lines[cue]), (cue, j)
        assert j >= last[cue], f"{cue}: sentence {j} after {last[cue]}"
        last[cue] = j
        count += 1
print(f"{count} sentence and phrase waits: every index exists, every cue runs in order, every phrase is in its sentence")

# Scan for the F1 failure: a handle in a wipe list that an earlier fade already removed.
# Names in one-line fade(.., false) calls and in vec!/extend wipe lists of the changed
# film; a regex over names, so a name that is reassigned counts as a new object.
removed = {}
hits = []
for n, line in enumerate(new_text.splitlines(), 1):
    for name in list(removed):
        if re.search(rf"\b(let (mut )?{name}\b|{name} = )", line):
            removed.pop(name)
    m = re.search(r"fade(?:_for)?\(stage, &\[([^\]]+)\], false", line)
    if not m:
        continue
    for name in re.split(r",\s*", m.group(1)):
        if name in removed:
            hits.append((n, name, removed[name]))
        removed[name] = n
print("names faded twice with no reassignment between:", hits or "none")
assert all(name in ("digits[3]", "products[3]") for _, name, _ in hits)

home = "/" + "Users/"
for path in sorted(R.rglob("*")):
    if path.is_file() and path.suffix in {".md", ".py", ".rs", ".patch", ".tsv", ".txt", ".sha256"}:
        assert home not in path.read_text(encoding="utf-8"), path
print("no absolute home path in the bundle's text files")
print("ALL CHECKS PASS (text only: uncompiled, unrendered)")
