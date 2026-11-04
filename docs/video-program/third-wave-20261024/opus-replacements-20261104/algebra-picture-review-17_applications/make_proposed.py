"""Build proposed-applications.rs and proposed.patch from before-applications.rs.

Run from the repo root: python3 -I -B <this file>
Every edit names an exact block of the baseline that must occur exactly once, so
the script refuses to run on any other baseline. Each edit names its finding in
REVIEW.md. No spoken text changes: script.tsv is not touched, and no k.say moves.
"""
import difflib
import hashlib
from pathlib import Path

HERE = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra-picture-review-17_applications")
BEFORE = HERE / "before-applications.rs"
PROPOSED = HERE / "proposed-applications.rs"
PATCH = HERE / "proposed.patch"
LABEL = "explainer_corpus/abstract_algebra_expanded/src/applications.rs"
BASE_SHA = "7cc3f0d08b61087206144a6f187b2ce1a21059632c4aec8aa95a1a9a46f90bef"

EDITS = []


def edit(finding, old, new):
    EDITS.append((finding, old, new))


edit(
    "imports",
    """use crate::d4_group as d4g;
use explainer_kit""",
    """use crate::axioms::until_phrase;
use crate::d4_group as d4g;
use explainer_kit""",
)

edit(
    "F4, F6 phrase and pace constants",
    """const DIGITS: [i32; 10] = [0, 3, 0, 6, 4, 0, 6, 1, 5, 2];
""",
    """const DIGITS: [i32; 10] = [0, 3, 0, 6, 4, 0, 6, 1, 5, 2];
/// The column walk starts when the contributions start being read.
const CONTRIBUTIONS_PHRASE: &str = "gives weighted contributions";
/// The swap's change appears when the voice reaches it.
const SWAP_CHANGE_PHRASE: &str = "changes the total";
/// One fade of the column walk: five of these and a short hold make 0.7 s a column.
const WALK_FADE: f64 = 0.1;
""",
)

edit(
    "F4 fade with a chosen run time",
    """fn fade(stage: &mut Stage<'_>, items: &[Mob], show: bool) -> fmn::Result<()> {
    let mut a: Vec<Box<dyn Animation>> = Vec::new();
""",
    """fn fade(stage: &mut Stage<'_>, items: &[Mob], show: bool) -> fmn::Result<()> {
    fade_for(stage, items, show, 0.35)
}
fn fade_for(stage: &mut Stage<'_>, items: &[Mob], show: bool, run_time: f64) -> fmn::Result<()> {
    let mut a: Vec<Box<dyn Animation>> = Vec::new();
""",
)
edit(
    "F4 fade with a chosen run time",
    """        a.push(stage.prepare(f.rt(0.35))?);
""",
    """        a.push(stage.prepare(f.rt(run_time))?);
""",
)

edit(
    "F10 17_1 digits and note wait for their sentences",
    """        fade(stage, &[title], true)?;
        fade(stage, &digits, true)?;
        let note = k.text(
""",
    """        fade(stage, &[title], true)?;
        k.until_sentence(stage, "17_1", 1)?;
        fade(stage, &digits, true)?;
        k.until_sentence(stage, "17_1", 3)?;
        let note = k.text(
""",
)

