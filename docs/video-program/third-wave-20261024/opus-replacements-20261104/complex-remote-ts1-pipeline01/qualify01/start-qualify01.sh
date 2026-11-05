#!/bin/bash
# Root runs this once on ts1, by its /Users path. A second start is refused before it can touch the
# first start's log or exit file.
set -uo pipefail
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/bin:/bin"
HERE="$(cd "$(dirname "$0")" && pwd)" || exit 1
cd "$HERE/../../../../../.." || exit 1
work=narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/remote-ts1-01
log="$work/qualify-controller01.log"
( set -C; : > "$log" ) 2>/dev/null || { echo "refused: $log already exists" >&2; exit 1; }
python3 -I -B "$HERE/ts1_qualify01.py" --launch >> "$log" 2>&1
result=$?
printf '%s\n' "$result" > "$work/qualify-controller01.exit"
exit "$result"
