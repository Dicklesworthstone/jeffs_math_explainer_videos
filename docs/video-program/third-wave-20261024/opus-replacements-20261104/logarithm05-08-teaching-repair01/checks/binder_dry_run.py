"""Run the sealed stage07 binder01's build() on a scratch mirror of its inputs, with and without this repair.

    python3 -I -B binder_dry_run.py <fresh-scratch-dir>

Run from the repo root. Reads the binder and every file it pins; writes only inside the scratch
folder, which must not exist yet. Its build() returns its outputs and writes nothing, and this keeps
only what it prints. It edits neither the binder nor any binder output: the proposed rows are set on
the module in memory. The binder wants an integration receipt naming every changed file's hash; root
writes that when it integrates, so each mirror holds a stand-in written here, under the name proposed.

binder01 pins folded_decade.rs c4f023ce, from before root's partial06. As in its own handoff, each
mirror uses root's preserved before-oliver-digits-mark01-folded_decade.rs, so the binder sees the
inputs it pins. This repair touches none of that file. binder02, sealed, binds the new one and pins
this repair's four files at their stage07 bytes; the later update built on it needs the same row sets.
binder01 stops on any new phrase anchor, and binder02 has an ADDED_ANCHORS slot for that, so here the
eight new anchors are kept out of the stage06 comparison. That is the slot's rule, and it is the only
change made to how the binder runs.

Four runs, each in its own mirror:

  baseline               binder01's exact inputs, its pins untouched: must build
  staged, binder as is   this repair's four files: must stop, or the rows below are not needed
  staged, proposed rows  this repair's four files, PHRASE_MARK_USES, FALLBACK, SITES, RECEIPTS and added anchors
  plant                  the same with a hook moved ahead of an earlier one: must stop

A pass says the text rules agree. Nothing is compiled, rendered, measured or heard.
"""
import hashlib
import importlib.util
import json
import pathlib
import shutil
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
OUT = HERE.parent
REPO = pathlib.Path.cwd()
BINDER = "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage07-binder01/bind_final80-stage07.py"
BINDER_SHA = "4de1eec4f611aec3b1a14a68e08f43d71e271f53d9f8b92cfe1fb63a7c8660be"
PRESERVED = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage07/before-oliver-digits-mark01-folded_decade.rs"
FILES = ("complex_turns.rs", "lifting_loops.rs", "branch_cut.rs", "product_rule.rs")
RUNNER = r'''
import importlib.util, json, sys
from pathlib import Path
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("binder_under_test", sys.argv[1])
binder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(binder)
rows = json.loads(sys.argv[2])
binder.PHRASE_MARK_USES.update(rows["phrase_mark_uses"])
binder.FALLBACK.update(rows["fallback"])
for name, line, kind, note in rows["sites"]:
    binder.SITES[(name, line)] = (kind, note)
binder.RECEIPTS.update({receipt: tuple(files) for receipt, files in rows["receipts"].items()})
added = {tuple(row) for row in rows["added_anchors"]}
if added:
    compare = binder.stage06_compare
    def without_added(*args):
        doc = args[6]
        if not added <= {(r["cue"], r["phrase"]) for r in doc["anchors"]}:
            raise SystemExit("an added anchor is not in the source")
        kept = {**doc, "anchors": [r for r in doc["anchors"] if (r["cue"], r["phrase"]) not in added]}
        return compare(*args[:6], kept, *args[7:])
    binder.stage06_compare = without_added
if not rows["keep_pins"]:
    binder.CH_LISTING = tuple(sorted(p.name for p in binder.CH.iterdir()))
    binder.PINS = {p: binder.file_sha(p) for p in binder.pin_paths() if Path(p).is_file()}
    binder.EXPECTED_SCRIPT = binder.file_sha(binder.STAGE / "script.tsv")
try:
    binder.refuse_stale_inputs()
    built = binder.build()
except SystemExit as stop:
    print("STOP:", stop)
    sys.exit(0)
print("BUILT")
'''


def load_binder():
    spec = importlib.util.spec_from_file_location("binder_for_paths", REPO / BINDER)
    binder = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(binder)
    return binder


