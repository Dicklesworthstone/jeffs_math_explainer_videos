//! Source-span records: the composition-root seam between Scribe span
//! maps and the Studio inspector.
//!
//! `fmn-tex`'s [`fmn_tex::Typeset`] and `fmn-text`'s
//! [`fmn_text::TextLayout`] already carry per-submobject source byte
//! ranges ([`TexMobject::span_map`](crate::tex::TexMobject::span_map),
//! [`TextMobject::span_map`](crate::text::TextMobject::span_map)). The
//! Studio inspector consumes those spans only through its lower-DAG
//! `SpanRegistry` (`fmn-studio::inspect`), keyed by live [`Mob`]
//! handles — handles a reconstructed frame stage never shares with the
//! constructing arena. This module is the translation seam the crate DAG
//! mandates (Studio must not depend on the Scribe crates): a
//! [`SpanCollector`] records each built value's span map keyed by the
//! value's **stable** top-level draw-list ordinal, and the composition
//! root (the Studio worker) re-resolves that ordinal against the live
//! stage on every inspection.

use std::fmt;
use std::sync::Arc;

use fmn_mobject::{Mob, Mobject, Stage, StageError};

/// Upper bound on records one collector retains.
///
/// A scene that streams unboundedly many span-mapped values through one
/// collector hits this typed ceiling instead of an unbounded table.
pub const MAX_SPAN_RECORDS: usize = 4096;

/// Library-local kind of the source construct behind one span entry.
///
/// The vocabulary mirrors `fmn-studio::inspect::SpanKind` one-to-one; the
/// composition root translates. Kept here so the library's span data
/// stays inside the governed DAG (no Studio edge).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SpanKindU8 {
    /// Plain-text glyph.
    TextGlyph = 0,
    /// Math glyph.
    MathGlyph = 1,
    /// Math rule.
    MathRule = 2,
    /// Math outline/path.
    MathPath = 3,
}

/// One source byte range with its construct kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpanMapEntry {
    /// Inclusive source byte offset.
    pub start: usize,
    /// Exclusive source byte offset.
    pub end: usize,
    /// Kind of source construct.
    pub kind: SpanKindU8,
}

/// One value's native span map: the verbatim source plus one entry per
/// span-mapped submobject, in child order (entry `i` is child ordinal
/// `i`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpanMapData {
    /// The source string, verbatim.
    pub source: Arc<str>,
    /// The ordered entries.
    pub entries: Vec<SpanMapEntry>,
}

/// One collected span map, keyed by stable identity: the root's ordinal
/// in the stage's top-level draw list at record time.
///
/// The draw list is z-sorted on every rooting, so ordinals are meaningful
/// for scenes whose spanned roots share one z (the ordinary case); the
/// worker re-validates each record against the live stage and skips any
/// that no longer resolves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpanRecord {
    /// Index into the stage's `roots()` at record time.
    pub root_ordinal: usize,
    /// The source string, verbatim.
    pub source: Arc<str>,
    /// The ordered entries; entry `i` binds to child ordinal `i`.
    pub entries: Vec<SpanMapEntry>,
}

/// Span-record refusal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpanCollectorError {
    /// The collector is at [`MAX_SPAN_RECORDS`].
    RecordCapacity {
        /// Records already held.
        held: usize,
        /// The ceiling.
        limit: usize,
    },
    /// Record or entry storage could not be reserved.
    Allocation {
        /// What could not be reserved.
        context: &'static str,
        /// Elements requested.
        requested: usize,
    },
    /// The handle is not a root of the stage's draw list, so it has no
    /// stable top-level ordinal to record.
    RootNotListed,
    /// Rooting the freshly added value in the draw list failed.
    Stage(StageError),
}

impl fmt::Display for SpanCollectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecordCapacity { held, limit } => {
                write!(f, "span records at capacity: {held} held, limit {limit}")
            }
            Self::Allocation { context, requested } => {
                write!(f, "could not reserve {context} ({requested} elements)")
            }
            Self::RootNotListed => write!(
                f,
                "span-mapped value is not a root of the stage's draw list, \
                 so it has no stable top-level ordinal"
            ),
            Self::Stage(error) => write!(f, "stage operation failed: {error}"),
        }
    }
}

