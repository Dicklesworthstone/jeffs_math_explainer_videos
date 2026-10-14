//! Video B's recurring props, built once and called from every chapter that
//! shows them: the phone's bucket of useful cards (03, 05, 13, 15, 17) with
//! the danger meter on its rim, the risk ladder (06, 13, 17), the ring of
//! five cards (06, 16) and the cards-as-a-grid view (03, 15). Builders
//! return detached mobjects, like `Kit::text`: bring them in with an
//! animation. They follow the engine rules in `explainer_kit::kit`.

use std::f64::consts::{FRAC_PI_2, TAU};

use explainer_kit::{Kit, Palette, hex, tween, v};
use fmn::prelude::*;

use super::draw::{Card, Role, badge, card, flat, group_of, line_style};
use crate::model::{RING, pieces_of, result};

/// A slot's paper, a shade above the background.
fn slot_fill() -> Srgb {
    hex("#1A2130")
}

// ------------------------------------------------------------------ bucket

/// The phone's bucket (03_5): `k` slots stacked bottom-up in an open-topped
/// container. It fills one slot per useful card; the filled slots are the
/// rank.
pub struct BucketProp {
    pub group: Mob,
    pub outline: Mob,
    /// Slot i, 0 at the bottom: an empty flat rectangle.
    pub slots: Vec<Mob>,
    pub centre: Vec3,
    pub width: f64,
    pub height: f64,
    pub k: usize,
}

/// The walls' inset from the bucket's outer width and height.
const WALL: f64 = 0.14;

impl BucketProp {
    /// A bucket of `k` slots, `width` × `height`, centred at `centre`.
    pub fn build(stage: &mut Stage<'_>, k: usize, centre: Vec3, width: f64, height: f64) -> fmn::Result<Self> {
        assert!(k > 0, "a bucket has at least one slot");
        let (l, r) = (centre[0] - width / 2.0, centre[0] + width / 2.0);
        let (b, t) = (centre[1] - height / 2.0, centre[1] + height / 2.0);
        let ink = Role::card();
        let outline = v_group([
            Line::new(v(l, t + 0.12), v(l, b)).style(line_style(ink, 5.0)).build()?,
            Line::new(v(l, b), v(r, b)).style(line_style(ink, 5.0)).build()?,
            Line::new(v(r, b), v(r, t + 0.12)).style(line_style(ink, 5.0)).build()?,
        ]);
        let outline = stage.arena_mut().add(outline);
        let mut slots = Vec::with_capacity(k);
        let me = Self {
            group: outline,
            outline,
            slots: Vec::new(),
            centre,
            width,
            height,
            k,
        };
        for i in 0..k {
            let (c, w, h) = me.slot_rect(i);
            let s = Rectangle::new()
                .width(w)
                .height(h)
                .corner_radius((h * 0.12).min(0.06))
                .style(flat(slot_fill(), 0.95))
                .build()?
                .moved_to(c);
            slots.push(stage.arena_mut().add(s));
        }
        let mut all = vec![outline];
        all.extend(&slots);
        let group = group_of(stage, &all)?;
        Ok(Self { group, slots, ..me })
    }

    /// Slot i's centre, width and height (0 at the bottom).
    pub fn slot_rect(&self, i: usize) -> (Vec3, f64, f64) {
        let inner_h = self.height - 2.0 * WALL;
        let pitch = inner_h / self.k as f64;
        let gap = (pitch * 0.12).min(0.06);
        let y = self.centre[1] - self.height / 2.0 + WALL + pitch * (i as f64 + 0.5);
        (v(self.centre[0], y), self.width - 2.0 * WALL, pitch - gap)
    }

    /// Slot i's centre.
    pub fn slot_centre(&self, i: usize) -> Vec3 {
        self.slot_rect(i).0
    }

