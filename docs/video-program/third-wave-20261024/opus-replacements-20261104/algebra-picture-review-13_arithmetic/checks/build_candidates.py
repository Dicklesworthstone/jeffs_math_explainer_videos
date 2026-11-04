"""Build the two stacked candidates of arithmetic.rs from canonical, in scratch only. Run from the repo root.

Each edit is an exact replacement that must match exactly once, so a moved canonical fails loudly.
Layer A: what is wrong or early on screen. Layer B, on top of A: teaching pictures and sentence sync.
"""
import pathlib
import sys

OUT = pathlib.Path(sys.argv[1])
CRATE = pathlib.Path("explainer_corpus/abstract_algebra_expanded")
base = (CRATE / "src/arithmetic.rs").read_text()
oracle = (CRATE / "oracle.py").read_text()


def sub(text, old, new):
    n = text.count(old)
    if n != 1:
        raise SystemExit(f"expected exactly one match, found {n}:\n{old[:200]}")
    return text.replace(old, new)


# ---------------------------------------------------------------- layer A
A = base

A = sub(A, """fn grouping(stage: &mut Stage<'_>, left: f64, right: f64, y: f64) -> fmn::Result<Mob> {""",
"""/// Run a move and rename the tile. The old name leaves first, so no frame shows
/// a new pose above the old name. It takes the same 3 s as `advance` then `replace`.
fn step(
    stage: &mut Stage<'_>,
    kit: &Kit,
    tracker: Mob,
    to: f64,
    old: Mob,
    text: &str,
    at: (f64, f64),
) -> fmn::Result<Mob> {
    fade(stage, &[old], false)?;
    advance(stage, tracker, to)?;
    caption(stage, kit, text, at.0, at.1)
}
fn grouping(stage: &mut Stage<'_>, left: f64, right: f64, y: f64) -> fmn::Result<Mob> {""")

A = sub(A, """        let left_objects = tile_actor::add_tile(stage, &kit, left, -3.1, 0., 1.25, |t| pose(&LEFT, t))?;
        let right_objects = tile_actor::add_tile(stage, &kit, right, 3.1, 0., 1.25, |t| pose(&RIGHT, t))?;
        let mut left_label = caption(stage, &kit, "e", -3.1, -2.)?;
        let mut right_label = caption(stage, &kit, "e", 3.1, -2.)?;
        kit.say(stage, "13_2")?;
        // The new symbols first name actions we can actually see.
        // Separate demonstration actors leave the worked-calculation histories intact.
        fade(stage, &left_objects, false)?;
        fade(stage, &right_objects, false)?;
        fade(stage, &[left_label, right_label], false)?;
        let demo_turn = stage.add_value_tracker(0.);
""", """        // Separate demonstration actors leave the worked-calculation histories intact.
        let demo_turn = stage.add_value_tracker(0.);
""")

A = sub(A, """        let demo_r = caption(stage, &kit, "r", -3.1, -2.)?;
        advance(stage, demo_turn, 1.)?;
        kit.until_sentence(stage, "13_2", 1)?;
        let mut demo_s = caption(stage, &kit, "s", 3.1, -2.)?;
""", """        kit.say(stage, "13_2")?;
        // The new symbols first name actions we can actually see.
        advance(stage, demo_turn, 1.)?;
        let demo_r = caption(stage, &kit, "r", LEFT_LABEL.0, LEFT_LABEL.1)?;
        kit.until_sentence(stage, "13_2", 1)?;
        let mut demo_s = caption(stage, &kit, "s", RIGHT_LABEL.0, RIGHT_LABEL.1)?;
""")

