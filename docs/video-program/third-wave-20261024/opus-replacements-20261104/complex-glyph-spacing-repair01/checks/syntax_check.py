"""Syntax-only checks on scratch copies: rustfmt parse and format, with two controls, and a patch round trip.

python3 -I -B syntax_check.py SRC_ROOT EDITED_ROOT PATCH_FILE WORK_DIR

WORK_DIR must not exist. Nothing is written under SRC_ROOT. No Cargo, no compile, no type check.
rustfmt on main.rs follows every declared module, so a parse error in a chapter fails the run (the broken control).
"""
import filecmp
import shutil
import subprocess
import sys
from pathlib import Path

CRATE = "explainer_corpus/complex_analysis_visual"
CH = CRATE + "/src/chapters/"
FILES = ["closing.rs", "counting.rs", "loop_sums.rs", "mod.rs", "one_derivative.rs", "residues.rs"]


def tree(work, name, src_root, edited_root=None):
    top = work / name
    shutil.copytree(Path(src_root) / CRATE / "src", top / "src")
    if edited_root:
        for f in FILES:
            shutil.copyfile(Path(edited_root) / CH / f, top / "src/chapters" / f)
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
    return done.returncode


def main():
    src, edited, patch, work = sys.argv[1:5]
    work = Path(work).resolve()
    if work.exists():
        raise SystemExit("refusing: WORK_DIR exists")
    work.mkdir(parents=True)
    config = (Path(src) / "rustfmt.toml").resolve()
    print("rustfmt", subprocess.run(["rustfmt", "--version"], capture_output=True, text=True).stdout.strip())
    print("config", config.name, "=", " ".join(config.read_text().split()))

    base = fmt("baseline", tree(work, "baseline", src), config, work)
    new = fmt("patched", tree(work, "patched", src, edited), config, work)

    # Control A: the new literal without the reflow. Must draw a format diff, or the reflow is not needed.
    top = tree(work, "no-reflow", src, edited)
    path = top / "src/chapters/one_derivative.rs"
    text = path.read_text(encoding="utf-8")
    vertical = ('            kit.tex(\n                stage,\n                r"z_{\\,0}^2 = 2i",\n                28.0,\n'
                '                output(),\n                &[],\n                out.p(w0 + C::new(-0.6, 0.3)),\n            )?,\n')
    flat = '            kit.tex(stage, r"z_{\\,0}^2 = 2i", 28.0, output(), &[], out.p(w0 + C::new(-0.6, 0.3)))?,\n'
    if text.count(vertical) != 1:
        raise SystemExit("control A could not find the reflowed call")
    path.write_text(text.replace(vertical, flat), encoding="utf-8")
    control_a = fmt("control, literal without the reflow (expect a diff)", top, config, work)

    # Control B: a stray brace in a nested chapter. Must fail, or the run does not reach the chapters.
    top = tree(work, "broken", src, edited)
    path = top / "src/chapters/closing.rs"
    path.write_text(path.read_text(encoding="utf-8") + "\n}\n", encoding="utf-8")
    control_b = fmt("control, stray brace in closing.rs (expect a failure)", top, config, work)

    # Round trip with the tool root would use, outside any repository.
    rt = work / "round-trip"
    for f in FILES:
        (rt / CH).mkdir(parents=True, exist_ok=True)
        shutil.copyfile(Path(src) / CH / f, rt / CH / f)
    patch_abs = str(Path(patch).resolve())
    forward = subprocess.run(["git", "apply", patch_abs], cwd=rt, capture_output=True, text=True)
    same_new = all(filecmp.cmp(rt / CH / f, Path(edited) / CH / f, shallow=False) for f in FILES)
    back = subprocess.run(["git", "apply", "-R", patch_abs], cwd=rt, capture_output=True, text=True)
    same_old = all(filecmp.cmp(rt / CH / f, Path(src) / CH / f, shallow=False) for f in FILES)
    print(f"round trip: forward exit {forward.returncode}, equals the edited copies {same_new}; "
          f"reverse exit {back.returncode}, equals the baseline {same_old}")

    ok = (base == 0 and new == 0 and control_a != 0 and control_b != 0 and same_new and same_old
          and forward.returncode == 0 and back.returncode == 0)
    print("result:", "PASS" if ok else "CHECK ABOVE")
    raise SystemExit(0 if ok else 1)


if __name__ == "__main__":
    main()
