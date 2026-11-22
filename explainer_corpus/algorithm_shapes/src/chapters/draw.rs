//! Native graph primitives. The same coordinates and road identities persist
//! when later chapters change the arithmetic. No simulation runs in callbacks.
use crate::model::{PLACES, ROADS};
use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;

pub const POINTS: [[f64; 2]; 4] = [[-5.3, 0.0], [-3.2, 1.55], [-3.2, -1.55], [-1.1, 0.0]];
const LABEL_OFFSETS: [[f64; 2]; 5] = [[-0.2, 0.3], [-0.2, -0.3], [0.3, 0.0], [0.2, 0.3], [0.2, -0.3]];
pub const ROUTES: [&[usize]; 3] = [&[0, 3], &[1, 4], &[0, 2, 4]];

pub fn show(stage: &mut Stage<'_>, mobs: &[Mob], seconds: f64) -> fmn::Result<()> {
    let mut prepared = Vec::new();
    for &mob in mobs {
        let a = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?.rt(seconds);
        prepared.push(stage.prepare(a)?);
    }
    stage.play_prepared(prepared)?;
    Ok(())
}

pub fn hide(stage: &mut Stage<'_>, mobs: &[Mob], seconds: f64) -> fmn::Result<()> {
    let mut prepared = Vec::new();
    for &mob in mobs {
        let a = fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?.rt(seconds);
        prepared.push(stage.prepare(a)?);
    }
    stage.play_prepared(prepared)?;
    Ok(())
}

pub fn dot(stage: &mut Stage<'_>, point: Vec3, radius: f64, color: Srgb) -> Mob {
    stage.arena_mut().add(
        Dot::new()
            .point(point)
            .radius(radius)
            .style(Style::default().fill(color, 1.0).stroke(color, 0.0, 1.0))
            .build(),
    )
}

pub fn route_color(index: usize) -> Srgb {
    explainer_kit::palette::hex(["#FF8A80", "#79BFFF", "#F29EC7"][index])
}

pub struct Map {
    pub nodes: Vec<Mob>,
    pub names: Vec<Mob>,
    pub roads: Vec<Mob>,
    pub costs: Vec<Mob>,
    pub points: [Vec3; 4],
}

impl Map {
    /// Small copies use the same five-road geometry with no unreadable labels.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, shift: [f64; 2], scale: f64, labels: bool) -> fmn::Result<Self> {
        let points = POINTS.map(|p| v(p[0] * scale + shift[0], p[1] * scale + shift[1]));
        let mut nodes = Vec::new();
        let mut names = Vec::new();
        let mut roads = Vec::new();
        let mut costs = Vec::new();
        for (i, &point) in points.iter().enumerate() {
            nodes.push(dot(stage, point, 0.10 * scale, Palette::ink()));
            if labels {
                let dy = if i == 2 { -0.42 } else { 0.42 };
                names.push(kit.text(stage, PLACES[i], 29.0, Palette::ink(), v(point[0], point[1] + dy))?);
            }
        }
        for (i, road) in ROADS.iter().enumerate() {
            roads.push(
                stage.arena_mut().add(
                    Arrow::new(points[road.from], points[road.to])
                        .buff(0.16 * scale)
                        .thickness(2.5 * scale)
                        .style(
                            Style::default()
                                .fill(Palette::dim(), 1.0)
                                .stroke(Palette::dim(), 2.5 * scale, 1.0),
                        )
                        .build()?,
                ),
            );
            if labels {
                let from = points[road.from];
                let to = points[road.to];
                let at = v(
                    (from[0] + to[0]) * 0.5 + LABEL_OFFSETS[i][0],
                    (from[1] + to[1]) * 0.5 + LABEL_OFFSETS[i][1],
                );
                costs.push(kit.text(stage, &road.value.to_string(), 31.0, Palette::gold(), at)?);
            }
        }
        Ok(Self {
            nodes,
            names,
            roads,
            costs,
            points,
        })
    }

    pub fn all(&self) -> Vec<Mob> {
        self.nodes
            .iter()
            .chain(&self.names)
            .chain(&self.roads)
            .chain(&self.costs)
            .copied()
            .collect()
    }

    pub fn travel(&self, stage: &mut Stage<'_>, route: &[usize], color: Srgb) -> fmn::Result<()> {
        let token = dot(stage, self.points[ROADS[route[0]].from], 0.14, color);
        show(stage, &[token], 0.2)?;
        for &index in route {
            let road = ROADS[index];
            let a = token
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.85),
                    ..AnimateArgs::default()
                })?
                .move_to(self.points[road.to], ORIGIN)?;
            stage.play(a)?;
        }
        hide(stage, &[token], 0.2)
    }
}

pub fn route_total(route: &[usize]) -> u64 {
    route.iter().map(|&i| ROADS[i].value).sum()
}

/// The question stays visible; no answer is created until the timer ends.
pub fn prediction(kit: &Kit, stage: &mut Stage<'_>, seconds: u32, at: Vec3) -> fmn::Result<()> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    let caption = kit.text(
        stage,
        "Pause for more time",
        24.0,
        Palette::dim(),
        v(at[0], at[1] - 0.55),
    )?;
    show(stage, &[caption], 0.2)?;
    for remaining in (1..=seconds).rev() {
        let number = kit.text(stage, &remaining.to_string(), 38.0, Palette::ink(), at)?;
        show(stage, &[number], 0.1)?;
        stage.wait(0.8)?;
        hide(stage, &[number], 0.1)?;
    }
    hide(stage, &[caption], 0.2)
}
