//! Automatic layout lint: a second pair of eyes on every draft (ported from
//! the explainer skill's template).
//!
//! With a [`Linter`] on the [`Kit`](crate::Kit), every narration beat
//! (`kit.say`) and every `kit.hold` checks the scene as it stands and records:
//! - `outside`: a visible top-level mobject crossing the frame edge (cut off);
//! - `edge`: one inside the frame but within [`MARGIN`] of an edge (cramped,
//!   and player controls cover the bottom strip);
//! - `overlap`: two visible texts (everything the kit's text, Tex and live
//!   helpers built, also inside groups) whose boxes overlap by more than
//!   [`MAX_OVERLAP`] of the smaller one;
//! - `tight`: a text within [`TEXT_MARGIN`] of the left, right or top edge
//!   (title-safe: a text wants more room than a shape; the bottom keeps
//!   [`MARGIN`], where the lower third stands);
//! - `crossed` and `behind`: a stroke of another mobject (a border, a link, a
//!   curve, an arrow) passes through the core of a glyph of a visible text,
//!   painted over it (`crossed`: struck through) or under it (`behind`: the
//!   text stands on the line). A frame the text sits inside does not cross it
//!   (its sides are outside the glyphs), nor does a stroke as faint as a
//!   background grid ([`FAINT_INK`]), nor one hidden from the text by an opaque
//!   fill painted over it (the disc under a node's numeral). Seen in Video J: a
//!   box edge cutting its own number, a lit link striking the numerals of its
//!   nodes, a tag standing on a border. One finding for each text and kind;
//! - `openfill`: an open path (a bracket, a polyline) painted with a fill,
//!   which fills the area between the path and its chord into a solid bar
//!   (`set_opacity(.., true)` on a bracket, the pitfall in `fmn-opacity`).
//!
//! `outside` and `edge` leave out what is not on the screen at all (a shape
//! parked far outside the frame to come in later) and a backdrop that covers
//! the whole frame.
//!
//! "Visible" means painted with alpha > 0: vector strokes and fills
//! (including a parent's own paint, such as an arrow's shaft), dot clouds
//! (padded by their radius), surfaces and images. Each problem (kind + the
//! mobjects involved) is reported once, at the first checkpoint it appears,
//! with the number of later checkpoints that still showed it. A [`Linter`]
//! made with [`Linter::sampled`] also looks every [`SAMPLE_S`] seconds of the
//! running scene (the `--lint` of the CLI does), so what is only there in the
//! middle of a beat is seen too: the checkpoints are the ends of beats. A
//! sampled look reports a problem only after it has stood for [`PERSIST_S`]:
//! a picture changes over a second or so, its old and new parts passing through
//! each other. Contact and cue sheets remain the final word: the lint sees
//! boxes, strokes and painter's order, not pixels. The template's calibration
//! on the eight Hoeffding's D
//! chapters: no false overlaps, seven real findings (one label cut off at the
//! bottom). Assumes the 16:9 frame (8 units tall).

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use crate::kit::Registered;

/// Safe-area margin inside the frame edge (scene units): closer is `edge`.
pub const MARGIN: f64 = 0.15;
/// Overlap share of the smaller box that counts as a collision.
pub const MAX_OVERLAP: f64 = 0.10;
/// A text closer than this to the left, right or top edge of the frame is `tight`: 3% of the frame's height. The
/// defects of Video J's dense read stood 0.10 to 0.19 from the edge; at 0.30 the other E-K drafts would give
/// thirty-odd findings, most of them a label at 0.27.
pub const TEXT_MARGIN: f64 = 0.25;
/// Seconds of scene time between two looks at a running scene ([`Linter::sampled`]).
pub const SAMPLE_S: f64 = 0.2;
/// Seconds a problem has to stand, look after look, before a sampled lint reports it.
pub const PERSIST_S: f64 = 1.0;
/// How deep inside a glyph's box a stroke has to reach to cross it, as a share of the box's smaller side.
pub const GLYPH_CORE: f64 = 0.15;
/// How much of a glyph's core a stroke has to run through, as a share of the core's smaller side, to cross it: a
/// leader line that starts at the edge of its label does not.
pub const INSIDE_MIN: f64 = 0.3;
/// A fill with every point's alpha at least this hides what is under it (see [`opaque_fill_core`]): a label's
/// card is a little translucent and still hides the line it stands on.
pub const OPAQUE: f32 = 0.8;
/// A stroke fainter than this (see [`stroke_contrast`]) crosses nothing: it is a background texture.
pub const FAINT_INK: f64 = 0.35;
/// The least area (square units) between an open path and its chord that makes a fill on it a finding.
pub const OPEN_FILL_AREA: f64 = 0.02;
/// Stroke width (as fmn stores it) to scene units.
const STROKE_UNIT: f64 = 0.01;

/// An axis-aligned box: (lower-left, upper-right).
pub type Box2 = ([f64; 2], [f64; 2]);

