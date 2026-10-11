"""Gate G1 for one video: every number the video computes equals its oracle's.

    ./x <slug> facts-check            (python3 -I tools/facts_check.py <slug>)

Runs the video's `facts` command (cargo run --release; the binary is named in
explainer_corpus/<slug>/Cargo.toml) and its `oracle.py` (with numpy, via uv),
then compares: every key the video prints must appear in the oracle's output
with the identical value. The oracle may print more (research rows the video
does not show). Exit 1 on any missing key or differing value, 2 on usage.
Needs the build environment of the session (CARGO_TARGET_DIR and friends).
"""

import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]


def parse(text):
    out = {}
    for line in text.splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        key, _, value = line.partition("\t")
        out[key.strip()] = value.strip()
    return out


def main(argv):
    if len(argv) != 1:
        sys.exit(__doc__)
    slug = argv[0]
    crate = ROOT / "explainer_corpus" / slug
    manifest = (crate / "Cargo.toml").read_text()
    package = re.search(r'^name = "([^"]+)"', manifest, re.M).group(1)
    binary = re.search(r'\[\[bin\]\]\s*name = "([^"]+)"', manifest).group(1)
    video = subprocess.run(
        ["cargo", "run", "--release", "-q", "-p", package, "--bin", binary,
         "-Zgit=shallow-deps", "-Zgitoxide=fetch", "--", "facts"],
        cwd=ROOT, capture_output=True, text=True)
    if video.returncode != 0:
        sys.stderr.write(video.stderr)
        sys.exit(f"facts-check: {binary} facts failed ({video.returncode})")
    oracle = subprocess.run(
        ["uv", "run", "--quiet", "--no-project", "--with", "numpy", "python3", "-I", "-B", "oracle.py"],
        cwd=crate, capture_output=True, text=True)
    if oracle.returncode != 0:
        sys.stderr.write(oracle.stderr[-4000:])
        sys.exit(f"facts-check: oracle.py failed ({oracle.returncode})")
    ours, theirs = parse(video.stdout), parse(oracle.stdout)
    problems = []
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
