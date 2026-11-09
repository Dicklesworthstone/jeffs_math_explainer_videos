//! The text-backed half of the de-TeX'd natives (fm-ebl):
//! [`BulletedList`] and [`Title`] (§11.6, Appendix A
//! `mobject/svg/special_tex`).
//!
//! The Reference routes both through LaTeX for want of anything better:
//! a `BulletedList` is one `TexText` of an `itemize`/`enumerate`
//! environment with each `\item` isolated, and a `Title` is a `TexText`
//! with a `Line` under it. Neither needs an typesetting engine — the
//! bullets and digits are ordinary text glyphs — so both are composed
//! here on the Scribe bridge ([`crate::text::Text`]) and the positional
//! layer ([`VMobject::arranged`], [`VMobject::next_to`]).
//!
//! Behaviour notes:
//!
//! * **BN (bullet mechanism).** The Reference's bullet is whatever
//!   `itemize` draws; ours is the literal `•` (U+2022) set as a `Text`
//!   at the item's own font size — the bundled faces map it, so no
//!   drawn-shape fallback is needed. `numbered=True` sets `"1."`,
//!   `"2."`, … labels the same way, the `enumerate` counterpart.
//! * **BN (label separation).** The gap between bullet and item is the
//!   fixed [`BULLET_BUFF`] (`MED_SMALL_BUFF`); the Reference defers it
//!   to LaTeX's `\labelsep`. Bullet and item are aligned centre to
//!   centre vertically; the Reference aligns them on a baseline.
//! * **BN (fade).** [`BulletedList::fade_all_but`] is a builder setter
//!   (this crate's value doctrine), applied to the arranged list at
//!   `build` time with the Reference's exact formula: the selected item
//!   keeps fill opacity 1.0 and is scaled so its bullet stands at the
//!   tallest bullet's height; every other item gets the fade opacity
//!   and `scale_factor` of that height, scaled about its left edge so
//!   the left alignment survives.

use fmn_core::constants::{
    DOWN, FRAME_WIDTH, FRAME_Y_RADIUS, GREY_C, LEFT, MED_LARGE_BUFF, MED_SMALL_BUFF, ORIGIN, RIGHT,
    SMALL_BUFF, UP,
};
use fmn_core::types::Vec3;
use fmn_mobject::Mobject;
use fmn_text::FontBook;

use crate::line::Line;
use crate::style::Style;
use crate::text::{DEFAULT_FONT_SIZE, Text, TextMobjectError};
use crate::vmobject::{VMobject, v_group};

/// The bullet glyph: U+2022, set as ordinary text (see the module's
/// behaviour notes).
const BULLET: &str = "\u{2022}";

/// The fixed gap between a bullet/number label and its item —
/// `MED_SMALL_BUFF`, standing in for LaTeX's `\labelsep`.
pub const BULLET_BUFF: f64 = MED_SMALL_BUFF;

/// The Reference's `fade_all_but` default `opacity=0.25`.
pub const DEFAULT_FADE_OPACITY: f64 = 0.25;

/// The Reference's `fade_all_but` default `scale_factor=0.7`.
pub const DEFAULT_FADE_SCALE_FACTOR: f64 = 0.7;

/// The Reference's `Title` default `font_size=72`.
pub const DEFAULT_TITLE_FONT_SIZE: f64 = 72.0;

/// The Reference's `Title` underline style: `stroke_width=2`,
/// `stroke_color=GREY_C`, fill off (a bare `Line`).
#[must_use]
pub fn default_underline_style() -> Style {
    Style::default().stroke(GREY_C, 2.0, 1.0)
}

/// `BulletedList(*items, buff=MED_LARGE_BUFF, aligned_edge=LEFT,
/// numbered=False)` — each item a line of bullet + text, the lines
/// arranged DOWN `buff` apart and left-aligned.
///
/// The built family's children are the item lines in order; each line
/// is a group of `[bullet, text]`, so `vmob.children()[i].children()[0]`
/// is item `i`'s bullet — the `part[0]` the Reference's
/// `fade_all_but` measures.
#[derive(Debug, Clone)]
pub struct BulletedList<'a> {
    items: &'a [&'a str],
    buff: f64,
    aligned_edge: Vec3,
    numbered: bool,
    font_size: f64,
    fade: Option<(usize, f64, f64)>,
}

