"""Compile and run Bronze's six new tests with rustc alone: no crate, no fmn, no render.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex13-14-cross-review01/checks/standalone_tests.py

The handoffs say all six are unrun. This lifts them word for word from the two staged files, and only the pure
items they need, builds them with the film's real model.rs and a shim whose Plane and Frame2 bodies are lifted
from draw.rs and plot.rs, then puts defects back one at a time and requires a failure (a run failure, or a
compile failure for the const assertions). It also compiles `root_under` verbatim against a small hand-written
stub of Stage, for the borrow and iterator shape only. It is a stand-in, not the crate's `cargo test`: `construct`
is not type-checked, and fmn is not involved. Writes only to a new temporary directory.
"""
import os
import re
import shutil
import subprocess
import tempfile

STAGE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex13-radius-repair01/"
CHAPTERS = "explainer_corpus/complex_analysis_visual/src/chapters/"
RI = STAGE + CHAPTERS + "real_integral.rs"
CO = STAGE + "continuation14/" + CHAPTERS + "continuing.rs"
MODEL = "explainer_corpus/complex_analysis_visual/src/model.rs"
DRAW = CHAPTERS + "draw.rs"
PLOT = "crates/explainer_kit/src/plot.rs"


def read(path):
    return open(path, encoding="utf-8").read()


def balanced(text, start):
    at = text.index("{", start)
    depth = 0
    while True:
        depth += {"{": 1, "}": -1}.get(text[at], 0)
        at += 1
        if depth == 0:
            return text[start:at]


def item(text, head):
    start = text.index(head)
    end = text.index("\n", start)
    if head.startswith(("const", "pub const")) and text[start : end].rstrip().endswith(";"):
        return text[start:end]
    return balanced(text, start)


def shim():
    draw, plot = read(DRAW), read(PLOT)
    frame = plot[plot.index("pub struct Frame2") :]
    frame_struct = balanced(frame, 0)
    p_fn = balanced(plot, plot.index("pub fn p(&self, x: f64, y: f64) -> Vec3"))
    plane_new = balanced(draw, draw.index("pub fn new(center: Vec3, width: f64, x: (f64, f64), y: (f64, f64)) -> Self"))
    plane_p = balanced(draw, draw.index("pub fn p(&self, z: C) -> Vec3"))
    plane_unit = balanced(draw, draw.index("pub fn unit(&self) -> f64"))
    return f"""
use crate::model::C;
pub type Vec3 = [f64; 3];
pub fn v(x: f64, y: f64) -> Vec3 {{ [x, y, 0.0] }}
#[derive(Clone, Copy, Debug, PartialEq)]
{frame_struct}
impl Frame2 {{
    {p_fn}
}}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {{ pub frame: Frame2 }}
impl Plane {{
    {plane_new}
    {plane_p}
    {plane_unit}
}}
"""


def source_ri(text):
    items = "\n".join(
        item(text, h)
        for h in (
            "fn plane() -> Plane",
            "fn pole_height(r: f64, at: f64) -> f64",
            "fn r_name_z(r: f64, th: f64) -> C",
            "const R_NAME_SHOWN: [f64; 2]",
            "fn r_name_opacity(th: f64) -> f64",
            "const POLE_NAME_X: f64",
            "const MINUS_POLE_NAME_X: f64",
            "const R_BACK: f64",
            "const I_BACK: f64",
        )
    )
    a = text.index("    /// What the walk's names have to stay clear of, in plane units.")
    b = text.index("    #[test]\n    fn the_sum_of_the_upper_poles_is_i_root_two")
    return f"""
mod ri {{
    use super::*;
    use crate::model::{{self, C}};
    use std::f64::consts::PI;
    {items}
    #[cfg(test)]
    mod tests {{
        use super::*;
        {text[a:b]}
    }}
}}
"""


