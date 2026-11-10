//! The font book: the bundled sovereign faces plus user faces, under the
//! D-08 policy — the bundled default renders identically on every machine,
//! user TTFs load by path, family-name lookup is a convenience that
//! **never** silently substitutes (a missing family is a named
//! [`TextError::FontUnavailable`]).
//!
//! Per-character coverage is a separate question from family selection: a
//! character the selected face has no glyph for (U+2212 MINUS SIGN in
//! Computer Modern, say) is drawn from the fixed, bundled
//! [`FontBook::glyph_fallback`] chain — Computer Modern, then the Noto Sans
//! Math symbol face, then IBM Plex Sans — and the shaped glyph records the
//! face it actually came from. Only a character no bundled face covers is a
//! [`TextError::UnmappedChar`]. The chain is compiled in, so the result is
//! still identical on every machine.

use crate::error::TextError;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, PoisonError};

/// Maximum retained glyphs per loaded face. The independent command budget
/// also bounds fonts whose individual glyphs have unusually large outlines.
pub const GLYPH_CACHE_MAX_ENTRIES: usize = 512;
/// Maximum retained outline commands per loaded face.
pub const GLYPH_CACHE_MAX_COMMANDS: usize = 32_768;

/// Work and resident storage in a face's outline cache. These are diagnostics,
/// never scene state: eviction and scheduling cannot change geometry or spans.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlyphCacheStats {
    /// Requests served without decoding the font's outline tables.
    pub hits: u64,
    /// Outline decode attempts (failed and deliberately uncacheable included).
    pub decodes: u64,
    /// Currently retained glyphs.
    pub entries: usize,
    /// Currently retained outline commands.
    pub commands: usize,
}

// Cache font-unit commands, not positioned paths. In particular, the midpoint
// of a line must still be computed *after* placement by QuadPath::add_line_to,
// exactly as on the uncached path. Translating a cached positioned QuadPath
// would change floating-point association and break certified output.
#[derive(Clone, Copy, Debug)]
pub(crate) enum OutlineCommand {
    Move([f64; 2]),
    Line([f64; 2]),
    Quad([f64; 2], [f64; 2]),
}

#[derive(Default)]
struct GlyphCache {
    glyphs: BTreeMap<u16, Arc<[OutlineCommand]>>,
    order: VecDeque<u16>,
    stats: GlyphCacheStats,
}

impl GlyphCache {
    fn insert(&mut self, gid: u16, commands: Arc<[OutlineCommand]>) {
        if commands.len() > GLYPH_CACHE_MAX_COMMANDS {
            return;
        }
        while self.glyphs.len() >= GLYPH_CACHE_MAX_ENTRIES
            || self.stats.commands + commands.len() > GLYPH_CACHE_MAX_COMMANDS
        {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if let Some(old) = self.glyphs.remove(&oldest) {
                self.stats.commands -= old.len();
            }
        }
        self.stats.commands += commands.len();
        self.glyphs.insert(gid, commands);
        self.order.push_back(gid);
        self.stats.entries = self.glyphs.len();
    }
}

/// A face variant within a family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FaceKey {
    /// Bold?
    pub bold: bool,
    /// Italic?
    pub italic: bool,
}

/// One face: the parsed font plus its GPOS kerning, parsed once (CM
/// Unicode kerns through GPOS; the legacy `kern` table is consulted too).
pub struct Face {
    /// The parsed font.
    pub font: fmd_font::Font,
    gpos: fmd_font::Kerning,
    // Identity is the actual loaded face, not a family alias, requested
    // variant, character, or gid shared across different fonts. FontBook only
    // publishes immutable face references; adding a family gets a new cache.
    outlines: Mutex<GlyphCache>,
}

impl Face {
    fn new(font: fmd_font::Font) -> Self {
        let gpos = font.gpos_kerning();
        Self {
            font,
            gpos,
            outlines: Mutex::new(GlyphCache::default()),
        }
    }

    /// Diagnostic reuse counters for this actual face (aliases and variant
    /// fallbacks resolve to the same cache).
    #[must_use]
    pub fn glyph_cache_stats(&self) -> GlyphCacheStats {
        self.outlines
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .stats
    }

