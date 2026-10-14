//! Scene-time marks for the chapter tests: a scene says "this is whole on
//! screen now" or "this starts to leave now", and a test reads the moment
//! back from a headless run, so a picture's time on screen is measured and
//! not read off the constants that were meant to produce it (the same device
//! as `draw::quiz_log`, for pictures that are not quiz cards).
//!
//! Outside tests [`mark`] does nothing.

use explainer_kit::Kit;
use fmn::prelude::*;

/// Note that `what` is so now, in the run this kit names. `what` reads
/// `"<line>.<thing>"`: `"05_4.why_in"`.
pub fn mark(kit: &Kit, stage: &Stage<'_>, what: &'static str) {
    #[cfg(test)]
    {
        if let Ok(mut all) = log::MARKS.lock() {
            all.push((kit.chapter.clone(), what, stage.scene().time().to_f64()));
        }
    }
    #[cfg(not(test))]
    {
        let _ = (kit, stage, what);
    }
}

/// Note a measured `value` (scene units) of what the scene has just laid out,
/// under `what`, in the run this kit names; read back with [`at`].
pub fn note(kit: &Kit, what: &'static str, value: f64) {
    #[cfg(test)]
    {
        if let Ok(mut all) = log::MARKS.lock() {
            all.push((kit.chapter.clone(), what, value));
        }
    }
    #[cfg(not(test))]
    {
        let _ = (kit, what, value);
    }
}

/// How many of `mobs` have nothing on screen to fade (fill and stroke both at
/// 0 where either is known): a fade-out of one is a flash, up and down.
pub fn count_unseen(stage: &Stage<'_>, mobs: &[Mob]) -> usize {
    mobs.iter()
        .filter(|&&m| {
            let (fill, stroke) = (stage.get_fill_opacity(m), stage.get_stroke_opacity(m));
            (fill.is_some() || stroke.is_some()) && fill.unwrap_or(0.0) <= 0.01 && stroke.unwrap_or(0.0) <= 0.01
        })
        .count()
}

/// When `what` was marked (or what value it was noted at) in the run named
/// `chapter`, the first time.
#[cfg(test)]
pub fn at(chapter: &str, what: &str) -> f64 {
    all(chapter)
        .into_iter()
        .find(|(w, _)| *w == what)
        .map(|(_, t)| t)
        .unwrap_or_else(|| panic!("{chapter}: nothing marked {what:?} ({:?})", all(chapter)))
}

