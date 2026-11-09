//! The §6.2 constants — kept EXACTLY as the Reference defines them.
//!
//! Values mirror `manimlib/constants.py` over `default_config.yml` at the
//! pinned commit, locked by `fixtures/constants.txt` (see `tests/parity.rs`).
//! These are the shared language of every manim scene; nothing here is a
//! tuning knob.

use crate::color::Srgb;
use crate::types::Vec3;

// --- Frame geometry -------------------------------------------------------

/// Default output width in pixels (1080p).
pub const DEFAULT_PIXEL_WIDTH: u32 = 1920;
/// Default output height in pixels.
pub const DEFAULT_PIXEL_HEIGHT: u32 = 1080;
/// Width over height of the default frame.
pub const ASPECT_RATIO: f64 = DEFAULT_PIXEL_WIDTH as f64 / DEFAULT_PIXEL_HEIGHT as f64;
/// The scene coordinate system is FRAME_HEIGHT units tall, always.
pub const FRAME_HEIGHT: f64 = 8.0;
/// Scene width in scene units (aspect-dependent).
pub const FRAME_WIDTH: f64 = FRAME_HEIGHT * ASPECT_RATIO;
/// Half the frame height.
pub const FRAME_Y_RADIUS: f64 = FRAME_HEIGHT / 2.0;
/// Half the frame width.
pub const FRAME_X_RADIUS: f64 = FRAME_WIDTH / 2.0;

// --- Buffs ----------------------------------------------------------------

/// Small spacing nudge (0.1 scene units).
pub const SMALL_BUFF: f64 = 0.1;
/// Medium-small spacing (0.25 scene units).
pub const MED_SMALL_BUFF: f64 = 0.25;
/// Medium-large spacing (0.5 scene units).
pub const MED_LARGE_BUFF: f64 = 0.5;
/// Large spacing (1.0 scene units).
pub const LARGE_BUFF: f64 = 1.0;
/// Default buffer for `Mobject.to_edge`.
pub const DEFAULT_MOBJECT_TO_EDGE_BUFF: f64 = 0.5;
/// Default buffer for `Mobject.next_to`.
pub const DEFAULT_MOBJECT_TO_MOBJECT_BUFF: f64 = 0.25;

// --- Directions -----------------------------------------------------------

/// The scene origin.
pub const ORIGIN: Vec3 = [0.0, 0.0, 0.0];
/// Unit vector up (+y).
pub const UP: Vec3 = [0.0, 1.0, 0.0];
/// Unit vector down (-y).
pub const DOWN: Vec3 = [0.0, -1.0, 0.0];
/// Unit vector right (+x).
pub const RIGHT: Vec3 = [1.0, 0.0, 0.0];
/// Unit vector left (-x).
pub const LEFT: Vec3 = [-1.0, 0.0, 0.0];
/// Into the screen (-z).
pub const IN: Vec3 = [0.0, 0.0, -1.0];
/// Out of the screen (+z).
pub const OUT: Vec3 = [0.0, 0.0, 1.0];
/// The +x axis.
pub const X_AXIS: Vec3 = [1.0, 0.0, 0.0];
/// The +y axis.
pub const Y_AXIS: Vec3 = [0.0, 1.0, 0.0];
/// The +z axis.
pub const Z_AXIS: Vec3 = [0.0, 0.0, 1.0];

/// Up-left diagonal (UP + LEFT).
pub const UL: Vec3 = [-1.0, 1.0, 0.0];
/// Up-right diagonal (UP + RIGHT).
pub const UR: Vec3 = [1.0, 1.0, 0.0];
/// Down-left diagonal (DOWN + LEFT).
pub const DL: Vec3 = [-1.0, -1.0, 0.0];
/// Down-right diagonal (DOWN + RIGHT).
pub const DR: Vec3 = [1.0, -1.0, 0.0];

/// Center of the top frame edge.
pub const TOP: Vec3 = [0.0, FRAME_Y_RADIUS, 0.0];
/// Center of the bottom frame edge.
pub const BOTTOM: Vec3 = [0.0, -FRAME_Y_RADIUS, 0.0];
/// Center of the left frame edge.
pub const LEFT_SIDE: Vec3 = [-FRAME_X_RADIUS, 0.0, 0.0];
/// Center of the right frame edge.
pub const RIGHT_SIDE: Vec3 = [FRAME_X_RADIUS, 0.0, 0.0];

