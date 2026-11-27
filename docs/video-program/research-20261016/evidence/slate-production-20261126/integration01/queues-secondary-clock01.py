"""Run the exact Power of Two Choices clock on the secondary Mac."""

import argparse
from pathlib import Path
import runpy
import shutil
import socket
import sys


ENTRY = Path(__file__).resolve()
REPO = ENTRY.parent.parents[5]
WORK = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
CLOCK = ENTRY.with_name("companion-clock-captions01.py")
WRAPPER = REPO / ("docs/video-program/third-wave-20261024/opus-replacements-20261104/"
                  "engine-secondary-validation01/guard_owner56g_target20g.py")
JOB = "queues-final-clock01"
OUTPUT = WORK / JOB
VOICE = WORK / "queues-frederick-voice01"
INPUTS = WORK / "navier-production-20261120" / f"{JOB}-inputs.json"
REMOTE = REPO / "narration_lab/companion-native-transport-probe-20261127"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("check", "launch", "execute"))
    stage = parser.parse_args().stage
    assert socket.gethostname().lower().split(".")[0] == "mac-mini-max"
    scope = runpy.run_path(str(CLOCK))
    audio, digest, write = (scope[name] for name in ("audio", "digest", "write"))
    _, plan, speech, _, paths = audio["film_inputs"]("queues")
    package = audio["successful_receipt"](VOICE, "asr")
    narration = VOICE / "narration"
    lines, voice_paths = scope["package_inputs"](narration, speech, plan["script_sha256"])
    binary = REPO / plan["native_executable"]
    assert digest(binary) == "11f7c5441a482347e3c827053b1f34abe96eb77475496f32f2c79fd631e63c96"
    assert len(speech) == 37 and len(plan["chapters"]) == 10

    if stage == "execute":
        state = scope["read"](OUTPUT / "resource-receipt.json")
        assert state["admitted"] and state["phase"] == "native" and not state.get("finished")
        scope["clock_captions_score"](OUTPUT, binary, plan["chapters"], narration, speech, lines)
        return

    assert not OUTPUT.exists() and not INPUTS.exists()
    assert digest(WRAPPER) == "1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097"
    wrapper = runpy.run_path(str(WRAPPER))
    guard = wrapper["load"]()
    assert guard.ROOT / "expanded-native-lane.lock" == REPO / (
        "narration_lab/production-20261020/second-wave-central/"
        "duration-and-storage-20261020/root/expanded-native-lane.lock")
    assert guard.REPO == REPO and guard.WORK == WORK and guard.TARGET == REPO / "target"
    assert guard.ADMISSION_GIB["native"] == 10 and guard.STOP_FREE_GIB == 8
    assert guard.CAPS["native"] == 4 and guard.TARGET_GROWTH_CAP_GIB == 20
    guard.WORK_CAP_GIB = 200

    source_receipt = REMOTE / "clock-source-receipt02.json"
    exports = scope["read"](source_receipt)
    for name, expected in exports["files"].items():
        path = REPO / name
        assert digest(path) == expected, path
        paths.add(path)
    paths.update((*voice_paths, package, ENTRY, CLOCK, WRAPPER, wrapper["SOURCE"],
                  source_receipt, Path(shutil.which("ffprobe")).resolve(),
                  Path(shutil.which("ffmpeg")).resolve()))
    if stage == "check":
        print(f"Verified {len(paths)} remote dependencies and all 37 exact selected recordings.")
        return
    INPUTS.parent.mkdir(parents=True, exist_ok=True)
    write(INPUTS, {"files": [{"path": str(path), "sha256": digest(path)} for path in sorted(paths)]})
    sys.argv = [str(ENTRY), "--phase", "native", "--job", JOB, "--inputs", str(INPUTS),
                "--", sys.executable, "-I", "-B", str(ENTRY), "execute"]
    raise SystemExit(guard.main())


if __name__ == "__main__":
    main()
