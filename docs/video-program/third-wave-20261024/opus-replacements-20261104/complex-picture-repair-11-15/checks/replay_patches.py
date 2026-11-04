"""Replay patches/ on a fresh copy of baseline/ and compare with candidate/, then re-run the light source checks.

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

STAGE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-picture-repair-11-15/"
LANES = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
CRATE = "explainer_corpus/complex_analysis_visual/"
PATCHES = {
    "shared-facts.patch": ["oracle.py", "src/facts.rs", "src/model.rs"],
    "11-counting.patch": ["src/chapters/counting.rs"],
    "12-residues.patch": ["src/chapters/residues.rs"],
    "13-real-integral.patch": ["src/chapters/real_integral.rs"],
    "14-continuing.patch": ["src/chapters/continuing.rs"],
    "15-closing.patch": ["src/chapters/closing.rs"],
    "16-storyboard.patch": ["docs/storyboard.md"],
    "shared-registry.patch": ["src/chapters/mod.rs"],
    "shared-chain-arrows-fallback.patch": ["src/chapters/draw.rs"],
}
FALLBACK = "shared-chain-arrows-fallback.patch"
# the other lanes' patches, in the order root might stack them
OTHER_LANES = [
    "complex-picture-repair-02-05/patches/chapters-02-05.patch",
    "complex-picture-repair-02-05/patches/shared-P1-crisp-strips.patch",
    "complex-picture-repair-02-05/patches/shared-P2-half-step-bands.patch",
    "complex-picture-repair-02-05/patches/shared-P3-optional-four-samples.patch",
    "complex-picture-repair-06-10/patches/00-draw-helpers.patch",
    "complex-picture-repair-06-10/patches/06-one-derivative.patch",
    "complex-picture-repair-06-10/patches/07-one-point.patch",
    "complex-picture-repair-06-10/patches/08-loop-sums.patch",
    "complex-picture-repair-06-10/patches/09-boundary.patch",
    "complex-picture-repair-06-10/patches/10-rigidity.patch",
    "complex-picture-repair-06-10/patches/11-storyboard.patch",
    "complex-picture-repair-06-10/patches/12-mod-lists-06.patch",
    "complex-picture-repair-06-10/patches/12-mod-lists-07.patch",
    "complex-picture-repair-06-10/patches/12-mod-lists-08.patch",
    "complex-picture-repair-06-10/patches/12-mod-lists-09.patch",
    "complex-picture-repair-06-10/patches/12-mod-lists-10.patch",
]


def sha(path):
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def fresh(folder):
    for names in PATCHES.values():
        for name in names:
            target = os.path.join(folder, CRATE + name)
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copyfile(STAGE + "baseline/" + CRATE + name, target)


def apply(folder, path):
    return subprocess.run(["git", "apply", "--unsafe-paths", path], cwd=folder, capture_output=True, text=True)


def main():
    root = os.getcwd() + "/"
    assert not os.listdir(sys.argv[1]), "the replay folder must be empty"
    replay = os.path.join(sys.argv[1], "alone", "")
    os.makedirs(replay)
    failed = False
    fresh(replay)
    print("patches applied one by one to a fresh copy of baseline/")
    for patch in PATCHES:
        run = apply(replay, root + STAGE + "patches/" + patch)
        print(f"  {patch}: git apply exit {run.returncode}  sha256 {sha(STAGE + 'patches/' + patch)}")
        failed |= run.returncode != 0
    print("file by file: baseline against the canonical file now, replay against candidate/")
    for names in PATCHES.values():
        for name in names:
            base_same = sha(STAGE + "baseline/" + CRATE + name) == sha(CRATE + name)
            replay_same = sha(replay + CRATE + name) == sha(STAGE + "candidate/" + CRATE + name)
            print(f"  {name}: baseline=canonical {base_same}; replay=candidate {replay_same}; candidate {sha(STAGE + 'candidate/' + CRATE + name)[:16]}")
            failed |= not (base_same and replay_same)
    print("script.tsv: the candidate carries no copy; canonical sha256 " + sha(CRATE + "script.tsv")[:16])

    print("stacked on the other lanes' patches to the same files; the 06-10 helper owns chain_arrows here")
    both = os.path.join(sys.argv[1], "both", "")
    os.makedirs(both)
    fresh(both)
    # files only those patches change, copied from canonical so they can apply
    for entry in sorted(os.listdir(CRATE + "src/chapters")):
        target = both + CRATE + "src/chapters/" + entry
        if entry.endswith(".rs") and not os.path.exists(target):
            shutil.copyfile(CRATE + "src/chapters/" + entry, target)
    for patch in OTHER_LANES:
        run = apply(both, root + LANES + patch)
        print(f"  theirs {patch}: git apply exit {run.returncode}" + (" " + run.stderr.strip()[:200] if run.returncode else ""))
        # a conflict between two other lanes is root's to know about, but it is not this lane's failure
    for patch in PATCHES:
        if patch == FALLBACK:
            continue
        run = apply(both, root + STAGE + "patches/" + patch)
        print(f"  mine {patch}: git apply exit {run.returncode}" + (" " + run.stderr.strip()[:150] if run.returncode else ""))
        failed |= run.returncode != 0
    draw = open(both + CRATE + "src/chapters/draw.rs", encoding="utf-8").read()
    has = "Arrow::new(a, b).thickness(3.0).color(color).buff(0.0).build()?" in draw
    print(f"  chain_arrows has buff(0.0) with their patch alone: {has}")
    failed |= not has
    print("the registry's duplicate-value test, by checks/registry_check.py, over all lanes stacked")
    run = subprocess.run([sys.executable, "-I", "-B", STAGE + "checks/registry_check.py", both + CRATE + "src/chapters"], capture_output=True, text=True)
    alone = subprocess.run([sys.executable, "-I", "-B", STAGE + "checks/registry_check.py"], capture_output=True, text=True)
    mine = [line for line in alone.stdout.splitlines() if line.startswith("  + ")]
    for line in run.stdout.splitlines():
        if "UNRESOLVED" in line or "unresolved" in line:
            print("  " + line)
            failed |= "UNRESOLVED" in line
        elif "DUPLICATE" in line:
            value = line.split("DUPLICATE ", 1)[1]
            ours = any(("= " + value) in m for m in mine)
            print("  " + line + ("  from this lane" if ours else "  NOT from this lane: tell root and the two owners"))
            failed |= ours

    print("rustfmt --check, edition 2024, repo rustfmt.toml; parse and format only")
    for name in sorted(n for names in PATCHES.values() for n in names if n.endswith(".rs")):
        path = STAGE + "candidate/" + CRATE + name
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
    for label, path in (("canonical", CRATE + "oracle.py"), ("candidate", STAGE + "candidate/" + CRATE + "oracle.py")):
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
