"""Run the unchanged native delivery verifier from its source-bound cwd."""
import json
import os
from pathlib import Path

ROOT = Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
SOURCE = Path('/data/projects/jmev-stroke-probe-20261106/work/production-build07/workspace')
CHAPTERS = ('01_the_bump', '02_turning_numbers', '03_two_planes',
            '04_reading_color', '05_gallery', '06_one_derivative',
            '07_one_point', '08_loop_sums', '09_boundary', '10_rigidity',
            '11_counting', '12_residues', '13_real_integral',
            '14_continuation', '15_closing')

receipt = json.loads((ROOT/'jobs/fmd-delivery02/resource-receipt.json').read_text())
assert receipt['admitted'] and 'exit' not in receipt
assert receipt['policy'] == dict(admission_gib=10, stop_free_gib=8,
    phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20,
    sample_seconds=10)
assert SOURCE.resolve() == ROOT/'work/production-build07/workspace'
inputs = ROOT/'production-inputs01'
os.chdir(inputs/'production-repo07')
command = ['/usr/bin/python3', '-I', '-B', str(SOURCE/'tools/narration_delivery.py'),
           '--topic', str(SOURCE/'explainer_corpus/complex_analysis_visual'),
           '--narration', str(inputs/'narration'),
           '--score', str(inputs/'scored/stems'), '--consumer', 'render',
           '--gain-db', '0']
for chapter in CHAPTERS:
    command.extend(['--chapter-key', chapter])
os.execv(command[0], command)
