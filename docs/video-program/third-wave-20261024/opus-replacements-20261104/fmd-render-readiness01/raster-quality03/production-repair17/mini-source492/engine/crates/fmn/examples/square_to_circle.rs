use fmn::prelude::*;

#[derive(Default)]
struct SquareToCircle;

const FORMULA: &str = r"\int_0^\infty e^{-x^2}\,dx = \frac{\sqrt{\pi}}{2}";

impl SceneConstruct for SquareToCircle {
    fn tex_preflight(&self) -> Vec<TypesetRequest<'_>> {
        vec![TypesetRequest::math(FORMULA)]
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let square = stage.add(Square::new().side_length(2.0).color(BLUE))?;
        let label = Tex::new(FORMULA).build(stage.tex_engine()?)?;
        let label = stage.add(label)?;
        stage.next_to(label, square, UP, DEFAULT_MOBJECT_TO_MOBJECT_BUFF, ORIGIN);

        let write_label = write(stage.arena(), label);
        stage.play((show_creation(square), write_label))?;
        let restyle = square.animate().rotate(PI / 4.0)?.set_opacity(0.5)?;
        stage.play(restyle)?;

        let circle = stage
            .arena_mut()
            .add(Circle::new().radius(1.0).color(YELLOW));
        stage.play(Transform::new(square, circle))?;
        stage.wait(1.0)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "media/square_to_circle".to_owned());
    let mut options = RenderOptions::new(output)?;
    options.config.camera.resolution = (640, 360);
    options.config.camera.fps = 30;
    let report = render(&mut SquareToCircle, options)?;
    println!(
        "Published {} frames to {}",
        report.artifact.frame_count,
        report.artifact.path.display()
    );
    Ok(())
}
