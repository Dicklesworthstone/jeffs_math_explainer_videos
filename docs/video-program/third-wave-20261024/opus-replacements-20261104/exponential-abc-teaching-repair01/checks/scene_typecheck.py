"""Type-check the repaired scenes against the pinned engine's existing metadata. No Cargo, no codegen, no linking.

Usage from the repo root:  python3 -I -B scene_typecheck.py SCRATCH_DIR
Reads existing rmeta from target/debug/deps (written by an earlier Cargo build at the pin; the dep-info names the
a399d62 checkout). Emits metadata for explainer_score and explainer_kit from the CURRENT working-tree source into
SCRATCH_DIR, then for one scene crate holding the three candidate scenes, their models and facts01's registry.
Nothing is written in the repo.
"""
import glob
import os
import subprocess
import sys
import time

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
os.makedirs(S, exist_ok=True)
DEPS = ROOT + "/target/debug/deps"
R = ROOT + "/docs/video-program/third-wave-20261024/opus-replacements-20261104/"
C = R + "exponential-abc-teaching-repair01/candidate/"
F = R + "exponential-rate-facts01/candidate/"
fails = 0


def rel(s):
    return s.replace(ROOT + "/", "").replace(S + "/", "<scratch>/").replace(
        "docs/video-program/third-wave-20261024/opus-replacements-20261104/", ""
    )


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def one(pat):
    hits = [h for h in glob.glob(f"{DEPS}/lib{pat}-*.rmeta") if os.path.getsize(h) > 0]
    assert len(hits) == 1, (pat, hits)
    return hits[0]


def rustc(driver, args, label):
    t = time.time()
    r = subprocess.run(
        [driver, "--edition", "2024", "--emit=metadata", "-L", f"dependency={S}", "-L", f"dependency={DEPS}"] + args,
        capture_output=True,
        text=True,
        cwd=ROOT,
        env={**os.environ, "CARGO_PKG_VERSION": "0.1.0", "CARGO_PKG_NAME": "x", "CARGO_MANIFEST_DIR": ROOT},
    )
    ck(r.returncode == 0, label)
    if r.stderr.strip():
        print(rel(r.stderr)[-16000:])
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
EXT = ["--extern", f"fmn={fmn}", "--extern", f"explainer_kit={kit}"]

main = f'''
#[path = "{F}src/rate_model.rs"]
mod rate_model;
#[path = "{F}src/facts.rs"]
mod facts;
#[path = "{C}series01/series_model.rs"]
mod series_model;
#[path = "{C}product01/product_model.rs"]
mod product_model;
#[path = "{C}rate02/rate_law.rs"]
mod rate_law;
#[path = "{C}series01/factorials.rs"]
mod factorials;
#[path = "{C}product01/product_grid.rs"]
mod product_grid;

use fmn::prelude::SceneConstruct;

// Stands in for the registry: each scene boxed the way chapters/mod.rs does, and every fact model reached.
pub fn registry(kit: explainer_kit::Kit) -> Vec<Box<dyn SceneConstruct>> {{
    vec![
        Box::new(rate_law::RateLaw {{ kit: kit.clone() }}),
        Box::new(factorials::Factorials {{ kit: kit.clone() }}),
        Box::new(product_grid::ProductGrid {{ kit }}),
    ]
}}
fn main() {{
    let f = facts::facts().and_then(|mut f| {{
        series_model::append_facts(&mut f)?;
        product_model::append_facts(&mut f)?;
        Ok(f)
    }});
    let _ = (f.map(|f| f.print()), registry as fn(explainer_kit::Kit) -> Vec<Box<dyn SceneConstruct>>);
}}
'''
p = os.path.join(S, "scene_crate.rs")
with open(p, "w", encoding="utf-8") as out:
    out.write(main)

rustc("rustc", EXT + ["-D", "warnings", "--crate-name", "scene_crate", "-o", os.path.join(S, "scene_crate.rmeta"), p],
      "rustc -D warnings: scenes and models type-check against pinned fmn, normal cfg")
rustc("rustc", EXT + ["--test", "-A", "dead_code", "--crate-name", "scene_crate_t", "-o", os.path.join(S, "scene_crate_t.rmeta"), p],
      "rustc: same crate in test cfg")
rustc("clippy-driver", EXT + ["-D", "warnings", "--crate-name", "scene_clip", "-o", os.path.join(S, "scene_clip.rmeta"), p],
      "clippy-driver -D warnings, normal cfg")
rustc("clippy-driver", EXT + ["-D", "warnings", "-A", "dead_code", "--test", "--crate-name", "scene_clip_t", "-o", os.path.join(S, "scene_clip_t.rmeta"), p],
      "clippy-driver -D warnings, test cfg")
print("RESULT", "PASS" if not fails else f"{fails} FAIL")
sys.exit(1 if fails else 0)
