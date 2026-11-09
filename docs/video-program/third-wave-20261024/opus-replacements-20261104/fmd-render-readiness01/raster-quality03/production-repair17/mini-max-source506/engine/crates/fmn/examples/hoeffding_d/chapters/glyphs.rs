//! A typesetting probe: every Text and Tex string the video intends to
//! use, laid out on one still, with per-string failures reported by name.

use fmn::prelude::*;

use crate::kit::{Kit, Palette, ink_style, v};

pub struct Glyphs {
    pub kit: Kit,
}

const TEXT: &[&str] = &[
    "Pearson  Spearman  Kendall  Hoeffding's D",
    "ρ τ α θ ≈ × − → · • … ≤ ≥ “quotes”",
    "✓ ✗",
    "IBM Plex Sans: 0123456789 ρ τ ≈ × − →",
];

const TEX: &[&str] = &[
    r"\rho \quad \tau \quad \alpha \quad \theta \quad \varrho \quad \vartheta",
    r"r_s \quad \hat\rho \quad \mathrm{\rho}",
    r"Q_i = 1 + \#\{\, j : R_j < R_i,\ S_j < S_i \,\}",
    r"\binom{N}{4} = \frac{N(N-1)(N-2)(N-3)}{24} \approx 2.6\times 10^{13}",
    r"D_1 = \sum_i (Q_i-1)(Q_i-2) \qquad -0.5 \le D \le 1",
    r"\text{Pearson } r \qquad \text{Spearman } \rho \qquad \text{Kendall } \tau",
];

impl SceneConstruct for Glyphs {
    fn name(&self) -> &str {
        "glyphs"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let mut y = 3.5;
        for (i, s) in TEXT.iter().enumerate() {
            let mut t = Text::new(s)
                .font_size(34.0)
                .style(ink_style(Palette::ink()));
            if i == 3 {
                t = t.font("IBM Plex Sans");
            }
            match t.build(&self.kit.book) {
                Ok(t) => {
                    let m = stage.add(t)?;
                    stage.move_to(m, v(-6.8, y), LEFT);
                }
                Err(e) => eprintln!("TEXT FAIL {s:?}: {e}"),
            }
            y -= 0.8;
        }
        for s in TEX {
            match Tex::new(s)
                .font_size(36.0)
                .style(ink_style(Palette::ink()))
                .build(stage.tex_engine()?)
            {
                Ok(t) => {
                    let m = stage.add(t)?;
                    stage.move_to(m, v(-6.8, y), LEFT);
                }
                Err(e) => eprintln!("TEX FAIL {s:?}: {e}"),
            }
            y -= 0.85;
        }
        stage.wait(1.0)?;
        Ok(())
    }
}
