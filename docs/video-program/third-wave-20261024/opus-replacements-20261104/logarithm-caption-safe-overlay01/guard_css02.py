"""Isolated css lane for the positioned logarithm caption overlay: the preserved guard, pinned, on a new lane.

Only ROOT, WORK and TARGET move, as in each remote lane's guard01.py. Its baseline file starts in
this new lane, by the guard's own first-run rule. The policy is asserted, not changed.
"""
import hashlib
import importlib.util
from pathlib import Path

HERE = Path(__file__).resolve().parent
R = HERE.parents[4]
LANE = R / 'overlay-lane02'
G = R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py'
SOURCE = R / 'narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/run_bounded_phase_oliver_owner48g_target14g_20261102.py'
PINS = {G: '1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097',
        SOURCE: '819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da'}


def main():
    for path, digest in PINS.items():
        assert hashlib.sha256(path.read_bytes()).hexdigest() == digest, path
    LANE.mkdir(exist_ok=True)
    spec = importlib.util.spec_from_file_location('preserved_guard', G)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    guard = module.load()
    guard.ROOT = LANE
    guard.WORK = LANE / 'jobs'
    guard.TARGET = LANE / 'target'
    assert guard.ADMISSION_GIB['native'] == 10
    assert guard.STOP_FREE_GIB == 8 and guard.CAPS['native'] == 4
    assert guard.WORK_CAP_GIB == 56 and guard.TARGET_GROWTH_CAP_GIB == 20
    return guard.main()


if __name__ == '__main__':
    raise SystemExit(main())
