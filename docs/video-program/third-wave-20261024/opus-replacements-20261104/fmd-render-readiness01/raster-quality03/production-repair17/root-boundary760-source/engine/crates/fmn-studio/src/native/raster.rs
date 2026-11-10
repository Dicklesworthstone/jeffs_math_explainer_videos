//! Live worker output through Lumen's sole retained CPU renderer.

use fmn_frame::convert::rgba16f_to_rgba8;
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_hash::{Schema, Writer};
use fmn_render::engine::journal as render_engine_journal;
use fmn_render::{
    Binning, Camera, CameraConfig, RetainedFrameRenderer, RetainedFrameRendererConfig, ScreenMap,
};
use fmn_scene::{RenderBackendRecord, RenderBackendRole};

use crate::{
    DebugLayerSet, DebugOverlaySnapshot, FrameEncoding, FramePayload, FrameStream, InspectorLimits,
    InspectorSnapshot, InspectorView, ServiceError, WorkerResponse, protocol_digest,
};

use super::NativeSceneProgram;
use super::camera::CameraCapture;
use super::program::{execution_error, invalid};

pub(super) struct NativeRaster {
    renderer: RetainedFrameRenderer,
    rgba: FrameBuffer,
    camera: Option<CameraCapture>,
    base_camera: Option<CameraConfig>,
    rig_backend: Option<RenderBackendRecord>,
}

impl NativeRaster {
    pub(super) fn new(
        config: RetainedFrameRendererConfig,
        base_camera: Option<CameraConfig>,
    ) -> Result<Self, ServiceError> {
        let camera = base_camera
            .clone()
            .map(|camera| CameraCapture::new(camera, config))
            .transpose()?;
        let layout = FrameLayout::tight(
            PixelFormat::Rgba8,
            config.frame.viewport.width,
            config.frame.viewport.height,
        )
        .map_err(execution_error)?;
        Ok(Self {
            renderer: RetainedFrameRenderer::new(config).map_err(execution_error)?,
            rgba: FrameBuffer::new(layout),
            camera,
            base_camera,
            rig_backend: None,
        })
    }

    /// Bind a factory's canonical rig location into the stable renderer input
    /// contract. Per-frame pose is in the Stage/SceneState, not an accumulating
    /// stream of backend identities or a second renderer callback journal.
    pub(super) fn bind_camera(&mut self, index: Option<u64>) -> Result<(), ServiceError> {
        self.rig_backend = if let Some(index) = index {
            let camera = self
                .camera
                .as_ref()
                .ok_or_else(|| invalid("camera rig requires an explicit worker CameraConfig"))?;
            let mut writer = Writer::new(Schema::new(*b"FMRC", 1, 1, 0));
            writer
                .put_str("lumen-native-camera-rig-v1")
                .put_u64(index)
                .put_bytes(camera.backend.identity());
            Some(
                RenderBackendRecord::new(
                    RenderBackendRole::FrameStream,
                    writer.finish().map_err(execution_error)?,
                )
                .map_err(execution_error)?,
            )
        } else {
            None
        };
        Ok(())
    }

    fn sampled_camera(&self, program: &NativeSceneProgram) -> Result<Option<Camera>, ServiceError> {
        match program.camera_rig() {
            Some(rig) => {
                let base = self.base_camera.as_ref().ok_or_else(|| {
                    invalid("camera rig requires an explicit worker CameraConfig")
                })?;
                let config = rig
                    .sample(program.preview().stage(), base)
                    .map_err(execution_error)?;
                Ok(Some(Camera::new(config).map_err(execution_error)?))
            }
            None => Ok(self.camera.as_ref().map(|camera| camera.camera.clone())),
        }
    }

