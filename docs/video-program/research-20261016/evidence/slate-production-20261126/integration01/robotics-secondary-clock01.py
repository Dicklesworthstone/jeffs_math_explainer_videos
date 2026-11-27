"""Build the exact FrankenRobotics clock on mac-mini-max using the guarded native lane."""

import argparse
from pathlib import Path
import runpy
import shutil
import socket
import sys


ENTRY = Path(__file__).resolve()
HERE = ENTRY.parent
secondary = runpy.run_path(str(HERE / "queues-secondary-clock01.py"))
REPO, WORK, CLOCK, WRAPPER, REMOTE = (secondary[key] for key in ("REPO", "WORK", "CLOCK", "WRAPPER", "REMOTE"))
JOB = "robotics-final-clock01"
VOICE = WORK / "robotics-frederick-voice01"
OUTPUT = WORK / JOB
INPUTS = WORK / "navier-production-20261120" / f"{JOB}-inputs.json"
SOURCE = REMOTE / "robotics-clock-source-receipt01.json"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("check", "launch", "execute"))
    stage = parser.parse_args().stage
    assert socket.gethostname().lower().split(".")[0] == "mac-mini-max"
    scope = runpy.run_path(str(CLOCK))
    audio, digest, read, write = (scope[name] for name in ("audio", "digest", "read", "write"))
    _, plan, speech, _, paths = audio["film_inputs"]("robotics")
    package = audio["successful_receipt"](VOICE, "asr")
    narration = VOICE / "narration"
    lines, voice_paths = scope["package_inputs"](narration, speech, plan["script_sha256"])
    binary = REPO / plan["native_executable"]
    native_name = REMOTE / "frankenrobotics_native_representation"
    # The kit names its artifacts from the compiled crate, not the installed executable.
    assert digest(binary) == digest(native_name) == plan["native_executable_sha256"]
    assert len(speech) == 133 and len(plan["chapters"]) == 12

    if stage == "execute":
        state = read(OUTPUT / "resource-receipt.json")
        assert state["admitted"] and state["phase"] == "native" and not state.get("finished")
        scope["clock_captions_score"](OUTPUT, native_name, plan["chapters"], narration, speech, lines)
        return

    assert not OUTPUT.exists() and not INPUTS.exists()
    assert digest(WRAPPER) == "1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097"
    wrapper = runpy.run_path(str(WRAPPER))
    guard = wrapper["load"]()
    assert guard.REPO == REPO and guard.WORK == WORK and guard.TARGET == REPO / "target"
    assert guard.ADMISSION_GIB["native"] == 10 and guard.STOP_FREE_GIB == 8
    assert guard.CAPS["native"] == 4 and guard.TARGET_GROWTH_CAP_GIB == 20
    guard.WORK_CAP_GIB = 200

    for name, expected in read(SOURCE)["files"].items():
        path = REPO / name
        assert digest(path) == expected, path
        paths.add(path)
    paths.update((*voice_paths, package, native_name, ENTRY, CLOCK, WRAPPER, wrapper["SOURCE"],
                  SOURCE, Path(shutil.which("ffprobe")).resolve(), Path(shutil.which("ffmpeg")).resolve()))
    if stage == "check":
        print(f"Verified {len(paths)} remote inputs with the 133 exact original cues and native artifact name.")
        return
    INPUTS.parent.mkdir(parents=True, exist_ok=True)
    write(INPUTS, {"files": [{"path": str(path), "sha256": digest(path)} for path in sorted(paths)]})
    sys.argv = [str(ENTRY), "--phase", "native", "--job", JOB, "--inputs", str(INPUTS),
                "--", sys.executable, "-I", "-B", str(ENTRY), "execute"]
    raise SystemExit(guard.main())


if __name__ == "__main__":
    main()