def source_co(text):
    items = item(text, "pub fn chain_plane() -> Plane")
    a = text.index("    /// One mark, from the first fade to the last:")
    b = text.index("    #[test]\n    fn the_square_root_comes_back_as_minus_one")
    return f"""
mod co {{
    use super::*;
    use crate::model::{{self, C, ONE}};
    {items}
    #[cfg(test)]
    mod tests {{
        use super::*;
        {text[a:b]}
    }}
}}
"""


def stub_root_under(text):
    return f"""
mod stub {{
    pub mod fmn {{ pub type Result<T> = std::result::Result<T, super::Err>; }}
    #[derive(Debug)] pub struct Err;
    impl From<StageError> for Err {{ fn from(_: StageError) -> Self {{ Err }} }}
    #[derive(Debug)] pub struct StageError;
    #[derive(Clone, Copy, Debug, PartialEq)] pub struct Mob(pub u32);
    pub struct Stage<'a> {{ pub roots: Vec<Mob>, pub life: &'a () }}
    impl Stage<'_> {{
        pub fn add_to_scene(&mut self, m: Mob) -> Result<(), StageError> {{
            self.roots.retain(|&r| r != m);
            self.roots.push(m);
            Ok(())
        }}
    }}
    {item(text, "fn root_under(stage: &mut Stage<'_>, mobs: &[Mob], mark: &[Mob; 2]) -> fmn::Result<()>")}
    #[test]
    fn the_shape_compiles_and_the_ring_ends_on_top() {{
        let life = ();
        let mut stage = Stage {{ roots: vec![Mob(8), Mob(9), Mob(1)], life: &life }};
        let mark = [Mob(8), Mob(9)];
        root_under(&mut stage, &[Mob(2), Mob(3)], &mark).unwrap();
        assert_eq!(stage.roots, vec![Mob(1), Mob(2), Mob(3), Mob(8), Mob(9)]);
        let _ = mark;
    }}
}}
"""


def build_and_run(ri_text, co_text):
    with tempfile.TemporaryDirectory(prefix="complex13-14-tests-") as work:
        shutil.copy(MODEL, os.path.join(work, "model.rs"))
        for name, text in (("real_integral.rs", ri_text), ("continuing.rs", co_text)):
            with open(os.path.join(work, name), "w", encoding="utf-8") as out:
                out.write(text)
        main = "#![allow(dead_code, unused_imports, unused_variables)]\nmod model;\n" + shim() + source_ri(ri_text) + source_co(co_text) + stub_root_under(co_text) + "\nfn main() {}\n"
        with open(os.path.join(work, "main.rs"), "w", encoding="utf-8") as out:
            out.write(main)
        built = subprocess.run(["rustc", "--edition", "2024", "--test", os.path.join(work, "main.rs"), "-o", os.path.join(work, "t.bin"), "--cap-lints", "allow"], capture_output=True, text=True, check=False)
        if built.returncode != 0:
            return None, built.stderr.replace(work, "<work>")
        ran = subprocess.run([os.path.join(work, "t.bin"), "--skip", "model::tests"], capture_output=True, text=True, check=False, cwd=work)
        return ran.stdout, ran.stderr


TESTS = {
    "ri::tests::the_walks_r_is_never_shown_across_the_segment",
    "ri::tests::the_walks_r_and_its_backing_keep_off_everything_else",
    "ri::tests::the_poles_name_keeps_its_backing_off_the_axis_and_the_ring",
    "ri::tests::the_poles_names_stay_clear_as_they_close_in",
    "ri::tests::the_spoke_passes_behind_the_pole_and_both_stay_on_screen",
    "co::tests::the_poles_mark_stays_on_top_of_all_that_reaches_it",
    "stub::the_shape_compiles_and_the_ring_ends_on_top",
}


def results(out):
    return dict(re.findall(r"^test (\S+) \.\.\. (\w+)", out, re.M))


