//! Scene.add_sound → native sound assets → Reel mix, with decode receipts.
//!
//! The portal owns no decoder, resampler, process, or audio publication path.
//! Capture the host configuration before authored lifecycle callbacks, then
//! consume the same rational cue requests as the native scene runtime.

use super::*;
use fmn_output::{AudioDecodeError, AudioDecodeLimits, AudioDecodeReport, AudioDecoder};

pub(crate) struct PortalAudioInput {
    path: PathBuf,
    report: AudioDecodeReport,
    channels: u16,
    sample_rate: u32,
    sample_frames: u64,
}

pub(crate) struct PortalAudio {
    decoder: AudioDecoder,
    first_request: usize,
    scene_window: bool,
    minimum_window_frames: i64,
    pub(crate) inputs: Vec<PortalAudioInput>,
}

impl PortalAudio {
    pub(crate) fn from_scene(
        scene: &Bound<'_, PyScene>,
        format: &PortalOutputFormat,
        video: Option<&portal_video::PortalVideoConfig>,
    ) -> PyResult<Box<Self>> {
        let executable = if let Some(video) = video {
            Some(video.ffmpeg_bin.clone())
        } else if matches!(format, PortalOutputFormat::Wav) {
            Some(
                scene
                    .getattr("file_writer")?
                    .getattr("ffmpeg_bin")?
                    .extract::<String>()?,
            )
        } else {
            None
        };
        if executable
            .as_ref()
            .is_some_and(|value| value.is_empty() || value.contains('\0'))
        {
            return Err(PyValueError::new_err(
                "file_writer.ffmpeg_bin must be nonempty and contain no NUL",
            ));
        }
        let decoder =
            AudioDecoder::new(AudioDecodeLimits::default(), executable.map(PathBuf::from))
                .map_err(decode_error)?;
        Ok(Box::new(Self {
            decoder,
            first_request: 0,
            scene_window: false,
            minimum_window_frames: i64::from(video.is_some()),
            inputs: Vec::new(),
        }))
    }

    /// A live recording includes only cues authored after it acquired output.
    pub(crate) fn begin_at_request(&mut self, first_request: usize) {
        self.first_request = first_request;
    }

    /// Subdivided clips hear all currently authored cues, cropped on the
    /// absolute scene-sample timeline. This is distinct from live inserts,
    /// which intentionally include only cues added after recording starts.
    pub(crate) fn use_scene_window(&mut self) {
        self.first_request = 0;
        self.scene_window = true;
    }

