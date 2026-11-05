#!/bin/bash
# Root runs this on ts1, once the selected verify job's post-check record exists. It doesn't wait: anything not ready
# refuses at once, and a refusal before step 3 uses no job name. Each NN is used once.
set -euo pipefail
set -o noclobber
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/bin:/bin"
refuse() { printf 'REFUSED: %s\n' "$1" >&2; exit 2; }
test "$#" -eq 3 || refuse "usage: start-asr-audio01.sh VERIFY_JOB MASTER_JOB NN"
verify=$1; master=$2; nn=$3
[[ $verify =~ ^complex-remote-final-verify01(-r[2-9])?$ ]] || refuse "bad verify job: $verify"
[[ $master =~ ^complex-remote-master01(-r[2-9])?$ ]] || refuse "bad master job: $master"
[[ $nn =~ ^[0-9][0-9]$ ]] && [ "$nn" != 00 ] || refuse "bad attempt: $nn"
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../../../.." && pwd)
cd "$repo"
O="$repo/narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/remote-ts1-01"
job="complex-remote-asr-audio$nn"; probe="$job-preflight"; log="$O/asr-audio$nn.log"; status="$O/asr-audio$nn.exit"
for path in "$O/jobs/$job" "$O/jobs/$probe" "$O/asr-audio-inputs$nn.json" "$log" "$status"; do
    test ! -e "$path" || refuse "$path exists: use the next attempt number"
done
command -v sbh >/dev/null || refuse "sbh is not on PATH: the guard would fail after making its job folder"
command -v ffmpeg >/dev/null || refuse "ffmpeg is not on PATH"
exec > "$log" 2>&1
trap 'code=$?; printf "%s\n" "$code" > "$status"' EXIT
echo "step 1 of 3: every gate on $verify and $master, read-only"
python3 -I -B "$here/ts1_asr_audio01.py" --check "$verify" "$master" "$nn"
echo "step 2 of 3: guard preflight, phase asr, spare job $probe"
python3 -I -B "$O/guard01.py" --phase asr --job "$probe" --preflight-only
echo "step 3 of 3: decode as $job"
python3 -I -B "$here/ts1_asr_audio01.py" "$verify" "$master" "$nn"
