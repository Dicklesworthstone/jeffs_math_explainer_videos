#!/bin/bash
# Root runs this on mac-mini-max, after transferring transfer-files01.json's files. It doesn't wait for transfers: a missing
# or changed file refuses at once. While another job holds the lane, the guard queues steps 2 and 3 behind it. A refusal
# before step 3 leaves the job name unused. Each NN is used once.
set -euo pipefail
set -o noclobber
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin"
refuse() { printf 'REFUSED: %s\n' "$1" >&2; exit 2; }
test "$#" -eq 1 || refuse "usage: start-take-asr01.sh NN"
nn=$1
[[ $nn =~ ^[0-9][0-9]$ ]] && [ "$nn" != 00 ] || refuse "bad attempt: $nn"
here=$(cd "$(dirname "$0")" && pwd -P)
repo=$(cd "$here/../../../../.." && pwd -P)
cd "$repo"
O="$repo/narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01"
job="complex-cauchy-take-asr$nn"; probe="$job-preflight"; log="$O/$job.log"; status="$O/$job.exit"
for path in "$O/jobs/$job" "$O/jobs/$probe" "$O/complex-cauchy-take-asr-inputs$nn.json" "$log" "$status"; do
    test ! -e "$path" || refuse "$path exists: use the next attempt number"
done
command -v sbh >/dev/null || refuse "sbh is not on PATH: the guard would fail after making its job folder"
python=$(python3 -I -B -c 'import json, sys; print(json.load(open(sys.argv[1]))["python3"]["path"])' "$O/tools01.json")
test -x "$python" || refuse "the pinned interpreter is missing: $python"
exec > "$log" 2>&1
trap 'code=$?; printf "%s\n" "$code" > "$status"' EXIT
echo "step 1 of 3: every gate on both takes, read-only"
"$python" -I -B "$here/mini_take_asr01.py" --check "$nn"
echo "step 2 of 3: guard preflight, phase asr, spare job $probe"
"$python" -I -B "$O/guard01.py" --phase asr --job "$probe" --preflight-only
echo "step 3 of 3: recognition as $job"
"$python" -I -B "$here/mini_take_asr01.py" "$nn"
