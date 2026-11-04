#!/usr/bin/env python3
"""Run stage04's own docs/check_cues.py twice: on stage04 as it is, and on registered scene code.

    python3 -I -B <this file> EMPTY_SCRATCH_DIR

Run from the repo root. The checker globs every src/chapters/*.rs and reads each file whole, so
it also reads retired chapter files and the strings inside test modules. The second run gives it
a scratch copy with only the registered chapters, cut at their tests module. Stage04 is not
changed; nothing here touches anything outside EMPTY_SCRATCH_DIR.
"""
import re
import shutil
import subprocess
import sys
from pathlib import Path

REL = "explainer_corpus/logarithm_mysteries"
STAGE = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage04") / REL
CHECK = "docs/check_cues.py"


def run(crate):
    done = subprocess.run([sys.executable, "-I", "-B", str(crate / CHECK)], capture_output=True, text=True)
    lines = (done.stdout + done.stderr).splitlines()
    failures = lines[lines.index("FAIL") + 1 :] if "FAIL" in lines else []
    return done.returncode, lines[0], [f.strip() for f in failures], lines[-1]


def main(argv):
    scratch = Path(argv[0])
    if not scratch.is_dir() or any(scratch.iterdir()):
        raise SystemExit("scratch must be an existing empty directory")
    chapters = STAGE / "src/chapters"
    mod = (chapters / "mod.rs").read_text(encoding="utf-8")
    registered = [f"{m}.rs" for m in re.findall(r"make: \|kit\| Box::new\((\w+)::", mod)]
    retired = sorted(p.name for p in chapters.glob("*.rs") if p.name not in registered + ["mod.rs", "draw.rs", "thumbnail.rs"])

    code, first, failures, _ = run(STAGE)
    print(f"stage04 as it is: exit {code}; {first}; {len(failures)} failures")
    from_retired = [f for f in failures if any(name in f for name in retired)]
    rest = [f for f in failures if f not in from_retired]
    print(f"  {len(from_retired)} name a retired, unregistered file: {retired}")
    print(f"  {len(rest)} others:")
    for f in rest:
        print(f"    {f}")

    crate = scratch / REL
    (crate / "docs").mkdir(parents=True)
    (crate / "src/chapters").mkdir(parents=True)
    shutil.copyfile(STAGE / "script.tsv", crate / "script.tsv")
    shutil.copyfile(STAGE / CHECK, crate / CHECK)
    shutil.copyfile(chapters / "mod.rs", crate / "src/chapters/mod.rs")
    for name in registered:
        text = (chapters / name).read_text(encoding="utf-8")
        at = text.find("#[cfg(test)]\nmod tests")
        (crate / "src/chapters" / name).write_text(text if at < 0 else text[:at], encoding="utf-8")
    code2, first2, failures2, last2 = run(crate)
    print(f"registered scene code only: exit {code2}; {first2}; {len(failures2)} failures; last line {last2!r}")
    for f in failures2:
        print(f"    {f}")
    if code2:
        print("A REAL FAILURE: the checker still fails on registered scene code")
        return 1
    print("So the failures above come from the retired files and from strings in test modules. Source text only; nothing compiled.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
