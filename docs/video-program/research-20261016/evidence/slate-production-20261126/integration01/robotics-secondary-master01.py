"""Join the reviewed robotics chapters inside the secondary Mac's original lane."""

import argparse
from pathlib import Path
import runpy
import socket
import sys


HERE = Path(__file__).resolve().parent
host = runpy.run_path(str(HERE / "queues-secondary-clock01.py"))
REPO, WORK, WRAPPER = (host[name] for name in ("REPO", "WORK", "WRAPPER"))
DELIVERY = HERE / "companion-native-delivery02.py"
VOICE = WORK / "robotics-frederick-voice01"
CLOCK = WORK / "robotics-final-clock01"
BASELINE = WORK / "allocation-baseline.json"
BASELINE_SHA256 = "5df0f17125e127d943c4478cc21d23439902a8e7e8ee696bf9008831fd484595"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("check", "launch"))
    args = parser.parse_args()
    assert __debug__ and socket.gethostname().lower().split(".")[0] == "mac-mini-max"
    production = runpy.run_path(str(DELIVERY))
    digest, read, write = (production[key] for key in ("digest", "read", "write"))
    assert digest(BASELINE) == BASELINE_SHA256
    assert digest(WRAPPER) == "1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097"
    layer = runpy.run_path(str(WRAPPER))
    guard = layer["load"]()
    assert guard.WORK == WORK and guard.REPO == REPO and guard.TARGET == REPO / "target"
    assert guard.ADMISSION_GIB["native"] == 10 and guard.STOP_FREE_GIB == 8
    assert guard.CAPS["native"] == 4 and guard.TARGET_GROWTH_CAP_GIB == 20
    guard.WORK_CAP_GIB = 200

    inputs = production["score_inputs"]("robotics", VOICE, CLOCK)
    assert len(inputs[3]) == 12 and sum(inputs[3].values()) == 128487
    expected = production["render_sources"](inputs)
    missing = []
    for chapter in inputs[3]:
        base = production["chapter_job"]("robotics", chapter)
        try:
            folder = WORK / production["stage_job"](base)
        except RuntimeError:
            missing.append(chapter)
            continue
        if not (folder / "resource-receipt.json").is_file():
            missing.append(chapter)
            continue
        production["bind_guard_inputs"](folder, expected)
    if missing:
        print(f"Master waiting on {len(missing)} stable chapter receipts: {', '.join(missing)}")
        raise SystemExit(3)
    production["finished_chapters"]("robotics", inputs, expected)
    job = production["stage_job"]("robotics-native-master01")
    manifest = production["input_manifest"](job)
    assert not (WORK / job).exists() and not manifest.exists()
    source = host["REMOTE"] / "robotics-clock-source-receipt01.json"
    paths = inputs[-1] | {Path(__file__).resolve(), DELIVERY, WRAPPER, layer["SOURCE"], BASELINE, source}
    for name, sha in read(source)["files"].items():
        path = REPO / name
        assert digest(path) == sha
        paths.add(path)
    if args.mode == "check":
        print(f"Verified 12 stable chapters, 128,487 frames and {len(paths)} master bindings.")
        return
    write(manifest, {"files": [{"path": str(path), "sha256": digest(path)} for path in sorted(paths)]})
    sys.argv = [__file__, "--phase", "native", "--job", job, "--inputs", str(manifest), "--",
                sys.executable, "-I", "-B", str(DELIVERY), "robotics", "master",
                "--package", str(VOICE), "--clock", str(CLOCK), "--job", job]
    raise SystemExit(guard.main())


if __name__ == "__main__":
    main()
