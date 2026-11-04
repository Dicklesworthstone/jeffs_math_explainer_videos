"""Build the review's proposed fixes as one patch per finding, each against the candidate bytes.

Run from the repo root: python3 -I -B build_fixes.py <scratch dir> <output patches dir>.
Writes patches to the output dir and scratch copies under the scratch dir; touches nothing else.
Then applies each patch alone and all four together to scratch copies, and runs rustfmt on the
whole crate with everything applied. No Cargo: this shows they apply and parse, not that they compile.
"""
import pathlib
import shutil
import subprocess
import sys
import tempfile

SCRATCH, OUT = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
CRATE = "explainer_corpus/logarithm_mysteries"
STAGE = pathlib.Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-repair08-10")
CAND = STAGE / "candidate" / CRATE / "src/chapters"
REL = CRATE + "/src/chapters/"

R1 = {
    "matrix_logs.rs": [
        (
            """        assert!(CENTRE[0] - UNIT * CLIP > -6.86);
        assert!(CENTRE[0] + UNIT * CLIP < 1.0);
""",
            """        const { assert!(CENTRE[0] - UNIT * CLIP > -6.86) };
        const { assert!(CENTRE[0] + UNIT * CLIP < 1.0) };
""",
        )
    ]
}

R2 = {
    "doing_nothing.rs": [
        (
            """    model::ellipse_generator(a, b, tilt.to_radians())
}
""",
            """    model::ellipse_generator(a, b, tilt.to_radians())
}

/// Three orbits of the 10_2 family's member with half-axes `a` and 1, tilted 15°.
fn family_orbits(s: &mut Arena, a: f64, c: Srgb) -> Mob {
    let x = generator(a, 1.0, 15.0);
    let g = s.add(VMobject::new());
    for r in [0.6, 1.2, 1.8] {
        let pts: Vec<Vec3> = (0..=80)
            .map(|k| model::apply(model::flow(x, f64::from(k) / 80.0), [r, 0.0]))
            .map(screen)
            .collect();
        let m = s.add(polyline(pts).style(line_style(c, 2.5)).build());
        let _ = s.attach(g, m);
    }
    g
}
""",
        ),
        (
            """        let family = stage.always_redraw(move |s: &mut Arena| {
            let a = value(s, stretch);
            let x = generator(a, 1.0, 15.0);
            let g = s.add(VMobject::new());
            for r in [0.6, 1.2, 1.8] {
                let pts: Vec<Vec3> = (0..=80)
                    .map(|k| model::apply(model::flow(x, f64::from(k) / 80.0), [r, 0.0]))
                    .map(screen)
                    .collect();
                let m = s.add(polyline(pts).style(line_style(gold, 2.5)).build());
                let _ = s.attach(g, m);
            }
            g
        });
        stage.add_to_scene(family)?;
        let family_word""",
            """        let resting_family = family_orbits(stage.arena_mut(), 1.0, gold);
        let family_word""",
        ),
        (
            """        fade_in_all(stage, &[family_word], 0.4)?;
""",
            """        fade_in_all(stage, &[resting_family, family_word], 0.4)?;
        stage.scene_mut().remove(&[resting_family]);
        let family = stage.always_redraw(move |s: &mut Arena| {
            let a = value(s, stretch);
            family_orbits(s, a, gold)
        });
        stage.add_to_scene(family)?;
""",
        ),
    ]
}

