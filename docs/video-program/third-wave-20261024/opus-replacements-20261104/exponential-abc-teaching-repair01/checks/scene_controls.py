"""Negative controls and stage-alone variants for scene_typecheck.py. Run it after that script.

Usage from the repo root:  python3 -I -B scene_controls.py SCRATCH_DIR   (the dir scene_typecheck used)
A check that cannot fail proves nothing, so each control plants one defect in a scratch copy and must fail.
Variants ask what an integrator would hit with each stage applied alone.
"""
import glob
import os
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
DEPS = ROOT + "/target/debug/deps"
R = ROOT + "/docs/video-program/third-wave-20261024/opus-replacements-20261104/"
C = R + "exponential-abc-teaching-repair01/candidate/"
F = R + "exponential-rate-facts01/candidate/"
fails = 0
hits = [h for h in glob.glob(f"{DEPS}/libfmn-*.rmeta") if os.path.getsize(h) > 0]
EXT = ["--extern", f"fmn={hits[0]}", "--extern", f"explainer_kit={S}/libexplainer_kit.rmeta"]


def rel(s):
    return s.replace(ROOT + "/", "").replace(S + "/", "<scratch>/").replace(
        "docs/video-program/third-wave-20261024/opus-replacements-20261104/", ""
    )


def build(name, mods, main_body, driver="rustc", extra=()):
    src = "".join(f'#[path = "{p}"]\nmod {m};\n' for m, p in mods) + main_body
    p = os.path.join(S, name + ".rs")
    with open(p, "w", encoding="utf-8") as out:
        out.write(src)
    args = [driver, "--edition", "2024", "--emit=metadata", "-L", f"dependency={S}", "-L", f"dependency={DEPS}"] + EXT
    args += list(extra) + ["--crate-name", name, "-o", os.path.join(S, name + ".rmeta"), p]
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
    ("series_model", C + "series01/series_model.rs"),
    ("product_model", C + "product01/product_model.rs"),
    ("rate_law", C + "rate02/rate_law.rs"),
    ("factorials", C + "series01/factorials.rs"),
    ("product_grid", C + "product01/product_grid.rs"),
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
}
"""
expect("baseline: candidate set still type-checks under -D warnings", build("c_ok", CAND, MAIN, extra=["-D", "warnings"]), False)


def planted(src_path, old, new):
    text = open(src_path, encoding="utf-8").read()
    assert text.count(old) >= 1, old
    p = os.path.join(S, "planted_" + os.path.basename(src_path))
    with open(p, "w", encoding="utf-8") as out:
        out.write(text.replace(old, new, 1))
    return p


def swap(m, p):
    return [(a, (p if a == m else b)) for a, b in CAND]


expect(
    "control: a misspelt method deep inside the factorials scene fails",
    build("c_bad_method", swap("factorials", planted(C + "series01/factorials.rs", "stage.wait(", "stage.waitt(")), MAIN),
    True,
    "waitt",
)
expect(
    "control: a wrong argument type in the new product cue fails",
    build("c_bad_arg", swap("product_grid", planted(C + "product01/product_grid.rs", "s.play(show_creation(solid).rt(0.9))?;", 's.play(show_creation(solid).rt("slow"))?;')), MAIN),
    True,
    "mismatched types",
)
expect(
    "control: a TeX constant left unused in factorials fails under -D warnings",
    build("c_dead", swap("factorials", planted(C + "series01/factorials.rs", "const DOTS: &str", 'const NEVER_DRAWN: &str = "x";\nconst DOTS: &str')), MAIN, extra=["-D", "warnings"]),
    True,
    "NEVER_DRAWN",
)
expect(
    "control: a dead function in series_model fails under -D warnings",
    build("c_dead_fn", swap("series_model", planted(C + "series01/series_model.rs", "pub fn factorial(", "pub fn never_called_control() -> u8 { 1 }\npub fn factorial(")), MAIN, extra=["-D", "warnings"]),
    True,
    "never_called_control",
)
expect(
    "control: clippy::needless_return fails under clippy-driver -D warnings",
    build(
        "c_clip",
        swap("series_model", planted(C + "series01/series_model.rs", "    RADIUS * index as f64 / PLOT_SEGMENTS as f64\n", "    return RADIUS * index as f64 / PLOT_SEGMENTS as f64;\n")),
        MAIN,
        driver="clippy-driver",
        extra=["-D", "warnings"],
    ),
    True,
    "needless_return",
)

print("--- stages alone (dead code tells root what must land together)")
ALONE = {
    "rate scene + facts01 model, nothing else": (
        [("rate_model", F + "src/rate_model.rs"), ("rate_law", C + "rate02/rate_law.rs")],
        "use fmn::prelude::SceneConstruct;\nfn main() { let _ = (|kit: explainer_kit::Kit| -> Box<dyn SceneConstruct> { Box::new(rate_law::RateLaw { kit }) }) as fn(explainer_kit::Kit) -> Box<dyn SceneConstruct>; let mut f = explainer_kit::Facts::new(); let _ = rate_model::append_facts(&mut f); }\n",
    ),
    "series scene + model, append_facts reached": (
        [("series_model", C + "series01/series_model.rs"), ("factorials", C + "series01/factorials.rs")],
        "use fmn::prelude::SceneConstruct;\nfn main() { let _ = (|kit: explainer_kit::Kit| -> Box<dyn SceneConstruct> { Box::new(factorials::Factorials { kit }) }) as fn(explainer_kit::Kit) -> Box<dyn SceneConstruct>; let mut f = explainer_kit::Facts::new(); let _ = series_model::append_facts(&mut f); }\n",
    ),
    "product scene + model, append_facts reached": (
        [("product_model", C + "product01/product_model.rs"), ("product_grid", C + "product01/product_grid.rs")],
        "use fmn::prelude::SceneConstruct;\nfn main() { let _ = (|kit: explainer_kit::Kit| -> Box<dyn SceneConstruct> { Box::new(product_grid::ProductGrid { kit }) }) as fn(explainer_kit::Kit) -> Box<dyn SceneConstruct>; let mut f = explainer_kit::Facts::new(); let _ = product_model::append_facts(&mut f); }\n",
    ),
}
for i, (label, (mods, body)) in enumerate(ALONE.items()):
    r = build(f"alone_{i}", mods, body, extra=["-D", "warnings"])
    print(f"{label}: exit {r.returncode}")
    for e in first_errors(r, 12):
        print("    ", e)
print("RESULT", "controls all behaved" if not fails else f"{fails} control FAIL")
sys.exit(1 if fails else 0)
