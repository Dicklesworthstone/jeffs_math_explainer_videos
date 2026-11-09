//! Source-span matching for native Rust scenes (§11.3, D-09).
//!
//! Save [`TexMobject::span_map`](crate::TexMobject::span_map) or
//! [`TextMobject::span_map`](crate::TextMobject::span_map) before moving the
//! built value into a Stage. The map binds to that root's direct children;
//! moving or restyling them is fine, but changing their topology invalidates
//! the binding. Text decorations after the mapped glyphs fade as unmatched
//! pieces. Matching never infers text identity from glyph geometry.
//!
//! [`TransformMatchingTex`] pairs equal native source slices in occurrence
//! order. [`TransformMatchingStrings`] repeatedly claims the longest common
//! contiguous run, so reordered words can move together. Claimed slots remain
//! barriers: removing them would invent substrings that never existed.
//! Both are ordinary Choreo animations: groups of Transform and directional
//! fades, followed by removal of the source and publication of the real target.

use std::collections::{BTreeMap, HashSet, VecDeque};

use fmn_anim::{
    AnimError, AnimState, Animation, AnimationGroup, AnimationSignature, RateFunc, Transform,
    fade_in_from_point, fade_out_to_point,
};
use fmn_mobject::{IdBuildHasher, Mob, Mobject, Stage, StageError};

use crate::spans::SpanMapData;

/// Maximum direct children on either operand of a matching animation.
pub const MAX_MATCHING_PARTS: usize = 65_536;
/// Maximum equal-key comparisons in a longest-block search, across all rounds.
pub const MAX_MATCHING_COMPARISONS: usize = 1_048_576;

/// Timing of a native text-matching composition. Child transforms retain their
/// normal smooth rate curve; `rate_func` shapes the composition's timeline.
#[derive(Clone, Debug)]
pub struct TextMatchingConfig {
    /// Total duration in seconds (the Reference's matching default is two).
    pub run_time: f64,
    /// Stagger between matched blocks and unmatched pieces.
    pub lag_ratio: f64,
    /// Composition rate curve; linear avoids applying smooth twice.
    pub rate_func: RateFunc,
}

impl Default for TextMatchingConfig {
    fn default() -> Self {
        Self {
            run_time: 2.0,
            lag_ratio: 0.0,
            rate_func: RateFunc::linear(),
        }
    }
}

/// A matching plan was refused before any groups or animations were created.
#[derive(Debug)]
pub enum TextMatchingError {
    /// A Choreo or arena operation failed.
    Animation(AnimError),
    /// A native source span is empty, out of bounds, or splits UTF-8.
    InvalidSpan {
        /// `source` or `target`.
        side: &'static str,
        /// Span ordinal.
        index: usize,
    },
    /// The family no longer has the native map's direct-child layout.
    InvalidLayout {
        /// `source` or `target`.
        side: &'static str,
        /// The violated binding contract.
        reason: &'static str,
    },
    /// Both operands contain the same live member. Copy the target first.
    AliasedFamilies,
    /// Matching exceeded a fixed resource ceiling.
    BudgetExceeded {
        /// The bounded resource.
        resource: &'static str,
        /// Its ceiling.
        limit: usize,
    },
}

impl std::fmt::Display for TextMatchingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Animation(error) => error.fmt(f),
            Self::InvalidSpan { side, index } => {
                write!(
                    f,
                    "{side} native span {index} is not a nonempty UTF-8 source range"
                )
            }
            Self::InvalidLayout { side, reason } => {
                write!(f, "{side} native span layout is invalid: {reason}")
            }
            Self::AliasedFamilies => {
                write!(
                    f,
                    "matching operands share live members; copy the target first"
                )
            }
            Self::BudgetExceeded { resource, limit } => {
                write!(f, "text matching exceeds the {limit} {resource} budget")
            }
        }
    }
}

impl std::error::Error for TextMatchingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Animation(error) => Some(error),
            _ => None,
        }
    }
}

impl From<AnimError> for TextMatchingError {
    fn from(error: AnimError) -> Self {
        Self::Animation(error)
    }
}

struct Binding {
    root: Mob,
    children: Vec<Mob>,
}

