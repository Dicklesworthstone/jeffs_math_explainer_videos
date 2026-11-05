#!/bin/bash
# Waits for the verified, compared final-picture timeline, then runs one guarded score. Nothing after it.
# Root launches this. Argument: this bundle's SHA256SUMS hash. Exit 0 verified, 1 refused or failed, 2 wait deadline.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd -P)
repo=$(cd "$here/../../../../.." && pwd -P)
cd "$repo"
seal=${1:?usage: seal}
helper="$here/prepare_final_picture_pipeline.py"
guard=docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py
repair="$repo/narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/final-picture-repair01"
jobs="$repair/jobs"
job=logarithm-final-picture-score01
python3 -I -B "$helper" seal --sums "$seal"
state=waiting
for ((attempt=0; attempt<2160; attempt++)); do
  state=$(python3 -I -B "$helper" poll --job logarithm-final-picture-timeline01 --needs "$repair/schedule-comparison01.json")
  if [[ "$state" != waiting ]]; then break; fi
  sleep 20
done
if [[ "$state" == failed ]]; then echo 'Timeline failed; no score launched'; exit 1; fi
if [[ "$state" != ready ]]; then echo 'Controller wait deadline; no score launched'; exit 2; fi
python3 -I -B "$helper" verify-step --step timeline
python3 -I -B "$helper" step --step score
python3 -I -B "$guard" --phase native --job "$job" --inputs "$jobs/$job-inputs.json" -- python3 -I -B "$jobs/$job.py"
python3 -I -B "$helper" verify-step --step score
python3 -I -B "$helper" compare --step score
echo 'Score finished, verified and compared; render not launched'
