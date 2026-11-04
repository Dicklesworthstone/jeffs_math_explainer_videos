//! Chapter 17, applications. The ISBN-10 check (detection) is kept as
//! recorded; the second application is now Hamming's (7,4) code
//! (correction): all 128 seven-bit words are drawn as a grid whose columns
//! are the sixteen codewords and whose rows are the eight cosets, so a flipped
//! bit visibly moves a word down its column; the syndrome, computed on screen
//! as an XOR of positions, names the row; and two flips land in the row of a
//! third position, so the "repair" lands in a different column.
use crate::axioms::until_phrase;
use crate::d4_group as d4g;
use explainer_kit::{Kit, Palette, Timed, ink_style, play, stroke_style, tween, v};
use fmn::prelude::*;
pub struct Applications {
    pub kit: Kit,
}
const DIGITS: [i32; 10] = [0, 3, 0, 6, 4, 0, 6, 1, 5, 2];
/// The column walk starts when the contributions start being read.
const CONTRIBUTIONS_PHRASE: &str = "gives weighted contributions";
/// The swap's change appears when the voice reaches it.
const SWAP_CHANGE_PHRASE: &str = "changes the total";
/// One fade of the column walk: five of these and a short hold make 0.7 s a column.
const WALK_FADE: f64 = 0.1;
fn checksum(d: &[i32; 10]) -> i32 {
    d.iter().enumerate().map(|(i, x)| (10 - i as i32) * x).sum()
}
fn x(i: usize) -> f64 {
    -4.95 + i as f64 * 1.1
}
fn fade(stage: &mut Stage<'_>, items: &[Mob], show: bool) -> fmn::Result<()> {
    fade_for(stage, items, show, 0.35)
}
fn fade_for(stage: &mut Stage<'_>, items: &[Mob], show: bool, run_time: f64) -> fmn::Result<()> {
    let mut a: Vec<Box<dyn Animation>> = Vec::new();
    for &m in items {
        if !show {
            stage.clear_updaters(m, true);
        }
        let f = if show {
            fade_in(stage.arena_mut(), m, ORIGIN, 1.)?
        } else {
            fade_out(stage.arena_mut(), m, ORIGIN, 1.)?
        };
        a.push(stage.prepare(f.rt(run_time))?);
    }
    stage.play_prepared(a)?;
    Ok(())
}
fn outline(stage: &mut Stage<'_>, i: usize) -> Mob {
    stage.arena_mut().add(
        Polygon::new(vec![
            v(x(i) - 0.43, 1.95),
            v(x(i) + 0.43, 1.95),
            v(x(i) + 0.43, -0.25),
            v(x(i) - 0.43, -0.25),
        ])
        .style(stroke_style(Palette::hero(), 3.))
        .build(),
    )
}
impl SceneConstruct for Applications {
    fn name(&self) -> &str {
        "17_applications"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        k.say(stage, "17_1")?;
        let title = k.text(
            stage,
            "Can arithmetic catch a copying error?",
            36.,
            Palette::ink(),
            v(0., 3.),
        )?;
        let mut digits = Vec::new();
        for (i, d) in DIGITS.iter().enumerate() {
            digits.push(k.text(stage, &d.to_string(), 44., Palette::blue(), v(x(i), 1.55))?);
        }
        fade(stage, &[title], true)?;
        k.until_sentence(stage, "17_1", 1)?;
        fade(stage, &digits, true)?;
        k.until_sentence(stage, "17_1", 3)?;
        let note = k.text(
            stage,
            "ISBN-10: an error check, not a security system",
            25.,
            Palette::ink(),
            v(0., -2.8),
        )?;
        fade(stage, &[note], true)?;
        k.say(stage, "17_2")?;
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
            let next = k.text(
                stage,
                &format!("Running total: {sum}"),
                32.,
                Palette::ink(),
                v(0., -1.5),
            )?;
            fade_for(stage, &[total], false, WALK_FADE)?;
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
        k.say(stage, "17_4")?;
        let ring = outline(stage, 3);
        fade(stage, &[ring], true)?;
        let mut changed = DIGITS;
        changed[3] += 1;
        let digit = k.text(stage, &changed[3].to_string(), 44., Palette::red(), v(x(3), 1.55))?;
        let product = k.text(
            stage,
            &(changed[3] * 7).to_string(),
            28.,
            Palette::red(),
            v(x(3), -0.05),
        )?;
        // Sentence 0 changes the symbol and nothing else. The replaced handles leave
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
        old.extend(weights);
        old.extend(products);
        fade(stage, &old, false)?;
        let heading = k.text(
            stage,
            "Why no single changed symbol can hide",
            34.,
            Palette::ink(),
            v(0., 3.),
        )?;
        fade(stage, &[heading], true)?;
        k.until_sentence(stage, "17_5", 1)?;
        let rule = k.tex(stage, r"\Delta S=w\,\Delta d", 40., Palette::blue(), &[], v(0., 2.))?;
        fade(stage, &[rule], true)?;
        k.until_sentence(stage, "17_5", 3)?;
        // One row of the multiplication table: every nonzero change times the weight 7.
        let caption = k.text(
            stage,
            "Weight 7: nonzero changes give nonzero remainders",
            25.,
            Palette::hero(),
            v(0., 0.95),
        )?;
        let change_label = k.tex(stage, r"\Delta d", 24., Palette::blue(), &[], v(-6.05, 0.1))?;
        let product_label = k.tex(
            stage,
            r"7\Delta d\pmod{11}",
            20.,
            Palette::green(),
            &[],
            v(-6.05, -0.85),
        )?;
        fade(stage, &[caption, change_label, product_label], true)?;
        let mut row = vec![change_label, product_label];
        for d in 1..11 {
            let i = (d - 1) as usize;
            let a = k.text(stage, &d.to_string(), 28., Palette::blue(), v(x(i), 0.1))?;
            let b = k.text(
                stage,
                &((7 * d) % 11).to_string(),
                28.,
                Palette::green(),
                v(x(i), -0.85),
            )?;
            fade(stage, &[a, b], true)?;
            row.extend([a, b]);
        }
        k.until_sentence(stage, "17_5", 4)?;
        let condition = k.text(
            stage,
            "Prime modulus: two nonzero factors cannot make zero",
            27.,
            Palette::ink(),
            v(0., -2.2),
        )?;
        fade(stage, &[condition], true)?;
        k.say(stage, "17_6")?;
        let mut old = vec![heading, rule, caption, condition];
        old.extend(row);
        fade(stage, &old, false)?;
        let title = k.text(
            stage,
            "Why the extra assumption matters",
            34.,
            Palette::ink(),
            v(0., 3.),
        )?;
        fade(stage, &[title], true)?;
        let mut dots = Vec::new();
        for i in 0..6 {
            let angle = std::f64::consts::TAU * i as f64 / 6.;
            let p = v(1.6 * angle.sin(), 1.6 * angle.cos());
            let n = k.text(
                stage,
                &i.to_string(),
                32.,
                if i == 0 { Palette::green() } else { Palette::ink() },
                p,
            )?;
            dots.push(n);
        }
        fade(stage, &dots, true)?;
        let jump = k.text(stage, "+3", 32., Palette::hero(), v(2.8, 0.))?;
        fade(stage, &[jump], true)?;
        let at0 = stage.arena_mut().add(
            Circle::new()
                .radius(0.4)
                .style(stroke_style(Palette::hero(), 3.))
                .build()
                .moved_to(v(0., 1.6)),
        );
        fade(stage, &[at0], true)?;
        let m = at0
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.9),
                ..AnimateArgs::default()
            })?
            .move_to(v(0., -1.6), ORIGIN)?;
        stage.play(m)?;
        stage.wait(0.5)?;
        let m = at0
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.9),
                ..AnimateArgs::default()
            })?
            .move_to(v(0., 1.6), ORIGIN)?;
        stage.play(m)?;
        let fact = k.tex(stage, r"2\times3=0\pmod6", 36., Palette::red(), &[], v(0., -2.6))?;
        fade(stage, &[fact], true)?;
        k.say(stage, "17_7")?;
        let mut old = vec![title, jump, at0, fact];
        old.extend(dots);
        fade(stage, &old, false)?;
        let title = k.text(
            stage,
            "Swap the fourth and fifth symbols",
            34.,
            Palette::ink(),
            v(0., 3.),
        )?;
        fade(stage, &[title], true)?;
        let mut swap_digits = Vec::new();
        for (i, d) in DIGITS.iter().enumerate() {
            swap_digits.push(k.text(stage, &d.to_string(), 40., Palette::blue(), v(x(i), 1.5))?);
        }
        fade(stage, &swap_digits, true)?;
        // The weights stay put while the digits cross: they are the 7 and 6 of the formula.
        let swap_weights = [
            k.text(stage, "× 7", 25., Palette::hero(), v(x(3), 0.35))?,
            k.text(stage, "× 6", 25., Palette::hero(), v(x(4), 0.35))?,
        ];
        fade(stage, &swap_weights, true)?;
        k.until_sentence(stage, "17_7", 1)?;
        let before = k.tex(stage, r"7\times6+6\times4=66", 34., Palette::ink(), &[], v(0., -0.2))?;
        fade(stage, &[before], true)?;
        // Separate vertical lanes keep the crossing digits legible.
        for (i, y) in [(3, 2.1), (4, 0.9)] {
            let a = swap_digits[i]
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.4),
                    ..AnimateArgs::default()
                })?
                .move_to(v(x(i), y), ORIGIN)?;
            stage.play(a)?;
        }
        let a = swap_digits[3]
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.),
                ..AnimateArgs::default()
            })?
            .move_to(v(x(4), 2.1), ORIGIN)?;
        let b = swap_digits[4]
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.),
                ..AnimateArgs::default()
            })?
            .move_to(v(x(3), 0.9), ORIGIN)?;
        let a = stage.prepare(a)?;
        let b = stage.prepare(b)?;
        stage.play_prepared(vec![a, b])?;
        for (i, dest) in [(3, 4), (4, 3)] {
            let a = swap_digits[i]
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.4),
                    ..AnimateArgs::default()
                })?
                .move_to(v(x(dest), 1.5), ORIGIN)?;
            stage.play(a)?;
        }
        let after = k.tex(stage, r"7\times4+6\times6=64", 34., Palette::red(), &[], v(0., -1.2))?;
        fade(stage, &[after], true)?;
        until_phrase(stage, &k, "17_7", 1, SWAP_CHANGE_PHRASE)?;
        let delta = k.tex(stage, r"\Delta S=(7-6)(4-6)=-2", 32., Palette::hero(), &[], v(0., -2.3))?;
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
        let mut shown = Vec::new();
        let mut weights = Vec::new();
        for (i, d) in DIGITS.iter().enumerate() {
            shown.push(k.text(stage, &d.to_string(), 40., Palette::blue(), v(x(i), 1.9))?);
            weights.push(k.text(stage, &format!("× {}", 10 - i), 25., Palette::hero(), v(x(i), 1.2))?);
        }
        fade(stage, &shown, true)?;
        fade(stage, &weights, true)?;
        let mut total = k.text(
            stage,
            &format!("Weighted total: {}", checksum(&corrupt)),
            29.,
            Palette::ink(),
            v(-2.8, -0.1),
        )?;
        fade(stage, &[total], true)?;
        let mut wheel = Vec::new();
        for n in 0..11 {
            let t = std::f64::consts::TAU * f64::from(n) / 11.;
            wheel.push(k.text(
                stage,
                &n.to_string(),
                25.,
                if n == 0 { Palette::green() } else { Palette::ink() },
                v(3. + 1.4 * t.sin(), -0.6 + 1.4 * t.cos()),
            )?);
        }
        fade(stage, &wheel, true)?;
        let remainder = stage.add_value_tracker(0.);
        let marker = stage.always_redraw(move |arena| {
            let t = arena.tracker_value(remainder).unwrap_or(0.) * std::f64::consts::TAU / 11.;
            arena.add(
                Circle::new()
                    .radius(0.13)
                    .style(ink_style(Palette::hero()))
                    .build()
                    .moved_to(v(3. + 1.05 * t.sin(), -0.6 + 1.05 * t.cos())),
            )
        });
        stage.add_to_scene(marker)?;
        let legend = k.text(stage, "Remainder modulo 11", 23., Palette::ink(), v(3., -2.4))?;
        fade(stage, &[legend], true)?;
        k.until_sentence(stage, "17_8", 2)?;
        let title = k.text(stage, "Two errors can cancel", 36., Palette::ink(), v(0., 3.))?;
        fade(stage, &[title], true)?;
        k.until_sentence(stage, "17_8", 3)?;
        let initial = checksum(&corrupt);
        corrupt[0] = 1;
        let first = checksum(&corrupt);
        let first_delta = first - initial;
        let next = k.text(stage, &corrupt[0].to_string(), 40., Palette::red(), v(x(0), 1.9))?;
        fade(stage, &[shown[0]], false)?;
        shown[0] = next;
        fade(stage, &[next], true)?;
        let delta = k.text(
            stage,
            &format!("First change adds {first_delta}"),
            28.,
            Palette::blue(),
            v(-2.8, -1.1),
        )?;
        fade(stage, &[delta], true)?;
        let next = k.text(
            stage,
            &format!("Weighted total: {first}"),
            29.,
            Palette::red(),
            v(-2.8, -0.1),
        )?;
        fade(stage, &[total], false)?;
        total = next;
        fade(stage, &[total], true)?;
        stage.play(tween(remainder, f64::from(first_delta % 11), 1.5)?)?;
        k.until_sentence(stage, "17_8", 4)?;
        corrupt[1] = 8;
        let final_total = checksum(&corrupt);
        let second_delta = final_total - first;
        let next = k.text(stage, &corrupt[1].to_string(), 40., Palette::red(), v(x(1), 1.9))?;
        fade(stage, &[shown[1]], false)?;
        shown[1] = next;
        fade(stage, &[next], true)?;
        fade(stage, &[delta], false)?;
        let delta = k.text(
            stage,
            &format!("+{second_delta} = {} × 11 + {}", second_delta / 11, second_delta % 11),
            28.,
            Palette::hero(),
            v(-2.8, -1.1),
        )?;
        fade(stage, &[delta], true)?;
        let next = k.text(
            stage,
            &format!("Weighted total: {final_total}"),
            29.,
            Palette::ink(),
            v(-2.8, -0.1),
        )?;
        fade(stage, &[total], false)?;
        total = next;
        fade(stage, &[total], true)?;
        stage.play(tween(remainder, f64::from(first_delta % 11 + second_delta % 11), 1.5)?)?;
        k.until_sentence(stage, "17_8", 5)?;
        let limit = k.text(
            stage,
            &format!("Two changed digits. Remainder {}. The error hides.", final_total % 11),
            27.,
            Palette::red(),
            v(0., -3.),
        )?;
        fade(stage, &[limit], true)?;
        k.say(stage, "17_14")?;
        // 17_9 .. 17_18: Hamming's code, a quotient that repairs one flipped bit.
        k.say(stage, "17_9")?;
        let mut old = vec![title, total, delta, legend, marker, limit];
        old.extend(shown);
        old.extend(weights);
        old.extend(wheel);
        fade(stage, &old, false)?;
        let hamming = hamming_film(stage, &k)?;
        k.say(stage, "17_11")?;
        fade(stage, &hamming, false)?;
        let title = k.text(stage, "Follow the operation", 36., Palette::ink(), v(0., 3.))?;
        fade(stage, &[title], true)?;
        let mut clock = Vec::new();
        for n in 0..12 {
            let t = std::f64::consts::TAU * f64::from(n) / 12.;
            let p = v(-3. + 2.05 * t.sin(), 0.3 + 2.05 * t.cos());
            let label = k.text(stage, &n.to_string(), 29., Palette::ink(), p)?;
            clock.push(label);
        }
        fade(stage, &clock, true)?;
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
        let prompts = [
            "1. Which residues does 6 generate on the twelve-clock?",
            "2. Can a four-clock be isomorphic to two two-position clocks?",
            "3. Why does changing representative within a coset keep it?",
        ];
        let mut quiz = Vec::new();
        for (i, p) in prompts.iter().enumerate() {
            k.until_sentence(stage, "17_12", i + 1)?;
            let m = k.text_left(stage, p, 27., Palette::ink(), v(-3.6, 2. - i as f64 * 1.25))?;
            fade(stage, &[m], true)?;
            quiz.push(m);
        }
        k.hold(stage)?;
        stage.wait(5.)?;
        k.say(stage, "17_13")?;
        fade(stage, &quiz, false)?;
        let title = k.text(
            stage,
            "Answers you can test with the moves",
            34.,
            Palette::ink(),
            v(0., 3.),
        )?;
        fade(stage, &[title], true)?;
        let mut answer = Vec::new();
        for n in 0..12 {
            let t = std::f64::consts::TAU * f64::from(n) / 12.;
            let m = k.text(
                stage,
                &n.to_string(),
                27.,
                if n == 0 || n == 6 {
                    Palette::hero()
                } else {
                    Palette::dim()
                },
                v(-3. + 1.5 * t.sin(), 0.3 + 1.5 * t.cos()),
            )?;
            answer.push(m);
        }
        fade(stage, &answer, true)?;
        let selection = stage.arena_mut().add(
            Circle::new()
                .radius(0.35)
                .style(stroke_style(Palette::hero(), 3.))
                .build()
                .moved_to(v(-3., 1.8)),
        );
        fade(stage, &[selection], true)?;
        for y in [-1.2, 1.8] {
            let a = selection
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.8),
                    ..AnimateArgs::default()
                })?
                .move_to(v(-3., y), ORIGIN)?;
            stage.play(a)?;
            stage.wait(0.4)?;
        }
        k.until_sentence(stage, "17_13", 1)?;
        let a = k.text(
            stage,
            "Four-clock: a move needs four repeats",
            25.,
            Palette::blue(),
            v(2., 1.4),
        )?;
        let b = k.text(
            stage,
            "Two toggles: every move repeats in at most two",
            24.,
            Palette::hero(),
            v(2., 0.3),
        )?;
        fade(stage, &[a, b], true)?;
        k.until_sentence(stage, "17_13", 2)?;
        let c = k.tex(stage, r"hH=H\quad(h\in H)", 32., Palette::green(), &[], v(2., -1.))?;
        fade(stage, &[c], true)?;
        let d = k.text(
            stage,
            "Closure keeps it in. The inverse brings everything back.",
            23.,
            Palette::ink(),
            v(0., -2.5),
        )?;
        fade(stage, &[d], true)?;
        k.finish(stage)
    }
}

