"""Negative controls and integration variants for scene_typecheck.py. Run it after that script.

Usage from the repo root:  python3 -I -B scene_controls.py SCRATCH_DIR   (same dir scene_typecheck used)

A check that cannot fail proves nothing, so each control plants one defect in a scratch copy and must fail.
Variants ask what an integrator would hit: the unpatched base, and each stage applied alone.
"""
import glob
import os
import shutil
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
D = ROOT + "/target/debug/deps"
R = ROOT + "/docs/video-program/third-wave-20261024/opus-replacements-20261104/"
W = R + "exponential-expanded-review01/"
F = R + "exponential-rate-facts01/candidate/"
RATE = R + "exponential-expanded-rate02/rate_law.rs"
fails = 0
hits = [h for h in glob.glob(f"{D}/libfmn-*.rmeta") if os.path.getsize(h) > 0]
EXT = ["--extern", f"fmn={hits[0]}", "--extern", f"explainer_kit={S}/libexplainer_kit.rmeta"]


def rel(s):
    return s.replace(ROOT + "/", "").replace(S + "/", "<scratch>/").replace(
        "docs/video-program/third-wave-20261024/opus-replacements-20261104/", ""
    )


def build(name, mods, main_body, driver="rustc", extra=(), test=False):
    src = "".join(f'#[path = "{p}"]\nmod {m};\n' for m, p in mods) + main_body
    p = os.path.join(S, name + ".rs")
    with open(p, "w", encoding="utf-8") as out:
        out.write(src)
    args = [driver, "--edition", "2024", "--emit=metadata", "-L", f"dependency={S}", "-L", f"dependency={D}"] + EXT
    args += list(extra) + (["--test"] if test else []) + ["--crate-name", name, "-o", os.path.join(S, name + ".rmeta"), p]
    return subprocess.run(args, capture_output=True, text=True, cwd=ROOT)


def first_errors(r, n=6):
    out = []
    lines = rel(r.stderr).splitlines()
    for i, l in enumerate(lines):
        if l.startswith("error"):
            where = next((x.strip() for x in lines[i + 1 : i + 3] if "-->" in x), "")
            out.append(f"{l[:150]}  {where}")
    return out[:n]


def expect(label, r, should_fail, needle=""):
    global fails
    got_fail = r.returncode != 0
    ok = got_fail == should_fail and (not needle or needle in r.stderr)
    print(("PASS " if ok else "FAIL ") + label + f"  (exit {r.returncode})")
    fails += 0 if ok else 1
    if got_fail:
        for e in first_errors(r):
            print("    ", e)


CAND = [
    ("rate_model", F + "src/rate_model.rs"),
    ("facts", F + "src/facts.rs"),
    ("series_model", W + "candidate/series01/series_model.rs"),
    ("product_model", W + "candidate/product01/product_model.rs"),
    ("rate_law", RATE),
    ("factorials", W + "candidate/series01/factorials.rs"),
    ("product_grid", W + "candidate/product01/product_grid.rs"),
]
MAIN = """
use fmn::prelude::SceneConstruct;
pub fn registry(kit: explainer_kit::Kit) -> Vec<Box<dyn SceneConstruct>> {
    vec![
        Box::new(rate_law::RateLaw { kit: kit.clone() }),
        Box::new(factorials::Factorials { kit: kit.clone() }),
        Box::new(product_grid::ProductGrid { kit }),
    ]
}
fn main() {
    let f = facts::facts().and_then(|mut f| {
        series_model::append_facts(&mut f)?;
        product_model::append_facts(&mut f)?;
        Ok(f)
    });
    let _ = (f.map(|f| f.print()), registry as fn(explainer_kit::Kit) -> Vec<Box<dyn SceneConstruct>>);
    let _ = (factorials::TEXT, factorials::TEX, product_grid::TEXT, product_grid::TEX);
}
"""
expect("baseline: candidate set still type-checks", build("c_ok", CAND, MAIN, extra=["-D", "warnings"]), False)


def planted(src_path, old, new):
    text = open(src_path, encoding="utf-8").read()
    assert text.count(old) >= 1, old
    p = os.path.join(S, "planted_" + os.path.basename(src_path))
    with open(p, "w", encoding="utf-8") as out:
        out.write(text.replace(old, new, 1))
    return p


def swap(m, p):
    return [(a, (p if a == m else b)) for a, b in CAND]