R3 = {
    "draw.rs": [
        (
            """#[cfg(test)]
mod tests {
    use super::*;
""",
            """/// Run `f` as a scene's `construct`, headless, at 10 fps.
#[cfg(test)]
pub fn probe_scene(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
    use fmn::scene::{NullSceneSink, RuntimeConfig};
    struct Probe<F>(F);
    impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            (self.0)(stage)
        }
    }
    let config = RuntimeConfig {
        fps: 10,
        ..RuntimeConfig::default()
    };
    fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink).expect("the probe scene runs");
}

#[cfg(test)]
mod tests {
    use super::*;
""",
        ),
        (
            """        assert!(!inside_clip([0.0, 3.001], 3.0));
    }
""",
            """        assert!(!inside_clip([0.0, 3.001], 3.0));
    }

    #[test]
    fn real_flows_keep_the_box_and_the_stretch_lines_when_they_come_home() {
        use crate::model::{self, J};
        let home = model::flow(model::mscale(J, TAU), 1.0);
        let stretch = model::flow([[1.5f64.ln(), 0.0], [0.0, 0.6f64.ln()]], 1.0);
        for k in -36..=36 {
            let t = f64::from(k) / 12.0;
            for p in [[3.0, t], [-3.0, t], [t, 3.0], [t, -3.0]] {
                assert!(inside_clip(model::apply(home, p), 3.0), "{p:?}");
            }
            for p in [[2.0, t], [-2.0, t]] {
                assert!(inside_clip(model::apply(stretch, p), 3.0), "{p:?}");
            }
        }
    }

    #[test]
    fn a_real_card_sets_its_brackets_from_its_own_typeset_entries() {
        probe_scene(|stage| {
            let kit = Kit::new()?;
            let ink = Palette::ink();
            for (entries, size) in [
                ([["0.5", "-0.866"], ["0.866", "0.5"]], 34.0),
                ([["0.905", "-0.97"], ["0.776", "0.517"]], 30.0),
            ] {
                let mut widest: f64 = 0.0;
                for e in entries.iter().flatten() {
                    let alone = kit.text(stage, &shown(e), size, ink, ORIGIN)?;
                    widest = widest.max(stage.get_width(alone));
                }
                let (_, w) = card_widths(size, widest);
                let card = matrix_card(&kit, stage, entries, ORIGIN, size, ink)?;
                // the brackets are the card's outer edge, at ±w
                let drawn = stage.get_width(card);
                assert!((drawn - 2.0 * w).abs() < 1e-3, "{drawn} against {}", 2.0 * w);
            }
            Ok(())
        });
    }
""",
        ),
    ],
    "matrix_logs.rs": [
        (
            """const DICTIONARY_X: f64 = 4.95;
""",
            """const DICTIONARY_X: f64 = 4.95;
const DICTIONARY: &str = "turn-and-scale matrix ↔ complex number";
""",
        ),
        (
            """        let dictionary = kit.text(
            stage,
            "turn-and-scale matrix ↔ complex number",
            20.0,
            teal,
            v(DICTIONARY_X, 0.05),
        )?;
""",
            """        let dictionary = kit.text(stage, DICTIONARY, 20.0, teal, v(DICTIONARY_X, 0.05))?;
""",
        ),
        (
            """        // the line's width, from the lint's span on preview02: 3.54 to 7.26
        let half = (7.26 - 3.54) / 2.0;
        assert!(DICTIONARY_X + half < 6.86);
        assert!(DICTIONARY_X - half > SMALL[2][0] + 2.0 * SMALL_UNIT + 0.3);
""",
            """        super::super::draw::probe_scene(|stage| {
            let kit = Kit::new()?;
            let line = kit.text(stage, DICTIONARY, 20.0, Palette::ink(), v(DICTIONARY_X, 0.05))?;
            assert!(stage.get_bounding_box_point(line, RIGHT)[0] < 6.86);
            assert!(stage.get_bounding_box_point(line, LEFT)[0] > SMALL[2][0] + 2.0 * SMALL_UNIT + 0.3);
            Ok(())
        });
""",
        ),
    ],
    "doing_nothing.rs": [
        (
            """const LAP_ASIDE: f64 = 0.15;
""",
            """const LAP_ASIDE: f64 = 0.15;
const RULE_WORD: &str = "keep −π < turning < π";
const LAP_WORD: &str = "ellipse flows: ±2π";
""",
        ),
        (
            """            "ellipse flows: ±2π",
            20.0,
""",
            """            LAP_WORD,
            20.0,
""",
        ),
        (
            """        let rule = kit.text(
            stage,
            "keep −π < turning < π",
            20.0,
            green,
            v(TURN_X - RULE_BACK, TURN_Y - 0.7),
        )?;
""",
            """        let rule = kit.text(stage, RULE_WORD, 20.0, green, v(TURN_X - RULE_BACK, TURN_Y - 0.7))?;
""",
        ),
        (
            """        // widths from preview02 frame 170: the rule 3.10 to 5.14, the label 5.03 to 6.56
        let (rule_w, lap_w) = (5.14 - 3.10, 6.56 - 5.03);
        let rule_right = TURN_X - RULE_BACK + rule_w / 2.0;
        let lap_at = turn_at(TAU)[0] + LAP_ASIDE;
        assert!(lap_at - lap_w / 2.0 - rule_right > 0.25);
        assert!(lap_at + lap_w / 2.0 < 6.86);
""",
            """        super::super::draw::probe_scene(|stage| {
            let kit = Kit::new()?;
            let y = TURN_Y - 0.7;
            let rule = kit.text(stage, RULE_WORD, 20.0, Palette::ink(), v(TURN_X - RULE_BACK, y))?;
            let lap = kit.text(stage, LAP_WORD, 20.0, Palette::ink(), v(turn_at(TAU)[0] + LAP_ASIDE, y))?;
            let rule_right = stage.get_bounding_box_point(rule, RIGHT)[0];
            let lap_left = stage.get_bounding_box_point(lap, LEFT)[0];
            assert!(lap_left - rule_right > 0.25, "gap {}", lap_left - rule_right);
            assert!(stage.get_bounding_box_point(lap, RIGHT)[0] < 6.86);
            Ok(())
        });
""",
        ),
    ],
}

