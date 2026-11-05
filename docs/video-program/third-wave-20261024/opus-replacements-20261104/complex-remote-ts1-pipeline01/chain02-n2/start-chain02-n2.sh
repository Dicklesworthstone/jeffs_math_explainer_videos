#!/bin/bash
# Root runs this once per stage on ts1, by its /Users path: derive, timeline, score, preview or
# pictures. Each start waits for its upstream, then runs ts1's guard. A second start of a stage is
# refused before it can touch the first start's log or exit file.
set -uo pipefail
case "${1:-}" in
    derive|timeline|score|preview|pictures) stage="$1" ;;
    *) echo "usage: start-chain02-n2.sh derive|timeline|score|preview|pictures" >&2; exit 2 ;;
esac
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/bin:/bin"
HERE="$(cd "$(dirname "$0")" && pwd)" || exit 1
cd "$HERE/../../../../../.." || exit 1
work=narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/remote-ts1-01
log="$work/chain-$stage-controller02.log"
( set -C; : > "$log" ) 2>/dev/null || { echo "refused: $log already exists" >&2; exit 1; }
python3 -I -B "$HERE/ts1_chain02_n2.py" --launch "$stage" >> "$log" 2>&1
result=$?
printf '%s\n' "$result" > "$work/chain-$stage-controller02.exit"
exit "$result"