A = sub(A, """        advance(stage, demo_flip, 2.)?;
        demo_s = replace(stage, &kit, demo_s, "r s", 3.1, -2.)?;
        kit.hold(stage)?;
        fade(stage, &demo_left, false)?;
        fade(stage, &demo_right, false)?;
        fade(stage, &[demo_r, demo_s], false)?;
        let left_objects = tile_actor::add_tile(stage, &kit, left, -3.1, 0., 1.25, |t| pose(&LEFT, t))?;
        let right_objects = tile_actor::add_tile(stage, &kit, right, 3.1, 0., 1.25, |t| pose(&RIGHT, t))?;
        left_label = caption(stage, &kit, "e", -3.1, -2.)?;
        right_label = caption(stage, &kit, "e", 3.1, -2.)?;
        kit.say(stage, "13_3")?;
        for (index, text) in ["r", "r²", "r³", "r⁴ = e"].into_iter().enumerate() {
            advance(stage, left, (index + 1) as f64)?;
            left_label = replace(stage, &kit, left_label, text, -3.1, -2.)?;
            stage.wait(1.)?;
        }
        kit.say(stage, "13_4")?;
        advance(stage, left, 5.)?;
        left_label = replace(stage, &kit, left_label, "s", -3.1, -2.)?;
        stage.wait(1.)?;
        advance(stage, left, 6.)?;
        left_label = replace(stage, &kit, left_label, "s² = e", -3.1, -2.)?;
        kit.say(stage, "13_5")?;
        advance(stage, right, 1.)?;
        right_label = replace(stage, &kit, right_label, "r", 3.1, -2.)?;
        advance(stage, right, 2.)?;
        right_label = replace(stage, &kit, right_label, "s r", 3.1, -2.)?;
        advance(stage, left, 7.)?;
        left_label = replace(stage, &kit, left_label, "s", -3.1, -2.)?;
        advance(stage, left, 8.)?;
        left_label = replace(stage, &kit, left_label, "r s", -3.1, -2.)?;
        kit.say(stage, "13_6")?;
        advance(stage, left, 9.)?;
        left_label = replace(stage, &kit, left_label, "s r s", -3.1, -2.)?;
        // Undo the right history visibly; then apply the single inverse turn.
        for (step, text) in [(3., "r"), (4., "e"), (5., "r⁻¹")] {
            advance(stage, right, step)?;
            right_label = replace(stage, &kit, right_label, text, 3.1, -2.)?;
        }
        let relation = caption(stage, &kit, "s r s = r⁻¹", 0., -2.8)?;
        kit.say(stage, "13_7")?;
        fade(stage, &left_objects, false)?;
        fade(stage, &right_objects, false)?;
        fade(stage, &[left_label, right_label, relation], false)?;
""", """        demo_s = step(stage, &kit, demo_flip, 2., demo_s, "r s", RIGHT_LABEL)?;
        kit.hold(stage)?;
        let mut leaving = demo_left;
        leaving.extend(demo_right);
        leaving.extend([demo_r, demo_s]);
        fade(stage, &leaving, false)?;
        let left_objects = tile_actor::add_tile(stage, &kit, left, -3.1, 0., 1.25, |t| pose(&LEFT, t))?;
        let right_objects = tile_actor::add_tile(stage, &kit, right, 3.1, 0., 1.25, |t| pose(&RIGHT, t))?;
        // `e` is not defined until 13_3's last sentence; until then this pose is "start".
        let mut left_label = caption(stage, &kit, "start", LEFT_LABEL.0, LEFT_LABEL.1)?;
        let mut right_label = caption(stage, &kit, "start", RIGHT_LABEL.0, RIGHT_LABEL.1)?;
        kit.say(stage, "13_3")?;
        for (index, text) in ["r", "r²", "r³"].into_iter().enumerate() {
            left_label = step(stage, &kit, left, (index + 1) as f64, left_label, text, LEFT_LABEL)?;
            stage.wait(1.)?;
        }
        // Four turns bring the tile home; the name e waits for the sentence defining it.
        fade(stage, &[left_label], false)?;
        advance(stage, left, 4.)?;
        kit.until_sentence(stage, "13_3", 6)?;
        left_label = caption(stage, &kit, "r⁴ = e", LEFT_LABEL.0, LEFT_LABEL.1)?;
        kit.say(stage, "13_4")?;
        left_label = step(stage, &kit, left, 5., left_label, "s", LEFT_LABEL)?;
        stage.wait(1.)?;
        left_label = step(stage, &kit, left, 6., left_label, "s² = e", LEFT_LABEL)?;
        kit.say(stage, "13_5")?;
        right_label = step(stage, &kit, right, 1., right_label, "r", RIGHT_LABEL)?;
        right_label = step(stage, &kit, right, 2., right_label, "s r", RIGHT_LABEL)?;
        left_label = step(stage, &kit, left, 7., left_label, "s", LEFT_LABEL)?;
        left_label = step(stage, &kit, left, 8., left_label, "r s", LEFT_LABEL)?;
        kit.say(stage, "13_6")?;
        left_label = step(stage, &kit, left, 9., left_label, "s r s", LEFT_LABEL)?;
        // Undo the right history visibly; then apply the single inverse turn.
        for (to, text) in [(3., "r"), (4., "e"), (5., "r⁻¹")] {
            right_label = step(stage, &kit, right, to, right_label, text, RIGHT_LABEL)?;
        }
        // The relation waits for the words that state it.
        until_phrase(stage, &kit, "13_6", 4, RELATION_PHRASE)?;
        let relation = caption(stage, &kit, "s r s = r⁻¹", 0., -2.8)?;
        kit.say(stage, "13_7")?;
        let mut leaving = left_objects;
        leaving.extend(right_objects);
        leaving.extend([left_label, right_label, relation]);
        fade(stage, &leaving, false)?;
""")

