"""Gate G1 for one video: every number the video computes equals its oracle's.

    ./x <slug> facts-check            (python3 -I tools/facts_check.py <slug>)

Runs the video's `facts` command (cargo run --release; the binary is named in
explainer_corpus/<slug>/Cargo.toml) and its `oracle.py` (with numpy, via uv;
an oracle that needs more names it in a `# oracle-packages: scipy` line among
its first ten lines), then compares: every key the video prints must appear in
the oracle's output with the identical value. The oracle may print more
(research rows the video does not show). Exit 1 on any missing key, differing value, repeated key or an
empty list, 2 on usage.

Once a video has its documented allow-list (docs/description_numbers.tsv), the
published description is checked too: every number in docs/description.md must
be one of these facts or an allow-listed row (tools/description_numbers.py,
doctrine 2; Video J's description once said 81.9 g for the facts' 90.9).

Where `rch` is installed the build runs on a worker (nothing compiles on the
owner's Mac): HEAD plus the changed and new tracked-or-unignored files as
overlays, so gitignored renders are never synced. Set RUSTUP_TOOLCHAIN (see
AGENTS.md, Toolchain). On a build machine that opts out of the fleet
(RCH_SHIM_LOCAL_IDE set, as the build host's project.env does), and where rch
is absent, it is plain `cargo`.
"""

import importlib.util
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


def oracle_packages(oracle):
    """numpy, then whatever the oracle's `# oracle-packages: a b` line names."""
    packages = ["numpy"]
    for line in oracle.read_text().splitlines()[:10]:
        m = re.match(r"#\s*oracle-packages:\s*(.*)$", line)
        if m:
            for name in m.group(1).split():
                if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", name):
                    sys.exit(f"facts-check: bad package name {name!r} in {oracle.name}")
                if name not in packages:
                    packages.append(name)
    return packages


def description_problems(crate, facts_text):
    """None when the video has no description allow-list yet; otherwise (summary line, problems)
    from tools/description_numbers.py over docs/description.md against `facts_text`."""
    allow = crate / "docs" / "description_numbers.tsv"
    description = crate / "docs" / "description.md"
    if not allow.is_file():
        return None
    if not description.is_file():
        return f"description-numbers {crate.name}: no docs/description.md", ["description: missing"]
    # loaded by path: python3 -I keeps this script's directory off sys.path
    spec = importlib.util.spec_from_file_location("description_numbers", ROOT / "tools" / "description_numbers.py")
    numbers = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(numbers)
    _, problems, c = numbers.check(description.read_text(), facts_text, numbers.read_allow(allow))
    summary = (f"description-numbers {crate.name}: {c['numbers']} numbers, {c['fact']} facts, "
               f"{c['allowed']} allow-listed, {c['unmatched']} unmatched: {'PASS' if not problems else f'{len(problems)} FAIL'}")
    return summary, [f"description: {p}" for p in problems]


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
        ["uv", "run", "--quiet", "--no-project",
         *(arg for name in oracle_packages(crate / "oracle.py") for arg in ("--with", name)),
         "python3", "-I", "-B", "oracle.py"],
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
    described = description_problems(crate, oracle.stdout + "\n" + video.stdout)
    for p in problems + (described[1] if described else []):
        print("MISMATCH " + p)
    print(f"facts-check {slug}: {len(ours)} video facts against {len(theirs)} oracle keys: "
          f"{'PASS' if not problems else f'{len(problems)} FAIL'}")
    if described:
        print(described[0])
        problems += described[1]
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
