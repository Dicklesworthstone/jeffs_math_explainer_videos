//! An n-hour clock face (07's 256-hour clock, 08's seven-hour one, 11_8's
//! pair): a ring, a tick per hour, labels on the named hours, and a hand
//! that a tracker drives. Hour 0 sits at the top; hours run clockwise.

use std::f64::consts::{FRAC_PI_2, TAU};
use std::rc::Rc;

use explainer_kit::{Kit, Palette, stroke_style, v};
use fmn::prelude::*;

use super::draw::group_of;

pub struct Clock {
    pub hours: usize,
    pub centre: Vec3,
    pub radius: f64,
    pub ring: Mob,
    pub ticks: Mob,
    /// The labelled hours and their labels.
    pub labels: Vec<(usize, Mob)>,
    /// Ring, ticks and labels.
    pub group: Mob,
}

impl Clock {
    /// A clock of `hours` hours, `labelled` hours named at `label_size` pt
    /// outside the ring. Clocks of more than 24 hours draw short ticks, every
    /// sixteenth longer; small clocks draw a dot per hour.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        hours: usize,
        centre: Vec3,
        radius: f64,
        color: Srgb,
        labelled: &[usize],
        label_size: f64,
    ) -> fmn::Result<Self> {
        let ring = Circle::new()
            .radius(radius)
            .arc_center(centre)
            .style(stroke_style(color, 3.0))
            .build();
        let ring = stage.arena_mut().add(ring);
        let at = |h: f64, r: f64| {
            let a = FRAC_PI_2 - TAU * h / hours as f64;
            v(centre[0] + r * a.cos(), centre[1] + r * a.sin())
        };
        let mut marks: Vec<VMobject> = Vec::with_capacity(hours);
        for h in 0..hours {
            if hours > 24 {
                let long = h % 16 == 0;
                let inner = radius * if long { 0.9 } else { 0.95 };
                let tick = Line::new(at(h as f64, inner), at(h as f64, radius))
                    .style(stroke_style(color, if long { 2.5 } else { 1.2 }))
                    .build()?;
                marks.push(tick);
            } else {
                marks.push(Dot::new().point(at(h as f64, radius)).radius(0.07).color(color).build());
            }
        }
        let ticks = stage.arena_mut().add(v_group(marks));
        let mut labels = Vec::new();
        for &h in labelled {
            let p = at(h as f64, radius + 0.42 * label_size / 30.0);
            labels.push((h, kit.text(stage, &h.to_string(), label_size, Palette::ink(), p)?));
        }
        let mut all = vec![ring, ticks];
        all.extend(labels.iter().map(|&(_, m)| m));
        let group = group_of(stage, &all)?;
        Ok(Self {
            hours,
            centre,
            radius,
            ring,
            ticks,
            labels,
            group,
        })
    }

    /// The point at `hour` (fractional hours allowed), `scale` × the radius
    /// from the centre.
    pub fn point(&self, hour: f64, scale: f64) -> Vec3 {
        let a = FRAC_PI_2 - TAU * hour / self.hours as f64;
        v(
            self.centre[0] + scale * self.radius * a.cos(),
            self.centre[1] + scale * self.radius * a.sin(),
        )
    }

    /// A hand from the centre to the ring at the hour `tracker` holds, with
    /// a dot at its tip; rebuilt each frame (two small shapes). Not yet in
    /// the scene: `stage.add_to_scene` it.
    pub fn hand(&self, stage: &mut Stage<'_>, tracker: Mob, color: Srgb) -> Mob {
        let (centre, radius, hours) = (self.centre, self.radius, self.hours as f64);
        stage.always_redraw(move |s| {
            let h = s.tracker_value(tracker).unwrap_or(0.0);
            let a = FRAC_PI_2 - TAU * h / hours;
            let tip = v(centre[0] + radius * a.cos(), centre[1] + radius * a.sin());
            let stem = v(centre[0] + 0.86 * radius * a.cos(), centre[1] + 0.86 * radius * a.sin());
            let line = Line::new(centre, stem)
                .style(stroke_style(color, 3.0))
                .build()
                .expect("a hand has length");
            let dot = Dot::new().point(tip).radius(0.09).color(color).build();
            s.add(v_group([line, dot]))
        })
    }

    /// The hour the hand shows, wrapped to the clock: a live number at
    /// `at`, rebuilt each frame from `tracker` (its whole part, mod hours).
    pub fn readout(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        tracker: Mob,
        size: f64,
        color: Srgb,
        at: Vec3,
    ) -> fmn::Result<Mob> {
        let hours = self.hours as f64;
        let book = Rc::clone(&kit.book);
        let m = stage.always_redraw(move |s| {
            let h = s.tracker_value(tracker).unwrap_or(0.0).floor().rem_euclid(hours);
            let d = DecimalNumber::new(h)
                .num_decimal_places(0)
                .font_size(size)
                .color(color)
                .build(&book)
                .expect("bundled digits always lay out");
            explainer_kit::glyphs::record(explainer_kit::TextKind::Text, d.num_string());
            let m = s.add(d.into_vmob());
            s.move_to(m, at, ORIGIN);
            m
        });
        stage.add_to_scene(m)?;
        Ok(m)
    }
}
