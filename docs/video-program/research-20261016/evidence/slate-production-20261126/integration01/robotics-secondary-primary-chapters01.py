"""Reuse the two reviewed primary chapters on the secondary Mac."""

import argparse
from datetime import datetime, timezone
import fcntl
from pathlib import Path
import os
import runpy
import socket


HERE = Path(__file__).resolve().parent
namespace = runpy.run_path(str(HERE / "queues-secondary-clock01.py"))
REPO, WORK, WRAPPER = (namespace[key] for key in ("REPO", "WORK", "WRAPPER"))
STAGING = namespace["REMOTE"] / "robotics-primary-early-chapters01"
SOURCE_READBACK = STAGING / "robotics-primary-early-chapters01.json"
SECONDARY_READBACK = STAGING / "secondary-verified01.json"


def checked_inputs():
    delivery = runpy.run_path(str(HERE / "companion-native-delivery02.py"))
    digest, read = (delivery[key] for key in ("digest", "read"))
    original = read(SOURCE_READBACK)
    verification = read(SECONDARY_READBACK)
    assert verification["source_receipt_sha256"] == digest(SOURCE_READBACK)
    clock = WORK / "robotics-final-clock01"
    inputs = delivery["score_inputs"]("robotics", WORK / "robotics-frederick-voice01", clock)
    required = delivery["render_sources"](inputs)
    assert len(required) == verification["render_input_bindings"]
    assert verification["chapters"] == {"01_three_ways_to_multiply": 9073, "02_arm_on_a_torus": 7754}
    installs = []
    for chapter, count in verification["chapters"].items():
        name = delivery["chapter_job"]("robotics", chapter)
        entry = original["chapters"][name]
        assert entry["chapter"] == chapter and entry["frames"] == inputs[3][chapter] == count
        payload = STAGING / name
        assert not any(path.is_symlink() for path in payload.rglob("*"))
        assert {str(path.relative_to(payload)): digest(path)
                for path in payload.rglob("*") if path.is_file()} == entry["files"]
        source_manifest = STAGING / entry["manifest"]["file"]
        guard = read(payload / "resource-receipt.json")
        assert guard["phase"] == "native" and guard["admitted"] and guard["finished"] and guard["exit"] == 0
        assert not guard.get("stop_reason") and not guard.get("stopped_for_resource_limit")
        assert guard["inputs_sha256"] == guard["final_inputs_sha256"] == entry["manifest"]["sha256"] == digest(source_manifest)
        frozen = {Path(row["path"]): row["sha256"] for row in read(source_manifest)["files"]}
        assert all(frozen.get(path) == sha for path, sha in required.items())
        assert digest(STAGING / entry["review"]["file"]) == entry["review"]["sha256"]
        installs.append((payload, WORK / name, source_manifest, delivery["input_manifest"](name)))
    return delivery, installs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--adopt", action="store_true", help="install the two verified staged chapters under the original lane lock")
    args = parser.parse_args()
    if not __debug__:
        parser.error("Run with assertions enabled.")
    assert socket.gethostname().lower().split(".")[0] == "mac-mini-max"
    if not args.adopt:
        _, candidates = checked_inputs()
        assert all(not destination.exists() and not manifest.exists() for _, destination, _, manifest in candidates)
        print("The exact 16,827 primary frames are verified and both secondary destinations are empty.")
        return
    guard = runpy.run_path(str(WRAPPER))["load"]()
    with (guard.ROOT / "expanded-native-lane.lock").open("rb") as lane:
        try:
            fcntl.flock(lane, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise SystemExit("The secondary native lane is still occupied.")
        storage = os.statvfs(REPO)
        assert storage.f_bavail * storage.f_frsize >= guard.STOP_FREE_GIB * (1 << 30)
        delivery, candidates = checked_inputs()
        assert all(not destination.exists() and not manifest.exists() for _, destination, _, manifest in candidates)
        for payload, destination, source_manifest, manifest in candidates:
            os.link(source_manifest, manifest)
            payload.rename(destination)
        delivery["write"](STAGING / "secondary-adopted01.json", {
            "adopted_utc": datetime.now(timezone.utc).isoformat(),
            "source_receipt_sha256": delivery["digest"](SOURCE_READBACK),
            "installed": [str(destination) for _, destination, _, _ in candidates],
            "frames": 16827,
        })
        print("Adopted the two verified 4K60 chapters without new rendering.")


if __name__ == "__main__":
    main()