/// Hamming(7,4). Positions 1..=7 are written left to right; position p is
/// bit 7 − p of the byte. A word's syndrome is the XOR of the positions that
/// hold a one; the sixteen codewords are the words with syndrome zero.
pub(crate) const SENT: u8 = 0b0110011;
const OTHER_CODEWORD: u8 = 0b1110000;
pub(crate) const SINGLE_ERROR: u8 = 5;
pub(crate) const DOUBLE_ERROR: [u8; 2] = [2, 5];
const WORD_Y: f64 = 2.2;
/// Check bits sit at the powers of two; the other four positions carry data.
pub(crate) const CHECK_POSITIONS: [u8; 3] = [1, 2, 4];

fn has_one(word: u8, p: u8) -> bool {
    (word >> (7 - p)) & 1 == 1
}

pub(crate) fn flip(word: u8, p: u8) -> u8 {
    word ^ (1 << (7 - p))
}

pub(crate) fn syndrome(word: u8) -> u8 {
    (1..=7).filter(|&p| has_one(word, p)).fold(0, |s, p| s ^ p)
}

pub(crate) fn codewords() -> Vec<u8> {
    (0..128_u8).filter(|&w| syndrome(w) == 0).collect()
}

pub(crate) fn bits(word: u8) -> String {
    (1..=7).map(|p| if has_one(word, p) { '1' } else { '0' }).collect()
}

