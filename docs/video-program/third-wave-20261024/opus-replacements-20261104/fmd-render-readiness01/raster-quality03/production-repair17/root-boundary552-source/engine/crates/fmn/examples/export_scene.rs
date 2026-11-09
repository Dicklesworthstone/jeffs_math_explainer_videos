//! cargo run -p fmn --example export_scene -- scene.fmtl
//! Then pass scene.fmtl to the standalone fmn CLI or an FMTL player.
use fmn::prelude::*;

struct MovingCircle;

impl SceneConstruct for MovingCircle {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let circle = stage.add(Circle::new().radius(0.7).color(BLUE))?;
        stage.set_fill(circle, Some(BLUE), Some(1.0), None, true);
        stage.play(circle.animate().shift(RIGHT)?)?;
        stage.wait(0.5)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .unwrap_or_else(|| "scene.fmtl".into());
    let report = export_bundle(&mut MovingCircle, output, BundleExportOptions::new()?)?;
    println!(
        "{}: {} frames, {} bytes, sha256 {}",
        report.output.display(),
        report.frame_count,
        report.bytes,
        report.digest,
    );
    Ok(())
}
