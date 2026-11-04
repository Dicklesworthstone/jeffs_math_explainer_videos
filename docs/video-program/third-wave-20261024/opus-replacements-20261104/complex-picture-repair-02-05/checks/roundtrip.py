"""Do the patches apply, alone and together, and does the stack equal candidate/?

    python3 -I -B checks/roundtrip.py SCRATCH_DIR

Works only in SCRATCH_DIR (an empty folder outside the repo). Uses `git apply` with
--unsafe-paths off, in a throwaway git repository, on byte copies of baseline/. Each
shared patch is tried on the baseline alone and on the chapter layer: a patch that
needs the chapter layer says so. Exit 1 if the stack does not reproduce candidate/.
"""
import filecmp
import pathlib
import shutil
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
STAGE = HERE.parent
PATCHES = STAGE / "patches"
CRATE = "explainer_corpus/complex_analysis_visual"
scratch = pathlib.Path(sys.argv[1]).resolve()
failures = []


def fresh(name):
    d = scratch / name
    if d.exists():
        raise SystemExit(f"{d} already exists: give an empty scratch folder")
    shutil.copytree(STAGE / "baseline", d)
    subprocess.run(["git", "init", "-q", str(d)], check=True)
    return d


def apply(d, patch):
    r = subprocess.run(
        ["git", "-C", str(d), "apply", "--check", str(PATCHES / patch)], capture_output=True, text=True
    )
    if r.returncode:
        return False, r.stderr.strip().splitlines()[:2]
    subprocess.run(["git", "-C", str(d), "apply", str(PATCHES / patch)], check=True)
    return True, []


CHAPTERS = "chapters-02-05.patch"
P1, P2, P3 = "shared-P1-crisp-strips.patch", "shared-P2-half-step-bands.patch", "shared-P3-optional-four-samples.patch"

print("-- the stack, in order, against candidate/")
d = fresh("stack")
for patch in (CHAPTERS, P1, P2):
    ok, err = apply(d, patch)
    print(f"  {'ok  ' if ok else 'FAIL'} {patch} {err if err else ''}")
    if not ok:
        failures.append(patch)
files = sorted(p.relative_to(STAGE / "candidate") for p in (STAGE / "candidate").rglob("*") if p.is_file())
same = [f for f in files if filecmp.cmp(d / f, STAGE / "candidate" / f, shallow=False)]
print(f"  {'ok  ' if len(same) == len(files) else 'FAIL'} chapters + P1 + P2 equals candidate/: {len(same)} of {len(files)} files")
if len(same) != len(files):
    failures.append("stack differs from candidate")
ok, err = apply(d, P3)
print(f"  {'ok  ' if ok else 'FAIL'} {P3} on top {err if err else ''}")
if not ok:
    failures.append(P3)

print("-- each piece on its own, on the baseline, as root may want it")
for patch in (CHAPTERS, P1, P2, P3):
    d = fresh("alone-" + patch.split(".")[0])
    ok, err = apply(d, patch)
    print(f"  {'applies' if ok else 'does not apply alone'}: {patch} {err if err else ''}")
    if patch in (CHAPTERS,) and not ok:
        failures.append(patch + " alone")

print("-- P1 and P3 on the chapter layer without P2, and P2 without P1")
for name, chain in (("chapters+P1", (CHAPTERS, P1)), ("chapters+P2", (CHAPTERS, P2)), ("chapters+P3", (CHAPTERS, P3))):
    d = fresh("chain-" + name.replace("+", "-"))
    results = [apply(d, p) for p in chain]
    print(f"  {'applies' if all(r[0] for r in results) else 'does not apply'}: {name} {[r[1] for r in results if not r[0]] or ''}")

print(f"\n{len(failures)} failed" + (": " + "; ".join(failures) if failures else ""))
sys.exit(1 if failures else 0)
