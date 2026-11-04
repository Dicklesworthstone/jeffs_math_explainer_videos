"""Type-check and Clippy this bundle's candidate against the pinned engine's existing metadata. No Cargo, no codegen, no link.

Usage from the repo root:  python3 -I -B .../complex-color-repair03/checks/candidate_typecheck.py SCRATCH_DIR

Copies the crate's source from the working tree into SCRATCH_DIR/integrated, lays this bundle's candidate/ over it, and
compiles the film's src/main.rs and the lab's examples/plate_lab.rs in normal and test cfg, under rustc and under
clippy-driver with -D warnings. rustfmt --check on the changed files, with the repo's rustfmt.toml. Nothing here can
run a scene: only metadata exists for the engine. The method is exponential-de-native01/checks/scene_typecheck.py.
"""
import glob
import os
import pathlib
import shutil
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
DEPS = ROOT + "/target/debug/deps"
CRATE = S + "/integrated"
BUNDLE = pathlib.Path(__file__).resolve().parent.parent
SOURCE = ROOT + "/explainer_corpus/complex_analysis_visual"
fails = 0


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def one(pat):
    hits = [h for h in glob.glob(f"{DEPS}/lib{pat}-*.rmeta") if os.path.getsize(h) > 0]
    assert len(hits) == 1, (pat, hits)
    return hits[0]


def rel(s):
    return s.replace(CRATE + "/", "").replace(ROOT + "/", "").replace(S + "/", "<scratch>/")


def rustc(driver, args, label):
    r = subprocess.run(
        [driver, "--edition", "2024", "--emit=metadata", "-L", f"dependency={S}", "-L", f"dependency={DEPS}"] + args,
        capture_output=True,
        text=True,
        cwd=ROOT,
        env={**os.environ, "CARGO_PKG_VERSION": "0.1.0", "CARGO_PKG_NAME": "x", "CARGO_MANIFEST_DIR": CRATE},
    )
    ck(r.returncode == 0, label)
    if r.stderr.strip():
        print(rel(r.stderr)[-30000:])


if os.path.exists(CRATE):
    os.rename(CRATE, CRATE + f".{os.getpid()}")
os.makedirs(S, exist_ok=True)
os.makedirs(CRATE)
for name in ("src", "docs", "script.tsv", "Cargo.toml"):
    src = os.path.join(SOURCE, name)
    if os.path.isdir(src):
        shutil.copytree(src, os.path.join(CRATE, name))
    else:
        shutil.copy2(src, os.path.join(CRATE, name))
overlay = BUNDLE / "candidate/explainer_corpus/complex_analysis_visual"
changed = []
for f in sorted(overlay.rglob("*.rs")):
    target = pathlib.Path(CRATE) / f.relative_to(overlay)
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(f, target)
    changed.append(str(target))

fmn = one("fmn")
dep = open(fmn.replace("/libfmn-", "/fmn-").replace(".rmeta", ".d"), encoding="utf-8").read()
ck("franken_manim-6641d0e09642f63e/a399d62/crates/fmn/src/lib.rs" in dep, "fmn metadata was built from the a399d62 checkout")
score = os.path.join(S, "libexplainer_score.rmeta")
kit = os.path.join(S, "libexplainer_kit.rmeta")
if not (os.path.exists(score) and os.path.exists(kit)) or "--rebuild-kit" in sys.argv:
    rustc("rustc", ["--crate-type", "rlib", "--crate-name", "explainer_score", "-A", "warnings", "-o", score,
                    ROOT + "/crates/explainer_score/src/lib.rs"], "explainer_score metadata from current source")
    rustc("rustc", ["--crate-type", "rlib", "--crate-name", "explainer_kit", "-A", "warnings",
                    "--extern", f"fmn={fmn}", "--extern", f"fmn_output={one('fmn_output')}",
                    "--extern", f"fmn_codec={one('fmn_codec')}", "--extern", f"serde_json={one('serde_json')}",
                    "--extern", f"explainer_score={score}", "-o", kit,
                    ROOT + "/crates/explainer_kit/src/lib.rs"], "explainer_kit metadata from current source")
EXT = ["--extern", f"fmn={fmn}", "--extern", f"explainer_kit={kit}", "--extern", f"explainer_score={score}"]
FILM = ["--crate-type", "bin", "--crate-name", "complex_analysis_visual"]
LAB = ["--crate-type", "bin", "--crate-name", "plate_lab"]
MAIN, EXAMPLE = CRATE + "/src/main.rs", CRATE + "/examples/plate_lab.rs"
for driver, flags, tag in (("rustc", [], ""), ("clippy-driver", ["-D", "warnings"], "Clippy, ")):
    rustc(driver, FILM + EXT + flags + ["-o", f"{S}/{driver}_film.rmeta", MAIN], f"{tag}the film, normal cfg")
    rustc(driver, FILM + EXT + flags + ["--test", "-o", f"{S}/{driver}_film_t.rmeta", MAIN], f"{tag}the film, test cfg")
    rustc(driver, LAB + EXT + flags + ["-o", f"{S}/{driver}_lab.rmeta", EXAMPLE], f"{tag}the lab, normal cfg")
    rustc(driver, LAB + EXT + flags + ["--test", "-o", f"{S}/{driver}_lab_t.rmeta", EXAMPLE], f"{tag}the lab, test cfg")
fmt = subprocess.run(["rustfmt", "--edition", "2024", "--config-path", ROOT + "/rustfmt.toml", "--config", "skip_children=true",
                      "--unstable-features", "--check", *changed], capture_output=True, text=True, cwd=CRATE)
ck(fmt.returncode == 0, "rustfmt --check on the changed files")
if fmt.returncode:
    print(rel(fmt.stdout)[-12000:])
    print(rel(fmt.stderr)[-3000:])
# Each choice root could make: the film, under each select patch on A, with its tests.
for name in ("tile", "blend", "rows2", "cells2"):
    draw = pathlib.Path(CRATE) / "src/chapters/draw.rs"
    kept = draw.read_bytes()
    r = subprocess.run(["patch", "-p3", "--fuzz=0", "--no-backup-if-mismatch", "-i", str(BUNDLE / f"patches/select-{name}.patch")],
                       cwd=CRATE, capture_output=True, text=True)
    ck(r.returncode == 0, f"select-{name} applies")
    for driver, flags, tag in (("rustc", [], ""), ("clippy-driver", ["-D", "warnings"], "Clippy, ")):
        rustc(driver, FILM + EXT + flags + ["-o", f"{S}/{driver}_{name}.rmeta", MAIN], f"{tag}the film with select-{name}, normal cfg")
        rustc(driver, FILM + EXT + flags + ["--test", "-o", f"{S}/{driver}_{name}_t.rmeta", MAIN], f"{tag}the film with select-{name}, test cfg")
    draw.write_bytes(kept)
print("CANDIDATE TYPE CHECK:", "OK" if fails == 0 else f"{fails} FAIL")
sys.exit(1 if fails else 0)
