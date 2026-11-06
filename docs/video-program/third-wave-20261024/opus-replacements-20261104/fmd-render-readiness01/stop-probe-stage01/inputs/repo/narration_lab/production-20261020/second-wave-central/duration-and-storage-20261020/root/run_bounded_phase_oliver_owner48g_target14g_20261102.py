"""Root's serialized expanded-review lane; refuse before launching without admission.

This runner only supplies resource and input-binding checks. It does not qualify
audio, mathematical content, timelines, pictures, or publication.
"""
import argparse
import datetime
import fcntl
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[4]
WORK = ROOT / "expanded-review-01"
TARGET = REPO / "target"
GIB = 1024 ** 3
CAPS = {"native": 4, "tts": 1, "asr": 1, "preview": 4}
# Owner 2026-10-21: replace the speculative 232 GiB reserve with bounded
# artifact estimates and a low-space guard. Prior receipts keep the old policy.
ADMISSION_GIB = {"native": 20, "tts": 12, "asr": 12, "preview": 20}
STOP_FREE_GIB = 8
# Owner explicitly waived the 20 GB requirement for the new videos. Keep a
# bounded 48 GiB work ceiling and all other independent resource guards.
WORK_CAP_GIB = 48


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def allocated(root):
    """Allocated bytes, deduplicating hard links; never follow symlinks."""
    seen = set()
    total = 0
    for parent, dirs, files in os.walk(root, followlinks=False):
        dirs[:] = [d for d in dirs if not (Path(parent) / d).is_symlink()]
        for name in files:
            path = Path(parent) / name
            try:
                stat = path.lstat()
            except FileNotFoundError:
                continue  # Cargo may atomically replace a temporary file.
            if path.is_symlink() or (stat.st_dev, stat.st_ino) in seen:
                continue
            seen.add((stat.st_dev, stat.st_ino))
            total += stat.st_blocks * 512
    return total


def sample():
    s = os.statvfs(REPO)
    return {"utc": utc(), "free_bytes": s.f_bavail * s.f_frsize,
            "work_bytes": allocated(WORK), "target_bytes": allocated(TARGET)}


def check_inputs(manifest):
    rows = json.loads(manifest.read_text())["files"]
    if not rows:
        raise ValueError("empty reviewed input manifest")
    for row in rows:
        path = Path(row["path"])
        if not path.is_absolute():
            raise ValueError("manifest paths must be absolute")
        with path.open("rb") as handle:
            digest = hashlib.file_digest(handle, "sha256").hexdigest()
        if digest != row["sha256"]:
            raise ValueError(f"reviewed input changed: {path}")
    return hashlib.sha256(manifest.read_bytes()).hexdigest()


def stop_owned(child):
    if child.poll() is None:
        os.killpg(child.pid, signal.SIGTERM)
        try:
            child.wait(timeout=20)
        except subprocess.TimeoutExpired:
            if child.poll() is None:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait(timeout=10)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", choices=CAPS, required=True)
    p.add_argument("--job", required=True)
    p.add_argument("--inputs", type=Path)
    p.add_argument("--preflight-only", action="store_true")
    p.add_argument("command", nargs=argparse.REMAINDER)
    args = p.parse_args()
    if not args.job or any(c not in "abcdefghijklmnopqrstuvwxyz0123456789-_" for c in args.job):
        p.error("job must be a simple lowercase directory name")
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not args.preflight_only and (not command or not args.inputs):
        p.error("execution requires an explicit command and reviewed input manifest")
    WORK.mkdir(exist_ok=True)
    with (ROOT / "expanded-native-lane.lock").open("a") as lock:
        # Root producers may queue independent topics, but only one native,
        # synthesis, ASR or render process group may run. Check admission and
        # the frozen inputs after acquiring the lane, not while waiting.
        fcntl.flock(lock, fcntl.LOCK_EX)
        job = WORK / args.job
        job.mkdir()  # Every attempt retains its own immutable directory.
        receipt = {"started": utc(), "phase": args.phase, "command": command,
                   "preflight_only": args.preflight_only, "samples": []}
        def save():
            (job / "resource-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
        cap = CAPS[args.phase] * GIB
        need = ADMISSION_GIB[args.phase]
        gate = subprocess.run(["sbh", "check", "--target-free", "0.2", "--need", f"{need}G", "--json"],
                              cwd=REPO, text=True, capture_output=True, timeout=45)
        receipt["sbh"] = {"exit": gate.returncode, "stdout": gate.stdout, "stderr": gate.stderr}
        before = sample()
        receipt["admission"] = before
        receipt["policy"] = {"admission_gib": need, "stop_free_gib": STOP_FREE_GIB,
                             "phase_growth_gib": CAPS[args.phase], "work_cap_gib": WORK_CAP_GIB,
                             "target_growth_cap_gib": 14, "sample_seconds": 10}
        baseline_path = WORK / "allocation-baseline.json"
        if baseline_path.exists():
            baseline = json.loads(baseline_path.read_text())
        else:
            baseline = before
        admitted = (gate.returncode == 0 and before["free_bytes"] >= need * GIB
                    and before["work_bytes"] < WORK_CAP_GIB * GIB
                    and before["target_bytes"] - baseline["target_bytes"] <= 14 * GIB)
        receipt["admitted"] = admitted
        if not admitted or args.preflight_only:
            receipt["finished"] = utc()
            save()
            print(json.dumps({"admitted": admitted, "launched": False, "receipt": str(job)}))
            return 0 if admitted else 75
        receipt["inputs_sha256"] = check_inputs(args.inputs)
        if not baseline_path.exists():
            with baseline_path.open("x") as handle:
                json.dump(baseline, handle, indent=2)
        env = dict(os.environ, RCH_SHIM_LOCAL_IDE="1", RUSTUP_TOOLCHAIN="nightly-2026-08-31",
                   CARGO_TARGET_DIR=str(TARGET), CARGO_BUILD_JOBS="2", RAYON_NUM_THREADS="4",
                   FW_ENC_ATTN_OUT_I8I32="0")
        save()
        child = None
        try:
            with (job / "command.log").open("x") as log:
                child = subprocess.Popen(command, cwd=REPO, env=env, stdout=log,
                                         stderr=subprocess.STDOUT, start_new_session=True)
                receipt["pid"] = child.pid
                while True:
                    now = sample()
                    receipt["samples"].append(now)
                    growth = max(0, now["work_bytes"] - before["work_bytes"]) + max(0, now["target_bytes"] - before["target_bytes"])
                    if (now["free_bytes"] < STOP_FREE_GIB * GIB or growth > cap
                        or now["work_bytes"] > WORK_CAP_GIB * GIB
                        or now["target_bytes"] - baseline["target_bytes"] > 14 * GIB):
                        receipt["stopped_for_resource_limit"] = True
                        stop_owned(child)
                        break
                    save()
                    if child.poll() is not None:
                        break
                    try:
                        child.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        pass
                receipt["exit"] = child.returncode
            receipt["final_inputs_sha256"] = check_inputs(args.inputs)
            if receipt["final_inputs_sha256"] != receipt["inputs_sha256"]:
                raise ValueError("input manifest changed while command ran")
        except BaseException as error:
            if child is not None:
                stop_owned(child)
            receipt["error"] = repr(error)
            raise
        finally:
            receipt["finished"] = utc()
            save()
        print(json.dumps({"exit": receipt["exit"], "resource_stop": receipt.get("stopped_for_resource_limit", False), "receipt": str(job)}))
        return 75 if receipt.get("stopped_for_resource_limit") else receipt["exit"]


if __name__ == "__main__":
    raise SystemExit(main())
