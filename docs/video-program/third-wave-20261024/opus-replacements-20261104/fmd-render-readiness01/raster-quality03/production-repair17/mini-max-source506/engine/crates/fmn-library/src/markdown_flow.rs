//! Measured inline boxes for Markdown. Text and Tex still own all shaping and
//! formula layout; this module only seats their immutable results on baselines.
use crate::vmobject::VMobject;

pub(super) struct InlineBox {
    pub geometry: VMobject,
    pub advance: f64,
    pub height: f64,
    pub depth: f64,
    pub space: bool,
    pub hard_break: bool,
}

impl InlineBox {
    pub(super) fn line_break() -> Self {
        Self {
            geometry: VMobject::new(),
            advance: 0.0,
            height: 0.0,
            depth: 0.0,
            space: false,
            hard_break: true,
        }
    }
}

type MeasuredLine = (Vec<(VMobject, f64)>, f64, f64);

pub(super) fn flow(boxes: Vec<InlineBox>, width: Option<f64>, leading: f64) -> VMobject {
    let mut lines: Vec<MeasuredLine> = Vec::new();
    let (mut line, mut x, mut height, mut depth) = (Vec::new(), 0.0, leading * 0.8, leading * 0.2);
    let mut pending_space = 0.0;
    for item in boxes {
        if item.hard_break {
            lines.push((std::mem::take(&mut line), height, depth));
            (x, height, depth, pending_space) = (0.0, leading * 0.8, leading * 0.2, 0.0);
            continue;
        }
        if item.space {
            if !line.is_empty() {
                pending_space += item.advance;
            }
            continue;
        }
        if !line.is_empty()
            && pending_space > 0.0
            && width.is_some_and(|w| x + pending_space + item.advance > w)
        {
            lines.push((std::mem::take(&mut line), height, depth));
            (x, height, depth, pending_space) = (0.0, leading * 0.8, leading * 0.2, 0.0);
        }
        x += pending_space;
        pending_space = 0.0;
        line.push((item.geometry, x));
        x += item.advance;
        height = height.max(item.height);
        depth = depth.max(item.depth);
    }
    if !line.is_empty() {
        lines.push((line, height, depth));
    }
    let mut children = Vec::new();
    let mut top = 0.0;
    for (line, height, depth) in lines {
        let baseline = top - height;
        children.extend(
            line.into_iter()
                .map(|(geometry, x)| geometry.shifted([x, baseline, 0.0])),
        );
        top = baseline - depth - leading * 0.2;
    }
    VMobject::new().with_children(children)
}