    pub(crate) fn glyph_commands(
        &self,
        gid: u16,
        ch: char,
    ) -> Result<Arc<[OutlineCommand]>, TextError> {
        let mut cache = self.outlines.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(hit) = cache.glyphs.get(&gid).cloned() {
            cache.stats.hits = cache.stats.hits.saturating_add(1);
            return Ok(hit);
        }
        // Keep the lock through this face's decode: concurrent requests for a
        // cold glyph decode once. Other faces remain independently usable.
        cache.stats.decodes = cache.stats.decodes.saturating_add(1);
        let outline = self
            .font
            .glyph_outline(gid)
            .map_err(|error| TextError::Outline {
                ch,
                what: format!("{error:?}"),
            })?;
        let mut commands = Vec::new();
        for contour in &outline.contours {
            commands.push(OutlineCommand::Move([contour.start.x, contour.start.y]));
            for segment in &contour.segments {
                commands.push(match segment {
                    fmd_font::outline::Segment::Line { to } => OutlineCommand::Line([to.x, to.y]),
                    fmd_font::outline::Segment::Quad { ctrl, to } => {
                        OutlineCommand::Quad([ctrl.x, ctrl.y], [to.x, to.y])
                    }
                });
            }
        }
        let commands: Arc<[OutlineCommand]> = commands.into();
        cache.insert(gid, Arc::clone(&commands));
        Ok(commands)
    }

    /// Kerning between two glyphs, font units: legacy `kern` table plus
    /// the GPOS `kern` feature.
    #[must_use]
    pub fn kern_units(&self, left: u16, right: u16) -> i32 {
        i32::from(self.font.kerning_between_glyphs(left, right))
            + i32::from(self.gpos.pair(left, right))
    }
}

/// One loaded family: up to four variant faces (regular required).
pub struct Family {
    /// Canonical family name.
    pub name: String,
    regular: Face,
    bold: Option<Face>,
    italic: Option<Face>,
    bold_italic: Option<Face>,
}

impl Family {
    /// The face for a variant, falling back to regular within the family
    /// (a bold request on a single-face family renders regular — the
    /// family the user named is still the family used).
    #[must_use]
    pub fn face(&self, key: FaceKey) -> &Face {
        match (key.bold, key.italic) {
            (true, true) => self
                .bold_italic
                .as_ref()
                .or(self.bold.as_ref())
                .or(self.italic.as_ref())
                .unwrap_or(&self.regular),
            (true, false) => self.bold.as_ref().unwrap_or(&self.regular),
            (false, true) => self.italic.as_ref().unwrap_or(&self.regular),
            (false, false) => &self.regular,
        }
    }
}

/// The book: families by canonical name, with the bundled set as the
/// sovereign default.
pub struct FontBook {
    families: Vec<Family>,
    /// Bundled glyph-coverage faces that are deliberately not selectable by
    /// name (the mathematics symbol face); reachable only through
    /// [`FontBook::glyph_fallback`] and [`FontBook::resolve_face`].
    coverage: Vec<Family>,
    /// Index of the default family.
    default_ix: usize,
    /// Index of the monospace family (`<tt>`).
    mono_ix: usize,
    /// Index of the bundled sans family (the last-resort coverage face).
    sans_ix: usize,
}

/// The bundled default family's canonical name.
pub const DEFAULT_FAMILY: &str = "Computer Modern";
/// The bundled monospace family's canonical name.
pub const MONO_FAMILY: &str = "CM Typewriter";
/// The bundled sans family's canonical name.
pub const SANS_FAMILY: &str = "IBM Plex Sans";
/// The bundled mathematics symbol-fallback family's canonical name.
pub const MATH_FAMILY: &str = "Noto Sans Math";
const SELECTABLE_BUNDLED_TEXT_FAMILIES: [&str; 3] = [DEFAULT_FAMILY, MONO_FAMILY, SANS_FAMILY];
/// Every font family compiled into the sovereign Scribe bundle.
///
/// The first three families are selectable by [`FontBook`]. The fourth is
/// the curated symbol fallback consumed by the mathematics engine.
pub const BUNDLED_FONT_FAMILIES: [&str; 4] =
    [DEFAULT_FAMILY, MONO_FAMILY, SANS_FAMILY, MATH_FAMILY];

