//! A standalone picture probe; not a narrated chapter or release artifact.
use fmn::prelude::*;

const FORMULAS: &[&str] = &[
    r"a_i,\quad b_i",
    r"\partial_y f_2\,dy",
    r"-\partial_y f_2\,dy\wedge dx\wedge dz=+\partial_y f_2\,dx\wedge dy\wedge dz",
    r"d\omega=(\sum_i\partial_i f_i)\,dx^1\wedge\cdots\wedge dx^n",
    r"\int_{a_i}^{b_i}\partial_i f_i\,dx^i=f_i(b_i)-f_i(a_i)",
];

struct Probe;
impl SceneConstruct for Probe {
    fn tex_preflight(&self) -> Vec<TypesetRequest<'_>> {
        FORMULAS.iter().map(|s| TypesetRequest::math(s)).collect()
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        for (i, source) in FORMULAS.iter().enumerate() {
            let formula = Tex::new(source).font_size(27.0).build(stage.tex_engine()?)?;
            let mob = stage.add(formula)?;
            stage.move_to(mob, [0.0, 2.8 - 1.35 * i as f64, 0.0], ORIGIN);
        }
        stage.wait(1.0)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args().nth(1).ok_or("output directory required")?;
    let mut options = RenderOptions::new(output)?;
    options.config.camera.resolution = (960, 540);
    options.config.camera.fps = 1;
    let report = render(&mut Probe, options)?;
    println!("{} frames: {}", report.artifact.frame_count, report.artifact.path.display());
    Ok(())
}
