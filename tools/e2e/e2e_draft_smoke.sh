#!/bin/sh
# End-to-end draft smoke test (plan §15.3, bead mx-e-w1-4qa.12):
#   tools/e2e/e2e_draft_smoke.sh <slug> [chapter]
# Renders one chapter at 480x270@15 with --scratch-voice --lint into a temp
# output directory, verifies that the MP4 decodes at 480x270@15, its frame count
# covers all estimated cues in `<chapter>.cues.tsv`, `<chapter>.layout.tsv`
# parses cleanly, and writes a contact sheet.
. "$(dirname "$0")/_lib.sh"

slug="${1:?usage: e2e_draft_smoke.sh <slug> [chapter]}"
crate="$E2E_ROOT/explainer_corpus/$slug"
[ -d "$crate" ] || { echo "no explainer at $crate" >&2; exit 2; }
pkg=$(sed -n 's/^name = "\(.*\)"$/\1/p' "$crate/Cargo.toml" | head -n 1)
bin=$(awk '/^\[\[bin\]\]/ { f = 1 } f && /^name/ { gsub(/.*= *"|"/, ""); print; exit }' "$crate/Cargo.toml")
binary="${CARGO_TARGET_DIR:-$E2E_ROOT/target}/release/$bin"

e2e_step "build_binary" cargo build --release -p "$pkg" --manifest-path "$E2E_ROOT/Cargo.toml" -Zgit=shallow-deps -Zgitoxide=fetch
chapter="${2:-$("$binary" chapters | head -n 1)}"
out_dir=$(python3 -I -c 'import tempfile; print(tempfile.mkdtemp(prefix="e2e_draft_smoke_"))')

E2E_ARTIFACTS="$out_dir/$chapter.mp4 $out_dir/$chapter.cues.tsv $out_dir/$chapter.layout.tsv"
e2e_step "render_draft_480x270_15" "$binary" render "$chapter" --res 480x270 --fps 15 --scratch-voice --lint --out "$out_dir"

verify_outputs() {
  python3 -I - "$out_dir" "$chapter" <<'PY'
import json
import pathlib
import subprocess
import sys

out = pathlib.Path(sys.argv[1])
ch = sys.argv[2]
mp4 = out / f"{ch}.mp4"
cues = out / f"{ch}.cues.tsv"
layout = out / f"{ch}.layout.tsv"
for p in (mp4, cues, layout):
    assert p.is_file(), f"missing output file: {p}"
assert mp4.stat().st_size > 1024, "mp4 too small"

cues_rows = [
    line.split("\t")
    for line in cues.read_text(encoding="utf-8").splitlines()
    if line.strip() and not line.startswith("#")
]
assert len(cues_rows) > 0, "expected at least one row in cues.tsv"
max_cue_end = max(float(r[1]) + float(r[2]) for r in cues_rows)

probe = subprocess.run(
    [
        "ffprobe",
        "-v",
        "error",
        "-count_frames",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=width,height,r_frame_rate,nb_read_frames",
        "-of",
        "json",
        str(mp4),
    ],
    capture_output=True,
    text=True,
    check=True,
)
stream = json.loads(probe.stdout)["streams"][0]
assert (stream["width"], stream["height"]) == (480, 270), f"unexpected dims: {stream}"
assert stream["r_frame_rate"] == "15/1", f"unexpected fps: {stream}"
actual_frames = int(stream["nb_read_frames"])
assert actual_frames >= int(max_cue_end * 15), f"frames ({actual_frames}) shorter than cues ({max_cue_end}s)"
PY
}
e2e_step "verify_mp4_cues_and_layout" verify_outputs

SKILLS="${SKILLS:-$HOME/.claude/skills}"
sheet_script="$SKILLS/franken-manim-mega-skill/scripts/contact_sheet.sh"
if [ -x "$sheet_script" ]; then
  E2E_ARTIFACTS="$out_dir/${chapter}_sheet.png"
  e2e_step "contact_sheet" "$sheet_script" "$out_dir/$chapter.mp4" "$out_dir/${chapter}_sheet.png" 4x3
fi
e2e_summary
