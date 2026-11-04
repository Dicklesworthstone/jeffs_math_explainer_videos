//! The fifteen chapters, in storyboard order, all native scenes (none plays
//! as the kit's storyboard animatic any more). `docs/storyboard.md`
//! gives every chapter's beats; the tests in `main.rs` hold the registry, the
//! script and the prediction windows to it.

pub mod thumbnail;

mod boundary;
mod closing;
mod continuing;
mod counting;
mod draw;
mod gallery;
mod loop_sums;
mod one_derivative;
mod one_point;
mod reading_color;
mod real_integral;
mod residues;
mod rigidity;
mod the_bump;
mod turning;
mod two_planes;

use explainer_kit::Chapter;

/// Every ⟨predict⟩ beat in the storyboard with its listen window, seconds.
/// The native scenes count down exactly this long (`draw::predict`); the
/// test `every_prediction_has_its_window` checks the list against the
/// storyboard both ways.
pub const PREDICTS: &[(&str, f64)] = &[
    ("01_4", 5.0),
    ("02_5", 5.0),
    ("03_3", 4.0),
    ("04_6", 5.0),
    ("06_8", 5.0),
    ("07_3", 5.0),
    ("09_4", 5.0),
    ("09_8", 6.0),
    ("10_4", 5.0),
    ("11_5", 6.0),
    ("13_5", 5.0),
    ("14_4", 5.0),
    ("15_3", 6.0),
];

/// Seconds the picture holds still after these cues (`draw::reading_window`), past the
/// voice and the kit's breath, so a `precisely:` card can be read with nothing else being
/// said before the next cue clears it. Each is a teaching choice made from the card's text
/// and what competes with it, not a measurement of any take or reader. The test
/// `every_reading_window_is_in_the_storyboard` checks the list against the storyboard both
/// ways; `each_card_outlasts_its_reading_window` checks the order at each site.
pub const READING_WINDOWS: &[(&str, f64)] = &[
    // a relation in words with three letters; the voice is counting turns, not reading it out
    ("04_4", 2.5),
    // one clause on how the plot was made; only the cue's last five words overlap it
    ("05_8", 2.0),
    // a condition on m − 1 derivatives and its result, as a new heading arrives beside it
    ("07_2", 3.0),
    // one exact hypothesis; the voice is saying nearly the same thing
    ("09_5", 2.0),
    // a whole theorem, three hypotheses and a conclusion, beside the condition card
    ("10_5", 3.0),
    // a note on how the output is drawn, with no argument to follow
    ("11_6", 1.5),
    // two uniqueness claims told apart, at the end of the longest cue, before a question
    ("14_3", 2.5),
];