/// Verify the compiled font bundle and return its canonical family roster.
///
/// This is the runtime source of truth for capability reporting: it parses
/// all selectable text faces through [`FontBook::bundled`] and separately
/// verifies the mathematics symbol fallback, which is deliberately not a
/// general-purpose selectable text family.
///
/// # Errors
///
/// [`TextError::FontParse`] if any compiled face is corrupt.
pub fn bundled_font_inventory() -> Result<Vec<String>, TextError> {
    let _book = FontBook::bundled()?;
    fmd_font::Font::parse(fmd_font::bundled::NOTO_SANS_MATH_SYMBOLS.to_vec()).map_err(|error| {
        TextError::FontParse {
            path: "bundled:noto-sans-math-symbols".to_owned(),
            what: error.to_string(),
        }
    })?;
    Ok(BUNDLED_FONT_FAMILIES
        .iter()
        .map(|family| (*family).to_owned())
        .collect())
}

/// Every face compiled into the binary, by bundle name, with its exact
/// bytes (text faces and the mathematics fallback alike): the font input a
/// certified manifest's C6 records.
#[must_use]
pub fn bundled_faces() -> &'static [(&'static str, &'static [u8])] {
    &fmd_font::bundled::ALL_FACES
}

/// Whether a configured text-family name resolves inside the compiled bundle.
///
/// Selectable text-family aliases are accepted exactly as [`FontBook::family`]
/// accepts them. The mathematics fallback is intentionally not selectable as
/// a general-purpose text family.
#[must_use]
pub fn is_bundled_text_family(name: &str) -> bool {
    let want = normalize(name);
    SELECTABLE_BUNDLED_TEXT_FAMILIES.iter().any(|canonical| {
        normalize(canonical) == want
            || aliases(canonical)
                .iter()
                .any(|alias| normalize(alias) == want)
    })
}

impl FontBook {
    /// The bundled sovereign book: Computer Modern (four variants), CM
    /// Typewriter, IBM Plex Sans (regular + bold + italics).
    ///
    /// # Errors
    ///
    /// Propagates a bundled-face parse failure (build corruption, not a
    /// runtime condition).
    pub fn bundled() -> Result<Self, TextError> {
        let parse = |bytes: &[u8], what: &str| {
            fmd_font::Font::parse(bytes.to_vec())
                .map(Face::new)
                .map_err(|e| TextError::FontParse {
                    path: format!("bundled:{what}"),
                    what: e.to_string(),
                })
        };
        let cm = Family {
            name: DEFAULT_FAMILY.to_owned(),
            regular: parse(fmd_font::bundled::CM_REGULAR, "cm-regular")?,
            bold: Some(parse(fmd_font::bundled::CM_BOLD, "cm-bold")?),
            italic: Some(parse(fmd_font::bundled::CM_ITALIC, "cm-italic")?),
            bold_italic: Some(parse(fmd_font::bundled::CM_BOLD_ITALIC, "cm-bold-italic")?),
        };
        let tt = Family {
            name: MONO_FAMILY.to_owned(),
            regular: parse(fmd_font::bundled::CM_TYPEWRITER, "cm-typewriter")?,
            bold: None,
            italic: None,
            bold_italic: None,
        };
        let sans = Family {
            name: SANS_FAMILY.to_owned(),
            regular: parse(fmd_font::bundled::PLEX_REGULAR, "plex-regular")?,
            bold: Some(parse(fmd_font::bundled::PLEX_BOLD, "plex-bold")?),
            italic: Some(parse(fmd_font::bundled::PLEX_ITALIC, "plex-italic")?),
            bold_italic: Some(parse(
                fmd_font::bundled::PLEX_BOLD_ITALIC,
                "plex-bold-italic",
            )?),
        };
        let math = Family {
            name: MATH_FAMILY.to_owned(),
            regular: parse(
                fmd_font::bundled::NOTO_SANS_MATH_SYMBOLS,
                "noto-sans-math-symbols",
            )?,
            bold: None,
            italic: None,
            bold_italic: None,
        };
        Ok(Self {
            families: vec![cm, tt, sans],
            coverage: vec![math],
            default_ix: 0,
            mono_ix: 1,
            sans_ix: 2,
        })
    }

