#!/usr/bin/env bash
# Render all eight chapters of the narrated 4K explainer in parallel, one
# process per chapter, all into one output directory (every file a chapter
# writes is named by its key, and generated sounds are content-addressed), so
# `hoeffding master --out OUT_ROOT` finds every chapter afterwards.
#
#   [OUT_ROOT=out] [GPU_COUNT=2] [NARRATION=dir] \
#   [HOEFFDING_ARGS="--threads 16 --encoder h264_nvenc --crf 16 --score out/stems"] \
#       ./render_parallel.sh BUILD_DIR
#
# GPU_COUNT > 0 spreads chapters over NVENC devices round-robin (--gpu i % n).
# Run it outside a memory-capped login session: eight x264 "slow" 4K
# encoders hold ~6 GB each. On a systemd host:
#   systemd-run --user --unit=hoeffding-4k --collect \
#       -p MemoryHigh=infinity -p MemoryMax=infinity ./render_parallel.sh BUILD_DIR
set -u
ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
cd "$ROOT"
B="${HOEFFDING_BIN:-target/release/hoeffding}"
N="${NARRATION:-hoeffding_d/narration}"
OUT="${OUT_ROOT:-out}"
mkdir -p "$OUT" logs
exec >>logs_render_all.txt 2>&1
start=$(date +%s)
echo "start $(date) args: ${HOEFFDING_ARGS:-} gpus: ${GPU_COUNT:-0}"
i=0
for ch in 01_hook 02_gallery 03_quadruples 04_ranks 05_counting 06_formula 07_shuffle 08_outro; do
    gpu_args=()
    if [[ "${GPU_COUNT:-0}" -gt 0 ]]; then
        gpu_args=(--gpu $((i % GPU_COUNT)))
    fi
    # shellcheck disable=SC2086 # HOEFFDING_ARGS is a deliberate word list
    "$B" render "$ch" --res 3840x2160 --fps 60 --preset slow \
        --narration "$N" --out "$OUT" ${HOEFFDING_ARGS:-} ${gpu_args[@]+"${gpu_args[@]}"} >>"logs/$ch.log" 2>&1 &
    i=$((i + 1))
done
wait
echo "all chapters done in $(($(date +%s) - start))s"