    pub(super) fn frame(
        &mut self,
        scene: &str,
        program: &NativeSceneProgram,
    ) -> Result<WorkerResponse, ServiceError> {
        if let Some(sample) = self.sampled_camera(program)? {
            let capture = self
                .camera
                .as_mut()
                .ok_or_else(|| invalid("missing native camera capture"))?;
            // Keep one monotone Camera revision. Freshly constructed Camera
            // values all start at revision 1 and must not alias retained keys.
            if capture.camera.frame() != sample.frame() {
                *capture.camera.frame_mut() = sample.frame().clone();
            }
            if capture.camera.light_source_position() != sample.light_source_position() {
                capture
                    .camera
                    .set_light_source_position(sample.light_source_position())
                    .map_err(execution_error)?;
            }
            self.renderer
                .render_with_camera(program.preview().stage(), &capture.camera)
                .map_err(execution_error)?;
        } else {
            self.renderer
                .render(program.preview().stage(), 0)
                .map_err(execution_error)?;
        }
        rgba16f_to_rgba8(self.renderer.frame(), &mut self.rgba).map_err(execution_error)?;
        let config = self.renderer.config();
        let width = config.frame.viewport.width;
        let height = config.frame.viewport.height;
        let bytes = fmn_codec::encode_rgba8(
            width,
            height,
            self.rgba.as_bytes(),
            fmn_codec::CompressionLevel::Fast,
        );
        let digest = protocol_digest(&bytes);
        Ok(WorkerResponse::Frame(FrameStream {
            scene: scene.to_owned(),
            frame_index: program.frame_index(),
            width,
            height,
            stride: 0,
            encoding: FrameEncoding::Png,
            payload: FramePayload::Pipe { bytes, digest },
            render_backends: vec![self.backend()?],
        }))
    }

    pub(super) fn backend(&self) -> Result<RenderBackendRecord, ServiceError> {
        if let Some(backend) = &self.rig_backend {
            return backend.try_clone().map_err(execution_error);
        }
        if let Some(camera) = &self.camera {
            return camera.backend.try_clone().map_err(execution_error);
        }
        let config = self.renderer.config();
        RenderBackendRecord::new(
            RenderBackendRole::FrameStream,
            render_engine_journal(config.engine, &config.frame, config.tiling),
        )
        .map_err(execution_error)
    }

    pub(super) fn inspect(
        &self,
        program: &NativeSceneProgram,
        frame_count: u64,
        max_bytes: usize,
        input_revision: Option<u64>,
    ) -> Result<Vec<u8>, ServiceError> {
        let limits = InspectorLimits {
            max_json_bytes: max_bytes,
            ..InspectorLimits::default()
        };
        let mut snapshot =
            InspectorSnapshot::capture(program.preview().stage(), program.spans(), limits)
                .map_err(execution_error)?;
        let config = self.renderer.config();
        let map = self
            .sampled_camera(program)?
            .map_or(config.frame.map, |camera| ScreenMap {
                scale: 1.0 / camera.pixel_size(),
                origin: [
                    f64::from(camera.pixel_width()) * 0.5,
                    f64::from(camera.pixel_height()) * 0.5,
                ],
                y_up: true,
            });
        snapshot.view = Some(
            InspectorView::new(
                program.frame_index(),
                frame_count,
                program.preview().scene().fps(),
                config.frame.viewport,
                map,
                self.camera.is_none(),
            )
            .map_err(execution_error)?,
        );
        if let Some(view) = &mut snapshot.view {
            view.input_revision = input_revision;
        }
        snapshot.to_json(limits).map_err(execution_error)
    }

    pub(super) fn overlay(
        &mut self,
        program: &NativeSceneProgram,
        layers: DebugLayerSet,
        max_bytes: usize,
    ) -> Result<Vec<u8>, ServiceError> {
        if self.camera.is_some() {
            return Err(invalid(
                "native camera overlays require projected diagnostics; affine overlays are unavailable",
            ));
        }
        self.renderer
            .render(program.preview().stage(), 0)
            .map_err(execution_error)?;
        let config = self.renderer.config();
        let mut binning = Binning::build(
            self.renderer.plan(),
            config.frame.viewport,
            config.tiling,
            config.frame.map,
        )
        .map_err(execution_error)?;
        binning
            .prune_occluded(self.renderer.plan())
            .map_err(execution_error)?;
        let limits = InspectorLimits {
            max_json_bytes: max_bytes,
            ..InspectorLimits::default()
        };
        DebugOverlaySnapshot::capture(
            program.preview().stage(),
            Some((&binning, config.frame.viewport)),
            layers,
            limits,
        )
        .map_err(execution_error)?
        .to_json(limits)
        .map_err(execution_error)
    }
}
