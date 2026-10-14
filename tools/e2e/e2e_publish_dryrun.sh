#!/bin/sh
# End-to-end publish dry-run check on Hoeffding v5 records (plan §13, bead mx-e-w8-4ol.1):
#   tools/e2e/e2e_publish_dryrun.sh
# Synthesizes a 1-second 3840x2160 60fps H.264/AAC master MP4 in a temp dir,
# runs `./x hoeffding_d publish --version v5 --master <mp4> --out <bundle>`,
# verifies every bundle artifact (4K master, 1080p proxy, SRT/VTT captions from
# narration_v5/report.json, contact_sheet.png, release_notes.md, SHA256SUMS),
# verifies SHA-256 checksums, and verifies that `--upload` is refused without
# explicit owner approval (G10).
. "$(dirname "$0")/_lib.sh"

WORK=$(python3 -I -c 'import tempfile; print(tempfile.mkdtemp(prefix="e2e_publish_"))')
MASTER_MP4="$WORK/hoeffding_master.mp4"
BUNDLE="$WORK/release_v5"

make_synthetic_4k_master() {
  ffmpeg -hide_banner -loglevel error -y \
    -f lavfi -i "color=c=0x0E1116:s=3840x2160:r=60:d=1.0" \
    -f lavfi -i "sine=frequency=220:sample_rate=48000:duration=1.0" \
    -c:v libx264 -crf 18 -preset ultrafast -pix_fmt yuv420p \
    -c:a aac -b:a 256k \
    "$MASTER_MP4"
}
e2e_step "make_synthetic_4k_master" make_synthetic_4k_master

E2E_ARTIFACTS="$BUNDLE/hoeffding_d_4k_v5.mp4 $BUNDLE/hoeffding_d_1080p_v5.mp4 $BUNDLE/hoeffding_d.en.srt $BUNDLE/hoeffding_d.en.vtt $BUNDLE/contact_sheet.png $BUNDLE/release_notes.md $BUNDLE/SHA256SUMS"
e2e_step "publish_dryrun_hoeffding_v5" "$E2E_ROOT/x" hoeffding_d publish --version v5 --master "$MASTER_MP4" --out "$BUNDLE"

verify_bundle() {
  python3 -I - "$BUNDLE" "$E2E_ROOT/explainer_corpus/hoeffding_d/publish/youtube.md" <<'PY'
import hashlib
import json
import pathlib
import subprocess
import sys

bundle = pathlib.Path(sys.argv[1])
yt_md = pathlib.Path(sys.argv[2])

expected = [
    "hoeffding_d_4k_v5.mp4",
    "hoeffding_d_1080p_v5.mp4",
    "hoeffding_d.en.srt",
    "hoeffding_d.en.vtt",
    "contact_sheet.png",
    "release_notes.md",
    "SHA256SUMS",
]
for name in expected:
    p = bundle / name
    assert p.is_file() and p.stat().st_size > 0, f"missing or empty {p}"

# Verify 1080p proxy dimensions
res = subprocess.run(
    [
        "ffprobe",
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=width,height",
        "-of",
        "json",
        str(bundle / "hoeffding_d_1080p_v5.mp4"),
    ],
    capture_output=True,
    text=True,
    check=True,
)
stream = json.loads(res.stdout)["streams"][0]
assert (stream["width"], stream["height"]) == (1920, 1080), f"bad proxy dims: {stream}"

# Verify captions came from narration_v5/report.json (61 lines)
srt_text = (bundle / "hoeffding_d.en.srt").read_text(encoding="utf-8")
assert "-->" in srt_text and len(srt_text.splitlines()) >= 100, "unexpectedly short SRT"
vtt_text = (bundle / "hoeffding_d.en.vtt").read_text(encoding="utf-8")
assert vtt_text.startswith("WEBVTT"), "invalid VTT header"

# Verify SHA256SUMS matches every file in the bundle
sums = {}
for line in (bundle / "SHA256SUMS").read_text(encoding="utf-8").splitlines():
    if line.strip():
        digest, fname = line.split(None, 1)
        sums[fname.strip()] = digest.strip()

for name in expected:
    if name == "SHA256SUMS":
        continue
    actual = hashlib.sha256((bundle / name).read_bytes()).hexdigest()
    assert sums.get(name) == actual, f"SHA256 mismatch for {name}: {sums.get(name)} != {actual}"

# Verify youtube.md exists and has Description + Chapters + Sources
assert yt_md.is_file(), f"missing {yt_md}"
yt_text = yt_md.read_text(encoding="utf-8")
for section in ("Title:", "Description", "Chapters", "Sources"):
    assert section in yt_text, f"missing {section} in {yt_md}"
PY
}
e2e_step "verify_bundle_and_checksums" verify_bundle

refuse_upload_without_approval() {
  unset ALLOW_PUBLISH_UPLOAD
  if "$E2E_ROOT/x" hoeffding_d publish --version v5 --master "$MASTER_MP4" --out "$BUNDLE" --upload 2>/dev/null; then
    echo "expected --upload without ALLOW_PUBLISH_UPLOAD=1 to fail" >&2
    return 1
  fi
  return 0
}
e2e_step "refuse_upload_planted_negative" refuse_upload_without_approval
e2e_summary
