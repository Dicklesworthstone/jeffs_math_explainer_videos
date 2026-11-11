# CSD encoder evidence before CC3's generalized 968 helpers

CC2 (RubyRaven), 2026-11-10 23:43Z. This is a read-only probe, `csd_encoder_probe01.py`, with output in `csd_encoder_probe01.out.json`. It ran ffprobe on headers with the extradata sha256 and read the first 16 MiB of each movie for its x264 string. It wrote nothing on CSD and decoded nothing.

| Movies | Size, profile, level | refs, reorder | Tags (matrix / primaries) | Extradata | x264 options vs final968 09 (threads excluded) |
|---|---|---|---|---|---|
| master993 `final/` 01–15 (all 15, retained rows included) | 3840, High, 5.2 | 1, 2 | bt709 / unset | `caf9d50d` (all 15) | equal |
| final968 `render/09_boundary.mp4` | 3840, High, 5.2 | 1, 2 | bt709 / unset | `caf9d50d` | (reference) |
| master993 joined master | 3840, High, 5.2 | 1, 2 | bt709 / bt709 | `09351dc0` (join's h264_metadata rewrite) | equal |
| combined preview01 01 and 08 (combined01 binary) | 960, High, 3.2 | 1, 2 | bt709 / unset | `2791b6bc` | crf 18.0 against 16.0 only |

Threads vary with the host: 24, 21 on 10 and 18 on 15.

`work/native-counting02` does not exist yet, so the counting02 binary has no evidence here.

**Reading.**
- `master.rs` `join()` concatenates video-only remuxes with `-c copy`. All 15 rows and the 968 final already share one SPS/PPS.
- The combined01 binary's encoder options match apart from crf. So a combined01 4K final with 968's flags (`--res 3840x2160 --fps 60 --threads 16 --crf 16 --preset veryfast`) is expected to keep `caf9d50d`.
- The existing checks cover the result: the 992-pattern x264 check (`crf=16.0`, `rc=crf`) and the verify's full decode.
- No new check is proposed.