/// Glyph supplements: the scenes' fixed strings, plus the characters the
/// fact-built readouts can draw (digits, signs, the minus sign, degrees).
/// The kit's glyph probe also captures every string a chapter registers
/// when it renders; this list makes the fixed ones visible without a render.
pub const TEXT: &[&str] = &[
    "0123456789 . , + − × ÷ ≈ → ° i π √ ≥ <",
    the_bump::TITLE,
    the_bump::DEGREE,
    the_bump::RECIPE,
    the_bump::BUMP_IS,
    the_bump::ASK[0],
    the_bump::ASK[1],
    the_bump::POLE_I,
    the_bump::POLE_MINUS_I,
    the_bump::RADIUS,
    the_bump::AT_X,
    the_bump::COMING_UP,
    reading_color::DIRECTIONS[0],
    reading_color::DIRECTIONS[1],
    reading_color::DIRECTIONS[2],
    reading_color::DIRECTIONS[3],
    reading_color::BAND,
    reading_color::PHASE,
    reading_color::TURNS,
    reading_color::ORDER_TWO,
    reading_color::SIMPLE_POLE,
    reading_color::NEAR_ZERO,
    reading_color::ASK[0],
    reading_color::ASK[1],
    reading_color::EACH,
    one_derivative::LABELS[0],
    one_derivative::LABELS[1],
    one_derivative::STRETCH,
    one_derivative::TURN,
    one_derivative::GAP,
    one_derivative::ASK[0],
    one_derivative::RIGHT_UP[0],
    one_derivative::RIGHT_UP[1],
    one_derivative::STRETCH_TURN,
    one_derivative::MOVED,
    one_derivative::ZOOMED,
    one_point::KEPT,
    one_point::MULTIPLIED,
    one_point::CRITICAL,
    one_point::ONE_HUE,
    one_point::ASK[0],
    one_point::RATIO,
    one_point::SETTLES,
    one_point::RIGHT_UP[0],
    one_point::RIGHT_UP[1],
    one_point::ONLY_HERE,
    one_point::PRODUCT,
    one_point::INSIDE_RADIUS,
    one_point::HOLOMORPHIC[0],
    one_point::HOLOMORPHIC[1],
    one_point::HOLOMORPHIC[2],
    one_point::LONE,
    one_point::FROM_HERE,
    counting::RULE,
    counting::NEEDS[0],
    counting::NEEDS[1],
    counting::NEEDS[2],
    // counting::LOCAL[1] and [2] repeat reading_color::ORDER_TWO and SIMPLE_POLE
    counting::LOCAL[0],
    counting::TURNS,
    counting::NET,
    counting::THROUGH_POLE,
    counting::ASK[0],
    counting::ASK[1],
    counting::INSIDE,
    counting::SQUASHED,
    counting::WALKER_DOG[0],
    counting::WALKER_DOG[1],
    counting::STEP_SKETCH[0],
    counting::STEP_SKETCH[1],
    counting::STEP_SKETCH[2],
    counting::SAMPLES_SAY[0],
    counting::SAMPLES_SAY[1],
    counting::TRULY,
    counting::REACH_IS,
    counting::REFUSE,
    counting::PASSES,
    counting::REACH_COVERS,
    counting::CERTAIN,
    real_integral::ASK[0],
    real_integral::APPROX,
    real_integral::NUMERIC[0],
    real_integral::NUMERIC[1],
    real_integral::HONEST,
    real_integral::SIZE_ALONG,
    real_integral::READ_R,
    real_integral::AT_MOST,
    real_integral::UNDER,
    boundary::REST,
    boundary::AVERAGE,
    boundary::VALUES,
    boundary::AVG_OF,
    boundary::COLON,
    boundary::ASK[0],
    boundary::LIMIT,
    boundary::SHARES,
    boundary::NEEDS,
    boundary::TURNED[0],
    boundary::TURNED[1],
    boundary::ASK_RECENTRE[0],
    boundary::ASK_RECENTRE[1],
    boundary::WORKS,
    boundary::CHECK,
    rigidity::SIN_3I[0],
    rigidity::SIN_3I[1],
    rigidity::THEOREM[0],
    rigidity::THEOREM[1],
    rigidity::THEOREM[2],
    rigidity::WHY[0],
    rigidity::WHY[1],
    rigidity::PILING,
    rigidity::REAL_LINE,
    rigidity::SPREADS,
    rigidity::SUM,
    rigidity::DERIVS,
    rigidity::RECIPE_AT,
    rigidity::ZEROS[0],
    rigidity::ASK[0],
    rigidity::NOT_IN,
    rigidity::CARD[0],
    rigidity::CARD[1],
    rigidity::CARD[2],
    rigidity::EXACT,
    rigidity::LIOUVILLE[0],
    rigidity::LIOUVILLE[1],
    rigidity::REAL_BOUND,
    residues::H_NOTE,
    residues::TOTALS[0],
    residues::TOTALS[1],
    residues::INSIDE,
    residues::SHRINK[0],
    residues::SHRINK[1],
    residues::EACH,
    residues::ZERO_SAYS,
    residues::POLE_SAYS,
    residues::RESIDUE,
    residues::TURNS[0],
    residues::TURNS[1],
    residues::TURNS[2],
    continuing::RECIPE_HERE,
    continuing::EVERYWHERE,
    continuing::SERIES_SAYS[0],
    continuing::SERIES_SAYS[1],
    continuing::AT_MOST_ONE,
    continuing::REGION,
    continuing::STEP,
    continuing::FORCED,
    continuing::NOT_ALWAYS,
    continuing::PATH_EXACT,
    continuing::ASK[0],
    continuing::DIVERGES,
    continuing::EXTENDS,
    continuing::SQRT_SAYS,
    continuing::LOG_SAYS,
    continuing::DRAWN[0],
    continuing::DRAWN[1],
    continuing::NATURAL,
    continuing::RIGHT_ONLY,
    continuing::NEW_POLES,
    gallery::CUBIC_SAYS,
    gallery::ZDP_SAYS,
    gallery::SIMPLE_ZERO,
    gallery::POLE2,
    gallery::NO_ZEROS_POLES,
    gallery::BY_HEIGHT,
    gallery::BY_RIGHT,
    gallery::SIN_ZEROS,
    gallery::SIN_REAL,
    gallery::ALTERNATE,
    gallery::INTEGRAL_RIGHT,
    gallery::NO_ZEROS,
    gallery::CHAPTER_14,
    gallery::CONVERGES,
    gallery::ONE_POLE,
    gallery::TRIVIAL,
    gallery::CRITICAL,
    gallery::SQUASHED,
    gallery::RH[0],
    gallery::RH[1],
    gallery::FOUR,
    gallery::ONE_TILE,
    gallery::NEXT,
    turning::AXES[0],
    turning::AXES[1],
    turning::ADDING,
    turning::TIMES_I,
    turning::TURNED,
    turning::DEGREE,
    turning::STRETCH_TURN[0],
    turning::STRETCH_TURN[1],
    turning::ANGLE,
    turning::NAMES[0],
    turning::NAMES[1],
    turning::ASK[0],
    turning::TOTAL[0],
    turning::TOTAL[1],
    turning::RULE,
    two_planes::TITLES[0],
    two_planes::TITLES[1],
    two_planes::GRAPH,
    two_planes::TURNS_LABEL,
    two_planes::ASK[0],
    two_planes::RAYS[0],
    two_planes::RAYS[1],
    two_planes::KEPT,
    two_planes::TRADE,
    two_planes::UNDEFINED,
    loop_sums::SHRINK,
    loop_sums::DISTANCE,
    loop_sums::CLOSED,
    loop_sums::UP,
    loop_sums::HEIGHT,
    loop_sums::STEPS_SAID,
    loop_sums::LIMIT,
    loop_sums::BIGGER[0],
    loop_sums::BIGGER[1],
    loop_sums::SURVIVES,
    loop_sums::HYPOTHESIS,
    loop_sums::EACH,
    loop_sums::CANCEL[0],
    loop_sums::CANCEL[1],
    loop_sums::GOURSAT,
    closing::DEMAND,
    closing::CAPTIONS[0][0],
    closing::CAPTIONS[0][1],
    closing::CAPTIONS[1][0],
    closing::CAPTIONS[1][1],
    closing::CAPTIONS[2][0],
    closing::CAPTIONS[2][1],
    closing::CAPTIONS[3][0],
    closing::CAPTIONS[3][1],
    closing::CAPTIONS[4][0],
    closing::CAPTIONS[4][1],
    closing::ASK_PREFIX,
    closing::ROOT,
    closing::ABOUT,
    closing::TITLE_CARD,
];

