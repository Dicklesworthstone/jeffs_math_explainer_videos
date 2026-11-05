"""Type-check the new film code with rustc against stubs: a stand-in for the build root will run.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-teaching02/checks/typecheck_stub.py

It lifts `Turning`, its helpers and constants, the whole of 11_7 up to the wide plane, and the plate's lines
out of the staged counting.rs, with the film's real model.rs, and checks them (types, moves, borrows)
against stubs of Stage, Kit, Spoken and the draw helpers. The stubs' signatures were copied by hand from
franken_manim at the pin (fmn-mobject stage.rs) and from the crate's kit.rs and draw.rs. They prove nothing
about those sources, so a wrong copy here would hide a real error. Not `cargo check`, and no substitute for it.
Writes only to a new temporary directory.
"""
import os
import re
import subprocess
import tempfile

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-teaching02/"
STAGED = HERE + "explainer_corpus/complex_analysis_visual/src/chapters/counting.rs"
MODEL = "explainer_corpus/complex_analysis_visual/src/model.rs"

STUBS = r"""
#![allow(dead_code, unused)]
mod model;
use model::{C, Func, ONE, ZERO};
use std::f64::consts::TAU;
pub type Vec3 = [f64; 3];
pub const ORIGIN: Vec3 = [0.0; 3];
pub fn v(x: f64, y: f64) -> Vec3 { [x, y, 0.0] }
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Mob(u32);
#[derive(Clone, Copy, Debug)]
pub struct Srgb;
#[derive(Debug)]
pub struct StageError;
#[derive(Debug)]
pub struct UpdaterId;
pub mod fmn {
    #[derive(Debug)]
    pub struct Error;
    pub type Result<T> = std::result::Result<T, Error>;
    impl From<super::StageError> for Error { fn from(_: super::StageError) -> Self { Error } }
}
pub struct VMobject;
impl VMobject {
    pub fn new() -> Self { VMobject }
    pub fn points(&self) -> Vec<Vec3> { Vec::new() }
}
pub struct PolyBuilder;
impl PolyBuilder { pub fn build(self) -> VMobject { VMobject } }
pub fn polyline(_: Vec<Vec3>) -> PolyBuilder { PolyBuilder }
pub struct Arena;
impl Arena {
    pub fn add(&mut self, _: VMobject) -> Mob { Mob(0) }
    pub fn set_points(&mut self, _: Mob, _: Vec<Vec3>) -> Result<(), StageError> { Ok(()) }
    pub fn move_to(&mut self, _: Mob, _: Vec3, _: Vec3) {}
    pub fn tracker_value(&self, _: Mob) -> Option<f64> { None }
}
pub struct Time;
impl Time { pub fn to_f64(&self) -> f64 { 0.0 } }
pub struct SceneView;
impl SceneView { pub fn time(&self) -> Time { Time } }
pub struct AnimBuilder;
pub struct Report;
pub struct Stage<'a> { arena: Arena, life: &'a () }
impl Stage<'_> {
    pub fn arena_mut(&mut self) -> &mut Arena { &mut self.arena }
    pub fn scene(&self) -> SceneView { SceneView }
    pub fn add_value_tracker(&mut self, _: f64) -> Mob { Mob(0) }
    pub fn add_updater(&mut self, _: Mob, _: impl FnMut(&mut Arena, Mob) + 'static, _: bool) -> Result<UpdaterId, StageError> { Ok(UpdaterId) }
    pub fn add_to_scene(&mut self, _: Mob) -> Result<(), StageError> { Ok(()) }
    pub fn remove_from_scene(&mut self, _: Mob) {}
    pub fn clear_updaters(&mut self, _: Mob, _: bool) {}
    pub fn play(&mut self, _: AnimBuilder) -> fmn::Result<Report> { Ok(Report) }
}
pub fn tween(_: Mob, _: f64, _: f64) -> fmn::Result<AnimBuilder> { Ok(AnimBuilder) }
pub fn value(_: &Arena, _: Mob) -> f64 { 0.0 }
pub fn squash(w: C, _: f64, _: f64) -> C { w }
pub const WALK_SAMPLES: usize = 720;
pub fn hero() -> Srgb { Srgb }
pub fn ground() -> Srgb { Srgb }
pub struct Palette;
impl Palette {
    pub fn ink() -> Srgb { Srgb }
    pub fn faint() -> Srgb { Srgb }
}
#[derive(Clone)]
pub struct Kit;
impl Kit {
    pub fn text(&self, _: &mut Stage<'_>, _: &str, _: f64, _: Srgb, _: Vec3) -> fmn::Result<Mob> { Ok(Mob(0)) }
    pub fn tex(&self, _: &mut Stage<'_>, _: &str, _: f64, _: Srgb, _: &[(&str, Srgb)], _: Vec3) -> fmn::Result<Mob> { Ok(Mob(0)) }
}
#[derive(Clone, Copy)]
pub struct Frame2 { pub x: (f64, f64), pub y: (f64, f64) }
#[derive(Clone, Copy)]
pub struct Plane { pub frame: Frame2 }
impl Plane {
    pub fn new(_: Vec3, _: f64, _: (f64, f64), _: (f64, f64)) -> Self { Plane { frame: Frame2 { x: (0.0, 0.0), y: (0.0, 0.0) } } }
    pub fn p(&self, _: C) -> Vec3 { ORIGIN }
    pub fn axes(&self, _: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> { Ok(Vec::new()) }
    pub fn circle(&self, _: &mut Stage<'_>, _: C, _: f64, _: Srgb, _: f64) -> fmn::Result<Mob> { Ok(Mob(0)) }
}
pub struct RowStrips { pub group: Mob, pub rows: Vec<Mob>, pub cols: usize }
impl RowStrips { pub fn recolor(&self, _: &mut Arena, _: &[Vec<Srgb>]) -> bool { true } }
pub fn portrait(_: &mut Stage<'_>, _: Func, _: &Plane, _: f64) -> fmn::Result<RowStrips> { Ok(RowStrips { group: Mob(0), rows: Vec::new(), cols: 0 }) }
pub fn picture_cells(_: &Plane) -> (usize, usize) { (0, 0) }
pub fn portrait_colors(_: impl Fn(C) -> Option<C>, _: &Plane, _: usize, _: usize, _: f64) -> Vec<Vec<Srgb>> { Vec::new() }
pub struct Painter;
impl Painter { pub fn leave(&self, _: &mut Stage<'_>, _: f64) -> fmn::Result<()> { Ok(()) } }
pub struct Spoken;
impl Spoken {
    pub fn sentence(&self, _: usize) -> fmn::Result<f64> { Ok(0.0) }
    pub fn wait_for(&self, _: &mut Stage<'_>, _: &str) -> fmn::Result<()> { Ok(()) }
    pub fn wait_sentence(&self, _: &mut Stage<'_>, _: usize) -> fmn::Result<()> { Ok(()) }
    pub fn until(&self, _: &Stage<'_>, _: &str, _: f64) -> fmn::Result<f64> { Ok(0.0) }
}
pub fn say(_: &Kit, _: &mut Stage<'_>, _: &str) -> fmn::Result<Spoken> { Ok(Spoken) }
pub fn fade_in_all(_: &mut Stage<'_>, _: &[Mob], _: f64) -> fmn::Result<()> { Ok(()) }
pub fn fade_out_all(_: &mut Stage<'_>, _: &[Mob], _: f64) -> fmn::Result<()> { Ok(()) }
pub fn carded(_: &Kit, _: &mut Stage<'_>, _: &str, _: f64, _: Srgb, _: Vec3, _: Srgb) -> fmn::Result<[Mob; 2]> { Ok([Mob(0), Mob(0)]) }
pub fn dot(_: &mut Stage<'_>, _: Vec3, _: f64, _: Srgb) -> Mob { Mob(0) }
"""