impl Binding {
    fn read<'a>(
        stage: &Stage,
        root: Mob,
        spans: &'a SpanMapData,
        side: &'static str,
    ) -> Result<(Self, Vec<&'a str>), TextMatchingError> {
        let entry = stage.get(root).ok_or(AnimError::StaleHandle(root))?;
        let children = entry.submobjects();
        let invalid = |reason| TextMatchingError::InvalidLayout { side, reason };
        if children.len() > MAX_MATCHING_PARTS {
            return Err(TextMatchingError::BudgetExceeded {
                resource: "parts per operand",
                limit: MAX_MATCHING_PARTS,
            });
        }
        if !entry.buffer.is_empty() {
            return Err(invalid(
                "the source-mapped root must be a point-free container",
            ));
        }
        if spans.entries.len() > children.len() {
            return Err(invalid("there are more spans than direct children"));
        }
        for &child in children {
            let part = stage.get(child).ok_or(AnimError::StaleHandle(child))?;
            if !part.submobjects().is_empty() {
                return Err(invalid(
                    "a native primitive was regrouped; rebuild its span map",
                ));
            }
            if spans.entries.is_empty() && !part.buffer.is_empty() {
                return Err(invalid("point-bearing strings require a native span map"));
            }
        }
        let keys = spans
            .entries
            .iter()
            .enumerate()
            .map(|(index, span)| {
                spans
                    .source
                    .get(span.start..span.end)
                    .filter(|key| !key.is_empty())
                    .ok_or(TextMatchingError::InvalidSpan { side, index })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok((
            Self {
                root,
                children: children.to_vec(),
            },
            keys,
        ))
    }

    fn validate(&self, stage: &Stage) -> Result<(), AnimError> {
        let entry = stage
            .get(self.root)
            .ok_or(AnimError::StaleHandle(self.root))?;
        if entry.submobjects() != self.children.as_slice() || !entry.buffer.is_empty() {
            return Err(AnimError::Stage(StageError::FamilyShapeMismatch));
        }
        for &child in &self.children {
            let part = stage.get(child).ok_or(AnimError::StaleHandle(child))?;
            if !part.submobjects().is_empty() {
                return Err(AnimError::Stage(StageError::FamilyShapeMismatch));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum MatchingKind {
    Tex,
    Strings,
}

type PartPair = (Vec<usize>, Vec<usize>);

fn tex_pairs(source: &[&str], target: &[&str]) -> Vec<PartPair> {
    let mut available: BTreeMap<&str, VecDeque<usize>> = BTreeMap::new();
    for (index, key) in target.iter().enumerate() {
        available.entry(*key).or_default().push_back(index);
    }
    source
        .iter()
        .enumerate()
        .filter_map(|(index, key)| {
            let other = available.get_mut(*key)?.pop_front()?;
            Some((vec![index], vec![other]))
        })
        .collect()
}

/// SequenceMatcher's longest contiguous block, with deterministic earliest
/// source/target tie breaking. Index equal keys rather than scanning the full
/// Cartesian product; used slots are barriers, not deleted sequence entries.
fn string_pairs(source: &[&str], target: &[&str]) -> Result<Vec<PartPair>, TextMatchingError> {
    if source.is_empty() || target.is_empty() {
        return Ok(Vec::new());
    }
    if source == target {
        return Ok(vec![(
            (0..source.len()).collect(),
            (0..target.len()).collect(),
        )]);
    }
    let mut positions: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (index, key) in target.iter().enumerate() {
        positions.entry(*key).or_default().push(index);
    }
    let mut used_source = vec![false; source.len()];
    let mut used_target = vec![false; target.len()];
    let mut pairs = Vec::new();
    let mut comparisons = 0;
    loop {
        let mut previous: BTreeMap<usize, usize> = BTreeMap::new();
        let mut best = (0, 0, 0);
        for (i, key) in source.iter().enumerate() {
            let mut current = BTreeMap::new();
            if !used_source[i] {
                for &j in positions.get(*key).into_iter().flatten() {
                    if used_target[j] {
                        continue;
                    }
                    comparisons += 1;
                    if comparisons > MAX_MATCHING_COMPARISONS {
                        return Err(TextMatchingError::BudgetExceeded {
                            resource: "equal-key comparisons",
                            limit: MAX_MATCHING_COMPARISONS,
                        });
                    }
                    let length = j
                        .checked_sub(1)
                        .and_then(|before| previous.get(&before))
                        .copied()
                        .unwrap_or(0)
                        + 1;
                    current.insert(j, length);
                    if length > best.2 {
                        best = (i + 1 - length, j + 1 - length, length);
                    }
                }
            }
            previous = current;
        }
        let (a, b, length) = best;
        if length == 0 {
            return Ok(pairs);
        }
        used_source[a..a + length].fill(true);
        used_target[b..b + length].fill(true);
        pairs.push(((a..a + length).collect(), (b..b + length).collect()));
    }
}

fn part_group(stage: &mut Stage, children: &[Mob], indices: &[usize]) -> Result<Mob, AnimError> {
    if indices.len() == 1 {
        return Ok(children[indices[0]]);
    }
    let group = stage.add(Mobject::new());
    let members: Vec<Mob> = indices.iter().map(|&index| children[index]).collect();
    stage.replace_children(group, &members)?;
    Ok(group)
}

struct MatchingAnimation {
    group: AnimationGroup,
    source: Binding,
    target: Binding,
    begun: bool,
    finished: bool,
    cleaned: bool,
    error: Option<AnimError>,
}

impl MatchingAnimation {
    fn new(
        stage: &mut Stage,
        source: Mob,
        target: Mob,
        source_spans: &SpanMapData,
        target_spans: &SpanMapData,
        config: TextMatchingConfig,
        kind: MatchingKind,
    ) -> Result<Self, TextMatchingError> {
        let (source, source_keys) = Binding::read(stage, source, source_spans, "source")?;
        let (target, target_keys) = Binding::read(stage, target, target_spans, "target")?;
        let source_family: HashSet<Mob, IdBuildHasher> = std::iter::once(source.root)
            .chain(source.children.iter().copied())
            .collect();
        if std::iter::once(target.root)
            .chain(target.children.iter().copied())
            .any(|mob| source_family.contains(&mob))
        {
            return Err(TextMatchingError::AliasedFamilies);
        }
        // Finish every fallible provenance/search check before editing the arena.
        let pairs = match kind {
            MatchingKind::Tex => tex_pairs(&source_keys, &target_keys),
            MatchingKind::Strings => string_pairs(&source_keys, &target_keys)?,
        };
        let source_center = stage.get_center(source.root);
        let target_center = stage.get_center(target.root);
        let mut used_source = vec![false; source.children.len()];
        let mut used_target = vec![false; target.children.len()];
        let mut animations: Vec<Box<dyn Animation>> = Vec::new();
        for (a, b) in pairs {
            let from = part_group(stage, &source.children, &a)?;
            let to = part_group(stage, &target.children, &b)?;
            animations.push(Box::new(Transform::new(from, to)));
            for index in a {
                used_source[index] = true;
            }
            for index in b {
                used_target[index] = true;
            }
        }
        for (index, &mob) in source.children.iter().enumerate() {
            if !used_source[index] {
                animations.push(Box::new(fade_out_to_point(stage, mob, target_center)?));
            }
        }
        for (index, &mob) in target.children.iter().enumerate() {
            if !used_target[index] {
                animations.push(Box::new(fade_in_from_point(stage, mob, source_center)?));
            }
        }
        // Empty text still replaces its root and consumes the authored time.
        // A self-transform of the point-free source uses the ordinary lifecycle.
        if animations.is_empty() {
            animations.push(Box::new(Transform::new(source.root, source.root)));
        }
        let name = match kind {
            MatchingKind::Tex => "TransformMatchingTex",
            MatchingKind::Strings => "TransformMatchingStrings",
        };
        let group = AnimationGroup::with_lag_ratio(stage, animations, config.lag_ratio)?
            .with_run_time(config.run_time)
            .with_rate_func(config.rate_func)
            .with_name(name);
        Ok(Self {
            group,
            source,
            target,
            begun: false,
            finished: false,
            cleaned: false,
            error: None,
        })
    }

    fn inventory(&self, mut members: Vec<Mob>) -> Vec<Mob> {
        for mob in [self.source.root, self.target.root] {
            if !members.contains(&mob) {
                members.push(mob);
            }
        }
        members
    }
}

macro_rules! matching_animation {
    ($name:ident, $kind:ident, $description:literal) => {
        #[doc = $description]
        pub struct $name {
            inner: MatchingAnimation,
        }

        impl $name {
            /// Bind the two native source maps and build a two-second animation.
            /// Neither target rooting nor interpolation happens at construction.
            ///
            /// # Errors
            /// Invalid/stale maps, shared operands, or exceeded search budgets.
            pub fn new(
                stage: &mut Stage,
                source: Mob,
                target: Mob,
                source_spans: &SpanMapData,
                target_spans: &SpanMapData,
            ) -> Result<Self, TextMatchingError> {
                Self::with_config(
                    stage,
                    source,
                    target,
                    source_spans,
                    target_spans,
                    TextMatchingConfig::default(),
                )
            }

            /// Build with explicit composition timing.
            ///
            /// # Errors
            /// As [`Self::new`].
            pub fn with_config(
                stage: &mut Stage,
                source: Mob,
                target: Mob,
                source_spans: &SpanMapData,
                target_spans: &SpanMapData,
                config: TextMatchingConfig,
            ) -> Result<Self, TextMatchingError> {
                Ok(Self {
                    inner: MatchingAnimation::new(
                        stage,
                        source,
                        target,
                        source_spans,
                        target_spans,
                        config,
                        MatchingKind::$kind,
                    )?,
                })
            }

            /// The actual Transform/fade plan, in composition order.
            #[must_use]
            pub fn animations(&self) -> &[Box<dyn Animation>] {
                self.inner.group.animations()
            }

            /// The original target, published on successful cleanup.
            #[must_use]
            pub fn target(&self) -> Mob {
                self.inner.target.root
            }
        }

        impl Animation for $name {
            fn state(&self) -> &AnimState {
                self.inner.group.state()
            }
            fn state_mut(&mut self) -> &mut AnimState {
                self.inner.group.state_mut()
            }
            fn all_mobjects(&self) -> Vec<Mob> {
                self.inner.inventory(self.inner.group.all_mobjects())
            }
            fn preflight_mobjects(&self) -> Vec<Mob> {
                self.inner.inventory(self.inner.group.preflight_mobjects())
            }
            fn validate_begin(&self, stage: &Stage) -> Result<(), AnimError> {
                self.inner.source.validate(stage)?;
                self.inner.target.validate(stage)?;
                self.inner.group.validate_begin(stage)
            }
            fn begin(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
                self.validate_begin(stage)?;
                self.inner.begun = false;
                self.inner.finished = false;
                self.inner.cleaned = false;
                self.inner.error = None;
                self.inner.group.begin(stage)?;
                self.inner.begun = true;
                Ok(())
            }
            fn interpolate_submobject(&mut self, _stage: &mut Stage, _mobs: &[Mob], _alpha: f64) {}
            fn interpolate(&mut self, stage: &mut Stage, alpha: f64) {
                self.inner.group.interpolate(stage, alpha);
            }
            fn update_mobjects(&mut self, stage: &mut Stage, dt: f64) {
                self.inner.group.update_mobjects(stage, dt);
            }
            fn finish(&mut self, stage: &mut Stage) {
                if self.inner.begun && !self.inner.finished {
                    self.inner.group.finish(stage);
                    self.inner.finished = true;
                }
            }
            fn abort(&mut self, stage: &mut Stage) {
                self.inner.group.abort(stage);
                self.inner.begun = false;
                self.inner.finished = false;
            }
            fn clean_up_from_scene(&mut self, stage: &mut Stage) {
                if self.inner.cleaned {
                    return;
                }
                if !self.inner.finished {
                    self.inner.error = Some(AnimError::InvalidFramePhase(
                        "text matching must finish before scene cleanup",
                    ));
                    return;
                }
                if !stage.contains(self.inner.target.root) {
                    self.inner.error = Some(AnimError::StaleHandle(self.inner.target.root));
                    return;
                }
                self.inner.group.clean_up_from_scene(stage);
                stage.remove_many_from_scene(&[self.state().mobject(), self.inner.source.root]);
                if let Err(error) = stage.add_to_scene(self.inner.target.root) {
                    self.inner.error = Some(AnimError::Stage(error));
                    return;
                }
                self.inner.cleaned = true;
            }
            fn deferred_error(&self) -> Option<AnimError> {
                self.inner
                    .error
                    .clone()
                    .or_else(|| self.inner.group.deferred_error())
            }
            fn collect_resumed_updater_mobjects(&self, out: &mut Vec<Mob>) {
                self.inner.group.collect_resumed_updater_mobjects(out);
            }
            fn effect_signature(&self) -> AnimationSignature {
                self.inner.group.effect_signature()
            }
        }
    };
}

matching_animation!(
    TransformMatchingTex,
    Tex,
    "Match mathematical primitives by native source identity, never by outline similarity."
);
matching_animation!(
    TransformMatchingStrings,
    Strings,
    "Move shared string runs together, including runs reordered between source and target."
);

#[cfg(test)]
mod tests {
    use super::{TextMatchingError, string_pairs, tex_pairs};

    #[test]
    fn tex_repeated_keys_claim_in_occurrence_order() {
        assert_eq!(
            tex_pairs(&["x", "x", "+"], &["x", "+", "x"]),
            [(vec![0], vec![0]), (vec![1], vec![2]), (vec![2], vec![1])]
        );
    }

    #[test]
    fn longest_block_ties_choose_earliest_source_then_target() {
        assert_eq!(
            string_pairs(&["A", "B", "A"], &["B", "A", "B"]).unwrap(),
            [(vec![0, 1], vec![1, 2])]
        );
        assert_eq!(
            string_pairs(&["A"], &["A", "A"]).unwrap(),
            [(vec![0], vec![0])]
        );
    }

    #[test]
    fn identical_large_inputs_have_a_linear_identity_path() {
        let keys = vec!["A"; 65_536];
        let pairs = string_pairs(&keys, &keys).unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0.len(), keys.len());
        assert_eq!(pairs[0].0, pairs[0].1);
    }

    #[test]
    fn pathological_repeated_keys_refuse_bounded_search() {
        assert!(matches!(
            string_pairs(&vec!["A"; 1025], &vec!["A"; 1026]),
            Err(TextMatchingError::BudgetExceeded { .. })
        ));
    }
}
