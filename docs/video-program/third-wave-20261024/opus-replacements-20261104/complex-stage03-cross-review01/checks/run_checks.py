"""Rerun every check of complex-stage03-cross-review01.

    python3 -I -B run_checks.py <fresh-scratch-dir> [--verify]

Run from the repo root. Without --verify it writes checks/outputs/*.txt, and refuses once the
bundle is sealed. With --verify it writes only into the scratch folder and says whether each
output is byte-equal to the sealed one. Reads the stage, the patches and the maps; writes nothing
else. Nothing is compiled, rendered, synthesized or heard.
"""
import pathlib
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
OUT = HERE.parent
CL = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
ST = f"{CL}/combined-repair-stage03"
CR = f"{ST}/explainer_corpus/complex_analysis_visual"
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
PATCHES = [
    f"{P}/complex13-radius-repair01/complex13-radius-repair01.patch",
    f"{P}/complex13-radius-repair01/continuation14/complex14-pole-order.patch",
    f"{P}/complex02-05-repair03/complex02-05-repair03.patch",
    f"{P}/complex02-05-cross-review04/rebinding-tex-storyboard.patch",
    f"{P}/complex13-radius-repair01/closing15/complex15-closing-sweep.patch",
    f"{P}/complex11-counting-teaching02/patches/counting-from-root.patch",
    f"{P}/complex11-counting-teaching02/patches/mod-registry.patch",
    f"{P}/complex11-counting-teaching02/patches/storyboard.patch",
    f"{P}/complex11-counting-cross-review01/corrections/C1-after-plate-core.patch",
    f"{P}/complex11-counting-cross-review01/corrections/C2-row-swap-on-its-words.patch",
    f"{P}/complex11-counting-cross-review01/corrections/C2-storyboard.patch",
    f"{P}/complex11-counting-cross-review01/corrections/C3-digit-on-zero.patch",
    f"{CL}/counting-wording-repair01/counting-sentence-shift.rs.patch",
    f"{CL}/counting-wording-repair01/script.tsv.patch",
    f"{P}/complex-glyph-spacing-repair01/complex-glyph-spacing.patch",
    f"{P}/complex06-09-repair03/patches/06-one-derivative.patch",
    f"{P}/complex06-09-repair03/patches/07-one-point.patch",
    f"{P}/complex06-09-repair03/patches/08-loop-sums.patch",
    f"{P}/complex06-09-repair03/patches/09-boundary.patch",
]
OLIVER10 = f"{CR}/docs/narration-direction-map-oliver10.json"
OLIVER11 = f"{P}/complex11-counting-teaching02/holds/narration-direction-map-oliver11.json"


def main():
    scratch = pathlib.Path(sys.argv[1])
    verify = "--verify" in sys.argv
    assert not scratch.exists(), "the scratch folder must be fresh"
    if not verify:
        assert not (OUT / "SHA256SUMS").exists(), "sealed: rerun with --verify"
    scratch.mkdir(parents=True)
    runs = [
        ("hooks_scan", ["hooks_scan.py", CR, f"{CR}/docs/script-as-reviewed-mail622.tsv"], True),
        ("patch_presence", ["patch_presence.py", ST, *PATCHES], True),
        ("holds_check", ["holds_check.py", ST, OLIVER10, OLIVER11], True),
        ("rebind_delta", ["rebind_delta.py", ST, OLIVER11, str(scratch / "hold-rebind-stage03.json")], False),
        ("stage03_checks", ["stage03_checks.py", str(scratch / "stage03")], False),
    ]
    outputs = scratch / "outputs" if verify else HERE / "outputs"
    outputs.mkdir(exist_ok=True)
    bad = []
    for name, argv, report_only in runs:
        r = subprocess.run([sys.executable, "-I", "-B", str(HERE / argv[0]), *argv[1:]], capture_output=True, text=True)
        (outputs / f"{name}.txt").write_text(r.stdout + (r.stderr and "\n[stderr]\n" + r.stderr), encoding="utf-8")
        ok = r.returncode == 0 or report_only
        print(f"{name}: exit {r.returncode}" + ("" if ok else "  FAIL"))
        if not ok:
            bad.append(name)
    proposed = (OUT / "proposed/hold-rebind-stage03.json").read_bytes()
    same = (scratch / "hold-rebind-stage03.json").read_bytes() == proposed
    print("proposed/hold-rebind-stage03.json regenerates byte-equal:", same)
    if not same:
        bad.append("proposed delta")
    if verify:
        for f in sorted(outputs.iterdir()):
            sealed = HERE / "outputs" / f.name
            print(f"{f.name}: equal to sealed output: {sealed.is_file() and sealed.read_bytes() == f.read_bytes()}")
    print("FAILURES: " + (", ".join(bad) if bad else "none"))
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