/// TeX supplements: every formula the native scenes typeset.
pub const TEX: &[&str] = &[
    the_bump::SERIES_TEX,
    the_bump::BUMP_TEX,
    one_derivative::QUOTIENT_TEX,
    one_derivative::RIGHT_TEX,
    one_derivative::UP_TEX,
    one_derivative::CR_TEX,
    r"z_0 = 1 + i",
    r"z_0^2 = 2i",
    one_point::MAP_TEX,
    one_point::DERIV_TEX,
    one_point::ABS_TEX,
    one_point::RATIO_TEX,
    one_point::SQUEEZE_TEX,
    "U",
    counting::LEASH_TEX,
    counting::COARSE_TEX,
    counting::LIP_TEX,
    real_integral::BUMP_AREA_TEX,
    real_integral::LOOP_TEX,
    real_integral::ZSQ_TEX,
    real_integral::MAX_TEX,
    real_integral::R_TEX,
    real_integral::LENGTH_TEX,
    real_integral::ARC_TEX,
    real_integral::QUARTIC_TEX,
    real_integral::ANTIDERIVATIVE_TEX[0],
    real_integral::ANTIDERIVATIVE_TEX[1],
    real_integral::RES_TEX,
    real_integral::FOURTH_TEX,
    real_integral::MINUS_P_TEX,
    real_integral::SUM_TEX,
    real_integral::VALUE_TEX,
    real_integral::QUARTIC_MAX_TEX,
    real_integral::QUARTIC_ARC_TEX,
    real_integral::P_NAMES[0],
    real_integral::P_NAMES[1],
    boundary::SPLIT_TEX,
    boundary::RESULT_TEX,
    boundary::FORMULA_TEX,
    boundary::ON_CIRCLE_TEX,
    boundary::AVERAGE_TEX,
    boundary::COEF_TEX,
    boundary::READ[0],
    boundary::READ[1],
    rigidity::IDENTITY_TEX,
    rigidity::AT_3I[0],
    rigidity::AT_3I[1],
    rigidity::SMOOTH_TEX,
    rigidity::ZERO_RECIPE_TEX,
    residues::LAURENT_TEX,
    residues::TERMS_TEX[0],
    residues::TERMS_TEX[1],
    residues::TERMS_TEX[2],
    residues::RESIDUE_TEX,
    residues::THEOREM_TEX,
    residues::NEAR_TEX,
    residues::SIMPLE_TEX,
    residues::ZERO_TEX,
    residues::POLE_TEX,
    residues::LOCAL_TEX[0],
    residues::LOCAL_TEX[1],
    residues::LOCAL_TEX[2],
    residues::LOG_TEX[0],
    residues::LOG_TEX[1],
    residues::LOG_TEX[2],
    residues::BUMP_STEPS_TEX[0],
    residues::BUMP_STEPS_TEX[1],
    continuing::GEOMETRIC_TEX,
    continuing::VALUE_TEX,
    continuing::DOUBLING_TEX,
    continuing::AT_TWO_TEX,
    continuing::NATURAL_SUM_TEX,
    continuing::ZETA_TEX,
    continuing::LACUNARY_TEX,
    continuing::GAMMA_INT_TEX,
    continuing::GAMMA_STEP_TEX,
    r"\sqrt{z}",
    gallery::TITLES_TEX[0],
    gallery::TITLES_TEX[1],
    gallery::TITLES_TEX[2],
    gallery::TITLES_TEX[3],
    gallery::TITLES_TEX[4],
    gallery::TITLES_TEX[5],
    gallery::TITLES_TEX[6],
    gallery::PERIOD_TEX,
    gallery::SIN_AT_TEX[0],
    gallery::SIN_AT_TEX[1],
    gallery::TAN_LIMIT_TEX[0],
    gallery::TAN_LIMIT_TEX[1],
    gallery::TAN_FAR_TEX[0],
    gallery::TAN_FAR_TEX[1],
    gallery::FACTORIAL_TEX,
    gallery::GAMMA_AT_TEX[0],
    gallery::GAMMA_AT_TEX[1],
    turning::WRITE_TEX,
    turning::I_I_TEX,
    turning::LENGTH_TEX,
    turning::SQUARE_TEX,
    "3 + 2i",
    "1 - i",
    "4 + i",
    "1 + i",
    "2i",
    "3",
    "1",
    "i",
    "-1",
    two_planes::FN_TEX[0],
    two_planes::FN_TEX[1],
    two_planes::SQUARE_RULE_TEX,
    two_planes::TIMES_E_TEX,
    two_planes::PERIOD_TEX,
    two_planes::NONZERO_TEX,
    two_planes::AFTER_TEX,
    loop_sums::SUM_TEX,
    loop_sums::PART_TEX[0],
    loop_sums::PART_TEX[1],
    loop_sums::PART_TEX[2],
    loop_sums::PART_TEX[3],
    loop_sums::IDENTITY_TEX,
    loop_sums::POWER_TEX[0],
    loop_sums::POWER_TEX[1],
    loop_sums::POWER_TEX[2],
    loop_sums::POWER_TEX[3],
    loop_sums::POWER_TEX[4],
    loop_sums::POWER_TEX[5],
    loop_sums::CAUCHY_TEX,
    loop_sums::GAMMA_TEX,
    loop_sums::LINEAR_TEX,
    loop_sums::EACH_TEX,
];

