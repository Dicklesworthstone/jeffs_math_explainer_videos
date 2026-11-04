"""Syntax-only checks on scratch copies: rustfmt parse and format, with controls, and a patch round trip.

python3 -I -B syntax_check.py REPO_ROOT PATCHED_FILE PATCH_FILE WORK_DIR

WORK_DIR must not exist. Nothing is written under REPO_ROOT. No Cargo, no compile, no type check. rustfmt on main.rs
follows every declared module, so a parse error in the chapter fails the run (the broken control).

rustfmt's exit status cannot be the test: src/score.rs is a canonical byte copy that already draws format diffs on the
baseline. The test is: no parse error, no diff in the chapter, and the same diff list as the baseline.
"""
import filecmp
import shutil
import subprocess
import sys
from pathlib import Path

CRATE = "explainer_corpus/logarithm_mysteries"
REL = "src/chapters/unreachable.rs"


def tree(work, name, repo, edited=None):
    top = work / name
    shutil.copytree(Path(repo) / CRATE / "src", top / "src")
    if edited:
        shutil.copyfile(edited, top / REL)
    return top


def fmt(label, top, config, work):
    done = subprocess.run(
        ["rustfmt", "--check", "--edition", "2024", "--config-path", str(config), str(top / "src/main.rs")],
        capture_output=True, text=True,
    )
    text = (done.stdout + done.stderr).replace(str(top) + "/", "").replace(str(work) + "/", "")
    diffs = [ln for ln in text.splitlines() if ln.startswith("Diff in")]
    errors = [ln for ln in text.splitlines() if ln.startswith("error")]
    print(f"{label}: rustfmt exit {done.returncode}; format diffs {len(diffs)}; parse errors {len(errors)}")
    for ln in diffs[:6] + errors[:3]:
        print("   ", ln)
    return diffs, errors


def main():
    repo, patched, patch, work = (Path(p).resolve() for p in sys.argv[1:5])
    if work.exists():
        raise SystemExit("refusing: WORK_DIR exists")
    work.mkdir(parents=True)
    config = repo / "rustfmt.toml"
    print("rustfmt", subprocess.run(["rustfmt", "--version"], capture_output=True, text=True).stdout.strip())
    print("config", config.name, "=", " ".join(config.read_text().split()))

    base_diffs, base_errors = fmt("baseline", tree(work, "baseline", repo), config, work)
    new_diffs, new_errors = fmt("patched", tree(work, "patched", repo, patched), config, work)
    chapter = "unreachable.rs"
    print("patched has no parse error:", not new_errors)
    print("patched has no diff in the chapter:", not [d for d in new_diffs if chapter in d])
    print("patched diff list equals the baseline's (the score.rs lines only):", new_diffs == base_diffs)

    # Control A: a misformatted neighbour in the same call. Must draw a diff, or the check does not reach this file.
    top = tree(work, "misformatted", repo, patched)
    path = top / REL
    text = path.read_text(encoding="utf-8")
    good = '            "flattening matrices, like zero, have no logarithm",\n'
    if text.count(good) != 1:
        raise SystemExit("control A could not find its line")
    path.write_text(text.replace(good, '  "flattening matrices, like zero, have no logarithm",\n'), encoding="utf-8")
    a_diffs, _ = fmt("control, misindented neighbour (expect a diff in the chapter)", top, config, work)
    control_a = bool([d for d in a_diffs if chapter in d])

    # Control B: a stray brace in the chapter. Must fail, or the run does not parse the chapter.
    top = tree(work, "broken", repo, patched)
    path = top / REL
    path.write_text(path.read_text(encoding="utf-8") + "\n}\n", encoding="utf-8")
    _, b_errors = fmt("control, stray brace (expect a parse error)", top, config, work)
    control_b = bool(b_errors)

    # Round trip: the delivered patch on a fresh copy of the source, compared byte for byte with the staged file.
    top = work / "roundtrip"
    shutil.copytree(repo / CRATE / "src", top / CRATE / "src")
    done = subprocess.run(["git", "init", "-q"], cwd=top, capture_output=True, text=True)
    done = subprocess.run(["git", "apply", "--check", str(patch)], cwd=top, capture_output=True, text=True)
    print("git apply --check on the scratch copy: exit", done.returncode, done.stderr.strip()[:200])
    done = subprocess.run(["git", "apply", str(patch)], cwd=top, capture_output=True, text=True)
    print("git apply on the scratch copy: exit", done.returncode, done.stderr.strip()[:200])
    same = filecmp.cmp(top / CRATE / REL, patched, shallow=False)
    print("round trip equals the staged file:", same)
    done = subprocess.run(["git", "apply", "--check", str(patch)], cwd=repo, capture_output=True, text=True)
    print("git apply --check against the live tree (reads only): exit", done.returncode, done.stderr.strip()[:200])

    if new_errors or [d for d in new_diffs if chapter in d] or new_diffs != base_diffs:
        raise SystemExit("rustfmt check failed")
    if not control_a or not control_b or not same:
        raise SystemExit("a control or round trip failed")
    print("all syntax checks and controls hold")


if __name__ == "__main__":
    main()