def lift(text):
    """The items and the two scene passages, verbatim, in a compilable wrapper."""
    consts = "\n".join(
        re.search(rf"^(?:pub )?const {name}: .*;$", text, re.M).group(0)
        for name in (
            "ROOTS_CLAIM", "LEADING_KEY", "BEFORE_DIVIDING_TEX", "AFTER_DIVIDING_TEX", "EVERY_OUTPUT", "ZERO_KEPT", "SAME_ROOTS",
            "DEGREE_N_TEX", "OUT_R", "ZERO_NAME_AT", "ZERO_PLATE",
        )
    )
    items = text[text.index("/// 11_6's output squash on the unit circle") : text.index("/// How long each of 11_1's counts")]
    start = text.index('        let line = say(&kit, stage, "11_7")?;')
    stop = text.index("        fade_in_all(stage, &wide_all, 0.5)?;", start)
    scene = text[start:stop] + "        fade_in_all(stage, &wide_all, 0.5)?;\n"
    a = text.index("    let zero_at = out.p(")
    b = text.index("    let image = dot(stage", a)
    c = text.index("    // the distance line and the disk's rim both pass the fixed 0")
    d = text.index("    let carried = stage.add_value_tracker(0.0);", c)
    return f"""
{consts}
const STEP_SKETCH: [&str; 3] = ["a", "0", "reach"];
{items}
#[allow(clippy::too_many_arguments)]
fn division(
    stage: &mut Stage<'_>,
    kit: Kit,
    inp: Plane,
    out: Plane,
    one: Vec<Mob>,
    reference: Mob,
    follows: Mob,
    leash: Mob,
    squashed: Mob,
    unit: Mob,
    inside: [Mob; 2],
    roots: Vec<Mob>,
    painter: Painter,
) -> fmn::Result<()> {{
{scene}
    Ok(())
}}
fn plate(stage: &mut Stage<'_>, kit: &Kit, out: Plane, mut frame: Vec<Mob>) -> fmn::Result<Vec<Mob>> {{
{text[a:b]}
{text[c:d]}
    Ok(frame)
}}
fn main() {{}}
"""