    /// A filled slot: a detached flat rectangle in `color` (`Role::ex_or`
    /// for an ex-or card, `Role::byte` for a byte card) over slot i.
    pub fn water(&self, stage: &mut Stage<'_>, i: usize, color: Srgb) -> fmn::Result<Mob> {
        let (c, w, h) = self.slot_rect(i);
        let m = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius((h * 0.12).min(0.06))
            .style(flat(color, 0.85))
            .build()?
            .moved_to(c);
        Ok(stage.arena_mut().add(m))
    }

    /// For a bucket too big to draw slot by slot (15_3's 10,269): one flat
    /// rectangle filling the fraction `from`..`to` of the inside height.
    pub fn level(&self, stage: &mut Stage<'_>, from: f64, to: f64, color: Srgb) -> fmn::Result<Mob> {
        let inner_h = self.height - 2.0 * WALL;
        let bottom = self.centre[1] - self.height / 2.0 + WALL;
        let h = (to - from).max(0.0) * inner_h;
        let m = Rectangle::new()
            .width(self.width - 2.0 * WALL)
            .height(h.max(1e-3))
            .style(flat(color, 0.85))
            .build()?
            .moved_to(v(self.centre[0], bottom + (from + to) / 2.0 * inner_h));
        Ok(stage.arena_mut().add(m))
    }

    /// The top of the rim.
    pub fn rim(&self) -> f64 {
        self.centre[1] + self.height / 2.0 + 0.12
    }
}

// ------------------------------------------------------------ danger meter

/// The danger meter (05_3): a track on the bucket's rim whose teal fill is
/// the chance that the next random card is useless, the "lit" share of all
/// the cards: what the bucket can already make. Bring in `group` with an
/// animation and `stage.add_to_scene(fill)` (a live bar, redrawn from
/// `chance`).
pub struct DangerMeter {
    pub group: Mob,
    /// The chance, 0..1: a value tracker; tween it with [`DangerMeter::to`].
    pub chance: Mob,
    pub track: Mob,
    pub fill: Mob,
    /// Where a readout's left edge goes.
    pub readout_at: Vec3,
}

impl DangerMeter {
    /// The meter on `bucket`'s rim, reading `chance`; its title above.
    pub fn on(kit: &Kit, stage: &mut Stage<'_>, bucket: &BucketProp, chance: f64) -> fmn::Result<Self> {
        let (w, h) = (bucket.width, 0.26);
        let y = bucket.rim() + 0.38;
        let left = bucket.centre[0] - w / 2.0;
        let track = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.06)
            .style(line_style(Palette::dim(), 2.5))
            .build()?
            .moved_to(v(bucket.centre[0], y));
        let track = stage.arena_mut().add(track);
        let tracker = stage.add_value_tracker(chance);
        let fill = stage.always_redraw(move |s| {
            let c = s.tracker_value(tracker).unwrap_or(0.0).clamp(0.0, 1.0);
            let bar = Rectangle::new()
                .width((c * w).max(0.004))
                .height(h * 0.7)
                .style(flat(Role::lit(), 0.9))
                .build()
                .expect("positive extents");
            let m = s.add(bar);
            s.move_to(m, v(left, y), LEFT);
            m
        });
        let title = kit.text(
            stage,
            "chance of a\nuseless card",
            44.0,
            Palette::ink(),
            v(bucket.centre[0], y + 0.72),
        )?;
        let group = group_of(stage, &[track, title])?;
        Ok(Self {
            group,
            chance: tracker,
            track,
            fill,
            readout_at: v(left + w + 0.25, y),
        })
    }

    /// Tween the meter to `chance` over `rt` seconds.
    pub fn to(&self, chance: f64, rt: f64) -> fmn::Result<AnimBuilder> {
        tween(self.chance, chance, rt)
    }

    /// A readout ("1/8"; the text from facts) beside the track, detached.
    pub fn readout(&self, kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<Mob> {
        kit.text_left(stage, text, 34.0, Role::lit(), self.readout_at)
    }
}

// -------------------------------------------------------------- risk ladder

