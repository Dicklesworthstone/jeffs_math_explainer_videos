#!/usr/bin/env bash
# Join the chapter MP4s in the current directory and master the audio with ONE
# fixed gain. The video stream is copied bit for bit. The soundtrack is raised
# as far as -16 LUFS integrated allows without passing -1.5 dBTP, then encoded
# as 256 kb/s AAC.
#
# There is no dynamic processing here. loudnorm silently switches to a dynamic
# mode whenever a linear gain would overshoot its peak target, and that mode
# rode the level audibly ("sudden volume changes"). The narration builder
# limits speech peaks instead, so this step only scales.
#
#   cd renders/final_4k && ../../master.sh [output.mp4]
set -euo pipefail
OUT="${1:-hoeffdings_d_explainer_4k.mp4}"
TARGET_I=-16
CEILING_TP=-1.5
printf "file '%s'\n" 0*.mp4 > concat.txt

measure() {  # integrated loudness and true peak of an input (EBU R128)
    ffmpeg -hide_banner -nostats "$@" -vn -af ebur128=peak=true -f null - 2>&1 |
        awk '/Summary/ {s = 1} s && /I:/ {i = $2} s && /Peak:/ {p = $2} END {print i, p}'
}
read -r in_i in_tp < <(measure -f concat -safe 0 -i concat.txt)
gain=$(awk -v i="$in_i" -v tp="$in_tp" -v ti="$TARGET_I" -v ct="$CEILING_TP" \
    'BEGIN {g = ti - i; if (ct - tp < g) g = ct - tp; printf "%.2f", g}')
echo "chapters: ${in_i} LUFS, ${in_tp} dBTP -> fixed gain ${gain} dB"

ffmpeg -hide_banner -loglevel error -y -f concat -safe 0 -i concat.txt \
    -map 0:v:0 -map 0:a:0 -c:v copy -af "volume=${gain}dB" \
    -c:a aac -b:a 256k -movflags +faststart "$OUT"
read -r out_i out_tp < <(measure -i "$OUT")
echo "mastered -> $OUT: ${out_i} LUFS, ${out_tp} dBTP"