    pub(crate) fn mix(
        &mut self,
        scene: &Scene,
        threads: usize,
        timeline: &OutputTimeline,
    ) -> PyResult<Option<fmn_output::MixReport>> {
        let requests = scene
            .sound_requests()
            .get(self.first_request..)
            .ok_or_else(|| {
                PyRuntimeError::new_err(
                    "recording sound history was truncated; cancel before restoring scene state",
                )
            })?;
        if requests.is_empty() && !self.scene_window {
            return Ok(None);
        }
        let config = fmn_output::MixerConfig::default();
        let time = scene.time();
        let window = if self.scene_window {
            let (start, end) = timeline.scene_window()?;
            let end = end.max(
                start
                    .checked_add(self.minimum_window_frames)
                    .ok_or_else(|| PyOverflowError::new_err("audio window overflow"))?,
            );
            let start = fmn_output::frames_to_samples(start, time.fps(), config.sample_rate)
                .map_err(native_error)?;
            let end = fmn_output::frames_to_samples(end, time.fps(), config.sample_rate)
                .map_err(native_error)?;
            if end < start || end as u64 > config.max_output_frames {
                return Err(PyValueError::new_err(
                    "audio window exceeds the native timeline budget",
                ));
            }
            Some((start as u64, (end - start) as u64))
        } else {
            None
        };
        let timeline_frames = fmn_output::frames_to_samples(
            timeline.output_frame(time.frames())?,
            time.fps(),
            config.sample_rate,
        )
        .map_err(native_error)?;
        let mut mixer = fmn_output::SoundMixer::new(config)
            .map_err(native_error)?
            .with_timeline_frames(u64::try_from(timeline_frames).map_err(native_error)?);
        self.inputs.clear();
        let fs = fmn_platform::fs::StdFs;
        for request in requests {
            let bytes = fmn_platform::fs::FileSystem::read_bounded(
                &fs,
                &request.sound_file,
                64 * 1024 * 1024,
            )
            .map_err(|error| {
                PyOSError::new_err(format!(
                    "sound cue {}: {error}",
                    request.sound_file.display()
                ))
            })?;
            let decoded = self.decoder.decode(&bytes).map_err(|error| {
                let message = format!("sound cue {}: {error}", request.sound_file.display());
                decode_message(&error, message)
            })?;
            let input = PortalAudioInput {
                path: request.sound_file.clone(),
                channels: decoded.audio.channels,
                sample_rate: decoded.audio.sample_rate,
                sample_frames: (decoded.audio.samples.len() / usize::from(decoded.audio.channels))
                    as u64,
                report: decoded.report,
            };
            mixer
                .add(fmn_output::SoundCue {
                    audio: decoded.audio,
                    frame: if window.is_some() {
                        request.time.frames()
                    } else {
                        timeline.output_frame(request.time.frames())?
                    },
                    fps: request.time.fps(),
                    time_offset: request.time_offset,
                    gain: request.gain,
                    gain_to_background: request.gain_to_background,
                })
                .map_err(native_error)?;
            self.inputs.push(input);
        }
        match window {
            Some((start, frames)) => mixer.mix_window(start, frames, threads),
            None => mixer.mix(threads),
        }
        .map(Some)
        .map_err(native_error)
    }

    pub(crate) fn invocations(&self) -> Vec<fmn_output::InvocationReport> {
        self.inputs
            .iter()
            .filter_map(|input| input.report.invocation.clone())
            .collect()
    }
}

fn decode_error(error: AudioDecodeError) -> PyErr {
    decode_message(&error, error.to_string())
}

fn decode_message(error: &AudioDecodeError, message: String) -> PyErr {
    match error {
        AudioDecodeError::Capability(_) | AudioDecodeError::TranscoderRequired { .. } => {
            CapabilityError::new_err(message)
        }
        AudioDecodeError::Boundary(_) => PyRuntimeError::new_err(message),
        _ => PyValueError::new_err(message),
    }
}

pub(crate) fn input_facts(scene: &Bound<'_, PyScene>) -> PyResult<Py<PyList>> {
    let facts = PyList::empty(scene.py());
    for input in &scene.borrow().render_audio_inputs {
        let fact = PyDict::new(scene.py());
        fact.set_item("path", input.path.to_string_lossy().as_ref())?;
        fact.set_item("source_sha256", input.report.source_digest.to_hex())?;
        fact.set_item("pcm_sha256", input.report.pcm_digest.to_hex())?;
        fact.set_item("container", input.report.format.demuxer())?;
        fact.set_item("sample_rate", input.sample_rate)?;
        fact.set_item("channels", input.channels)?;
        fact.set_item("sample_frames", input.sample_frames)?;
        fact.set_item(
            "decoder",
            if input.report.invocation.is_some() {
                "ffmpeg"
            } else {
                "native-wav"
            },
        )?;
        fact.set_item(
            "decoder_sha256",
            input
                .report
                .invocation
                .as_ref()
                .map(|invocation| &invocation.provenance.tool_sha256_hex),
        )?;
        facts.append(fact)?;
    }
    Ok(facts.unbind())
}

#[cfg(test)]
mod tests {
    use pyo3::types::PyDictMethods as _;
    #[test]
    fn portal_compressed_audio_acceptance() {
        crate::with_python_test_module("compressed audio", |py, _module, globals| {
            globals
                .set_item(
                    "_audio_fixture_generator",
                    concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../fmn-output/tests/make_audio_fixtures.py"
                    ),
                )
                .unwrap();
            let source = std::ffi::CString::new(include_str!("../tests/audio_inputs.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("real compressed sound and output receipts");
        });
    }
}
