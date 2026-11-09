"""Guard 490: root's exclusive native lane on mac-mini-old, in a fresh isolated root on the APFS volume EERCHExport.

Owner, 2026-11-09: rendering on mac-mini-old is requested. Task 490 sets the caps: 20 GiB native admission,
8 GiB stop, 4 GiB growth per native job, 80 GiB work, 20 GiB target growth. The owner's follow-up moved the root
from the ExFAT USBNVME16TB volume to the writable APFS disk image EERCHExport (owners disabled). Root creates
the root folder (mode 0700); Cargo and caches must not use ExFAT.

Everything else is root's guard, unchanged: run_bounded_phase_oliver_owner48g_target14g_20261102.py (819746fb),
loaded through the existing guard_owner56g_target20g.py (1c19a9b8). The 56g wrapper checks the original's hash and
makes its one in-memory change, naming the 20 GiB target cap TARGET_GROWTH_CAP_GIB. This file then sets globals on
the loaded module, as root's remote guard01.py lanes do:
  ROOT = /Volumes/EERCHExport/jmev-complex-native-20261109       (the lane lock: ROOT/expanded-native-lane.lock)
  REPO = ROOT/repo     (the child's cwd, the free-space volume, sbh's cwd)
  WORK = ROOT/jobs     (one immutable job folder per attempt; allocation-baseline.json)
  TARGET = ROOT/target (CARGO_TARGET_DIR)
  WORK_CAP_GIB = 80, ADMISSION_GIB['native'] = 20, TARGET_GROWTH_CAP_GIB = 20 (the 56g wrapper's value)
These are unchanged: the exclusive flock taken before admission; job.mkdir() refusing a used job name; the input
manifest hashed before and after the command; process-group stops (SIGTERM, then SIGKILL); the sbh admission call;
the 8 GiB stop and the 4 GiB per-job growth; the original allocation baseline, written once with mode 'x' by the
first admitted run in this WORK and never reset; the child environment; the receipt; and the exit codes.

Before loading, this file refuses (exit 1, nothing created) unless:
  - the short host name is mac-mini-old;
  - /Volumes/EERCHExport is a mount point, mounted as apfs;
  - ROOT and REPO are real, symlink-free folders on that volume, as are WORK and TARGET when they exist;
  - REPO holds the 56g wrapper and the original at their repository paths and pinned hashes;
  - Python is 3.11 or later (the original hashes inputs with hashlib.file_digest);
  - sbh is on PATH (otherwise the original would create the job folder and then fail at its sbh call);
  - none of CARGO_HOME, RUSTUP_HOME, TMPDIR or SCCACHE_DIR, when set, lies on an ExFAT or FAT mount. The child
    inherits them from the launching environment; CARGO_TARGET_DIR is always TARGET, set by the original.

Usage on the mini (the original's command line): python3 -I -B cc1-mini-guard490.py --phase native --job NAME
--inputs MANIFEST [--preflight-only] -- COMMAND...
Prepared only; nothing here has run on the mini. It authorizes no build, render or publication.
"""
import hashlib
import importlib.util
import os
from pathlib import Path
import shutil
import socket
import subprocess
import sys

HOST = 'mac-mini-old'
VOLUME = Path('/Volumes/EERCHExport')
FSTYPE = 'apfs'
NOT_FOR_CARGO = ('exfat', 'msdos')                  # file systems Cargo and its caches must not use (owner)
CACHE_VARS = ('CARGO_HOME', 'RUSTUP_HOME', 'TMPDIR', 'SCCACHE_DIR')
ROOT = VOLUME/'jmev-complex-native-20261109'
REPO = ROOT/'repo'
WORK = ROOT/'jobs'
TARGET = ROOT/'target'
WRAPPER56_REL = 'docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py'
WRAPPER56_SHA256 = '1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097'
SOURCE_REL = ('narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/'
              'run_bounded_phase_oliver_owner48g_target14g_20261102.py')
SOURCE_SHA256 = '819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da'
WORK_CAP_GIB = 80
NATIVE_ADMISSION_GIB = 20
TARGET_GROWTH_CAP_GIB = 20
CAPS = {'native': 4, 'tts': 1, 'asr': 1, 'preview': 4}                       # the original's, unchanged
ADMISSION_GIB = {'native': NATIVE_ADMISSION_GIB, 'tts': 12, 'asr': 12, 'preview': 20}
STOP_FREE_GIB = 8


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def host_problems():
    name = socket.gethostname().split('.')[0].lower()
    return [] if name == HOST else ['this host is %r, not %s' % (name, HOST)]


