#[cfg(feature = "metal")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io;
    use std::time::Duration;

    use fmn_core::color::Srgb;
    use fmn_mobject::Stage;
    use fmn_render::{Binning, FrameConfig, MonoTable, RenderPlan, ScreenMap, Tiling, Viewport};
    use fmn_studio::{
        PresentOutcome, PreviewFallback, PreviewRoute, StudioPreviewConfig, StudioPreviewOutput,
        StudioPreviewRenderer, StudioPreviewRoute,
    };

    const WIDTH: u32 = 640;
    const HEIGHT: u32 = 360;
    const FRAMES: usize = 90;

    let mut preview = StudioPreviewRenderer::new(StudioPreviewConfig::new(
        WIDTH,
        HEIGHT,
        "FrankenManim native Metal preview",
        1,
    ))?;
    if preview.route() != StudioPreviewRoute::NativeMetal {
        return Err(io::Error::other(format!(
            "native Metal route unavailable: {:?}",
            preview.route()
        ))
        .into());
    }

    let stage = Stage::new();
    let map = ScreenMap {
        scale: 1.0,
        origin: [f64::from(WIDTH) / 2.0, f64::from(HEIGHT) / 2.0],
        y_up: true,
    };
    let viewport = Viewport {
        width: WIDTH,
        height: HEIGHT,
    };
    let mut plan = RenderPlan::new();
    plan.sync(&stage, 0)?;
    let mono = MonoTable::build(&plan, map)?;
    let binning = Binning::build(&plan, viewport, Tiling::default(), map)?;

    let mut presented = 0usize;
    let mut occluded = 0usize;
    let mut device = None;
    let mut render_occupancy = None;
    let mut presentation_occupancy = None;
    let mut last_backend_digest = None;
    for frame_index in 0..FRAMES {
        let phase = u8::try_from((frame_index * 255) / FRAMES)?;
        let config = FrameConfig::new(
            viewport,
            map,
            Srgb::from_rgb8(
                24u8.saturating_add(phase / 3),
                32u8.saturating_add(phase / 5),
                72u8.saturating_add(phase / 2),
            )
            .to_linear(1.0),
        );
        let output = preview.render(&plan, &mono, &binning, config)?;
        let StudioPreviewOutput::Native(report) = output else {
            return Err(io::Error::other(format!(
                "native route demoted during frame {frame_index}: {:?}",
                preview.route()
            ))
            .into());
        };
        if report.frame_pixel_readback_bytes() != 0 {
            return Err(io::Error::other("native frame performed pixel readback").into());
        }
        device.get_or_insert_with(|| report.metal.device.clone());
        render_occupancy.get_or_insert([
            report.metal.threads_per_threadgroup,
            report.metal.max_threads_per_threadgroup,
            report.metal.thread_execution_width,
        ]);
        presentation_occupancy.get_or_insert(report.presentation);
        last_backend_digest = Some(report.backend_digest());
        match report.outcome {
            PresentOutcome::Presented => presented += 1,
            PresentOutcome::Occluded => occluded += 1,
        }
        std::thread::sleep(Duration::from_millis(16));
    }
    if presented == 0 {
        return Err(io::Error::other("native surface never acquired a drawable").into());
    }
    preview.close()?;
    let fallback_config =
        FrameConfig::new(viewport, map, Srgb::from_rgb8(24, 32, 72).to_linear(1.0));
    let fallback_output = preview.render(&plan, &mono, &binning, fallback_config)?;
    let StudioPreviewOutput::Stream(fallback_frame) = fallback_output else {
        return Err(io::Error::other("closed native surface did not select CPU stream").into());
    };
    let fallback_reason = match &fallback_frame.route {
        PreviewRoute::FastCpu(PreviewFallback::BackendFailure(reason))
            if reason.contains("closed") =>
        {
            reason.clone()
        }
        route => {
            return Err(io::Error::other(format!(
                "closed native surface reported the wrong fallback: {route:?}"
            ))
            .into());
        }
    };
    let fallback_bytes = fallback_frame.frame.as_bytes().len();

    let render_occupancy =
        render_occupancy.ok_or_else(|| io::Error::other("renderer occupancy was not observed"))?;
    let presentation_occupancy = presentation_occupancy
        .ok_or_else(|| io::Error::other("presentation occupancy was not observed"))?;
    let backend_digest =
        last_backend_digest.ok_or_else(|| io::Error::other("backend digest was not observed"))?;
    println!(
        "presented {presented} native frames ({occluded} occluded) on {} with zero frame-pixel \
         readback; render occupancy {}/{}/{}; presentation occupancy {}x{}/{}/{}; backend {}; \
         closed-surface fallback produced {fallback_bytes} CPU-stream bytes ({fallback_reason})",
        device.as_deref().unwrap_or("unknown Metal device"),
        render_occupancy[0],
        render_occupancy[1],
        render_occupancy[2],
        presentation_occupancy.threads_per_threadgroup[0],
        presentation_occupancy.threads_per_threadgroup[1],
        presentation_occupancy.max_threads_per_threadgroup,
        presentation_occupancy.thread_execution_width,
        backend_digest,
    );
    Ok(())
}

#[cfg(not(feature = "metal"))]
fn main() {
    eprintln!("native_preview requires `--features metal`");
}