ri_text, co_text = read(RI), read(CO)
print(subprocess.run(["rustc", "--version"], capture_output=True, text=True, check=False).stdout.strip())
out, err = build_and_run(ri_text, co_text)
if out is None:
    print("the stand-in did not build:\n" + err[:3000])
    print("RESULT FAIL")
    raise SystemExit(1)
got = results(out)
bad = 0
for name in sorted(TESTS):
    state = got.get(name, "missing")
    print(f"{'PASS' if state == 'ok' else 'FAIL'} {name}: {state}")
    bad += state != "ok"
if bad:
    print(out[-3000:])

# Clippy on the stand-in: lints in the lifted items and tests only, not in model.rs, which the crate already passes
with tempfile.TemporaryDirectory(prefix="complex13-14-tests-") as work:
    shutil.copy(MODEL, os.path.join(work, "model.rs"))
    for name, text in (("real_integral.rs", ri_text), ("continuing.rs", co_text)):
        with open(os.path.join(work, name), "w", encoding="utf-8") as out:
            out.write(text)
    main = "#![allow(dead_code, unused_imports, unused_variables)]\nmod model;\n" + shim() + source_ri(ri_text) + source_co(co_text) + stub_root_under(co_text) + "\nfn main() {}\n"
    main_path = os.path.join(work, "main.rs")
    with open(main_path, "w", encoding="utf-8") as out:
        out.write(main)
    lint = subprocess.run(["clippy-driver", "--edition", "2024", "--test", main_path, "-o", os.path.join(work, "c.bin"), "--error-format=short"], capture_output=True, text=True, check=False)
    rows = [l.replace(work, "<work>") for l in lint.stderr.splitlines() if ": warning" in l or ": error" in l]
    ours = [l for l in rows if "model.rs" not in l and "generated" not in l]
    print(f"clippy-driver exit {lint.returncode}; {len(rows)} rows, {len(ours)} outside model.rs")
    lines = main.split("\n")
    for row in ours[:20]:
        at = re.search(r"main\.rs:(\d+):", row)
        print("  " + row[:150] + ("  <- " + lines[int(at[1]) - 1].strip()[:90] if at else ""))
    clippy_clean = not ours
    # positive control: the same call must report a lint it is known to fire on, or zero rows would mean nothing
    control_path = os.path.join(work, "control.rs")
    with open(control_path, "w", encoding="utf-8") as out:
        out.write("fn main() {\n    let v = vec![1, 2, 3];\n    for i in 0..v.len() {\n        println!(\"{}\", v[i]);\n    }\n}\n")
    control = subprocess.run(["clippy-driver", "--edition", "2024", control_path, "-o", os.path.join(work, "k.bin"), "--error-format=short"], capture_output=True, text=True, check=False)
    fired = "needless_range_loop" in control.stderr or "loop variable" in control.stderr
    print(f"clippy control {'fired' if fired else 'DID NOT FIRE'}: a needless range loop")
    clippy_clean = clippy_clean and fired