// --- Angles ---------------------------------------------------------------

/// π.
pub const PI: f64 = std::f64::consts::PI;
/// 2π.
pub const TAU: f64 = 2.0 * PI;
/// One degree in radians; write `30.0 * DEG`.
pub const DEG: f64 = TAU / 360.0;
/// One radian, for readability beside `30.0 * DEG`.
pub const RADIANS: f64 = 1.0;

// --- Strokes, timing, resolutions ----------------------------------------

/// Default VMobject stroke width (in stroke units, not scene units).
pub const DEFAULT_STROKE_WIDTH: f64 = 4.0;
/// Stroke units → scene units (the Reference's stroke shader constant).
pub const STROKE_WIDTH_CONVERSION: f64 = 0.01;
/// Default frames per second.
pub const DEFAULT_FPS: u32 = 30;

/// `-l` resolution (854 x 480).
pub const RESOLUTION_LOW: (u32, u32) = (854, 480);
/// `-m` resolution (1280 x 720).
pub const RESOLUTION_MED: (u32, u32) = (1280, 720);
/// `--hd` resolution (1920 x 1080).
pub const RESOLUTION_HIGH: (u32, u32) = (1920, 1080);
/// `--uhd` resolution (3840 x 2160).
pub const RESOLUTION_4K: (u32, u32) = (3840, 2160);

// --- The manim palette ----------------------------------------------------

