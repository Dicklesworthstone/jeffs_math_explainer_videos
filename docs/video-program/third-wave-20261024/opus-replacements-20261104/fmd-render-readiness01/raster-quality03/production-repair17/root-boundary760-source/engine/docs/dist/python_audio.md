# Sound assets in Python renders

`Scene.add_sound` accepts native PCM/float WAV and, in standard-mode host
renders, FLAC, MP3, ADTS AAC, M4A/MP4, Ogg Vorbis/Opus, and AIFF. Recognition
uses the file bytes, not the filename extension. Mono and stereo inputs retain
their source sample rate and channels until Reel's native sound mixer performs
resampling, timeline placement, gain, and ducking.

```python
from manimlib import Scene
from fmn_python import render_scene

class NarratedScene(Scene):
    def construct(self):
        self.wait(0.5)
        self.add_sound("narration.mp3", gain=-3)
        self.wait(5)

receipt = render_scene(NarratedScene, "narrated.mp4")
print(receipt.audio_inputs)
```

For a soundtrack without video, select WAV:

```bash
fmn-python lesson.py NarratedScene --format wav --video_dir narration.wav
fmn-python lesson.py First Second --format wav --ffmpeg_bin '/opt/media tools/ffmpeg' --video_dir soundtracks
```

The decoder uses `SceneFileWriter.ffmpeg_bin`, including explicit paths with
spaces. Its host search path and temporary-directory root are captured when
the render generation opens, before authored lifecycle callbacks run. Relative
sound paths are resolved when their cues are recorded in an active generation,
so later `chdir` calls do not redirect pending sound requests.

PCM WAV never resolves, probes, or launches ffmpeg. Compressed inputs require
the optional governed ffmpeg capability; a missing capability names PCM WAV as
the native-only alternative. The decoder uses a seekable snapshot in its
private work directory, an explicitly selected demuxer, a local-only protocol
allowlist, and disabled external MOV track references. Playlists are not audio
assets. It produces float32 PCM without filters, resampling, or an implicit
channel downmix; multichannel audio is rejected rather than silently changed.

Every source is limited to 64 MiB. The native sample budget and independent
bounded reads also limit decompression. Reaching the output cap is an error,
not a successful truncated soundtrack. Source, decode, mix, or mux failures do
not replace an existing destination. The ordinary ordered video generation and
native WAV publisher retain ownership of final artifact publication.

## Receipts and reproducibility

`RenderResult.audio_inputs` contains per-cue source path, source SHA-256,
normalized PCM SHA-256, recognized container, source sample rate, channels,
sample-frame count, decoder kind, and optional decoder binary SHA-256.
`ffmpeg_invocations` includes compressed audio decoding even for WAV-only
exports. Movie receipts retain video encoding, audio decoding, and final mux
facts. `as_dict()` returns independent copies suitable for JSON consumers.

The PCM digest hashes a versioned, little-endian normalized stream, including
sample count and format metadata, with negative zero canonicalized. Different
WAV sample packings can have the same decoded PCM identity while retaining
different source identities. The mixer remains native and thread-count
independent. **Compressed decoding is not certified across ffmpeg versions or
platforms**, and these receipts do not enable the still-incomplete certified
Python input closure. The portal continues to report `certified=False`.
