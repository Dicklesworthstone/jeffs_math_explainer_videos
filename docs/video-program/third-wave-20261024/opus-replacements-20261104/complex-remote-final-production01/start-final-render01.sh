#!/bin/bash
# Root runs this on ts1, by its /Users path, after the preview review exists. RUN is a new two-digit
# number each time: 01 first, then 02 and on to resume or to name a retry. A repeated RUN is refused
# before it can touch that run's log or exit file. No argument here or in the helper skips the review.
set -uo pipefail
case "${1:-}" in
    [0-9][0-9]) run="$1"; shift ;;
    *) echo "usage: start-final-render01.sh RUN [--retry CHAPTER=rN]..." >&2; exit 2 ;;
esac
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/bin:/bin"
HERE="$(cd "$(dirname "$0")" && pwd)" || exit 1
cd "$HERE/../../../../.." || exit 1
work=narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/remote-ts1-01/final-production01
log="$work/final-render-controller$run.log"
mkdir -p "$work" || exit 1
( set -C; : > "$log" ) 2>/dev/null || { echo "refused: $log already exists" >&2; exit 1; }
python3 -I -B "$HERE/ts1_final_render01.py" --all "$@" >> "$log" 2>&1
result=$?
printf '%s\n' "$result" > "$work/final-render-controller$run.exit"
exit "$result"
