"""Same CSS lane and limits; new outputs on SSD, both locations counted."""
import hashlib
import importlib.util
import os
from pathlib import Path

OLD = Path('/data/projects/jmev-css-render-worker-20261107')
NEW = Path('/data/ssd/work/jmev-gallery-recovery440')
GUARD_SHA = '4f7edaba7963af580245a6809af0386becba8056daec5d65626cc0ff1ef8ce73'

def load():
    source = OLD/'guard_css74.py'
    assert hashlib.sha256(source.read_bytes()).hexdigest() == GUARD_SHA
    spec = importlib.util.spec_from_file_location('css_original_guard', source)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    guard = module.load()
    assert guard.ROOT == OLD
    assert (NEW/'jobs/allocation-baseline.json').read_bytes() == (OLD/'jobs/allocation-baseline.json').read_bytes()
    assert NEW.resolve() == NEW and NEW.stat().st_dev != OLD.stat().st_dev
    guard.REPO = NEW
    guard.WORK = NEW/'jobs'
    guard.TARGET = NEW/'work'
    def sample():
        disk = os.statvfs(NEW)
        return dict(utc=guard.utc(), free_bytes=disk.f_bavail*disk.f_frsize,
                    work_bytes=guard.allocated(OLD/'jobs')+guard.allocated(NEW/'jobs'),
                    target_bytes=guard.allocated(OLD/'work')+guard.allocated(NEW/'work'))
    guard.sample = sample
    assert guard.ADMISSION_GIB['native'] == 10 and guard.STOP_FREE_GIB == 8
    assert guard.CAPS['native'] == 4 and guard.WORK_CAP_GIB == 56
    assert guard.TARGET_GROWTH_CAP_GIB == 20
    # ROOT remains OLD: every old and new CSS job shares the original flock.
    return guard

if __name__ == '__main__':
    raise SystemExit(load().main())
