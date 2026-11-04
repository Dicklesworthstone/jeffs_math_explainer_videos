"""Assemble algebra17-patch-cross-review01 and prove its patches reproduce the candidates.

Run from the repo root with python3 -I -B. Writes only inside the reserved folder and a
throwaway tree in scratch. RubyRaven's bundle and canonical files are only read.
"""
import difflib
import hashlib
import pathlib
import re
import shutil
import subprocess
import sys
import time

SP = pathlib.Path(sys.argv[0]).resolve().parent
REPO = pathlib.Path.cwd()
STAGE = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104"
OUT = STAGE / "algebra17-patch-cross-review01"
THEIRS = STAGE / "algebra-picture-review-17_applications"
REL = "explainer_corpus/abstract_algebra_expanded/src/applications.rs"
SCRIPT = REPO / "explainer_corpus/abstract_algebra_expanded/script.tsv"
log = []


def say(line):
    print(line)
    log.append(line)


def scrub(text):
    return text.replace(str(SP.parent), "<scratch>").replace(str(REPO), "<repo>").replace(str(pathlib.Path.home()), "<home>")


def run(args, **kw):
    return subprocess.run(args, capture_output=True, text=True, **kw)


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


r = run([sys.executable, "-I", "-B", str(SP / "build_corrections.py"), str(SP / "work")])
say("build: " + r.stdout.strip().replace("\n", "; ") + r.stderr.strip())
if r.returncode:
    raise SystemExit(1)

proposed = THEIRS / "proposed-applications.rs"
c1 = SP / "work/C1/applications.rs"
c2 = SP / "work/C1C2/applications.rs"
canonical = REPO / REL
say(f"canonical applications.rs equals RubyRaven's baseline: {sha(canonical) == sha(THEIRS / 'before-applications.rs')} ({sha(canonical)[:16]})")
say(f"script.tsv: {sha(SCRIPT)[:16]}")

for sub in ("base", "candidate-C1", "candidate-C1C2", "patches", "checks/outputs", "coverage"):
    (OUT / sub).mkdir(parents=True, exist_ok=True)
shutil.copyfile(proposed, OUT / "base/proposed-applications.rs")
shutil.copyfile(c1, OUT / "candidate-C1/applications.rs")
shutil.copyfile(c2, OUT / "candidate-C1C2/applications.rs")


def udiff(old, new):
    return "".join(
        difflib.unified_diff(
            old.read_text(encoding="utf-8").splitlines(keepends=True),
            new.read_text(encoding="utf-8").splitlines(keepends=True),
            fromfile=f"a/{REL}",
            tofile=f"b/{REL}",
        )
    )


(OUT / "patches/C1-false-labels-and-guards.patch").write_text(udiff(proposed, c1), encoding="utf-8")
(OUT / "patches/C2-voice-sync.patch").write_text(udiff(c1, c2), encoding="utf-8")

# The whole chain from canonical, forward and back, in a throwaway tree.
check = SP / f"apply-{time.strftime('%H%M%S')}"
target = check / REL
target.parent.mkdir(parents=True)
shutil.copyfile(canonical, target)
assert run(["git", "init", "-q"], cwd=check).returncode == 0
chain = [
    (THEIRS / "proposed.patch", proposed),
    (OUT / "patches/C1-false-labels-and-guards.patch", c1),
    (OUT / "patches/C2-voice-sync.patch", c2),
]
for patch, want in chain:
    r = run(["git", "apply", "--whitespace=nowarn", str(patch)], cwd=check)
    same = target.read_bytes() == want.read_bytes()
    say(f"apply {patch.name}: exit {r.returncode}, equals its candidate: {same} {scrub(r.stderr).strip()}")
    if r.returncode or not same:
        raise SystemExit("chain does not reproduce the candidates")
for patch, want in [(chain[2][0], c1), (chain[1][0], proposed), (chain[0][0], canonical)]:
    r = run(["git", "apply", "-R", "--whitespace=nowarn", str(patch)], cwd=check)
    same = target.read_bytes() == want.read_bytes()
    say(f"reverse {patch.name}: exit {r.returncode}, back to the earlier file: {same}")
    if r.returncode or not same:
        raise SystemExit("chain does not reverse")

# Text checks on the final candidate.
fmt = []
for name in ("candidate-C1", "candidate-C1C2"):
    r = run(["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", str(OUT / name / "applications.rs")])
    fmt.append(f"{name}: rustfmt --check exit {r.returncode}")
    say(fmt[-1])
    if r.returncode:
        print(scrub(r.stdout + r.stderr)[:3000])
(OUT / "checks/outputs/rustfmt.out").write_text("\n".join(fmt) + "\n")

