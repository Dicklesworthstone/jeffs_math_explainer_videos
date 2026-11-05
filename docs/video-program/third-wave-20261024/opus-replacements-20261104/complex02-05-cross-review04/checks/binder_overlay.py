#!/usr/bin/env python3
"""Does the patched binder clear the 8 problems, change nothing else, and agree with the repair's
hold delta? Run from the repo root:

    python3 -I -B binder_overlay.py REPO_RELATIVE_OUT_DIR

Reads the crate from canonical and from the candidate, with and without the four staged files, in
temp copies. Writes only what it is told under OUT (rebound maps and units for the staged trees).
It runs the binder's `build`; it does not run its `main`, compile anything or read any audio.

Controls: the patched binder must still refuse a changed call, a stale entry and a new post-voice
wait, so the patch cannot pass by switching a check off.
"""
import hashlib
import importlib.util
import json
import shutil
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
O = HERE.parents[1]
REPAIR = O / "complex02-05-repair03"
DELTA = json.loads((REPAIR / "bindings/visual-holds-delta.json").read_text())
REL = Path("explainer_corpus/complex_analysis_visual")
CANDIDATE = Path("narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/combined-picture-stage01/candidate") / REL
MAP = "docs/narration-direction-map-oliver10.json"
NAMES = ("turning", "two_planes", "reading_color", "gallery")
EXTRA = HERE.parent / "rebinding-tex-storyboard.patch"
spec = importlib.util.spec_from_file_location("binder", HERE.parent / "binder/bind_oliver_plan.py")
binder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(binder)
failures = []


def sha(data):
    return hashlib.sha256(data).hexdigest()


def check(ok, text):
    print(("ok   " if ok else "FAIL ") + text)
    if not ok:
        failures.append(text)


def tree(base, staged):
    tmp = Path(tempfile.mkdtemp(prefix="cross04-binder-"))
    shutil.copytree(base, tmp / "crate", ignore=shutil.ignore_patterns("target"))
    if staged:
        for name in NAMES:
            shutil.copyfile(REPAIR / REL / "src/chapters" / f"{name}.rs", tmp / "crate/src/chapters" / f"{name}.rs")
    return tmp / "crate"


def run(crate, version):
    plan, outputs, _audit, _canonical, map_text = binder.build(".", str(crate), version)
    return plan.problems, outputs, json.loads(map_text), map_text


def applied(base_map):
    """The repair's delta, written onto a map exactly as its note says."""
    out = json.loads(json.dumps(base_map))
    rows = {r["cue"]: r for r in out["visual_holds"]}
    for row in DELTA["rows"]:
        target = rows[row["cue"]]
        for field in ("sha256", "line", "marker"):
            if target[field] != row["old"][field]:
                failures.append(f"delta old side {row['cue']} {field}")
            target[field] = row["new"][field]
    return out


def main():
    out_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else None
    for label, base in (("canonical", REL), ("candidate", CANDIDATE)):
        on_disk = (base / MAP).read_bytes()
        problems, _, _, map_text = run(tree(base, False), "oliver10")
        check(not problems, f"{label} without the repair: {len(problems)} binder problems {problems[:3]}")
        check(map_text.encode() == on_disk, f"{label} without the repair: rebuilt oliver10 is byte-equal to the map on disk ({sha(on_disk)[:12]})")
        crate = tree(base, True)
        problems, outputs, rebuilt, map_text = run(crate, "oliver10")
        check(not problems, f"{label} + repair: {len(problems)} binder problems {problems[:3]}")
        by_hand = applied(json.loads(on_disk))
        check(rebuilt == by_hand, f"{label} + repair: binder's map equals the old map with the delta applied")
        if rebuilt != by_hand:
            for section in rebuilt:
                if rebuilt[section] != by_hand.get(section):
                    print("     differs:", section)
                    if section == "visual_holds":
                        for a, b in zip(rebuilt[section], by_hand[section]):
                            if a != b:
                                print("      ", a["cue"], {k: (a.get(k), b.get(k)) for k in a if a.get(k) != b.get(k)})
                    elif section == "helpers":
                        for a, b in zip(rebuilt[section], by_hand[section]):
                            if a != b:
                                print("      ", {k: (a.get(k), b.get(k)) for k in a if a.get(k) != b.get(k)})
        for row in rebuilt["visual_holds"]:
            path = crate / "src/chapters" / Path(row["path"]).name
            lines = path.read_text(encoding="utf-8").split("\n")
            if lines[row["line"] - 1].strip() != row["marker"] or sha(path.read_bytes()) != row["sha256"]:
                failures.append(f"{label} {row['cue']} marker or hash")
        print(f"     {label} + repair: all {len(rebuilt['visual_holds'])} hold rows land on their markers and hashes")
        units = json.loads(outputs[f"{binder.TOPIC_REL}/narration_units.json"])
        print(f"     units' contract pins the map at {units['direction_contract']['map']['sha256'][:12]}, not the old {sha(on_disk)[:12]}: the units file must change with the map")
        check(units["direction_contract"]["map"]["sha256"] == sha(map_text.encode()), f"{label} + repair: units pin the rebuilt map")
    # Controls on a temp copy, always on canonical + repair.
    for label, path, old, new, want in (
        ("a changed post-voice beat", "closing.rs", "pause(stage, 1.0)?;", "pause(stage, 2.5)?;", "unclassified post-voice hold"),
        ("a stale NAMED phrase", "reading_color.rs", '("pointing down is violet", 3.0 * FRAC_PI_2),', '("pointing down is blue", 3.0 * FRAC_PI_2),', "DYNAMIC's source line is gone"),
        ("a changed walk phrase", "gallery.rs", '"backwards twice",', '"run backwards",', "no longer a call's literal"),
        ("a changed needs list", "counting.rs", "line.wait_for(stage, NEEDS_SAID[i])?;", "line.wait_for(stage, NEEDS_SAID[i + 0])?;", "nobody has resolved"),
    ):
        crate = tree(REL, True)
        target = crate / "src/chapters" / path
        text = target.read_text(encoding="utf-8")
        if old not in text:
            check(False, f"control {label}: its anchor is missing")
            continue
        target.write_text(text.replace(old, new), encoding="utf-8")
        problems, _, _, _ = run(crate, "oliver10")
        check(any(want in p for p in problems), f"control {label}: refused ({want})")
    if out_dir:
        for label, base in (("canonical", REL), ("candidate", CANDIDATE)):
            crate = tree(base, True)
            if EXTRA.exists():
                import subprocess

                done = subprocess.run(["git", "apply", "--whitespace=nowarn", "-p3", str(EXTRA.resolve())], cwd=crate, capture_output=True, text=True)
                check(done.returncode == 0, f"{label}: rebinding-tex-storyboard.patch applies at zero fuzz {done.stderr[:200]}")
            problems, outputs, _, _ = run(crate, "oliver11")
            check(not problems, f"{label} + repair + TEX/storyboard patch: oliver11 builds with {len(problems)} problems")
            for rel, text in outputs.items():
                target = out_dir / label / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(text, encoding="utf-8")
            print(f"     wrote {out_dir / label}")
    print(f"{len(failures)} failures")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
