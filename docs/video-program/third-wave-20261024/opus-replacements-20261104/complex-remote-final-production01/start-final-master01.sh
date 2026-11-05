#!/bin/bash
# Root runs this on ts1, by its /Users path, once the render exits 0. RUN is a new two-digit number
# each time. A repeated RUN is refused before it can touch that run's log or exit file.
set -uo pipefail
case "${1:-}" in
    [0-9][0-9]) run="$1"; shift ;;
    *) echo "usage: start-final-master01.sh RUN [--retry master=rN]" >&2; exit 2 ;;
esac
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/bin:/bin"
HERE="$(cd "$(dirname "$0")" && pwd)" || exit 1
cd "$HERE/../../../../.." || exit 1
work=narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/remote-ts1-01/final-production01
log="$work/final-master-controller$run.log"
mkdir -p "$work" || exit 1
( set -C; : > "$log" ) 2>/dev/null || { echo "refused: $log already exists" >&2; exit 1; }
python3 -I -B "$HERE/ts1_final_master01.py" --launch "$@" >> "$log" 2>&1
result=$?
printf '%s\n' "$result" > "$work/final-master-controller$run.exit"
exit "$result"
