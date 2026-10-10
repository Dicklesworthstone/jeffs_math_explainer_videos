#!/usr/bin/env bash
# Speak every narration line with FrankenTTS (voice "robert") into
# narration/<id>.wav. Resumable: lines whose WAV already exists are kept.
#
#   ./narrate.sh [path/to/hoeffding] [path/to/ftts]
set -euo pipefail
cd "$(dirname "$0")"
HOEFFDING="${1:-${CARGO_TARGET_DIR:-target}/release/hoeffding}"
FTTS="${2:-${CARGO_TARGET_DIR:-target}/release/ftts}"
VOICE="${FTTS_VOICE:-robert}"
MODEL_ARGS=()
[[ -n "${FTTS_MODEL:-}" ]] && MODEL_ARGS=(--model "$FTTS_MODEL")
mkdir -p narration

"$HOEFFDING" script | while IFS=$'\t' read -r id text; do
    out="narration/${id}.wav"
    if [[ -s "$out" ]]; then
        echo "keep  $id"
        continue
    fi
    start=$(date +%s)
    "$FTTS" say --voice "$VOICE" "${MODEL_ARGS[@]}" "$text" "narration/${id}.partial.wav" </dev/null >/dev/null
    mv "narration/${id}.partial.wav" "$out"
    echo "spoke $id in $(( $(date +%s) - start ))s"
done
