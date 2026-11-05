"""Re-run this folder's checks and save their output. No Cargo, no render, no job. Reads root's lab folders only.

    python3 -B .../checks/run_checks.py NEW_SCRATCH SESSION_ID     (from the repo root)

Builds a fresh symlink view under NEW_SCRATCH, then re-measures it and requires the same numbers as the first run saved in
measure_lab.out. Saved outputs have the home, repo and scratch paths replaced.
"""
import pathlib
import re
import subprocess
import sys

CHECKS = pathlib.Path(__file__).resolve().parent
PY = [sys.executable, "-B"]


def main(scratch, session):
    scratch = pathlib.Path(scratch)
    scratch.mkdir(parents=True, exist_ok=False)
    view = scratch / "view"
    failed = []

    def clean(text):
        for old, new in ((str(scratch), "<scratch>"), (str(pathlib.Path.home()), "<home>"), (str(pathlib.Path.cwd()), "<repo>")):
            text = text.replace(old, new)
        return text

    def run(name, *args, save=True):
        r = subprocess.run(PY + [str(CHECKS / f"{name}.py"), *map(str, args)], capture_output=True, text=True)
        if save:
            (CHECKS / f"{name}.out").write_text(clean(r.stdout + r.stderr))
        print(f"{'ok  ' if r.returncode == 0 else 'FAIL'} {name} (exit {r.returncode})")
        if r.returncode:
            failed.append(name)
        return r

    run("consolidate", view)
    # the frozen measure again, compared with the first run on every line but its own name
    r = subprocess.run(PY + [str(CHECKS.parents[1] / "complex-color-repair03/checks/measure_lab.py"), str(view)], capture_output=True, text=True)
    def numbers(text):
        out = text[: text.index("CONTROLS")]
        out = re.sub(r"(?m)^sheet .*\n", "", out)
        return out.splitlines()[1:]
    same = numbers(r.stdout) == numbers((CHECKS / "measure_lab.out").read_text())
    print(f"{'ok  ' if same and r.returncode == 0 else 'FAIL'} measure_lab again, same numbers as the saved run: {same}")
    if not same or r.returncode:
        failed.append("measure_lab")
    run("repaint_geometry", view)
    run("tables", view)
    run("measure_probe", "--self-test", scratch / "probe-selftest")
    run("evidence_table")
    run("viewing_proof", session)
    print("ALL OK" if not failed else f"FAILED: {failed}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], sys.argv[2]))
