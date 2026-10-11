"""Gate G1 for one video: every number the video computes equals its oracle's.

    ./x <slug> facts-check            (python3 -I tools/facts_check.py <slug>)

Runs the video's `facts` command (cargo run --release; the binary is named in
explainer_corpus/<slug>/Cargo.toml) and its `oracle.py` (with numpy, via uv),
then compares: every key the video prints must appear in the oracle's output
with the identical value. The oracle may print more (research rows the video
does not show). Exit 1 on any missing key, differing value, repeated key or an
empty list, 2 on usage.

Where `rch` is installed the build runs on a worker (nothing compiles on the
owner's Mac): HEAD plus the changed and new tracked-or-unignored files as
overlays, so gitignored renders are never synced. Set RUSTUP_TOOLCHAIN (see
AGENTS.md, Toolchain). On a build machine that opts out of the fleet
(RCH_SHIM_LOCAL_IDE set, as the build host's project.env does), and where rch
is absent, it is plain `cargo`.
"""

import os
import pathlib
import re
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
# A cold build on a worker can take most of an hour; the slowest oracle (Video
# C's model-checker sweep) takes about 4-7 minutes.
BUILD_TIMEOUT_S = 2 * 3600
ORACLE_TIMEOUT_S = 30 * 60


def parse(text):
    """key -> value, and the keys printed more than once."""
    out, repeated = {}, []
    for line in text.splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        key, _, value = line.partition("\t")
        key = key.strip()
        if key in out:
            repeated.append(key)
        out[key] = value.strip()
    return out, repeated


def cargo_prefix():
    """`cargo`, or `rch exec` on HEAD with every changed or new file overlaid."""
    if not shutil.which("rch") or os.environ.get("RCH_SHIM_LOCAL_IDE"):
        return ["cargo"]
    git = lambda *a: subprocess.run(["git", *a], cwd=ROOT, capture_output=True,
                                    text=True, check=True, timeout=60).stdout.split("\n")
    paths = sorted({f for f in git("diff", "--name-only", "HEAD")
                    + git("ls-files", "--others", "--exclude-standard") if f})
    overlays = [arg for f in paths if (ROOT / f).is_file() for arg in ("-o", f)]
    return ["rch", "exec", "--base", "HEAD", "--clean-overlay", *overlays, "--", "cargo"]


def main(argv):
    if len(argv) != 1:
        sys.exit(__doc__)
    slug = argv[0]
    crate = ROOT / "explainer_corpus" / slug
    manifest = (crate / "Cargo.toml").read_text()
    package = re.search(r'^name = "([^"]+)"', manifest, re.M).group(1)
    binary = re.search(r'\[\[bin\]\]\s*name = "([^"]+)"', manifest).group(1)
    video = subprocess.run(
        [*cargo_prefix(), "run", "--release", "-q", "-p", package, "--bin", binary,
         "-Zgit=shallow-deps", "-Zgitoxide=fetch", "--", "facts"],
        cwd=ROOT, capture_output=True, text=True, timeout=BUILD_TIMEOUT_S)
    if video.returncode != 0:
        sys.stderr.write(video.stderr)
        sys.exit(f"facts-check: {binary} facts failed ({video.returncode})")
    oracle = subprocess.run(
        ["uv", "run", "--quiet", "--no-project", "--with", "numpy", "python3", "-I", "-B", "oracle.py"],
        cwd=crate, capture_output=True, text=True, timeout=ORACLE_TIMEOUT_S)
    if oracle.returncode != 0:
        sys.stderr.write(oracle.stderr[-4000:])
        sys.exit(f"facts-check: oracle.py failed ({oracle.returncode})")
    (ours, repeated), (theirs, oracle_repeated) = parse(video.stdout), parse(oracle.stdout)
    problems = [f"{key}: printed more than once by the video" for key in repeated]
    problems += [f"{key}: printed more than once by the oracle" for key in oracle_repeated]
    if not ours:
        problems.append("the video printed no facts")
    for key, value in ours.items():
        if key not in theirs:
            problems.append(f"{key}: only in the video ({value})")
        elif theirs[key] != value:
            problems.append(f"{key}: video {value}, oracle {theirs[key]}")
    for p in problems:
        print("MISMATCH " + p)
    print(f"facts-check {slug}: {len(ours)} video facts against {len(theirs)} oracle keys: "
          f"{'PASS' if not problems else f'{len(problems)} FAIL'}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