edit(
    "F10, F18 17_2 weights, total and the divisibility rule wait for their sentences",
    """        k.say(stage, "17_2")?;
        let mut weights = Vec::new();
        for i in 0..10 {
            weights.push(k.text(stage, &format!("x {}", 10 - i), 28., Palette::hero(), v(x(i), 0.65))?);
        }
        fade(stage, &weights, true)?;
        let mut products = Vec::new();
        let mut sum = 0;
        let mut total = k.text(stage, "Running total: 0", 32., Palette::ink(), v(0., -1.5))?;
        fade(stage, &[total], true)?;
        k.say(stage, "17_3")?;
        for (i, d) in DIGITS.iter().enumerate() {
            let ring = outline(stage, i);
            fade(stage, &[ring], true)?;
            let contribution = (10 - i as i32) * d;
            sum += contribution;
            let p = k.text(stage, &contribution.to_string(), 28., Palette::ink(), v(x(i), -0.05))?;
            products.push(p);
            fade(stage, &[p], true)?;
""",
    """        k.say(stage, "17_2")?;
        k.until_sentence(stage, "17_2", 1)?;
        let mut weights = Vec::new();
        for i in 0..10 {
            weights.push(k.text(stage, &format!("× {}", 10 - i), 28., Palette::hero(), v(x(i), 0.65))?);
        }
        fade(stage, &weights, true)?;
        k.until_sentence(stage, "17_2", 2)?;
        let mut products = Vec::new();
        let mut sum = 0;
        let mut total = k.text(stage, "Running total: 0", 32., Palette::ink(), v(0., -1.5))?;
        fade(stage, &[total], true)?;
        k.until_sentence(stage, "17_2", 3)?;
        let divisible = k.text(
            stage,
            "Valid only if the total is divisible by 11",
            28.,
            Palette::hero(),
            v(0., -2.2),
        )?;
        fade(stage, &[divisible], true)?;
        k.say(stage, "17_3")?;
        until_phrase(stage, &k, "17_3", 1, CONTRIBUTIONS_PHRASE)?;
        for (i, d) in DIGITS.iter().enumerate() {
            let ring = outline(stage, i);
            fade_for(stage, &[ring], true, WALK_FADE)?;
            let contribution = (10 - i as i32) * d;
            sum += contribution;
            let p = k.text(stage, &contribution.to_string(), 28., Palette::ink(), v(x(i), -0.05))?;
            products.push(p);
            fade_for(stage, &[p], true, WALK_FADE)?;
""",
)

edit(
    "F4, F18 17_3 walk pace, verdict at its sentence",
    """            fade(stage, &[total], false)?;
            fade(stage, &[next], true)?;
            total = next;
            stage.wait(0.15)?;
            fade(stage, &[ring], false)?;
        }
        let valid = k.text(
            stage,
            &format!("{} = {} x 11: passes", sum, sum / 11),
            30.,
            Palette::green(),
            v(0., -2.2),
        )?;
        fade(stage, &[valid], true)?;
""",
    """            fade_for(stage, &[total], false, WALK_FADE)?;
            fade_for(stage, &[next], true, WALK_FADE)?;
            total = next;
            stage.wait(0.2)?;
            fade_for(stage, &[ring], false, WALK_FADE)?;
        }
        k.until_sentence(stage, "17_3", 3)?;
        let valid = k.text(
            stage,
            &format!("{} = {} × 11: passes", sum, sum / 11),
            30.,
            Palette::green(),
            v(0., -2.2),
        )?;
        fade(stage, &[divisible], false)?;
        fade(stage, &[valid], true)?;
""",
)

edit(
    "F1 ghost, F3 17_4 beats: replaced handles leave the lists the next wipe fades",
    """        fade(stage, &[digits[3], products[3], total, valid], false)?;
        fade(stage, &[digit, product], true)?;
        let changed_sum = checksum(&changed);
        total = k.text(
            stage,
            &format!("{sum} + 7 = {changed_sum}"),
            32.,
            Palette::red(),
            v(0., -1.5),
        )?;
        let fails = k.text(
            stage,
            &format!("Remainder {}: copying error detected", changed_sum % 11),
            28.,
            Palette::red(),
            v(0., -2.2),
        )?;
        fade(stage, &[total, fails], true)?;
        k.say(stage, "17_5")?;
        let mut old = vec![title, note, ring, digit, product, total, fails];
        old.extend(digits.iter().copied());
""",
    """        // Sentence 0 changes the symbol and nothing else. The replaced handles leave
        // `digits` and `products`, so the wipe below cannot re-root them: fading a
        // removed mobject out shows it at full opacity first.
        fade(stage, &[digits[3]], false)?;
        fade(stage, &[digit], true)?;
        digits[3] = digit;
        k.until_sentence(stage, "17_4", 1)?;
        fade(stage, &[products[3], total, valid], false)?;
        fade(stage, &[product], true)?;
        products[3] = product;
        let changed_sum = checksum(&changed);
        total = k.text(
            stage,
            &format!("{sum} + 7 = {changed_sum}"),
            32.,
            Palette::red(),
            v(0., -1.5),
        )?;
        fade(stage, &[total], true)?;
        k.until_sentence(stage, "17_4", 2)?;
        let fails = k.text(
            stage,
            &format!("Remainder {}: copying error detected", changed_sum % 11),
            28.,
            Palette::red(),
            v(0., -2.2),
        )?;
        fade(stage, &[fails], true)?;
        k.say(stage, "17_5")?;
        let mut old = vec![title, note, ring, total, fails];
        old.extend(digits.iter().copied());
""",
)

