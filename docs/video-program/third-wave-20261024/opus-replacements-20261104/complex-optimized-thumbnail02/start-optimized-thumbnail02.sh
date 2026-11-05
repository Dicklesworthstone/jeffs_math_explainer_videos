#!/bin/bash
# Root runs this once. It waits for its upstream, then runs the unchanged guard. A second start is
# refused before it can touch the first start's log or exit file.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)" || exit 1
cd "$HERE/../../../../.." || exit 1
work=narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/optimized-thumbnail02
log="$work/optimized-thumbnail02-controller01.log"
mkdir -p "$work" || exit 1
( set -C; : > "$log" ) 2>/dev/null || { echo "refused: $log already exists" >&2; exit 1; }
/opt/homebrew/opt/python@3.15/bin/python3.15 -I -B "$HERE/thumbnail_optimized02.py" >> "$log" 2>&1
result=$?
printf '%s\n' "$result" > "$work/optimized-thumbnail02-controller01.exit"
exit "$result"
