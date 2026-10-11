//! The mystery lake's score curve (05_6): every lake size from the smallest
//! possible to 60, each with its score (the chance of the catch marked 10,
//! caught 10, caught twice 3, if the lake were that size) as a purple bar
//! on one shared frame. Chapter 05 builds it, 06 ends on it (the landscape's
//! k = 3 row seen side-on), and 07 opens with two of its bars.

use std::ops::RangeInclusive;

use explainer_kit::{Frame2, Kit, Palette, rect, v};
use fmn::prelude::*;

use super::draw::{Role, flat, line_style};
use crate::facts::TOY;
use crate::law::Law;

/// The largest lake size on the curve.
pub const SIZE_MAX: u64 = 60;
/// A bar's width, as a share of one lake size.
pub const BAR: f64 = 0.72;

/// The sizes on the curve: from the smallest lake that fits the fish we
/// have seen (marked + caught − caught twice) to [`SIZE_MAX`].
pub fn sizes() -> RangeInclusive<u64> {
    (TOY.0 + TOY.1 - TOY.2)..=SIZE_MAX
}

/// The frame every chapter draws the curve in: x in lake sizes (half a size
/// of room either side), y in score from 0 to 0.35.
pub fn frame() -> Frame2 {
    Frame2 {
        center: v(0.4, -0.45),
        width: 11.6,
        height: 4.4,
        x: (*sizes().start() as f64 - 1.5, SIZE_MAX as f64 + 1.5),
        y: (0.0, 0.35),
    }
}

/// The score of a lake size: P_N(K = 3) for the mystery lake's catch.
pub fn score(lake: u64) -> f64 {
    Law::new(lake, TOY.0, TOY.1).at(TOY.2)
}

/// One bar's rectangle for lake size `lake`, standing on the frame's floor.
pub fn bar(frame: &Frame2, lake: u64, height: f64) -> fmn::Result<VMobject> {
    let w = BAR * frame.width / (frame.x.1 - frame.x.0);
    let base = frame.p(lake as f64, 0.0);
    let top = frame.p(lake as f64, height);
    rect(
        v(base[0] - w / 2.0, base[1]),
        w,
        (top[1] - base[1]).max(0.004),
        flat(Role::score(), 0.9),
    )
}

/// Every size's bar, in size order.
pub fn bars(frame: &Frame2) -> fmn::Result<Vec<VMobject>> {
    sizes().map(|n| bar(frame, n, score(n))).collect()
}

/// The axis, detached: the floor, a tick and its number at the smallest
/// size and every ten, the words "lake size" under it and "score" by it.
pub fn axis(kit: &Kit, stage: &mut Stage<'_>, frame: &Frame2) -> fmn::Result<Mob> {
    let floor = frame.bottom();
    let mut lines = vec![
        Line::new(v(frame.left(), floor), v(frame.right(), floor))
            .style(line_style(Palette::dim(), 2.5))
            .build()?,
    ];
    let first = *sizes().start();
    let mut ticks: Vec<u64> = vec![first];
    ticks.extend(
        (first.div_ceil(10) * 10..=SIZE_MAX)
            .step_by(10)
            .filter(|&t| t > first + 2),
    );
    for &t in &ticks {
        let x = frame.p(t as f64, 0.0)[0];
        lines.push(
            Line::new(v(x, floor), v(x, floor - 0.12))
                .style(line_style(Palette::dim(), 2.0))
                .build()?,
        );
    }
    let g = stage.arena_mut().add(v_group(lines));
    for &t in &ticks {
        let x = frame.p(t as f64, 0.0)[0];
        let m = kit.text(stage, &t.to_string(), 22.0, Palette::dim(), v(x, floor - 0.32))?;
        stage.attach(g, m)?;
    }
    let size_word = kit.text(
        stage,
        "lake size",
        26.0,
        Palette::dim(),
        v(frame.center[0], floor - 0.72),
    )?;
    stage.attach(g, size_word)?;
    let score_word = kit.text(
        stage,
        "score",
        26.0,
        Role::score(),
        v(frame.left() + 0.55, frame.top() - 0.1),
    )?;
    stage.attach(g, score_word)?;
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::best_sizes;

    #[test]
    fn the_curve_runs_from_the_smallest_lake_and_peaks_at_the_soup_guess() {
        assert_eq!(*sizes().start(), 17, "A-F5.min_lake");
        let best = sizes().max_by(|&a, &b| score(a).total_cmp(&score(b))).unwrap();
        assert_eq!(Some(&best), best_sizes(TOY).last(), "A-F5.argmax");
        // the exact score at 33 is 7429/23374 (A-F5.L33)
        assert!((score(33) - 7429.0 / 23374.0).abs() < 1e-12);
        // every bar fits under the frame's top
        assert!(sizes().all(|n| score(n) < frame().y.1));
    }

    #[test]
    fn the_frame_puts_every_size_inside_it() {
        let f = frame();
        for n in sizes() {
            let x = f.p(n as f64, 0.0)[0];
            assert!(x > f.left() && x < f.right());
        }
    }
}