/// `outside` (crosses the frame), `edge` (inside but within the margin), or
/// nothing for a box comfortably inside a `±xr × ±yr` frame.
pub fn edge_kind((lo, hi): Box2, xr: f64, yr: f64) -> Option<&'static str> {
    let clipped = lo[0] < -xr || hi[0] > xr || lo[1] < -yr || hi[1] > yr;
    let (sx, sy) = (xr - MARGIN, yr - MARGIN);
    if clipped {
        Some("outside")
    } else if lo[0] < -sx || hi[0] > sx || lo[1] < -sy || hi[1] > sy {
        Some("edge")
    } else {
        None
    }
}

/// The share of the smaller box that two boxes overlap (0 when they only
/// touch, or overlap by a sliver under 0.05 units high).
pub fn overlap_share(a: Box2, b: Box2) -> f64 {
    let w = a.1[0].min(b.1[0]) - a.0[0].max(b.0[0]);
    let h = a.1[1].min(b.1[1]) - a.0[1].max(b.0[1]);
    let smaller = area(a).min(area(b));
    if w <= 0.0 || h <= 0.05 || smaller <= 0.0 {
        0.0
    } else {
        w * h / smaller
    }
}

fn area((lo, hi): Box2) -> f64 {
    (hi[0] - lo[0]) * (hi[1] - lo[1])
}

struct Finding {
    row: String,
    /// Later looks at which the same problem was still there.
    again: usize,
}

/// A problem one look found, before it is a finding.
struct Problem {
    key: String,
    kind: String,
    detail: String,
}

/// A problem seen by sampled looks that has not stood long enough yet.
struct Pending {
    first: f64,
    at: String,
    kind: String,
    detail: String,
}

/// The lint of one chapter.
#[derive(Default)]
pub struct Linter {
    findings: RefCell<Vec<Finding>>,
    /// Problem key (kind + mobjects) -> index into `findings`.
    seen: RefCell<HashMap<String, usize>>,
    /// Look at the running scene every [`SAMPLE_S`] too (see [`Linter::sampled`]).
    sampling: bool,
    /// The narration line being said, and the scene seconds since it began (names a sampled look).
    line: RefCell<String>,
    line_start: Cell<f64>,
    since_line: Cell<f64>,
    since_look: Cell<f64>,
    /// Scene seconds the sampled looks have seen so far.
    now: Cell<f64>,
    /// While a sampled look runs: where its problems go, instead of straight into the findings.
    sink: RefCell<Option<Vec<Problem>>>,
    /// Problems the sampled looks saw and have not yet seen for [`PERSIST_S`].
    pending: RefCell<HashMap<String, Pending>>,
    /// The mobject whose updater does the looking, while it is on the stage.
    probe: Cell<Option<Mob>>,
    /// Mobjects whose strokes are meant to cross text (a strike-through, a highlighted diagonal): see
    /// [`Linter::allow_crossing`].
    allowed: RefCell<Vec<Mob>>,
}

impl Linter {
    /// A lint that looks at the checkpoints only: every `say` and `hold`.
    pub fn new() -> Self {
        Self::default()
    }

    /// A lint that looks at the checkpoints and every [`SAMPLE_S`] seconds of the running scene in between,
    /// so a stroke that crosses a numeral for two seconds in the middle of a beat is seen. The kit puts a
    /// bare mobject on the stage whose updater does the looking (see `Kit::say`).
    pub fn sampled() -> Self {
        Self {
            sampling: true,
            ..Self::default()
        }
    }

    /// Does this lint look at the running scene?
    pub fn is_sampling(&self) -> bool {
        self.sampling
    }

    /// The narration line `id` begins now, at `scene_s` seconds of the scene: the next sampled looks are named
    /// after it and say when they were.
    pub fn begin_line(&self, id: &str, scene_s: f64) {
        *self.line.borrow_mut() = id.to_owned();
        self.line_start.set(scene_s);
        self.since_line.set(0.0);
    }

    /// The strokes of `mob` (and its family) are meant to cross text: a strike-through of a wrong answer, a
    /// diagonal drawn over the entries it marks. The lint cannot know that; the chapter says so.
    pub fn allow_crossing(&self, mob: Mob) {
        self.allowed.borrow_mut().push(mob);
    }

    /// The probe is not on the stage, or has lost its updater (a wipe such as `fade_all` clears every top-level
    /// mobject's updaters and fades it out): the kit has to put a new one back.
    pub fn needs_probe(&self, arena: &Arena) -> bool {
        self.sampling
            && !self
                .probe
                .get()
                .is_some_and(|p| arena.contains(p) && arena.roots().contains(&p) && arena.family_updater_kinds(p).1 > 0)
    }

    /// Remember the mobject that carries the updater calling [`Linter::tick`].
    pub fn set_probe(&self, probe: Mob) {
        self.probe.set(Some(probe));
    }