A = sub(A, """        let mut undo_label = caption(stage, &kit, "r", 0., -2.)?;
        advance(stage, undo, 1.)?;
        for n in 2..=4 {
            advance(stage, undo, f64::from(n))?;
        }
        undo_label = replace(stage, &kit, undo_label, "r³ r = e", 0., -2.)?;
        kit.until_sentence(stage, "13_8", 3)?;
        undo_label = replace(stage, &kit, undo_label, "Half-turn, then half-turn", 0., -2.)?;
""", """        let first_turn = caption(stage, &kit, "r", 0., -2.)?;
        advance(stage, undo, 1.)?;
        // `r` names the first turn only: it leaves before the three undoing turns.
        fade(stage, &[first_turn], false)?;
        for n in 2..=4 {
            advance(stage, undo, f64::from(n))?;
        }
        let mut undo_label = caption(stage, &kit, "r³ r = e", 0., -2.)?;
        kit.until_sentence(stage, "13_8", 3)?;
        undo_label = replace(stage, &kit, undo_label, "Half-turn, then half-turn", 0., BELOW_TURNING)?;
""")
A = sub(A, """        undo_label = replace(stage, &kit, undo_label, "Flip, then flip", 0., -2.)?;
""", """        undo_label = replace(stage, &kit, undo_label, "Flip, then flip", 0., BELOW_TURNING)?;
""")

# 13_9: the word reads as one word, and the lit letter no longer sits above the idle tile.
A = sub(A, """        for (x, text) in [(-3., w0), (0., w1), (3., w2)] {
""", """        for (x, text) in [(TOKEN_X[0], w0), (TOKEN_X[1], w1), (TOKEN_X[2], w2)] {
""")
A = sub(A, """        let first = grouping(stage, -3.6, 0.6, 1.8)?;
""", """        let first = grouping(stage, TOKEN_X[0] - 0.45, TOKEN_X[1] + 0.35, 1.8)?;
""")
A = sub(A, """        let last = grouping(stage, -0.6, 3.6, 1.8)?;
""", """        let last = grouping(stage, TOKEN_X[1] - 0.35, TOKEN_X[2] + 0.45, 1.8)?;
""")

# 13_11: the caption's minus matches the mathematics above it.
A = sub(A, """            &format!("Undoing +{STEP} needs -{STEP}, which is missing"),
""", """            &format!("Undoing +{STEP} needs −{STEP}, which is missing"),
""")

# 13_10: a package's box holds its input and its output, not one machine's arrow.
A = sub(A, """        let first_bracket = frame(stage, (-1.5, ROW_Y[0]), (4.1, 0.66), Palette::gold(), 3.)?;
""", """        let first_bracket = trail_box(stage, FIRST_TWO_TRAIL, ROW_Y[0])?;
""")
A = sub(A, """        let last_bracket = frame(stage, (1.5, ROW_Y[1]), (4.1, 0.66), Palette::gold(), 3.)?;
""", """        let last_bracket = trail_box(stage, LAST_TWO_TRAIL, ROW_Y[1])?;
""")
A = sub(A, """/// The value travelling through the machines: a disc with its number.
""", """/// The gold box round a package's stretch of a trail.
fn trail_box(stage: &mut Stage<'_>, span: (f64, f64), y: f64) -> fmn::Result<Mob> {
    frame(
        stage,
        ((span.0 + span.1) / 2., y),
        (span.1 - span.0, 0.66),
        Palette::gold(),
        3.,
    )
}

/// The value travelling through the machines: a disc with its number.
""")

