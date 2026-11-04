"""Build the D and E integration against root's staged A-C crate, in scratch, and write the patch that does it.

Usage from the repo root:  python3 -I -B make_integration.py SCRATCH_DIR
Reads root's stage, changes nothing there. SCRATCH_DIR gets base/ (the stage) and integrated/ (the stage with D and E).
Writes patches/integration.patch in this bundle: `patch -p1 --fuzz=0` from inside native-crate applies it.
"""
import filecmp
import os
import shutil
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STAGE = ROOT + "/narration_lab/third-wave-20261024/root/elevenlabs-exponential-20261101/abc-teaching-stage01/native-crate"
L = HERE + "/candidate/limit01/"
O = HERE + "/candidate/only01/"


def tree(src, dst):
    if os.path.exists(dst):
        moved = dst + ".old"
        n = 0
        while os.path.exists(f"{moved}{n}"):
            n += 1
        os.rename(dst, f"{moved}{n}")
    shutil.copytree(src, dst, ignore=shutil.ignore_patterns("target", "renders", "logs", ".git"))


def edit(path, old, new):
    t = open(path, encoding="utf-8").read()
    assert t.count(old) == 1, (path, old, t.count(old))
    open(path, "w", encoding="utf-8").write(t.replace(old, new))


base = S + "/base"
out = S + "/integrated"
os.makedirs(S, exist_ok=True)
tree(STAGE, base)
tree(STAGE, out)

for src, dst in [
    (L + "limit_model.rs", "src/limit_model.rs"),
    (O + "solution_model.rs", "src/solution_model.rs"),
    (L + "tiny_multipliers.rs", "src/chapters/tiny_multipliers.rs"),
    (O + "only_solution.rs", "src/chapters/only_solution.rs"),
]:
    shutil.copyfile(src, f"{out}/{dst}")

edit(out + "/src/main.rs", "mod facts;\n", "mod facts;\nmod limit_model;\n")
edit(out + "/src/main.rs", "mod series_model;\n", "mod series_model;\nmod solution_model;\n")
edit(
    out + "/src/chapters/mod.rs",
    "//! A-C development registry; D-I remain required for the full film.",
    "//! A-E development registry; F-I remain required for the full film.",
)
edit(out + "/src/chapters/mod.rs", "mod factorials;\n", "mod factorials;\nmod only_solution;\n")
edit(out + "/src/chapters/mod.rs", "pub mod thumbnail;\n", "pub mod thumbnail;\nmod tiny_multipliers;\n")

t = open(out + "/src/chapters/mod.rs", encoding="utf-8").read()
key = '"03_product_grid"'
assert t.count(key) == 1
at = t.index(key)
end = t.index("},", at) + 2
row = """
        Chapter {
            key: "04_tiny_multipliers",
            make: |kit| Box::new(tiny_multipliers::TinyMultipliers { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "05_whole_sum",
            make: |kit| Box::new(tiny_multipliers::WholeSum { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "06_only_solution",
            make: |kit| Box::new(only_solution::OnlySolution { kit }),
            intensity: 0.45,
        },"""
open(out + "/src/chapters/mod.rs", "w", encoding="utf-8").write(t[:end] + row + t[end:])

facts = out + "/src/facts.rs"
edit(
    facts,
    "    crate::series_model::append_facts(&mut facts)?;\n",
    "    crate::series_model::append_facts(&mut facts)?;\n"
    "    crate::limit_model::append_facts(&mut facts)?;\n"
    "    crate::solution_model::append_facts(&mut facts)?;\n",
)

with open(out + "/oracle.py", "a", encoding="utf-8") as f:
    f.write(open(L + "oracle.py", encoding="utf-8").read())
    f.write(open(O + "oracle.py", encoding="utf-8").read())
with open(out + "/script.tsv", "a", encoding="utf-8") as f:
    f.write(open(L + "script-slice.tsv", encoding="utf-8").read())
    f.write(open(O + "script-slice.tsv", encoding="utf-8").read())

# Paths in the patch are relative to native-crate. git diff --no-index exits 1 when the trees differ.
r = subprocess.run(
    ["git", "diff", "--no-index", "--no-color", "--no-ext-diff", "--src-prefix=a/", "--dst-prefix=b/", "base", "integrated"],
    cwd=S,
    capture_output=True,
    text=True,
)
assert r.returncode == 1, r.stderr
text = r.stdout.replace("a/base/", "a/").replace("b/integrated/", "b/").replace("a/integrated/", "a/")
text = "".join(l for l in text.splitlines(True) if not l.startswith("index "))
os.makedirs(HERE + "/patches", exist_ok=True)
with open(HERE + "/patches/integration.patch", "w", encoding="utf-8") as f:
    f.write(text)

cmp = filecmp.dircmp(base, out)
print("files changed at the crate top:", sorted(cmp.diff_files))
print("patch lines:", text.count("\n"))