    /// `dt` seconds of the scene have passed: look when [`SAMPLE_S`] have since the last look. A sampled look
    /// reports a problem only once it has stood for [`PERSIST_S`] in consecutive looks: a picture changes
    /// over a second or so, with its old and new parts passing through each other, and that is no layout.
    pub fn tick(&self, arena: &Arena, texts: &[Registered], dt: f64) {
        self.now.set(self.now.get() + dt);
        self.since_line.set(self.since_line.get() + dt);
        self.since_look.set(self.since_look.get() + dt);
        if self.since_look.get() + 1e-9 >= SAMPLE_S {
            self.since_look.set(0.0);
            let (line, off) = (self.line.borrow(), self.since_line.get());
            let at = format!("{line} +{off:.1}s @{:.1}s", self.line_start.get() + off);
            *self.sink.borrow_mut() = Some(Vec::new());
            self.check(arena, texts, &at);
            let found = self.sink.borrow_mut().take().unwrap_or_default();
            self.observe(found, &at);
        }
    }

    /// Settle the problems one sampled look found against those it found before.
    fn observe(&self, found: Vec<Problem>, at: &str) {
        let now = self.now.get();
        let keys: HashSet<&str> = found.iter().map(|p| p.key.as_str()).collect();
        self.pending.borrow_mut().retain(|k, _| keys.contains(k.as_str()));
        for p in found {
            if self.seen.borrow().contains_key(&p.key) {
                self.record(p.key, at, &p.kind, &p.detail); // already reported: one more look that still shows it
                continue;
            }
            let first = self
                .pending
                .borrow_mut()
                .entry(p.key.clone())
                .or_insert_with(|| Pending {
                    first: now,
                    at: at.to_owned(),
                    kind: p.kind.clone(),
                    detail: p.detail.clone(),
                })
                .first;
            if now - first + 1e-9 >= PERSIST_S
                && let Some(pending) = self.pending.borrow_mut().remove(&p.key)
            {
                self.record(p.key, &pending.at, &pending.kind, &pending.detail);
            }
        }
    }

    pub fn count(&self) -> usize {
        self.findings.borrow().len()
    }

    /// The findings as TSV rows: `checkpoint  kind  detail`.
    pub fn report(&self) -> String {
        self.findings
            .borrow()
            .iter()
            .map(|f| match f.again {
                0 => format!("{}\n", f.row),
                n => format!("{} (still there at {n} later looks)\n", f.row),
            })
            .collect()
    }

    /// Check the scene now against the kit's registered texts; `at` names
    /// the checkpoint ("before 01_2", "hold").
    pub fn check(&self, arena: &Arena, texts: &[Registered], at: &str) {
        let short = |s: &str| s.chars().take(32).collect::<String>();
        let label_of = |root: Mob| -> String {
            if let Some(t) = texts.iter().find(|t| t.mob == root) {
                return format!("{:?}", short(&t.source));
            }
            let family = arena.family(root);
            match texts.iter().find(|t| family.contains(&t.mob)) {
                Some(t) => format!("a group holding {:?}", short(&t.source)),
                None => "an unlabeled shape".to_owned(),
            }
        };
        let mut on_stage: HashSet<Mob> = HashSet::new();
        let mut families: Vec<(Mob, Vec<Mob>)> = Vec::new();
        for &m in arena.roots() {
            let family = arena.family(m);
            on_stage.extend(family.iter().copied());
            families.push((m, family));
            let Some(b) = visible_box(arena, m) else {
                continue;
            };
            if let Some(kind) = edge_kind(b, FRAME_X_RADIUS, FRAME_Y_RADIUS) {
                // not on the screen at all (parked, to come in later), or a backdrop that covers all of it
                let frame = ([-FRAME_X_RADIUS, -FRAME_Y_RADIUS], [FRAME_X_RADIUS, FRAME_Y_RADIUS]);
                let backdrop = b.0[0] <= frame.0[0] + 0.02
                    && b.0[1] <= frame.0[1] + 0.02
                    && b.1[0] >= frame.1[0] - 0.02
                    && b.1[1] >= frame.1[1] - 0.02;
                if boxes_apart(b, frame) || backdrop {
                    continue;
                }
                self.record(
                    format!("{kind}|{m:?}"),
                    at,
                    kind,
                    &format!(
                        "{} spans x {:.2}..{:.2}, y {:.2}..{:.2}; frame ±{:.2} × ±{:.2}, safe area ±{:.2} × ±{:.2}",
                        label_of(m),
                        b.0[0],
                        b.1[0],
                        b.0[1],
                        b.1[1],
                        FRAME_X_RADIUS,
                        FRAME_Y_RADIUS,
                        FRAME_X_RADIUS - MARGIN,
                        FRAME_Y_RADIUS - MARGIN
                    ),
                );
            }
        }
        let shown: Vec<(Mob, String, Box2)> = texts
            .iter()
            .filter(|t| on_stage.contains(&t.mob))
            .filter_map(|t| visible_box(arena, t.mob).map(|b| (t.mob, short(&t.source), b)))
            .collect();
        for (i, (ma, la, a)) in shown.iter().enumerate() {
            for (mb, lb, b) in &shown[i + 1..] {
                let share = overlap_share(*a, *b);
                if share <= MAX_OVERLAP {
                    continue;
                }
                // a text nested in another registered text is part of it
                if arena.family(*ma).contains(mb) || arena.family(*mb).contains(ma) {
                    continue;
                }
                // keyed by what they say, not by handle: a live readout is rebuilt (a new handle) every frame
                self.record(
                    format!("overlap|{la}|{lb}"),
                    at,
                    "overlap",
                    &format!("{la:?} and {lb:?} overlap ({:.0}% of the smaller)", 100.0 * share),
                );
            }
        }
        self.check_tight(&shown, at);
        self.check_crossed(arena, texts, &shown, &families, &label_of, at);
        self.check_open_fills(arena, texts, &families, &label_of, at);
    }