A = sub(A, """pub(crate) const WORD: [Elem; 3] = [S, R, S];
""", """pub(crate) const WORD: [Elem; 3] = [S, R, S];
/// The word's letters, close enough to read as one word and centred between the tiles.
const TOKEN_X: [f64; 3] = [-0.9, 0., 0.9];
""")
A = sub(A, """const ROW_Y: [f64; 2] = [-0.55, -1.55];
""", """const ROW_Y: [f64; 2] = [-0.55, -1.55];
/// From a package's input through its output: `1 → 3 → 9` for g∘h, `3 → 9 → 8` for f∘g.
const FIRST_TWO_TRAIL: (f64, f64) = (CHIP_IN - 0.35, MACHINE_X[1] + 0.35);
const LAST_TWO_TRAIL: (f64, f64) = (MACHINE_X[0] - 0.35, MACHINE_X[2] + 0.35);
// Each box holds its values and stops short of the next arrow out.
const _: () = assert!(FIRST_TWO_TRAIL.1 < (MACHINE_X[1] + MACHINE_X[2]) / 2. - 0.2);
const _: () = assert!(LAST_TWO_TRAIL.0 > (CHIP_IN + MACHINE_X[0]) / 2. + 0.2);
""")
A = sub(A, """const EQ_X: f64 = 2.4;
""", """/// The chapter's two tile-name positions.
const LEFT_LABEL: (f64, f64) = (-3.1, -2.);
const RIGHT_LABEL: (f64, f64) = (3.1, -2.);
/// A label that stays while a 1.25-scale tile turns: its corner discs reach 1.81 below centre.
const BELOW_TURNING: f64 = -2.3;
const EQ_X: f64 = 2.4;
""")
A = sub(A, """const GROUPING_PHRASE: &str = "grouping the first two";
""", """const RELATION_PHRASE: &str = "s r s equals";
const GROUPING_PHRASE: &str = "grouping the first two";
""")
A = sub(A, """const PHRASES: [(&str, usize, &str); 13] = [
    ("13_9", 3, GROUPING_PHRASE),
""", """const PHRASES: [(&str, usize, &str); 14] = [
    ("13_6", 4, RELATION_PHRASE),
    ("13_9", 3, GROUPING_PHRASE),
""")
A = sub(A, """        for (cue, sentences) in [("13_9", 6), ("13_10", 3), ("13_11", 5), ("13_12", 6)] {
""", """        for (cue, sentences) in [
            ("13_1", 3),
            ("13_2", 6),
            ("13_3", 7),
            ("13_4", 6),
            ("13_5", 5),
            ("13_6", 5),
            ("13_7", 5),
            ("13_8", 5),
            ("13_9", 6),
            ("13_10", 3),
            ("13_11", 5),
            ("13_12", 6),
        ] {
""")
A = sub(A, """    #[test]
    fn the_number_line_holds_every_value_it_names() {
""", """    #[test]
    fn each_trail_box_holds_its_packages_input_and_output() {
        let values = trail(INPUT);
        assert_eq!(g(h(values[0])), values[2]);
        assert_eq!(f(g(values[1])), values[3]);
    }

    #[test]
    fn the_number_line_holds_every_value_it_names() {
""")

# ---------------------------------------------------------------- layer B
B = A

# 13_1: the sentences get their pictures: two routes to one destination, then the eight.
B = sub(B, """        // Separate demonstration actors leave the worked-calculation histories intact.
        let demo_turn = stage.add_value_tracker(0.);
""", """        // Two histories, one destination: a long route and one quarter-turn.
        let long_route = stage.add_value_tracker(0.);
        let short_route = stage.add_value_tracker(0.);
        let mut routes = tile_actor::add_tile(stage, &kit, long_route, -3.1, 0., 1.25, |t| pose(&LONG_ROUTE, t))?;
        routes.extend(tile_actor::add_tile(
            stage,
            &kit,
            short_route,
            3.1,
            0.,
            1.25,
            tile_model::turn,
        )?);
        kit.until_sentence(stage, "13_1", 1)?;
        perform(stage, long_route, LONG_ROUTE.len() as f64, 3.4, &[])?;
        perform(stage, short_route, 1., LONE_RT, &[])?;
        // The eight arrangements, unnamed: their names arrive in 13_2 and 13_3.
        until_phrase(stage, &kit, "13_1", 2, EIGHT_PHRASE)?;
        fade(stage, &routes, false)?;
        let eight = bare_rack(stage, &kit)?;
        appear(stage, &eight, 0.3, 0.1)?;
        kit.say(stage, "13_2")?;
        fade(stage, &eight, false)?;
        // Separate demonstration actors leave the worked-calculation histories intact.
        let demo_turn = stage.add_value_tracker(0.);
""")
B = sub(B, """        kit.say(stage, "13_2")?;
        // The new symbols first name actions we can actually see.
        advance(stage, demo_turn, 1.)?;""", """        // The new symbols first name actions we can actually see.
        advance(stage, demo_turn, 1.)?;""")