impl std::error::Error for SpanCollectorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Stage(error) => Some(error),
            _ => None,
        }
    }
}

/// Bounded collector of [`SpanRecord`]s.
///
/// The composition root owns one collector per scene run, hands it to the
/// construct path, and harvests it into worker state afterwards.
#[derive(Clone, Debug, Default)]
pub struct SpanCollector {
    records: Vec<SpanRecord>,
}

impl SpanCollector {
    /// Record one value's span map against its just-added root handle.
    ///
    /// `stage` must be the stage the value was just rooted in; the stable
    /// identity is resolved from the stage's own root enumeration.
    ///
    /// # Errors
    ///
    /// [`SpanCollectorError::RootNotListed`] if the handle is not a root
    /// of the draw list; [`SpanCollectorError::RecordCapacity`] and
    /// [`SpanCollectorError::Allocation`] per the bounded table.
    pub fn record(
        &mut self,
        stage: &Stage,
        mob: Mob,
        data: SpanMapData,
    ) -> Result<(), SpanCollectorError> {
        let root_ordinal = stage
            .roots()
            .iter()
            .position(|root| *root == mob)
            .ok_or(SpanCollectorError::RootNotListed)?;
        self.push(SpanRecord {
            root_ordinal,
            source: data.source,
            entries: data.entries,
        })
    }

    /// Push one fully-resolved record.
    ///
    /// # Errors
    ///
    /// [`SpanCollectorError::RecordCapacity`] at the ceiling;
    /// [`SpanCollectorError::Allocation`] if the table cannot grow.
    pub fn push(&mut self, record: SpanRecord) -> Result<(), SpanCollectorError> {
        let requested = self.records.len() + 1;
        if requested > MAX_SPAN_RECORDS {
            return Err(SpanCollectorError::RecordCapacity {
                held: self.records.len(),
                limit: MAX_SPAN_RECORDS,
            });
        }
        self.records
            .try_reserve_exact(1)
            .map_err(|_| SpanCollectorError::Allocation {
                context: "span records",
                requested,
            })?;
        self.records.push(record);
        Ok(())
    }

    /// The records, in collection order.
    #[must_use]
    pub fn records(&self) -> &[SpanRecord] {
        &self.records
    }

    /// The number of records held.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// True when nothing was recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Consume the collector into its records.
    #[must_use]
    pub fn into_records(self) -> Vec<SpanRecord> {
        self.records
    }
}