    /// `tight`: a text nearer than [`TEXT_MARGIN`] to the left, right or top edge, but not yet within
    /// [`MARGIN`] (which `edge` reports for every shape).
    fn check_tight(&self, shown: &[(Mob, String, Box2)], at: &str) {
        let (xr, yr) = (FRAME_X_RADIUS, FRAME_Y_RADIUS);
        for (_, label, (lo, hi)) in shown {
            let (side, room) = [("left", lo[0] + xr), ("right", xr - hi[0]), ("top", yr - hi[1])]
                .into_iter()
                .fold(("", f64::INFINITY), |a, s| if s.1 < a.1 { s } else { a });
            if (MARGIN..TEXT_MARGIN).contains(&room) {
                self.record(
                    format!("tight|{label}|{side}"),
                    at,
                    "tight",
                    &format!(
                        "{label:?} stands {room:.2} from the {side} edge; a text wants {TEXT_MARGIN:.2} (title-safe)"
                    ),
                );
            }
        }
    }

    /// `crossed`: a visible stroke of anything that is not text passes through the core of a glyph of a
    /// shown text. A frame the text sits inside does not (its sides are outside the glyphs).
    fn check_crossed(
        &self,
        arena: &Arena,
        texts: &[Registered],
        shown: &[(Mob, String, Box2)],
        families: &[(Mob, Vec<Mob>)],
        label_of: &dyn Fn(Mob) -> String,
        at: &str,
    ) {
        let in_text: HashSet<Mob> = texts.iter().flat_map(|t| arena.family(t.mob)).collect();
        // the strokes the chapter declared to cross text are no business of this rule
        let declared: HashSet<Mob> = self.allowed.borrow().iter().flat_map(|&m| arena.family(m)).collect();
        // painter's order: the scene list back to front, each family depth first (fmn's draw order)
        let mut order: HashMap<Mob, usize> = HashMap::new();
        for (_, members) in families {
            for &m in members {
                order.insert(m, order.len());
            }
        }
        let mut strokes: Vec<(Mob, usize, Vec<Piece>, Box2)> = Vec::new();
        for (root, members) in families {
            for &m in members.iter().filter(|m| !in_text.contains(m) && !declared.contains(m)) {
                let pieces = stroke_pieces(arena, m);
                if let Some(bounds) = pieces_box(&pieces)
                    && stroke_contrast(arena, m) >= FAINT_INK
                {
                    strokes.push((*root, order[&m], pieces, bounds));
                }
            }
        }
        if strokes.is_empty() {
            return;
        }
        // an opaque fill painted over a stroke hides it where it covers it (the disc under a node's numeral)
        let occluders: Vec<(usize, Box2)> = families
            .iter()
            .flat_map(|(_, members)| members.iter().copied())
            .filter(|m| !in_text.contains(m))
            .filter_map(|m| opaque_fill_core(arena, m).map(|b| (order[&m], b)))
            .collect();
        for (text, label, tbox) in shown {
            let glyphs: Vec<(Mob, Box2)> = arena
                .family(*text)
                .into_iter()
                .filter_map(|g| member_box(arena, g).map(|b| (g, b)))
                .collect();
            let cores: Vec<Box2> = glyphs.iter().filter_map(|&(_, b)| glyph_core(b)).collect();
            if cores.is_empty() {
                continue;
            }
            // the text's own place in the painter's order: its lowest glyph
            let first = glyphs
                .iter()
                .filter_map(|(g, _)| order.get(g).copied())
                .min()
                .unwrap_or(usize::MAX);
            // one finding for each text, of each kind, whichever stroke it is and wherever on the text
            let (cx, cy) = ((tbox.0[0] + tbox.1[0]) / 2.0, (tbox.0[1] + tbox.1[1]) / 2.0);
            let place = format!("{:.1},{:.1}", (cx * 2.0).round() / 2.0, (cy * 2.0).round() / 2.0);
            let mut reported = [false; 2];
            for (root, rank, pieces, sbox) in &strokes {
                // over the text it strikes through it; under it, it is a line the text stands on
                let under = *rank < first;
                if reported[usize::from(under)] || boxes_apart(*tbox, *sbox) {
                    continue;
                }
                // how much of the stroke lies inside a glyph's core, not counting what something opaque painted after
                // the stroke covers; a leader line that only touches the text's edge stays under `INSIDE_MIN`
                let hit = cores.iter().find_map(|&core| {
                    let need = INSIDE_MIN * (core.1[0] - core.0[0]).min(core.1[1] - core.0[1]);
                    let (mut inside, mut best) = (0.0_f64, (0.0_f64, [0.0_f64; 2]));
                    for &(a, b, _) in pieces {
                        let Some((t0, t1)) = segment_clip(a, b, core, 0.0) else {
                            continue;
                        };
                        let len = (b[0] - a[0]).hypot(b[1] - a[1]) * (t1 - t0);
                        let t = (t0 + t1) / 2.0;
                        let at = [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])];
                        let hidden = occluders.iter().any(|&(o, (lo, hi))| {
                            o > *rank && (lo[0]..=hi[0]).contains(&at[0]) && (lo[1]..=hi[1]).contains(&at[1])
                        });
                        if !hidden {
                            inside += len;
                            if len > best.0 {
                                best = (len, at);
                            }
                        }
                    }
                    (inside > 0.0 && inside >= need).then_some((best.1[0], best.1[1]))
                });
                if let Some((x, y)) = hit {
                    reported[usize::from(under)] = true;
                    let (kind, how) = if under {
                        ("behind", "under")
                    } else {
                        ("crossed", "over")
                    };
                    self.record(
                        format!("{kind}|{label}|{place}"),
                        at,
                        kind,
                        &format!(
                            "a stroke of {} (x {:.2}..{:.2}, y {:.2}..{:.2}), painted {how} {label:?}, passes through it near ({x:.2}, {y:.2})",
                            label_of(*root),
                            sbox.0[0],
                            sbox.1[0],
                            sbox.0[1],
                            sbox.1[1]
                        ),
                    );
                }
            }
        }
    }

    /// `openfill`: an open path (a bracket) with a visible fill; the fill closes it by its chord and paints the
    /// area between the two, a solid bar where a thin line was drawn.
    fn check_open_fills(
        &self,
        arena: &Arena,
        texts: &[Registered],
        families: &[(Mob, Vec<Mob>)],
        label_of: &dyn Fn(Mob) -> String,
        at: &str,
    ) {
        let in_text: HashSet<Mob> = texts.iter().flat_map(|t| arena.family(t.mob)).collect();
        for (root, members) in families {
            for &m in members.iter().filter(|m| !in_text.contains(m)) {
                let Some((area, (lo, hi))) = open_fill(arena, m) else {
                    continue;
                };
                if area > OPEN_FILL_AREA {
                    self.record(
                        format!("openfill|{}|{:.1},{:.1}", label_of(*root), (lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0),
                        at,
                        "openfill",
                        &format!(
                            "{} paints a fill on an open path: {area:.2} square units between it and its chord (x {:.2}..{:.2}, y {:.2}..{:.2})",
                            label_of(*root),
                            lo[0],
                            hi[0],
                            lo[1],
                            hi[1]
                        ),
                    );
                }
            }
        }
    }

    fn record(&self, key: String, at: &str, kind: &str, detail: &str) {
        if let Some(sink) = self.sink.borrow_mut().as_mut() {
            sink.push(Problem {
                key,
                kind: kind.to_owned(),
                detail: detail.to_owned(),
            });
            return; // a sampled look: the caller settles it against the earlier looks
        }
        let mut seen = self.seen.borrow_mut();
        let mut findings = self.findings.borrow_mut();
        match seen.get(&key) {
            Some(&i) => findings[i].again += 1,
            None => {
                seen.insert(key, findings.len());
                findings.push(Finding {
                    row: format!("{at}\t{kind}\t{detail}"),
                    again: 0,
                });
            }
        }
    }
}