impl<'a> BulletedList<'a> {
    /// A bulleted list of `items` with the Reference's defaults.
    #[must_use]
    pub fn new(items: &'a [&'a str]) -> Self {
        Self {
            items,
            buff: MED_LARGE_BUFF,
            aligned_edge: LEFT,
            numbered: false,
            font_size: DEFAULT_FONT_SIZE,
            fade: None,
        }
    }

    /// The `buff=` surface: vertical gap between items.
    #[must_use]
    pub fn buff(mut self, buff: f64) -> Self {
        self.buff = buff;
        self
    }

    /// The `aligned_edge=` surface (LEFT keeps every item's left edge
    /// on one x).
    #[must_use]
    pub fn aligned_edge(mut self, aligned_edge: Vec3) -> Self {
        self.aligned_edge = aligned_edge;
        self
    }

    /// The `numbered=` surface: `"1."`, `"2."`, … labels instead of
    /// bullets (the Reference's `enumerate`).
    #[must_use]
    pub fn numbered(mut self, numbered: bool) -> Self {
        self.numbered = numbered;
        self
    }

    /// The `font_size=` surface (the Reference passes it through to the
    /// underlying `TexText`).
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    /// The Reference's `fade_all_but(index)` with its defaults
    /// (`opacity=0.25`, `scale_factor=0.7`), applied at `build`.
    #[must_use]
    pub fn fade_all_but(self, index: usize) -> Self {
        self.fade_all_but_with(index, DEFAULT_FADE_OPACITY, DEFAULT_FADE_SCALE_FACTOR)
    }

    /// `fade_all_but(index, opacity, scale_factor)` with explicit
    /// parameters.
    #[must_use]
    pub fn fade_all_but_with(mut self, index: usize, opacity: f64, scale_factor: f64) -> Self {
        self.fade = Some((index, opacity, scale_factor));
        self
    }

    /// Build the list: one text per bullet and per item, lines arranged
    /// DOWN, the fade applied afterwards exactly as the Reference
    /// applies it after `arrange`.
    ///
    /// # Errors
    ///
    /// [`TextMobjectError`]: any item or label failing to lay out
    /// (unmapped characters, font policy) surfaces at construction
    /// time, never as a blank render.
    pub fn build(&self, book: &FontBook) -> Result<BulletedListMobject, TextMobjectError> {
        let mut lines = Vec::with_capacity(self.items.len());
        for (i, item) in self.items.iter().enumerate() {
            let label;
            let label_text = if self.numbered {
                label = format!("{}.", i + 1);
                label.as_str()
            } else {
                BULLET
            };
            let bullet = Text::new(label_text)
                .font_size(self.font_size)
                .build(book)?
                .vmob;
            let text = Text::new(item)
                .font_size(self.font_size)
                .build(book)?
                .vmob
                .next_to(&bullet, RIGHT, BULLET_BUFF, ORIGIN);
            lines.push(v_group([bullet, text]));
        }
        // `arrange(DOWN, buff, aligned_edge)` with its default
        // `center=True`: the arranged list is centred on the origin, as
        // every freshly built Reference mobject is (fm-5wq.55).
        let mut group =
            VMobject::arranged(lines, DOWN, self.buff, self.aligned_edge).moved_to(ORIGIN);
        if let Some((index, opacity, scale_factor)) = self.fade {
            group = fade_group(group, index, opacity, scale_factor);
        }
        Ok(BulletedListMobject { vmob: group })
    }
}

/// A built [`BulletedList`]: the arranged, optionally faded family.
#[derive(Debug, Clone, PartialEq)]
pub struct BulletedListMobject {
    /// The family: one child per item line, each `[bullet, text]`.
    pub vmob: VMobject,
}

impl From<BulletedListMobject> for Mobject {
    fn from(list: BulletedListMobject) -> Self {
        list.vmob.into()
    }
}