/// Add a value to the arena, root it in the draw list, and record its
/// span map — the raw-arena form of the scene path's
/// `stage.add(value)` + `collector.record(stage.arena(), mob, data)`.
///
/// # Errors
///
/// [`SpanCollectorError::Stage`] if rooting fails; otherwise as
/// [`SpanCollector::record`].
pub fn add_with_spans(
    stage: &mut Stage,
    value: impl Into<Mobject>,
    data: SpanMapData,
    collector: &mut SpanCollector,
) -> Result<Mob, SpanCollectorError> {
    let mob = stage.add(value);
    stage.add_to_scene(mob).map_err(SpanCollectorError::Stage)?;
    collector.record(stage, mob, data)?;
    Ok(mob)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tex::Tex;
    use crate::text::Text;
    use crate::{FontBook, TexEngine};

    fn engine() -> TexEngine {
        TexEngine::new("fmd-math/pack/default", None).expect("engine")
    }

    fn book() -> FontBook {
        FontBook::bundled().expect("bundled faces parse")
    }

    #[test]
    fn tex_span_map_entries_are_child_ordered_and_source_exact() {
        let tex = Tex::new(r"\frac{x}{y}").build(&engine()).expect("builds");
        let data = tex.span_map();

        assert_eq!(data.source.as_ref(), r"\frac{x}{y}");
        // One entry per Sub, in child order: the family's child count.
        assert_eq!(data.entries.len(), tex.len());
        assert_eq!(data.entries.len(), tex.vmob.children().len());

        // Every entry slices the source cleanly; the formula's `x` and
        // `y` glyphs carry their exact byte ranges; the fraction bar is
        // a rule.
        let mut glyphs = 0;
        let mut rules = 0;
        let mut saw_x = false;
        let mut saw_y = false;
        for (ordinal, entry) in data.entries.iter().enumerate() {
            let slice = &data.source[entry.start..entry.end];
            assert!(
                data.source.is_char_boundary(entry.start)
                    && data.source.is_char_boundary(entry.end)
                    && entry.start <= entry.end,
                "entry {ordinal} range {}..{} is not a clean slice",
                entry.start,
                entry.end
            );
            match entry.kind {
                SpanKindU8::MathGlyph => {
                    glyphs += 1;
                    if slice == "x" {
                        saw_x = true;
                    }
                    if slice == "y" {
                        saw_y = true;
                    }
                }
                SpanKindU8::MathRule => rules += 1,
                SpanKindU8::TextGlyph | SpanKindU8::MathPath => {}
            }
        }
        assert_eq!(glyphs, 2, "the two variable glyphs");
        assert_eq!(rules, 1, "the fraction bar");
        assert!(saw_x && saw_y, "x and y glyphs must carry their own spans");
    }

    #[test]
    fn text_span_map_entries_are_the_glyphs_and_exclude_decorations() {
        let text = Text::new("hi").build(&book()).expect("builds");
        let data = text.span_map();

        assert_eq!(data.source.as_ref(), "hi");
        assert_eq!(data.entries.len(), text.len());
        for (ordinal, entry) in data.entries.iter().enumerate() {
            assert_eq!(entry.kind, SpanKindU8::TextGlyph);
            let glyph = &text.layout.glyphs[ordinal];
            assert_eq!(
                (entry.start, entry.end),
                glyph.span,
                "entry {ordinal} must be glyph {ordinal}'s own span"
            );
            assert_eq!(
                &data.source[entry.start..entry.end],
                &"hi"[ordinal..ordinal + 1]
            );
        }
    }

    #[test]
    fn add_with_spans_records_stable_root_ordinals_in_insertion_order() {
        let tex = Tex::new("x+y").build(&engine()).expect("builds");
        let text = Text::new("ab").build(&book()).expect("builds");
        let mut stage = Stage::new();
        let mut collector = SpanCollector::default();

        let (tex_data, text_data) = (tex.span_map(), text.span_map());
        let tex_root =
            add_with_spans(&mut stage, tex.vmob, tex_data, &mut collector).expect("records");
        let text_root =
            add_with_spans(&mut stage, text.vmob, text_data, &mut collector).expect("records");

        let records = collector.records();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].root_ordinal, 0);
        assert_eq!(records[1].root_ordinal, 1);
        // The ordinals name the recorded roots in the live draw list.
        assert_eq!(stage.roots()[0], tex_root);
        assert_eq!(stage.roots()[1], text_root);
        // Entry i is child ordinal i for the formula family.
        assert_eq!(
            records[0].entries.len(),
            stage.get(tex_root).expect("live").submobjects().len()
        );
    }

    #[test]
    fn record_refuses_a_handle_outside_the_draw_list() {
        let tex = Tex::new("x").build(&engine()).expect("builds");
        let mut stage = Stage::new();
        let data = tex.span_map();
        let detached = stage.add(tex.vmob);
        let mut collector = SpanCollector::default();

        let error = collector
            .record(&stage, detached, data)
            .expect_err("an unrooted handle has no ordinal");
        assert_eq!(error, SpanCollectorError::RootNotListed);
        assert!(collector.is_empty());
    }

    #[test]
    fn push_enforces_the_record_ceiling() {
        let mut collector = SpanCollector::default();
        let record = SpanRecord {
            root_ordinal: 0,
            source: Arc::from("s"),
            entries: Vec::new(),
        };
        for ordinal in 0..MAX_SPAN_RECORDS {
            collector
                .push(SpanRecord {
                    root_ordinal: ordinal,
                    source: Arc::from("s"),
                    entries: Vec::new(),
                })
                .expect("under the ceiling");
        }
        let error = collector.push(record).expect_err("at the ceiling");
        assert_eq!(
            error,
            SpanCollectorError::RecordCapacity {
                held: MAX_SPAN_RECORDS,
                limit: MAX_SPAN_RECORDS,
            }
        );
        assert_eq!(collector.len(), MAX_SPAN_RECORDS);
    }
}