edit(
    "F5, F11 17_5 beats and row labels",
    """        fade(stage, &[heading], true)?;
        let rule = k.tex(stage, r"\\Delta S=w\\,\\Delta d", 40., Palette::blue(), &[], v(0., 2.))?;
        fade(stage, &[rule], true)?;
        // A full nonzero row of the multiplication table, followed by its inverse decoding.
        let caption = k.text(
""",
    """        fade(stage, &[heading], true)?;
        k.until_sentence(stage, "17_5", 1)?;
        let rule = k.tex(stage, r"\\Delta S=w\\,\\Delta d", 40., Palette::blue(), &[], v(0., 2.))?;
        fade(stage, &[rule], true)?;
        k.until_sentence(stage, "17_5", 3)?;
        // One row of the multiplication table: every nonzero change times the weight 7.
        let caption = k.text(
""",
)
edit(
    "F5, F11 17_5 beats and row labels",
    """        fade(stage, &[caption], true)?;
        let mut row = Vec::new();
        for d in 1..11 {
""",
    """        let change_label = k.tex(stage, r"\\Delta d", 24., Palette::blue(), &[], v(-6.05, 0.1))?;
        let product_label = k.tex(
            stage,
            r"7\\Delta d\\pmod{11}",
            20.,
            Palette::green(),
            &[],
            v(-6.05, -0.85),
        )?;
        fade(stage, &[caption, change_label, product_label], true)?;
        let mut row = vec![change_label, product_label];
        for d in 1..11 {
""",
)
edit(
    "F5 17_5 beats",
    """            row.extend([a, b]);
        }
        let condition = k.text(
""",
    """            row.extend([a, b]);
        }
        k.until_sentence(stage, "17_5", 4)?;
        let condition = k.text(
""",
)

