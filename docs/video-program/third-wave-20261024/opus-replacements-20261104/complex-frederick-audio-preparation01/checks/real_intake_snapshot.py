"""Dry intake of root's real Frederick take folder into scratch. From the repository root:

    python3 -I -B .../complex-frederick-audio-preparation01/checks/real_intake_snapshot.py FRESH_SCRATCH

Runs frederick_intake.py status and plan against EC/frederick-stage05-01 with a lane under FRESH_SCRATCH, then
checks the plan root would get: root's pinned guard re-hashes every frozen input (its check_inputs function
only; nothing is launched), and the runner's fw arguments equal root's proven run01.py except the inputs list.
Reads the take folder, crate, Oliver inventory, guard, fw and model; writes only under FRESH_SCRATCH.
The take folder is still growing, so this is a snapshot of whatever receipts exist when it runs.
"""
import ast
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

REPO = Path.cwd()
HERE = Path(__file__).resolve().parent
TOOL = HERE.parent / "frederick_intake.py"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    scratch = Path(sys.argv[1])
    assert not scratch.exists(), "fresh scratch please"
    scratch.mkdir(parents=True)
    fi = load("frederick_intake", TOOL)
    results = []

    def check(name, ok, detail=""):
        results.append(bool(ok))
        print(f"{'PASS' if ok else 'FAIL'} {name}" + (f": {detail}" if detail else ""))

    def run(*args):
        r = subprocess.run([sys.executable, "-I", "-B", str(TOOL), *args], cwd=REPO, capture_output=True, text=True)
        return r.returncode, r.stdout, r.stderr

    code, out, err = run("status", "--lane", str(scratch / "lane-status"))
    status = json.loads(out)
    cues = sorted({w.split()[0] for w in status["eligible_wavs"]})
    check("status", code == 0 and not err and not (scratch / "lane-status").exists(),
          f"{status['downloads_receipts']} downloads receipts; {len(status['eligible_wavs'])} WAVs eligible in "
          f"{len(cues)} cues ({', '.join(cues)}); refused {len(status['refused'])}; awaiting download "
          f"{status['awaiting_download']}; not submitted {status['not_submitted']} of {status['script_cues']}; "
          f"unreceipted {status['unreceipted_wavs']}; Oliver inventory {status['other_voice_inventory']}")
    for row in status["refused"]:
        print(f"  refused {row['receipt']}: {'; '.join(row['reasons'])}")

    lane = scratch / "frederick-stage05-recognition01"
    code, out, err = run("plan", "--lane", str(lane))
    plan = json.loads((lane / "plan-01/plan.json").read_text(encoding="utf-8"))
    names = [t["name"] for t in plan["takes"]]
    check("plan-01", code == 0 and names == [w.split()[1] for w in status["eligible_wavs"]]
          and [Path(x).name for x in (lane / "plan-01/inputs.txt").read_text(encoding="utf-8").splitlines()] == names,
          f"{len(names)} WAVs in inputs.txt, in plan order")
    frozen_path = lane / "plan-01/frozen-inputs.json"
    frozen = json.loads(frozen_path.read_text(encoding="utf-8"))["files"]
    roles = {}
    for row in frozen:
        roles[row["role"]] = roles.get(row["role"], 0) + 1
    guard = load("guard_wrapper", REPO / fi.GUARD).load()
    check("guard-check-inputs", guard.check_inputs(frozen_path) == hashlib.sha256(frozen_path.read_bytes()).hexdigest(),
          f"root's pinned guard re-hashed {len(frozen)} frozen inputs: {dict(sorted(roles.items()))}")
    for take in plan["takes"]:
        receipt = json.loads((Path(take["path"]).parent / take["receipts"]["downloads"]["name"]).read_text(encoding="utf-8"))
        claimed = next(t for t in receipt["takes"] if Path(t["path"]).name == take["name"])
        ok = (take["channels"], take["sample_rate"], take["sample_bits"]) == (1, 48000, 16) and \
            (claimed["sha256"], claimed["pcm_sha256"], claimed["frames"]) == (take["sha256"], take["pcm_sha256"], take["frames"])
        results.append(ok)
        print(f"{'PASS' if ok else 'FAIL'} {take['name']}: {take['frames']} frames, {take['seconds']:.2f} s, chunks "
              f"{'/'.join(c['id'].strip() for c in take['chunks'])}, peak {take['peak']}, sha {take['sha256'][:12]}, "
              f"pcm {take['pcm_sha256'][:12]}, submission {take['receipts']['submission']['name']}")
    dry = subprocess.run([sys.executable, "-I", "-B", str(lane / "plan-01/run-asr.py"), "--attempt", "1", "--dry-run"],
                         cwd=REPO, capture_output=True, text=True)
    command = json.loads(dry.stdout)
    root_run = (REPO / fi.EL / "remaining-recognition01/run01.py").read_text(encoding="utf-8")
    root_cmd = next(ast.literal_eval(node.value) for node in ast.walk(ast.parse(root_run))
                    if isinstance(node, ast.Assign) and getattr(node.targets[0], "id", "") == "cmd")
    ours, theirs = command[command.index("--") + 1:], root_cmd[root_cmd.index("--") + 1:]
    at = theirs.index("--inputs-from") + 1
    check("run-asr-command", dry.returncode == 0 and ours[:at] + ours[at + 1:] == theirs[:at] + theirs[at + 1:]
          and command[3] == root_cmd[2] and ours[at] == str((lane / "plan-01/inputs.txt").resolve())
          and command[command.index("--job") + 1] == "complex-frederick-stage05-recognition01-plan-01-a1"
          and plan["recognition"]["fw_command"] == ours,
          "root's guard and fw arguments; only the inputs list and job name are new")
    code, out, err = run("plan", "--lane", str(lane))
    check("replan-holds-in-flight", code == 0 and json.loads(out)["plan"] is None and not (lane / "plan-02").exists(),
          "a second plan before any ingest writes nothing")
    print(f"SNAPSHOT: {'PASS' if all(results) else 'FAIL'} ({sum(results)}/{len(results)})")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
