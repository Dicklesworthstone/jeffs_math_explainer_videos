//! A finite counterexample to an overbroad impossibility claim. Unbuilt draft.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use super::visual::fade;

pub struct ThreeRows { pub kit: Kit }

impl SceneConstruct for ThreeRows {
    fn name(&self) -> &str { "07_three_rows" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let rows = [[2_u32, 3, 10], [2, 4, 10], [2, 4, 11]];
        for (index, row) in rows.iter().enumerate() {
            let house = 15 + index as u32;
            if row.iter().sum::<u32>() != house {
                return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("bad seat total")));
            }
            for i in 0..3 {
                let (lower, upper) = crate::facts::quota_bounds(house, i).ok_or(
                    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("bad region")))?;
                if !(lower..=upper).contains(&u64::from(row[i])) || (index > 0 && row[i] < rows[index - 1][i]) {
                    return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("quota or monotonicity failure")));
                }
            }
        }
        kit.say(stage, "07_1")?;
        let title = kit.text(stage, "Must quota and no take-backs conflict?", 35.0, Palette::ink(), v(0.0, 2.8))?;
        fade(stage, &[title], true)?;
        kit.until_sentence(stage, "07_1", 1)?;
        let heading = kit.text(stage, "House size          A       B       C", 29.0, Palette::dim(), v(0.0, 1.5))?;
        fade(stage, &[heading], true)?;
        for (i, row) in rows.iter().enumerate() {
            let line = format!("{} seats               {}       {}       {}", 15 + i, row[0], row[1], row[2]);
            let mob = kit.text(stage, &line, 31.0, Palette::green(), v(0.0, 0.55 - i as f64 * 0.9))?;
            fade(stage, &[mob], true)?;
        }
        kit.say(stage, "07_2")?;
        let note = kit.text(stage, "Three valid rows, not a general algorithm", 27.0, Palette::gold(), v(0.0, -2.5))?;
        fade(stage, &[note], true)?;
        kit.finish(stage)
    }
}