pub(crate) fn binary3(n: u8) -> String {
    format!("{n:03b}")
}

/// What the three places of a label are worth, left to right (17_16).
pub(crate) const PLACE_WEIGHTS: [u8; 3] = [4, 2, 1];
const PLACE_X: [f64; 3] = [5.0, 5.6, 6.2];
const PLACE_Y: f64 = 1.95;

/// Each digit of `label`'s three-bit name times its place's worth.
pub(crate) fn worths(label: u8) -> [u8; 3] {
    PLACE_WEIGHTS.map(|weight| if (label & weight) != 0 { weight } else { 0 })
}

/// The extended code of 17_18: each codeword plus an overall parity bit at
/// position 8. Returns how many single flips (of 8 positions) the decoder
/// corrects and how many double flips it reports instead of "repairing",
/// over every codeword. The decoder: parity broken means one flip, repaired
/// by its label (label 000: the parity bit itself); parity intact with a
/// nonzero label means two flips, reported.
pub(crate) fn extended_code_counts() -> (usize, usize) {
    let (mut corrected, mut reported) = (0, 0);
    for c in codewords() {
        let stored = c.count_ones() % 2 == 1;
        let corrupt = |positions: &[u8]| {
            positions
                .iter()
                .fold((c, stored), |(w, p), &q| if q == 8 { (w, !p) } else { (flip(w, q), p) })
        };
        let parity_broken = |(w, p): (u8, bool)| (w.count_ones() % 2 == 1) != p;
        for a in 1..=8_u8 {
            let once = corrupt(&[a]);
            let label = syndrome(once.0);
            let repaired = if label == 0 { once.0 } else { flip(once.0, label) };
            if parity_broken(once) && repaired == c {
                corrected += 1;
            }
            for b in a + 1..=8 {
                let twice = corrupt(&[a, b]);
                if !parity_broken(twice) && syndrome(twice.0) != 0 {
                    reported += 1;
                }
            }
        }
    }
    (corrected, reported)
}

