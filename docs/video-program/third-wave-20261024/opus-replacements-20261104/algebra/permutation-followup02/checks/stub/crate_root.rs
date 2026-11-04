//! Compiles the real candidate permutations.rs against the stand-in crates, runs
//! its tests, and runs `construct` to report the clock and the object ledger.
use explainer_kit::Kit;
use fmn::prelude::*;

#[path = "../../candidate/explainer_corpus/abstract_algebra_expanded/src/permutations.rs"]
mod permutations;

#[allow(dead_code)]
mod tile_model {
    pub type Point = [f64; 3];
    pub const CORNERS: [Point; 4] = [[1., 1., 0.], [1., -1., 0.], [-1., -1., 0.], [-1., 1., 0.]];
}

#[allow(dead_code)]
mod d4 {
    pub fn apply(mut point: (i32, i32), turns: u8, flip: bool) -> (i32, i32) {
        if flip {
            point.1 = -point.1;
        }
        for _ in 0..turns {
            point = (-point.1, point.0);
        }
        point
    }
}

#[allow(dead_code)]
mod d4_group {
    use explainer_kit::{Kit, Palette};
    use fmn::prelude::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Elem {
        pub turns: u8,
        pub flip: bool,
    }

    pub const E: Elem = Elem::new(0, false);
    pub const R: Elem = Elem::new(1, false);
    pub const R2: Elem = Elem::new(2, false);
    pub const R3: Elem = Elem::new(3, false);
    pub const S: Elem = Elem::new(0, true);
    pub const RS: Elem = Elem::new(1, true);
    pub const R2S: Elem = Elem::new(2, true);
    pub const R3S: Elem = Elem::new(3, true);
    pub const ALL: [Elem; 8] = [E, R, R2, R3, S, RS, R2S, R3S];

    impl Elem {
        pub const fn new(turns: u8, flip: bool) -> Self {
            Self { turns: turns % 4, flip }
        }
        pub fn mul(self, other: Elem) -> Elem {
            let carried = if self.flip { (4 - other.turns) % 4 } else { other.turns };
            Elem::new(self.turns + carried, self.flip ^ other.flip)
        }
    }

    pub fn group_of(_: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
        // A token: its disk and label move as one, so only the disk's box counts.
        let (x, y, w, h) = fmn::stub::with(|book| {
            let disk = &book.mobs[members[0].0];
            (disk.x, disk.y, disk.w, disk.h)
        });
        Ok(fmn::stub::add(x, y, w, h, None))
    }

    pub fn corner_color(corner: usize) -> Srgb {
        [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()][corner]
    }

    pub fn thumb(_: &mut Stage<'_>, _: &Kit, _: Elem, at: (f64, f64), k: f64) -> fmn::Result<Mob> {
        Ok(fmn::stub::add(at.0, at.1, 0.96 * k, 0.96 * k, None))
    }

    fn play(run_time: f64, effects: Vec<fmn::Effect>) {
        let animation = fmn::Anim { run_time, effects };
        fmn::stub::play(&[&animation]);
    }

    pub fn show(_: &mut Stage<'_>, mobs: &[Mob], run_time: f64) -> fmn::Result<()> {
        play(run_time, mobs.iter().map(|&mob| fmn::Effect::Show(mob)).collect());
        Ok(())
    }

    pub fn hide(_: &mut Stage<'_>, mobs: &[Mob], run_time: f64) -> fmn::Result<()> {
        play(run_time, mobs.iter().map(|&mob| fmn::Effect::Hide(mob)).collect());
        Ok(())
    }

    pub fn glide(_: &mut Stage<'_>, mob: Mob, to: (f64, f64), run_time: f64) -> fmn::Result<()> {
        play(run_time, vec![fmn::Effect::Move(mob, to.0, to.1)]);
        Ok(())
    }

    pub fn glide_all(_: &mut Stage<'_>, moves: &[(Mob, (f64, f64))], run_time: f64) -> fmn::Result<()> {
        play(
            run_time,
            moves.iter().map(|&(mob, to)| fmn::Effect::Move(mob, to.0, to.1)).collect(),
        );
        Ok(())
    }
}

fn listed<T: std::fmt::Display>(values: impl IntoIterator<Item = T>) -> String {
    values
        .into_iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

const SCRIPT: &str = include_str!("../../candidate/explainer_corpus/abstract_algebra_expanded/script.tsv");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut facts = explainer_kit::Facts::default();
    permutations::add_facts(&mut facts)?;

    let rate: f64 = std::env::args().nth(1).map_or(Ok(2.68), |rate| rate.parse())?;
    let kit = Kit::with_script(SCRIPT, rate);
    let mut scene = permutations::Permutations { kit: kit.clone() };
    let mut stage = Stage::new();
    println!("scene {} at {rate} words per second", scene.name());
    scene.construct(&mut stage)?;

    println!("id\tstart\tvoice\tpicture_over\tlatest_beat\tin_sentence");
    let mut end: f64 = 0.0;
    for cue in kit.reports() {
        println!(
            "{}\t{:.1}\t{:.1}\t{:+.1}\t{:.1}\t{}",
            cue.id,
            cue.start,
            cue.length,
            cue.picture_end - cue.start - cue.length,
            cue.latest,
            cue.latest_sentence
        );
        end = end.max(cue.picture_end);
    }
    println!("chapter ends at {end:.1} s");

    let problems = fmn::stub::with(|book| book.problems.clone());
    println!("{} object-ledger problems", problems.len());
    for problem in problems {
        println!("  {problem}");
    }
    let overlaps = kit.overlaps();
    println!("{} estimated overlaps or frame exits", overlaps.len());
    for overlap in overlaps {
        println!("  {overlap}");
    }
    Ok(())
}