# 13_4: the question gets its sentence; the two sequences stand side by side.
B = sub(B, """        kit.say(stage, "13_4")?;
        left_label = step(stage, &kit, left, 5., left_label, "s", LEFT_LABEL)?;
        stage.wait(1.)?;
        left_label = step(stage, &kit, left, 6., left_label, "s² = e", LEFT_LABEL)?;
        kit.say(stage, "13_5")?;
        right_label = step(stage, &kit, right, 1., right_label, "r", RIGHT_LABEL)?;
        right_label = step(stage, &kit, right, 2., right_label, "s r", RIGHT_LABEL)?;
        left_label = step(stage, &kit, left, 7., left_label, "s", LEFT_LABEL)?;
        left_label = step(stage, &kit, left, 8., left_label, "r s", LEFT_LABEL)?;
        kit.say(stage, "13_6")?;
        left_label = step(stage, &kit, left, 9., left_label, "s r s", LEFT_LABEL)?;
        // Undo the right history visibly; then apply the single inverse turn.
        for (to, text) in [(3., "r"), (4., "e"), (5., "r⁻¹")] {
            right_label = step(stage, &kit, right, to, right_label, text, RIGHT_LABEL)?;
        }
        // The relation waits for the words that state it.
        until_phrase(stage, &kit, "13_6", 4, RELATION_PHRASE)?;
        let relation = caption(stage, &kit, "s r s = r⁻¹", 0., -2.8)?;
        kit.say(stage, "13_7")?;
        let mut leaving = left_objects;
        leaving.extend(right_objects);
        leaving.extend([left_label, right_label, relation]);
        fade(stage, &leaving, false)?;
""", """        kit.say(stage, "13_4")?;
        kit.until_sentence(stage, "13_4", 1)?;
        left_label = step(stage, &kit, left, 5., left_label, "s", LEFT_LABEL)?;
        stage.wait(1.)?;
        left_label = step(stage, &kit, left, 6., left_label, "s² = e", LEFT_LABEL)?;
        kit.until_sentence(stage, "13_4", 5)?;
        right_label = replace(stage, &kit, right_label, "r⁴ = e", RIGHT_LABEL.0, RIGHT_LABEL.1)?;
        kit.say(stage, "13_5")?;
        // Each spoken pair of moves runs as one motion, named when it stops.
        kit.until_sentence(stage, "13_5", 1)?;
        fade(stage, &[right_label], false)?;
        perform(stage, right, 2., PACKAGE_RT, &[])?;
        right_label = caption(stage, &kit, "s r", RIGHT_LABEL.0, RIGHT_LABEL.1)?;
        kit.until_sentence(stage, "13_5", 2)?;
        fade(stage, &[left_label], false)?;
        perform(stage, left, 8., PACKAGE_RT, &[])?;
        left_label = caption(stage, &kit, "r s", LEFT_LABEL.0, LEFT_LABEL.1)?;
        kit.say(stage, "13_6")?;
        // Both tiles start again: the left performs s r s as spoken, the right
        // one quarter-turn the other way; then their corners can be compared.
        kit.until_sentence(stage, "13_6", 1)?;
        let mut leaving = left_objects;
        leaving.extend(right_objects);
        leaving.extend([left_label, right_label]);
        fade(stage, &leaving, false)?;
        let conjugate = stage.add_value_tracker(0.);
        let opposite = stage.add_value_tracker(0.);
        let left_objects = tile_actor::add_tile(stage, &kit, conjugate, -3.1, 0., 1.25, |t| pose(&SEQUENCE, t))?;
        let right_objects = tile_actor::add_tile(stage, &kit, opposite, 3.1, 0., 1.25, |t| tile_model::turn(-t))?;
        perform(stage, conjugate, 3., PACKAGE_RT, &[])?;
        left_label = caption(stage, &kit, "s r s", LEFT_LABEL.0, LEFT_LABEL.1)?;
        kit.until_sentence(stage, "13_6", 3)?;
        perform(stage, opposite, 1., LONE_RT, &[])?;
        right_label = caption(stage, &kit, "r⁻¹", RIGHT_LABEL.0, RIGHT_LABEL.1)?;
        // The relation waits for the words that state it.
        until_phrase(stage, &kit, "13_6", 4, RELATION_PHRASE)?;
        let relation = caption(stage, &kit, "s r s = r⁻¹ = r³", 0., -2.8)?;
        kit.say(stage, "13_7")?;
        let mut leaving = left_objects;
        leaving.extend(right_objects);
        leaving.extend([left_label, right_label, relation]);
        fade(stage, &leaving, false)?;
""")

