//! Byte-bound, non-publishing audio ingestion inside the existing private session.

use super::*;
use crate::audio::{
    AudioDecodeError, AudioDecodeLimits, AudioInputFormat, DecodedAudio, decoded_audio,
};

/// A single allowlisted demuxer with no network protocols or external MOV
/// tracks. Never infer a format from a pathname or let a playlist dispatch a
/// second input. Do not resample/downmix: those are native mixer semantics.
fn decode_argv(format: AudioInputFormat, input: &Path, output: &Path, cap: u64) -> Vec<String> {
    let mut argv: Vec<String> = [
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-xerror",
        "-threads",
        "1",
        "-protocol_whitelist",
        "file",
        "-f",
        format.demuxer(),
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    if format == AudioInputFormat::IsoMedia {
        argv.extend(["-enable_drefs", "0", "-use_absolute_path", "0"].map(str::to_owned));
    }
    argv.extend(["-i".into(), input.display().to_string()]);
    argv.extend(
        [
            "-map",
            "0:a:0",
            "-vn",
            "-sn",
            "-dn",
            "-map_metadata",
            "-1",
            "-map_chapters",
            "-1",
            "-threads",
            "1",
            "-c:a",
            "pcm_f32le",
            "-f",
            "wav",
            "-fs",
        ]
        .map(str::to_owned),
    );
    argv.extend([cap.to_string(), "-y".into(), output.display().to_string()]);
    argv
}

impl Boundary {
    /// Decode an immutable byte snapshot to float32 PCM without publishing a
    /// file. The returned PCM is checked by the owned WAV decoder. Source bytes
    /// are staged in the exact private job directory; the caller's asset path
    /// is never reopened by ffmpeg. This prevents source-path substitution and
    /// supplies a seekable input for ordinary, non-faststart M4A files.
    ///
    /// The soft ffmpeg file cap is backed by an independent bounded read and
    /// native sample budget. Reaching the cap is a refusal, never a silently
    /// truncated successful soundtrack. External decode is not certified.
    pub fn decode_audio_bytes(
        &self,
        bytes: &[u8],
        format: AudioInputFormat,
        limits: &AudioDecodeLimits,
    ) -> Result<DecodedAudio, AudioDecodeError> {
        let cap = limits
            .decoded_byte_limit()?
            .min(self.limits.max_artifact_bytes);
        if cap == 0 {
            return Err(AudioDecodeError::InvalidLimits);
        }
        if bytes.len() as u64 > limits.max_input_bytes {
            return Err(AudioDecodeError::InputOversized {
                bytes: bytes.len() as u64,
                max: limits.max_input_bytes,
            });
        }
        let workdir = self.make_workdir()?;
        let result = (|| {
            let mut bound_tool = self.tool.bind_into(&workdir)?;
            let input = workdir.join("source.audio");
            let artifact = workdir.join("decoded.wav");
            let mut source = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&input)
                .map_err(|e| BoundaryError::Workdir {
                    detail: format!("create staged audio: {e}"),
                })?;
            source
                .write_all(bytes)
                .and_then(|()| source.sync_all())
                .map_err(|e| BoundaryError::Workdir {
                    detail: format!("stage audio snapshot: {e}"),
                })?;
            drop(source);
            let argv = decode_argv(format, &input, &artifact, cap);
            workdir.verify_current("start audio decode")?;
            bound_tool.verify_current(&self.tool)?;
            let outcome =
                self.runner
                    .run(&self.spec(bound_tool.path(), argv.clone(), &workdir, None));
            workdir.verify_current("finish audio decode")?;
            bound_tool.verify_current(&self.tool)?;
            let outcome = outcome.map_err(BoundaryError::from)?;
            self.check_outcome(&outcome)?;
            // Hash the staged source too: a child cannot silently change the
            // input whose digest is being attached to the decoded PCM.
            let (source_len, source_hash) = hash_private_artifact(&input, limits.max_input_bytes)?;
            if source_len != bytes.len() as u64 || source_hash != fmn_hash::sha256(bytes) {
                return Err(AudioDecodeError::InvalidPcm(
                    "staged source changed during decode",
                ));
            }
            verify_private_artifact(&artifact, cap)?;
            let file = File::open(&artifact).map_err(|e| BoundaryError::Workdir {
                detail: format!("read decoded audio: {e}"),
            })?;
            let mut wav = Vec::new();
            file.take(cap + 1)
                .read_to_end(&mut wav)
                .map_err(|e| BoundaryError::Workdir {
                    detail: format!("read bounded decoded audio: {e}"),
                })?;
            if wav.len() as u64 >= cap {
                return Err(BoundaryError::ArtifactOversized {
                    bytes: wav.len() as u64,
                    max: cap,
                }
                .into());
            }
            let audio = fmn_codec::decode_wav(
                &wav,
                &fmn_codec::WavLimits {
                    max_samples: limits.max_samples,
                },
            )
            .map_err(AudioDecodeError::Native)?;
            let report = invocation_report(
                &self.tool,
                bound_tool.path(),
                self.runner.mechanism(),
                Some("pcm_f32le".into()),
                argv,
                artifact,
                outcome.stderr,
            );
            decoded_audio(bytes, audio, format, Some(report))
        })();
        self.cleanup(&workdir);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_decode_argv_is_closed_and_keeps_native_mixer_authority() {
        for format in [
            AudioInputFormat::Wav,
            AudioInputFormat::Flac,
            AudioInputFormat::MpegAudio,
            AudioInputFormat::AacAdts,
            AudioInputFormat::Ogg,
            AudioInputFormat::IsoMedia,
            AudioInputFormat::Aiff,
        ] {
            let argv = decode_argv(
                format,
                Path::new("/private/source.audio"),
                Path::new("/private/decoded.wav"),
                12345,
            );
            assert!(
                argv.windows(2)
                    .any(|v| v == ["-protocol_whitelist", "file"])
            );
            assert!(argv.windows(2).any(|v| v == ["-f", format.demuxer()]));
            assert!(argv.windows(2).any(|v| v == ["-fs", "12345"]));
            assert!(argv.windows(2).any(|v| v == ["-map", "0:a:0"]));
            assert!(
                !argv
                    .iter()
                    .any(|v| ["-ar", "-ac", "-af", "-filter_complex", "-vf"].contains(&v.as_str()))
            );
            if format == AudioInputFormat::IsoMedia {
                assert!(argv.windows(2).any(|v| v == ["-enable_drefs", "0"]));
                assert!(argv.windows(2).any(|v| v == ["-use_absolute_path", "0"]));
            }
        }
    }
}