edit(
    "F6, F12 17_7 weights stay on screen, formula beats, 17_8 setup waits",
    """        fade(stage, &swap_digits, true)?;
        let before = k.tex(stage, r"7\\times6+6\\times4=66", 34., Palette::ink(), &[], v(0., -0.2))?;
""",
    """        fade(stage, &swap_digits, true)?;
        // The weights stay put while the digits cross: they are the 7 and 6 of the formula.
        let swap_weights = [
            k.text(stage, "× 7", 25., Palette::hero(), v(x(3), 0.35))?,
            k.text(stage, "× 6", 25., Palette::hero(), v(x(4), 0.35))?,
        ];
        fade(stage, &swap_weights, true)?;
        k.until_sentence(stage, "17_7", 1)?;
        let before = k.tex(stage, r"7\\times6+6\\times4=66", 34., Palette::ink(), &[], v(0., -0.2))?;
""",
)
edit(
    "F6, F12 17_7 weights stay on screen, formula beats, 17_8 setup waits",
    """        fade(stage, &[after], true)?;
        let delta = k.tex(stage, r"\\Delta S=(7-6)(4-6)=-2", 32., Palette::hero(), &[], v(0., -2.3))?;
        fade(stage, &[delta], true)?;
        k.say(stage, "17_8")?;
        let mut old = vec![title, before, after, delta];
        old.extend(swap_digits);
        fade(stage, &old, false)?;
        let title = k.text(stage, "Two errors can cancel", 36., Palette::ink(), v(0., 3.))?;
        fade(stage, &[title], true)?;
        let mut corrupt = DIGITS;
""",
    """        fade(stage, &[after], true)?;
        until_phrase(stage, &k, "17_7", 1, SWAP_CHANGE_PHRASE)?;
        let delta = k.tex(stage, r"\\Delta S=(7-6)(4-6)=-2", 32., Palette::hero(), &[], v(0., -2.3))?;
        let mut why = k.text(
            stage,
            "weight difference × value difference",
            24.,
            Palette::dim(),
            v(0., -3.05),
        )?;
        fade(stage, &[delta, why], true)?;
        for (j, line) in [
            (2, "Distinct positions: the weight difference is nonzero"),
            (3, "Distinct symbols: the value difference is nonzero"),
            (4, "Two nonzero factors: this swap is detected"),
        ] {
            k.until_sentence(stage, "17_7", j)?;
            let next = k.text(stage, line, 24., Palette::ink(), v(0., -3.05))?;
            fade(stage, &[why], false)?;
            fade(stage, &[next], true)?;
            why = next;
        }
        k.say(stage, "17_8")?;
        // The swap picture still fits "adjacent swaps"; the failure's setup waits for the
        // second sentence and its title for "Here is a concrete failure".
        k.until_sentence(stage, "17_8", 1)?;
        let mut old = vec![title, before, after, delta, why];
        old.extend(swap_digits);
        old.extend(swap_weights);
        fade(stage, &old, false)?;
        let mut corrupt = DIGITS;
""",
)
edit(
    "F18 letter x for times",
    """            weights.push(k.text(stage, &format!("x {}", 10 - i), 25., Palette::hero(), v(x(i), 1.2))?);
""",
    """            weights.push(k.text(stage, &format!("× {}", 10 - i), 25., Palette::hero(), v(x(i), 1.2))?);
""",
)
edit(
    "F17 wheel clears the weight row",
    """                v(3. + 1.5 * t.sin(), -0.5 + 1.5 * t.cos()),
""",
    """                v(3. + 1.4 * t.sin(), -0.6 + 1.4 * t.cos()),
""",
)
edit(
    "F17 wheel clears the weight row",
    """                    .moved_to(v(3. + 1.13 * t.sin(), -0.5 + 1.13 * t.cos())),
""",
    """                    .moved_to(v(3. + 1.05 * t.sin(), -0.6 + 1.05 * t.cos())),
""",
)
edit(
    "F12 17_8 title at its sentence",
    """        fade(stage, &[legend], true)?;
        k.until_sentence(stage, "17_8", 3)?;
""",
    """        fade(stage, &[legend], true)?;
        k.until_sentence(stage, "17_8", 2)?;
        let title = k.text(stage, "Two errors can cancel", 36., Palette::ink(), v(0., 3.))?;
        fade(stage, &[title], true)?;
        k.until_sentence(stage, "17_8", 3)?;
""",
)
edit(
    "F18 letter x for times",
    """            &format!("+{second_delta} = {} x 11 + {}", second_delta / 11, second_delta % 11),
""",
    """            &format!("+{second_delta} = {} × 11 + {}", second_delta / 11, second_delta % 11),
""",
)

