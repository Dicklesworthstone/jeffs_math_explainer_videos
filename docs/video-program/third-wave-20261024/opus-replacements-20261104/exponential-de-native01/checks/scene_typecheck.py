"""Type-check the whole integrated crate against the pinned engine's existing metadata. No Cargo, no codegen, no link.

Usage from the repo root:  python3 -I -B scene_typecheck.py SCRATCH_DIR
SCRATCH_DIR/integrated must come from make_integration.py. This compiles that crate's real src/main.rs, so root's
A-C scenes, the module list, the registry and the fact wiring are all under the check, in normal and test cfg, and
under clippy-driver with -D warnings. Reads existing rmeta from target/debug/deps (the dep-info names the a399d62
checkout) and emits explainer_score and explainer_kit metadata from current source into SCRATCH_DIR.
"""
import glob
import os
import subprocess
import sys
import time

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
DEPS = ROOT + "/target/debug/deps"
CRATE = S + "/integrated"
MAIN = CRATE + "/src/main.rs"
fails = 0


def rel(s):
    return s.replace(CRATE + "/", "").replace(ROOT + "/", "").replace(S + "/", "<scratch>/")


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def one(pat):
    hits = [h for h in glob.glob(f"{DEPS}/lib{pat}-*.rmeta") if os.path.getsize(h) > 0]
    assert len(hits) == 1, (pat, hits)
    return hits[0]


def rustc(driver, args, label, show=True):
    t = time.time()
    r = subprocess.run(
        [driver, "--edition", "2024", "--emit=metadata", "-L", f"dependency={S}", "-L", f"dependency={DEPS}"] + args,
        capture_output=True,
        text=True,
        cwd=ROOT,
        env={**os.environ, "CARGO_PKG_VERSION": "0.1.0", "CARGO_PKG_NAME": "x", "CARGO_MANIFEST_DIR": CRATE},
    )
    ck(r.returncode == 0, label)
    if show and r.stderr.strip():
        print(rel(r.stderr)[-24000:])
    return r


fmn = one("fmn")
print("fmn rmeta:", rel(fmn))
dep = open(fmn.replace("/libfmn-", "/fmn-").replace(".rmeta", ".d"), encoding="utf-8").read()
ck("franken_manim-6641d0e09642f63e/a399d62/crates/fmn/src/lib.rs" in dep, "fmn metadata was built from the a399d62 checkout")

score = os.path.join(S, "libexplainer_score.rmeta")
rustc("rustc", ["--crate-type", "rlib", "--crate-name", "explainer_score", "-A", "warnings", "-o", score,
                ROOT + "/crates/explainer_score/src/lib.rs"], "explainer_score metadata from current source")
kit = os.path.join(S, "libexplainer_kit.rmeta")
rustc("rustc", ["--crate-type", "rlib", "--crate-name", "explainer_kit", "-A", "warnings",
                "--extern", f"fmn={fmn}", "--extern", f"fmn_output={one('fmn_output')}",
                "--extern", f"fmn_codec={one('fmn_codec')}", "--extern", f"serde_json={one('serde_json')}",
                "--extern", f"explainer_score={score}", "-o", kit,
                ROOT + "/crates/explainer_kit/src/lib.rs"], "explainer_kit metadata from current source")
EXT = ["--extern", f"fmn={fmn}", "--extern", f"explainer_kit={kit}", "--extern", f"explainer_score={score}"]
NAME = ["--crate-type", "bin", "--crate-name", "exponential_magic_de"]

rustc("rustc", NAME + EXT + ["-o", S + "/typecheck_main.rmeta", MAIN], "the integrated crate in normal cfg")
rustc("rustc", NAME + EXT + ["--test", "-o", S + "/typecheck_test.rmeta", MAIN], "the integrated crate in test cfg")
rustc("clippy-driver", NAME + EXT + ["-D", "warnings", "-o", S + "/clippy_main.rmeta", MAIN],
      "Clippy, normal cfg, -D warnings")
rustc("clippy-driver", NAME + EXT + ["--test", "-D", "warnings", "-o", S + "/clippy_test.rmeta", MAIN],
      "Clippy, test cfg, -D warnings (the all-targets half)")

print("SCENE TYPE CHECK:", "OK" if fails == 0 else f"{fails} FAIL")
sys.exit(1 if fails else 0)
