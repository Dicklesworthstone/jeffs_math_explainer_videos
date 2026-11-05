#!/usr/bin/env python3
"""Source-only checks for engine-budget-error-precedence.patch. Compiles and runs nothing.

    python3 -I -B source_checks.py STAGE_ENGINE NEW_EMPTY_SCRATCH_DIR

STAGE_ENGINE: root's engine-secondary-validation01 `engine` folder, as attempt 04 staged it (read
only). NEW_EMPTY_SCRATCH_DIR: a folder that does not exist yet; the stage's crates/fmn-runtime and
crates/fmn are copied there and patched. Output prints paths relative to these folders.

Checks: the stage still has the baseline bytes and carries the sealed F1 repair; the F1 bundle
still verifies; `git apply --check` and `patch -p1` both take the patch; the patched files match
expected-engine.sha256; the slow-sink test keeps every assertion line and its eight rounds; the
new hook has exactly the expected definitions and the coordinator one constructor call; rustfmt
(the pinned nightly, with the runner's flags) finds nothing to change in the three files. A PASS
says nothing about compiling, Clippy or test behaviour.
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
F1 = BUNDLE.parent / "engine-budget-error-propagation01"
PATCH = BUNDLE / "engine-budget-error-precedence.patch"
TOOLCHAIN = "+nightly-2026-08-31"
CRATES = ("crates/fmn-runtime", "crates/fmn")
TEST = "crates/fmn/tests/slow_sink_budget_refusal.rs"
failures = []


def check(ok, what):
    print(("  PASS " if ok else "  FAIL ") + what)
    if not ok:
        failures.append(what)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows(path):
    return [line.split(maxsplit=1) for line in path.read_text().splitlines()]


def run(args, cwd):
    done = subprocess.run(args, cwd=cwd, capture_output=True, text=True)
    return done.returncode, (done.stdout + done.stderr).strip()


def code_lines(path):
    return [line for line in path.read_text().splitlines() if not line.strip().startswith("//")]


print("-- baseline")
for digest, rel in rows(BUNDLE / "baseline-engine.sha256"):
    check(sha(STAGE / rel) == digest, f"stage {rel} is the baseline {digest[:12]}")
for digest, rel in rows(F1 / "expected-engine.sha256"):
    check(sha(STAGE / rel) == digest, f"stage {rel} carries sealed F1 {digest[:12]}")
for digest, rel in rows(F1 / "SHA256SUMS"):
    check(sha(F1 / rel) == digest, f"F1 bundle {rel} unchanged")

print("-- replay")
if SCRATCH.exists():
    sys.exit(f"{SCRATCH.name} exists; give a new folder")
for crate in CRATES:
    shutil.copytree(STAGE / crate, SCRATCH / crate)
code, out = run(["git", "apply", "--check", str(PATCH)], SCRATCH)
check(code == 0, f"git apply --check exit {code} {out}")
code, out = run(["patch", "-p1", "--forward", "--batch", "-i", str(PATCH)], SCRATCH)
check(code == 0, f"patch -p1 exit {code}: {' / '.join(out.splitlines())}")
for digest, rel in rows(BUNDLE / "expected-engine.sha256"):
    check(sha(SCRATCH / rel) == digest, f"patched {rel} is {digest[:12]}")
check(not list(SCRATCH.rglob("*.rej")) and not list(SCRATCH.rglob("*.orig")), "no .rej or .orig files")

print("-- regression strength")
before, after = code_lines(STAGE / TEST), code_lines(SCRATCH / TEST)
changed = [(old, new) for old, new in zip(before, after) if old != new]
renamed = [(old.replace("frame 2 needs", "frame 1 needs"), new) for old, new in changed]
check(len(before) == len(after) and len(changed) == 1 and renamed[0][0] == renamed[0][1],
      f"slow-sink test: outside comments only the panic message changes ({len(before)} code lines)")
kept = sum("assert" in line for line in after)
check(kept == sum("assert" in line for line in before), f"slow-sink test keeps its {kept} assert lines")
check("const ROUNDS: usize = 8;" in (SCRATCH / TEST).read_text(), "slow-sink test keeps 8 rounds")
hooks = sorted(
    f"{path.relative_to(SCRATCH)}:{number}"
    for path in SCRATCH.rglob("*.rs")
    for number, line in enumerate(path.read_text().splitlines(), 1)
    if "fn emit_failure_caused_by(" in line
)
print("  hook definitions: " + ", ".join(hooks))
check(len(hooks) == 4, "4 hook definitions: the runtime default, the facade, and one test stage each")
runtime = (SCRATCH / "crates/fmn-runtime/src/pipeline.rs").read_text()
check(runtime.count("Coordinator::new(") == 1, "one coordinator constructor call")
check(runtime.count("self.fail_frame(") == 3, "error, panic and disconnection of a frame go through fail_frame")

print("-- rustfmt (format only)")
code, out = run(["rustfmt", TOOLCHAIN, "--version"], SCRATCH)
print("  " + out)
files = [rel for _, rel in rows(BUNDLE / "expected-engine.sha256")]
code, out = run(["rustfmt", TOOLCHAIN, "--check", "--edition", "2024", "--config", "skip_children=true",
                 *files], SCRATCH)
check(code == 0 and not out, f"rustfmt --check (runner flags) on {len(files)} files exit {code}"
      + (f": {out}" if out else ""))

print()
print("RESULT " + ("PASS" if not failures else f"FAIL {len(failures)}"))
sys.exit(1 if failures else 0)
