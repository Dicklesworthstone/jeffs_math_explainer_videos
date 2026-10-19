//! Chapter 05, why written proofs hide the search: chapter 04's last
//! picture, then a finished proof of continuity for x² at x = 1/2 on the
//! domain [0, 1], whose first line chooses δ = (2/3)ε without saying why
//! (05_1). The search behind it runs backward from the goal: x² must be
//! within ε of 1/4 for x near 1/2 (05_2); the gap factors as
//! |x − 1/2|·|x + 1/2| (05_3); on the domain the second factor is at most
//! 3/2, a deliberate, generous step, since near 1/2 it is about 1 (05_4);
//! so a distance below (2/3)ε is enough, and that is δ (05_5). The
//! prediction: against 1/10, where the window found is 1/15, about how large
//! could δ be (05_6)? Only after the think window, the answer: about 0.092,
//! where the curve leaves the band, measured, not derived; 1/15 is smaller
//! and still enough, since a window only has to work for every input (05_7).
//! Then the proof read forward, line by line, in the definition's order, for
//! every x in the domain and in the window (05_8), and the two directions
//! named: the search ran backward, the proof runs forward and hides it
//! (05_9).
//!
//! Every number drawn comes from the facts registry; the window found is the
//! model's `found_window`, the widest its certified `widest_window_rounded`.

use explainer_kit::layout::TEXT_MARGIN;
use explainer_kit::{Facts, Kit, Palette, rect, v};
use fmn::prelude::*;

use super::challenge_game::end_picture;
use super::draw::{
    NUMBER_DROP, PLOT, fact, fade_in_all, fade_in_lagged, fade_out_all, fail, fixed, flat, limit_color, line_style,
    moving, say, think_pause,
};
use super::old_objection::{RIGHT_X, swap, tex_frac};
use super::shrinking_region::glow;
use crate::facts::facts;
use crate::model::{BASE, SEARCH_EPS, found_window};

