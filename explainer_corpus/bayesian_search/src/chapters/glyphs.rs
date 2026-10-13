//! Every fixed on-screen string of Video H, for the glyph probe (`search
//! glyphs`): the chapters build their text from these constants, and a
//! string that carries a number is built from its fixed parts here and a
//! value from `facts` (the probe covers the digits through [`DIGITS`]).

/// The digits and separators numbers are printed with.
pub const DIGITS: &str = "0123456789 , . % + − × ÷ = ( ) ½ ⅓ ⅔";

pub const TITLE: &str = "How to Find a Lost Submarine";

/// The code panel, as the storyboard pins it: the update (07_2), then the
/// choice (08_3). `p` is the map, `d` each cell's chance of being seen, `i`
/// the cell just searched. ASCII only: CM Typewriter has no × or −.
pub const CODE: [&str; 4] = [
    "p[i] *= 1 - d[i]                       # the search of cell i came back empty",
    "t = sum(p); p = [x / t for x in p]     # the map adds up to one again",
    "find = [x * y for x, y in zip(p, d)]   # chance of a find, cell by cell",
    "i = find.index(max(find))              # search there next",
];

/// Plain text (IBM Plex Sans).
pub const TEXT: &[&str] = &[DIGITS, TITLE];
/// Mathematics (Computer Modern, native TeX).
pub const TEX: &[&str] = &[];
/// Code, set in CM Typewriter.
pub const MONO: &[&str] = &CODE;