def check(text, label, work):
    folder = os.path.join(work, label)
    os.makedirs(folder)
    with open(os.path.join(folder, "model.rs"), "w", encoding="utf-8") as out:
        out.write(open(MODEL, encoding="utf-8").read())
    with open(os.path.join(folder, "main.rs"), "w", encoding="utf-8") as out:
        out.write(STUBS + lift(text))
    done = subprocess.run(
        ["rustc", "--edition", "2024", "--emit=metadata", "-o", os.path.join(folder, "out.rmeta"), os.path.join(folder, "main.rs"), "--cap-lints", "allow"],
        capture_output=True,
        text=True,
        check=False,
    )
    errors = re.findall(r"^error(?:\[E\d+\])?: .*$", done.stderr, re.M)
    return done.returncode, errors, done.stderr.replace(folder, "<work>")


staged = open(STAGED, encoding="utf-8").read()
bad = 0
with tempfile.TemporaryDirectory(prefix="complex11-typecheck-") as work:
    code, errors, detail = check(staged, "staged", work)
    print(("PASS" if code == 0 else "FAIL") + " the lifted film code type-checks against the stubs")
    if code:
        print(detail[:3000])
        bad += 1
    for k, (what, old, new) in enumerate(
        [
            ("a mark list used after it has moved", "        acted.extend(roots);\n", "        acted.extend(roots);\n        acted.extend(roots);\n"),
            ("a tracker given a string", "tween(turning.turned, 0.0, rt)", 'tween(turning.turned, 0.0, "rt")'),
            ("one mob where a list is due", "fade_in_all(stage, &[before], 0.3)?;", "fade_in_all(stage, before, 0.3)?;"),
            ("the strips used after the updater took them", "        stage.add_to_scene(field)?;\n", "        let again = strips.group;\n        stage.add_to_scene(again)?;\n"),
            ("a closure that outlives what it borrows", "move |s, _| {\n                let t = (value(s, turned)", "|s, _| {\n                let t = (value(s, turned)"),
            ("the plate missing from the frame's type", "frame.extend([zero_plate, zero_name]);", 'frame.extend(["plate"]);'),
        ]
    ):
        assert old in staged, what
        code, errors, _ = check(staged.replace(old, new, 1), f"control{k}", work)
        caught = code != 0
        print(f"{'caught' if caught else 'MISSED'} control: {what} -> {errors[0] if errors else 'compiled'}")
        bad += 0 if caught else 1
print("RESULT " + ("PASS" if bad == 0 else f"FAIL {bad}"))