/// Monospace supplements: the counting chapter's code.
pub const MONO: &[&str] = &[
    counting::CODE[0],
    counting::CODE[1],
    counting::CODE[2],
    counting::CODE[3],
    counting::CODE[4],
    counting::CODE[5],
];

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_the_bump",
            make: |kit| Box::new(the_bump::TheBump { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "02_turning_numbers",
            make: |kit| Box::new(turning::Turning { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "03_two_planes",
            make: |kit| Box::new(two_planes::TwoPlanes { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "04_reading_color",
            make: |kit| Box::new(reading_color::ReadingColor { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "05_gallery",
            make: |kit| Box::new(gallery::Gallery { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "06_one_derivative",
            make: |kit| Box::new(one_derivative::OneDerivative { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "07_one_point",
            make: |kit| Box::new(one_point::OnePoint { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "08_loop_sums",
            make: |kit| Box::new(loop_sums::LoopSums { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "09_boundary",
            make: |kit| Box::new(boundary::Boundary { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "10_rigidity",
            make: |kit| Box::new(rigidity::Rigidity { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "11_counting",
            make: |kit| Box::new(counting::Counting { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "12_residues",
            make: |kit| Box::new(residues::Residues { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "13_real_integral",
            make: |kit| Box::new(real_integral::RealIntegral { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "14_continuation",
            make: |kit| Box::new(continuing::Continuing { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "15_closing",
            make: |kit| Box::new(closing::Closing { kit }),
            intensity: 0.3,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registered_key_is_a_storyboard_chapter_in_order() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let theirs: Vec<&str> = board.iter().map(|c| c.key.as_str()).collect();
        let ours: Vec<&str> = registry().iter().map(|c| c.key).collect();
        assert_eq!(ours, theirs);
    }

    #[test]
    fn no_supplement_is_empty_or_duplicated() {
        for list in [TEXT, TEX, MONO] {
            let mut seen = std::collections::BTreeSet::new();
            for s in list {
                assert!(!s.trim().is_empty());
                assert!(seen.insert(*s), "duplicate supplement {s:?}");
            }
        }
    }

    /// A window is short enough to stay a hold, is in script order, and never sits
    /// between a prediction and its answer.
    #[test]
    fn reading_windows_stay_short_and_never_delay_an_answer() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for &(id, seconds) in READING_WINDOWS {
            assert!(script.text(id).is_some(), "{id} is not a cue");
            // under 4 s of silence the score treats the gap as a hold (HOLD_MERGE) and the
            // cue sheets do not call it dead air (--max-gap)
            assert!(
                seconds >= 1.0 && seconds + explainer_kit::narration::BREATH < 4.0,
                "{id}: {seconds} s"
            );
            assert!(
                PREDICTS.iter().all(|&(asked, _)| asked != id),
                "{id}: a window after a question delays its answer"
            );
        }
        assert!(READING_WINDOWS.windows(2).all(|w| w[0].0 < w[1].0));
    }

    /// Hold before clear, read from the source because no render can run here: the card
    /// comes up, then the window, then the next cue with nothing in between, and only
    /// that cue's fade clears the card.
    #[test]
    fn each_card_outlasts_its_reading_window() {
        let sites = [
            ("04_4", "04_5", "exact", include_str!("reading_color.rs")),
            ("05_8", "05_9", "four", include_str!("gallery.rs")),
            ("07_2", "07_3", "exact", include_str!("one_point.rs")),
            ("09_5", "09_6", "needs", include_str!("boundary.rs")),
            ("10_5", "10_6", "exact", include_str!("rigidity.rs")),
            ("11_6", "11_7", "squashed", include_str!("counting.rs")),
            ("14_3", "14_4", "exact", include_str!("continuing.rs")),
        ];
        let listed: Vec<&str> = READING_WINDOWS.iter().map(|&(id, _)| id).collect();
        let sited: Vec<&str> = sites.iter().map(|site| site.0).collect();
        assert_eq!(listed, sited);
        let window = "reading_window(&kit, stage, &line)?;";
        let any_say = "let line = say(&kit, stage, ";
        for (cue, next, card, source) in sites {
            assert_eq!(cue[..2], next[..2], "{cue}: the next cue is in another chapter");
            assert_eq!(source.matches(window).count(), 1, "{cue}: one window in the file");
            let here = source.find(&format!("{any_say}\"{cue}\")?;")).unwrap();
            let then = source.find(&format!("{any_say}\"{next}\")?;")).unwrap();
            let shown = source.find(&format!("let {card} = precisely_now(")).unwrap();
            let held = source.find(window).unwrap();
            assert!(here < shown && shown < held && held < then, "{cue}: out of order");
            assert!(!source[here + 1..then].contains(any_say), "{cue}: {next} is not next");
            assert!(
                source[held + window.len()..then].trim().is_empty(),
                "{cue}: {next} waits on more"
            );
            let before = &source[shown..held];
            assert!(
                !before.contains("fade_out") && !before.contains("remove_from_scene"),
                "{cue}: cleared early"
            );
            let rest = &source[then + 1..];
            let cleared_by = &rest[..rest.find(any_say).unwrap_or(rest.len())];
            assert!(cleared_by.contains("fade_out_all"), "{cue}: {next} clears nothing");
            assert!(
                cleared_by
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .any(|w| w == card),
                "{cue}: {next} does not clear {card}"
            );
        }
    }
}
