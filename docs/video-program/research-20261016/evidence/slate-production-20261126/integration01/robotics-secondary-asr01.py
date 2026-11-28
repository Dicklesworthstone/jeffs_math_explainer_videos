"""Check the exact FrankenRobotics package on its master host."""

import json
from pathlib import Path
import runpy
import socket
import subprocess
import sys


HERE = Path(__file__).resolve().parent
ENTRY = HERE / "slate-export-joins01.py"
JOB = "robotics-native-master-join-asr01"
BASELINE_SHA256 = "5df0f17125e127d943c4478cc21d23439902a8e7e8ee696bf9008831fd484595"


def main():
    assert __debug__ and socket.gethostname().lower().split(".")[0] == "mac-mini-max"
    mode, = sys.argv[1:]
    host = runpy.run_path(str(HERE / "queues-secondary-clock01.py"))
    repo, work, wrapper = (host[name] for name in ("REPO", "WORK", "WRAPPER"))
    joins = runpy.run_path(str(ENTRY))
    sha, read = joins["sha"], joins["read"]
    baseline = work / "allocation-baseline.json"
    assert sha(baseline) == BASELINE_SHA256
    assert sha(wrapper) == "1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097"
    tools = {joins[key] for key in ("FW", "FFMPEG", "MODEL", "JOIN_AUDIT")}
    assert all(path.is_file() for path in tools)
    if mode == "inspect":
        print(json.dumps({"host": socket.gethostname(), "tools": {path.name: sha(path) for path in sorted(tools)}}))
        return

    directory = work / JOB
    movie, script, binding, paths = joins["export_sources"]("robotics")
    assert binding["frames"] == 128487 and len(binding["chapters"]) == 12
    packaged = movie.parent.parent / "publication-package01"
    package_manifest = packaged / "manifest01.json"
    package_movie = packaged / movie.name
    assert read(package_manifest)["assets"][movie.name]["sha256"] == sha(package_movie) == binding["master_sha256"]
    if mode == "verify":
        audit = joins["verify"](directory, binding)
        check = read(directory / "videotoolbox01.json")
        assert check["master_sha256"] == binding["master_sha256"] and check["full_decode_exit"] == 0
        print(audit)
        return
    if mode == "execute":
        receipt = read(directory / "resource-receipt.json")
        assert receipt["admitted"] and receipt["phase"] == "asr" and not receipt.get("finished")
        log = directory / "videotoolbox01.log"
        with log.open("x") as handle:
            subprocess.run([
                joins["FFMPEG"], "-nostdin", "-v", "error", "-xerror", "-hwaccel", "videotoolbox",
                "-i", package_movie, "-map", "0:v:0", "-map", "0:a:0", "-f", "null", "-",
            ], check=True, timeout=3600, stdout=handle, stderr=handle)
        assert not log.stat().st_size
        with (directory / "videotoolbox01.json").open("x") as handle:
            json.dump({"master_sha256": binding["master_sha256"], "full_decode_exit": 0,
                       "video_frames": binding["frames"], "log_sha256": sha(log)}, handle, indent=2)
            handle.write("\n")
        joins["recognize"](directory, package_movie, script, binding)
        return

    assert mode == "launch"
    manifest = work / "navier-production-20261120" / f"{JOB}-inputs.json"
    assert not directory.exists() and not manifest.exists()
    layer = runpy.run_path(str(wrapper))
    guard = layer["load"]()
    assert guard.REPO == repo and guard.WORK == work and guard.TARGET == repo / "target"
    assert guard.ADMISSION_GIB["asr"] == 12 and guard.STOP_FREE_GIB == 8
    assert guard.CAPS["asr"] == 1 and guard.TARGET_GROWTH_CAP_GIB == 20
    guard.WORK_CAP_GIB = 200
    paths.update(tools | {ENTRY, Path(__file__).resolve(), joins["PACKAGE"], wrapper, layer["SOURCE"],
                          baseline, package_manifest, package_movie})
    with manifest.open("x") as handle:
        json.dump({"files": [{"path": str(path), "sha256": sha(path)} for path in sorted(paths)]}, handle, indent=2)
        handle.write("\n")
    sys.argv = [__file__, "--phase", "asr", "--job", JOB, "--inputs", str(manifest), "--",
                sys.executable, "-I", "-B", __file__, "execute"]
    raise SystemExit(guard.main())


if __name__ == "__main__":
    main()