/// The risk ladder (06_1): a vertical log scale from 1 at the top down to
/// 10^-decades, labelled "model"; each spare steps a marker down a rung.
pub struct RiskLadder {
    pub group: Mob,
    /// The scale's x, and the heights of chance 1 and of 10^-decades.
    pub x: f64,
    pub top: f64,
    pub bottom: f64,
    pub decades: u32,
}

impl RiskLadder {
    /// The scale, `height` tall, centred at `centre`, with a tick and label
    /// at every power of ten ("1", "1/10", …, "1/1,000,000").
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, centre: Vec3, height: f64, decades: u32) -> fmn::Result<Self> {
        let (top, bottom) = (centre[1] + height / 2.0, centre[1] - height / 2.0);
        let x = centre[0];
        let mut parts = vec![
            stage.arena_mut().add(
                Line::new(v(x, top), v(x, bottom))
                    .style(line_style(Palette::dim(), 3.0))
                    .build()?,
            ),
        ];
        let me = Self {
            group: parts[0],
            x,
            top,
            bottom,
            decades,
        };
        for d in 0..=decades {
            let y = me.y(10f64.powi(-(d as i32)));
            parts.push(
                stage.arena_mut().add(
                    Line::new(v(x - 0.12, y), v(x + 0.12, y))
                        .style(line_style(Palette::dim(), 3.0))
                        .build()?,
                ),
            );
            let label = if d == 0 {
                "1".to_owned()
            } else {
                format!("1/{}", explainer_kit::thousands(10u64.pow(d)))
            };
            let text = kit.text(stage, &label, 44.0, Palette::ink(), v(x - 0.35, y))?;
            stage.move_to(text, v(x - 0.35, y), RIGHT);
            parts.push(text);
        }
        parts.push(kit.text_styled(stage, "model", 24.0, Palette::dim(), v(x, top + 0.4), false, true)?);
        let group = group_of(stage, &parts)?;
        Ok(Self { group, ..me })
    }

    /// The height of chance `p` on the scale.
    pub fn y(&self, p: f64) -> f64 {
        let t = (-p.log10() / f64::from(self.decades)).clamp(0.0, 1.0);
        self.top + (self.bottom - self.top) * t
    }

    /// A rung at `p`: a short bar across the scale (detached).
    pub fn rung(&self, stage: &mut Stage<'_>, p: f64, color: Srgb) -> fmn::Result<Mob> {
        let y = self.y(p);
        let m = Line::new(v(self.x - 0.32, y), v(self.x + 0.32, y))
            .style(line_style(color, 4.0))
            .build()?;
        Ok(stage.arena_mut().add(m))
    }

    /// A marker at `p` with its `label` to the right (detached): where the
    /// risk stands now.
    pub fn marker(&self, kit: &Kit, stage: &mut Stage<'_>, p: f64, label: &str, color: Srgb) -> fmn::Result<Mob> {
        let y = self.y(p);
        let tip = Polygon::new([v(self.x + 0.2, y), v(self.x + 0.5, y + 0.15), v(self.x + 0.5, y - 0.15)])
            .style(flat(color, 1.0))
            .build();
        let tip = stage.arena_mut().add(tip);
        let text = kit.text_left(stage, label, 30.0, color, v(self.x + 0.65, y))?;
        group_of(stage, &[tip, text])
    }

    /// A labelled tick on the left of the scale at `p` (17's 1/100, 1/10⁴,
    /// 1/10⁶ requirements), detached.
    pub fn tick(&self, kit: &Kit, stage: &mut Stage<'_>, p: f64, label: &str) -> fmn::Result<Mob> {
        let y = self.y(p);
        let line = stage.arena_mut().add(
            Line::new(v(self.x - 0.3, y), v(self.x, y))
                .style(line_style(Palette::ink(), 3.0))
                .build()?,
        );
        let text = kit.text(stage, label, 48.0, Palette::ink(), v(self.x - 0.6, y))?;
        stage.move_to(text, v(self.x - 0.4, y), RIGHT);
        group_of(stage, &[line, text])
    }
}

// --------------------------------------------------------------------- ring

