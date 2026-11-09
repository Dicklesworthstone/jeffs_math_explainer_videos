#!/bin/bash
set -o pipefail
ssh trj 'python3 -I -B /data/projects/jmev-render-worker-20261107/qualification01/production19/hz2-runtime-sender47.py' | ssh hz2 'python3 -I -B /data/projects/jmev-hz2-render-worker-20261107/guard_hz201.py --phase native --job hz2-runtime-transfer47 --inputs /data/projects/jmev-hz2-render-worker-20261107/qualification01/production19/runtime47-inputs.json -- python3 -I -B /data/projects/jmev-hz2-render-worker-20261107/qualification01/production19/hz2-runtime-receiver47.py'
result=$?
printf 'PIPELINE_EXIT=%s\n' "$result"
exit "$result"
