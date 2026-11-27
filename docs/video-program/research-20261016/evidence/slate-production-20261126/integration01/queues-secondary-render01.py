"""Run one Power of Two Choices 4K60 chapter on the secondary Mac."""

import argparse
from pathlib import Path
import runpy
import socket
import sys


HERE = Path(__file__).resolve().parent
CLOCK = runpy.run_path(str(HERE / "queues-secondary-clock01.py"))
REPO, WORK, VOICE, WRAPPER = (CLOCK[name] for name in ("REPO", "WORK", "VOICE", "WRAPPER"))
PRODUCTION = HERE / "companion-native-delivery01.py"
KEY = "07_the_old_bulletin"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("check", "launch", "execute"))
    stage = parser.parse_args().stage
    assert socket.gethostname().lower().split(".")[0] == "mac-mini-max"
    production = runpy.run_path(str(PRODUCTION))
    clock = WORK / "queues-final-clock01"
    inputs = production["score_inputs"]("queues", VOICE, clock)
    digest, read, write = (production[name] for name in ("digest", "read", "write"))
    job = production["chapter_job"]("queues", KEY)
    output = WORK / job
    manifest = production["input_manifest"](job)
    if stage == "execute":
        receipt = read(output / "resource-receipt.json")
        assert receipt["admitted"] and receipt["phase"] == "native" and not receipt.get("finished")
        production["render_chapter"](job, KEY, inputs)
        return

    assert not output.exists() and not manifest.exists()
    assert digest(WRAPPER) == "1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097"
    wrapper = runpy.run_path(str(WRAPPER))
    guard = wrapper["load"]()
    assert guard.WORK == WORK and guard.REPO == REPO and guard.TARGET == REPO / "target"
    assert guard.ADMISSION_GIB["native"] == 10 and guard.STOP_FREE_GIB == 8
    assert guard.CAPS["native"] == 4 and guard.TARGET_GROWTH_CAP_GIB == 20
    guard.WORK_CAP_GIB = 200
    paths = inputs[-1] | {Path(__file__).resolve(), PRODUCTION, WRAPPER, wrapper["SOURCE"]}
    staged = CLOCK["REMOTE"] / "render-source-receipt01.json"
    for name, expected in read(staged)["files"].items():
        path = REPO / name
        assert digest(path) == expected, path
        paths.add(path)
    paths.add(staged)
    if stage == "check":
        print(f"Verified {KEY} at {inputs[3][KEY]} frames and {len(paths)} guarded inputs.")
        return
    write(manifest, {"files": [{"path": str(path), "sha256": digest(path)} for path in sorted(paths)]})
    sys.argv = [__file__, "--phase", "native", "--job", job, "--inputs", str(manifest),
                "--", sys.executable, "-I", "-B", __file__, "execute"]
    raise SystemExit(guard.main())


if __name__ == "__main__":
    main()
