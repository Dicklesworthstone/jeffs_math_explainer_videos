"""Replay patches/ on a fresh copy of base/ and compare with candidate/, then re-run the light source checks.

Run from the repo root with a new, empty folder outside the repo:
    python3 -I -B <this file> <empty folder>
Writes nothing inside the repo. Prints paths relative to the repo root only. This proves that the patches apply and
that the files parse and format. It compiles nothing, runs no Rust test and renders nothing.
"""

import hashlib
import os
import shutil
import subprocess
import sys

REVIEW = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-preview01-review-11-15/"
CRATE = "explainer_corpus/complex_analysis_visual/"
PATCHES = {
    "11-counting.patch": ["src/chapters/counting.rs"],
    "12-residues.patch": ["src/chapters/residues.rs"],
    "13-real-integral.patch": ["src/chapters/real_integral.rs"],
    "14-continuing.patch": ["src/chapters/continuing.rs"],
    "15-closing.patch": ["src/chapters/closing.rs"],
    "shared-chain-arrows.patch": ["src/chapters/draw.rs"],
    "shared-facts.patch": ["oracle.py", "src/facts.rs", "src/model.rs"],
    "shared-registry.patch": ["src/chapters/mod.rs"],
}


def sha(path):
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def main():
    replay = os.path.join(sys.argv[1], "")
    assert not os.listdir(replay), "the replay folder must be empty"
    failed = False
    for names in PATCHES.values():
        for name in names:
            target = replay + CRATE + name
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copyfile(REVIEW + "base/" + CRATE + name, target)
    root = os.getcwd() + "/" + REVIEW + "patches/"
    print("patches applied one by one to a fresh copy of base/")
    for patch in PATCHES:
        run = subprocess.run(["git", "apply", "--unsafe-paths", root + patch], cwd=replay, capture_output=True, text=True)
        print(f"  {patch}: git apply exit {run.returncode}  sha256 {sha(REVIEW + 'patches/' + patch)}")
        failed |= run.returncode != 0
    print("file by file: base against the canonical file now, replay against candidate/")
    for names in PATCHES.values():
        for name in names:
            base_same = sha(REVIEW + "base/" + CRATE + name) == sha(CRATE + name)
            replay_same = sha(replay + CRATE + name) == sha(REVIEW + "candidate/" + CRATE + name)
            print(f"  {name}: base=canonical {base_same}; replay=candidate {replay_same}; candidate {sha(REVIEW + 'candidate/' + CRATE + name)}")
            failed |= not (base_same and replay_same)
    print("rustfmt --check, edition 2024, repo rustfmt.toml; parse and format only")
    for name in sorted(n for names in PATCHES.values() for n in names if n.endswith(".rs")):
        path = REVIEW + "candidate/" + CRATE + name
        if name.endswith("/mod.rs"):
            # by stdin: rustfmt would otherwise follow `mod` lines into chapters this folder does not hold
            with open(path, "rb") as handle:
                run = subprocess.run(["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml"], stdin=handle, capture_output=True)
        else:
            run = subprocess.run(["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", path], capture_output=True)
        print(f"  {name}: exit {run.returncode}")
        failed |= run.returncode != 0
    print("oracle.py: rows printed by canonical and by candidate, and the rows only the candidate has")
    rows = {}
    for label, path in (("canonical", CRATE + "oracle.py"), ("candidate", REVIEW + "candidate/" + CRATE + "oracle.py")):
        run = subprocess.run([sys.executable, "-I", "-B", path], capture_output=True, text=True)
        rows[label] = [line for line in run.stdout.splitlines() if "\t" in line]
        print(f"  {label}: exit {run.returncode}, {len(rows[label])} rows")
        failed |= run.returncode != 0
    for line in rows["candidate"]:
        if line not in rows["canonical"]:
            print("  new: " + line.replace("\t", " = "))
    gone = [line for line in rows["canonical"] if line not in rows["candidate"]]
    print(f"  canonical rows changed or gone: {len(gone)}")
    failed |= bool(gone)
    print("FAILED" if failed else "all of the above passed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
