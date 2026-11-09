from __future__ import annotations

import base64
import gzip
import subprocess
import sys
from pathlib import Path

PATCH = gzip.decompress(base64.b64decode("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"))


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    if not (root / "Cargo.toml").is_file():
        raise SystemExit("activation utility must run from the FrankenManim source tree")
    check = subprocess.run(
        ["git", "apply", "--check", "--whitespace=error-all", "-"],
        cwd=root,
        input=PATCH,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if check.returncode == 0:
        apply = subprocess.run(
            ["git", "apply", "--whitespace=error-all", "-"],
            cwd=root,
            input=PATCH,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        if apply.returncode != 0:
            sys.stderr.buffer.write(apply.stderr)
            return apply.returncode
        print("applied idempotent Animation semantic integration patch")
        return 0
    reverse = subprocess.run(
        ["git", "apply", "--reverse", "--check", "--whitespace=error-all", "-"],
        cwd=root,
        input=PATCH,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if reverse.returncode == 0:
        print("Animation semantic integration is already active")
        return 0
    sys.stderr.write("Animation semantic integration anchors drifted.\n")
    sys.stderr.write("Forward check:\n" + check.stderr.decode("utf-8", "replace"))
    sys.stderr.write("Reverse check:\n" + reverse.stderr.decode("utf-8", "replace"))
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
