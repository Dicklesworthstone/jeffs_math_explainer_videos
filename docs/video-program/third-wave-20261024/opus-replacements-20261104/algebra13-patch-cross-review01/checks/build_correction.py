"""Build candidate-ABC from BronzeJaguar's candidate-AB by exact, single-occurrence replacements.

Writes only inside this review's folder. Usage (repo root): python3 -I -B <this file>
"""
import pathlib
import shutil

O = pathlib.Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
AB = O / "algebra-picture-review-13_arithmetic/candidate-AB/explainer_corpus/abstract_algebra_expanded"
OUT = O / "algebra13-patch-cross-review01/candidate-ABC/explainer_corpus/abstract_algebra_expanded"

EDITS = [
    (
        "X1 13_2: the flip runs before its name",
        '''        let mut demo_s = caption(stage, &kit, "s", RIGHT_LABEL.0, RIGHT_LABEL.1)?;
        advance(stage, demo_flip, 1.)?;
''',
        '''        advance(stage, demo_flip, 1.)?;
        let mut demo_s = caption(stage, &kit, "s", RIGHT_LABEL.0, RIGHT_LABEL.1)?;
''',
    ),
    (
        "X2 13_3: turns under their words; each power up when it is named",
        '''        for (index, text) in ["r", "r²", "r³"].into_iter().enumerate() {
            left_label = step(stage, &kit, left, (index + 1) as f64, left_label, text, LEFT_LABEL)?;
            stage.wait(1.)?;
        }
''',
        '''        // The turns run under the words that ask for them; each power is up when the voice names it.
        kit.until_sentence(stage, "13_3", 1)?;
        fade(stage, &[left_label], false)?;
        perform(stage, left, 1., LONE_RT, &[])?;
        perform(stage, left, 2., LONE_RT, &[])?;
        kit.until_sentence(stage, "13_3", 3)?;
        left_label = caption(stage, &kit, "r²", LEFT_LABEL.0, LEFT_LABEL.1)?;
        kit.until_sentence(stage, "13_3", 4)?;
        left_label = step(stage, &kit, left, 3., left_label, "r³", LEFT_LABEL)?;
        stage.wait(0.6)?;
''',
    ),
    (
        "X3 13_7: the reversed turn waits for the words that reverse it",
        '''        let rule = caption(stage, &kit, "s r = r⁻¹ s", 0., -0.7)?;
        let bracket = grouping(stage, -2.4, -0.6, 0.6)?;
        stage.wait(1.6)?;
        fade(stage, &[bracket, factors[0], factors[1]], false)?;
''',
        '''        let bracket = grouping(stage, -2.4, -0.6, 0.6)?;
        // The reversed turn waits for the words that reverse it.
        until_phrase(stage, &kit, "13_7", 1, REVERSE_PHRASE)?;
        let rule = caption(stage, &kit, "s r = r⁻¹ s", 0., -0.7)?;
        fade(stage, &[bracket, factors[0], factors[1]], false)?;
''',
    ),
    (
        "X4 13_8: the tile is ready before its sentence, so r³ r = e can be read",
        '''        // The motions fit the sentences that name them.
        kit.until_sentence(stage, "13_8", 2)?;
        d4g::hide(stage, &moves.slots, 0.4)?;
        let undo = stage.add_value_tracker(0.);
        let undo_objects = tile_actor::add_tile(stage, &kit, undo, 0., 0., 1.25, tile_model::turn)?;
''',
        '''        // The rack leaves on "an undo move": the next sentence's motions then end in time to read r³ r = e.
        until_phrase(stage, &kit, "13_8", 1, UNDO_PHRASE)?;
        d4g::hide(stage, &moves.slots, 0.4)?;
        let undo = stage.add_value_tracker(0.);
        let undo_objects = tile_actor::add_tile(stage, &kit, undo, 0., 0., 1.25, tile_model::turn)?;
        // The motions fit the sentences that name them.
        kit.until_sentence(stage, "13_8", 2)?;
''',
    ),
    (
        "X5 13_12: the comment names the new pair",
        "        // Closure: r times s is again one of the eight.\n",
        "        // Closure: s times r is again one of the eight.\n",
    ),
    (
        "X3, X4: the two new phrases",
        '''const RELATION_PHRASE: &str = "s r s equals";
''',
        '''const RELATION_PHRASE: &str = "s r s equals";
const REVERSE_PHRASE: &str = "if we reverse";
const UNDO_PHRASE: &str = "an undo move";
''',
    ),
    (
        "X3, X4: the phrase table",
        '''const PHRASES: [(&str, usize, &str); 15] = [
    ("13_1", 2, EIGHT_PHRASE),
    ("13_6", 4, RELATION_PHRASE),
''',
        '''const PHRASES: [(&str, usize, &str); 17] = [
    ("13_1", 2, EIGHT_PHRASE),
    ("13_6", 4, RELATION_PHRASE),
    ("13_7", 1, REVERSE_PHRASE),
    ("13_8", 1, UNDO_PHRASE),
''',
    ),
    (
        "X6 test: the poses the labels name",
        '''    #[test]
    fn the_word_reduction_uses_the_spoken_rules() {
''',
        '''    #[test]
    fn the_tiles_stop_under_their_names() {
        for (t, name) in [(2., R2), (3., R.inv()), (4., E), (5., S), (6., E), (8., R.mul(S))] {
            assert!(same_corners(pose(&LEFT, t), name.matrix()), "left at {t}");
        }
        assert!(same_corners(pose(&RIGHT, 2.), S.mul(R).matrix()));
    }

    #[test]
    fn the_word_reduction_uses_the_spoken_rules() {
''',
    ),
]

src = (AB / "src/arithmetic.rs").read_text(encoding="utf-8")
for name, old, new in EDITS:
    if src.count(old) != 1:
        raise SystemExit(f"{name}: found {src.count(old)} times")
    src = src.replace(old, new)
    print("applied", name)
(OUT / "src").mkdir(parents=True, exist_ok=True)
(OUT / "src/arithmetic.rs").write_text(src, encoding="utf-8")
shutil.copyfile(AB / "oracle.py", OUT / "oracle.py")
print("wrote", OUT)
