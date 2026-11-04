"""Scratch only: both 08-10 patches on a copy of the canonical crate; parse, cue and literal checks.
Run from the repo root: python3 -I -B <this file> <scratch dir>. Touches nothing outside the scratch dir."""
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

SP = pathlib.Path(sys.argv[1])
CRATE = pathlib.Path("explainer_corpus/logarithm_mysteries")
STAGE = pathlib.Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-repair08-10")
CH = CRATE / "src/chapters"
FILES = ["src/chapters/draw.rs", "src/chapters/product_rule.rs", "src/chapters/matrix_logs.rs", "src/chapters/doing_nothing.rs", "docs/storyboard.md"]


def run(*cmd, cwd=None):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)


work = pathlib.Path(tempfile.mkdtemp(dir=SP))
(work / CRATE).mkdir(parents=True)
for sub in ["src", "docs"]:
    shutil.copytree(CRATE / sub, work / CRATE / sub)
for name in ["script.tsv", "oracle.py", "Cargo.toml"]:
    shutil.copy2(CRATE / name, work / CRATE / name)
for patch in ["shared-draw-helpers.patch", "chapters-08-10.patch"]:
    r = run("patch", "-p1", "--no-backup-if-mismatch", "-i", str((STAGE / "patches" / patch).resolve()), cwd=work)
    print(patch, "exit", r.returncode, "fuzz or reject:", [l for l in r.stdout.splitlines() if "fuzz" in l or "FAILED" in l])
for rel in FILES:
    print(" ", rel, "equals candidate:", (work / CRATE / rel).read_bytes() == (STAGE / "candidate" / CRATE / rel).read_bytes())

fmt = run("rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", str(work / CRATE / "src/main.rs"))
print("rustfmt whole crate: parse errors:", [l for l in fmt.stderr.splitlines() if l.startswith("error")],
      "| files with diffs:", sorted({l.split(":")[0].split("/")[-1] for l in fmt.stdout.splitlines() if l.startswith("Diff in")}))

fails = []
for root in (CRATE, work / CRATE):
    out = run(sys.executable, "-I", "-B", str(root / "docs/check_cues.py")).stdout
    fails.append(out[out.index("FAIL"):] if "FAIL" in out else "PASS")
print("check_cues failure lists identical to canonical:", fails[0] == fails[1], f"({len(fails[1].splitlines())} lines)")

# every say/until/hold anchor, baseline against candidate, scene code only
ANCHOR = re.compile(r'kit\.(say|until_sentence|hold)\(\s*stage(?:,\s*"([0-9_]+)"(?:,\s*(\d+))?)?')
for name in ["product_rule", "matrix_logs", "doing_nothing"]:
    a = ANCHOR.findall((CH / f"{name}.rs").read_text().split("#[cfg(test)]")[0])
    b = ANCHOR.findall((STAGE / "candidate" / CH / f"{name}.rs").read_text().split("#[cfg(test)]")[0])
    extra = [x for x in b if x not in a]
    gone = [x for x in a if x not in b]
    print(f"{name}: anchors {len(a)} -> {len(b)}, added {extra}, removed {gone}, says same: {[x for x in a if x[0]=='say'] == [x for x in b if x[0]=='say']}")
    pa = (CH / f"{name}.rs").read_text().split("#[cfg(test)]")[0].count("predict(")
    pb = (STAGE / "candidate" / CH / f"{name}.rs").read_text().split("#[cfg(test)]")[0].count("predict(")
    print(f"  predict calls {pa} -> {pb}")

# Clippy's approx_constant table against every float literal the patches add
CONST = {"E": "2.718281828459045", "FRAC_1_PI": "0.318309886183791", "FRAC_1_SQRT_2": "0.707106781186548", "FRAC_2_PI": "0.636619772367581",
         "FRAC_2_SQRT_PI": "1.128379167095513", "FRAC_PI_2": "1.570796326794897", "FRAC_PI_3": "1.047197551196598", "FRAC_PI_4": "0.785398163397448",
         "FRAC_PI_6": "0.523598775598299", "FRAC_PI_8": "0.392699081698724", "LN_2": "0.693147180559945", "LN_10": "2.302585092994046",
         "LOG10_2": "0.301029995663981", "LOG10_E": "0.434294481903252", "LOG2_10": "3.321928094887362", "LOG2_E": "1.442695040888963",
         "PI": "3.141592653589793", "SQRT_2": "1.414213562373095", "TAU": "6.283185307179586"}
added = []
for patch in ["shared-draw-helpers.patch", "chapters-08-10.patch"]:
    for line in (STAGE / "patches" / patch).read_text().splitlines():
        if line.startswith("+") and not line.startswith("+++") and "//" not in line:
            added += re.findall(r"(?<![\w.])\d+\.\d+(?![\w.])", line)
close = sorted({lit for lit in added for n, c in CONST.items() if len(lit) >= 4 and (c.startswith(lit) or c[: len(lit)] == lit)})
print("added float literals:", len(set(added)), "| near a std constant:", close)

# constant-valued asserts among the added lines: only consts and literals in the condition
for patch in ["shared-draw-helpers.patch", "chapters-08-10.patch"]:
    for line in (STAGE / "patches" / patch).read_text().splitlines():
        m = re.match(r"\+\s*assert!\((.*)\);$", line)
        if m:
            body = re.sub(r"\[\d+\]", "", m.group(1))
            words = re.findall(r"[A-Za-z_][A-Za-z_0-9.]*", body)
            if all(w.isupper() or w.replace("_", "").isupper() for w in words) and words:
                print("constant-only assert:", line.strip()[:110])
