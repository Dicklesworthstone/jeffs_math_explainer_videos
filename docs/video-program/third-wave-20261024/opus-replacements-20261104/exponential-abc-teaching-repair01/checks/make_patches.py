"""Write patches/{rate,series,product}.patch from base/ to candidate/. Paths relative to each stage, for patch -p1.

Usage from the repo root:  python3 -I -B make_patches.py
"""
import os
import subprocess

D = "docs/video-program/third-wave-20261024/opus-replacements-20261104/exponential-abc-teaching-repair01/"
STAGES = {"rate": "rate02", "series": "series01", "product": "product01"}
for name, folder in STAGES.items():
    out = []
    base = f"{D}base/{folder}/"
    cand = f"{D}candidate/{folder}/"
    assert sorted(os.listdir(base)) == sorted(os.listdir(cand)), folder
    for f in sorted(os.listdir(base)):
        r = subprocess.run(
            ["diff", "-u", "--label", f"a/{f}", "--label", f"b/{f}", base + f, cand + f],
            capture_output=True,
            text=True,
        )
        assert r.returncode in (0, 1), r.stderr
        out.append(r.stdout)
    with open(f"{D}patches/{name}.patch", "w", encoding="utf-8") as handle:
        handle.write("".join(out))
    print(name, len("".join(out).splitlines()), "lines")
