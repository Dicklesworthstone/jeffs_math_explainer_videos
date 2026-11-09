//! `cargo run --locked -p fmn --example native_cube -- media/native_cube`
use fmn::prelude::*;

struct CubeAndMarker;

impl SceneConstruct for CubeAndMarker {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let cube = stage.add(Cube::new(1.6).color(BLUE))?;
        let marker = stage.add(Circle::new().radius(0.2).color(YELLOW))?;
        stage.set_fill(marker, Some(YELLOW), Some(1.0), None, true);
        stage.shift(marker, [-2.0, 0.0, 0.0]);
        stage.play(cube.animate().shift(RIGHT)?)?;
        stage.wait(0.5)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "media/native_cube".to_owned());
    let mut options = RenderOptions::new(output)?;
    options.config.camera.resolution = (640, 360);
    options.config.camera.fps = 30;
    options.config.sizes.frame_height = 5.0;
    let mut camera = options.camera_config()?;
    camera.frame.set_euler_angles(Some(0.5), Some(0.8), None)?;
    options.camera = Some(camera);
    let report = render(&mut CubeAndMarker, options)?;
    println!(
        "Published {} frames to {}",
        report.artifact.frame_count,
        report.artifact.path.display()
    );
    Ok(())
}
