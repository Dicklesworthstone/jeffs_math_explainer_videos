"""Read-only source checks: which call sites sit outside test code, and whether the patches replay.

Replays happen on copies under the directory given as argv[1], which must be outside the repository.
Nothing here writes to the pinned checkout, Ruby's folder or the lab crate. No Cargo.

    python3 -I -B sites.py SCRATCH_DIR
"""
import hashlib
import pathlib
import re
import shutil
import subprocess
import sys

REPO = pathlib.Path(__file__).resolve().parents[6]
P = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104"
B = P / "engine-geometry-budget01"
C = P / "complex-color-comparison01"
LAB = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/color-native-stage01/crate"
PIN = pathlib.Path.home() / ".cargo/git/checkouts/franken_manim-6641d0e09642f63e/a399d62"
WORK = pathlib.Path(sys.argv[1])


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def test_start(path, cache={}):
    """The line of the file's first #[cfg(test)] module, or None. A lone #[cfg(test)] on an import does not count."""
    if path not in cache:
        cache[path] = None
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        for number, line in enumerate(lines[:-1], 1):
            if line.strip() == "#[cfg(test)]" and re.match(r"\s*(pub(\(\w+\))? )?mod \w+", lines[number]):
                cache[path] = number
                break
    return cache[path]


def non_test(pattern, globs=("crates/*/src/**/*.rs",)):
    seen = []
    for g in globs:
        for path in sorted(PIN.glob(g)):
            if path.name.endswith("_tests.rs"):
                continue
            start = test_start(path)
            for number, line in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
                if start and number >= start:
                    break
                if re.search(pattern, line) and not line.strip().startswith("//"):
                    seen.append(f"{path.relative_to(PIN).as_posix()}:{number}: {line.strip()[:72]}")
    return seen


def run(args, cwd):
    r = subprocess.run(args, cwd=cwd, capture_output=True, text=True)
    return r.returncode, (r.stdout + r.stderr).strip()


def main():
    head = run(["git", "rev-parse", "HEAD"], PIN)[1]
    status = run(["git", "status", "--short"], PIN)[1]
    print("pin", head)
    print("pin status:", status or "clean")

    print("\nnon-test `.sync(` in crates/*/src (before each file's first #[cfg(test)])")
    print("\n".join(non_test(r"\.sync\(")))

    print("\nnon-test renderer and compiler constructors")
    print("\n".join(non_test(r"RetainedFrameRenderer::new\(|VectorFrameCompiler::new\(")))

    print("\nnon-test RenderPlan::new() users outside fmn-render")
    print("\n".join(s for s in non_test(r"RenderPlan::new\(\)") if not s.startswith("crates/fmn-render")))

    print("\nsnapshot, journal or any second limit on the three patched paths")
    for rel in ("crates/fmn-render/src/retained.rs", "crates/fmn-render/src/vector_frame.rs", "crates/fmn/src/rendering/pipeline.rs"):
        path = PIN / rel
        start = test_start(path)
        lines = path.read_text(encoding="utf-8").splitlines()[: (start - 1) if start else None]
        hits = [f"{i}: {l.strip()[:70]}" for i, l in enumerate(lines, 1) if re.search(r"snapshot::|journal|RenderPlanLimits|LimitExceeded", l)]
        print(f"  {rel}: {hits or 'none'}")

    print("\nengine patch")
    patch = B / "engine-plan-limits.patch"
    print("  sha256", sha(patch))
    names = re.findall(r"^\+\+\+ b/(\S+)", patch.read_text(encoding="utf-8"), re.M)
    print(f"  files {len(names)}")
    tree = WORK / "pin-files"
    for name in names:
        (tree / name).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(PIN / name, tree / name)
    code, text = run(["patch", "-p1", "-F0", "-i", str(patch)], tree)
    print("  patch -p1 -F0 exit", code)
    assert code == 0, text
    for rel in ("crates/fmn-render/src/retained.rs", "crates/fmn-render/src/vector_frame.rs"):
        body = (tree / rel).read_text(encoding="utf-8")
        start = next(i for i, l in enumerate(body.splitlines()) if l.strip() == "#[cfg(test)]") if "#[cfg(test)]" in body else None
        lines = body.splitlines()[:start]
        print(f"  {rel}: plain .sync( left {sum(1 for l in lines if re.search(r'\.sync\(', l))}, sync_with_limits {sum(1 for l in lines if 'sync_with_limits' in l)}")
    tests = (tree / "crates/fmn-render/tests/stroke_profiles.rs").read_text(encoding="utf-8")
    print("  segment assertion in the patched test:", [l.strip() for l in tests.splitlines() if "plan.segments().len()" in l])

    print("\nlab patches, on one copy of examples/plate_lab.rs")
    source = LAB / "examples/plate_lab.rs"
    print("  lab plate_lab.rs sha256", sha(source))
    lab = WORK / "lab-files/examples"
    lab.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, lab / "plate_lab.rs")
    ruby = B / "lab-plate_lab.patch"
    print("  Ruby's patch sha256", sha(ruby))
    code, text = run(["patch", "-p1", "-F0", "-i", str(ruby)], WORK / "lab-files")
    print("  Ruby's, patch -p1 -F0 exit", code)
    assert code == 0, text
    body = (lab / "plate_lab.rs").read_text(encoding="utf-8")
    print("  cells4 jobs in plan():", [l.strip() for l in body.splitlines() if '"cells4"' in l and "add(" in l])
    mine = C / "patches/probe-plate-lab.patch"
    print("  frozen probe patch sha256", sha(mine))
    head_lines = [l for l in mine.read_text(encoding="utf-8").splitlines() if l.startswith(("--- ", "+++ "))]
    print("  frozen probe patch paths:", head_lines)
    code, text = run(["patch", "-p3", "-F0", "--dry-run", "-i", str(mine)], WORK / "lab-files")
    print("  frozen probe patch stacked on Ruby's, patch -p3 -F0 --dry-run exit", code)
    if code:
        print("   ", text.splitlines()[-1][:110] if text else "")
    minus = [l.strip() for l in mine.read_text(encoding="utf-8").splitlines() if l.startswith("-") and "cells4" in l]
    print("  what that patch removes for cells4:", minus)

    print("\nfilm's options and the kit's rebuilds")
    kit = REPO / "crates/explainer_kit/src"
    for rel, pattern in (("cli.rs", r"RenderOptions::with_format|delivery_options\(|o\.config = |o\.max_frames = |opts = o;"), ("encode.rs", r"RenderOptions::with_format")):
        for i, l in enumerate((kit / rel).read_text(encoding="utf-8").splitlines(), 1):
            if re.search(pattern, l):
                print(f"  crates/explainer_kit/src/{rel}:{i}: {l.strip()[:72]}")
    print("  RenderOptions in the film crate's src/:", sum(1 for p in (LAB / "src").rglob("*.rs") if "RenderOptions" in p.read_text(encoding="utf-8")), "files")
    print("  camera shots in the film crate's src/:", sum(1 for p in (LAB / "src").rglob("*.rs") if re.search(r"CameraShot|set_camera_shot|camera_shot", p.read_text(encoding="utf-8"))), "files")
    print("  plan_limits in the kit:", sum(1 for p in kit.rglob("*.rs") if "plan_limits" in p.read_text(encoding="utf-8")), "files")


if __name__ == "__main__":
    main()
