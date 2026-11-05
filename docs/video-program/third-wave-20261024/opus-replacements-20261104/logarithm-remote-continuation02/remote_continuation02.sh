#!/bin/bash
# The second logarithm continuation on mac-mini-max: rebind02, retained-audio assembly, binding, timeline and score,
# each guarded and verified before the next. Stops after score; render is not here. Sealed01 stays as it is, and its
# pipeline helper and assembly run unchanged. Root launches this once, after the transfer. Arguments: this bundle's
# SHA256SUMS hash, the approved integration.json hash, the approved holds-bound01.json hash.
# Exit 0 verified; anything else means stop, and later steps did not run.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd -P)
repo=$(cd "$here/../../../../.." && pwd -P)
cd "$repo"
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin"
seal=${1:?usage: seal integration-sha256 holds-sha256}
integration=${2:?usage: seal integration-sha256 holds-sha256}
holds=${3:?usage: seal integration-sha256 holds-sha256}
sealed01=26c0b341f877e6b492a4d1798e37ce11698c9d8282d759248cd9914dab2890d8
helper="$repo/docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-remote-mac-mini-max01/remote_pipeline01.py"
remote="$repo/narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01"
guard="$remote/guard01.py"
scripts="$remote/job-scripts"
python3 -I -B "$helper" seal --sums "$sealed01"
python3 -I -B "$here/remote_audio_after_build02.py" "$seal"
python3 -I -B "$helper" bind --integration-sha256 "$integration" --holds-sha256 "$holds"
for step in timeline score; do
  job="logarithm-remote-$step"01
  python3 -I -B "$helper" step --step "$step"
  python3 -I -B "$guard" --phase native --job "$job" --inputs "$scripts/$job-inputs.json" -- python3 -I -B "$scripts/$job.py"
  python3 -I -B "$helper" verify-step --step "$step"
  python3 -I -B "$helper" compare --step "$step"
done
echo 'Timeline and score finished, verified and compared; render not launched'
