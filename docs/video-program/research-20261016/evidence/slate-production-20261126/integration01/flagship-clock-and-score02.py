"""Build the native clock and score for the revised flagship narration."""

import argparse
from pathlib import Path
import runpy
import shutil
import sys


HERE = Path(__file__).resolve().parent
CLOCK = HERE / "flagship-clock-and-score-common03.py"
VOICE = HERE / "flagship-final-voice02.py"
shared = runpy.run_path(str(CLOCK))
voice = runpy.run_path(str(VOICE))
WORK = voice["WORK"]
EXECUTABLE = voice["REPO"] / "target/debug/probability_native_revised"
PACKAGE = WORK / voice["PACKAGE"]
JOB = "anytime-valid-revised-clock-score01"


def frozen():
    speech, _ = voice["revision"]["revision_cues"]()
    narration = PACKAGE / "narration"
    speech, rows, paths = shared["package_inputs"](narration, voice["SCRIPT"], speech)
    report_path = narration / "report.json"
    compatible = runpy.run_path(str(voice["REPORT"]))["compatible_report"]
    report = voice["read"](report_path)
    assert compatible(report) == report
    package_guard = PACKAGE / "resource-receipt.json"
    render_guard = voice["RENDER"] / "resource-receipt.json"
    repair = voice["BET_VISUAL"]
    repair_guard = repair / "resource-receipt.json"
    binary_receipt = repair / "binary-sha256.txt"
    for receipt in (package_guard, render_guard, repair_guard):
        voice["checked_receipt"](receipt, 0)
    sha, path = binary_receipt.read_text().split()
    assert Path(path) == EXECUTABLE and voice["digest"](EXECUTABLE) == sha
    script = shared["native_list"](EXECUTABLE, "script")
    assert [tuple(line.split("\t", 1)) for line in script] == list(speech.items())
    keys = shared["native_list"](EXECUTABLE, "chapters")
    assert len(set(keys)) == len(keys) == 14
    assert [key[:2] for key in keys] == [f"{n:02d}" for n in range(1, 15)]
    prior = shared["PRIOR"] / "clock60"
    paths.update((Path(__file__).resolve(), CLOCK, VOICE, voice["REVISION"], voice["PROOF"],
                  voice["REPORT"], package_guard, render_guard, repair_guard, binary_receipt, EXECUTABLE,
                  Path(shutil.which("ffprobe")).resolve()))
    paths.update(prior / (key + ".timeline.tsv") for key in keys[:11])
    return narration, speech, rows, keys, paths


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("inspect", "launch", "execute"))
    parser.add_argument("--reuse", nargs="*")
    args = parser.parse_args()
    if args.mode == "inspect":
        _, pending, originals, _, _ = voice["retained"]()
        shared["inspect_preview"]()
        print(f"Revised candidate retains {len(originals)} full WAVs and still needs {len(pending)} exact cues.")
        return
    narration, speech, rows, keys, paths = frozen()
    reuse = shared["clock_reuse"]["bind"]
    entry = Path(__file__).resolve()
    if args.mode == "launch":
        if args.reuse is not None:
            parser.error("The launcher selects reusable clocks from the verified final package.")
        reusable, clock_paths = reuse(WORK, keys, narration, rows, EXECUTABLE, voice["SCRIPT"])
        command = [sys.executable, "-I", "-B", entry, "execute", "--reuse", *reusable]
        voice["guard"](JOB, "native", paths | clock_paths | {shared["CLOCK_REUSE"]}, command)
        return
    if args.reuse is None:
        parser.error("Run the guarded launcher to select and freeze clocks.")
    output = WORK / JOB
    receipt = voice["read"](output / "resource-receipt.json")
    assert receipt["admitted"] and receipt["phase"] == "native" and not receipt.get("finished")
    assert receipt["command"][-(3 + len(args.reuse)):] == [str(entry), "execute", "--reuse", *args.reuse]
    reusable, clock_paths = reuse(WORK, args.reuse, narration, rows, EXECUTABLE, voice["SCRIPT"])
    assert list(reusable) == args.reuse, "A selected clock no longer matches the final voice."
    manifest = WORK / "navier-production-20261120" / f"{JOB}-inputs.json"
    assert voice["digest"](manifest) == receipt["inputs_sha256"]
    bound = {Path(row["path"]): row["sha256"] for row in voice["read"](manifest)["files"]}
    assert all(bound.get(path) == voice["digest"](path) for path in clock_paths)
    shared["run_clock_and_score"](narration, speech, rows, keys, EXECUTABLE, output, reusable=reusable)


if __name__ == "__main__":
    main()