/// The Reference's `fade_all_but`, verbatim:
///
/// ```python
/// max_dot_height = max(item[0].get_height() for item in self.submobjects)
/// for i, part in enumerate(self.submobjects):
///     trg_dot_height = (1.0 if i == index else scale_factor) * max_dot_height
///     part.set_fill(opacity=(1.0 if i == index else opacity))
///     part.scale(trg_dot_height / part[0].get_height(), about_edge=LEFT)
/// ```
///
/// The selected item is *also* scaled when its bullet is shorter than
/// the tallest (its target is the maximum); with a uniform font size
/// every bullet is equally tall, so the selected item's factor is 1.0.
/// `set_fill` propagates to the family, hence `map_style_deep`; an
/// item whose bullet has no ink (height 0 — no real label reaches
/// this) keeps its scale.
fn fade_group(group: VMobject, index: usize, opacity: f64, scale_factor: f64) -> VMobject {
    let dot_height = |part: &VMobject| {
        part.children()
            .first()
            .map_or(0.0, |bullet| bullet.length_over_dim(1))
    };
    let max_dot_height = group.children().iter().map(dot_height).fold(0.0, f64::max);
    let mut parts = Vec::with_capacity(group.children().len());
    for (i, part) in group.children().iter().enumerate() {
        let selected = i == index;
        let target = if selected {
            max_dot_height
        } else {
            scale_factor * max_dot_height
        };
        let current = dot_height(part);
        let factor = if current > 0.0 { target / current } else { 1.0 };
        let fill = if selected { 1.0 } else { opacity };
        let mut faded = part.clone().map_style_deep(|s| s.fill_opacity(fill));
        if let Some(about) = faded.bbox_point(LEFT) {
            faded = faded.scaled_about(factor, about);
        }
        parts.push(faded);
    }
    v_group(parts)
}

/// `Title(*text_parts, font_size=72, include_underline=True,
/// underline_width=FRAME_WIDTH-2, match_underline_width_to_text=False,
/// underline_buff=SMALL_BUFF, underline_style=(stroke_width=2,
/// stroke_color=GREY_C))`.
///
/// The Reference joins the parts with a space and isolates each as its
/// own submobject, so part `i` stays addressable; here one `Text` of
/// the joined string is built and its glyph children are re-grouped by
/// source span, giving the same contract: the built family's children
/// are `[part_0, …, part_n-1]` plus the underline trailing when
/// included.
#[derive(Debug, Clone)]
pub struct Title<'a> {
    parts: &'a [&'a str],
    font_size: f64,
    include_underline: bool,
    underline_width: f64,
    match_underline_width_to_text: bool,
    underline_buff: f64,
    underline_style: Style,
}

