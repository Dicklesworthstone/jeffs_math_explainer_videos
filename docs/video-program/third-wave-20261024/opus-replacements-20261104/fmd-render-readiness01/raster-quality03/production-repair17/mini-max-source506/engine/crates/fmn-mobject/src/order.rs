//! The render-order model (§8.5, §1.5, fm-jsc): what draws over what.
//!
//! Draw order is **meaning, not pixels** — a scene whose overlay lands under
//! its diagram is wrong no matter how beautifully it rasterizes — so the
//! Reference's actual ordering semantics are implemented here, engine-side,
//! and pinned by the ordering-trace corpus. Lumen consumes this as its
//! draw-order model (§10.1); painter's order for transparent content and
//! §10.8's per-tile command runs descend from it. The written spec,
//! including every citation into the pinned Reference, is
//! `docs/RENDER_ORDER.md`; it is normative for the W5 beads.
//!
//! **Two levels, one sequence.**
//!
//! 1. *The scene list.* Top-level scene members, ordered back to front. The
//!    Reference re-sorts on `add` by `(z_index, current position)` — a
//!    **stable** sort, so equal z_index keeps insertion order
//!    (`scene.py:338`). Adjacent members sharing `(class, shader id,
//!    z_index)` form one **render group** (`assemble_render_groups`,
//!    `scene.py:300`).
//! 2. *The families.* Each group draws its members' families in
//!    depth-first order, pointless members skipped, and adjacent members
//!    sharing a shader id draw as one batch (`get_shader_wrapper_list`,
//!    `mobject.py:2056`).
//!
//! A batch therefore never spans a render-group boundary — which is the one
//! consequence of the two-level shape that a flat "batch the flattened
//! list" model would get wrong, and the corpus pins it.
//!
//! **The batch key** is our [`BatchKey`]: the drawable program plus the
//! per-object uniform inventory. It stands in for the Reference's shader
//! id, whose hash is over exactly this material — program code, mobject
//! uniforms, `depth_test`, render primitive, textures
//! (`shader_wrapper.py:117`) — plus `stroke_behind`
//! (`shader_wrapper.py:354`). Float uniforms compare **bitwise**: two
//! objects whose `anti_alias_width` differs by an ulp are two batches,
//! exactly as two shader ids differ.
//!
//! **`depth_test` and `is_fixed_in_frame` partition, they do not reorder.**
//! Both live in the batch key, so depth-tested content can never share a
//! draw call with flat content and a frame-fixed overlay can never share
//! one with world content — but the *sequence* is still painter order over
//! the scene list. The depth test is a per-fragment operation inside that
//! sequence, not a re-sort of it (§10.4).
//!
//! **Two deliberate divergences (D5), both already ours:**
//!
//! - A mobject reachable twice inside one family (diamond composition) is
//!   drawn **once**: [`Stage::family`] deduplicates, where the Reference's
//!   `get_family` concatenates and would draw it twice — which double-
//!   composites transparent content. A mobject under two *separate scene
//!   roots added in one batch* is still drawn once per root: that is two
//!   placements in the scene, not one. A later singleton parent add moves
//!   its family out of the old placement first (`scene.py:333`).
//! - `z_index` on a *child* has no effect on order in either engine — the
//!   scene sort reads top-level members only — but here that is stated
//!   rather than discovered. `set_z_index(recurse = true)` still writes the
//!   whole family, because the value is per-object data and a child may be
//!   promoted to a root later.

use crate::mobject::RenderPrimitive;
use crate::stage::{Mob, Stage};
use crate::uniforms::Uniforms;

/// Which drawable program an object renders through — the Reference's
/// `str(type(m))` component of the batch key, reduced to what actually
/// changes the pipeline.
///
/// Vector paths, images, surfaces, and point clouds partition correctly by
/// construction rather than relying on each renderer adapter to remember the
/// concrete library class that produced them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgramKind {
    /// Filled and stroked quadratic paths — every `VMobject`.
    #[default]
    Vector,
    /// Fixed-grid or explicitly indexed surface triangles.
    Surface,
    /// Camera-facing radial point sprites.
    DotCloud,
    /// Textured image quads.
    Image,
}

impl RenderPrimitive {
    const fn program_kind(self) -> ProgramKind {
        match self {
            Self::Vector => ProgramKind::Vector,
            Self::SurfaceGrid { .. } | Self::TriangleMesh => ProgramKind::Surface,
            Self::DotCloud => ProgramKind::DotCloud,
            Self::ImageQuad => ProgramKind::Image,
        }
    }
}

/// What makes two adjacent drawn objects shareable in one draw call.
#[derive(Debug, Clone, Copy)]
pub struct BatchKey {
    /// The drawable program.
    pub program: ProgramKind,
    /// The per-object uniform inventory the program is parameterized by.
    pub uniforms: Uniforms,
}

impl BatchKey {
    /// The key an entry draws under.
    #[must_use]
    pub fn of(stage: &Stage, mob: Mob) -> Option<Self> {
        stage.get(mob).map(|entry| Self {
            program: entry.render_primitive().program_kind(),
            uniforms: *entry.uniforms(),
        })
    }
}

