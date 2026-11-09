//! cargo run -p fmn --example camera_pipeline_orbit -- /new/output/directory
//! Camera motion, geometry and lighting share the ordinary scene clock.

use fmn::prelude::*;
use fmn::rendering::{CameraConfig, render_camera};
use fmn::scene::CameraRig;

struct Orbit {
    rig: CameraRig,
    target: CameraConfig,
}

impl SceneConstruct for Orbit {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add(Cube::new(1.6).color(BLUE))?;
        let animation = self.rig.animate_to(stage.scene_mut(), &self.target)?;
        stage.play_prepared_with(
            vec![Box::new(animation)],
            PlayOverrides {
                run_time: Some(2.0),
                ..PlayOverrides::default()
            },
        )?;
        Ok(())
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .unwrap_or_else(|| "camera-orbit".into());
    let mut options = RenderOptions::new(std::path::PathBuf::from(output))?;
    options.config.camera.resolution = (320, 180);
    options.config.camera.fps = 24;
    options.config.sizes.frame_height = 4.0;
    let report = render_camera(
        |scene, base| {
            let rig = CameraRig::new(scene, base)?;
            let mut target = base.clone();
            target
                .frame
                .set_euler_angles(Some(0.7), Some(0.9), None)
                .map_err(SceneError::from)?;
            target.light_source_position = [5.0, 4.0, 8.0];
            Ok((Orbit { rig, target }, rig))
        },
        options,
    )?;
    println!(
        "Published {} frames to {}",
        report.artifact.frame_count,
        report.artifact.path.display()
    );
    Ok(())
}