PLANTS = [
    ("R shown from the walk's start", "ri", "const R_NAME_SHOWN: [f64; 2] = [0.28, 0.42];", "const R_NAME_SHOWN: [f64; 2] = [0.0, 0.14];"),
    ("R shown a little early", "ri", "const R_NAME_SHOWN: [f64; 2] = [0.28, 0.42];", "const R_NAME_SHOWN: [f64; 2] = [0.24, 0.42];"),
    ("R's ramp ending too late", "ri", "const R_NAME_SHOWN: [f64; 2] = [0.28, 0.42];", "const R_NAME_SHOWN: [f64; 2] = [0.28, 0.9];"),
    ("R's backing too small for the letter", "ri", "const R_BACK: f64 = 0.135;", "const R_BACK: f64 = 0.1;"),
    ("R's backing wide enough to touch the spoke", "ri", "const R_BACK: f64 = 0.135;", "const R_BACK: f64 = 0.2;"),
    ("i's backing too small for the letter", "ri", "const I_BACK: f64 = 0.1;", "const I_BACK: f64 = 0.07;"),
    ("i's backing onto the ring and axis", "ri", "const I_BACK: f64 = 0.1;", "const I_BACK: f64 = 0.17;"),
    ("i written over the ring", "ri", "const POLE_NAME_X: f64 = -0.28;", "const POLE_NAME_X: f64 = -0.18;"),
    ("-i written over the arrow head", "ri", "const MINUS_POLE_NAME_X: f64 = 0.36;", "const MINUS_POLE_NAME_X: f64 = 0.46;"),
    ("poles closing in too far", "ri", "fn pole_height(r: f64, at: f64) -> f64 {\n    r / at\n}", "fn pole_height(r: f64, at: f64) -> f64 {\n    0.8 * r / at\n}"),
    ("R on the far side of the spoke", "ri", "C::polar(0.25, th - PI / 2.0)", "C::polar(0.1, th - PI / 2.0)"),
    ("the spoke rooted after the re-adds", "ri", "        stage.add_to_scene(spoke)?;\n        for m in [arrows[0]", "        for m in [arrows[0]"),
    ("R's backing left out of the fade", "ri", "fade_out_all(stage, &[zdot, spoke, r_name, r_back, i_back], 0.3)?;", "fade_out_all(stage, &[zdot, spoke, r_name, i_back], 0.3)?;"),
    ("i's backing left out of the fade", "ri", "fade_out_all(stage, &[zdot, spoke, r_name, r_back, i_back], 0.3)?;", "fade_out_all(stage, &[zdot, spoke, r_name, r_back], 0.3)?;"),
    ("no opacity ramp on the backing", "ri", "                a.set_opacity(r_back, shown, true);\n", ""),
    ("the gold disk not routed under", "co", "        root_under(stage, &[first], &pole1)?;\n", ""),
    ("the region after its fade", "co", "        root_under(stage, &[region], &pole1)?;\n        fade_in_all(stage, &[region, region_says], 0.5)?;", "        fade_in_all(stage, &[region, region_says], 0.5)?;\n        root_under(stage, &[region], &pole1)?;"),
    ("the second mark back", "co", "        line.wait_sentence(stage, 6)?;\n        let same", "        let pole_top = pole_mark(stage, cp.p(ONE));\n        line.wait_sentence(stage, 6)?;\n        let same"),
    ("a fourth disk in the scene", "co", "        line.wait_sentence(stage, 6)?;\n        let same", "        let extra = disk(stage, &cp, ONE, 1.0, output(), 0.1)?;\n        line.wait_sentence(stage, 6)?;\n        let same"),
    ("root_under leaving the mark out", "co", "for &m in mobs.iter().chain(mark) {", "for &m in mobs.iter() {"),
]
caught = 0
for what, which, old, new in PLANTS:
    text = ri_text if which == "ri" else co_text
    # the source-text tests quote some of these lines: plant in the film code, so a test that matches its own literal is seen
    assert text[: text.index("#[cfg(test)]\nmod tests")].count(old) == 1, what
    r_new, c_new = (text.replace(old, new, 1), co_text) if which == "ri" else (ri_text, text.replace(old, new, 1))
    out_p, err_p = build_and_run(r_new, c_new)
    if out_p is None:
        how = "compile: " + next((l for l in err_p.splitlines() if l.startswith("error")), "error")[:80]
        hit = True
    else:
        failed = sorted(k for k, s in results(out_p).items() if s != "ok")
        hit = bool(failed)
        how = ", ".join(n.split("::")[-1] for n in failed)
    caught += hit
    print(f"{'caught' if hit else 'MISSED'} {what} -> {how}")
print(f"planted defects caught: {caught} of {len(PLANTS)}")
print(f"clippy clean on the lifted code: {clippy_clean}")
print("RESULT " + ("PASS" if not bad and caught == len(PLANTS) and clippy_clean else "FAIL"))
