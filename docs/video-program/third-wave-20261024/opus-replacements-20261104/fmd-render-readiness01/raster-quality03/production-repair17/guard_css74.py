"""CSS native lane; unchanged owner resource guards and owned-child cleanup."""
import hashlib
import importlib.util
from pathlib import Path
import socket

ROOT = Path(__file__).resolve().parent
PINS = {
    'inputs/repo/docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py': '1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097',
    'inputs/repo/narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/run_bounded_phase_oliver_owner48g_target14g_20261102.py': '819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da',
}

def load():
    assert socket.gethostname().split('.')[0] == 'superserver'
    for name, digest in PINS.items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest() == digest, name
    spec = importlib.util.spec_from_file_location('owner_guard', ROOT/next(iter(PINS)))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    guard = module.load()
    guard.ROOT = ROOT
    guard.REPO = ROOT
    guard.WORK = ROOT/'jobs'
    guard.TARGET = ROOT/'work'
    assert guard.ADMISSION_GIB['native'] == 10 and guard.STOP_FREE_GIB == 8
    assert guard.CAPS['native'] == 4 and guard.WORK_CAP_GIB == 56
    assert guard.TARGET_GROWTH_CAP_GIB == 20
    path = ROOT/'qualification01/owned-process-stop02.py'
    assert hashlib.sha256(path.read_bytes()).hexdigest() == 'a2fe7985bf76a1982e8d08b3dfb23354d2f60b322b95ed82ca83fc5e39a09236'
    spec = importlib.util.spec_from_file_location('owned_stop', path)
    stop = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(stop)
    stop.install(guard)
    assert guard.stop_owned is stop.stop_owned
    return guard

if __name__ == '__main__':
    raise SystemExit(load().main())