def mirror(binder, where):
    where.mkdir(parents=True)
    (where / "mirror-tree.marker").write_text("scratch mirror\n", encoding="utf-8")
    shutil.copytree(REPO / binder.CH, where / binder.CH)
    shutil.copyfile(REPO / PRESERVED, where / binder.CH / "folded_decade.rs")
    # the request texts: pinned by root's own files, so the binder's pin list leaves them out
    for folder in ("narration-request-text01", "narration-request-final80-01", "narration-request-rounding01"):
        shutil.copytree(REPO / binder.LANE / folder, where / binder.LANE / folder, dirs_exist_ok=True)
    for rel in binder.pin_paths():
        source, target = REPO / rel, where / rel
        if rel.endswith("candidate/explainer_corpus/logarithm_mysteries/src/chapters/folded_decade.rs"):
            continue
        if source.is_file() and not target.exists():
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
    return where / binder.CH


def stand_in_receipts(binder, where, rows):
    crate = where / binder.STAGE
    for receipt, files in rows["receipts"].items():
        body = {name: hashlib.sha256((crate / name).read_bytes()).hexdigest() for name in files}
        (where / binder.STAGE07_ROOT / receipt).write_text(json.dumps({"stand_in_for_this_dry_run_only": True, "files": body}, indent=2) + "\n", encoding="utf-8")


def run(name, where, binder_file, rows):
    result = subprocess.run(
        [sys.executable, "-I", "-B", "-c", RUNNER, str(binder_file), json.dumps(rows)],
        cwd=where, capture_output=True, text=True,
    )
    tail = (result.stdout.strip() or result.stderr.strip()[-400:]).splitlines()[-1]
    print(f"{name}\t{tail}")
    return tail


def main():
    scratch = pathlib.Path(sys.argv[1])
    assert not scratch.exists(), "the scratch folder must be fresh"
    binder = load_binder()
    binder_file = REPO / BINDER
    proposed = json.loads((OUT / "binder-rows.json").read_text(encoding="utf-8"))
    proposed["keep_pins"] = False
    no_rows = {"phrase_mark_uses": {}, "fallback": {}, "sites": [], "receipts": {}, "added_anchors": [], "keep_pins": False}
    assert hashlib.sha256(binder_file.read_bytes()).hexdigest() == BINDER_SHA, "binder01 is no longer the sealed bytes"
    print("binder01 sha256", BINDER_SHA)

    base = scratch / "baseline"
    mirror(binder, base)
    first = run("baseline, pins untouched", base, binder_file, {**no_rows, "keep_pins": True})

    staged = scratch / "staged"
    chapters = mirror(binder, staged)
    for name in FILES:
        shutil.copyfile(OUT / "explainer_corpus/logarithm_mysteries/src/chapters" / name, chapters / name)
    stand_in_receipts(binder, staged, proposed)
    second = run("staged, binder as is", staged, binder_file, no_rows)
    third = run("staged, proposed rows", staged, binder_file, proposed)

    plant = scratch / "plant"
    chapters = mirror(binder, plant)
    for name in FILES:
        shutil.copyfile(OUT / "explainer_corpus/logarithm_mysteries/src/chapters" / name, chapters / name)
    planted = chapters / "complex_turns.rs"
    text = planted.read_text(encoding="utf-8")
    a = 'until_phrase(&kit, stage, "05_20", TAKE_X_PHRASE)?;'
    b = 'until_phrase(&kit, stage, "05_20", LAND_PHRASE)?;'
    assert text.count(a) == 1 and text.count(b) == 1
    planted.write_text(text.replace(a, "@@").replace(b, a).replace("@@", b), encoding="utf-8")
    stand_in_receipts(binder, plant, proposed)
    fourth = run("plant: hooks out of spoken order", plant, binder_file, proposed)

    assert first == "BUILT", first
    assert second.startswith("STOP"), second
    assert third == "BUILT", third
    assert fourth.startswith("STOP"), fourth
    print("OK")


if __name__ == "__main__":
    main()