impl<'a> Title<'a> {
    /// A title of `parts` with the Reference's defaults.
    #[must_use]
    pub fn new(parts: &'a [&'a str]) -> Self {
        Self {
            parts,
            font_size: DEFAULT_TITLE_FONT_SIZE,
            include_underline: true,
            underline_width: FRAME_WIDTH - 2.0,
            match_underline_width_to_text: false,
            underline_buff: SMALL_BUFF,
            underline_style: default_underline_style(),
        }
    }

    /// The `font_size=` surface (default 72).
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    /// The `include_underline=` surface.
    #[must_use]
    pub fn include_underline(mut self, include: bool) -> Self {
        self.include_underline = include;
        self
    }

    /// The `underline_width=` surface (default `FRAME_WIDTH - 2`);
    /// overridden by [`Title::match_underline_width_to_text`].
    #[must_use]
    pub fn underline_width(mut self, width: f64) -> Self {
        self.underline_width = width;
        self
    }

    /// The `match_underline_width_to_text=` surface: the underline
    /// takes the text's own width instead of `underline_width`.
    #[must_use]
    pub fn match_underline_width_to_text(mut self, on: bool) -> Self {
        self.match_underline_width_to_text = on;
        self
    }

    /// The `underline_buff=` surface (default `SMALL_BUFF`).
    #[must_use]
    pub fn underline_buff(mut self, buff: f64) -> Self {
        self.underline_buff = buff;
        self
    }

    /// The `underline_style=` surface (default
    /// [`default_underline_style`]).
    #[must_use]
    pub fn underline_style(mut self, style: Style) -> Self {
        self.underline_style = style;
        self
    }

    /// Build the title: the text to the top edge of the frame
    /// (`to_edge(UP, buff=MED_SMALL_BUFF)`), then the underline a
    /// `Line(LEFT, RIGHT)` placed `underline_buff` below and given its
    /// width — in that order, exactly the Reference's.
    ///
    /// # Errors
    ///
    /// [`TextMobjectError`]: the text failing to lay out surfaces at
    /// construction time.
    pub fn build(&self, book: &FontBook) -> Result<TitleMobject, TextMobjectError> {
        let joined = self.parts.join(" ");
        let text = Text::new(&joined).font_size(self.font_size).build(book)?;

        // Re-group the glyph children by part, the Reference's
        // `isolate=[*tex_strings]`: part i's source span in the joined
        // string names its glyphs, and decorations trail into the last
        // part. (The Reference strips the joined string first; parts
        // here are titles, not whitespace-padded fragments, so the
        // strip is dropped and the spans stay exact.)
        let mut part_groups: Vec<VMobject> = Vec::with_capacity(self.parts.len());
        let mut offset = 0;
        for part in self.parts {
            let range = (offset, offset + part.len());
            offset = range.1 + 1; // the joining space
            let ordinals = text.select(range);
            let glyphs = ordinals
                .iter()
                .map(|&o| text.vmob.children()[o].clone())
                .collect::<Vec<_>>();
            part_groups.push(v_group(glyphs).with_style(text.vmob.style()));
        }
        if let Some(last) = part_groups.last_mut() {
            for decoration in text.vmob.children().iter().skip(text.len()) {
                *last = last.clone().with_child(decoration.clone());
            }
        }
        // The Reference's TexText is centred on the origin at construction
        // (SVGMobject's `should_center`); Scribe lays text out from its
        // baseline origin, so centre the block before positioning
        // (fm-5wq.54: an uncentred block put the whole title right of
        // centre and its underline off the frame).
        let text_block = v_group(part_groups)
            .with_style(text.vmob.style())
            .moved_to(ORIGIN);

        // to_edge(UP, buff=MED_SMALL_BUFF), applied to the text alone
        // before the underline exists, as in the Reference.
        let text_block = match text_block.bbox_point(UP) {
            Some(top) => text_block.shifted([0.0, FRAME_Y_RADIUS - MED_SMALL_BUFF - top[1], 0.0]),
            None => text_block,
        };

        if !self.include_underline {
            return Ok(TitleMobject {
                vmob: text_block,
                include_underline: false,
            });
        }
        let underline = Line::new(LEFT, RIGHT)
            .style(self.underline_style)
            .build()
            .expect("the Underline template is a straight segment")
            .next_to(&text_block, DOWN, self.underline_buff, ORIGIN);
        let underline = if self.match_underline_width_to_text {
            underline.with_width(text_block.length_over_dim(0), false)
        } else {
            underline.with_width(self.underline_width, false)
        };
        Ok(TitleMobject {
            vmob: text_block.with_child(underline),
            include_underline: true,
        })
    }
}

/// A built [`Title`]: the part groups plus the trailing underline when
/// included.
#[derive(Debug, Clone, PartialEq)]
pub struct TitleMobject {
    /// The family: `[part_0, …, part_n-1]`, underline last when
    /// `include_underline`.
    pub vmob: VMobject,
    include_underline: bool,
}

impl TitleMobject {
    /// The underline child, when the title was built with one.
    #[must_use]
    pub fn underline(&self) -> Option<&VMobject> {
        if self.include_underline {
            self.vmob.children().last()
        } else {
            None
        }
    }
}

