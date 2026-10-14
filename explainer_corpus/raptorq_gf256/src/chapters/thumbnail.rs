//! The thumbnail (one still, 1280×720): the byte field's whole times table
//! with its rows and columns in click order, so every product's colour is
//! set by its total of clicks and the table is diagonal stripes (12_4);
//! over it, the title and the secret in chalk, `3 × 7 = 9` (01_2), with the
//! bits that make it so beneath, `011 × 111 = 1001`. Every number comes from
//! `model.rs`: the table is `mul` on every pair, the product and the bits too.

use explainer_kit::props::RowStrips;
use explainer_kit::{BACKGROUND, Kit, Palette, hex, v};
use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
use fmn::prelude::*;

use super::draw::{Role, flat};
use super::hook::{TITLE_FACTORS, chalk};
use super::times_table::palette;
use crate::model::{FOLD, exps, mul};

/// The thumbnail's scene, for the explainer's `thumbnail` field.
pub fn make(kit: Kit) -> Box<dyn SceneConstruct> {
    Box::new(Thumbnail { kit })
}

pub struct Thumbnail {
    pub kit: Kit,
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str {
        "thumbnail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (w, h) = (2.0 * FRAME_X_RADIUS, 2.0 * FRAME_Y_RADIUS);

        // the table in click order, edge to edge: a x b for every pair of the
        // 255 nonzero bytes, rows and columns taken round the byte clock
        let exp = exps();
        let pal = palette();
        let rows: Vec<Vec<Srgb>> = exp
            .iter()
            .map(|&a| exp.iter().map(|&b| pal[usize::from(mul(a, b, FOLD))]).collect())
            .collect();
        let n = exp.len() as f64;
        let table = RowStrips::build(stage, v(-FRAME_X_RADIUS, FRAME_Y_RADIUS), (w / n, h / n), &rows)?;
        for &m in &table.rows {
            // crisp cells: no antialiasing seam between strips (spike S-3)
            stage.set_anti_alias_width(m, 0.5, false);
        }
        stage.add_to_scene(table.group)?;

        // a veil over the stripes, and a slate behind the words, so the chalk carries
        let veil = Rectangle::new()
            .width(w)
            .height(h)
            .style(flat(hex(BACKGROUND), 0.5))
            .build()?;
        stage.add(veil)?;
        let slate = Rectangle::new()
            .width(11.6)
            .height(5.7)
            .corner_radius(0.3)
            .style(flat(hex(BACKGROUND), 0.84))
            .build()?
            .moved_to(v(0.0, -0.05));
        stage.add(slate)?;

        // the title's words, then the secret in chalk, then its bits
        let words = kit.text_styled(
            stage,
            "The Arithmetic Where",
            92.0,
            Palette::ink(),
            v(0.0, 1.95),
            true,
            false,
        )?;
        stage.add_to_scene(words)?;
        let eq = chalk(&kit, stage, 250.0, v(0.0, -0.15))?;
        stage.add_to_scene(eq)?;
        let (a, b) = TITLE_FACTORS;
        let product = mul(a, b, FOLD);
        let bits = kit.tex(
            stage,
            &format!(r"{a:03b} \times {b:03b} = {product:b}"),
            64.0,
            Role::ex_or(),
            &[],
            v(0.0, -2.2),
        )?;
        stage.add_to_scene(bits)?;

        // one second at the thumbnail's 1 fps: the one captured frame
        stage.wait(1.0)?;
        Ok(())
    }
}
