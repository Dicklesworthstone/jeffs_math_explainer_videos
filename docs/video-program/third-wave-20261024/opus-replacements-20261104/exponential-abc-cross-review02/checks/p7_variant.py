"""Answer finding P7's open question: does TransformMatchingTex type-check in product_grid at this pin?

Usage from the repo root, after scene_typecheck.py:  python3 -I -B p7_variant.py SCRATCH_DIR
Rewrites a scratch copy of the candidate's 03_3 morph and 03_6 morph to the precedent's form
(explainer_corpus/square_packing/src/chapters/the_turned_tile.rs:248-252) and type-checks it.
Types only. Whether the pairing reads well is a picture question nobody can answer from here.
"""
import glob
import os
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
D = ROOT + "/target/debug/deps"
R = ROOT + "/docs/video-program/third-wave-20261024/opus-replacements-20261104/"
W = R + "exponential-expanded-review01/"
F = R + "exponential-rate-facts01/candidate/"
text = open(W + "candidate/product01/product_grid.rs", encoding="utf-8").read()

OLD_33 = """        let expression = k.tex(s, TEX[0], 27.0, Palette::blue(), &[], v(WORK_X, 0.0))?;
        reveal(s, expression)?;
        for (sentence, index) in [(1, 1), (3, 2)] {
            k.until_sentence(s, "03_3", sentence)?;
            let to = k.tex(s, TEX[index], 27.0, Palette::blue(), &[], v(WORK_X, 0.0))?;
            s.play(Transform::new(expression, to).rt(0.6))?;
        }
"""
NEW_33 = """        let (mut expression, mut spans) = k.tex_spans(s, TEX[0], 27.0, Palette::blue(), &[], v(WORK_X, 0.0))?;
        reveal(s, expression)?;
        for (sentence, index) in [(1, 1), (3, 2)] {
            k.until_sentence(s, "03_3", sentence)?;
            let (to, to_spans) = k.tex_spans(s, TEX[index], 27.0, Palette::blue(), &[], v(WORK_X, 0.0))?;
            let m = TransformMatchingTex::new(s.arena_mut(), expression, to, &spans, &to_spans)?;
            let a = s.prepare(m.rt(0.6))?;
            s.play_prepared(vec![a])?;
            expression = to;
            spans = to_spans;
        }
"""
OLD_36 = """        let half = k.tex(s, TEX[10], 31.0, Palette::green(), &[], v(0.0, -0.5))?;
        reveal(s, half)?;
"""
NEW_36 = """        let (half, half_spans) = k.tex_spans(s, TEX[10], 31.0, Palette::green(), &[], v(0.0, -0.5))?;
        reveal(s, half)?;
"""
OLD_36B = """        let positive = k.tex(s, TEX[6], 31.0, Palette::green(), &[], v(0.0, -0.5))?;
        s.play(Transform::new(half, positive).rt(0.5))?;
"""
NEW_36B = """        let (positive, positive_spans) = k.tex_spans(s, TEX[6], 31.0, Palette::green(), &[], v(0.0, -0.5))?;
        let m = TransformMatchingTex::new(s.arena_mut(), half, positive, &half_spans, &positive_spans)?;
        let a = s.prepare(m.rt(0.5))?;
        s.play_prepared(vec![a])?;
"""
for old, new in ((OLD_33, NEW_33), (OLD_36, NEW_36), (OLD_36B, NEW_36B)):
    assert text.count(old) == 1, old
    text = text.replace(old, new)
# Not in fmn::prelude at this pin (the first run of this check failed with E0433 and rustc named this path).
IMPORT = "use fmn::prelude::*;\n"
assert text.count(IMPORT) == 1
text = text.replace(IMPORT, "use fmn::library::TransformMatchingTex;\n" + IMPORT)
v = os.path.join(S, "product_grid_p7.rs")
with open(v, "w", encoding="utf-8") as out:
    out.write(text)
fmn = [h for h in glob.glob(f"{D}/libfmn-*.rmeta") if os.path.getsize(h) > 0][0]
main = f'''
#[path = "{W}candidate/product01/product_model.rs"]
mod product_model;
#[path = "{v}"]
mod product_grid;
use fmn::prelude::SceneConstruct;
fn main() {{
    let mut f = explainer_kit::Facts::new();
    let _ = product_model::append_facts(&mut f);
    let _ = (product_grid::TEXT, product_grid::TEX);
    let _ = (|kit: explainer_kit::Kit| -> Box<dyn SceneConstruct> {{ Box::new(product_grid::ProductGrid {{ kit }}) }}) as fn(explainer_kit::Kit) -> Box<dyn SceneConstruct>;
}}
'''
p = os.path.join(S, "p7_crate.rs")
with open(p, "w", encoding="utf-8") as out:
    out.write(main)
r = subprocess.run(
    ["clippy-driver", "--edition", "2024", "--emit=metadata", "-D", "warnings", "-L", f"dependency={S}", "-L", f"dependency={D}",
     "--extern", f"fmn={fmn}", "--extern", f"explainer_kit={S}/libexplainer_kit.rmeta", "--crate-name", "p7_crate",
     "-o", os.path.join(S, "p7.rmeta"), p],
    capture_output=True, text=True, cwd=ROOT,
)
print("P7 variant, clippy-driver -D warnings, exit", r.returncode)
print(r.stderr.replace(ROOT + "/", "").replace(S + "/", "<scratch>/").replace(R.replace(ROOT + "/", ""), "")[-1800:])