# 13_7: the rules as spoken, on one word with no holes, ending on the rack's two rows.
B = sub(B, """        // A concrete word reduction; the equality rule explains each rewrite.
        // Keep the factors separate: the highlighted subword has explicit
        // positions, rather than guessing its location inside a TeX object.
        let mut factors = Vec::new();
        for (x, text) in [(-2., "s"), (-1., "r"), (0., "s"), (1., "r²"), (2., "s")] {
            factors.push(caption(stage, &kit, text, x, 0.6)?);
        }
        let rule = caption(stage, &kit, "s r s = r⁻¹", 0., -0.7)?;
        let bracket = grouping(stage, -2.4, 0.4, 0.6)?;
        stage.wait(2.)?;
        fade(stage, &[bracket], false)?;
        fade(stage, &factors[..3], false)?;
        let inverse = caption(stage, &kit, "r⁻¹", -1., 0.6)?;
        stage.wait(2.)?;
        let combine = grouping(stage, -1.5, 1.5, 0.6)?;
        stage.wait(1.)?;
        fade(stage, &[combine, inverse, factors[3]], false)?;
        let reduced = caption(stage, &kit, "r", 0., 0.6)?;
        let normal = caption(stage, &kit, "rᵏ  or  rᵏ s", 0., -1.6)?;
        kit.say(stage, "13_8")?;
        fade(stage, &[reduced, factors[4], rule, normal], false)?;
        let undo = stage.add_value_tracker(0.);
        let undo_objects = tile_actor::add_tile(stage, &kit, undo, 0., 0., 1.25, tile_model::turn)?;
        let first_turn = caption(stage, &kit, "r", 0., -2.)?;
        advance(stage, undo, 1.)?;
        // `r` names the first turn only: it leaves before the three undoing turns.
        fade(stage, &[first_turn], false)?;
        for n in 2..=4 {
            advance(stage, undo, f64::from(n))?;
        }
        let mut undo_label = caption(stage, &kit, "r³ r = e", 0., -2.)?;
        kit.until_sentence(stage, "13_8", 3)?;
        undo_label = replace(stage, &kit, undo_label, "Half-turn, then half-turn", 0., BELOW_TURNING)?;
        advance(stage, undo, 6.)?;
        advance(stage, undo, 8.)?;
        kit.until_sentence(stage, "13_8", 4)?;
""", """        // A concrete word, reduced by the rules as the voice states them. Keep the
        // factors separate: the highlighted subword has explicit positions, rather
        // than guessing its location inside a TeX object.
        let mut factors = Vec::new();
        for (x, text) in [(-2., "s"), (-1., "r"), (0., "s"), (1., "r²"), (2., "s")] {
            factors.push(caption(stage, &kit, text, x, 0.6)?);
        }
        // Move the flip past the turn, reversing the turn.
        kit.until_sentence(stage, "13_7", 1)?;
        let rule = caption(stage, &kit, "s r = r⁻¹ s", 0., -0.7)?;
        let bracket = grouping(stage, -2.4, -0.6, 0.6)?;
        stage.wait(1.6)?;
        fade(stage, &[bracket, factors[0], factors[1]], false)?;
        let inverse = caption(stage, &kit, "r⁻¹", -2., 0.6)?;
        let moved = caption(stage, &kit, "s", -1., 0.6)?;
        // The pair of flips cancels; then the two turns combine.
        kit.until_sentence(stage, "13_7", 2)?;
        let pair = grouping(stage, -1.4, 0.4, 0.6)?;
        stage.wait(1.)?;
        fade(stage, &[pair, moved, factors[2]], false)?;
        d4g::glide(stage, inverse, (0., 0.6), 0.4)?;
        let combine = grouping(stage, -0.5, 1.5, 0.6)?;
        stage.wait(0.8)?;
        fade(stage, &[combine, inverse, factors[3]], false)?;
        let reduced = caption(stage, &kit, "r", 1., 0.6)?;
        kit.until_sentence(stage, "13_7", 3)?;
        let normal = caption(stage, &kit, "rᵏ  or  rᵏ s", 0., -1.6)?;
        // Those two forms are the rack's two rows.
        kit.until_sentence(stage, "13_7", 4)?;
        fade(stage, &[reduced, factors[4], rule], false)?;
        d4g::glide(stage, normal, (0., 1.25), 0.4)?;
        let moves = rack(stage, &kit, MOVES_AT, MOVES_STEP, 0.8)?;
        appear(stage, &moves.slots, 0.3, 0.08)?;
        let row_frames = [row_frame(stage, 0)?, row_frame(stage, 1)?];
        d4g::show(stage, &row_frames, 0.35)?;
        kit.say(stage, "13_8")?;
        // The rack stays while the voice talks about it.
        fade(stage, &[row_frames[0], row_frames[1], normal], false)?;
        // The motions fit the sentences that name them.
        kit.until_sentence(stage, "13_8", 2)?;
        d4g::hide(stage, &moves.slots, 0.4)?;
        let undo = stage.add_value_tracker(0.);
        let undo_objects = tile_actor::add_tile(stage, &kit, undo, 0., 0., 1.25, tile_model::turn)?;
        perform(stage, undo, 1., 0.8, &[])?;
        perform(stage, undo, 4., 1.4, &[])?;
        let mut undo_label = caption(stage, &kit, "r³ r = e", 0., -2.)?;
        kit.until_sentence(stage, "13_8", 3)?;
        undo_label = replace(stage, &kit, undo_label, "Half-turn, then half-turn", 0., BELOW_TURNING)?;
        perform(stage, undo, 6., 0.7, &[])?;
        perform(stage, undo, 8., 0.7, &[])?;
        kit.until_sentence(stage, "13_8", 4)?;
""")
B = sub(B, """        undo_label = replace(stage, &kit, undo_label, "Flip, then flip", 0., BELOW_TURNING)?;
        advance(stage, flip_back, 1.)?;
        advance(stage, flip_back, 2.)?;
""", """        undo_label = replace(stage, &kit, undo_label, "Flip, then flip", 0., BELOW_TURNING)?;
        perform(stage, flip_back, 1., 1., &[])?;
        perform(stage, flip_back, 2., 1., &[])?;
""")
B = sub(B, """        "s r s = r⁻¹" => Some("srs=r^{-1}"),
""", """        "s r s = r⁻¹ = r³" => Some("srs=r^{-1}=r^3"),
        "s r = r⁻¹ s" => Some("sr=r^{-1}s"),
""")
B = sub(B, """        "s r s r² s" => Some("s r s r^2 s"),
        "r⁻¹ r² s" => Some("r^{-1} r^2 s"),
""", "")