/// The union of the own-point boxes of every family member that actually
/// paints (alpha > 0). A parent with its own paint counts (an arrow's shaft
/// holds its tip as a child); a plain group has no paint fields and does not.
pub fn visible_box(arena: &Arena, root: Mob) -> Option<Box2> {
    arena
        .family(root)
        .into_iter()
        .filter_map(|m| member_box(arena, m))
        .reduce(union_box)
}

fn union_box((a, z): Box2, (lo, hi): Box2) -> Box2 {
    ([a[0].min(lo[0]), a[1].min(lo[1])], [z[0].max(hi[0]), z[1].max(hi[1])])
}

/// The box of the own points of one mobject, when it paints (alpha > 0) and has points.
pub fn member_box(arena: &Arena, m: Mob) -> Option<Box2> {
    let e = arena.get(m)?;
    if !e.paints() {
        return None;
    }
    let b = &e.buffer;
    let any_alpha = |col: Option<Vec<f32>>| col.is_some_and(|c| c.as_chunks::<4>().0.iter().any(|c| c[3] > 0.0));
    let lit = match (b.read_column("fill_rgba"), b.read_column("stroke_rgba")) {
        (None, None) => match (b.read_column("rgba"), b.read_column("opacity")) {
            (Some(rgba), _) => any_alpha(Some(rgba)), // dot clouds, surfaces
            (None, Some(opacity)) => opacity.iter().any(|&o| o > 0.0), // images
            (None, None) => true,                     // unknown primitive: assume it shows
        },
        (fill, stroke) => {
            let width = b.read_column("stroke_width").unwrap_or_default();
            any_alpha(fill)
                || stroke.is_some_and(|s| {
                    s.as_chunks::<4>()
                        .0
                        .iter()
                        .zip(&width)
                        .any(|(c, w)| c[3] > 0.0 && *w > 0.0)
                })
        }
    };
    if !lit {
        return None;
    }
    let points = b.read_column("point")?;
    let pad = b
        .read_column("radius")
        .map_or(0.0, |r| r.iter().fold(0.0_f64, |a, &x| a.max(f64::from(x))));
    let placement = e.placement();
    points
        .as_chunks::<3>()
        .0
        .iter()
        .map(|p| {
            let q = placement.apply_point([f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]);
            ([q[0] - pad, q[1] - pad], [q[0] + pad, q[1] + pad])
        })
        .reduce(union_box)
}