pub struct TheHiddenSearch {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const WITHIN: &str = r"\text{within } \varepsilon \text{ of } ";
const GOAL: &str = r"\text{goal: } ";
const WHEN: &str = r"\text{ when } ";
const NEAR: &str = r"\text{near } x = ";
const ONLY_ABOUT: &str = r"\text{ it is only about } ";
const WIDEST: &str = r"\text{widest window: } \delta \approx ";
const LET: &str = r"\text{Let } \varepsilon > ";
const AND: &str = r"\text{ and } ";
const IF: &str = r"\text{If } x \in ";
const NEXT: &str = "next: making that search systematic";

// what the pictures wait for, each in the line the test below holds it to
const WHERE_AT: &str = "never says where it came from";
const SEARCH_AT: &str = "Here is the search";
const DEMANDS_AT: &str = "what the challenger demands";
const NEAR_AT: &str = "for every x near one half";
const DOMAIN_AT: &str = "On the domain";
const GENEROUS_AT: &str = "deliberate, generous step";
const ENOUGH_AT: &str = "So a distance below";
const CAME_AT: &str = "That is where delta came from";
const TENTH_AT: &str = "Against one tenth";
const HOW_AT: &str = "about how large could delta be";
const ANSWER_AT: &str = "About point zero nine two";
const LEAVES_AT: &str = "where the curve leaves the band";
const SMALLER_AT: &str = "One fifteenth is smaller";
const GIVEN_AT: &str = "Given epsilon";
const EVERY_AT: &str = "For every x in the domain and in the window";
const BELOW_AT: &str = "the gap is below";
const BACKWARD_AT: &str = "The search ran backward";
const FORWARD_AT: &str = "The proof runs forward";
const NEXT_AT: &str = "Next, ways";

// --- layout (scene units; the frame is 14.2 by 8; the plot is draw::PLOT)

/// The finished proof's three lines, top right, under the top edge.
const PROOF_Y: [f64; 3] = [3.35, 2.92, 2.45];
/// The search's rows, from the goal down to δ (the fourth is a dim note).
const SEARCH_Y: [f64; 6] = [1.6, 1.0, 0.4, 0.0, -0.55, -1.16];
/// The comparison at 1/10 under the search.
const COMPARE_Y: [f64; 2] = [-1.75, -2.2];
/// "next:", centred under everything.
const NEXT_Y: f64 = -2.75;
/// The band's label, inside the plot's empty left, over the band (data).
const BAND_LABEL: (f64, f64) = (0.02, 0.4);
/// The window's label, over the window near the plot's top (data).
const WINDOW_LABEL_Y: f64 = 0.95;
/// The think ring, in the plot's empty top left, far enough left that the
/// "pause for more time" under it ends short of the window's left edge (at
/// x = -4.4 its last letters crossed it in draft12, Mail 4165).
const RING_AT: Vec3 = [-4.65, 1.6, 0.0];
const RING_R: f64 = 0.45;
/// Where the curve leaves the band on the right is marked by a hollow ring.
const EDGE_R: f64 = 0.07;

// the proof stays title-safe and apart; the search rows are apart and under
// the proof; the comparison is under the search and over a lower-third's
// top (-2.42, though this chapter shows none); "next:" stays inside the
// frame; the ring is inside the frame
const _: () = assert!(PROOF_Y[0] + 0.2 < 4.0 - TEXT_MARGIN && PROOF_Y[0] - PROOF_Y[1] > 0.4);
const _: () = assert!(PROOF_Y[1] - PROOF_Y[2] > 0.4 && PROOF_Y[2] - 0.25 > SEARCH_Y[0] + 0.25);
const _: () = assert!(SEARCH_Y[0] - SEARCH_Y[1] >= 0.6 && SEARCH_Y[1] - SEARCH_Y[2] >= 0.6);
const _: () = assert!(SEARCH_Y[2] - SEARCH_Y[3] >= 0.4 && SEARCH_Y[3] - SEARCH_Y[4] >= 0.55);
const _: () = assert!(SEARCH_Y[4] - SEARCH_Y[5] >= 0.6 && SEARCH_Y[5] - 0.25 > COMPARE_Y[0] + 0.2);
const _: () = assert!(COMPARE_Y[0] - COMPARE_Y[1] >= 0.45 && COMPARE_Y[1] - 0.2 > -2.42);
const _: () = assert!(NEXT_Y - 0.2 > -4.0 + TEXT_MARGIN && NEXT_Y + 0.2 < COMPARE_Y[1] - 0.2);
const _: () = assert!(RING_AT[0] - RING_R > -7.1 + TEXT_MARGIN);

/// The window |x − a| < d on the plot, its full height, detached.
fn plot_window(stage: &mut Stage<'_>, a: f64, d: f64, style: Style) -> fmn::Result<Mob> {
    let (lo, hi) = (PLOT.p(a - d, 0.0), PLOT.p(a + d, 1.0));
    let r = rect(lo, hi[0] - lo[0], hi[1] - lo[1], style)?;
    Ok(stage.arena_mut().add(r))
}

/// The band |y − c| < t across the plot, detached.
fn plot_band(stage: &mut Stage<'_>, c: f64, t: f64) -> fmn::Result<Mob> {
    let (lo, hi) = (PLOT.p(0.0, c - t), PLOT.p(1.0, c + t));
    let r = rect(lo, hi[0] - lo[0], hi[1] - lo[1], flat(limit_color(), 0.22))?;
    Ok(stage.arena_mut().add(r))
}

/// A TeX row of the right column, left-aligned, detached.
fn row(kit: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, y: f64) -> fmn::Result<Mob> {
    kit.tex_left(stage, src, size, color, &[], v(RIGHT_X, y))
}

/// The finished proof's three lines in `color`, detached.
fn proof_lines(kit: &Kit, stage: &mut Stage<'_>, f: &Facts, color: Srgb) -> fmn::Result<[Mob; 3]> {
    let half = tex_frac(&fact(f, "C5.a")?);
    let quarter = tex_frac(&fact(f, "C5.value")?);
    let bound = tex_frac(&fact(f, "C5.bound")?);
    let ratio = tex_frac(&fact(f, "C5.ratio")?);
    let zero = fact(f, "C3.h.to")?;
    let n = fact(f, "C3.power")?;
    let domain = format!("[{}, {}]", fact(f, "C1.domain.left")?, fact(f, "C1.domain.right")?);
    Ok([
        row(
            kit,
            stage,
            &format!(r"{LET}{zero}{AND}\delta = {ratio}\varepsilon."),
            22.0,
            color,
            PROOF_Y[0],
        )?,
        row(
            kit,
            stage,
            &format!(r"{IF}{domain}{AND}|x - {half}| < \delta,"),
            22.0,
            color,
            PROOF_Y[1],
        )?,
        row(
            kit,
            stage,
            &format!(
                r"|x^{{{n}}} - {quarter}| = |x - {half}|\,|x + {half}| \le {bound}|x - {half}| < {bound}\delta = \varepsilon."
            ),
            22.0,
            color,
            PROOF_Y[2],
        )?,
    ])
}

/// Where on a proof line to put a glow: `share` of the way along it.
fn along(stage: &Stage<'_>, line: Mob, y: f64, share: f64) -> Vec3 {
    v(RIGHT_X + stage.get_width(line) * share, y)
}

impl SceneConstruct for TheHiddenSearch {
    fn name(&self) -> &str {
        "05_why_written_proofs_hide_the_search"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts().map_err(fail)?;
        let end = end_picture(&kit, stage, &f)?;
        let half = tex_frac(&fact(&f, "C5.a")?);
        let quarter = tex_frac(&fact(&f, "C5.value")?);
        let bound = tex_frac(&fact(&f, "C5.bound")?);
        let ratio = tex_frac(&fact(&f, "C5.ratio")?);
        let n = fact(&f, "C3.power")?;
        let domain = format!("[{}, {}]", fact(&f, "C1.domain.left")?, fact(&f, "C1.domain.right")?);
        let a = BASE.f64();
        let level = a * a;
        let eps = SEARCH_EPS.f64();
        let delta = found_window(BASE, SEARCH_EPS).map_err(fail)?.f64();

        // 05_1: chapter 04's picture goes; a finished proof, its δ unexplained
        let line = say(&kit, stage, "05_1")?;
        fade_out_all(stage, &end.clear, 0.6)?;
        let proof = proof_lines(&kit, stage, &f, Palette::ink())?;
        fade_in_lagged(stage, &proof, 1.0, 0.3)?;
        line.wait_for(stage, WHERE_AT)?;
        let at = along(stage, proof[0], PROOF_Y[0], 0.8);
        glow(&kit, stage, at, 0.3)?;
        line.wait_for(stage, SEARCH_AT)?;
        let proof_dim = proof_lines(&kit, stage, &f, Palette::dim())?;
        let anims = swap(stage, &proof, &proof_dim, 0.6)?;
        stage.play_prepared(anims)?;

        // 05_2: the goal; the band around 1/4 and the point 1/2 on the plot;
        // a window of unknown width
        let line = say(&kit, stage, "05_2")?;
        line.wait_for(stage, DEMANDS_AT)?;
        let goal = row(
            &kit,
            stage,
            &format!(r"{GOAL}|x^{{{n}}} - {quarter}| < \varepsilon"),
            26.0,
            Palette::ink(),
            SEARCH_Y[0],
        )?;
        let band = plot_band(stage, level, eps)?;
        let band_label = kit.tex_left(
            stage,
            &format!("{WITHIN}{quarter}"),
            22.0,
            limit_color(),
            &[],
            PLOT.p(BAND_LABEL.0, BAND_LABEL.1),
        )?;
        let guide = PLOT.dashed(stage, (a, 0.0), (a, level), fixed())?;
        let foot = PLOT.p(a, 0.0);
        let half_mark = kit.tex(stage, &half, 26.0, fixed(), &[], v(foot[0], foot[1] - NUMBER_DROP))?;
        let point = PLOT.dot(stage, a, level, fixed());
        fade_in_lagged(stage, &[goal, band, band_label, guide, half_mark, point], 1.0, 0.2)?;
        line.wait_for(stage, NEAR_AT)?;
        let window = plot_window(stage, a, delta, flat(moving(), 0.22))?;
        let unknown = kit.tex(stage, r"\delta = ?", 26.0, moving(), &[], PLOT.p(a, WINDOW_LABEL_Y))?;
        fade_in_all(stage, &[window, unknown], 0.5)?;
        stage.scene_mut().bring_to_front(&[point])?;

        // 05_3: the gap factors
        say(&kit, stage, "05_3")?;
        let factored = row(
            &kit,
            stage,
            &format!(r"|x^{{{n}}} - {quarter}| = |x - {half}| \cdot |x + {half}|"),
            26.0,
            Palette::ink(),
            SEARCH_Y[1],
        )?;
        fade_in_all(stage, &[factored], 0.6)?;

        // 05_4: on the domain the second factor is at most 3/2, largest at
        // x = 1; near 1/2 it is only about 1: generous on purpose
        let line = say(&kit, stage, "05_4")?;
        line.wait_for(stage, DOMAIN_AT)?;
        let bounded = row(
            &kit,
            stage,
            &format!(r"x \in {domain} \Rightarrow |x + {half}| \le {bound}"),
            26.0,
            Palette::ink(),
            SEARCH_Y[2],
        )?;
        fade_in_all(stage, &[bounded], 0.5)?;
        glow(&kit, stage, PLOT.p(1.0, 0.0), 0.2)?;
        line.wait_for(stage, GENEROUS_AT)?;
        let generous = row(
            &kit,
            stage,
            &format!(r"{NEAR}{half}{ONLY_ABOUT}{}", fact(&f, "C5.near")?),
            22.0,
            Palette::dim(),
            SEARCH_Y[3],
        )?;
        fade_in_all(stage, &[generous], 0.5)?;

        // 05_5: a distance below (2/3)ε is enough; that is δ
        let line = say(&kit, stage, "05_5")?;
        line.wait_for(stage, ENOUGH_AT)?;
        let enough = row(
            &kit,
            stage,
            &format!(r"{bound}|x - {half}| < \varepsilon{WHEN}|x - {half}| < {ratio}\varepsilon"),
            26.0,
            Palette::ink(),
            SEARCH_Y[4],
        )?;
        fade_in_all(stage, &[enough], 0.5)?;
        line.wait_for(stage, CAME_AT)?;
        let found = row(
            &kit,
            stage,
            &format!(r"\delta = {ratio}\varepsilon"),
            30.0,
            moving(),
            SEARCH_Y[5],
        )?;
        let named = kit.tex(
            stage,
            &format!(r"\delta = {ratio}\varepsilon"),
            26.0,
            moving(),
            &[],
            PLOT.p(a, WINDOW_LABEL_Y),
        )?;
        let mut anims = swap(stage, &[unknown], &[named], 0.6)?;
        anims.extend(swap(stage, &[], &[found], 0.6)?);
        stage.play_prepared(anims)?;
        let at = along(stage, proof_dim[0], PROOF_Y[0], 0.8);
        glow(&kit, stage, at, 0.3)?;

        // 05_6: the prediction at 1/10, beside the window found there, 1/15;
        // no widest outline or number until the ring has drained
        let line = say(&kit, stage, "05_6")?;
        line.wait_for(stage, TENTH_AT)?;
        let tenth = tex_frac(&fact(&f, "C5.eps")?);
        let numbered = kit.tex_left(
            stage,
            &format!(r"\varepsilon = {tenth}"),
            22.0,
            limit_color(),
            &[],
            PLOT.p(BAND_LABEL.0, BAND_LABEL.1),
        )?;
        let ours = row(
            &kit,
            stage,
            &format!(
                r"\varepsilon = {tenth}:\ \delta = {} \approx {}",
                tex_frac(&fact(&f, "C5.delta")?),
                fact(&f, "C5.delta.3dp")?
            ),
            26.0,
            moving(),
            COMPARE_Y[0],
        )?;
        let mut anims = swap(stage, &[band_label], &[numbered], 0.6)?;
        anims.extend(swap(stage, &[], &[ours], 0.6)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, HOW_AT)?;
        let ask = kit.text(stage, "?", 60.0, Palette::hero(), RING_AT)?;
        fade_in_all(stage, &[ask], 0.3)?;
        think_pause(&kit, stage, RING_AT, RING_R)?;

        // 05_7: the answer, said and drawn together: the widest, measured,
        // about 0.092, where the curve leaves the band on the right; the
        // window found, 1/15, is smaller and still enough
        let line = say(&kit, stage, "05_7")?;
        fade_out_all(stage, &[ask], 0.3)?;
        line.wait_for(stage, ANSWER_AT)?;
        let widest_at: f64 = fact(&f, "C5.widest.3dp")?
            .parse()
            .map_err(|_| fail("C5.widest.3dp is not a decimal".to_owned()))?;
        let widest = plot_window(stage, a, widest_at, line_style(moving(), 2.5))?;
        let widest_row = row(
            &kit,
            stage,
            &format!("{WIDEST}{}", fact(&f, "C5.widest.3dp")?),
            24.0,
            Palette::dim(),
            COMPARE_Y[1],
        )?;
        fade_in_lagged(stage, &[widest, widest_row], 0.8, 0.3)?;
        line.wait_for(stage, LEAVES_AT)?;
        let edge_at = PLOT.p((level + eps).sqrt(), level + eps);
        let edge = Circle::new()
            .radius(EDGE_R)
            .arc_center(edge_at)
            .style(line_style(Palette::hero(), 3.0))
            .build();
        let edge = stage.arena_mut().add(edge);
        fade_in_all(stage, &[edge], 0.3)?;
        glow(&kit, stage, edge_at, 0.2)?;
        line.wait_for(stage, SMALLER_AT)?;
        let at = along(stage, ours, COMPARE_Y[0], 0.5);
        glow(&kit, stage, at, 0.25)?;

        // 05_8: the proof read forward, a line at a time
        let line = say(&kit, stage, "05_8")?;
        let lit = proof_lines(&kit, stage, &f, Palette::ink())?;
        for (i, at) in [GIVEN_AT, EVERY_AT, BELOW_AT].into_iter().enumerate() {
            line.wait_for(stage, at)?;
            let anims = swap(stage, &[proof_dim[i]], &[lit[i]], 0.5)?;
            stage.play_prepared(anims)?;
            let at = along(stage, lit[i], PROOF_Y[i], 0.5);
            glow(&kit, stage, at, 0.25)?;
        }

        // 05_9: the search ran from the goal down to δ; the proof runs from δ
        // to the goal
        let line = say(&kit, stage, "05_9")?;
        line.wait_for(stage, BACKWARD_AT)?;
        for (m, y, share) in [(goal, SEARCH_Y[0], 0.4), (found, SEARCH_Y[5], 0.5)] {
            let at = along(stage, m, y, share);
            glow(&kit, stage, at, 0.25)?;
        }
        line.wait_for(stage, FORWARD_AT)?;
        for (m, y, share) in [(lit[0], PROOF_Y[0], 0.8), (lit[2], PROOF_Y[2], 0.9)] {
            let at = along(stage, m, y, share);
            glow(&kit, stage, at, 0.25)?;
        }
        line.wait_for(stage, NEXT_AT)?;
        let next = kit.text(stage, NEXT, 22.0, Palette::dim(), v(0.0, NEXT_Y))?;
        fade_in_all(stage, &[next], 0.6)?;
        kit.hold(stage)?;
        kit.music_breath(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every phrase a picture waits for is said once, in the line that
    /// waits for it, in the order the pictures take them.
    #[test]
    fn the_pictures_wait_on_words_the_script_says() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let text = |id: &str| script.text(id).unwrap();
        let waits = [
            ("05_1", WHERE_AT),
            ("05_1", SEARCH_AT),
            ("05_2", DEMANDS_AT),
            ("05_2", NEAR_AT),
            ("05_4", DOMAIN_AT),
            ("05_4", GENEROUS_AT),
            ("05_5", ENOUGH_AT),
            ("05_5", CAME_AT),
            ("05_6", TENTH_AT),
            ("05_6", HOW_AT),
            ("05_7", ANSWER_AT),
            ("05_7", LEAVES_AT),
            ("05_7", SMALLER_AT),
            ("05_8", GIVEN_AT),
            ("05_8", EVERY_AT),
            ("05_8", BELOW_AT),
            ("05_9", BACKWARD_AT),
            ("05_9", FORWARD_AT),
            ("05_9", NEXT_AT),
        ];
        for (id, phrase) in waits {
            assert_eq!(text(id).matches(phrase).count(), 1, "{id} says {phrase:?} once");
        }
        for pair in waits.windows(2).filter(|w| w[0].0 == w[1].0) {
            let line = text(pair[0].0);
            assert!(line.find(pair[0].1) < line.find(pair[1].1), "{pair:?}");
        }
    }

    /// The search comes before the forward proof, the question before its
    /// answer. The question asks how large δ could be at one tenth; no line
    /// before the answer says the widest's number; the answer opens with it,
    /// then says the window found is smaller and only has to work for every
    /// input. The forward proof says both conditions on x.
    #[test]
    fn the_search_runs_backward_and_the_proof_forward() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let text = |id: &str| script.text(id).unwrap();
        assert!(text("05_2").starts_with("Start from what the challenger demands"));
        assert!(text("05_6").ends_with('?') && text("05_6").contains(HOW_AT));
        for id in ["05_1", "05_2", "05_3", "05_4", "05_5", "05_6"] {
            assert!(!text(id).contains("point zero nine"), "{id} says the answer early");
        }
        assert!(text("05_7").starts_with(ANSWER_AT));
        assert!(text("05_7").contains("is smaller, and still enough"));
        assert!(text("05_7").contains("work for every input, not be the widest"));
        assert!(text("05_8").starts_with("Now read the proof forward"));
        assert!(text("05_8").contains(EVERY_AT));
    }

    /// The windows drawn are the model's: the found one is narrower than the
    /// widest, both inside the plot; the edge where the curve leaves the band
    /// is inside the widest window's right end.
    #[test]
    fn the_windows_fit_the_plot() {
        let a = BASE.f64();
        let delta = found_window(BASE, SEARCH_EPS).unwrap().f64();
        let f = facts().unwrap();
        let widest: f64 = f.get("C5.widest.3dp").unwrap().parse().unwrap();
        assert!(delta < widest && a - widest > 0.0 && a + widest < 1.0);
        let edge = (a * a + SEARCH_EPS.f64()).sqrt();
        assert!((edge - (a + widest)).abs() < 0.0005);
        // the band's label sits over the band, clear of the curve at its x
        let top = a * a + SEARCH_EPS.f64();
        assert!(BAND_LABEL.1 > top && BAND_LABEL.0 * BAND_LABEL.0 < BAND_LABEL.1 - 0.2);
    }
}
