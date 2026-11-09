//! File-output stage overlap over the production Lumen and runtime boundaries.
//!
//! Only owned render data and Reel reservations enter these workers. They do
//! not attach to CPython, traverse a Stage, evaluate an updater or read a live
//! NumPy view. Publication remains a separate owner-thread finish operation.

use fmn_frame::FrameBuffer;
use fmn_output::{FrameReservation, OrderedEmitter};
use fmn_render::PreparedCameraFrame;
use fmn_runtime::{ExecutionPlan, FrameStream, FrameStreamError, PipelineStages, TeamPlan};
use std::sync::Mutex;

pub(super) type Stream = FrameStream<Work, String>;

pub(super) struct Work {
    pub(super) frame: PreparedCameraFrame,
    pub(super) destination: FrameReservation,
}

pub(super) struct Rasterized {
    raw: FrameBuffer,
    destination: FrameReservation,
}

struct Stages {
    // The runtime has one converter. Keeping its scratch here reuses the same
    // conversion kernels and buffer across frames, including NV12/P010/BGRA.
    rgba8_scratch: Mutex<Option<FrameBuffer>>,
    raw_pool: Mutex<Vec<FrameBuffer>>,
}

impl PipelineStages for Stages {
    type Frame = Work;
    type Prepared = Work;
    type Rasterized = Rasterized;
    type Output = FrameReservation;
    type Error = String;

    fn prepare(&self, work: Work, _: &TeamPlan) -> Result<Work, String> {
        // Record synchronization already ran on the live arena's owner. The
        // shared ThreeDJob builds camera-bound raster data on a native worker.
        Ok(work)
    }

    fn rasterize(&self, work: Work, team: &TeamPlan) -> Result<Rasterized, String> {
        let Work { frame, destination } = work;
        let layout = frame.layout().map_err(|error| format!("lumen: {error}"))?;
        let mut raw = self.raw_pool.lock().map_err(|_| "raw frame pool poisoned")?
            .pop().unwrap_or_else(|| FrameBuffer::new(layout.clone()));
        if raw.layout() != &layout {
            return Err("lumen: frozen frame changed output geometry".into());
        }
        frame.render_into(team.threads(), &mut raw)
            .map_err(|error| format!("lumen: {error}"))?;
        Ok(Rasterized { raw, destination })
    }

    fn convert(&self, rasterized: Rasterized, _: &TeamPlan) -> Result<FrameReservation, String> {
        let Rasterized { raw, mut destination } = rasterized;
        let mut scratch = self.rgba8_scratch.lock().map_err(|_| "conversion scratch poisoned")?;
        super::portal_video::convert_frame(&raw, destination.frame_mut(), scratch.as_mut())
            .map_err(|error| format!("reel conversion: {error}"))?;
        drop(scratch);
        self.raw_pool.lock().map_err(|_| "raw frame pool poisoned")?.push(raw);
        Ok(destination)
    }
}

pub(super) fn reserve(stream: &mut Stream)
    -> Result<fmn_runtime::FramePermit<'_, Work>, FrameStreamError<String>>
{
    stream.reserve()
}

pub(super) fn start(plan: ExecutionPlan, scratch: Option<FrameBuffer>)
    -> Result<Stream, FrameStreamError<String>>
{
    FrameStream::new(plan, Stages {
        rgba8_scratch: Mutex::new(scratch), raw_pool: Mutex::new(Vec::new()),
    }, |_, reservation| reservation.publish().map_err(|error| format!("reel: {error}")))
}

/// Cancel the sink first so a blocked reservation/emission cannot hold the
/// pipeline join hostage. Finalizers are native and may run without the GIL.
pub(super) fn cancel(stream: Option<Stream>, emitter: Option<OrderedEmitter>) {
    if let Some(emitter) = &emitter { emitter.cancel(); }
    if let Some(stream) = stream { let _ = stream.cancel(); }
    if let Some(emitter) = emitter { let _ = emitter.finish(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn worker_inputs_contain_only_send_native_resources() {
        fn send<T: Send + 'static>() {}
        fn sync<T: Sync>() {}
        send::<Work>();
        send::<Rasterized>();
        send::<Stream>();
        sync::<Stages>();
    }

    #[test]
    fn production_frame_pipeline_acceptance() {
        crate::with_python_test_module("native file frame pipeline", |py, _module, globals| {
            use pyo3::types::PyDictMethods as _;
            globals.set_item("__file__", concat!(env!("CARGO_MANIFEST_DIR"), "/tests/frame_pipeline.py")).unwrap();
            let source = std::ffi::CString::new(include_str!("../tests/frame_pipeline.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("native pipeline pixel, clock, ownership and cancellation acceptance");
        });
    }
}
