//! The fixed on-screen strings the glyph probe checks beyond what the
//! chapters record themselves (`squares glyphs`): the chapters build their
//! text from constants and from `facts`, and the probe covers the digits and
//! signs numbers are printed with through [`DIGITS`], and the thumbnail's
//! words (the probe renders chapters, not the thumbnail).

/// The digits and signs numbers and labels are printed with.
pub const DIGITS: &str = "0123456789 . , : ? ° ö ×";

pub const TITLE: &str = "Why Turning a Square Can Make the Whole Box Smaller";

/// Plain text (Computer Modern roman, the kit's default face).
pub const TEXT: &[&str] = &[
    DIGITS,
    TITLE,
    super::thumbnail::HOOK[0],
    super::thumbnail::HOOK[1],
    super::thumbnail::THREE,
];
/// Mathematics (Computer Modern, native TeX).
pub const TEX: &[&str] = &[
    super::the_turned_tile::GAP_TIMES,
    super::the_turned_tile::GAP_IS,
    super::the_turned_tile::ONE_OVER_ROOT,
    super::four_pins::MARK_LO,
    super::four_pins::MARK_HI,
    super::four_pins::UNDER_ONE,
    super::a_turn_by_a_hair::HALF,
    super::a_turn_by_a_hair::FORMULA,
    super::the_corner_lemma::AT_MOST,
    super::the_corner_lemma::BOUND,
    super::near_a_side::D_TEX,
    super::near_a_side::THETA,
    super::near_a_side::D_TAN,
    super::near_a_side::D_COT,
    super::near_a_side::CUT_IS,
    super::near_a_side::BEATS,
    super::near_a_side::SLIP,
    super::near_a_side::IDENTITY,
    super::four_points_five_tiles::LAMBDA,
    super::four_points_five_tiles::DEPTH,
    super::four_points_five_tiles::B_IS,
    super::four_points_five_tiles::NO_GROWTH,
    super::what_is_proved::TEN_SIDE,
    super::what_is_proved::ELEVEN_SIDE,
    super::what_is_proved::ELEVEN_TURN,
    super::thumbnail::SIDE,
];
/// Code, set in CM Typewriter.
pub const MONO: &[&str] = &[];