/// The ring of five (06_4, 16): the cards A ⊕ B, B ⊕ C, C ⊕ D, D ⊕ E,
/// E ⊕ A on a pentagon, the first at the top, going clockwise.
pub struct RingProp {
    pub group: Mob,
    pub cards: Vec<Card>,
    pub centre: Vec3,
    pub radius: f64,
}

impl RingProp {
    /// Cards `h` tall on a circle of `radius` round `centre`, with their
    /// results when `results`.
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        centre: Vec3,
        radius: f64,
        h: f64,
        results: bool,
    ) -> fmn::Result<Self> {
        let mut cards = Vec::with_capacity(RING.len());
        for (k, &c) in RING.iter().enumerate() {
            let a = FRAC_PI_2 - TAU * k as f64 / RING.len() as f64;
            let at = v(centre[0] + radius * a.cos(), centre[1] + radius * a.sin());
            cards.push(card(kit, stage, c, results.then(|| result(c)), h, at)?);
        }
        let groups: Vec<Mob> = cards.iter().map(|c| c.group).collect();
        let group = group_of(stage, &groups)?;
        Ok(Self {
            group,
            cards,
            centre,
            radius,
        })
    }
}

// --------------------------------------------------------------------- grid

/// Cards as a grid (03_2): a row per card, a column per piece (headed by
/// its badge), a dot where the card uses the piece, and the result at the
/// end of the row.
pub struct GridProp {
    pub group: Mob,
    /// The header badges, one per piece column.
    pub heads: Vec<Mob>,
    /// One row per card: its dots (with their columns) and its result.
    pub rows: Vec<(Vec<(usize, Mob)>, Mob)>,
    /// Where row r and column c meet.
    pub origin: Vec3,
    pub pitch: f64,
}

impl GridProp {
    /// The grid of `cards` over `pieces` columns, cells `pitch` apart, its
    /// top-left cell centred at `origin`. Rows are added detached.
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        cards: &[u32],
        pieces: usize,
        origin: Vec3,
        pitch: f64,
    ) -> fmn::Result<Self> {
        let at = |r: usize, c: usize| v(origin[0] + pitch * c as f64, origin[1] - pitch * r as f64);
        let mut heads = Vec::with_capacity(pieces);
        for p in 0..pieces {
            let c = at(0, p);
            heads.push(badge(kit, stage, p, pitch * 0.3, v(c[0], c[1] + pitch))?);
        }
        let mut parts: Vec<Mob> = heads.clone();
        let mut rows = Vec::with_capacity(cards.len());
        for (r, &card) in cards.iter().enumerate() {
            let mut dots = Vec::new();
            for p in pieces_of(card) {
                let d = Dot::new()
                    .point(at(r, p))
                    .radius(pitch * 0.16)
                    .color(Role::ex_or())
                    .build();
                let d = stage.arena_mut().add(d);
                dots.push((p, d));
                parts.push(d);
            }
            let end = at(r, pieces);
            let res = kit.text(
                stage,
                &result(card).to_string(),
                pitch * 62.0,
                Palette::ink(),
                v(end[0] + pitch * 0.2, end[1]),
            )?;
            parts.push(res);
            rows.push((dots, res));
        }
        // faint guides between the columns and the result
        let rule_x = at(0, pieces)[0] - pitch * 0.45;
        let rule = Line::new(
            v(rule_x, origin[1] + pitch * 0.5),
            v(rule_x, origin[1] - pitch * (cards.len() as f64 - 0.5)),
        )
        .style(line_style(Palette::faint(), 2.5))
        .build()?;
        parts.push(stage.arena_mut().add(rule));
        let group = group_of(stage, &parts)?;
        Ok(Self {
            group,
            heads,
            rows,
            origin,
            pitch,
        })
    }
}

#[cfg(test)]
mod tests {
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::*;
    use crate::model::CARDS;

