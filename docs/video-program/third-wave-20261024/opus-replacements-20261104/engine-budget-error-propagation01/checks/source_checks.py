#!/usr/bin/env python3
"""Source-only checks for engine-budget-error-propagation.patch. Compiles and runs nothing.

    python3 -I -B source_checks.py STAGE_ENGINE NEW_EMPTY_SCRATCH_DIR

STAGE_ENGINE: root's engine-secondary-validation01 `engine` folder (read only).
NEW_EMPTY_SCRATCH_DIR: a folder that does not exist yet; the stage's crates/fmn is copied there
and patched. Output prints paths relative to these folders.

Checks: the stage still has the baseline bytes; `git apply --check` and `patch -p1` both take the
patch; the patched files match expected-engine.sha256; rustfmt (the pinned nightly, edition 2024)
finds nothing to change in the patched rendering.rs (which also walks compiled.rs and the other
rendering submodules) or in the new test file. A PASS here says nothing about compiling,
Clippy or test behaviour.
"""
import hashlib
import shutil
import subprocess
import sys
from pathlib import Path

if len(sys.argv) != 3:
    sys.exit(__doc__)
STAGE, SCRATCH = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
HERE = Path(__file__).resolve().parent
BUNDLE = HERE.parent
PATCH = BUNDLE / "engine-budget-error-propagation.patch"
TOOLCHAIN = "+nightly-2026-08-31"
failures = []


def check(ok, what):
    print(("  PASS " if ok else "  FAIL ") + what)
    if not ok:
        failures.append(what)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows(name):
    return [line.split(maxsplit=1) for line in (BUNDLE / name).read_text().splitlines()]


def run(args, cwd):
    done = subprocess.run(args, cwd=cwd, capture_output=True, text=True)
    return done.returncode, (done.stdout + done.stderr).strip()


print("-- baseline")
for digest, rel in rows("baseline-engine.sha256"):
    check(sha(STAGE / rel) == digest, f"stage {rel} is the baseline {digest[:12]}")
check(not (STAGE / "crates/fmn/tests/slow_sink_budget_refusal.rs").exists(), "stage has no slow_sink_budget_refusal.rs yet")

print("-- replay")
if SCRATCH.exists():
    sys.exit(f"{SCRATCH.name} exists; give a new folder")
shutil.copytree(STAGE / "crates/fmn", SCRATCH / "crates/fmn")
code, out = run(["git", "apply", "--check", str(PATCH)], SCRATCH)
check(code == 0, f"git apply --check exit {code} {out}")
code, out = run(["patch", "-p1", "--forward", "--batch", "-i", str(PATCH)], SCRATCH)
check(code == 0, f"patch -p1 exit {code}: {' / '.join(out.splitlines())}")
for digest, rel in rows("expected-engine.sha256"):
    check(sha(SCRATCH / rel) == digest, f"patched {rel} is {digest[:12]}")
check(not list(SCRATCH.rglob("*.rej")) and not list(SCRATCH.rglob("*.orig")), "no .rej or .orig files")

print("-- rustfmt (format only)")
code, out = run(["rustfmt", TOOLCHAIN, "--version"], SCRATCH)
print("  " + out)
for rel in ("crates/fmn/src/rendering.rs", "crates/fmn/tests/slow_sink_budget_refusal.rs"):
    code, out = run(["rustfmt", TOOLCHAIN, "--edition", "2024", "--check", rel], SCRATCH)
    check(code == 0 and not out, f"rustfmt --check {rel} exit {code}{(': ' + out) if out else ''}")

print()
print("RESULT " + ("PASS" if not failures else f"FAIL {len(failures)}"))
sys.exit(1 if failures else 0)
