"""Negative controls for scene_typecheck.py. Run it after that script, with the same scratch folder.

Usage from the repo root:  python3 -I -B scene_controls.py SCRATCH_DIR
A check that cannot fail proves nothing, so each control plants one defect in a scratch copy of the integrated crate
and must fail. Wiring controls take out one line of the integration, so the patch's own lines are shown to matter.
"""
import glob
import os
import shutil
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
DEPS = ROOT + "/target/debug/deps"
CRATE = S + "/integrated"
WORK = S + "/controls"
os.makedirs(WORK, exist_ok=True)
hits = [h for h in glob.glob(f"{DEPS}/libfmn-*.rmeta") if os.path.getsize(h) > 0]
EXT = ["--extern", f"fmn={hits[0]}", "--extern", f"explainer_kit={S}/libexplainer_kit.rmeta", "--extern", f"explainer_score={S}/libexplainer_score.rmeta"]
fails = 0


def build(name, rel, old, new, driver="rustc", extra=()):
    d = os.path.join(WORK, name)
    if os.path.exists(d):
        os.rename(d, d + f".old{len(glob.glob(d + '.old*'))}")
    shutil.copytree(CRATE, d)
    p = os.path.join(d, rel)
    text = open(p, encoding="utf-8").read()
    assert text.count(old) >= 1, (rel, old)
    with open(p, "w", encoding="utf-8") as out:
        out.write(text.replace(old, new, 1))
    args = [driver, "--edition", "2024", "--emit=metadata", "-L", f"dependency={S}", "-L", f"dependency={DEPS}"] + EXT
    args += list(extra) + ["--crate-type", "bin", "--crate-name", "control", "-o", os.path.join(d, "out.rmeta"), os.path.join(d, "src/main.rs")]
    return subprocess.run(
        args,
        capture_output=True,
        text=True,
        cwd=ROOT,
        env={**os.environ, "CARGO_PKG_VERSION": "0.1.0", "CARGO_PKG_NAME": "x", "CARGO_MANIFEST_DIR": d},
    )


def expect(label, r, needle):
    global fails
    ok = r.returncode != 0 and needle in r.stderr
    print(("PASS " if ok else "FAIL ") + label + f"  (exit {r.returncode})")
    fails += 0 if ok else 1
    if not ok:
        print(r.stderr.replace(ROOT + "/", "").replace(S + "/", "<scratch>/")[-1200:])


D = "src/chapters/tiny_multipliers.rs"
E = "src/chapters/only_solution.rs"
expect(
    "a misspelt method deep in the D scene",
    build("bad_method", D, "stage.play_prepared(vec![paid])?;", "stage.play_preparedd(vec![paid])?;"),
    "play_preparedd",
)
expect(
    "a wrong argument type in the E scene",
    build("bad_arg", E, "stage.play(pulse.rt(0.9))?;", 'stage.play(pulse.rt("slow"))?;'),
    "mismatched types",
)
expect(
    "a text constant left unused under -D warnings",
    build("dead_const", E, 'const THIRD_DEBT: &str', 'const NEVER_DRAWN: &str = "x";\nconst THIRD_DEBT: &str', extra=["-D", "warnings"]),
    "NEVER_DRAWN",
)
expect(
    "a clippy lint under clippy-driver -D warnings",
    build(
        "clip",
        "src/solution_model.rs",
        "    (1..=degree as u64).product()\n",
        "    return (1..=degree as u64).product();\n",
        driver="clippy-driver",
        extra=["-D", "warnings"],
    ),
    "needless_return",
)
print("--- wiring: each line of the integration is load-bearing")
expect("without mod limit_model", build("no_limit", "src/main.rs", "mod limit_model;\n", ""), "limit_model")
expect("without mod solution_model", build("no_solution", "src/main.rs", "mod solution_model;\n", ""), "solution_model")
expect(
    "without the D row in the registry, the D scene is dead code",
    build(
        "no_row",
        "src/chapters/mod.rs",
        """        Chapter {
            key: "04_tiny_multipliers",
            make: |kit| Box::new(tiny_multipliers::TinyMultipliers { kit }),
            intensity: 0.5,
        },
""",
        "",
        extra=["-D", "warnings"],
    ),
    "never constructed",
)
expect(
    "without the solution facts line, the model is dead code",
    build("no_facts", "src/facts.rs", "    crate::solution_model::append_facts(&mut facts)?;\n", "", extra=["-D", "warnings"]),
    "never used",
)
print("RESULT", "controls all behaved" if not fails else f"{fails} control FAIL")
sys.exit(1 if fails else 0)