edit(
    "F7 17_11 recap follows the sentences and names what the voice names",
    """        fade(stage, &clock, true)?;
        let subgroup = k.text(stage, "Moves that stay together", 28., Palette::blue(), v(2.5, 1.5))?;
        let partition = k.text(
            stage,
            "Equal pieces that cover the group",
            25.,
            Palette::hero(),
            v(2.5, 0.2),
        )?;
        let quotient = k.text(
            stage,
            "An operation on those pieces?",
            26.,
            Palette::green(),
            v(2.5, -1.1),
        )?;
        fade(stage, &[subgroup], true)?;
        for offset in 0..4 {
            let color = [Palette::blue(), Palette::hero(), Palette::green(), Palette::gold()][offset];
            let points = (0..3)
                .map(|j| {
                    let t = std::f64::consts::TAU * (offset + j * 4) as f64 / 12.;
                    v(-3. + 1.65 * t.sin(), 0.3 + 1.65 * t.cos())
                })
                .collect::<Vec<_>>();
            let triangle = stage
                .arena_mut()
                .add(Polygon::new(points).style(stroke_style(color, 2.)).build());
            fade(stage, &[triangle], true)?;
            clock.push(triangle);
        }
        fade(stage, &[partition, quotient], true)?;
        k.say(stage, "17_12")?;
        clock.extend([title, subgroup, partition, quotient]);
        fade(stage, &clock, false)?;
""",
    """        fade(stage, &clock, true)?;
        // "Our square and clock": the voice names both, so both are on screen.
        let square = d4g::thumb(stage, &k, d4g::E, (2.5, -2.5), 1.3)?;
        fade(stage, &[square], true)?;
        clock.push(square);
        k.until_sentence(stage, "17_11", 1)?;
        let group = k.text(stage, "Group: how moves combine", 25., Palette::ink(), v(2.5, 1.9))?;
        fade(stage, &[group], true)?;
        k.until_sentence(stage, "17_11", 2)?;
        let subgroup = k.text(
            stage,
            "Subgroup: moves that stay together",
            25.,
            Palette::blue(),
            v(2.5, 1.1),
        )?;
        let mut triangles = Vec::new();
        for offset in 0..4 {
            let color = [Palette::blue(), Palette::hero(), Palette::green(), Palette::gold()][offset];
            let points = (0..3)
                .map(|j| {
                    let t = std::f64::consts::TAU * (offset + j * 4) as f64 / 12.;
                    v(-3. + 1.65 * t.sin(), 0.3 + 1.65 * t.cos())
                })
                .collect::<Vec<_>>();
            triangles.push(
                stage
                    .arena_mut()
                    .add(Polygon::new(points).style(stroke_style(color, 2.)).build()),
            );
        }
        fade(stage, &[triangles[0], subgroup], true)?;
        k.until_sentence(stage, "17_11", 3)?;
        let isomorphism = k.text(
            stage,
            "Isomorphism: same structure, new description",
            25.,
            Palette::ink(),
            v(2.5, 0.3),
        )?;
        fade(stage, &[isomorphism], true)?;
        k.until_sentence(stage, "17_11", 4)?;
        let partition = k.text(
            stage,
            "Cosets: equal pieces that cover the group",
            25.,
            Palette::hero(),
            v(2.5, -0.5),
        )?;
        let quotient = k.text(
            stage,
            "Normal subgroup: pieces inherit an operation",
            25.,
            Palette::green(),
            v(2.5, -1.3),
        )?;
        for &triangle in &triangles[1..] {
            fade(stage, &[triangle], true)?;
        }
        fade(stage, &[partition], true)?;
        fade(stage, &[quotient], true)?;
        k.say(stage, "17_12")?;
        clock.extend(triangles);
        clock.extend([title, group, subgroup, isomorphism, partition, quotient]);
        fade(stage, &clock, false)?;
""",
)

edit(
    "F13 17_12 prompts follow their sentences, name isomorphism, align at the numerals",
    """            "2. Can a four-clock equal two two-position clocks?",
""",
    """            "2. Can a four-clock be isomorphic to two two-position clocks?",
""",
)
edit(
    "F13 17_12 prompts follow their sentences, name isomorphism, align at the numerals",
    """            let m = k.text(stage, p, 27., Palette::ink(), v(0., 2. - i as f64 * 1.25))?;
            fade(stage, &[m], true)?;
""",
    """            k.until_sentence(stage, "17_12", i + 1)?;
            let m = k.text_left(stage, p, 27., Palette::ink(), v(-3.6, 2. - i as f64 * 1.25))?;
            fade(stage, &[m], true)?;
""",
)

