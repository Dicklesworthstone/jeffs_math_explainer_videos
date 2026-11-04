"""Run this follow-up's checks and write checks/outputs. No Cargo, no render, no audio is produced.

python3 -I -B <this file> WORK_DIR   (from the repo root; WORK_DIR must be outside the repo and must not exist)

Writes only to this folder's checks/outputs and to WORK_DIR. RubyRaven's replay.py and group_and_boxes.py are run from
their own frozen folder as they stand; with -I -B they leave nothing there. Stops at the first failing check.
"""
import difflib
import hashlib
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
FOLDER = HERE.parent
REPO = Path.cwd().resolve()
BASE = FOLDER.relative_to(REPO).parent
CRATE = "explainer_corpus/abstract_algebra_expanded"
RS = f"{CRATE}/src/arithmetic.rs"
STAGE = Path("narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/arithmetic-repair-stage01/candidate")
ABC = STAGE / RS
ABCD = FOLDER.relative_to(REPO) / "candidate-ABCD" / RS
ABCD_ROOT = FOLDER.relative_to(REPO) / "candidate-ABCD"
AB = BASE / "algebra-picture-review-13_arithmetic/candidate-AB" / RS
BASELINE = BASE / "algebra-picture-review-13_arithmetic/baseline"
THEIRS = BASE / "algebra13-patch-cross-review01/checks"
OUT = HERE / "outputs"


def run(name, args, work):
    done = subprocess.run([sys.executable, "-I", "-B", *map(str, args)], capture_output=True, text=True)
    text = done.stdout + (f"\n[stderr]\n{done.stderr}" if done.stderr else "")
    text = text.replace(str(work), "WORK").replace(str(REPO) + "/", "")
    (OUT / name).write_text(text + f"\n[exit {done.returncode}]\n", encoding="utf-8")
    print(f"{name}: exit {done.returncode}, {len(text.splitlines())} lines")
    if done.returncode:
        raise SystemExit(f"{name} failed")
    return text


def compare(name, old_name, old, new):
    lines = difflib.unified_diff(old.splitlines(), new.splitlines(), old_name, name, lineterm="", n=0)
    (OUT / f"{name}.diff").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main():
    work = Path(sys.argv[1]).resolve()
    if work.exists():
        raise SystemExit("refusing: WORK_DIR exists")
    work.mkdir(parents=True)
    OUT.mkdir(exist_ok=True)
    run("word_times.out", [HERE / "word_times.py"], work)
    run("speech_end.out", [HERE / "speech_end.py"], work)
    run("timeline.out", [HERE / "timeline.py", f"AB:13_8={AB}", f"ABC={ABC}", f"ABCD={ABCD}"], work)
    tiles = [f"{CRATE}/src/d4_group.rs", f"{CRATE}/src/tile_model.rs"]
    run("pose_mirror-ABC.out", [HERE / "pose_mirror.py", ABC, *tiles], work)
    run("pose_mirror-ABCD.out", [HERE / "pose_mirror.py", ABCD, *tiles], work)
    run("source_checks.out", [HERE / "source_checks.py", ".", ABC, AB, ABCD, FOLDER / "patches/D-followup.patch", work / "source"], work)
    replay = run("replay-ABCD.out", [THEIRS / "replay.py", ABCD, "ABCD"], work)
    compare("replay-ABCD.out", "replay-ABC.out", (THEIRS / "outputs/replay-ABC.out").read_text(encoding="utf-8"), replay)
    group = run("group_and_boxes-ABCD.out", [THEIRS / "group_and_boxes.py", ABCD_ROOT, BASELINE], work)
    compare("group_and_boxes-ABCD.out", "group_and_boxes-ABC.out",
            (THEIRS / "outputs/group_and_boxes-ABC.out").read_text(encoding="utf-8"), group)
    rows = []
    for rel in (RS, f"{CRATE}/oracle.py", f"{CRATE}/script.tsv", f"{CRATE}/src/main.rs", f"{CRATE}/src/d4_group.rs", f"{CRATE}/src/tile_model.rs"):
        rows.append(f"{hashlib.sha256(Path(rel).read_bytes()).hexdigest()}  {rel}")
    (OUT / "canonical-read.out").write_text("Read only, when these checks ran.\n" + "\n".join(rows) + "\n", encoding="utf-8")
    print("all checks ran")


if __name__ == "__main__":
    main()
