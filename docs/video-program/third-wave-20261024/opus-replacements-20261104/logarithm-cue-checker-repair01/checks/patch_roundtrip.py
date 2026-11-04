#!/usr/bin/env python3
"""The patches, applied and reversed on scratch copies. Then the patched checker on stage04.

    python3 -I -B <this file> EMPTY_SCRATCH_DIR

Run from the repo root. Writes only inside EMPTY_SCRATCH_DIR, which must not be inside a git
repository, or git apply would read paths against it. The canonical crate and stage04 are read
only.
"""

import filecmp
import shutil
import subprocess
import sys
from pathlib import Path

REL = "explainer_corpus/logarithm_mysteries"
STAGE = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage04") / REL
CANONICAL = Path(REL)
HERE = Path(__file__).resolve().parent.parent
CHECKER_PATCH = (HERE / "patches/01-check-cues.patch").resolve()
STORY_PATCH = (HERE / "patches/02-storyboard.patch").resolve()
BASES = ("docs/check_cues.py", "docs/storyboard.md")
CANDIDATES = (*BASES, "docs/test_check_cues.py")


def copy_bases(source, root):
    for rel in BASES:
        path = root / REL / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source / rel, path)


def tool(root, *argv):
    done = subprocess.run(argv, cwd=root, capture_output=True, text=True)
    return done.returncode, (done.stdout + done.stderr).strip().replace(str(root), "SCRATCH")


def main(argv):
    scratch = Path(argv[0]).resolve()
    if not scratch.is_dir() or any(scratch.iterdir()):
        raise SystemExit("scratch must be an existing empty directory")
    if tool(scratch, "git", "rev-parse", "--is-inside-work-tree")[0] == 0:
        raise SystemExit("scratch is inside a git repository")
    problems = []

    def need(ok, what):
        print(f"{'ok ' if ok else 'BAD'} {what}")
        if not ok:
            problems.append(what)

    def matches(root):
        return all(filecmp.cmp(root / REL / rel, HERE / REL / rel, shallow=False) for rel in CANDIDATES)

    # git apply, on stage04's bytes, forward and back.
    root = scratch / "git"
    copy_bases(STAGE, root)
    need(tool(root, "git", "apply", str(CHECKER_PATCH))[0] == 0, "git apply: the checker patch applies to stage04's bytes")
    need(tool(root, "git", "apply", str(STORY_PATCH))[0] == 0, "git apply: the storyboard patch applies to stage04's bytes")
    need(matches(root), "the result is byte for byte the candidates")
    need(tool(root, "git", "apply", "-R", str(STORY_PATCH))[0] == 0 and tool(root, "git", "apply", "-R", str(CHECKER_PATCH))[0] == 0, "git apply -R reverses both")
    need(all(filecmp.cmp(root / REL / rel, STAGE / rel, shallow=False) for rel in BASES), "the reversal is byte for byte stage04")
    need(not (root / REL / "docs/test_check_cues.py").exists(), "the reversal removes the new test file from the scratch copy")

    # patch(1), the other common tool.
    root = scratch / "patch"
    copy_bases(STAGE, root)
    need(tool(root, "/usr/bin/patch", "-p1", "-s", "-i", str(CHECKER_PATCH))[0] == 0 and tool(root, "/usr/bin/patch", "-p1", "-s", "-i", str(STORY_PATCH))[0] == 0, "patch -p1 applies both")
    need(matches(root), "the result is byte for byte the candidates")

    # Canonical, read only: root's call what to do with this.
    root = scratch / "canonical"
    copy_bases(CANONICAL, root)
    print()
    print("On a scratch copy of canonical's current bytes, for information:")
    code, out = tool(root, "git", "apply", "--check", str(CHECKER_PATCH))
    print(f"  checker patch: {'applies' if code == 0 else 'does not apply: ' + out[:300]}")
    code, out = tool(root, "git", "apply", "--check", str(STORY_PATCH))
    print(f"  storyboard patch: {'applies' if code == 0 else 'does not apply: ' + out[:300]}")
    print()

    # The patched checker, in a whole stage04 copy, from where it would sit.
    crate = scratch / "stage04-patched"
    (crate / "docs").mkdir(parents=True)
    (crate / "src").mkdir()
    shutil.copyfile(STAGE / "script.tsv", crate / "script.tsv")
    shutil.copytree(STAGE / "src/chapters", crate / "src/chapters")
    for rel in BASES:
        shutil.copyfile(STAGE / rel, crate / rel)
    target = scratch / "applied"
    target.mkdir()
    copy_bases(STAGE, target)
    tool(target, "git", "apply", str(CHECKER_PATCH))
    for rel in CANDIDATES[0], CANDIDATES[2]:
        shutil.copyfile(target / REL / rel, crate / rel)
    code, out = tool(crate, sys.executable, "-I", "-B", "docs/check_cues.py")
    lines = out.splitlines()
    need(code == 0 and lines[0] == "80 cues, 364 sentences, 80 said" and lines[-1] == "OK", f"the patched checker, in a stage04 copy: exit {code}; {lines[0]}; last line {lines[-1]!r}")
    empty = scratch / "fixtures"
    empty.mkdir()
    code, out = tool(crate, sys.executable, "-I", "-B", "docs/test_check_cues.py", str(empty))
    need(code == 0, f"the patched regression file, from the same place: exit {code}; {out.splitlines()[-1]}")

    print()
    print(f"{len(problems)} problems")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