# 13_12: the card change is one short play, and each calculation keeps its reading time.
B = sub(B, """        let lit = light(stage, &cards, 0)?;
""", """        let lit = relight(stage, &cards, None, 0, &[])?;
""")
B = sub(B, """        d4g::show(stage, &[closed, landed], 0.35)?;
        let aim = purpose(stage, &kit, "Stay inside the calculation")?;
        let mut shown""", """        let aim = purpose(stage, &kit, "Stay inside the calculation")?;
        d4g::show(stage, &[closed, landed, aim], 0.35)?;
        let mut shown""")
B = sub(B, """        kit.until_sentence(stage, "13_12", 2)?;
        d4g::hide(stage, &shown, 0.35)?;
        recolor(stage, &[cards[0].name, cards[0].rule], Palette::dim(), 0.2)?;
        let lit = light(stage, &cards, 1)?;
""", """        kit.until_sentence(stage, "13_12", 2)?;
        let lit = relight(stage, &cards, Some(0), 1, &shown)?;
""")
B = sub(B, """        d4g::show(stage, &[landed], 0.3)?;
        shown.extend([landed, aim]);
""", """        d4g::show(stage, &[landed, aim], 0.35)?;
        shown.extend([landed, aim]);
""")
B = sub(B, """        kit.until_sentence(stage, "13_12", 3)?;
        d4g::hide(stage, &shown, 0.35)?;
        recolor(stage, &[cards[1].name, cards[1].rule], Palette::dim(), 0.2)?;
        let lit = light(stage, &cards, 2)?;
""", """        kit.until_sentence(stage, "13_12", 3)?;
        let lit = relight(stage, &cards, Some(1), 2, &shown)?;
""")
B = sub(B, """        d4g::show(stage, &[neutral, unchanged], 0.35)?;
        let aim = purpose(stage, &kit, "Do nothing")?;
        shown = vec![lit, neutral, unchanged, aim];
""", """        let aim = purpose(stage, &kit, "Do nothing")?;
        d4g::show(stage, &[neutral, unchanged, aim], 0.35)?;
        shown = vec![lit, neutral, unchanged, aim];
""")
B = sub(B, """        kit.until_sentence(stage, "13_12", 4)?;
        d4g::hide(stage, &shown, 0.35)?;
        recolor(stage, &[cards[2].name, cards[2].rule], Palette::dim(), 0.2)?;
        let lit = light(stage, &cards, 3)?;
""", """        kit.until_sentence(stage, "13_12", 4)?;
        let lit = relight(stage, &cards, Some(2), 3, &shown)?;
""")
B = sub(B, """        d4g::show(stage, &[answer, check, checked], 0.35)?;
        let aim = purpose(stage, &kit, "Undo to solve")?;
        shown.extend""", """        let aim = purpose(stage, &kit, "Undo to solve")?;
        d4g::show(stage, &[answer, check, checked, aim], 0.35)?;
        shown.extend""")
