"""Root's exclusive native lane with the owner's raised caps: 56 GiB work, 20 GiB target growth.

Everything else is root's guard, loaded from its own file and pinned by hash: the lane lock, 10 GiB
admission, 8 GiB stop, 4 GiB growth per native job, the input manifest, and the original allocation
baseline in the same work folder. Only the two caps change, in memory; no file of root's is edited.
"""
import hashlib
from pathlib import Path
import types

REPO = Path(__file__).resolve().parents[5]
SOURCE = REPO / (
    "narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/"
    "run_bounded_phase_oliver_owner48g_target14g_20261102.py"
)
SOURCE_SHA256 = "819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da"
WORK = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
WORK_CAP_GIB = 56
TARGET_GROWTH_CAP_GIB = 20


def load():
    data = SOURCE.read_bytes()
    if hashlib.sha256(data).hexdigest() != SOURCE_SHA256:
        raise SystemExit("root's guard changed: review it and this wrapper before running")
    text = data.decode()
    if text.count("14 * GIB") != 2 or text.count('"target_growth_cap_gib": 14') != 1:
        raise SystemExit("root's guard no longer has the expected target cap")
    text = text.replace("14 * GIB", "TARGET_GROWTH_CAP_GIB * GIB")
    text = text.replace('"target_growth_cap_gib": 14', '"target_growth_cap_gib": TARGET_GROWTH_CAP_GIB')
    guard = types.ModuleType("bounded_owner56g_target20g")
    guard.__file__ = str(SOURCE)  # Same ROOT, so the same lane lock as every other root job.
    guard.TARGET_GROWTH_CAP_GIB = TARGET_GROWTH_CAP_GIB
    exec(compile(text, str(SOURCE), "exec"), guard.__dict__)
    guard.WORK = WORK
    guard.WORK_CAP_GIB = WORK_CAP_GIB
    guard.ADMISSION_GIB["native"] = 10
    return guard


if __name__ == "__main__":
    raise SystemExit(load().main())