    /// Add a user family from font bytes (the caller reads the file; the
    /// engine owns no filesystem access — capability doctrine). The name
    /// is the family name the user will select by.
    ///
    /// # Errors
    ///
    /// [`TextError::FontFamilyConflict`] when lookup would resolve the name to
    /// an existing canonical family or alias.
    /// [`TextError::FontParse`] when the bytes are not a usable font.
    pub fn add_family(&mut self, name: &str, bytes: Vec<u8>) -> Result<(), TextError> {
        let want = normalize(name);
        if let Some(existing) = self
            .families
            .iter()
            .chain(&self.coverage)
            .find(|family| matches_normalized_name(family, &want))
        {
            return Err(TextError::FontFamilyConflict {
                requested: name.to_owned(),
                existing: existing.name.clone(),
            });
        }
        let font = fmd_font::Font::parse(bytes).map_err(|e| TextError::FontParse {
            path: name.to_owned(),
            what: e.to_string(),
        })?;
        self.families.push(Family {
            name: name.to_owned(),
            regular: Face::new(font),
            bold: None,
            italic: None,
            bold_italic: None,
        });
        Ok(())
    }

    /// The default family.
    #[must_use]
    pub fn default_family(&self) -> &Family {
        &self.families[self.default_ix]
    }

    /// The monospace family (`<tt>`).
    #[must_use]
    pub fn mono_family(&self) -> &Family {
        &self.families[self.mono_ix]
    }

    /// Look a family up by name, case-insensitively, tolerating the
    /// common aliases of the bundled names. A miss is the named
    /// capability-style error — never a substitution.
    ///
    /// # Errors
    ///
    /// [`TextError::FontUnavailable`] naming the request and the roster.
    pub fn family(&self, name: &str) -> Result<&Family, TextError> {
        let want = normalize(name);
        let hit = self
            .families
            .iter()
            .find(|family| matches_normalized_name(family, &want));
        hit.ok_or_else(|| TextError::FontUnavailable {
            family: name.to_owned(),
            available: self.families.iter().map(|f| f.name.clone()).collect(),
        })
    }

    /// The face a shaped glyph's [`FaceSel`](crate::shape::FaceSel) names:
    /// a selectable family (aliases tolerated, as [`FontBook::family`]) or
    /// one of the bundled coverage faces [`FontBook::glyph_fallback`] can
    /// hand out. Use this, not [`FontBook::family`], to turn a laid-out
    /// glyph back into its outline and metrics.
    ///
    /// # Errors
    ///
    /// [`TextError::FontUnavailable`] when neither resolves.
    pub fn resolve_face(&self, family: &str, key: FaceKey) -> Result<&Face, TextError> {
        if let Some(coverage) = self.coverage.iter().find(|f| f.name == family) {
            return Ok(coverage.face(key));
        }
        Ok(self.family(family)?.face(key))
    }

    /// The bundled per-character fallback for a character `primary` (the
    /// selected family's canonical name) has no glyph for: the first face in
    /// the fixed chain Computer Modern → Noto Sans Math (symbols) → IBM Plex
    /// Sans, skipping `primary` itself, whose cmap maps `ch`. Returns the
    /// covering family's canonical name and the glyph id in its `key`
    /// variant (the math face has only a regular variant). `None` means no
    /// bundled face covers `ch`.
    #[must_use]
    pub fn glyph_fallback(&self, primary: &str, key: FaceKey, ch: char) -> Option<(&str, u16)> {
        let chain = [
            &self.families[self.default_ix],
            &self.coverage[0],
            &self.families[self.sans_ix],
        ];
        chain
            .into_iter()
            .filter(|family| family.name != primary)
            .find_map(|family| {
                let gid = family.face(key).font.glyph_index(ch);
                (gid != 0).then_some((family.name.as_str(), gid))
            })
    }

    /// The family names the book can serve.
    #[must_use]
    pub fn available(&self) -> Vec<String> {
        self.families.iter().map(|f| f.name.clone()).collect()
    }
}

