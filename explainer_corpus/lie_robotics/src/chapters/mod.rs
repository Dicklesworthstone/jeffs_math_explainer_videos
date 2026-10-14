//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where a picture first makes an idea true: the flat piece,
//! the screw, the product of exponentials; quieter for Apollo's story and
//! the honest limits). Every chapter is a real scene: the animatics that
//! stood in for them while they were built are gone.

pub mod by_hand;
pub mod draw;
pub mod everywhere;
pub mod glyphs;
pub mod halfway;
pub mod lost_directions;
pub mod one_turn;
pub mod screws;
pub mod seven_screws;
pub mod sound;
pub mod spin;
pub mod steps;
pub mod the_book;
pub mod the_platform;
pub mod the_spare_joint;
pub mod the_steady_path;
pub mod three_angles;
pub mod thumbnail;
pub mod velocities_add;

use explainer_kit::Chapter;

macro_rules! scene {
    ($key:literal, $intensity:expr, $module:ident :: $scene:ident) => {
        Chapter {
            key: $key,
            make: |kit| Box::new($module::$scene { kit }),
            intensity: $intensity,
        }
    };
}

pub fn registry() -> Vec<Chapter> {
    vec![
        scene!("01_the_book", 0.4, the_book::TheBook),
        scene!("02_one_turn", 0.5, one_turn::OneTurn),
        scene!("03_three_angles", 0.4, three_angles::ThreeAngles),
        scene!("04_halfway", 0.6, halfway::Halfway),
        scene!("05_spin", 0.55, spin::Spin),
        scene!("06_the_platform", 0.5, the_platform::ThePlatform),
        scene!("07_screws", 0.6, screws::Screws),
        scene!("08_seven_screws", 0.6, seven_screws::SevenScrews),
        scene!("09_by_hand", 0.5, by_hand::ByHand),
        scene!("10_velocities_add", 0.5, velocities_add::VelocitiesAdd),
        scene!("11_lost_directions", 0.4, lost_directions::LostDirections),
        scene!("12_the_spare_joint", 0.5, the_spare_joint::TheSpareJoint),
        scene!("13_steps", 0.55, steps::Steps),
        scene!("14_the_steady_path", 0.5, the_steady_path::TheSteadyPath),
        scene!("15_everywhere", 0.65, everywhere::Everywhere),
    ]
}

#[cfg(test)]
mod tests {
    use explainer_kit::{Kit, v};
    use fmn::prelude::*;

    use super::draw::{Ask, QuizCard, QuizLook};
    use super::*;

    /// Every chapter's quiz card, built at the one scale (the ratios are the
    /// same at any scale): the tick of the answer's row clears the row's
    /// words, and no row's words reach the card's right edge.
    #[test]
    fn every_quiz_answer_row_leaves_room_for_its_tick() {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let asks: [(&str, Ask); 15] = [
            ("01", the_book::ASK),
            ("02", one_turn::ASK),
            ("03", three_angles::ASK),
            ("04", halfway::ASK),
            ("05", spin::ASK),
            ("06", the_platform::ASK),
            ("07", screws::ASK),
            ("08", seven_screws::ASK),
            ("09", by_hand::ASK),
            ("10", velocities_add::ASK),
            ("11", lost_directions::ASK),
            ("12", the_spare_joint::ASK),
            ("13", steps::ASK),
            ("14", the_steady_path::ASK),
            ("15", everywhere::ASK),
        ];
        let mut crowded = Vec::new();
        fmn::run_scene(
            &mut Probe(|stage: &mut Stage<'_>| {
                let kit = Kit::new()?;
                for (chapter, ask) in &asks {
                    let card = QuizCard::build(&kit, stage, *ask, QuizLook::at(v(0.0, 0.0), 1.0))?;
                    let answer = card.rows[ask.answer_index()?];
                    let tick_left = stage.get_left(card.verdict[1])[0];
                    let words_right = stage.get_right(answer)[0];
                    if words_right > tick_left - 0.2 {
                        crowded.push(format!(
                            "{chapter} {:?}: words end at {words_right:.2}, the tick starts at {tick_left:.2}",
                            ask.choices[ask.answer_index()?].label
                        ));
                    }
                    let edge = stage.get_right(card.back)[0];
                    for &row in &card.rows {
                        if stage.get_right(row)[0] > edge - 0.15 {
                            crowded.push(format!("{chapter}: a row reaches the card's edge"));
                        }
                    }
                }
                Ok(())
            }),
            fmn::scene::RuntimeConfig::default(),
            7,
            &mut fmn::scene::NullSceneSink,
        )
        .expect("the probe scene runs");
        assert!(
            crowded.is_empty(),
            "answer rows crowd their tick:\n{}",
            crowded.join("\n")
        );
    }
}