impl From<TitleMobject> for Mobject {
    fn from(title: TitleMobject) -> Self {
        title.vmob.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::FRAME_HEIGHT;

    fn book() -> FontBook {
        FontBook::bundled().expect("bundled book parses")
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn left_edge(v: &VMobject) -> f64 {
        v.extent().expect("has extent").0[0]
    }

    fn right_edge(v: &VMobject) -> f64 {
        v.extent().expect("has extent").1[0]
    }

    fn top_edge(v: &VMobject) -> f64 {
        v.extent().expect("has extent").1[1]
    }

    fn bottom_edge(v: &VMobject) -> f64 {
        v.extent().expect("has extent").0[1]
    }

    const ITEMS: [&str; 3] = ["First point", "Second point", "Third point"];

    fn built_list() -> BulletedListMobject {
        BulletedList::new(&ITEMS).build(&book()).expect("builds")
    }

    #[test]
    fn item_count_and_order() {
        let list = built_list();
        assert_eq!(list.vmob.children().len(), 3);
        // Arranged DOWN: each item sits strictly below the previous.
        let tops: Vec<f64> = list.vmob.children().iter().map(top_edge).collect();
        assert!(tops[0] > tops[1] && tops[1] > tops[2], "tops {tops:?}");
    }

    #[test]
    fn every_item_is_a_bullet_then_text_group() {
        let list = built_list();
        for (i, line) in list.vmob.children().iter().enumerate() {
            assert_eq!(line.children().len(), 2, "item {i} is [bullet, text]");
            let bullet = &line.children()[0];
            let text = &line.children()[1];
            // The bullet is one glyph; the text carries the item's
            // non-whitespace glyph count.
            assert_eq!(bullet.children().len(), 1, "item {i} bullet glyphs");
            let want_glyphs = ITEMS[i].chars().filter(|c| !c.is_whitespace()).count();
            assert_eq!(text.children().len(), want_glyphs, "item {i} glyphs");
        }
    }

    #[test]
    fn bullet_sits_left_of_and_level_with_its_item() {
        let list = built_list();
        for (i, line) in list.vmob.children().iter().enumerate() {
            let bullet = &line.children()[0];
            let text = &line.children()[1];
            assert!(
                right_edge(bullet) < left_edge(text),
                "item {i}: bullet right {} !< text left {}",
                right_edge(bullet),
                left_edge(text),
            );
            // Centre-to-centre vertical alignment.
            let dy = (bullet.center_point()[1] - text.center_point()[1]).abs();
            assert!(dy < 1e-9, "item {i}: bullet off level by {dy}");
        }
    }

    #[test]
    fn buff_spacing_is_exact() {
        for buff in [MED_LARGE_BUFF, 0.0, 1.25] {
            let list = BulletedList::new(&ITEMS)
                .buff(buff)
                .build(&book())
                .expect("builds");
            let lines = list.vmob.children();
            for pair in lines.windows(2) {
                let gap = bottom_edge(&pair[0]) - top_edge(&pair[1]);
                assert!(close(gap, buff), "buff {buff}: gap {gap}");
            }
        }
    }

    #[test]
    fn aligned_edge_left_keeps_left_edges_equal() {
        let list = built_list();
        let lefts: Vec<f64> = list.vmob.children().iter().map(left_edge).collect();
        for (i, &left) in lefts.iter().enumerate().skip(1) {
            assert!(
                close(left, lefts[0]),
                "item {i} left {left} != {}",
                lefts[0]
            );
        }
    }

    #[test]
    fn numbered_labels_replace_bullets() {
        let list = BulletedList::new(&ITEMS)
            .numbered(true)
            .build(&book())
            .expect("builds");
        for (i, line) in list.vmob.children().iter().enumerate() {
            let label = &line.children()[0];
            // "1.", "2.", "3." — two glyphs each, where a bullet is one.
            assert_eq!(label.children().len(), 2, "item {i} label glyphs");
            // Labels of differing text differ in ink; all are non-empty.
            assert!(label.extent().is_some());
        }
        // And the plain list's bullets are one glyph each.
        let plain = built_list();
        for line in plain.vmob.children() {
            assert_eq!(line.children()[0].children().len(), 1);
        }
    }

    #[test]
    fn fade_all_but_dims_and_shrinks_the_rest() {
        let faded = BulletedList::new(&ITEMS)
            .fade_all_but(1)
            .build(&book())
            .expect("builds");
        let plain = built_list();

        let bullet_height = |list: &BulletedListMobject, i: usize| {
            list.vmob.children()[i].children()[0].length_over_dim(1)
        };
        let plain_heights: Vec<f64> = (0..3).map(|i| bullet_height(&plain, i)).collect();
        let max_height = plain_heights.iter().copied().fold(0.0, f64::max);
        // Uniform font size ⇒ every bullet is the tallest.
        for &h in &plain_heights {
            assert!(close(h, max_height), "bullet heights {plain_heights:?}");
        }

        for (i, line) in faded.vmob.children().iter().enumerate() {
            // Fill opacity reaches every glyph (family propagation).
            let fill = line.children()[1].children()[0].style().fill_opacity;
            let height = bullet_height(&faded, i);
            if i == 1 {
                assert!(close(fill, 1.0), "selected fill {fill}");
                assert!(close(height, max_height), "selected height {height}");
            } else {
                assert!(close(fill, DEFAULT_FADE_OPACITY), "item {i} fill {fill}");
                assert!(
                    close(height, DEFAULT_FADE_SCALE_FACTOR * max_height),
                    "item {i} height {height}"
                );
            }
        }
        // Scaling about the left edge keeps the alignment.
        let lefts: Vec<f64> = faded.vmob.children().iter().map(left_edge).collect();
        for (i, &left) in lefts.iter().enumerate().skip(1) {
            assert!(close(left, lefts[0]), "faded item {i} left {left}");
        }
    }

    #[test]
    fn fade_all_but_with_honours_explicit_parameters() {
        let faded = BulletedList::new(&ITEMS)
            .fade_all_but_with(0, 0.5, 0.9)
            .build(&book())
            .expect("builds");
        let plain = built_list();
        let max_height = (0..3)
            .map(|i| plain.vmob.children()[i].children()[0].length_over_dim(1))
            .fold(0.0, f64::max);
        for (i, line) in faded.vmob.children().iter().enumerate() {
            let fill = line.children()[1].children()[0].style().fill_opacity;
            let height = line.children()[0].length_over_dim(1);
            if i == 0 {
                assert!(close(fill, 1.0));
                assert!(close(height, max_height));
            } else {
                assert!(close(fill, 0.5), "item {i} fill {fill}");
                assert!(close(height, 0.9 * max_height), "item {i} height {height}");
            }
        }
    }

    #[test]
    fn fade_on_an_empty_list_is_a_no_op() {
        let empty: [&str; 0] = [];
        let list = BulletedList::new(&empty)
            .fade_all_but(0)
            .build(&book())
            .expect("builds");
        assert!(list.vmob.children().is_empty());
    }

    const TITLE_PARTS: [&str; 1] = ["Title"];

    fn built_title() -> TitleMobject {
        Title::new(&TITLE_PARTS).build(&book()).expect("builds")
    }

    #[test]
    fn title_font_size_is_72_by_default() {
        // Calibrated: a "0" stands font_size / 144 units tall.
        let title = Title::new(&["0"]).build(&book()).expect("builds");
        let digit = &title.vmob.children()[0].children()[0];
        let height = digit.length_over_dim(1);
        assert!(
            close(height, DEFAULT_TITLE_FONT_SIZE / 144.0),
            "digit height {height}"
        );
    }

    #[test]
    fn title_text_goes_to_the_top_edge() {
        let title = built_title();
        let text_top = top_edge(&title.vmob.children()[0]);
        let want = FRAME_HEIGHT / 2.0 - MED_SMALL_BUFF;
        assert!(close(text_top, want), "text top {text_top} != {want}");
    }

    #[test]
    fn title_underline_matches_the_reference_defaults() {
        let title = built_title();
        let underline = title.underline().expect("underline included");
        assert!(close(underline.length_over_dim(0), FRAME_WIDTH - 2.0));
        assert_eq!(underline.style().stroke_width, 2.0);
        assert_eq!(underline.style().stroke_color, GREY_C);
        assert_eq!(underline.style().stroke_opacity, 1.0);
        // SMALL_BUFF below the text, centred under it.
        let text = &title.vmob.children()[0];
        let gap = bottom_edge(text) - top_edge(underline);
        assert!(close(gap, SMALL_BUFF), "underline gap {gap}");
        assert!(close(underline.center_point()[0], text.center_point()[0]));
    }

    /// fm-5wq.54: the Reference's title text is centred on x = 0 (TexText
    /// centres at construction; `to_edge(UP)` moves only y), so the whole
    /// title, underline included, is symmetric about the frame's centre.
    #[test]
    fn title_is_centred_horizontally_like_the_reference() {
        for underline in [true, false] {
            for matched in [true, false] {
                let title = Title::new(&["Hello", "World"])
                    .include_underline(underline)
                    .match_underline_width_to_text(matched)
                    .build(&book())
                    .expect("builds");
                let center = title.vmob.center_point()[0];
                assert!(center.abs() < 1e-9, "title centre x {center}");
                assert!(
                    close(left_edge(&title.vmob), -right_edge(&title.vmob)),
                    "asymmetric title"
                );
            }
        }
        let title = built_title();
        let underline = title.underline().expect("underline included");
        assert!(close(left_edge(underline), -(FRAME_WIDTH - 2.0) / 2.0));
        assert!(
            right_edge(underline) < FRAME_WIDTH / 2.0,
            "underline leaves the frame"
        );
    }

    /// fm-5wq.55: `arrange` re-centres by default, so a fresh list sits on
    /// the origin in both axes.
    #[test]
    fn bulleted_list_is_centred_on_the_origin() {
        for numbered in [false, true] {
            let list = BulletedList::new(&ITEMS)
                .numbered(numbered)
                .build(&book())
                .expect("builds");
            let [x, y, _] = list.vmob.center_point();
            assert!(x.abs() < 1e-9 && y.abs() < 1e-9, "list centre ({x}, {y})");
        }
    }

    #[test]
    fn match_underline_width_to_text_uses_the_text_width() {
        let title = Title::new(&TITLE_PARTS)
            .match_underline_width_to_text(true)
            .build(&book())
            .expect("builds");
        let underline = title.underline().expect("underline included");
        let text = &title.vmob.children()[0];
        assert!(close(underline.length_over_dim(0), text.length_over_dim(0)));
        assert!(underline.length_over_dim(0) < FRAME_WIDTH - 2.0);
    }

    #[test]
    fn title_parts_stay_addressable() {
        let title = Title::new(&["Hello", "World"])
            .build(&book())
            .expect("builds");
        // Two part groups plus the underline.
        assert_eq!(title.vmob.children().len(), 3);
        assert_eq!(title.vmob.children()[0].children().len(), 5, "Hello");
        assert_eq!(title.vmob.children()[1].children().len(), 5, "World");
        // Part 0 sits left of part 1 (the joining space between them).
        assert!(right_edge(&title.vmob.children()[0]) < left_edge(&title.vmob.children()[1]));
    }

    #[test]
    fn title_without_underline_has_only_parts() {
        let title = Title::new(&["Hello", "World"])
            .include_underline(false)
            .build(&book())
            .expect("builds");
        assert_eq!(title.vmob.children().len(), 2);
    }

    #[test]
    fn underline_style_setter_overrides() {
        let style = Style::default().stroke(fmn_core::constants::RED, 6.0, 1.0);
        let title = Title::new(&TITLE_PARTS)
            .underline_style(style)
            .build(&book())
            .expect("builds");
        let underline = title.underline().expect("underline included");
        assert_eq!(underline.style().stroke_width, 6.0);
        assert_eq!(underline.style().stroke_color, fmn_core::constants::RED);
    }

    #[test]
    fn titles_and_lists_enter_the_arena() {
        let mut stage = fmn_mobject::stage::Stage::new();
        let title = built_title();
        let mob = stage.add(title.vmob);
        assert_eq!(
            stage.family(mob).len(),
            1 + 1 + 5 + 1,
            "title + part + glyphs + underline"
        );
        let list = built_list();
        let mob = stage.add(list.vmob);
        // Per line: the line group, the bullet family + its glyph, the
        // text family + its glyphs.
        let glyphs: usize = ITEMS
            .iter()
            .map(|s| s.chars().filter(|c| !c.is_whitespace()).count())
            .sum();
        assert_eq!(stage.family(mob).len(), 1 + 3 * 4 + glyphs);
    }

    #[test]
    fn a_title_of_no_parts_still_builds() {
        let empty: [&str; 0] = [];
        let title = Title::new(&empty).build(&book()).expect("builds");
        assert_eq!(title.vmob.children().len(), 1, "underline alone");
    }
}
