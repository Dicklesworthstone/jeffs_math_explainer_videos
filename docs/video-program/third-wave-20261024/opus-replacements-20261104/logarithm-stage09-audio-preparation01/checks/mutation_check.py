#!/usr/bin/env python3
"""Does run_checks.py notice when a stage09 guard is removed? Run from the repository root:

    python3 -I -B <bundle>/checks/mutation_check.py FRESH_SCRATCH_FOLDER > transcript.txt

For each mutant, a mutation root is built under FRESH_SCRATCH_FOLDER: symlinks to the repository's
narration_lab, tools and crates folders and to the program folder's other bundles, and this bundle
file by file as symlinks, except prepare_audio.py, which is a mutated copy. run_checks.py then runs
from that root into its own fresh scratch folder. A mutant is killed when run_checks reports a
FAIL. M0 is the unmutated helper, which must pass everything. Nothing is written to the repository.
"""

import os
import re
import subprocess
import sys

REPO = os.getcwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HERE = f"{P}/logarithm-stage09-audio-preparation01"
HELPER = f"{HERE}/prepare_audio.py"
LINKED = ("narration_lab", "tools", "crates")

MUTANTS = [
    ("M0", "unmutated", None, None),
    ("M1", "a bare file name shared with a superseded recording is accepted",
     "elif name in old and name in by_name:", "elif False:"),
    ("M2", "question and point ids use the bare file name for every recording",
     "label = rel if name in old_names else name", "label = name"),
    ("M3", "receipt_hash_of is ignored (every inventory read as hashing its downloads receipts)",
     'hashed = spec.get("receipt_hash_of", spec["take_receipt"]).format(takes_dir=spec["takes_dir"], cue=t["cue"])', "hashed = receipt"),
    ("M4", "a downloads receipt neither hashed nor pinned is accepted",
     'if hashed != receipt and b["pins"].get(receipt) is None:', "if False:"),
    ("M5", "requests are not checked against their cue and inventoried text",
     'if spec.get("request"):', "if False:"),
    ("M6", "a superseded recording named by its path is reported as merely unknown",
     "stale = name in old_rel", "stale = False"),
    ("M7", "a recording is selectable whatever text it records",
     'if t["cue"] in script and t["text_sha256"] == sha_bytes(script[t["cue"]].encode()):', 'if t["cue"] in script:'),
    ("M8", "a superseded recording named by its path can be selected",
     't = inputs["takes"].get(name)', 't = inputs["takes"].get(name) or inputs["superseded"].get(name)'),
]


def mutation_root(root, helper_text):
    os.makedirs(os.path.join(root, P))
    for name in LINKED:
        os.symlink(os.path.join(REPO, name), os.path.join(root, name))
    for name in sorted(os.listdir(os.path.join(REPO, P))):
        if name != os.path.basename(HERE):
            os.symlink(os.path.join(REPO, P, name), os.path.join(root, P, name))
    for dirpath, _dirs, files in os.walk(HERE):
        os.makedirs(os.path.join(root, dirpath), exist_ok=True)
        for f in files:
            rel = os.path.join(dirpath, f)
            if rel == HELPER:
                with open(os.path.join(root, rel), "x", encoding="utf8") as out:
                    out.write(helper_text)
            else:
                os.symlink(os.path.join(REPO, rel), os.path.join(root, rel))


def main():
    if len(sys.argv) != 2 or os.path.exists(sys.argv[1]) or not os.path.isfile(HELPER):
        print("usage: from the repository root, mutation_check.py FRESH_SCRATCH_FOLDER (must not exist)", file=sys.stderr)
        return 2
    scratch = os.path.realpath(os.path.dirname(os.path.abspath(sys.argv[1]))) + os.sep + os.path.basename(sys.argv[1])
    os.makedirs(scratch)
    original = open(HELPER, encoding="utf8").read()
    results = []
    for tag, what, old, new in MUTANTS:
        if old is not None and original.count(old) != 1:
            print(f"{tag} NOT APPLIED: the target text occurs {original.count(old)} times")
            results.append(False)
            continue
        root = os.path.join(scratch, f"{tag}-root")
        mutation_root(root, original if old is None else original.replace(old, new))
        r = subprocess.run([sys.executable, "-I", "-B", os.path.join(root, HERE, "checks", "run_checks.py"), os.path.join(scratch, f"{tag}-run")],
                           cwd=root, capture_output=True, text=True)
        fails = [m.group(1) for m in re.finditer(r"^FAIL (\S+)", r.stdout, re.M)]
        summary = next((x for x in r.stdout.splitlines() if x.endswith(" FAIL")), "no summary line")
        crashed = "Traceback" in r.stderr
        if old is None:
            ok = r.returncode == 0 and not fails and not crashed
            print(f"{'PASS' if ok else 'FAIL'} {tag} {what}: exit {r.returncode}, {summary}")
        else:
            ok = r.returncode != 0 and (fails or crashed)
            print(f"{'KILLED' if ok else 'SURVIVED'} {tag} {what}: exit {r.returncode}, {summary}; failing checks: "
                  f"{' '.join(sorted(set(fails))) or 'none'}" + ("; run_checks raised" if crashed else ""))
        results.append(bool(ok))
    print(f"{sum(results)} of {len(results)} as expected (M0 passes, every mutant killed)")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
