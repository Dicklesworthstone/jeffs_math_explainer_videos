#!/bin/bash
# Root runs this once. It makes one new folder, then the helper's --launch runs the unchanged guard.
set -u
HERE="$(cd "$(dirname "$0")" && pwd)" || exit 1
cd "$HERE/../../../../.." || exit 1
S=narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/optimized-timeline-recovery01
mkdir "$S" || exit 1
/opt/homebrew/opt/python@3.15/bin/python3.15 -I -B "$HERE/optimized_timeline_recovery01.py" --launch > "$S/controller01.log" 2>&1
status=$?
printf '%s\n' "$status" > "$S/controller01.exit"
exit "$status"