fn normalize(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

fn matches_normalized_name(family: &Family, want: &str) -> bool {
    normalize(&family.name) == want
        || aliases(&family.name)
            .iter()
            .any(|alias| normalize(alias) == want)
}

fn aliases(canonical: &str) -> &'static [&'static str] {
    match canonical {
        DEFAULT_FAMILY => &["CMU Serif", "Computer Modern Roman", "cmr"],
        MONO_FAMILY => &["Computer Modern Typewriter", "CMU Typewriter Text"],
        SANS_FAMILY => &["Plex Sans", "IBM Plex"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BUNDLED_FONT_FAMILIES, GLYPH_CACHE_MAX_COMMANDS, GLYPH_CACHE_MAX_ENTRIES, GlyphCache,
        OutlineCommand, bundled_font_inventory, is_bundled_text_family,
    };
    use std::sync::Arc;

    #[test]
    fn glyph_cache_evicts_by_both_budgets_and_keeps_borrowed_outlines_alive() {
        let mut cache = GlyphCache::default();
        let small: Arc<[OutlineCommand]> = vec![OutlineCommand::Move([0.0, 0.0])].into();
        for gid in 0..=GLYPH_CACHE_MAX_ENTRIES {
            cache.insert(u16::try_from(gid).unwrap(), Arc::clone(&small));
        }
        assert_eq!(cache.stats.entries, GLYPH_CACHE_MAX_ENTRIES);
        assert_eq!(cache.stats.commands, GLYPH_CACHE_MAX_ENTRIES);
        assert!(!cache.glyphs.contains_key(&0));
        let retained = Arc::clone(cache.glyphs.get(&1).unwrap());
        let large: Arc<[OutlineCommand]> =
            vec![OutlineCommand::Move([1.0, 2.0]); GLYPH_CACHE_MAX_COMMANDS].into();
        cache.insert(1000, large);
        assert_eq!(cache.stats.entries, 1);
        assert_eq!(cache.stats.commands, GLYPH_CACHE_MAX_COMMANDS);
        assert_eq!(retained.len(), 1);
        let oversized: Arc<[OutlineCommand]> =
            vec![OutlineCommand::Move([1.0, 2.0]); GLYPH_CACHE_MAX_COMMANDS + 1].into();
        cache.insert(1001, oversized);
        assert!(!cache.glyphs.contains_key(&1001));
        assert_eq!(cache.stats.entries, 1);
        assert_eq!(cache.stats.commands, GLYPH_CACHE_MAX_COMMANDS);
    }

    #[test]
    fn bundled_inventory_verifies_text_and_math_families() -> Result<(), crate::TextError> {
        let inventory = bundled_font_inventory()?;
        assert_eq!(
            inventory,
            BUNDLED_FONT_FAMILIES
                .iter()
                .map(|family| (*family).to_owned())
                .collect::<Vec<_>>()
        );
        assert!(is_bundled_text_family("CMU Serif"));
        assert!(!is_bundled_text_family("noto_sans-math"));
        assert!(!is_bundled_text_family("Host Font That Is Not Registered"));
        Ok(())
    }
}

/// Per-glyph metrics in ems of the face's design size.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct GlyphMetrics {
    /// Advance width.
    pub advance: f64,
    /// Ink extent above the baseline.
    pub height: f64,
    /// Ink extent below the baseline, positive.
    pub depth: f64,
}

/// Measure a glyph in ems.
#[must_use]
pub fn glyph_metrics(face: &Face, gid: u16) -> GlyphMetrics {
    let font = &face.font;
    let upm = f64::from(font.units_per_em.max(1));
    let advance = f64::from(font.advance_width(gid)) / upm;
    let (height, depth) = font.glyph_bbox(gid).map_or((0.0, 0.0), |bbox| {
        (
            f64::from(bbox[3]).max(0.0) / upm,
            (-f64::from(bbox[1])).max(0.0) / upm,
        )
    });
    GlyphMetrics {
        advance,
        height,
        depth,
    }
}

/// Kerning between two glyphs of one face, in ems (legacy kern table plus
/// the GPOS kern feature).
#[must_use]
pub fn kern_em(face: &Face, left: u16, right: u16) -> f64 {
    let upm = f64::from(face.font.units_per_em.max(1));
    f64::from(face.kern_units(left, right)) / upm
}
