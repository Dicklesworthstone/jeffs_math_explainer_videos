"""Run the crate's own docs/check_cues.py on scratch copies of the baseline and patched source, and compare.

python3 -I -B cue_check.py REPO_ROOT PATCHED_FILE WORK_DIR

WORK_DIR must not exist. check_cues.py finds its crate from its own path, so each copy is checked by itself and nothing
under REPO_ROOT is read by it or written. It reads text: no compile, no scene run.
"""
import difflib
import shutil
import subprocess
import sys
from pathlib import Path

CRATE = "explainer_corpus/logarithm_mysteries"


def copy(repo, work, name, patched=None):
    top = work / name
    shutil.copytree(repo / CRATE / "src", top / "src")
    (top / "docs").mkdir()
    shutil.copyfile(repo / CRATE / "docs/check_cues.py", top / "docs/check_cues.py")
    shutil.copyfile(repo / CRATE / "script.tsv", top / "script.tsv")
    if patched:
        shutil.copyfile(patched, top / "src/chapters/unreachable.rs")
    return top


def check(top):
    done = subprocess.run(["python3", "-I", "-B", "docs/check_cues.py"], cwd=top, capture_output=True, text=True)
    return done.returncode, (done.stdout + done.stderr).replace(str(top) + "/", "").splitlines()


def main():
    repo, patched, work = (Path(p).resolve() for p in sys.argv[1:4])
    if work.exists():
        raise SystemExit("refusing: WORK_DIR exists")
    work.mkdir(parents=True)
    base_code, base = check(copy(repo, work, "baseline"))
    new_code, new = check(copy(repo, work, "patched", patched))
    print(f"baseline: exit {base_code}, {len(base)} lines of output")
    print(f"patched: exit {new_code}, {len(new)} lines of output")
    print("output differences, baseline to patched:")
    delta = [ln for ln in difflib.unified_diff(base, new, lineterm="", n=0) if not ln.startswith(("---", "+++", "@@"))]
    for ln in delta:
        print("  ", ln[:170])
    print("  ", len(delta), "differing lines")
    print("last lines of the patched output:")
    for ln in new[-4:]:
        print("  ", ln[:170])
    # Control: a say removed from the chapter must fail, or a pass means nothing.
    top = copy(repo, work, "control", patched)
    path = top / "src/chapters/unreachable.rs"
    text = path.read_text(encoding="utf-8")
    needle = 'kit.say(stage, "11_6")?;'
    if text.count(needle) != 1:
        raise SystemExit("control could not find its line")
    path.write_text(text.replace(needle, ""), encoding="utf-8")
    code, out = check(top)
    print(f"control, 11_6's say removed: exit {code} (expect 1)")
    if base_code or new_code or code != 1:
        raise SystemExit("a check failed")
    print("cue check holds on both, and the control fails")


if __name__ == "__main__":
    main()
