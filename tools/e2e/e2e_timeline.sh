#!/bin/sh
# End-to-end construct-only timeline pass check (plan §15.3, bead mx-e-w1-4qa.12):
#   tools/e2e/e2e_timeline.sh <slug> [chapter]
# Synthesizes a valid 24 kHz mono s16 narration fixture (or uses existing
# narration/out), runs `render --timeline-only --fps 60 --narration` into a temp
# directory, and verifies that every cue in `<chapter>.timeline.tsv` and
# `<chapter>.cuelog.tsv` has integer frames, exact f64 hex bits, and sample-exact
# placement (frame * 800 == start_sample at 60 fps).
. "$(dirname "$0")/_lib.sh"

slug="${1:?usage: e2e_timeline.sh <slug> [chapter]}"
crate="$E2E_ROOT/explainer_corpus/$slug"
[ -d "$crate" ] || { echo "no explainer at $crate" >&2; exit 2; }
pkg=$(sed -n 's/^name = "\(.*\)"$/\1/p' "$crate/Cargo.toml" | head -n 1)
bin=$(awk '/^\[\[bin\]\]/ { f = 1 } f && /^name/ { gsub(/.*= *"|"/, ""); print; exit }' "$crate/Cargo.toml")
binary="${CARGO_TARGET_DIR:-$E2E_ROOT/target}/release/$bin"

e2e_step "build_binary" cargo build --release -p "$pkg" --manifest-path "$E2E_ROOT/Cargo.toml" -Zgit=shallow-deps -Zgitoxide=fetch
chapter="${2:-$("$binary" chapters | head -n 1)}"
out_dir=$(python3 -I -c 'import tempfile; print(tempfile.mkdtemp(prefix="e2e_timeline_"))')
narr_dir="$out_dir/narration"

prepare_narration() {
  python3 -I - "$crate/script.tsv" "$narr_dir" <<'PY'
import json
import math
import pathlib
import struct
import sys
import wave

script_path = pathlib.Path(sys.argv[1])
narr_dir = pathlib.Path(sys.argv[2])
narr_dir.mkdir(parents=True, exist_ok=True)

def split_sentences(text: str) -> list[str]:
    out = []
    start = 0
    b = text
    for i, ch in enumerate(b):
        if ch in ".?!" and (i + 1 == len(b) or b[i + 1] == " "):
            s = b[start : i + 1].strip()
            if s:
                out.append(s)
            start = i + 1
    rest = b[start:].strip()
    if rest:
        out.append(rest)
    return out

rate = 24000
lines_obj = {}
for raw in script_path.read_text(encoding="utf-8").splitlines():
    if not raw.strip() or raw.startswith("#"):
        continue
    parts = raw.split("\t")
    if len(parts) < 2 or parts[0] == "id":
        continue
    line_id, text = parts[0].strip(), parts[1].strip()
    sents = split_sentences(text)
    per_sent = 12000  # 0.5 s per sentence at 24 kHz
    total_samples = per_sent * max(1, len(sents))
    # Write a quiet 160 Hz tone in the interior with 50 ms quiet edges
    wav_path = narr_dir / f"{line_id}.wav"
    edge = int(0.05 * rate)
    pcm = bytearray()
    for n in range(total_samples):
        if edge <= n < total_samples - edge:
            sample = int(4000 * math.sin(2.0 * math.pi * 160.0 * n / rate))
        else:
            sample = 0
        pcm.extend(struct.pack("<h", sample))
    with wave.open(str(wav_path), "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(rate)
        wf.writeframes(bytes(pcm))
    lines_obj[line_id] = [
        {"sentence": s, "out_samples": per_sent, "gap_after_samples": 0}
        for s in sents
    ]

report = {"meta": {"rate": rate}, "lines": lines_obj}
(narr_dir / "report.json").write_text(json.dumps(report), encoding="utf-8")
PY
}
e2e_step "prepare_narration_fixture" prepare_narration

E2E_ARTIFACTS="$out_dir/$chapter.timeline.tsv $out_dir/$chapter.cuelog.tsv $out_dir/$chapter.cues.tsv"
e2e_step "render_timeline_only_60fps" "$binary" render "$chapter" --timeline-only --fps 60 --narration "$narr_dir" --out "$out_dir"

verify_timeline() {
  python3 -I - "$out_dir/$chapter.timeline.tsv" "$out_dir/$chapter.cuelog.tsv" "$out_dir/$chapter.cues.tsv" <<'PY'
import pathlib
import sys

tl_path = pathlib.Path(sys.argv[1])
cuelog_path = pathlib.Path(sys.argv[2])
cues_path = pathlib.Path(sys.argv[3])
assert tl_path.is_file(), f"missing {tl_path}"
assert cuelog_path.is_file(), f"missing {cuelog_path}"
assert cues_path.is_file(), f"missing {cues_path}"

fps = None
frames = None
line_rows = 0
for raw in tl_path.read_text(encoding="utf-8").splitlines():
    if not raw.strip() or raw.startswith("#"):
        continue
    parts = raw.split("\t")
    if parts[0] == "fps":
        fps = int(parts[1])
    elif parts[0] == "frames":
        frames = int(parts[1])
    elif parts[0] == "line":
        line_rows += 1
        int(parts[2])  # integer start_frame

assert fps == 60, f"expected fps=60, got {fps}"
assert frames is not None and frames > 0, f"expected positive frames, got {frames}"
assert line_rows > 0, "expected at least one line row in timeline.tsv"

cue_rows = 0
for raw in cuelog_path.read_text(encoding="utf-8").splitlines():
    if not raw.strip() or raw.startswith("#"):
        continue
    cols = raw.split("\t")
    assert len(cols) == 13, f"expected 13 columns in cuelog.tsv, got {len(cols)}: {raw}"
    frame = int(cols[3])
    row_fps = int(cols[4])
    offset_bits = cols[6]
    gain_bits = cols[8]
    start_sample = int(cols[12])
    assert row_fps == 60, f"expected fps=60 in cuelog, got {row_fps}"
    assert offset_bits.startswith("0x") and len(offset_bits) == 18, f"bad offset_bits: {offset_bits}"
    if gain_bits != "-":
        assert gain_bits.startswith("0x") and len(gain_bits) == 18, f"bad gain_bits: {gain_bits}"
    assert start_sample == frame * 800, f"expected start_sample={frame * 800}, got {start_sample}"
    cue_rows += 1

assert cue_rows > 0, "expected at least one cue in cuelog.tsv"
PY
}
e2e_step "verify_timeline_invariants" verify_timeline
e2e_summary
