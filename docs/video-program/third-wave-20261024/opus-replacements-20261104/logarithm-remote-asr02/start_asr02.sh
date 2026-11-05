#!/bin/bash
# Usage:   start_asr02.sh VERIFY_JOB MASTER_JOB NN
# Example: start_asr02.sh logarithm-remote-verify01 logarithm-remote-master01 02
#
# Verifier post-check, guard preflight, then ASR through the guard. Each step stops the next.
# NN is used once. After any refusal read O/asrNN.log, then run again with the next number.
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
[[ $verify =~ ^logarithm-remote-verify([0-9][0-9])$ ]] || refuse "bad verify job: $verify"
verify_nn=${BASH_REMATCH[1]}
[[ $master =~ ^logarithm-remote-master01(-r[2-9])?$ ]] || refuse "bad master job: $master"
[[ $nn =~ ^[0-9][0-9]$ ]] && [ "$nn" != 00 ] && [ "$nn" != 01 ] || refuse "bad attempt $nn: use 02 or later, asr01 stays root's"

# Physical paths: the helper compares them with its own resolved location.
here=$(cd "$(dirname "$0")" && pwd -P)
repo=$(cd "$here/../../../../.." && pwd -P)
cd "$repo"
lab=narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104
O="$repo/$lab/remote-mac-mini-max01"
S="$repo/$lab/final-picture-repair01"
verifier="$(dirname "$here")/logarithm-remote-final-verifier02/verify_remote_final.py"
verifier_sha256=361fe7334b1a3c332e4d7b19a8bba01457e1bf85f62459dc407474069d12c20d
job="logarithm-remote-asr$nn"
probe="$job-preflight"
record="$O/asr-postcheck$nn.json"
log="$O/asr$nn.log"
status="$O/asr$nn.exit"

for path in "$O/jobs/$job" "$O/jobs/$probe" "$O/asr-inputs$nn.json" "$record" "$log" "$status"; do
    test ! -e "$path" || refuse "$path exists: use the next attempt number"
done
command -v sbh >/dev/null || refuse "sbh is not on PATH: the guard would fail after making its job folder"
printf '%s  %s\n' "$verifier_sha256" "$verifier" | shasum -a 256 -c - >/dev/null || refuse "this is not the sealed verifier02 helper"
python=$(python3 -I -B -c 'import json, sys; print(json.load(open(sys.argv[1]))["python3"]["path"])' "$O/tools01.json")
test -x "$python" || refuse "the pinned interpreter is missing: $python"

exec > "$log" 2>&1
trap 'code=$?; printf "%s\n" "$code" > "$status"' EXIT

echo "step 1 of 3: verifier post-check of $verify and $master"
postcheck=$("$python" -I -B "$verifier" verify-job --lane "$O/jobs" --jobs "$O/render-jobs01" --series 01 \
    --crate "$S/workspace/explainer_corpus/logarithm_mysteries" \
    --narration "$O/jobs/logarithm-remote-assemble01/narration01" \
    --integration "$S/integration.json" --binding "$O/remote-binding01.json" --tools "$O/tools01.json" \
    --verify "$verify_nn" --master-job "$master" --job-dir "$O/job-scripts") || {
    printf '%s\n' "$postcheck"
    exit 1
}
printf '%s\n' "$postcheck" > "$record"

echo "step 2 of 3: guard preflight, phase asr, spare job $probe"
"$python" -I -B "$O/guard01.py" --phase asr --job "$probe" --preflight-only

echo "step 3 of 3: ASR as $job"
"$python" -I -B "$here/asr02.py" "$verify" "$master" "$nn"