/// Manim's blue, E (darkest) shade.
pub const BLUE_E: Srgb = Srgb::from_rgb8(0x1C, 0x75, 0x8A); // #1C758A
/// Manim's blue, D shade.
pub const BLUE_D: Srgb = Srgb::from_rgb8(0x29, 0xAB, 0xCA); // #29ABCA
/// Manim's blue, C (median) shade.
pub const BLUE_C: Srgb = Srgb::from_rgb8(0x58, 0xC4, 0xDD); // #58C4DD
/// Manim's blue, B shade.
pub const BLUE_B: Srgb = Srgb::from_rgb8(0x9C, 0xDC, 0xEB); // #9CDCEB
/// Manim's blue, A (lightest) shade.
pub const BLUE_A: Srgb = Srgb::from_rgb8(0xC7, 0xE9, 0xF1); // #C7E9F1
/// Manim's teal, E (darkest) shade.
pub const TEAL_E: Srgb = Srgb::from_rgb8(0x49, 0xA8, 0x8F); // #49A88F
/// Manim's teal, D shade.
pub const TEAL_D: Srgb = Srgb::from_rgb8(0x55, 0xC1, 0xA7); // #55C1A7
/// Manim's teal, C (median) shade.
pub const TEAL_C: Srgb = Srgb::from_rgb8(0x5C, 0xD0, 0xB3); // #5CD0B3
/// Manim's teal, B shade.
pub const TEAL_B: Srgb = Srgb::from_rgb8(0x76, 0xDD, 0xC0); // #76DDC0
/// Manim's teal, A (lightest) shade.
pub const TEAL_A: Srgb = Srgb::from_rgb8(0xAC, 0xEA, 0xD7); // #ACEAD7
/// Manim's green, E (darkest) shade.
pub const GREEN_E: Srgb = Srgb::from_rgb8(0x69, 0x9C, 0x52); // #699C52
/// Manim's green, D shade.
pub const GREEN_D: Srgb = Srgb::from_rgb8(0x77, 0xB0, 0x5D); // #77B05D
/// Manim's green, C (median) shade.
pub const GREEN_C: Srgb = Srgb::from_rgb8(0x83, 0xC1, 0x67); // #83C167
/// Manim's green, B shade.
pub const GREEN_B: Srgb = Srgb::from_rgb8(0xA6, 0xCF, 0x8C); // #A6CF8C
/// Manim's green, A (lightest) shade.
pub const GREEN_A: Srgb = Srgb::from_rgb8(0xC9, 0xE2, 0xAE); // #C9E2AE
/// Manim's yellow, E (darkest) shade.
pub const YELLOW_E: Srgb = Srgb::from_rgb8(0xE8, 0xC1, 0x1C); // #E8C11C
/// Manim's yellow, D shade.
pub const YELLOW_D: Srgb = Srgb::from_rgb8(0xF4, 0xD3, 0x45); // #F4D345
/// Manim's yellow, C (median) shade.
pub const YELLOW_C: Srgb = Srgb::from_rgb8(0xFF, 0xFF, 0x00); // #FFFF00
/// Manim's yellow, B shade.
pub const YELLOW_B: Srgb = Srgb::from_rgb8(0xFF, 0xEA, 0x94); // #FFEA94
/// Manim's yellow, A (lightest) shade.
pub const YELLOW_A: Srgb = Srgb::from_rgb8(0xFF, 0xF1, 0xB6); // #FFF1B6
/// Manim's gold, E (darkest) shade.
pub const GOLD_E: Srgb = Srgb::from_rgb8(0xC7, 0x8D, 0x46); // #C78D46
/// Manim's gold, D shade.
pub const GOLD_D: Srgb = Srgb::from_rgb8(0xE1, 0xA1, 0x58); // #E1A158
/// Manim's gold, C (median) shade.
pub const GOLD_C: Srgb = Srgb::from_rgb8(0xF0, 0xAC, 0x5F); // #F0AC5F
/// Manim's gold, B shade.
pub const GOLD_B: Srgb = Srgb::from_rgb8(0xF9, 0xB7, 0x75); // #F9B775
/// Manim's gold, A (lightest) shade.
pub const GOLD_A: Srgb = Srgb::from_rgb8(0xF7, 0xC7, 0x97); // #F7C797
/// Manim's red, E (darkest) shade.
pub const RED_E: Srgb = Srgb::from_rgb8(0xCF, 0x50, 0x44); // #CF5044
/// Manim's red, D shade.
pub const RED_D: Srgb = Srgb::from_rgb8(0xE6, 0x5A, 0x4C); // #E65A4C
/// Manim's red, C (median) shade.
pub const RED_C: Srgb = Srgb::from_rgb8(0xFC, 0x62, 0x55); // #FC6255
/// Manim's red, B shade.
pub const RED_B: Srgb = Srgb::from_rgb8(0xFF, 0x80, 0x80); // #FF8080
/// Manim's red, A (lightest) shade.
pub const RED_A: Srgb = Srgb::from_rgb8(0xF7, 0xA1, 0xA3); // #F7A1A3
/// Manim's maroon, E (darkest) shade.
pub const MAROON_E: Srgb = Srgb::from_rgb8(0x94, 0x42, 0x4F); // #94424F
/// Manim's maroon, D shade.
pub const MAROON_D: Srgb = Srgb::from_rgb8(0xA2, 0x4D, 0x61); // #A24D61
/// Manim's maroon, C (median) shade.
pub const MAROON_C: Srgb = Srgb::from_rgb8(0xC5, 0x5F, 0x73); // #C55F73
/// Manim's maroon, B shade.
pub const MAROON_B: Srgb = Srgb::from_rgb8(0xEC, 0x92, 0xAB); // #EC92AB
/// Manim's maroon, A (lightest) shade.
pub const MAROON_A: Srgb = Srgb::from_rgb8(0xEC, 0xAB, 0xC1); // #ECABC1
/// Manim's purple, E (darkest) shade.
pub const PURPLE_E: Srgb = Srgb::from_rgb8(0x64, 0x41, 0x72); // #644172
/// Manim's purple, D shade.
pub const PURPLE_D: Srgb = Srgb::from_rgb8(0x71, 0x55, 0x82); // #715582
/// Manim's purple, C (median) shade.
pub const PURPLE_C: Srgb = Srgb::from_rgb8(0x9A, 0x72, 0xAC); // #9A72AC
/// Manim's purple, B shade.
pub const PURPLE_B: Srgb = Srgb::from_rgb8(0xB1, 0x89, 0xC6); // #B189C6
/// Manim's purple, A (lightest) shade.
pub const PURPLE_A: Srgb = Srgb::from_rgb8(0xCA, 0xA3, 0xE8); // #CAA3E8
/// Manim's grey, E (darkest) shade.
pub const GREY_E: Srgb = Srgb::from_rgb8(0x22, 0x22, 0x22); // #222222
/// Manim's grey, D shade.
pub const GREY_D: Srgb = Srgb::from_rgb8(0x44, 0x44, 0x44); // #444444
/// Manim's grey, C (median) shade.
pub const GREY_C: Srgb = Srgb::from_rgb8(0x88, 0x88, 0x88); // #888888
/// Manim's grey, B shade.
pub const GREY_B: Srgb = Srgb::from_rgb8(0xBB, 0xBB, 0xBB); // #BBBBBB
/// Manim's grey, A (lightest) shade.
pub const GREY_A: Srgb = Srgb::from_rgb8(0xDD, 0xDD, 0xDD); // #DDDDDD
/// White.
pub const WHITE: Srgb = Srgb::from_rgb8(0xFF, 0xFF, 0xFF); // #FFFFFF
/// Black.
pub const BLACK: Srgb = Srgb::from_rgb8(0x00, 0x00, 0x00); // #000000
/// Manim's grey-brown.
pub const GREY_BROWN: Srgb = Srgb::from_rgb8(0x73, 0x63, 0x57); // #736357
/// Manim's dark brown.
pub const DARK_BROWN: Srgb = Srgb::from_rgb8(0x8B, 0x45, 0x13); // #8B4513
/// Manim's light brown.
pub const LIGHT_BROWN: Srgb = Srgb::from_rgb8(0xCD, 0x85, 0x3F); // #CD853F
/// Manim's pink.
pub const PINK: Srgb = Srgb::from_rgb8(0xD1, 0x47, 0xBD); // #D147BD
/// Manim's light pink.
pub const LIGHT_PINK: Srgb = Srgb::from_rgb8(0xDC, 0x75, 0xCD); // #DC75CD
/// Chroma-key green.
pub const GREEN_SCREEN: Srgb = Srgb::from_rgb8(0x00, 0xFF, 0x00); // #00FF00
/// Manim's orange.
pub const ORANGE: Srgb = Srgb::from_rgb8(0xFF, 0x86, 0x2F); // #FF862F
/// Pure sRGB red.
pub const PURE_RED: Srgb = Srgb::from_rgb8(0xFF, 0x00, 0x00); // #FF0000
/// Pure sRGB green.
pub const PURE_GREEN: Srgb = Srgb::from_rgb8(0x00, 0xFF, 0x00); // #00FF00
/// Pure sRGB blue.
pub const PURE_BLUE: Srgb = Srgb::from_rgb8(0x00, 0x00, 0xFF); // #0000FF