edit(
    "F8 17_13 answers follow their sentences",
    """            stage.wait(0.4)?;
        }
        let a = k.text(
            stage,
            "Four-clock: a move needs four repeats",
""",
    """            stage.wait(0.4)?;
        }
        k.until_sentence(stage, "17_13", 1)?;
        let a = k.text(
            stage,
            "Four-clock: a move needs four repeats",
""",
)
edit(
    "F8 17_13 answers follow their sentences",
    """        fade(stage, &[a, b], true)?;
        let c = k.tex(stage, r"hH=H\\quad(h\\in H)", 32., Palette::green(), &[], v(2., -1.))?;
""",
    """        fade(stage, &[a, b], true)?;
        k.until_sentence(stage, "17_13", 2)?;
        let c = k.tex(stage, r"hH=H\\quad(h\\in H)", 32., Palette::green(), &[], v(2., -1.))?;
""",
)

edit(
    "F14 heading for the XOR stack",
    """    let mut rows = Vec::new();
    let ones: Vec<u8> = (1..=7).filter(|&p| has_one(word, p)).collect();
""",
    """    let heading = k.text(stage, "positions holding a 1", 22., Palette::dim(), v(3.2, 2.8))?;
    fade(stage, &[heading], true)?;
    let mut rows = vec![heading];
    let ones: Vec<u8> = (1..=7).filter(|&p| has_one(word, p)).collect();
""",
)

edit(
    "F9, F15 17_15 fills keep pace, rows named as cosets",
    """    let colors = explainer_kit::point_colors(8);
    for (row, one_row) in cells.iter().enumerate().skip(1) {
        let mut fills: Vec<Box<dyn Animation>> = Vec::new();
        for &cell in one_row {
            fills.push(stage.prepare(cell.animate().set_fill(Some(colors[row]), Some(0.45))?)?);
        }
""",
    """    let rows_are = k.text(
        stage,
        "each row: a coset of 16 words",
        24.,
        Palette::ink(),
        v(3.3, 0.35),
    )?;
    fade(stage, &[rows_are], true)?;
    let colors = explainer_kit::point_colors(8);
    for (row, one_row) in cells.iter().enumerate().skip(1) {
        let mut fills: Vec<Box<dyn Animation>> = Vec::new();
        for &cell in one_row {
            let fill = cell
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.4),
                    ..AnimateArgs::default()
                })?
                .set_fill(Some(colors[row]), Some(0.45))?;
            fills.push(stage.prepare(fill)?);
        }
""",
)
edit(
    "F15 rows named as cosets; syndrome named",
    """    k.say(stage, "17_16")?;
    fade(stage, &[lagrange], false)?;
""",
    """    k.say(stage, "17_16")?;
    fade(stage, &[lagrange, rows_are], false)?;
""",
)
edit(
    "F15 rows named as cosets; syndrome named",
    """    fade(stage, &labels, true)?;
    let (mut stack, mut result) = syndrome_stack(stage, k, word.value)?;
""",
    """    let syndrome_name = k.text(stage, "syndrome", 22., Palette::dim(), v(4.95, 0.1))?;
    fade(stage, &labels, true)?;
    fade(stage, &[syndrome_name], true)?;
    let (mut stack, mut result) = syndrome_stack(stage, k, word.value)?;
""",
)
edit(
    "F15 rows named as cosets; syndrome named",
    """    let mut old = stack.clone();
    old.push(result);
    old.extend(digits.iter().chain(&worth).copied());
    old.push(total);
    fade(stage, &old, false)?;
    let quotient_frame = frame(
""",
    """    let mut old = stack.clone();
    old.push(result);
    old.extend(digits.iter().chain(&worth).copied());
    old.push(total);
    old.push(syndrome_name);
    fade(stage, &old, false)?;
    let quotient_frame = frame(
""",
)
edit(
    "L1 lint: forgets line inside the safe area",
    """"forgets the column", 24., Palette::dim(), v(-0.55, -3.75))?;
""",
    """"forgets the column", 24., Palette::dim(), v(-0.55, -3.7))?;
""",
)
edit(
    "F16 the two column frames are named",
    """    play!(stage; show_creation(sent_column).rt(0.5), show_creation(wrong_column).rt(0.5));
    let verdict = k.text(
""",
    """    play!(stage; show_creation(sent_column).rt(0.5), show_creation(wrong_column).rt(0.5));
    let sent_name = k.text(
        stage,
        "sent",
        22.,
        Palette::green(),
        v(grid_at(0, sent_place.1).0, 0.03),
    )?;
    let wrong_name = k.text(stage, "decoded", 22., Palette::red(), v(grid_at(0, wrong.1).0, 0.03))?;
    let verdict = k.text(
""",
)
edit(
    "F16 the two column frames are named",
    """    fade(stage, &[verdict], true)?;

""",
    """    fade(stage, &[verdict, sent_name, wrong_name], true)?;

""",
)
edit(
    "F2 17_18 the parity lines sit over the case they describe",
    """    k.until_sentence(stage, "17_18", 1)?;
    let parity = frame(stage, (box_x(7) + 0.95, WORD_Y), (0.6, 0.7), Palette::purple(), 3.0)?;
""",
    """    k.until_sentence(stage, "17_18", 1)?;
    // Back to the two-flip word, the case the extra bit reports: four ones, parity bit
    // zero, so parity holds while the label is not 000. The miscorrected word has three.
    word.flip(stage, k, miscorrect, Palette::ink())?;
    d4g::glide(stage, ring, grid_at(landed.0, landed.1), 0.6)?;
    fade(stage, &[sent_column, wrong_column, sent_name, wrong_name], false)?;
    let parity = frame(stage, (box_x(7) + 0.95, WORD_Y), (0.6, 0.7), Palette::purple(), 3.0)?;
""",
)
edit(
    "F2 17_18 parity digit",
    """    fade(stage, &[parity, parity_name], true)?;
""",
    """    let parity_bit = k.text(
        stage,
        &(SENT.count_ones() % 2).to_string(),
        36.,
        Palette::purple(),
        v(box_x(7) + 0.95, WORD_Y),
    )?;
    fade(stage, &[parity, parity_name, parity_bit], true)?;
""",
)
edit(
    "L1 lint, F19 summary wording",
    """        "The quotient keeps exactly what it was built to keep",
        26.,
        Palette::ink(),
        v(0.0, -3.75),
""",
    """        "The quotient keeps exactly what it was designed to keep",
        26.,
        Palette::ink(),
        v(0.0, -3.7),
""",
)
edit(
    "F2 out list: faded columns leave it, parity digit joins it",
    """        quotient_frame,
        keeps,
        sent_column,
        wrong_column,
        parity,
        parity_name,
""",
    """        quotient_frame,
        keeps,
        parity,
        parity_name,
        parity_bit,
""",
)