says = lambda t: re.findall(r'k\.say\(stage, "(17_\d+)"\)', t)
base_text, final = proposed.read_text(encoding="utf-8"), c2.read_text(encoding="utf-8")
say(f"k.say source order unchanged: {says(base_text) == says(final)}")


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start : i + 1].strip())
            start = i + 1
    return out


lines = dict(raw.split("\t", 1) for raw in SCRIPT.read_text(encoding="utf-8").splitlines() if raw.startswith("17_"))
constants = dict(re.findall(r'const (\w+_PHRASE): &str = "([^"]+)";', final))
calls = re.findall(r'until_phrase\(stage, &?k, "(17_\d+)", (\d+), (\w+)\)', final)
good = all(constants[c] in sentences(lines[cue])[int(j)] for cue, j, c in calls)
say(f"{len(calls)} until_phrase calls; {len(constants)} phrase constants; each phrase is in its sentence: {good}")
if not good or set(constants) != {c for _, _, c in calls}:
    raise SystemExit("a phrase constant and its calls disagree")

for name in ("oliver_ratio.py", "clock_mirror.py", "model_arithmetic.py", "new_text_extents.py", "run_desk_checks.py", "build_corrections.py", "package.py"):
    shutil.copyfile(SP / name, OUT / "checks" / name)
for name, cmd in (
    ("oliver_ratio", [sys.executable, "-I", "-B", str(SP / "oliver_ratio.py")]),
    ("clock_mirror", [sys.executable, "-I", "-B", str(SP / "clock_mirror.py")]),
    ("model_arithmetic", [sys.executable, "-I", "-B", str(SP / "model_arithmetic.py")]),
    ("new_text_extents", [sys.executable, "-B", "-W", "ignore", str(SP / "new_text_extents.py")]),
    (
        "desk_checks",
        [
            sys.executable, "-I", "-B", str(SP / "run_desk_checks.py"),
            f"proposed={proposed}", f"C1={c1}", f"C1C2={c2}",
        ],
    ),
):
    r = run(cmd)
    (OUT / "checks/outputs" / f"{name}.out").write_text(scrub(r.stdout + r.stderr))
    say(f"{name}: exit {r.returncode}")
    if r.returncode:
        print(scrub(r.stderr)[:2000])

(OUT / "checks/outputs/package.out").write_text(scrub("\n".join(log)) + "\n")

FRAMES = "narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-combined-repair-frames01/17_applications"
VIEWED = [
    ("0156", "F4: Running total 93 on the seventh column while the voice gives the total; the 93 is fading out, not in"),
    ("0185", "F3: 132 + 7 = 139 and the remainder are up 1.4 s into 17_4"),
    ("0229", "F1: clean just before the 17_5 wipe"),
    ("0231", "F1: old 6 and old 42 over the red 7 and 49"),
    ("0244", "F5, F11: argument finished early; the two rows have no labels"),
    ("0358", "F6: 7x6+6x4=66 with no weights drawn; used for the x 7 clearance"),
    ("0411", "F12, F17: full setup and title up early; green 0 close to the x 3 label"),
    ("0608", "F14, F15: stack without a heading; label column without a name"),
    ("0707", "F16: column frames unnamed; basis for the C1a reading of the stack over the miscorrected word"),
    ("0734", "F2: Two flips line beside 0010110, parity box empty"),
    ("0749", "F2, F19, L1: built to keep; line on screen at its lint anchor"),
    ("0779", "F7: recap complete early; no square; used for the caption clearance"),
    ("0830", "F13: all three prompts up at once, centred; used to calibrate text width"),
    ("0899", "F8: hH=H and closure line up before sentence 2"),
]
rows = ["frame\tsha256\tpath\twhat it confirmed"]
for number, what in VIEWED:
    path = f"{FRAMES}/frame-{number}.png"
    rows.append(f"{number}\t{sha(REPO / path)}\t{path}\t{what}")
(OUT / "coverage/frames-viewed.tsv").write_text("\n".join(rows) + "\n", encoding="utf-8")

files = sorted(p for p in OUT.rglob("*") if p.is_file() and p.name != "SHA256SUMS")
(OUT / "SHA256SUMS").write_text("".join(f"{sha(p)}  {p.relative_to(OUT).as_posix()}\n" for p in files))
print("files:", len(files))
for p in files:
    data = p.read_bytes()
    for needle in (str(pathlib.Path.home()).encode(), b"/private" + b"/tmp"):
        if needle in data:
            print("PRIVATE PATH in", p.relative_to(OUT))
print({p.name: sha(p)[:16] for p in (OUT / "patches").iterdir()})
print("C1", sha(c1)[:16], "C1C2", sha(c2)[:16], "SHA256SUMS", sha(OUT / "SHA256SUMS")[:16])