// Abbreviated names for the "median" colors, as constants.py binds them.
/// Alias for [`BLUE_C`].
pub const BLUE: Srgb = BLUE_C;
/// Alias for [`TEAL_C`].
pub const TEAL: Srgb = TEAL_C;
/// Alias for [`GREEN_C`].
pub const GREEN: Srgb = GREEN_C;
/// Alias for [`YELLOW_C`].
pub const YELLOW: Srgb = YELLOW_C;
/// Alias for [`GOLD_C`].
pub const GOLD: Srgb = GOLD_C;
/// Alias for [`RED_C`].
pub const RED: Srgb = RED_C;
/// Alias for [`MAROON_C`].
pub const MAROON: Srgb = MAROON_C;
/// Alias for [`PURPLE_C`].
pub const PURPLE: Srgb = PURPLE_C;
/// Alias for [`GREY_C`].
pub const GREY: Srgb = GREY_C;

/// The 3b1b colormap anchors: `[BLUE_E, GREEN, YELLOW, RED]`.
pub const COLORMAP_3B1B: [Srgb; 4] = [BLUE_E, GREEN, YELLOW, RED];

/// Default scene background (`#333333`).
pub const DEFAULT_BACKGROUND_COLOR: Srgb = Srgb::from_rgb8(0x33, 0x33, 0x33); // #333333
/// Default mobject color (text, tex, lines): WHITE.
pub const DEFAULT_MOBJECT_COLOR: Srgb = WHITE;
/// Default light color (axes, arrows, annuli): GREY_B.
pub const DEFAULT_LIGHT_COLOR: Srgb = GREY_B;
/// Default VMobject stroke color: GREY_A.
pub const DEFAULT_VMOBJECT_STROKE_COLOR: Srgb = GREY_A;
/// Default VMobject fill color: GREY_C.
pub const DEFAULT_VMOBJECT_FILL_COLOR: Srgb = GREY_C;
