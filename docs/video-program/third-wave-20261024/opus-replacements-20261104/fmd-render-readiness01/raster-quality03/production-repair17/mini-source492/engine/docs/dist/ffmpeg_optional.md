# The ffmpeg-Optional Posture (W11)

**FrankenManim renders completely without ffmpeg.** ffmpeg is an optional
encode accelerator at the end of the pipeline, never a requirement, never on
any path that produces scene content.

## What needs no ffmpeg

The native sinks in `fmn-output` cover the output surface directly:

| Output | Native sink | Notes |
|---|---|---|
| PNG sequence | canonical PNG encoder (owned) | The certified image product. |
| GIF | native GIF sink | `-i/--gif`; streaming, bounded-memory. |
| y4m | native y4m sink (NV12) | Lossless video interchange; feeds any external encoder. |
| WAV | native sound path | Sample-exact (BN-14). |

## What ffmpeg adds

Encoded/containerized video (MP4 and friends), hardware encoders, and media
transcode of audio inputs. These ride the **negotiated ffmpeg boundary** —
sandboxed execution, argv-only protocol, content-hashed artifacts, atomic
publication — specified in [../FFMPEG_PROTOCOL.md](../FFMPEG_PROTOCOL.md).

## The capability error names the alternative

An absent ffmpeg is a **capability error, never a silent substitution**. The
boundary fails with `BoundaryError::FfmpegUnavailable`, whose message spells
out the native alternative verbatim (`fmn_output::ffmpeg::NATIVE_ALTERNATIVE`):

> native outputs need no ffmpeg: y4m, PNG sequences, and GIF are built in;
> ffmpeg is only required for encoded video (mp4/mov), audio mux, and media
> transcode

If you asked for an encoded product without ffmpeg installed, you are told
exactly which native outputs would have worked instead. Nothing falls back to
a different format behind your back.

## Certification boundary

ffmpeg products are **excluded from certified determinism by construction**
(§16.7): with LaTeX and Pango gone, the encode boundary is the only
uncertified artifact class left. Encoded video is equivalence-classed, never
bit-promised. The certified products — raw frames, canonical PNGs, WAV — are
all native. See [determinism.md](determinism.md).