/// Bitwise on the float uniforms, structural on the rest — the shader-id
/// contract: same key ⇒ same draw call, and a difference of any kind
/// (however small) is a different call.
impl PartialEq for BatchKey {
    fn eq(&self, other: &Self) -> bool {
        let (a, b) = (&self.uniforms, &other.uniforms);
        self.program == other.program
            && a.is_fixed_in_frame.to_bits() == b.is_fixed_in_frame.to_bits()
            && a.shading
                .iter()
                .zip(&b.shading)
                .all(|(x, y)| x.to_bits() == y.to_bits())
            && a.clip_planes
                .iter()
                .flatten()
                .zip(b.clip_planes.iter().flatten())
                .all(|(x, y)| x.to_bits() == y.to_bits())
            && a.anti_alias_width.to_bits() == b.anti_alias_width.to_bits()
            && a.joint_type == b.joint_type
            && a.flat_stroke == b.flat_stroke
            && a.scale_stroke_with_zoom == b.scale_stroke_with_zoom
            && a.stroke_behind == b.stroke_behind
            && a.depth_test == b.depth_test
            && a.use_winding_fill == b.use_winding_fill
    }
}

/// The two passes a vector object draws, in the order it draws them —
/// `stroke_behind` swaps them (`shader_wrapper.py:277`). Lumen's compositor
/// consumes this as the within-object order; it is why `stroke_behind`
/// belongs in the batch key rather than being resolved later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassOrder {
    /// Fill, then stroke — the default.
    FillThenStroke,
    /// Stroke, then fill — `stroke_behind = true`.
    StrokeThenFill,
}

/// One object in the draw sequence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawItem {
    /// The object drawn.
    pub mob: Mob,
    /// The scene root whose placement put it here (a mobject under two
    /// roots appears once per root).
    pub root: Mob,
    /// Which render group it belongs to — group boundaries are batch
    /// boundaries.
    pub group: usize,
    /// Which draw call it belongs to. Adjacent items with equal keys inside
    /// one group share a batch.
    pub batch: usize,
    /// The key it draws under.
    pub key: BatchKey,
    /// The order of its own fill and stroke passes.
    pub passes: PassOrder,
}

/// The deterministic draw sequence for a scene: back to front, batched.
#[derive(Debug, Clone, Default)]
pub struct DrawPlan {
    items: Vec<DrawItem>,
    groups: usize,
    batches: usize,
}

impl DrawPlan {
    /// The sequence, back to front.
    #[must_use]
    pub fn items(&self) -> &[DrawItem] {
        &self.items
    }

    /// How many render groups the scene list formed.
    #[must_use]
    pub fn group_count(&self) -> usize {
        self.groups
    }

    /// How many draw calls the sequence needs.
    #[must_use]
    pub fn batch_count(&self) -> usize {
        self.batches
    }

    /// The drawn objects, in order — the ordering trace the fixtures
    /// compare (identity, not geometry).
    #[must_use]
    pub fn sequence(&self) -> Vec<Mob> {
        self.items.iter().map(|item| item.mob).collect()
    }

    /// The batch each object landed in, in order — the second half of the
    /// trace: *what* is drawn and *how many calls* it takes.
    #[must_use]
    pub fn batch_trace(&self) -> Vec<usize> {
        self.items.iter().map(|item| item.batch).collect()
    }
}

impl Stage {
    /// The scene's draw sequence (§8.5): the two-level model above, applied
    /// to the current scene list.
    ///
    /// Pure — no revision is touched, nothing is cached here. The retained
    /// render plan (§10.8, fm-gw7) is what caches this; keeping the model
    /// itself a pure function of scene state is what lets the cache be
    /// checked against it.
    #[must_use]
    pub fn draw_plan(&self) -> DrawPlan {
        let roots = self.roots().to_vec();
        // Level 1: adjacent scene members sharing (program, key, z_index)
        // form one render group.
        let mut groups: Vec<Vec<Mob>> = Vec::new();
        let mut previous: Option<(BatchKey, i32)> = None;
        for root in roots {
            let Some(key) = BatchKey::of(self, root) else {
                continue;
            };
            let z = self.z_index(root);
            let current = (key, z);
            let continues = previous
                .as_ref()
                .is_some_and(|(pk, pz)| *pk == current.0 && *pz == current.1);
            if continues {
                groups.last_mut().expect("a group exists").push(root);
            } else {
                groups.push(vec![root]);
            }
            previous = Some(current);
        }

        // Level 2: each group draws its members' families, depth-first,
        // pointless members skipped, adjacent equal keys batched — and a
        // batch never crosses a group boundary.
        let mut items = Vec::new();
        let mut batches = 0usize;
        for (group_index, group) in groups.iter().enumerate() {
            // A batch never crosses a group boundary.
            let mut open: Option<BatchKey> = None;
            for &root in group {
                for mob in self.family(root) {
                    let Some(entry) = self.get(mob) else { continue };
                    if entry.buffer.is_empty() {
                        continue; // family_members_with_points
                    }
                    if !entry.paints() {
                        // A bare Mobject's programless wrapper has its own
                        // shader id: it ends the batch and draws nothing.
                        open = None;
                        continue;
                    }
                    let key = BatchKey {
                        program: entry.render_primitive().program_kind(),
                        uniforms: *entry.uniforms(),
                    };
                    if open != Some(key) {
                        batches += 1;
                        open = Some(key);
                    }
                    items.push(DrawItem {
                        mob,
                        root,
                        group: group_index,
                        batch: batches - 1,
                        key,
                        passes: if key.uniforms.stroke_behind {
                            PassOrder::StrokeThenFill
                        } else {
                            PassOrder::FillThenStroke
                        },
                    });
                }
            }
        }
        DrawPlan {
            groups: groups.len(),
            batches,
            items,
        }
    }
}
