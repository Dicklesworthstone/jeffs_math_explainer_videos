"""BronzeJaguar's sealed run_checks.py run against the staged helper. From the repository root:

    python3 -I -B .../logarithm-stage09-mark-interval-fix01/checks/bronze_suite_on_staged.py FRESH_SCRATCH

Bronze's suite is imported unchanged from its sealed folder (logarithm-stage09-audio-preparation01, SHA256SUMS
2bb8e280...). The only change is in memory: its HELPER constant names the staged helper of this bundle, so every
helper run in the suite (report, decode by Bronze's labelled stand-in decimator, prepare, timings and the planted
faults) runs the staged code. Bronze's suite writes only under FRESH_SCRATCH, in a mirror of read-only links; it
never runs ffmpeg on a recording, and its selections and answers are test fixtures. Nothing here is production
audio, a choice of take or a listening check.
"""
import hashlib
import importlib.util
import sys

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
SUITE = f"{P}/logarithm-stage09-audio-preparation01/checks/run_checks.py"
SUITE_SHA256 = "85ff6c4206e5b43f712cd02d1ef8ede11b6d8a2510e80e74ebe9fd9c4a056b1d"  # as Bronze's SHA256SUMS lists it
STAGED = f"{P}/logarithm-stage09-mark-interval-fix01/staged/prepare_audio.py"


def main():
    if len(sys.argv) != 2:
        print("usage: from the repository root, bronze_suite_on_staged.py FRESH_SCRATCH", file=sys.stderr)
        return 2
    with open(SUITE, "rb") as f:
        if hashlib.sha256(f.read()).hexdigest() != SUITE_SHA256:
            print(f"refused: {SUITE} is not the sealed suite", file=sys.stderr)
            return 2
    spec = importlib.util.spec_from_file_location("bronze_run_checks", SUITE)
    suite = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(suite)
    suite.HELPER = STAGED
    print(f"Bronze's run_checks.py {SUITE_SHA256[:12]}, HELPER -> {STAGED}")
    sys.argv = [SUITE, sys.argv[1]]
    return suite.main()


if __name__ == "__main__":
    sys.exit(main())
