"""Exercise post_install_check.py on scratch copies of stage04. Writes only under FRESH_SCRATCH; reads the repo. Repo root:

    python3 -I -B .../complex-stage04-binder-review01/checks/post_install_selftest.py FRESH_SCRATCH

Each case copies the 55 files stage04's receipt lists into its own folder, changes that copy, and runs the check.
Nothing is deleted: a stray "moved aside" is renamed into a sibling folder of the copy.
"""
import json
import pathlib
import shutil
import subprocess
import sys

REPO = pathlib.Path.cwd()
HERE = pathlib.Path(__file__).resolve().parent
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
S4 = REPO / EC / "combined-repair-stage04"
R4 = REPO / EC / "combined-repair-stage04-integration01.json"
BX = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-stage03-binder01"
CR = "explainer_corpus/complex_analysis_visual"
UNITS4 = BX / "bound-plan-stage04" / CR / "narration_units.json"
MAP4 = BX / "bound-plan-stage04" / CR / "docs/narration-direction-map-oliver12.json"
UNITS3 = BX / "bound-plan-stage03" / CR / "narration_units.json"
STRAYS = ["src/chapters/counting.rs.orig", "src/chapters/mod.rs.orig", "src/chapters/one_derivative.rs.orig",
          ".ee/hook-state/session_start_orient.last.json"]


def copy_stage(dest):
    for f in json.loads(R4.read_text(encoding="utf-8"))["files"]:
        d = dest / f["path"]
        d.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(S4 / f["path"], d)


def install(root, units=UNITS4):
    shutil.copyfile(units, root / CR / "narration_units.json")
    shutil.copyfile(MAP4, root / CR / "docs/narration-direction-map-oliver12.json")


def receipt(root, path, skip=()):
    import hashlib
    files = [{"path": str(p.relative_to(root)), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()}
             for p in sorted(root.rglob("*")) if p.is_file() and str(p.relative_to(root)) not in skip]
    path.write_text(json.dumps({"files": files, "changed": [f"{CR}/narration_units.json", f"{CR}/docs/narration-direction-map-oliver12.json"]},
                               indent=2) + "\n", encoding="utf-8")


def main():
    scratch = pathlib.Path(sys.argv[1])
    assert not scratch.exists(), "fresh scratch please"
    assert "narration_lab" not in scratch.resolve().parts
    cases = []

    def case(name, want, build, with_receipt=True, skip=()):
        root = scratch / name / "stage"
        copy_stage(root)
        build(root)
        args = [str(root)]
        if with_receipt:
            receipt(root, scratch / name / "receipt.json", skip)
            args.append(str(scratch / name / "receipt.json"))
        r = subprocess.run([sys.executable, "-I", "-B", str(HERE / "post_install_check.py"), *args], cwd=REPO, capture_output=True, text=True)
        ok = r.returncode == want and not r.stderr
        fails = [line for line in r.stdout.splitlines() if line.startswith("FAIL")]
        cases.append(ok)
        print(f"{'PASS' if ok else 'FAIL'} {name}: exit {r.returncode}, expected {want}" + (f"; first failure: {fails[0]}" if fails else ""))

    def aside(root):
        for rel in STRAYS:
            dst = root.parent / "moved-aside" / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            (root / CR / rel).rename(dst)

    case("t0-stage04-not-installed", 1, lambda r: None, with_receipt=False)
    case("t1-installed", 0, install)
    case("t2-installed-strays-moved-aside", 0, lambda r: (install(r), aside(r)))
    case("t3-stage03-units-installed", 1, lambda r: install(r, UNITS3))
    case("t4-installed-plus-one-byte-in-the_bump", 1,
         lambda r: (install(r), (r / CR / "src/chapters/the_bump.rs").open("ab").write(b" ")))
    case("t5-installed-receipt-omits-the-map", 1, install, skip=(f"{CR}/docs/narration-direction-map-oliver12.json",))
    case("t6-installed-plus-an-extra-file", 1,
         lambda r: (install(r), shutil.copyfile(r / CR / "src/chapters/closing.rs", r / CR / "src/chapters/closing.rs.bak")))
    print("SELFTEST:", "PASS" if all(cases) else "FAIL", f"({sum(cases)}/{len(cases)})")
    return 0 if all(cases) else 1


if __name__ == "__main__":
    sys.exit(main())
