#!/bin/bash
# Run only after real review, seven final FMD chapters, export and CSD prepare.
# Local SSH configuration supplies authenticated host aliases; no remote key copy.
# A failed/truncated attempt is preserved and must not be automatically retried.
set -euo pipefail
ssh fmd '/usr/bin/python3 -I -B /data/projects/jmev-fmd-render-qualification-20261106-1111/qualification01/stream-fmd-final01.py --stream' |
ssh csd '/usr/bin/python3 -I -B /data/projects/jmev-stroke-probe-20261106/guard_csd02.py --phase native --job stage07-fmd-import01 --inputs /data/projects/jmev-stroke-probe-20261106/stage07-fmd-import01-inputs.json -- /usr/bin/python3 -I -B /data/projects/jmev-stroke-probe-20261106/qualification01/receive-fmd-final01.py --run'
