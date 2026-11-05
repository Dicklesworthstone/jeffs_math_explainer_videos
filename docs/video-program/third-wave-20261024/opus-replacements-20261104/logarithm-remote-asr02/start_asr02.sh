#!/bin/bash
# Usage:   start_asr02.sh VERIFY_JOB MASTER_JOB NN
# Example: start_asr02.sh logarithm-remote-verify01 logarithm-remote-master01 02
#
# Waits on the selected jobs' records and root's post-check, then a guard preflight, then ASR through the guard. Each
# step stops the next. NN is used once. After any refusal read O/asrNN.log, then run again with the next number.
# Nothing is replaced, and the job folders of a refused attempt stay where they are.
set -euo pipefail
set -o noclobber
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin"

refuse() {
    printf 'REFUSED: %s\n' "$1" >&2
    exit 2
}

test "$#" -eq 3 || refuse "usage: start_asr02.sh VERIFY_JOB MASTER_JOB NN"
verify=$1
master=$2
nn=$3
[[ $verify =~ ^logarithm-remote-verify[0-9][0-9]$ ]] || refuse "bad verify job: $verify"
[[ $master =~ ^logarithm-remote-master01(-r[2-9])?$ ]] || refuse "bad master job: $master"
[[ $nn =~ ^[0-9][0-9]$ ]] && [ "$nn" != 00 ] && [ "$nn" != 01 ] || refuse "bad attempt $nn: use 02 or later, asr01 stays root's"

# Physical paths: the helper compares them with its own resolved location.
here=$(cd "$(dirname "$0")" && pwd -P)
repo=$(cd "$here/../../../../.." && pwd -P)
cd "$repo"
O="$repo/narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01"
job="logarithm-remote-asr$nn"
probe="$job-preflight"
log="$O/asr$nn.log"
status="$O/asr$nn.exit"

for path in "$O/jobs/$job" "$O/jobs/$probe" "$O/asr-inputs$nn.json" "$log" "$status"; do
    test ! -e "$path" || refuse "$path exists: use the next attempt number"
done
command -v sbh >/dev/null || refuse "sbh is not on PATH: the guard would fail after making its job folder"
python=$(python3 -I -B -c 'import json, sys; print(json.load(open(sys.argv[1]))["python3"]["path"])' "$O/tools01.json")
test -x "$python" || refuse "the pinned interpreter is missing: $python"

exec > "$log" 2>&1
trap 'code=$?; printf "%s\n" "$code" > "$status"' EXIT

echo "step 1 of 3: waiting for $master, $verify and the verifier post-check"
"$python" -I -B "$here/asr02.py" --wait "$verify" "$master" "$nn"

echo "step 2 of 3: guard preflight, phase asr, spare job $probe"
"$python" -I -B "$O/guard01.py" --phase asr --job "$probe" --preflight-only

echo "step 3 of 3: ASR as $job"
"$python" -I -B "$here/asr02.py" "$verify" "$master" "$nn"