R4 = {
    "doing_nothing.rs": [
        (
            "// the turning line, in units of π, with the window −π < angle < π\n",
            "// the turning line, in units of π, with a number's window −π < angle ≤ π\n",
        )
    ]
}

FINDINGS = {"r1-const-asserts": R1, "r2-family-fade": R2, "r3-real-box-tests": R3, "r4-window-comment": R4}


def edit(text, pairs, name):
    for old, new in pairs:
        n = text.count(old)
        assert n == 1, (name, n, old[:60])
        text = text.replace(old, new)
    return text


OUT.mkdir(parents=True, exist_ok=True)
base = pathlib.Path(tempfile.mkdtemp(prefix="fixes", dir=SCRATCH))
for tag, finding in FINDINGS.items():
    root = base / tag
    for side in "ab":
        (root / side / REL).mkdir(parents=True)
    pieces = []
    for name, pairs in finding.items():
        before = (CAND / name).read_text()
        (root / "a" / REL / name).write_text(before)
        (root / "b" / REL / name).write_text(edit(before, pairs, f"{tag}:{name}"))
        d = subprocess.run(["diff", "-u", f"a/{REL}{name}", f"b/{REL}{name}"], cwd=root, capture_output=True, text=True)
        assert d.returncode == 1, d.stderr
        lines = d.stdout.splitlines(keepends=True)
        # no timestamps in the headers, so the patch's hash does not depend on when it was built
        lines[0] = f"--- a/{REL}{name}\n"
        lines[1] = f"+++ b/{REL}{name}\n"
        pieces.append("".join(lines))
    (OUT / f"{tag}.patch").write_text("".join(pieces))


def fresh():
    work = pathlib.Path(tempfile.mkdtemp(prefix="apply", dir=base))
    (work / CRATE).mkdir(parents=True)
    shutil.copytree(STAGE / "candidate" / CRATE / "src", work / CRATE / "src")
    # the rest of the crate's source, for a whole-crate parse, comes from canonical
    for src in (pathlib.Path(CRATE) / "src").rglob("*.rs"):
        rel = src.relative_to(CRATE)
        dest = work / CRATE / rel
        if not dest.exists():
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dest)
    return work


def apply(work, tag):
    p = subprocess.run(["patch", "-p1", "--no-backup-if-mismatch", "-i", str((OUT / f"{tag}.patch").resolve())], cwd=work, capture_output=True, text=True)
    bad = [l for l in p.stdout.splitlines() if "fuzz" in l or "FAILED" in l]
    return p.returncode, bad


for tag in FINDINGS:
    code, bad = apply(fresh(), tag)
    print(f"{tag}: alone, exit {code}, fuzz or reject {bad}")
for order in (list(FINDINGS), list(reversed(list(FINDINGS)))):
    work = fresh()
    results = [apply(work, tag) for tag in order]
    print("together, order", [t[:2] for t in order], "exits", [r[0] for r in results], "fuzz or reject", [r[1] for r in results if r[1]])
fmt = subprocess.run(["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", str(work / CRATE / "src/main.rs")], capture_output=True, text=True)
print("rustfmt whole crate, all four applied: parse errors:", [l for l in fmt.stderr.splitlines() if l.startswith("error")])
print("  files with diffs:", sorted({l.split(":")[0].split("/")[-1] for l in fmt.stdout.splitlines() if l.startswith("Diff in")}))
for l in fmt.stdout.splitlines():
    if l.startswith("Diff in") and any(n in l for n in ("draw.rs", "matrix_logs.rs", "doing_nothing.rs")):
        print("  ", l.split("/")[-1])
if any(n in fmt.stdout for n in ("draw.rs", "matrix_logs.rs", "doing_nothing.rs")):
    print(fmt.stdout[:3000].replace(str(work), "WORK"))