    fn run(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
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

    #[test]
    fn the_bucket_stacks_its_slots_inside_its_walls() {
        run(|stage| {
            let b = BucketProp::build(stage, 4, v(0.0, 0.0), 2.0, 3.0)?;
            assert_eq!(b.slots.len(), 4);
            let ys: Vec<f64> = (0..4).map(|i| b.slot_centre(i)[1]).collect();
            assert!(ys.windows(2).all(|w| w[1] > w[0]), "slot 0 at the bottom: {ys:?}");
            for &s in &b.slots {
                assert!(stage.get_bottom(s)[1] > -1.5 && stage.get_top(s)[1] < 1.5);
                assert!(stage.get_width(s) < 2.0);
            }
            let w = b.water(stage, 3, Role::ex_or())?;
            let dy = (stage.get_center(w)[1] - ys[3]).abs();
            assert!(dy < 0.01, "the water sits on its slot: {dy}");
            let half = b.level(stage, 0.0, 0.5, Role::ex_or())?;
            assert!(
                stage.get_top(half)[1] < 0.01 && stage.get_top(half)[1] > -0.01,
                "half full"
            );
            Ok(())
        });
    }

    #[test]
    fn the_danger_meter_reads_its_tracker() {
        run(|stage| {
            let kit = Kit::new()?;
            let b = BucketProp::build(stage, 10, v(0.0, -0.5), 2.0, 4.0)?;
            let m = DangerMeter::on(&kit, stage, &b, 0.0)?;
            let title = explainer_kit::children(stage, m.group)[1];
            assert!(
                stage.get_bottom(title)[1] > stage.get_top(m.track)[1] + 0.08,
                "the explanation needs daylight above the meter"
            );
            stage.add_to_scene(m.fill)?;
            stage.play(m.to(0.5, 0.3)?)?;
            assert!((stage.tracker_value(m.chance).unwrap() - 0.5).abs() < 1e-9);
            let w = stage.get_width(m.fill);
            assert!((w - 1.0).abs() < 0.05, "half the track: {w}");
            Ok(())
        });
    }

    #[test]
    fn the_ladder_puts_each_power_of_ten_on_its_rung() {
        run(|stage| {
            let kit = Kit::new()?;
            let l = RiskLadder::build(&kit, stage, v(0.0, 0.0), 6.0, 6)?;
            assert!((l.y(1.0) - 3.0).abs() < 1e-12 && (l.y(1e-6) + 3.0).abs() < 1e-12);
            assert!((l.y(1e-3)).abs() < 1e-12, "a thousandth halfway down");
            // a halving is the same step anywhere on the scale
            let step = |p: f64| l.y(p) - l.y(p / 2.0);
            assert!((step(0.5) - step(1e-4)).abs() < 1e-12);
            Ok(())
        });
    }

    #[test]
    fn large_ladder_labels_fit_the_narrowest_chapter_margin() {
        run(|stage| {
            let kit = Kit::new()?;
            for (x, decades) in [(-3.5, 6), (-1.6, 8)] {
                let l = RiskLadder::build(&kit, stage, v(x, 0.0), 5.8, decades)?;
                assert!(stage.get_left(l.group)[0] > -7.0, "risk fractions leave the frame");
                assert!(stage.get_right(l.group)[0] < 7.0);
            }
            Ok(())
        });
    }

    #[test]
    fn the_ring_and_the_grid_hold_their_cards() {
        run(|stage| {
            let kit = Kit::new()?;
            let r = RingProp::build(&kit, stage, v(0.0, 0.0), 2.2, 0.6, false)?;
            assert_eq!(r.cards.len(), 5);
            let top = stage.get_center(r.cards[0].group);
            assert!(top[1] > 1.9, "the first card at the top: {top:?}");
            let g = GridProp::build(&kit, stage, &CARDS, 4, v(-1.0, 1.0), 0.6)?;
            let dots: usize = g.rows.iter().map(|(d, _)| d.len()).sum();
            assert_eq!(dots, CARDS.iter().map(|c| c.count_ones() as usize).sum::<usize>());
            Ok(())
        });
    }
}