/// Every mark of the run named `chapter`, in order.
#[cfg(test)]
pub fn all(chapter: &str) -> Vec<(&'static str, f64)> {
    log::MARKS
        .lock()
        .map(|m| {
            m.iter()
                .filter(|(c, ..)| c == chapter)
                .map(|(_, w, t)| (*w, *t))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod log {
    use std::sync::Mutex;

    pub static MARKS: Mutex<Vec<(String, &'static str, f64)>> = Mutex::new(Vec::new());
}

/// The frame-by-frame watch of a chapter's run: what is on the stage at every
/// frame, so the invariants of the pictures (nothing is resized while it
/// fades, a lower-third stays up long enough and clear of the picture, the
/// stage is never empty between two beats) are measured on the run and not
/// read off the constants that were meant to produce them. Tests only.
#[cfg(test)]
pub mod probe {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::ops::RangeInclusive;
    use std::rc::Rc;

    use fmn::prelude::*;

    use super::super::draw::fade_log::{self, Short};
    use super::super::draw::testing::{self, Probe, Room, Voiced};

    /// A lower-third's card is as wide as the band the kit draws
    /// (`2 * FRAME_X_RADIUS - 0.8`) and stands on the foot of the frame.
    const BAND_WIDTH: RangeInclusive<f64> = 13.2..=13.6;
    const BAND_FOOT: RangeInclusive<f64> = -3.85..=-3.7;
    /// A leaf with less than this on screen is not counted as there.
    const SEEN: f64 = 0.05;
    /// A picture whose foot is nearer than this to a lower-third's top, or
    /// under it, is crowding it.
    pub const BAND_CLEARANCE: f64 = 0.25;
    /// The least a lower-third stays fully up, seconds.
    pub const BAND_DWELL: f64 = 2.5;
    /// The longest the stage may stand empty between two beats, seconds: a
    /// picture that fades out and the next one that fades in leave it bare
    /// for 0.3 or 0.4 s, which is a change of beat; half a second is a hole.
    pub const SEAM: f64 = 0.5;
    /// Two lower-thirds each this much on screen at one frame print their
    /// words over each other (a cross-fade does, for a quarter of a second).
    pub const OVERPRINT: f64 = 0.3;
    /// The scenes are run at this many frames a second.
    const FPS: f64 = 10.0;

    /// A lower-third's card at one frame.
    #[derive(Debug, Clone, Copy)]
    pub struct BandNow {
        pub id: Mob,
        /// The lower-third (the group) the card belongs to.
        pub root: Mob,
        pub at: f64,
        pub opacity: f64,
        pub room: Room,
        /// The foot of the lowest picture not in a lower-third, less the top
        /// of this card (negative when it stands over it), and that picture.
        pub gap: Option<(f64, Room)>,
    }

    /// What the probe saw, frame by frame.
    #[derive(Default)]
    pub struct Watch {
        /// Every frame's scene time and how many leaves were on screen.
        pub frames: Vec<(f64, usize)>,
        pub bands: Vec<BandNow>,
        /// The leaves a fade-in of the run ended under half their opacity
        /// (`draw::fade_log`).
        pub short: Vec<Short>,
        clock: f64,
    }

    impl Watch {
        /// Runs of frames, after the first thing is on the stage and before
        /// the last, with nothing on it: (start, seconds).
        pub fn seams(&self) -> Vec<(f64, f64)> {
            let first = self.frames.iter().position(|f| f.1 > 0);
            let last = self.frames.iter().rposition(|f| f.1 > 0);
            let (Some(first), Some(last)) = (first, last) else {
                return Vec::new();
            };
            let mut out = Vec::new();
            let mut run: Option<(f64, usize)> = None;
            for &(at, n) in &self.frames[first..=last] {
                match (n == 0, &mut run) {
                    (true, Some((_, len))) => *len += 1,
                    (true, None) => run = Some((at, 1)),
                    (false, _) => {
                        if let Some((start, len)) = run.take() {
                            out.push((start, len as f64 / FPS));
                        }
                    }
                }
            }
            out
        }

        /// Each lower-third card of the run: its longest stretch fully up
        /// (opacity at least 0.9 of the most it ever had), as (when it began,
        /// seconds, the room it fills).
        pub fn band_dwells(&self) -> Vec<(f64, f64, Room)> {
            let mut by_card: HashMap<Mob, Vec<&BandNow>> = HashMap::new();
            for b in &self.bands {
                by_card.entry(b.id).or_default().push(b);
            }
            let mut out = Vec::new();
            for seen in by_card.values() {
                let most = seen.iter().map(|b| b.opacity).fold(0.0, f64::max);
                let mut best: Option<(f64, f64)> = None;
                let mut run: Option<(f64, f64)> = None;
                for b in seen {
                    if b.opacity >= 0.9 * most {
                        let began = run.map_or(b.at, |r| r.0);
                        run = Some((began, b.at + 1.0 / FPS - began));
                        if best.is_none_or(|x| run.is_some_and(|r| r.1 > x.1)) {
                            best = run;
                        }
                    } else {
                        run = None;
                    }
                }
                if let Some((began, seconds)) = best {
                    out.push((began, seconds, seen[0].room));
                }
            }
            out.sort_by(|a, b| a.0.total_cmp(&b.0));
            out
        }

        /// The frames at which a lower-third is fully up and a picture stands
        /// nearer to it than [`BAND_CLEARANCE`] (or over it).
        pub fn crowded(&self) -> Vec<(f64, f64, Room)> {
            let mut most: HashMap<Mob, f64> = HashMap::new();
            for b in &self.bands {
                let m = most.entry(b.id).or_insert(0.0);
                *m = m.max(b.opacity);
            }
            self.bands
                .iter()
                .filter(|b| b.opacity >= 0.9 * most[&b.id])
                .filter_map(|b| b.gap.map(|(gap, room)| (b.at, gap, room)))
                .filter(|(_, gap, _)| *gap < BAND_CLEARANCE)
                .collect()
        }

        /// The frames at which two or more lower-thirds are each at least
        /// [`OVERPRINT`] on screen: (when, how many).
        pub fn overprinted(&self) -> Vec<(f64, usize)> {
            let mut frames: Vec<(f64, Vec<Mob>)> = Vec::new();
            for b in self.bands.iter().filter(|b| b.opacity >= OVERPRINT) {
                match frames.last_mut() {
                    Some((at, roots)) if *at == b.at => {
                        if !roots.contains(&b.root) {
                            roots.push(b.root);
                        }
                    }
                    _ => frames.push((b.at, vec![b.root])),
                }
            }
            frames
                .into_iter()
                .filter(|(_, roots)| roots.len() > 1)
                .map(|(at, roots)| (at, roots.len()))
                .collect()
        }
    }

    /// The box of a run of points.
    fn room_of(points: &[Vec3]) -> Room {
        let mut r = Room::spanning(points[0], points[0]);
        for p in points {
            r.x0 = r.x0.min(p[0]);
            r.x1 = r.x1.max(p[0]);
            r.y0 = r.y0.min(p[1]);
            r.y1 = r.y1.max(p[1]);
        }
        r
    }

    /// One frame of the stage, noted in `log`.
    fn sample(s: &mut fmn::mobject::Stage, dt: f64, log: &Rc<RefCell<Watch>>) {
        let at = {
            // an updater is not given the scene's clock: the frames add up to it
            let mut w = log.borrow_mut();
            w.clock += dt;
            w.clock
        };
        let roots: Vec<Mob> = s.roots().to_vec();
        // (root, leaf, opacity, box)
        let mut leaves: Vec<(usize, Mob, f64, Room)> = Vec::new();
        for (r, &root) in roots.iter().enumerate() {
            for m in s.family(root) {
                let Some(points) = s.get_points(m) else { continue };
                if points.is_empty() {
                    continue;
                }
                let opacity = s
                    .get_fill_opacity(m)
                    .unwrap_or(0.0)
                    .max(s.get_stroke_opacity(m).unwrap_or(0.0));
                leaves.push((r, m, opacity, room_of(&points)));
            }
        }
        let mut w = log.borrow_mut();
        w.frames.push((at, leaves.iter().filter(|l| l.2 > SEEN).count()));
        // the roots that are lower-thirds, and the lowest picture outside them
        let is_card = |room: &Room| BAND_WIDTH.contains(&(room.x1 - room.x0)) && BAND_FOOT.contains(&room.y0);
        let band_roots: Vec<usize> = leaves.iter().filter(|l| is_card(&l.3)).map(|l| l.0).collect();
        let lowest = leaves
            .iter()
            .filter(|l| l.2 > 0.1 && !band_roots.contains(&l.0))
            .map(|l| l.3)
            .min_by(|a, b| a.y0.total_cmp(&b.y0));
        for &(r, m, opacity, room) in leaves.iter().filter(|l| is_card(&l.3)) {
            w.bands.push(BandNow {
                id: m,
                root: roots[r],
                at,
                opacity,
                room,
                gap: lowest.map(|low| (low.y0 - room.y1, low)),
            });
        }
    }

    /// Put the probe on `stage`: it reads the stage at every frame from now
    /// on, into `log`.
    pub fn install(stage: &mut Stage<'_>, log: &Rc<RefCell<Watch>>) -> fmn::Result<()> {
        let log = Rc::clone(log);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_dt_updater(anchor, move |s, _, dt| sample(s, dt, &log), true)?;
        Ok(())
    }

    /// What `log` holds once the run is over, with the fades of the run that
    /// ended short.
    fn finish(log: Rc<RefCell<Watch>>) -> Watch {
        let mut watch = Rc::try_unwrap(log)
            .map(RefCell::into_inner)
            .unwrap_or_else(|rc| std::mem::take(&mut *rc.borrow_mut()));
        watch.short = fade_log::take();
        watch
    }

    /// Run chapter `key` with the scratch voice placed and the probe on its
    /// stage.
    pub fn watch(key: &str) -> Result<(Voiced, Watch), String> {
        let seen = Rc::new(RefCell::new(Watch::default()));
        let log = Rc::clone(&seen);
        let probe: Probe = Box::new(move |stage| install(stage, &log));
        fade_log::clear();
        let voiced = testing::run_probed(key, probe)?;
        Ok((voiced, finish(seen)))
    }

    /// Run `scene`, a scene of a test's own, with the probe on its stage.
    pub fn watch_scene(scene: impl FnOnce(&mut Stage<'_>) -> fmn::Result<()>) -> Watch {
        let seen = Rc::new(RefCell::new(Watch::default()));
        let log = Rc::clone(&seen);
        let mut scene = Some(scene);
        fade_log::clear();
        super::super::carts::probe::run_scene(|stage| {
            install(stage, &log)?;
            scene.take().map_or(Ok(()), |f| f(stage))
        });
        drop(log);
        finish(seen)
    }
}

#[cfg(test)]
mod tests {
    use explainer_kit::props::lower_third;
    use explainer_kit::{Kit, Palette, v};
    use fmn::prelude::*;

    use super::super::draw::testing::{Room, Voiced};
    use super::super::draw::{corners, cross_fade, fade_in_all, fade_out_all, faded_line, pause};
    use super::probe::{self, BAND_CLEARANCE, BAND_DWELL, SEAM, Watch};

    fn room(r: &Room) -> String {
        format!("x {:.2}..{:.2}, y {:.2}..{:.2}", r.x0, r.x1, r.y0, r.y1)
    }

    /// The line being said at scene time `at`, for a message.
    fn line_at(voiced: &Voiced, at: f64) -> &str {
        voiced
            .lines
            .iter()
            .filter(|(_, start, _)| *start <= at)
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map_or("before the first line", |l| l.0.as_str())
    }

    /// What a watch of `key` says is broken, one line each.
    fn broken(key: &str, voiced: &Voiced, watch: &Watch) -> Vec<String> {
        let mut found = Vec::new();
        for f in &watch.short {
            found.push(format!(
                "{key} I-1 a fade-in ended at {:.1} s ({}) with a leaf of {} points at opacity {:.3}, wanted {:.3}, at ({:.2}, {:.2})",
                f.at,
                line_at(voiced, f.at),
                f.points,
                f.reached,
                f.target,
                f.centre[0],
                f.centre[1]
            ));
        }
        for (began, seconds, r) in watch.band_dwells() {
            if seconds < BAND_DWELL {
                found.push(format!(
                    "{key} I-3 lower-third from {began:.1} s ({}) is up {seconds:.1} s, {}",
                    line_at(voiced, began),
                    room(&r)
                ));
            }
        }
        let mut last_crowd = f64::NEG_INFINITY;
        for (at, gap, low) in watch.crowded() {
            if at - last_crowd > 1.0 {
                found.push(format!(
                    "{key} I-3 lower-third at {at:.1} s ({}) has a picture {gap:.2} from its top (least {BAND_CLEARANCE}): {}",
                    line_at(voiced, at),
                    room(&low)
                ));
            }
            last_crowd = at;
        }
        let mut last_print = f64::NEG_INFINITY;
        for (at, n) in watch.overprinted() {
            if at - last_print > 0.15 {
                found.push(format!(
                    "{key} I-5 {n} lower-thirds print their words over each other at {at:.1} s ({})",
                    line_at(voiced, at)
                ));
            }
            last_print = at;
        }
        for (at, seconds) in watch.seams() {
            if seconds >= SEAM {
                found.push(format!(
                    "{key} I-4 the stage is empty for {seconds:.1} s from {at:.1} s ({})",
                    line_at(voiced, at)
                ));
            }
        }
        found
    }

    /// Invariants I-1, I-3, I-4 and I-5 of the pictures of a chapter, measured
    /// frame by frame on a run with the scratch voice:
    ///
    /// - I-1: no mobject is resized while it fades in and left where the
    ///   fade began (franken_manim ends such a fade at the opacity it began
    ///   at: 11_1's rays and 14_6's curve stayed invisible);
    /// - I-3: every lower-third stays fully up for at least 2.5 s and no
    ///   picture stands nearer than 0.25 to its top, or over it;
    /// - I-4: the stage is never empty for half a second between two beats;
    /// - I-5: no two lower-thirds print their words over each other (a
    ///   cross-fade of two bands does, for a quarter of a second).
    ///
    /// (I-2, the quiz cards' dwell, is `draw::tests::section_five`.) The real
    /// narration places phrases a little otherwise, so the scenes hold a
    /// band for `BAND_UP` = 2.6 s, a tenth over what is asked here.
    fn keeps_the_invariants(key: &str) {
        let (voiced, watch) = probe::watch(key).unwrap();
        assert!(voiced.ran.frames > 300, "{key}: only {} frames", voiced.ran.frames);
        assert!(
            watch.frames.len() as u64 + 2 >= voiced.ran.frames,
            "{key}: the probe saw {} of {} frames",
            watch.frames.len(),
            voiced.ran.frames
        );
        let found = broken(key, &voiced, &watch);
        assert!(
            found.is_empty(),
            "{} broken invariants:\n{}",
            found.len(),
            found.join("\n")
        );
    }

    #[test]
    fn the_bridge_that_swayed_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("01_the_bridge_that_swayed");
    }

    #[test]
    fn two_carts_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("02_two_carts");
    }

    #[test]
    fn own_patterns_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("03_own_patterns");
    }

    #[test]
    fn the_split_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("04_the_split");
    }

    #[test]
    fn a_steel_truss_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("05_a_steel_truss");
    }

    #[test]
    fn twenty_one_patterns_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("06_twenty_one_patterns");
    }

    #[test]
    fn every_mode_a_note_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("07_every_mode_a_note");
    }

    #[test]
    fn two_bridges_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("08_two_bridges");
    }

    #[test]
    fn the_air_in_a_room_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("09_the_air_in_a_room");
    }

    #[test]
    fn two_million_modes_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("10_two_million_modes");
    }

    #[test]
    fn where_modes_rule_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("11_where_modes_rule");
    }

    #[test]
    fn an_economy_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("12_an_economy");
    }

    #[test]
    fn can_it_feed_itself_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("13_can_it_feed_itself");
    }

    #[test]
    fn one_idea_keeps_the_invariants_of_its_pictures() {
        keeps_the_invariants("14_one_idea");
    }

    // ---- the probe against planted defects: each check fails on the old
    // behaviour and passes on the fix

    /// A curve of `n` vertices along a line.
    fn curve(n: usize) -> Vec<Vec3> {
        (0..n).map(|k| v(-3.0 + 6.0 * k as f64 / (n - 1) as f64, 0.0)).collect()
    }

    /// A gold curve like 14_6's, run as the chapter runs it: an updater on
    /// the scene rewrites it to 49 vertices, then it is faded in. Built with
    /// `built` vertices.
    fn curve_scene(built: usize) -> Watch {
        probe::watch_scene(|stage| {
            let m = stage.arena_mut().add(
                polyline(curve(built))
                    .style(faded_line(Palette::ink(), 4.0, 1.0))
                    .build(),
            );
            stage.add_dt_updater(
                m,
                move |s, _, _| {
                    let _ = s.set_points(m, &corners(&curve(49)));
                },
                false,
            )?;
            stage.add_to_scene(m)?;
            fade_in_all(stage, &[m], 0.8)?;
            pause(stage, 0.5)
        })
    }

    #[test]
    fn the_probe_finds_a_curve_resized_in_the_first_frame_of_its_fade_in() {
        let old = curve_scene(2);
        assert!(
            !old.short.is_empty(),
            "the old build (2 vertices) was not found: {:?}",
            old.short
        );
        assert!(
            old.short
                .iter()
                .all(|f| f.points > 40 && f.reached < 0.1 && f.target > 0.9),
            "{:?}",
            old.short
        );
        let fixed = curve_scene(49);
        assert!(fixed.short.is_empty(), "{:?}", fixed.short);
    }

    /// A lower-third faded in, held `up` seconds and faded out.
    fn band_scene(up: f64) -> Watch {
        probe::watch_scene(|stage| {
            let kit = Kit::new()?;
            let band = lower_third(&kit, stage, "names to search:", "aeroelastic flutter")?;
            fade_in_all(stage, &[band], 0.5)?;
            pause(stage, up)?;
            fade_out_all(stage, &[band], 0.4)?;
            pause(stage, 0.5)
        })
    }

    #[test]
    fn the_probe_measures_how_long_a_lower_third_stands() {
        let short = band_scene(1.2).band_dwells();
        assert_eq!(short.len(), 1, "{short:?}");
        assert!(short[0].1 < BAND_DWELL, "1.2 s held, {} s seen", short[0].1);
        let held = band_scene(2.6).band_dwells();
        assert!(held[0].1 >= BAND_DWELL, "2.6 s held, {} s seen", held[0].1);
        assert!((held[0].1 - 2.6).abs() < 0.35, "{} s", held[0].1);
    }

    /// A lower-third with a picture standing `gap` over its top.
    fn crowded_scene(gap: f64) -> Watch {
        probe::watch_scene(|stage| {
            let kit = Kit::new()?;
            let band = lower_third(&kit, stage, "names to search:", "aeroelastic flutter")?;
            let top = -3.8 + 0.66;
            let bar = stage.arena_mut().add(
                polyline(vec![v(-2.0, top + gap), v(2.0, top + gap), v(2.0, top + gap + 1.0)])
                    .style(faded_line(Palette::ink(), 4.0, 1.0))
                    .build(),
            );
            fade_in_all(stage, &[band, bar], 0.5)?;
            pause(stage, 1.0)
        })
    }

    #[test]
    fn the_probe_finds_a_picture_that_crowds_a_lower_third() {
        let near = crowded_scene(0.1).crowded();
        assert!(
            !near.is_empty() && near.iter().all(|c| c.1 < BAND_CLEARANCE),
            "{near:?}"
        );
        let clear = crowded_scene(0.5).crowded();
        assert!(clear.is_empty(), "{clear:?}");
    }

    /// One lower-third swapped for another, by a cross-fade or one after the
    /// other in the same half second.
    fn swap_scene(crossed: bool) -> Watch {
        probe::watch_scene(|stage| {
            let kit = Kit::new()?;
            let one = lower_third(&kit, stage, "precisely:", "below one, every demand can be met")?;
            let two = lower_third(&kit, stage, "precisely:", "at one or more, some demand fails")?;
            fade_in_all(stage, &[one], 0.5)?;
            pause(stage, 1.0)?;
            if crossed {
                cross_fade(stage, &[one], &[two], 0.5)?;
            } else {
                fade_out_all(stage, &[one], 0.2)?;
                fade_in_all(stage, &[two], 0.3)?;
            }
            pause(stage, 1.0)
        })
    }

    #[test]
    fn the_probe_finds_two_lower_thirds_printed_over_each_other() {
        let crossed = swap_scene(true).overprinted();
        assert!(
            !crossed.is_empty() && crossed.iter().all(|f| f.1 == 2),
            "a cross-fade of two bands was not found: {crossed:?}"
        );
        let one_after_the_other = swap_scene(false).overprinted();
        assert!(one_after_the_other.is_empty(), "{one_after_the_other:?}");
    }

    /// A picture that fades out and, `bare` seconds after it has gone,
    /// another that fades in.
    fn seam_scene(bare: f64) -> Watch {
        probe::watch_scene(|stage| {
            let a = stage
                .arena_mut()
                .add(polyline(curve(5)).style(faded_line(Palette::ink(), 4.0, 1.0)).build());
            let b = stage
                .arena_mut()
                .add(polyline(curve(7)).style(faded_line(Palette::ink(), 4.0, 1.0)).build());
            fade_in_all(stage, &[a], 0.5)?;
            pause(stage, 0.5)?;
            fade_out_all(stage, &[a], 0.5)?;
            pause(stage, bare)?;
            fade_in_all(stage, &[b], 0.5)?;
            pause(stage, 0.5)
        })
    }

    #[test]
    fn the_probe_finds_a_stage_left_bare_between_two_beats() {
        let hole = seam_scene(0.8).seams();
        assert!(hole.iter().any(|s| s.1 >= SEAM), "{hole:?}");
        let change = seam_scene(0.0).seams();
        assert!(change.iter().all(|s| s.1 < SEAM), "{change:?}");
    }
}
