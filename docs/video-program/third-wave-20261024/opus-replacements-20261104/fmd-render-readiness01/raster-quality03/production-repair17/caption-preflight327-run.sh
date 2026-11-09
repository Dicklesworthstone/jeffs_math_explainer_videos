#!/bin/sh
set -eu
D=/data/projects/jmev-stroke-probe-20261106
T=$D/jobs/csd-review-repair-timeline269/timeline-output
O=$D/jobs/csd-caption-preflight327/captions
mkdir "$O"
for k in 01_the_bump 02_turning_numbers 03_two_planes 04_reading_color 05_gallery 06_one_derivative 07_one_point 08_loop_sums 09_boundary 10_rigidity 11_counting 12_residues 13_real_integral 14_continuation 15_closing; do
    ln -s "$T/$k.timeline.tsv" "$O/$k.timeline.tsv"
done
cd "$D/work/production-repo07"
export PATH=/usr/bin:/bin TMPDIR="$D/work/tmp" XDG_CACHE_HOME="$D/work/cache"
exec "$D/work/bin/review-repair268" captions all --narration "$D/jobs/csd-review-repair-rebind269/narration" --out "$O"
