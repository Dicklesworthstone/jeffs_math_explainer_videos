"""Run LavenderPike's sealed checks/run_checks.py unchanged, with its HELPER pointed at the staged fix.

BronzeJaguar, bead 87do.63. From the repository root:

    python3 -I -B P/logarithm-stage09-final-verification-review01/checks/lavender_checks_on_fix.py NEW_SCRATCH W

Her module is loaded from her sealed folder and only its HELPER constant is replaced. Everything else (her fixtures,
her real Nov 2 inputs, her emulated guard receipt) is hers, unchanged; it writes only under NEW_SCRATCH.
"""
import importlib.util
import sys
from pathlib import Path

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
spec = importlib.util.spec_from_file_location("run_checks_sealed", f"{P}/logarithm-stage09-final-verification01/checks/run_checks.py")
checks = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checks)
checks.HELPER = Path.cwd() / P / "logarithm-stage09-final-verification-review01/verify_stage09_final_fix01.py"
sys.argv = ["run_checks.py", sys.argv[1], "--w", sys.argv[2]]
sys.exit(checks.main())