/// One visible straight piece of a stroke: its two ends and half the stroke's width (scene units).
pub type Piece = ([f64; 2], [f64; 2], f64);

/// The visible pieces of the stroke of the mobject `m` alone (not its children): each quadratic curve cut into
/// four pieces, a curve with alpha and width at one of its ends only, no jump between subpaths.
pub fn stroke_pieces(arena: &Arena, m: Mob) -> Vec<Piece> {
    let Some(e) = arena.get(m) else { return Vec::new() };
    if !e.paints() {
        return Vec::new();
    }
    let b = &e.buffer;
    let (Some(points), Some(stroke)) = (b.read_column("point"), b.read_column("stroke_rgba")) else {
        return Vec::new();
    };
    let width = b.read_column("stroke_width").unwrap_or_default();
    let (pts, cols) = (points.as_chunks::<3>().0, stroke.as_chunks::<4>().0);
    let lit = |i: usize| cols.get(i).is_some_and(|c| c[3] > 0.0) && width.get(i).is_some_and(|w| *w > 0.0);
    let placement = e.placement();
    let at = |i: usize| {
        let p = pts[i];
        let q = placement.apply_point([f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]);
        [q[0], q[1]]
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i + 2 < pts.len() {
        // a curve whose handle sits on its start and whose end is elsewhere is the jump to the next subpath
        let jump = pts[i] == pts[i + 1] && pts[i] != pts[i + 2];
        if !jump && (lit(i) || lit(i + 2)) {
            let w = width
                .get(i)
                .copied()
                .unwrap_or(0.0)
                .max(width.get(i + 2).copied().unwrap_or(0.0));
            let half = 0.5 * STROKE_UNIT * f64::from(w);
            let (p0, p1, p2) = (at(i), at(i + 1), at(i + 2));
            let mut prev = p0;
            for k in 1..=4 {
                let t = f64::from(k) / 4.0;
                let u = 1.0 - t;
                let q = [
                    u * u * p0[0] + 2.0 * u * t * p1[0] + t * t * p2[0],
                    u * u * p0[1] + 2.0 * u * t * p1[1] + t * t * p2[1],
                ];
                out.push((prev, q, half));
                prev = q;
            }
        }
        i += 2;
    }
    out
}

/// How much the stroke of `m` stands out from the background: the brightest lit point's alpha times its
/// luminance above the background's (0 to 1; the palette's `faint` grid is 0.19, `dim` 0.51, `ink` 0.87). A line
/// as faint as a background grid is a texture a title may stand on.
pub fn stroke_contrast(arena: &Arena, m: Mob) -> f64 {
    let Some(e) = arena.get(m) else { return 0.0 };
    let b = &e.buffer;
    let Some(stroke) = b.read_column("stroke_rgba") else {
        return 0.0;
    };
    let width = b.read_column("stroke_width").unwrap_or_default();
    // the corpus background, #0E1116, in the same gamma-encoded luminance
    let background = 0.2126 * (14.0 / 255.0) + 0.7152 * (17.0 / 255.0) + 0.0722 * (22.0 / 255.0);
    stroke
        .as_chunks::<4>()
        .0
        .iter()
        .zip(&width)
        .filter(|(c, w)| c[3] > 0.0 && **w > 0.0)
        .map(|(c, _)| {
            let y = 0.2126 * f64::from(c[0]) + 0.7152 * f64::from(c[1]) + 0.0722 * f64::from(c[2]);
            f64::from(c[3]) * (y - background).max(0.0)
        })
        .fold(0.0, f64::max)
}

/// For a mobject whose fill is opaque everywhere (a disc under a numeral, a card behind a caption): the middle of its
/// box, shrunk by a sixth of its size on every side, which the fill certainly covers. What it covers hides a
/// stroke painted under it.
pub fn opaque_fill_core(arena: &Arena, m: Mob) -> Option<Box2> {
    let e = arena.get(m)?;
    let fill = e.buffer.read_column("fill_rgba")?;
    let alphas = fill.as_chunks::<4>().0;
    if alphas.len() < 3 || alphas.iter().any(|c| c[3] < OPAQUE) {
        return None;
    }
    let (lo, hi) = member_box(arena, m)?;
    let (dx, dy) = ((hi[0] - lo[0]) / 6.0, (hi[1] - lo[1]) / 6.0);
    Some(([lo[0] + dx, lo[1] + dy], [hi[0] - dx, hi[1] - dy]))
}

/// The box that holds all of `pieces`, padded by their half widths.
pub fn pieces_box(pieces: &[Piece]) -> Option<Box2> {
    pieces
        .iter()
        .map(|&(a, b, r)| {
            (
                [a[0].min(b[0]) - r, a[1].min(b[1]) - r],
                [a[0].max(b[0]) + r, a[1].max(b[1]) + r],
            )
        })
        .reduce(union_box)
}

/// True when the boxes share no point.
pub fn boxes_apart(a: Box2, b: Box2) -> bool {
    a.1[0] < b.0[0] || b.1[0] < a.0[0] || a.1[1] < b.0[1] || b.1[1] < a.0[1]
}

/// The inner part of a glyph's box: shrunk on every side by [`GLYPH_CORE`] of its smaller side. A stroke has to
/// reach this to cross the glyph; a frame round the text, or a line that only touches the ink's box, does not.
pub fn glyph_core((lo, hi): Box2) -> Option<Box2> {
    let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    let d = GLYPH_CORE * w.min(h);
    Some(([lo[0] + d, lo[1] + d], [hi[0] - d, hi[1] - d]))
}

/// Does the segment `a..b`, thickened by `reach` on both sides, meet the box? (Liang-Barsky clipping against the
/// box grown by `reach`.)
pub fn segment_meets_box(a: [f64; 2], b: [f64; 2], bounds: Box2, reach: f64) -> bool {
    segment_clip(a, b, bounds, reach).is_some()
}

/// The part of the segment `a..b` (thickened by `reach`) inside the box, as the parameters `(t0, t1)` along it.
pub fn segment_clip(a: [f64; 2], b: [f64; 2], (lo, hi): Box2, reach: f64) -> Option<(f64, f64)> {
    let (lo, hi) = ([lo[0] - reach, lo[1] - reach], [hi[0] + reach, hi[1] + reach]);
    let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
    for axis in 0..2 {
        let (start, d) = (a[axis], b[axis] - a[axis]);
        if d.abs() < 1e-12 {
            if start < lo[axis] || start > hi[axis] {
                return None;
            }
        } else {
            let (mut ta, mut tb) = ((lo[axis] - start) / d, (hi[axis] - start) / d);
            if ta > tb {
                std::mem::swap(&mut ta, &mut tb);
            }
            t0 = t0.max(ta);
            t1 = t1.min(tb);
            if t0 > t1 {
                return None;
            }
        }
    }
    Some((t0, t1))
}

/// For a mobject with a visible fill and an open subpath: the largest area between such a subpath (through its
/// anchors) and the chord that closes it, and the box of that subpath. A closed shape has none.
pub fn open_fill(arena: &Arena, m: Mob) -> Option<(f64, Box2)> {
    let e = arena.get(m)?;
    let b = &e.buffer;
    let fill = b.read_column("fill_rgba")?;
    if !fill.as_chunks::<4>().0.iter().any(|c| c[3] > 0.0) {
        return None;
    }
    let points = b.read_column("point")?;
    let pts = points.as_chunks::<3>().0;
    let placement = e.placement();
    let at = |i: usize| {
        let p = pts[i];
        let q = placement.apply_point([f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]);
        [q[0], q[1]]
    };
    let mut best: Option<(f64, Box2)> = None;
    let mut sub: Vec<[f64; 2]> = Vec::new();
    let mut flush = |sub: &mut Vec<[f64; 2]>| {
        if sub.len() >= 3 {
            let (first, last) = (sub[0], sub[sub.len() - 1]);
            if (first[0] - last[0]).hypot(first[1] - last[1]) > 1e-3 {
                let twice: f64 = (0..sub.len())
                    .map(|k| {
                        let (p, q) = (sub[k], sub[(k + 1) % sub.len()]);
                        p[0] * q[1] - q[0] * p[1]
                    })
                    .sum();
                let area = twice.abs() / 2.0;
                let bounds = sub
                    .iter()
                    .map(|&p| (p, p))
                    .reduce(union_box)
                    .unwrap_or(([0.0; 2], [0.0; 2]));
                if best.is_none_or(|(a, _)| area > a) {
                    best = Some((area, bounds));
                }
            }
        }
        sub.clear();
    };
    let mut i = 0;
    while i + 2 < pts.len() {
        if sub.is_empty() {
            sub.push(at(i));
        }
        if pts[i] == pts[i + 1] && pts[i] != pts[i + 2] {
            flush(&mut sub); // the jump to the next subpath
            sub.push(at(i + 2));
        } else {
            sub.push(at(i + 2));
        }
        i += 2;
    }
    flush(&mut sub);
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_at_the_thresholds() {
        let (xr, yr) = (FRAME_X_RADIUS, FRAME_Y_RADIUS);
        let b = |x0: f64, x1: f64| ([x0, -1.0], [x1, 1.0]);
        // just inside the safe area, just inside the margin, just outside
        assert_eq!(edge_kind(b(-1.0, xr - MARGIN - 0.001), xr, yr), None);
        assert_eq!(edge_kind(b(-1.0, xr - MARGIN + 0.001), xr, yr), Some("edge"));
        assert_eq!(edge_kind(b(-1.0, xr + 0.001), xr, yr), Some("outside"));
        assert_eq!(edge_kind(([0.0, -yr - 0.01], [1.0, 0.0]), xr, yr), Some("outside"));
    }

    #[test]
    fn overlap_at_the_thresholds() {
        // a 1 × 1 box against one sliding over it: 9% and 11% of the smaller
        let a = ([0.0, 0.0], [1.0, 1.0]);
        let shifted = |dx: f64| ([1.0 - dx, 0.0], [2.0 - dx, 1.0]);
        assert!((overlap_share(a, shifted(0.09)) - 0.09).abs() < 1e-9);
        assert!(overlap_share(a, shifted(0.09)) <= MAX_OVERLAP);
        assert!(overlap_share(a, shifted(0.11)) > MAX_OVERLAP);
        assert_eq!(overlap_share(a, shifted(0.0)), 0.0, "touching is not overlapping");
        // a sliver under 0.05 high never counts
        assert_eq!(overlap_share(a, ([0.5, 0.97], [1.5, 1.5])), 0.0);
    }

    #[test]
    fn a_segment_meets_a_box_it_crosses_or_touches_with_its_width() {
        let b = ([0.0, 0.0], [1.0, 1.0]);
        assert!(segment_meets_box([-1.0, 0.5], [2.0, 0.5], b, 0.0), "straight through");
        assert!(
            segment_meets_box([0.5, -1.0], [0.5, 2.0], b, 0.0),
            "through the other way"
        );
        assert!(segment_meets_box([0.5, 0.5], [0.6, 0.6], b, 0.0), "wholly inside");
        assert!(segment_meets_box([-1.0, -1.0], [2.0, 2.0], b, 0.0), "the diagonal");
        assert!(!segment_meets_box([-1.0, 1.5], [2.0, 1.5], b, 0.0), "above");
        assert!(!segment_meets_box([1.5, -1.0], [1.5, 2.0], b, 0.0), "beside");
        assert!(!segment_meets_box([-1.0, 2.0], [2.0, 3.0], b, 0.0), "slanting past");
        // a thick stroke reaches what its centre line misses
        assert!(!segment_meets_box([-1.0, 1.05], [2.0, 1.05], b, 0.0));
        assert!(segment_meets_box([-1.0, 1.05], [2.0, 1.05], b, 0.06));
        // a segment that is a point
        assert!(segment_meets_box([0.5, 0.5], [0.5, 0.5], b, 0.0));
        assert!(!segment_meets_box([3.0, 3.0], [3.0, 3.0], b, 0.0));
    }

    #[test]
    fn a_glyphs_core_leaves_the_rim_to_a_frame() {
        // 0.3 wide, 0.5 tall: the core is 0.045 in from every side
        let core = glyph_core(([0.0, 0.0], [0.3, 0.5])).unwrap();
        assert!((core.0[0] - 0.045).abs() < 1e-12 && (core.1[0] - 0.255).abs() < 1e-12);
        assert!((core.0[1] - 0.045).abs() < 1e-12 && (core.1[1] - 0.455).abs() < 1e-12);
        // a frame 0.05 outside the glyph does not meet it; a line through the middle does
        let frame_side = ([-0.05, -0.05], [-0.05, 0.55]);
        assert!(!segment_meets_box(frame_side.0, frame_side.1, core, 0.02));
        assert!(segment_meets_box([-1.0, 0.25], [1.0, 0.25], core, 0.02));
        // no box, no core
        assert!(glyph_core(([0.0, 0.0], [0.0, 1.0])).is_none());
    }

    #[test]
    fn boxes_apart_and_pieces_boxes() {
        let a = ([0.0, 0.0], [1.0, 1.0]);
        assert!(boxes_apart(a, ([1.1, 0.0], [2.0, 1.0])));
        assert!(boxes_apart(a, ([0.0, -2.0], [1.0, -0.1])));
        assert!(!boxes_apart(a, ([1.0, 1.0], [2.0, 2.0])), "touching is not apart");
        let pieces: Vec<Piece> = vec![([0.0, 0.0], [1.0, 0.0], 0.1), ([1.0, 0.0], [1.0, 2.0], 0.1)];
        let (lo, hi) = pieces_box(&pieces).unwrap();
        assert!((lo[0] + 0.1).abs() < 1e-12 && (hi[0] - 1.1).abs() < 1e-12 && (hi[1] - 2.1).abs() < 1e-12);
        assert!(pieces_box(&[]).is_none());
    }
}
