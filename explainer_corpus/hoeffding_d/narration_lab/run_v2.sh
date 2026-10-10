#!/usr/bin/env bash
# Narration v2 end to end: plan -> parallel TTS reads -> sharded ASR ->
# objective read selection and assembly into OUT/<id>.wav.
#
#   ./run_v2.sh WORK_DIR OUT_DIR [TTS_WORKERS]
#
# SCRIPT (default script_v3.tsv) and ALTS (default alts.tsv) pick the inputs.
# Re-running after adding alternates only synthesizes the new reads.
set -euo pipefail
cd "$(dirname "$0")"
WORK="${1:?work dir}"
OUT="${2:?output dir}"
WORKERS="${3:-20}"
SCRIPT="${SCRIPT:-script_v4.tsv}"
ALTS="${ALTS:-alts.tsv}"
mkdir -p "$WORK"
exec >>"$WORK/run.log" 2>&1
echo "== $(date) plan ($SCRIPT, alternates from $ALTS)"
python3 build_v2.py plan "$SCRIPT" "$WORK" --alts "$ALTS"
echo "== $(date) tts"
python3 tts_batch.py "$WORK/takes.tsv" "$WORK/takes" --workers "$WORKERS" | tail -3
echo "== $(date) asr (one shard per GPU; only new reads are transcribed)"
gpu=0
for shard in "h,g,q,r" "c,f,s,o"; do
    CUDA_VISIBLE_DEVICES=$gpu uv run --quiet --with faster-whisper --with numpy --with nvidia-cublas-cu12 \
        --with "nvidia-cudnn-cu12>=9,<10" python3 asr_batch.py "$WORK/takes" \
        "$WORK/asr_${shard//,/}.json" --only "$shard" --device cuda >/dev/null &
    gpu=$((gpu + 1))
done
wait
python3 - "$WORK" <<'PY'
import json, sys, pathlib
work = pathlib.Path(sys.argv[1])
merged = {}
for p in sorted(work.glob("asr_*.json")):
    merged.update(json.loads(p.read_text()))
(work / "asr.json").write_text(json.dumps(merged, indent=1))
print(f"merged {len(merged)} transcripts")
PY
echo "== $(date) choose"
uv run --quiet --with numpy python3 build_v2.py choose "$SCRIPT" "$WORK" "$OUT"
echo "== $(date) done"
