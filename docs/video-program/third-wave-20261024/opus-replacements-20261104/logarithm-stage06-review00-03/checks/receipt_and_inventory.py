"""Bind this review to root's terminal receipts and list exactly which pictures exist.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage06-review00-03/checks/receipt_and_inventory.py

Read only outside this folder. It proves that the receipts are terminal and that the sheets, frames and sampling
tables agree; it proves nothing about what the pictures show. Which sheets came back to me as images is in
coverage.tsv, written by viewing_proof.py from this session's transcript.
"""
import hashlib
import json
import os
import sys

ROOT = os.getcwd() + "/"
B = "narration_lab/third-wave-20261024/root/five-film-release-20261027/"
PREVIEW, FRAMES = B + "logarithm-stage06-preview01/", B + "logarithm-stage06-frames01/"
STAGE = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/stage06-preview01/"
CHAPTERS = ["00_ratio_ruler", "01_building_ruler", "02_slide_rule", "03_folded_decade"]
SOURCES = ["ratio_ruler.rs", "building_ruler.rs", "slide_rule.rs", "folded_decade.rs", "draw.rs"]
C = "explainer_corpus/logarithm_mysteries/"
fails = 0


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


for name, folder in (("preview01", PREVIEW), ("frames01", FRAMES)):
    r = json.load(open(folder + "resource-receipt.json", encoding="utf-8"))
    check(r.get("exit") == 0 and r.get("finished") and r.get("inputs_sha256") == r.get("final_inputs_sha256"), f"{name}: exit {r.get('exit')}, finished {r.get('finished')}, inputs unchanged {r.get('inputs_sha256', '')[:16]}")
    check(r.get("admitted") is True and not r.get("preflight_only"), f"{name}: admitted, not preflight only")

inputs = {f["path"].replace(ROOT, ""): f["sha256"] for f in json.load(open(STAGE + "inputs.json", encoding="utf-8"))["files"]}
frame_inputs = {f["path"].replace(ROOT, ""): f["sha256"] for f in json.load(open(STAGE + "frames-inputs.json", encoding="utf-8"))["files"]}
for name in [C + "script.tsv"] + [C + "src/chapters/" + s for s in SOURCES]:
    check(inputs.get(name) == sha(name), f"{name[len(C):]} {sha(name)[:16]} is what preview01 read")

total_frames = total_sheets = 0
print("chapter\tduration_s\tselected\tframes\tsheets\tfindings\tmp4_sha256\tcues_sha256\tlayout_sha256\tsheets_digest")
for ch in CHAPTERS:
    folder = FRAMES + ch + "/"
    s = json.load(open(folder + "sampling.json", encoding="utf-8"))
    names = sorted(os.listdir(folder))
    frames = [n for n in names if n.startswith("frame-")]
    sheets = [n for n in names if n.startswith("sheet-")]
    digest = hashlib.sha256("".join(sha(folder + n) for n in sheets).encode()).hexdigest()
    render = PREVIEW + "render/" + ch
    print(f"{ch}\t{s['duration']}\t{len(s['times'])}\t{len(frames)}\t{len(sheets)}\t{len(s['layout_findings'])}\t{sha(render + '.mp4')[:16]}\t{sha(render + '.cues.tsv')[:16]}\t{sha(render + '.layout.tsv')[:16]}\t{digest[:16]}")
    check(len(s["times"]) == len(frames) and (len(frames) + 3) // 4 == len(sheets), f"{ch}: {len(frames)} frames on {len(sheets)} sheets, all in sampling.json")
    check(frame_inputs.get(render + ".mp4") == sha(render + ".mp4"), f"{ch}: the frames were cut from this video")
    gaps = [b - a for a, b in zip(s["times"], s["times"][1:])]
    check(max(gaps) <= 0.5 + 1e-6, f"{ch}: widest gap between selected pictures {max(gaps):.3f} s; end sample {s['times'][-1]} of {s['duration']}")
    total_frames += len(frames)
    total_sheets += len(sheets)
check(total_sheets == 255 and total_frames == 1016, f"in scope: {total_sheets} sheets, {total_frames} selected pictures")
print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
