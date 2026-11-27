"""Copy H.264 and AAC while placing the video packets on the exact 60 fps grid."""

from fractions import Fraction
import json
from pathlib import Path
import subprocess


FRAME = Fraction(1, 60)
# The existing kit master gate requires at most one millisecond of drift.
MAX_DRIFT = Fraction(1, 1000)
TIMESCALE = 15360
TICKS_PER_FRAME = TIMESCALE // 60
SETTS = (
    f"setts=time_base=1/{TIMESCALE}:prescale=1:"
    f"pts=round(PTS/{TICKS_PER_FRAME})*{TICKS_PER_FRAME}:"
    f"dts=round(DTS/{TICKS_PER_FRAME})*{TICKS_PER_FRAME}:duration={TICKS_PER_FRAME}"
)


def packet_clock(probe, path, stream):
    rows = subprocess.check_output([
        probe, "-v", "error", "-select_streams", stream, "-show_packets", "-show_streams",
        "-show_entries", "stream=time_base:packet=pts,dts,duration,flags,data_hash:packet_side_data",
        "-show_data_hash", "sha256", "-of", "json", path,
    ], timeout=300)
    data = json.loads(rows)
    metadata, = data["streams"]
    scale = Fraction(metadata["time_base"])
    clock = []
    for item in data["packets"]:
        if not item.get("data_hash"):
            raise ValueError(f"Packet payload was not verified: {path} {stream}")
        times = tuple(int(item[name]) * scale for name in ("pts", "dts", "duration"))
        clock.append((times, item["flags"], item["data_hash"], item.get("side_data_list")))
    if not clock:
        raise ValueError(f"No packets found: {path} {stream}")
    return clock


def exact_video_clock(packets, frames):
    if len(packets) != frames:
        raise ValueError(f"Got {len(packets)} video packets for {frames} frames.")
    result, peak_drift = [], Fraction(0)
    for times, flags, digest, side_data in packets:
        pts, dts, duration = times
        fixed = (round(pts / FRAME) * FRAME, round(dts / FRAME) * FRAME, FRAME)
        drift = max(abs(old - new) for old, new in zip(times, fixed, strict=True))
        if duration <= 0 or drift > MAX_DRIFT:
            raise ValueError(f"Input is outside the kit's one millisecond clock tolerance at PTS {pts}.")
        peak_drift = max(peak_drift, drift)
        result.append((fixed, flags, digest, side_data))
    presentation = sorted(times[0] for times, *_ in result)
    if any(pts != index * FRAME for index, pts in enumerate(presentation)):
        raise ValueError("Input has a missing or repeated presentation frame.")
    first_dts = result[0][0][1]
    if any(times[1] != first_dts + index * FRAME for index, (times, *_) in enumerate(result)):
        raise ValueError("Input has a missing or repeated decode frame.")
    return result, peak_drift


def remux(ffmpeg, probe, source, output, frames):
    video = packet_clock(probe, source, "v:0")
    expected, drift = exact_video_clock(video, frames)
    audio = packet_clock(probe, source, "a:0")
    command = [
        ffmpeg, "-nostdin", "-v", "error", "-n", "-copyts", "-i", source,
        "-map", "0:v:0", "-map", "0:a:0", "-c", "copy",
    ]
    if drift:
        command.extend(("-bsf:v", SETTS))
    command.extend((
        "-video_track_timescale", str(TIMESCALE), "-movie_timescale", "48000",
        "-avoid_negative_ts", "disabled",
        "-color_primaries", "bt709", "-color_trc", "bt709", "-colorspace", "bt709",
        "-movflags", "+faststart+write_colr", output,
    ))
    subprocess.run(command, check=True, timeout=900)
    if packet_clock(probe, output, "v:0") != expected:
        raise ValueError(f"Video packets differ from the proved grid and payloads: {output}")
    if packet_clock(probe, output, "a:0") != audio:
        raise ValueError(f"AAC packets or timings changed: {output}")
    return {
        "video_packets": frames,
        "aac_packets": len(audio),
        "input_largest_video_packet_drift_ms": float(drift * 1000),
        "corrected_submillisecond_video_grid": bool(drift),
        "video_packet_payloads_unchanged": True,
        "aac_packets_and_clocks_unchanged": True,
    }
