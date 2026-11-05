#!/bin/bash
# Waits for the final-picture build and assembly, freezes inputs, then runs one guarded timeline.
# Root launches this. Arguments: this bundle's SHA256SUMS hash, the approved integration.json hash,
# the approved holds-bound01.json hash. Exit 0 verified, 1 refused or failed, 2 wait deadline.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd -P)
repo=$(cd "$here/../../../../.." && pwd -P)
cd "$repo"
seal=${1:?usage: seal integration-sha256 holds-sha256}
integration=${2:?usage: seal integration-sha256 holds-sha256}
holds=${3:?usage: seal integration-sha256 holds-sha256}
helper="$here/prepare_final_picture_pipeline.py"
guard=docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py
jobs="$repo/narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/final-picture-repair01/jobs"
job=logarithm-final-picture-timeline01
python3 -I -B "$helper" seal --sums "$seal"
for waited in logarithm-final-picture-build01 logarithm-final-picture-assemble01; do
  state=waiting
  for ((attempt=0; attempt<2160; attempt++)); do
    state=$(python3 -I -B "$helper" poll --job "$waited")
    if [[ "$state" != waiting ]]; then break; fi
    sleep 20
  done
  if [[ "$state" == failed ]]; then echo "$waited failed; no timeline launched"; exit 1; fi
  if [[ "$state" != ready ]]; then echo "Controller wait deadline at $waited; no timeline launched"; exit 2; fi
done
python3 -I -B "$helper" bind --integration-sha256 "$integration" --holds-sha256 "$holds"
python3 -I -B "$helper" step --step timeline
python3 -I -B "$guard" --phase native --job "$job" --inputs "$jobs/$job-inputs.json" -- python3 -I -B "$jobs/$job.py"
python3 -I -B "$helper" verify-step --step timeline
python3 -I -B "$helper" compare --step timeline
echo 'Timeline finished, verified and compared; score not launched'