def mounts():
    """[(mount point, file system)] from /sbin/mount, read only."""
    out = subprocess.run(['/sbin/mount'], capture_output=True, text=True, timeout=30).stdout
    rows = []
    for line in out.splitlines():
        if ' on ' in line and ' (' in line:
            point, rest = line.split(' on ', 1)[1].rsplit(' (', 1)
            rows.append((Path(point), rest.split(',')[0].strip(' )').lower()))
    return rows


def volume_problems(rows):
    """VOLUME is a mount point whose one mount line names FSTYPE."""
    kinds = [kind for point, kind in rows if point == VOLUME]
    if not os.path.ismount(VOLUME) or kinds != [FSTYPE]:
        return ['%s is not mounted as %s (mount lines: %s)' % (VOLUME, FSTYPE, kinds or 'none')]
    return []


def env_problems(rows):
    """No inherited Cargo or cache location on an ExFAT or FAT mount (its longest-prefix mount)."""
    out = []
    for var in CACHE_VARS:
        value = os.environ.get(var)
        if not value:
            continue
        path = Path(value).resolve()
        under = [(len(str(point)), kind) for point, kind in rows if path == point or path.is_relative_to(point)]
        if under and max(under)[1] in NOT_FOR_CARGO:
            out.append('%s=%s is on a mount of type %s' % (var, value, max(under)[1]))
    return out


def tree_problems():
    """ROOT, REPO (and WORK, TARGET when present) real folders on VOLUME; the two pinned files in REPO."""
    out, device = [], os.stat(VOLUME).st_dev
    for path, required in ((ROOT, True), (REPO, True), (WORK, False), (TARGET, False)):
        if not required and not os.path.lexists(path):
            continue
        if not (path.is_dir() and not path.is_symlink() and path.resolve() == path and os.stat(path).st_dev == device):
            out.append('%s is not a real folder on %s' % (path, VOLUME))
    for rel, digest in ((WRAPPER56_REL, WRAPPER56_SHA256), (SOURCE_REL, SOURCE_SHA256)):
        path = REPO/rel
        if not (path.is_file() and not path.is_symlink() and path.resolve() == path and sha(path) == digest):
            out.append('%s is missing or not at %s' % (path, digest[:12]))
    return out


def tool_problems():
    out = [] if sys.version_info >= (3, 11) else ['python %d.%d is older than 3.11 (hashlib.file_digest)' % sys.version_info[:2]]
    return out + ([] if shutil.which('sbh') else ['sbh is not on PATH'])


def refusals():
    out = host_problems()
    if out:
        return out
    rows = mounts()
    return volume_problems(rows) or tree_problems() + tool_problems() + env_problems(rows)


def load():
    problems = refusals()
    if problems:
        raise SystemExit('cc1-mini-guard490 refused before loading the guard: '+'; '.join(problems))
    spec = importlib.util.spec_from_file_location('owner56g_target20g', REPO/WRAPPER56_REL)
    wrapper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(wrapper)                    # constants and load() only; its main() is behind __name__
    assert (wrapper.REPO, wrapper.SOURCE, wrapper.SOURCE_SHA256) == (REPO, REPO/SOURCE_REL, SOURCE_SHA256)
    guard = wrapper.load()                              # checks 819746fb; names the target cap; native admission 10
    assert guard.REPO == REPO, 'the original placed REPO at %s' % guard.REPO
    guard.ROOT = ROOT
    guard.REPO = REPO
    guard.WORK = WORK
    guard.TARGET = TARGET
    guard.WORK_CAP_GIB = WORK_CAP_GIB
    guard.ADMISSION_GIB['native'] = NATIVE_ADMISSION_GIB
    guard.TARGET_GROWTH_CAP_GIB = TARGET_GROWTH_CAP_GIB
    assert (guard.CAPS, guard.ADMISSION_GIB, guard.STOP_FREE_GIB) == (CAPS, ADMISSION_GIB, STOP_FREE_GIB)
    assert (guard.WORK_CAP_GIB, guard.TARGET_GROWTH_CAP_GIB) == (WORK_CAP_GIB, TARGET_GROWTH_CAP_GIB)
    assert (guard.ROOT, guard.REPO, guard.WORK, guard.TARGET) == (ROOT, REPO, WORK, TARGET)
    return guard


if __name__ == '__main__':
    raise SystemExit(load().main())