/// The place-value key: the worths four, two, one over three empty slots.
fn place_key(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Vec<Mob>> {
    let mut parts = Vec::new();
    for (&weight, &x) in PLACE_WEIGHTS.iter().zip(&PLACE_X) {
        parts.push(k.text(stage, &weight.to_string(), 24., Palette::dim(), v(x, PLACE_Y + 0.55))?);
        parts.push(frame(stage, (x, PLACE_Y), (0.5, 0.6), Palette::hero(), 3.0)?);
    }
    fade(stage, &parts, true)?;
    Ok(parts)
}

/// `label`'s digits, dropped into the slots.
fn place_digits(stage: &mut Stage<'_>, k: &Kit, label: u8) -> fmn::Result<Vec<Mob>> {
    let mut digits = Vec::new();
    for (digit, &x) in binary3(label).chars().zip(&PLACE_X) {
        digits.push(k.text(stage, &digit.to_string(), 32., Palette::hero(), v(x, PLACE_Y))?);
    }
    fade(stage, &digits, true)?;
    Ok(digits)
}

/// What each digit is worth, beneath its slot.
fn place_worths(stage: &mut Stage<'_>, k: &Kit, label: u8) -> fmn::Result<Vec<Mob>> {
    let mut worth = Vec::new();
    for (w, &x) in worths(label).into_iter().zip(&PLACE_X) {
        let color = if w == 0 { Palette::dim() } else { Palette::gold() };
        worth.push(k.text(stage, &w.to_string(), 24., color, v(x, PLACE_Y - 0.6))?);
    }
    fade(stage, &worth, true)?;
    Ok(worth)
}

/// The worths added: "4 + 0 + 1 = 5".
fn place_total(stage: &mut Stage<'_>, k: &Kit, label: u8) -> fmn::Result<Mob> {
    let [a, b, c] = worths(label);
    let line = k.text(
        stage,
        &format!("{a} + {b} + {c} = {}", a + b + c),
        28.,
        Palette::hero(),
        v(PLACE_X[1], PLACE_Y - 1.15),
    )?;
    fade(stage, &[line], true)?;
    Ok(line)
}

/// The coset representative for a syndrome: no error, or the one flip it names.
fn leader(s: u8) -> u8 {
    if s == 0 { 0 } else { flip(0, s) }
}

/// (row, column) of a word in the coset grid: its syndrome, and the
/// codeword it differs from by the row's single flip.
fn grid_place(word: u8) -> (usize, usize) {
    let s = syndrome(word);
    let c = word ^ leader(s);
    let column = codewords()
        .iter()
        .position(|&x| x == c)
        .expect("word plus leader is a codeword");
    (usize::from(s), column)
}

fn grid_at(row: usize, col: usize) -> (f64, f64) {
    (-5.2 + 0.62 * col as f64, -0.45 - 0.4 * row as f64)
}

fn box_x(p: u8) -> f64 {
    -5.4 + 0.75 * f64::from(p - 1)
}

fn frame(stage: &mut Stage<'_>, at: (f64, f64), size: (f64, f64), color: Srgb, width: f64) -> fmn::Result<Mob> {
    let shape = Rectangle::new()
        .width(size.0)
        .height(size.1)
        .corner_radius(0.05)
        .style(stroke_style(color, width))
        .build()?
        .moved_to(v(at.0, at.1));
    Ok(stage.arena_mut().add(shape))
}

/// The seven-bit word on screen, one box and one digit per position.
struct Word {
    value: u8,
    fixed: Vec<Mob>,
    digits: Vec<Mob>,
}

impl Word {
    fn new(stage: &mut Stage<'_>, k: &Kit, value: u8) -> fmn::Result<Self> {
        let mut fixed = Vec::new();
        let mut digits = Vec::new();
        for p in 1..=7_u8 {
            let color = if CHECK_POSITIONS.contains(&p) {
                Palette::gold()
            } else {
                Palette::blue()
            };
            fixed.push(frame(stage, (box_x(p), WORD_Y), (0.6, 0.7), color, 3.0)?);
            fixed.push(k.text(stage, &p.to_string(), 24., Palette::dim(), v(box_x(p), WORD_Y - 0.6))?);
            let digit = if has_one(value, p) { "1" } else { "0" };
            digits.push(k.text(stage, digit, 36., Palette::ink(), v(box_x(p), WORD_Y))?);
        }
        Ok(Self { value, fixed, digits })
    }

    fn all(&self) -> Vec<Mob> {
        let mut out = self.fixed.clone();
        out.extend(self.digits.iter().copied());
        out
    }

    fn flip(&mut self, stage: &mut Stage<'_>, k: &Kit, p: u8, color: Srgb) -> fmn::Result<()> {
        self.value = flip(self.value, p);
        let index = usize::from(p - 1);
        let digit = if has_one(self.value, p) { "1" } else { "0" };
        let next = k.text(stage, digit, 36., color, v(box_x(p), WORD_Y))?;
        play!(stage; fmn::animation::replacement_transform(self.digits[index], next).rt(0.5));
        self.digits[index] = next;
        Ok(())
    }
}

/// The XOR of the positions holding a one, written out as three-bit rows.
fn syndrome_stack(stage: &mut Stage<'_>, k: &Kit, word: u8) -> fmn::Result<(Vec<Mob>, Mob)> {
    let heading = k.text(stage, "positions holding a 1", 22., Palette::dim(), v(3.2, 2.8))?;
    fade(stage, &[heading], true)?;
    let mut rows = vec![heading];
    let ones: Vec<u8> = (1..=7).filter(|&p| has_one(word, p)).collect();
    for (i, &p) in ones.iter().enumerate() {
        let y = 2.4 - 0.38 * i as f64;
        let row = k.text(
            stage,
            &format!("{p}  →  {}", binary3(p)),
            26.,
            Palette::ink(),
            v(3.2, y),
        )?;
        fade(stage, &[row], true)?;
        rows.push(row);
    }
    let bottom = 2.4 - 0.38 * ones.len() as f64 + 0.12;
    let rule = stage.arena_mut().add(
        Line::new(v(2.4, bottom), v(4.2, bottom))
            .style(stroke_style(Palette::dim(), 2.0))
            .build()?,
    );
    let result = k.text(
        stage,
        &format!("XOR  =  {}", binary3(syndrome(word))),
        28.,
        Palette::hero(),
        v(3.1, bottom - 0.3),
    )?;
    fade(stage, &[rule, result], true)?;
    rows.push(rule);
    Ok((rows, result))
}

fn hamming_film(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Vec<Mob>> {
    let codes = codewords();
    let title = k.text(
        stage,
        "A code that repairs one flipped bit",
        34.,
        Palette::ink(),
        v(0., 3.35),
    )?;
    fade(stage, &[title], true)?;
    let mut word = Word::new(stage, k, SENT)?;
    fade(stage, &word.all(), true)?;
    k.until_sentence(stage, "17_9", 1)?;
    let data_note = k.text(
        stage,
        "Blue: four data bits. Gold: three check bits.",
        24.,
        Palette::ink(),
        v(-3.15, 1.0),
    )?;
    fade(stage, &[data_note], true)?;
    k.until_sentence(stage, "17_9", 2)?;
    // All 128 words: columns are the sixteen codewords, rows the eight cosets.
    let mut cells: Vec<Vec<Mob>> = Vec::new();
    for row in 0..8 {
        let mut one_row = Vec::new();
        for col in 0..codes.len() {
            one_row.push(frame(stage, grid_at(row, col), (0.5, 0.3), Palette::faint(), 2.0)?);
        }
        cells.push(one_row);
    }
    let all_cells: Vec<Mob> = cells.iter().flatten().copied().collect();
    fade(stage, &all_cells, true)?;
    let mut fills: Vec<Box<dyn Animation>> = Vec::new();
    for &cell in &cells[0] {
        fills.push(stage.prepare(cell.animate().set_fill(Some(Palette::green()), Some(0.7))?)?);
    }
    let fills = explainer_kit::lagged(stage, fills, 0.05)?;
    play!(stage; fills);
    let count = k.tex(
        stage,
        &format!(r"{}\ \text{{of}}\ 2^7={}", codes.len(), 1_u32 << 7),
        32.,
        Palette::green(),
        &[],
        v(3.3, 1.0),
    )?;
    fade(stage, &[count], true)?;
    k.until_sentence(stage, "17_9", 3)?;
    let sum = SENT ^ OTHER_CODEWORD;
    let xor_rows = [
        k.text(stage, &bits(SENT), 30., Palette::ink(), v(3.3, 2.5))?,
        k.text(
            stage,
            &format!("XOR  {}", bits(OTHER_CODEWORD)),
            30.,
            Palette::ink(),
            v(2.95, 2.05),
        )?,
        k.text(
            stage,
            &format!("=  {}", bits(sum)),
            30.,
            if codes.contains(&sum) {
                Palette::green()
            } else {
                Palette::red()
            },
            v(3.1, 1.6),
        )?,
    ];
    fade(stage, &[count], false)?;
    fade(stage, &xor_rows, true)?;

    // 17_15: eight cosets of sixteen; eight situations to repair.
    k.say(stage, "17_15")?;
    fade(stage, &xor_rows, false)?;
    fade(stage, &[data_note], false)?;
    k.until_sentence(stage, "17_15", 1)?;
    let rows_are = k.text(
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
        stage.play_prepared(fills)?;
    }
    k.until_sentence(stage, "17_15", 2)?;
    let cosets = (0..128_u8)
        .map(syndrome)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let lagrange = k.tex(
        stage,
        &format!(r"{}/{}={}", 1_u32 << 7, codes.len(), cosets),
        34.,
        Palette::ink(),
        &[],
        v(3.3, 1.0),
    )?;
    fade(stage, &[lagrange], true)?;
    k.until_sentence(stage, "17_15", 3)?;
    let mut situations = Vec::new();
    for row in 0..8_u8 {
        let text = if row == 0 {
            "none".to_owned()
        } else {
            format!("bit {row}")
        };
        situations.push(k.text(
            stage,
            &text,
            24.,
            Palette::ink(),
            v(-6.35, grid_at(usize::from(row), 0).1),
        )?);
    }
    k.until_sentence(stage, "17_15", 4)?;
    let mut appear: Vec<Box<dyn Animation>> = Vec::new();
    for &m in &situations {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.4);
        appear.push(stage.prepare(a)?);
    }
    let appear = explainer_kit::lagged(stage, appear, 0.2)?;
    play!(stage; appear);

    // 17_16: one flip; the syndrome names the row and the position.
    k.say(stage, "17_16")?;
    fade(stage, &[lagrange, rows_are], false)?;
    let sent_place = grid_place(SENT);
    let ring = frame(
        stage,
        grid_at(sent_place.0, sent_place.1),
        (0.62, 0.4),
        Palette::hero(),
        5.0,
    )?;
    fade(stage, &[ring], true)?;
    word.flip(stage, k, SINGLE_ERROR, Palette::gold())?;
    let received = grid_place(word.value);
    d4g::glide(stage, ring, grid_at(received.0, received.1), 1.2)?;
    k.until_sentence(stage, "17_16", 1)?;
    let mut labels = Vec::new();
    for row in 0..8_u8 {
        labels.push(k.text(
            stage,
            &binary3(row),
            24.,
            Palette::dim(),
            v(4.95, grid_at(usize::from(row), 0).1),
        )?);
    }
    let syndrome_name = k.text(stage, "syndrome", 22., Palette::dim(), v(4.95, 0.1))?;
    fade(stage, &labels, true)?;
    fade(stage, &[syndrome_name], true)?;
    let (mut stack, mut result) = syndrome_stack(stage, k, word.value)?;
    // Place values first; only then is the label read as a number.
    k.until_sentence(stage, "17_16", 3)?;
    let key = place_key(stage, k)?;
    k.until_sentence(stage, "17_16", 4)?;
    let pulse = fmn::animation::indicate(stage.arena_mut(), result, 1.25, None)?.rt(0.8);
    play!(stage; pulse);
    let mut digits = place_digits(stage, k, syndrome(word.value))?;
    k.until_sentence(stage, "17_16", 5)?;
    let mut worth = place_worths(stage, k, syndrome(word.value))?;
    k.until_sentence(stage, "17_16", 6)?;
    let mut total = place_total(stage, k, syndrome(word.value))?;
    let row_label = labels[usize::from(syndrome(word.value))];
    play!(stage; row_label.animate().set_color(Palette::hero())?);
    k.until_sentence(stage, "17_16", 7)?;
    let repair = syndrome(word.value);
    word.flip(stage, k, repair, Palette::green())?;
    let back = grid_place(word.value);
    d4g::glide(stage, ring, grid_at(back.0, back.1), 1.2)?;
    play!(stage; row_label.animate().set_color(Palette::dim())?);

    // 17_17: the syndrome is the map onto the quotient; two flips fool it.
    k.say(stage, "17_17")?;
    let mut old = stack.clone();
    old.push(result);
    old.extend(digits.iter().chain(&worth).copied());
    old.push(total);
    old.push(syndrome_name);
    fade(stage, &old, false)?;
    let quotient_frame = frame(
        stage,
        (4.95, grid_at(0, 0).1 - 0.4 * 3.5),
        (0.9, 3.35),
        Palette::hero(),
        4.0,
    )?;
    let keeps = k.text(stage, "keeps the row", 24., Palette::hero(), v(5.15, 0.1))?;
    play!(stage; show_creation(quotient_frame).rt(0.6));
    fade(stage, &[keeps], true)?;
    k.until_sentence(stage, "17_17", 1)?;
    let forgets = k.text(stage, "forgets the column", 24., Palette::dim(), v(-0.55, -3.7))?;
    fade(stage, &[forgets], true)?;
    k.until_sentence(stage, "17_17", 3)?;
    for p in DOUBLE_ERROR {
        word.flip(stage, k, p, Palette::gold())?;
    }
    let landed = grid_place(word.value);
    d4g::glide(stage, ring, grid_at(landed.0, landed.1), 1.2)?;
    (stack, result) = syndrome_stack(stage, k, word.value)?;
    k.until_sentence(stage, "17_17", 4)?;
    // The same key reads this label too: one four, one two, one one.
    digits = place_digits(stage, k, syndrome(word.value))?;
    worth = place_worths(stage, k, syndrome(word.value))?;
    total = place_total(stage, k, syndrome(word.value))?;
    let miscorrect = syndrome(word.value);
    word.flip(stage, k, miscorrect, Palette::red())?;
    let wrong = grid_place(word.value);
    d4g::glide(stage, ring, grid_at(wrong.0, wrong.1), 1.2)?;
    let sent_column = frame(
        stage,
        (grid_at(0, sent_place.1).0, grid_at(0, 0).1 - 0.4 * 3.5),
        (0.58, 3.3),
        Palette::green(),
        4.0,
    )?;
    let wrong_column = frame(
        stage,
        (grid_at(0, wrong.1).0, grid_at(0, 0).1 - 0.4 * 3.5),
        (0.58, 3.3),
        Palette::red(),
        4.0,
    )?;
    play!(stage; show_creation(sent_column).rt(0.5), show_creation(wrong_column).rt(0.5));
    let sent_name = k.text(
        stage,
        "sent",
        22.,
        Palette::green(),
        v(grid_at(0, sent_place.1).0, 0.03),
    )?;
    let wrong_name = k.text(stage, "decoded", 22., Palette::red(), v(grid_at(0, wrong.1).0, 0.03))?;
    let verdict = k.text(
        stage,
        &format!("decoded {}, but sent {}", bits(word.value), bits(SENT)),
        26.,
        Palette::red(),
        v(-1.8, 1.1),
    )?;
    fade(stage, &[verdict, sent_name, wrong_name], true)?;

    // 17_18: related codes guard memory; one more check bit turns a
    // miscorrection into a report (`extended_code_counts` checks every case).
    k.say(stage, "17_18")?;
    fade(stage, &[verdict], false)?;
    let memory = k.text(
        stage,
        "Related codes: long used in computer memory",
        24.,
        Palette::dim(),
        v(-1.8, 1.1),
    )?;
    fade(stage, &[memory], true)?;
    k.until_sentence(stage, "17_18", 1)?;
    // Back to the two-flip word, the case the extra bit reports: four ones, parity bit
    // zero, so parity holds while the label is not 000. The miscorrected word has three.
    word.flip(stage, k, miscorrect, Palette::ink())?;
    d4g::glide(stage, ring, grid_at(landed.0, landed.1), 0.6)?;
    fade(stage, &[sent_column, wrong_column, sent_name, wrong_name], false)?;
    let parity = frame(stage, (box_x(7) + 0.95, WORD_Y), (0.6, 0.7), Palette::purple(), 3.0)?;
    let parity_name = k.text(
        stage,
        "parity",
        24.,
        Palette::purple(),
        v(box_x(7) + 0.95, WORD_Y - 0.6),
    )?;
    let parity_bit = k.text(
        stage,
        &(SENT.count_ones() % 2).to_string(),
        36.,
        Palette::purple(),
        v(box_x(7) + 0.95, WORD_Y),
    )?;
    fade(stage, &[parity, parity_name, parity_bit], true)?;
    let one_flip = k.text(
        stage,
        "One flip: parity breaks, so repair it",
        24.,
        Palette::purple(),
        v(-1.8, 0.55),
    )?;
    fade(stage, &[one_flip], true)?;
    let two_flips = k.text(
        stage,
        "Two flips: parity holds, label not 000, so report",
        24.,
        Palette::purple(),
        v(-1.8, 0.05),
    )?;
    fade(stage, &[two_flips], true)?;
    k.until_sentence(stage, "17_18", 2)?;
    let summary = k.text(
        stage,
        "The quotient keeps exactly what it was designed to keep",
        26.,
        Palette::ink(),
        v(0.0, -3.7),
    )?;
    fade(stage, &[forgets], false)?;
    fade(stage, &[summary], true)?;

    let mut out = vec![
        title,
        ring,
        quotient_frame,
        keeps,
        parity,
        parity_name,
        parity_bit,
        memory,
        one_flip,
        two_flips,
        summary,
        result,
        total,
    ];
    out.extend(key);
    out.extend(digits);
    out.extend(worth);
    out.extend(word.all());
    out.extend(all_cells);
    out.extend(situations);
    out.extend(labels);
    out.extend(stack);
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
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
        assert_eq!(checksum(&DIGITS), 132);
        let mut d = DIGITS;
        d[3] = 7;
        assert_eq!(checksum(&d), 139);
        d = DIGITS;
        d.swap(3, 4);
        assert_eq!(checksum(&d), 130);
        d = DIGITS;
        d[0] = 1;
        d[1] = 8;
        assert_eq!(checksum(&d), 187);
        assert_eq!(checksum(&d) % 11, 0);
    }
    #[test]
    fn all_nonzero_weights_detect_all_nonzero_differences() {
        for w in 1..11 {
            for d in 1..11 {
                assert_ne!((w * d) % 11, 0);
            }
        }
    }

    #[test]
    fn hamming_codewords_form_a_subgroup_of_sixteen_with_eight_cosets() {
        let codes = codewords();
        assert_eq!(codes.len(), 16);
        assert!(codes.contains(&0));
        for &a in &codes {
            for &b in &codes {
                assert!(codes.contains(&(a ^ b)), "closed under exclusive or");
            }
        }
        // Every word lies in exactly one coset; each coset has sixteen words.
        for s in 0..8 {
            assert_eq!((0..128_u8).filter(|&w| syndrome(w) == s).count(), 16);
        }
        // The eight repairable situations land in eight different cosets.
        let situations: std::collections::BTreeSet<u8> = (0..8).map(|s| syndrome(leader(s))).collect();
        assert_eq!(situations.len(), 8);
        // The grid puts every word in its own cell.
        let places: std::collections::BTreeSet<(usize, usize)> = (0..128_u8).map(grid_place).collect();
        assert_eq!(places.len(), 128);
    }

    #[test]
    fn narrated_single_error_is_repaired() {
        assert_eq!(bits(SENT), "0110011");
        assert_eq!(syndrome(SENT), 0);
        let received = flip(SENT, SINGLE_ERROR);
        assert_eq!(bits(received), "0110111");
        assert_eq!(binary3(syndrome(received)), "101");
        assert_eq!(syndrome(received), SINGLE_ERROR);
        assert_eq!(flip(received, syndrome(received)), SENT);
        // Same column, row of the flipped position.
        assert_eq!(grid_place(received), (usize::from(SINGLE_ERROR), grid_place(SENT).1));
        // Every single flip of every codeword is repaired the same way.
        for &c in &codewords() {
            for p in 1..=7 {
                assert_eq!(syndrome(flip(c, p)), p);
            }
        }
    }

    #[test]
    fn narrated_double_error_is_miscorrected_to_another_codeword() {
        let received = DOUBLE_ERROR.iter().fold(SENT, |w, &p| flip(w, p));
        assert_eq!(bits(received), "0010111");
        assert_eq!(syndrome(received), 7);
        let decoded = flip(received, syndrome(received));
        assert_eq!(bits(decoded), "0010110");
        assert_eq!(syndrome(decoded), 0);
        assert_ne!(decoded, SENT);
        assert_ne!(grid_place(decoded).1, grid_place(SENT).1);
        // Any two flips: nonzero syndrome, never the original codeword after "repair".
        for &c in &codewords() {
            for p in 1..=7 {
                for q in (p + 1)..=7 {
                    let w = flip(flip(c, p), q);
                    assert_ne!(syndrome(w), 0);
                    assert_ne!(flip(w, syndrome(w)), c);
                }
            }
        }
        // The XOR demonstration stays inside the code.
        assert!(codewords().contains(&OTHER_CODEWORD));
        assert!(codewords().contains(&(SENT ^ OTHER_CODEWORD)));
    }

    #[test]
    fn place_values_read_every_label_and_the_two_narrated_ones() {
        for label in 0..8_u8 {
            let w = worths(label);
            assert_eq!(w.iter().sum::<u8>(), label);
            for ((digit, weight), worth) in binary3(label).chars().zip(PLACE_WEIGHTS).zip(w) {
                assert_eq!(worth, if digit == '1' { weight } else { 0 });
            }
        }
        assert_eq!(worths(syndrome(flip(SENT, SINGLE_ERROR))), [4, 0, 1]);
        let garbled = DOUBLE_ERROR.iter().fold(SENT, |w, &p| flip(w, p));
        assert_eq!(worths(syndrome(garbled)), [4, 2, 1]);
    }

    #[test]
    fn extended_code_corrects_every_single_flip_and_reports_every_double_flip() {
        let codes = codewords().len();
        assert_eq!(extended_code_counts(), (codes * 8, codes * 8 * 7 / 2));
    }
}