# 1. a wrong method deep inside a scene body must fail
expect(
    "control: stage.wait on a misspelt method inside factorials::construct fails",
    build("c_bad_method", swap("factorials", planted(W + "candidate/series01/factorials.rs", "stage.wait(", "stage.waitt(")), MAIN),
    True,
    "waitt",
)
# 2. a wrong argument type in the product scene body must fail
expect(
    "control: wrong argument type in product_grid fails",
    build("c_bad_arg", swap("product_grid", planted(W + "candidate/product01/product_grid.rs", "s.play(a.rt(0.35))?;", 's.play(a.rt("slow"))?;')), MAIN),
    True,
    "mismatched types",
)
# 3. a dead function must fail under -D warnings
expect(
    "control: a dead function in series_model fails under -D warnings",
    build(
        "c_dead",
        swap("series_model", planted(W + "candidate/series01/series_model.rs", "pub fn factorial(", "pub fn never_called_control() -> u8 { 1 }\npub fn factorial(")),
        MAIN,
        extra=["-D", "warnings"],
    ),
    True,
    "never_called_control",
)
# 4. a Clippy lint must fail under clippy-driver
expect(
    "control: clippy::needless_return fails under clippy-driver -D warnings",
    build(
        "c_clip",
        swap("series_model", planted(W + "candidate/series01/series_model.rs", "    radius / (n + 2) as f64\n", "    return radius / (n + 2) as f64;\n")),
        MAIN,
        driver="clippy-driver",
        extra=["-D", "warnings"],
    ),
    True,
    "needless_return",
)

print("--- unpatched series and product, as originally staged")
BASE = [
    ("rate_model", F + "src/rate_model.rs"),
    ("facts", F + "src/facts.rs"),
    ("series_model", W + "base/series01/series_model.rs"),
    ("product_model", W + "base/product01/product_model.rs"),
    ("rate_law", RATE),
    ("factorials", W + "base/series01/factorials.rs"),
    ("product_grid", W + "base/product01/product_grid.rs"),
]
MAIN_BASE = """
use fmn::prelude::SceneConstruct;
pub fn registry(kit: explainer_kit::Kit) -> Vec<Box<dyn SceneConstruct>> {
    vec![
        Box::new(rate_law::RateLaw { kit: kit.clone() }),
        Box::new(factorials::Factorials { kit: kit.clone() }),
        Box::new(product_grid::ProductGrid { kit }),
    ]
}
fn main() {
    let f = facts::facts().and_then(|mut f| {
        series_model::append_facts(&mut f)?;
        Ok(f)
    });
    let _ = (f.map(|f| f.print()), registry as fn(explainer_kit::Kit) -> Vec<Box<dyn SceneConstruct>>);
    let _ = (factorials::TEXT, factorials::TEX, product_grid::TEXT, product_grid::TEX);
}
"""
r = build("b_plain", BASE, MAIN_BASE)
print("base rustc exit", r.returncode, "; warnings:", [l[:120] for l in rel(r.stderr).splitlines() if l.startswith("warning")][:10])
r = build("b_d", BASE, MAIN_BASE, driver="clippy-driver", extra=["-D", "warnings"])
print("base clippy -D warnings exit", r.returncode)
for e in first_errors(r, 12):
    print("    ", e)
r = build("b_d_t", BASE, MAIN_BASE, driver="clippy-driver", extra=["-D", "warnings"], test=True)
print("base clippy -D warnings, test cfg, exit", r.returncode)
for e in first_errors(r, 12):
    print("    ", e)

print("--- stages applied alone (dead code tells root what must land together)")
ALONE = {
    "facts01 alone (canonical main + rate_model + candidate facts.rs, no rate scene)": (
        [("rate_model", F + "src/rate_model.rs"), ("facts", F + "src/facts.rs")],
        "fn main() { let _ = facts::facts().map(|f| f.print()); }\n",
    ),
    "rate scene with rate02's own model (no facts01)": (
        [("rate_model", R + "exponential-expanded-rate02/rate_model.rs"), ("rate_law", RATE)],
        "use fmn::prelude::SceneConstruct;\nfn main() { let _ = (|kit: explainer_kit::Kit| -> Box<dyn SceneConstruct> { Box::new(rate_law::RateLaw { kit }) }) as fn(explainer_kit::Kit) -> Box<dyn SceneConstruct>; }\n",
    ),
    "series scene + model, facts call absent": (
        [("series_model", W + "candidate/series01/series_model.rs"), ("factorials", W + "candidate/series01/factorials.rs")],
        "use fmn::prelude::SceneConstruct;\nfn main() { let _ = (factorials::TEXT, factorials::TEX); let _ = (|kit: explainer_kit::Kit| -> Box<dyn SceneConstruct> { Box::new(factorials::Factorials { kit }) }) as fn(explainer_kit::Kit) -> Box<dyn SceneConstruct>; }\n",
    ),
    "product scene + model, facts call absent": (
        [("product_model", W + "candidate/product01/product_model.rs"), ("product_grid", W + "candidate/product01/product_grid.rs")],
        "use fmn::prelude::SceneConstruct;\nfn main() { let _ = (product_grid::TEXT, product_grid::TEX); let _ = (|kit: explainer_kit::Kit| -> Box<dyn SceneConstruct> { Box::new(product_grid::ProductGrid { kit }) }) as fn(explainer_kit::Kit) -> Box<dyn SceneConstruct>; }\n",
    ),
}
for i, (label, (mods, body)) in enumerate(ALONE.items()):
    r = build(f"alone_{i}", mods, body, extra=["-D", "warnings"])
    print(f"{label}: exit {r.returncode}")
    for e in first_errors(r, 12):
        print("    ", e)
print("RESULT", "controls all behaved" if not fails else f"{fails} control FAIL")