edit(
    "F4, F6 phrase guard",
    """    #[test]
    fn worked_corruptions() {
""",
    """    #[test]
    fn every_waited_phrase_is_in_its_sentence() {
        use crate::axioms::{line_text, sentence_ranges};
        let waited: [(&str, usize, &str); 2] = [("17_3", 1, CONTRIBUTIONS_PHRASE), ("17_7", 1, SWAP_CHANGE_PHRASE)];
        for (cue, j, phrase) in waited {
            let text = line_text(cue).expect("cue in script");
            let (from, to) = sentence_ranges(text)[j];
            assert!(text[from..to].contains(phrase), "{cue} sentence {j}: {phrase}");
        }
    }
    #[test]
    fn worked_corruptions() {
""",
)


def main():
    before = BEFORE.read_bytes()
    assert hashlib.sha256(before).hexdigest() == BASE_SHA, "baseline is not the reviewed source"
    text = before.decode("utf-8")
    for finding, old, new in EDITS:
        count = text.count(old)
        assert count == 1, f"{finding}: expected one match, found {count}:\n{old}"
        text = text.replace(old, new)
    PROPOSED.write_bytes(text.encode("utf-8"))
    diff = difflib.unified_diff(
        before.decode("utf-8").splitlines(keepends=True),
        text.splitlines(keepends=True),
        fromfile=f"a/{LABEL}",
        tofile=f"b/{LABEL}",
        n=3,
    )
    PATCH.write_text("".join(diff), encoding="utf-8")
    print(f"{len(EDITS)} edits; wrote {PROPOSED.name} and {PATCH.name}")


if __name__ == "__main__":
    main()