B = sub(B, """/// Brighten card `k` and frame it; the returned frame is hidden with the demo.
fn light(stage: &mut Stage<'_>, cards: &[Card], k: usize) -> fmn::Result<Mob> {
    let lit = frame(stage, (CARD_X[k], CARD_Y), (3.35, 1.2), Palette::gold(), 4.)?;
    recolor(stage, &[cards[k].name], Palette::hero(), 0.25)?;
    recolor(stage, &[cards[k].rule], Palette::ink(), 0.25)?;
    d4g::show(stage, &[lit], 0.3)?;
    Ok(lit)
}
""", """/// Move the light to card `to` in one 0.4 s play: the last demonstration fades,
/// the last card dims, the new card brightens and gains its frame. Played one
/// after another these cost 1.35 s of a sentence that may last under 3 s.
fn relight(stage: &mut Stage<'_>, cards: &[Card], from: Option<usize>, to: usize, leaving: &[Mob]) -> fmn::Result<Mob> {
    const RUN: f64 = 0.4;
    let lit = frame(stage, (CARD_X[to], CARD_Y), (3.35, 1.2), Palette::gold(), 4.)?;
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in leaving {
        stage.clear_updaters(mob, true);
        let a = fade_out(stage.arena_mut(), mob, ORIGIN, 1.)?.rt(RUN);
        animations.push(stage.prepare(a)?);
    }
    let mut colors = vec![(cards[to].name, Palette::hero()), (cards[to].rule, Palette::ink())];
    if let Some(k) = from {
        colors.extend([(cards[k].name, Palette::dim()), (cards[k].rule, Palette::dim())]);
    }
    for (mob, color) in colors {
        let a = mob
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(RUN),
                ..AnimateArgs::default()
            })?
            .set_color(color)?;
        animations.push(stage.prepare(a)?);
    }
    let a = fade_in(stage.arena_mut(), lit, ORIGIN, 1.)?.rt(RUN);
    animations.push(stage.prepare(a)?);
    stage.play_prepared(animations)?;
    Ok(lit)
}
""")
B = sub(B, """fn purpose(stage: &mut Stage<'_>, kit: &Kit, text: &str) -> fmn::Result<Mob> {
    let mob = kit.text(stage, text, 30., Palette::hero(), v(0., -2.6))?;
    d4g::show(stage, &[mob], 0.35)?;
    Ok(mob)
}
""", """/// A beat's one-line purpose, not yet shown: it arrives with the beat's last marks.
fn purpose(stage: &mut Stage<'_>, kit: &Kit, text: &str) -> fmn::Result<Mob> {
    kit.text(stage, text, 30., Palette::hero(), v(0., -2.6))
}
""")
B = sub(B, """pub(crate) const CLOSURE_PAIR: (Elem, Elem) = (R, S);
""", """/// `s r` is not a name on the rack, and the product `r³ s` is: closure with content,
/// where the old `r s = rs` read as a tautology.
pub(crate) const CLOSURE_PAIR: (Elem, Elem) = (S, R);
""")
B = sub(B, """const RIGHT: [Move; 5] = [Move::Turn(1.), Move::Flip, Move::Flip, Move::Turn(-1.), Move::Turn(-1.)];
""", """const RIGHT: [Move; 5] = [Move::Turn(1.), Move::Flip, Move::Flip, Move::Turn(-1.), Move::Turn(-1.)];
/// 13_1's long route, which ends where one quarter-turn ends (a test checks).
const LONG_ROUTE: [Move; 5] = [Move::Turn(1.), Move::Flip, Move::Flip, Move::Turn(1.), Move::Turn(-1.)];
""")
B = sub(B, """const RELATION_PHRASE: &str = "s r s equals";
""", """const EIGHT_PHRASE: &str = "its final action";
const RELATION_PHRASE: &str = "s r s equals";
""")
B = sub(B, """const PHRASES: [(&str, usize, &str); 14] = [
""", """const PHRASES: [(&str, usize, &str); 15] = [
    ("13_1", 2, EIGHT_PHRASE),
""")
B = sub(B, """/// The chapter's two tile-name positions.
""", """/// The rack that 13_7 and 13_8 point at, and the frame round one of its rows.
const MOVES_AT: (f64, f64) = (-2.25, 0.1);
const MOVES_STEP: (f64, f64) = (1.5, 1.45);
/// The chapter's two tile-name positions.
""")
B = sub(B, """/// The integers drawn on 13_11's line, each with its tick and label.
""", """/// One row of the 13_7 rack, thumbnails and names together.
fn row_frame(stage: &mut Stage<'_>, row: usize) -> fmn::Result<Mob> {
    let y = MOVES_AT.1 - MOVES_STEP.1 * row as f64 - 0.2;
    frame(stage, (0., y), (5.9, 1.35), Palette::gold(), 3.)
}
/// The eight arrangements before they have names, rotations above.
fn bare_rack(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    ALL.into_iter()
        .enumerate()
        .map(|(i, g)| {
            let at = (-2.25 + 1.5 * (i % 4) as f64, 0.8 - 1.6 * (i / 4) as f64);
            d4g::thumb(stage, kit, g, at, 1.)
        })
        .collect()
}

/// The integers drawn on 13_11's line, each with its tick and label.
""")
B = sub(B, """    #[test]
    fn each_trail_box_holds_its_packages_input_and_output() {
""", """    #[test]
    fn the_routes_and_the_restarted_tiles_end_as_spoken() {
        assert!(same_corners(
            pose(&LONG_ROUTE, LONG_ROUTE.len() as f64),
            tile_model::turn(1.)
        ));
        assert!(same_corners(pose(&SEQUENCE, 3.), tile_model::turn(-1.)));
    }

    #[test]
    fn the_word_reduction_uses_the_spoken_rules() {
        assert_eq!(LONG_WORD, [S, R, S, R2, S]);
        assert_eq!(S.mul(R), R.inv().mul(S));
        assert_eq!(S.mul(S), E);
        assert_eq!(R.inv().mul(R2), R);
    }

    #[test]
    fn each_trail_box_holds_its_packages_input_and_output() {
""")

B_oracle = sub(oracle, """emit("arithmetic.closure.product", TEX[compose(r_, s_)])
""", """emit("arithmetic.closure.product", TEX[compose(s_, r_)])
""")

for name, rs, py in (("A", A, oracle), ("AB", B, B_oracle)):
    d = OUT / name
    d.mkdir(parents=True, exist_ok=True)
    (d / "arithmetic.rs").write_text(rs)
    (d / "oracle.py").write_text(py)
    print(name, "lines", rs.count("\n"), "oracle changed", py != oracle)
