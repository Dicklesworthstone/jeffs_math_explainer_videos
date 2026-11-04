"""Build the staged candidates from the canonical chapter sources.

Run from the repo root:  /usr/bin/python3 -I -B <this file>

Reads each canonical file, refuses if its hash moved, keeps a byte copy under
baseline/, applies literal replacements that must each match exactly once,
writes candidate/ and patches/, round-trips each patch with patch(1) in a
temporary directory, and runs rustfmt --check on the candidate leaf file.
Nothing outside this review's folder is written. It compiles nothing: text
checks cannot see types, borrows or lifetimes.
"""

import difflib
import hashlib
import pathlib
import shutil
import subprocess
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
CH = "explainer_corpus/logarithm_mysteries/src/chapters"

PINS = {
    "undoing_growth.rs": "8e26f3e4ae1e12b65bce5441e260f7beaf5d6fc0ab45b847f7a994f660f5ed36",
    "complex_turns.rs": "d5f202afb97debcef1a3f7abd872634b7cc7093108e87975167b04b065e212d5",
    "lifting_loops.rs": "78af9acdd126c12035ec865280b91b303921c228cab7f79023987fb90ec228c6",
}

# (finding, old, new): each old string must occur exactly once in the baseline.
EDITS = {
    "undoing_growth.rs": [
        (
            "S1 const",
            '''const HI: f64 = 3.0;
''',
            '''const HI: f64 = 3.0;
/// The mirror's name in graph units: its left edge, just right of the line's
/// upper end, and its height.
const LINE_NAME_AT: (f64, f64) = (3.05, 2.8);
''',
        ),
        (
            "S1 name",
            '''        let line_yx = dashed(stage, p(LO, LO), p(HI, HI), Palette::dim(), 2.0, 26)?;
        fade_in_all(stage, &[line_yx], 0.5)?;
''',
            '''        let line_yx = dashed(stage, p(LO, LO), p(HI, HI), Palette::dim(), 2.0, 26)?;
        let line_name = kit.tex_left(
            stage,
            r"y = x",
            24.0,
            Palette::dim(),
            &[],
            p(LINE_NAME_AT.0, LINE_NAME_AT.1),
        )?;
        fade_in_all(stage, &[line_yx, line_name], 0.5)?;
''',
        ),
        (
            "S1 leaves with the graph",
            '''            curve, line_yx, ln_name, swapped, slope_rule, one, tangent1, one_word, small, natural, why, wall, barred,
            no_log,
''',
            '''            curve, line_yx, line_name, ln_name, swapped, slope_rule, one, tangent1, one_word, small, natural, why,
            wall, barred, no_log,
''',
        ),
        (
            "S1 test",
            '''        // "run" sits above the level leg and under y = ln x
        let run_y = TRI_A + TRI_K + 0.17;
        assert!(1.7 - 0.17 - (run_y + 0.09) >= 0.05, "{run_y}");
''',
            '''        // "run" sits above the level leg and under y = ln x
        let run_y = TRI_A + TRI_K + 0.17;
        assert!(1.7 - 0.17 - (run_y + 0.09) >= 0.05, "{run_y}");
        // the mirror's name, about 0.6 wide and 0.25 tall, stands right of the line
        // at the height of its top, and short of "rise and run trade places" at 1.0
        const { assert!(LINE_NAME_AT.0 > LINE_NAME_AT.1 + 0.125 + 0.1) };
        const { assert!(OX + LINE_NAME_AT.0 + 0.6 < 1.0 - 0.1) };
''',
        ),
    ],
    "complex_turns.rs": [
        (
            "C1 constants",
            '''/// How much further out each lap of the wound trace is drawn (drawing aid).
const LAP_SPREAD: f64 = 0.045;
''',
            '''/// How much further out each lap of the wound trace is drawn (drawing aid):
/// wide enough that three laps stay three strokes.
const LAP_SPREAD: f64 = 0.06;
/// Where the climb of 05_3 stops: three whole laps and a quarter, far past the
/// top of the strip, so "again and again" is counted on screen, and between
/// whole laps, where rounding cannot read the count one short.
const CLIMB_END: f64 = 6.5 * PI;
/// The note beside the arrow that carries the climb on past the strip.
const GOES_ON: &str = "the line goes on";
''',
        ),
        (
            "C7 constant",
            '''const EACH: &str = r"e^{(\\pi + 2\\pi k) i} = -1,\\ \\ k \\text{ whole}";
''',
            '''const EACH: &str = r"e^{(\\pi + 2\\pi k) i} = -1,\\ \\ k \\text{ whole}";
/// Its place: clear of the strip on the left, and on the right short of the
/// output axis, which would otherwise run under "whole".
const EACH_AT: [f64; 2] = [IN.centre[0] + 2.05, -0.1];
''',
        ),
        (
            "C1 input dot waits on the strip",
            '''                Dot::new()
                    .point(IN.p(C::new(0.0, y)))
                    .radius(0.08)
''',
            '''                Dot::new()
                    // past the top of the strip the dot waits at its edge: the arrow
                    // above it says the climb goes on, and the output keeps winding
                    .point(IN.p(C::new(0.0, y.min(IN_Y))))
                    .radius(0.08)
''',
        ),
        (
            "C1 the climb goes on and the count moves",
            '''        fade_in_all(stage, &[laps_word, aid], 0.4)?;
        play!(stage; tween(climb, 3.0 * PI, 2.2)?);
''',
            '''        let strip_top = IN.p(C::new(0.0, IN_Y));
        let goes_on = super::draw::arrow(
            stage,
            v(strip_top[0], strip_top[1] + 0.12),
            v(strip_top[0], strip_top[1] + 0.48),
            blue,
            3.0,
        )?;
        let goes_on_word = kit.text_left(stage, GOES_ON, 18.0, blue, v(strip_top[0] + 0.2, strip_top[1] + 0.3))?;
        fade_in_all(stage, &[laps_word, aid, goes_on, goes_on_word], 0.4)?;
        play!(stage; tween(climb, CLIMB_END, 2.6)?);
''',
        ),
        (
            "C2 the climb leaves when the voice moves on",
            '''        kit.say(stage, "05_4")?;
        fade_out_all(stage, &[closes, h_line, h_dot], 0.3)?;
''',
            '''        kit.say(stage, "05_4")?;
        // the climb is over: its dot would stand away from the 1 the voice now names,
        // and the input dot would sit under the arrow to 4πi
        fade_out_all(
            stage,
            &[
                closes,
                h_line,
                h_dot,
                climber,
                laps,
                laps_word,
                aid,
                goes_on,
                goes_on_word,
            ],
            0.3,
        )?;
''',
        ),
        (
            "C2 not faded twice",
            '''        tidy.extend([
            bracket, lap_word, one_dot, one_word, climber, laps, laps_word, aid, v_line,
        ]);
''',
            '''        tidy.extend([bracket, lap_word, one_dot, one_word, v_line]);
''',
        ),
        (
            "S3 spoken order",
            '''        let column = fact_list("complex.minus_one_logs_pi")?;
        let mut col = Vec::new();
''',
            '''        // in the order the narration names them: πi, 3πi, −πi, then −3πi
        let mut column = fact_list("complex.minus_one_logs_pi")?;
        column.sort_by_key(|&k| (k < 0.0, k.abs() as i64));
        let mut col = Vec::new();
''',
        ),
        (
            "C7 scene",
            '''        let each = kit.tex(stage, EACH, 30.0, gold, &[], v(IN.centre[0] + 2.6, -0.1))?;
''',
            '''        let each = kit.tex(stage, EACH, 30.0, gold, &[], v(EACH_AT[0], EACH_AT[1]))?;
''',
        ),
        (
            "C7 test",
            '''            let each = kit.tex(stage, EACH, 30.0, Palette::ink(), &[], v(IN.centre[0] + 2.6, -0.1))?;
            let [left, right, _, _] = box_of(stage, each);
            assert!(left > IN.p(C::new(IN_X.1, 0.0))[0] + 0.05, "the rule reaches the strip");
            let outer = OUT.centre[0] - OUT.scale * XS[XS.len() - 1].exp();
            assert!(right < outer - 0.05, "the rule reaches the outer ring");
''',
            '''            let each = kit.tex(stage, EACH, 30.0, Palette::ink(), &[], v(EACH_AT[0], EACH_AT[1]))?;
            let [left, right, _, _] = box_of(stage, each);
            assert!(left > IN.p(C::new(IN_X.1, 0.0))[0] + 0.05, "the rule reaches the strip");
            let outer = OUT.centre[0] - OUT.scale * XS[XS.len() - 1].exp();
            assert!(right < outer - 0.05, "the rule reaches the outer ring");
            assert!(
                right < OUT.centre[0] - OUT.half_w - 0.03,
                "the output axis runs under the rule"
            );
''',
        ),
        (
            "C1 tests",
            '''        assert!((r - (1.0 + LAP_SPREAD * 1.5)).abs() < 1e-9);
    }
''',
            '''        assert!((r - (1.0 + LAP_SPREAD * 1.5)).abs() < 1e-9);
    }

    #[test]
    fn the_count_moves_on_screen_and_the_wound_trace_stays_inside_the_next_circle() {
        // the count appears at one lap and has to move: "again and again"
        assert_eq!((CLIMB_END / TAU).floor(), 3.0);
        // it stops between whole laps, where rounding cannot read it short
        let past = (CLIMB_END / TAU).fract();
        assert!(past > 0.2 && past < 0.8, "{past}");
        // the input runs off the strip, so its dot waits at the top edge
        const { assert!(CLIMB_END > IN_Y) };
        let t = wound(0.0, 0.0, CLIMB_END);
        let furthest = t
            .iter()
            .map(|q| (q[0] - OUT.centre[0]).hypot(q[1] - OUT.centre[1]))
            .fold(0.0, f64::max);
        assert!(furthest < OUT.scale * 0.5f64.exp() - 0.1, "{furthest}");
    }

    #[test]
    fn the_note_past_the_strip_clears_the_strip_and_the_title() {
        run(|stage| {
            let kit = Kit::new()?;
            let top = IN.p(C::new(0.0, IN_Y));
            let word = kit.text_left(stage, GOES_ON, 18.0, Palette::ink(), v(top[0] + 0.2, top[1] + 0.3))?;
            let [left, right, bottom, above] = box_of(stage, word);
            assert!(left > top[0] + 0.1, "the arrow reaches the note");
            assert!(bottom > top[1] + 0.05, "the strip's top line reaches the note");
            // the title's left end stands at about -2.2 in the rendered frames
            assert!(right < -2.4 && above < SAFE[1], "the note leaves its place");
            Ok(())
        });
    }
''',
        ),
    ],
    "lifting_loops.rs": [
        (
            "C3 the axis keeps inside the safe area",
            '''    scale: 1.25,
    half_w: 2.9,
    half_h: 2.7,
};
/// The logarithm's plane, tall: zero near the bottom so two laps fit.
''',
            '''    scale: 1.25,
    // the axis's left end stays inside the safe area, at -6.90
    half_w: 2.8,
    half_h: 2.7,
};
/// The logarithm's plane, tall: zero near the bottom so two laps fit.
''',
        ),
        (
            "C4 formula",
            '''const DIFFER: &str = r"e^{L(z)} = z = e^{\\ell(z)} \\;\\Rightarrow\\; L(z) - \\ell(z) = 2\\pi i\\,k,\\ \\ k \\text{ whole}";
''',
            '''/// Ours carries no `(z)`: it is the value carried along the walk, and back at 1
/// it is not the value it began with. Only L is a function of the point.
const DIFFER: &str = r"e^{L(z)} = z = e^{\\ell} \\;\\Rightarrow\\; L(z) - \\ell = 2\\pi i\\,k,\\ \\ k \\text{ whole}";
''',
        ),
        (
            "C6b note under the circle it names, and the point 0",
            '''/// "0 is not inside", right of the number's own axis and under both circles.
fn outside_word(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.text_left(
        stage,
        "0 is not inside",
        20.0,
        slide_c(),
        v(Z.centre[0] + 0.15, Z.centre[1] - 1.6),
    )
}
''',
            '''/// "0 is not inside", under the side circle it is about: under the unit circle
/// it would read as a claim about that one, which does hold 0.
fn outside_word(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.text_left(
        stage,
        "0 is not inside",
        20.0,
        slide_c(),
        v(Z.centre[0] + 1.5, Z.centre[1] - 1.6),
    )
}

/// The point 0 itself, which every loop is now measured against: a small dot and
/// its name, inside the unit circle and clear of both axes. It stays to the end,
/// because the closing theorem is about loops that wind around it.
fn zero_mark(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Mob, Mob)> {
    let at = Z.p(C::new(0.0, 0.0));
    let mark = dot(stage, at, 0.06, Palette::ink());
    let word = kit.text(stage, "0", 20.0, Palette::ink(), v(at[0] - 0.2, at[1] - 0.22))?;
    Ok((mark, word))
}
''',
        ),
        (
            "C4 colours",
            '''        &[(r"L(z)", slide_c()), (r"\\ell(z)", position_c())],
''',
            '''        &[(r"L(z)", slide_c()), (r"\\ell", position_c())],
''',
        ),
        (
            "S4 the number turns under the words",
            '''        fade_in_all(stage, &[straight], 0.4)?;
        kit.until_sentence(stage, "06_2", 1)?;
''',
            '''        fade_in_all(stage, &[straight], 0.4)?;
        // "as the number turns": it keeps turning under the words, short of the quarter
        play!(stage; tween(turn, 0.2, 2.4)?);
        kit.until_sentence(stage, "06_2", 1)?;
''',
        ),
        (
            "S6 the answer clears its ring",
            '''        let answer = kit.text_left(stage, "2πi", 26.0, gold, v(top[0] + 0.25, top[1]))?;
''',
            '''        let answer = kit.text_left(stage, "2πi", 26.0, gold, v(top[0] + 0.3, top[1]))?;
''',
        ),
        (
            "C6b scene in",
            '''        swap(stage, &[one], &[side_circle, outside], 0.4)?;
''',
            '''        let (zero_dot, zero_word) = zero_mark(&kit, stage)?;
        swap(stage, &[one], &[side_circle, outside, zero_dot, zero_word], 0.4)?;
''',
        ),
        (
            "C4 name beside ours",
            '''        let differ = differ_line(&kit, stage)?;
        fade_in_all(stage, &[differ], 0.5)?;
''',
            '''        let differ = differ_line(&kit, stage)?;
        // the formula's letter for the carried value, where "ours" is written
        let ours_right = stage.get_bounding_box_point(ours, RIGHT)[0];
        let ours_name = kit.tex_left(stage, r"(\\ell)", 22.0, blue, &[], v(ours_right + 0.12, 1.95))?;
        fade_in_all(stage, &[differ, ours_name], 0.5)?;
''',
        ),
        (
            "S5 fixed after the walk that shows it",
            '''        fade_in_all(stage, &[jump], 0.4)?;
        let fixed = kit.text(stage, "fixed", 22.0, teal, v(GAUGE_X - 0.8, rung(0.0)[1]))?;
        fade_in_all(stage, &[fixed], 0.3)?;
        play!(stage; tween(turn, 0.97, 4.0)?);
''',
            '''        fade_in_all(stage, &[jump], 0.4)?;
        play!(stage; tween(turn, 0.97, 4.0)?);
        // the sentence's last words: it lands after the walk has shown it
        let fixed = kit.text(stage, "fixed", 22.0, teal, v(GAUGE_X - 0.8, rung(0.0)[1]))?;
        fade_in_all(stage, &[fixed], 0.3)?;
''',
        ),
        (
            "C6b test",
            '''            let [left, _, _, top] = box_of(stage, word);
            assert!(left > Z.centre[0] + 0.05, "the number's axis crosses the note");
            let side = fact_list("loop.side")?;
''',
            '''            let [left, right, _, top] = box_of(stage, word);
            assert!(left > Z.centre[0] + 0.05, "the number's axis crosses the note");
            let side = fact_list("loop.side")?;
            let under = Z.p(C::new(side[0], 0.0))[0];
            assert!(((left + right) / 2.0 - under).abs() < 0.1, "the note leaves its circle");
''',
        ),
        (
            "C3 C4 C6b tests",
            '''    #[test]
    fn a_trail_too_short_to_draw_is_shorter_than_its_own_stroke() {
''',
            '''    #[test]
    fn zero_is_marked_inside_the_unit_circle_clear_of_both_axes() {
        run(|stage| {
            let kit = Kit::new()?;
            let (_, word) = zero_mark(&kit, stage)?;
            let [left, right, bottom, top] = box_of(stage, word);
            assert!(right < Z.centre[0] - 0.03, "the upright axis reaches the name");
            assert!(top < Z.centre[1] - 0.03, "the level axis reaches the name");
            let far = (left - Z.centre[0]).hypot(bottom - Z.centre[1]);
            assert!(far < Z.scale - 0.1, "the unit circle reaches the name");
            Ok(())
        });
    }

    #[test]
    fn the_formula_gives_ours_no_argument_because_ours_is_not_a_function_of_the_point() {
        assert!(DIFFER.contains(r"L(z) - \\ell ="));
        assert!(!DIFFER.contains(r"\\ell("));
        // back at 1 the carried value is one lap above where it began
        let lift = unit_lift(1.0);
        let (first, last) = (lift[0], lift[lift.len() - 1]);
        assert!((first.re - last.re).abs() < 1e-9);
        assert!((last.im - first.im - TAU).abs() < 1e-9);
    }

    #[test]
    fn the_numbers_axis_stays_inside_the_safe_area() {
        const { assert!(Z.centre[0] - Z.half_w > -SAFE[0]) };
    }

    #[test]
    fn a_trail_too_short_to_draw_is_shorter_than_its_own_stroke() {
''',
        ),
    ],
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    out = []
    for sub in ("baseline", "candidate", "patches"):
        (ROOT / sub).mkdir(exist_ok=True)
    for index, (name, pin) in enumerate(PINS.items(), start=4):
        canon = pathlib.Path(CH) / name
        data = canon.read_bytes()
        if sha(data) != pin:
            raise SystemExit(f"{name}: canonical moved to {sha(data)}")
        (ROOT / "baseline" / name).write_bytes(data)
        text = data.decode("utf-8")
        for label, old, new in EDITS[name]:
            count = text.count(old)
            if count != 1:
                raise SystemExit(f"{name}: {label!r} matches {count} times")
            text = text.replace(old, new)
        cand = ROOT / "candidate" / name
        cand.write_bytes(text.encode("utf-8"))
        rel = f"{CH}/{name}"
        patch = "".join(
            difflib.unified_diff(
                data.decode("utf-8").splitlines(True),
                text.splitlines(True),
                f"a/{rel}",
                f"b/{rel}",
                n=3,
            )
        )
        pfile = ROOT / "patches" / f"{index:02d}-{name.removesuffix('.rs') if hasattr(name, 'removesuffix') else name[:-3]}.patch"
        pfile.write_text(patch, encoding="utf-8")
        with tempfile.TemporaryDirectory() as tmp:
            work = pathlib.Path(tmp)
            target = work / rel
            target.parent.mkdir(parents=True)
            shutil.copyfile(canon, target)
            done = subprocess.run(
                ["patch", "-p1", "-s", "-i", str(pfile.resolve())], cwd=work, capture_output=True, text=True
            )
            same = target.read_bytes() == cand.read_bytes()
        fmt = subprocess.run(
            ["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", str(cand)],
            capture_output=True,
            text=True,
        )
        added = sum(1 for line in patch.splitlines() if line.startswith("+") and not line.startswith("+++"))
        removed = sum(1 for line in patch.splitlines() if line.startswith("-") and not line.startswith("---"))
        out.append(
            f"{name}: {len(EDITS[name])} edits, +{added} -{removed}; baseline {sha(data)}; "
            f"candidate {sha(cand.read_bytes())}; patch {sha(pfile.read_bytes())}; "
            f"patch exit {done.returncode}, round trip equal {same}; rustfmt --check exit {fmt.returncode}"
        )
        if fmt.returncode:
            out.append(fmt.stdout + fmt.stderr)
    report = "\n".join(out) + "\n"
    (HERE / "build_candidates.out").write_text(report, encoding="utf-8")
    print(report)


main()
