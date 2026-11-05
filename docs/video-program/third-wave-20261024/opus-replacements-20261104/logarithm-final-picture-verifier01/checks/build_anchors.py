"""Writes anchors.json: every flagged item, as a base on the schedule plus frame offsets. From the repository root:

    python3 -I -B .../logarithm-final-picture-verifier01/checks/build_anchors.py OUT.json

OUT must not exist. Each old flagged frame is converted once, here, on the real stage09 schedule, into the sentence,
hold, voice end or chapter start that holds it. `was` keeps that old frame for people and for run_checks.py G3; the
helper never reads it. The repair anchors are new and have no old frame. Item lists come from the sealed reviews:
pane 167 (sheets 001-112), pane 168 (113-225), pane 169 (226-339), the join review, root's sample and root's
dispositions. The notes are short labels for sheet captions, ASCII only; the reviews hold the findings.
"""
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import stage09_data as D  # noqa: E402

V = D.helper()

# finding, caption note, where. frame:N, seconds:T, mid:A-B, sheet:N, sheets:A-B, picture:N, pictures:A-B, frames:A-B
FLAGGED = [
    # pane 167
    ("p167-O1", "answer led sentence 6", ["sheets:107-108", "frame:36614", "frame:36644", "seconds:610.73", "seconds:618.70"]),
    ("p167-O2", "C 10 tight on right edge", ["sheets:56-59", "frame:18914"]),
    ("p167-O3", "C left end off frame", ["sheets:60-65", "frame:19766"]),
    ("p167-C1", "heavy operator glyphs", ["frame:28980", "frame:37256"]),
    ("p167-C2", "upright caption variables", ["sheet:95", "sheets:97-100", "sheets:111-112"]),
    ("p167-C3", "0 before its label", ["frame:22412", "frame:22430"]),
    ("p167-U1", "was a sampling gap", ["mid:10.37-19.92", "mid:610.93-618.70", "mid:542.42-549.57", "mid:593.20-600.23"]),
    ("p167-U1b", "line at -0.84, red, count 0", ["seconds:485.00"]),
    ("p167-seen", "concern closed by a full frame", ["frame:2745", "frame:5755", "seconds:103.00", "seconds:105.50"]),
    ("p167-fade", "cross-fade ghost", ["seconds:212.28", "seconds:539.98"]),
    ("p167-judge", "visible during hold, judged", ["sheets:49-52", "sheets:103-105"]),
    ("p167-answer", "first answer after a hold",
     ["seconds:82.35", "seconds:87.23", "seconds:186.28", "sheet:34", "seconds:297.40", "seconds:600.73"]),
    # pane 168 (O1, the missing pause line, is the pause rows below)
    ("p168-O2", "ln 2 approx, was = 0.693", ["pictures:558-571"]),
    ("p168-O3", "bar under question, was on axis", ["frame:44878"]),
    ("p168-O4", "gap before pi i in readout", ["picture:578", "picture:598"]),
    ("p168-O5", "teal circle past axis end", ["picture:623", "picture:634"]),
    ("p168-O6", "small alignment item", ["picture:743", "picture:779", "picture:866", "picture:880"]),
    ("p168-U1", "06_2 hold may telegraph", ["frame:50777"]),
    ("p168-U2", "k whole wording", ["sheet:139", "sheet:169"]),
    ("p168-U4", "may read as abrupt", ["sheet:190", "sheet:199"]),
    ("p168-U5", "rewind shows no gap dot", ["frame:56012"]),
    ("p168-U6", "Log i ahead of the voice?", ["picture:722"]),
    ("p168-U7", "readout near +-1 unseen", ["sheet:196"]),
    ("p168-seen", "candidate dismissed on a 4K crop",
     ["frame:44781", "frame:46800", "frame:43469", "frame:57585", "frame:65563", "frame:66445", "frame:77178", "frame:64363", "frame:74244"]),
    ("p168-answer", "first answer after a hold",
     ["seconds:654.30", "seconds:656.35", "seconds:752.22", "seconds:852.45", "seconds:892.38", "seconds:1067.57", "seconds:1069.98",
      "seconds:1186.30", "seconds:1190.57", "seconds:1265.55"]),
    # pane 169
    ("p169-F3", "old stop, no closing hold", ["frame:116197", "frame:116219", "frame:116244"]),
    ("p169-F1", "det before it is spoken", ["frame:91051", "frame:92858", "sheet:264", "sheet:271", "sheet:272", "seconds:1556.3"]),
    ("p169-F2", "X with e^X = A, was log A", ["sheets:309-311"]),
    ("p169-M1", "concrete X must be labelled", ["sheets:285-289"]),
    ("p169-P1", "grid cut on vertical lines", ["sheets:233-237"]),
    ("p169-P2", "-0.97 with the zero dropped", ["sheet:234"]),
    ("p169-P3", "text past title-safe line", ["sheet:240", "sheet:241", "sheet:254", "sheet:256", "sheet:284", "sheet:295"]),
    ("p169-P4", "clipped flow loses outer edge", ["frame:83946", "frame:84114", "frame:84444", "frame:85187", "frame:86311", "frame:86551"]),
    ("p169-P5", "arrow along the yellow arm", ["sheets:275-279"]),
    ("p169-P6", "weakest label contrast",
     ["picture:1125", "picture:1168", "picture:1199", "picture:1221", "picture:1289", "picture:1295", "picture:1356"]),
    ("p169-P8", "old rows beside the new", ["sheets:300-305"]),
    ("p169-H1", "11_20 row walk", ["frames:103972-105085"]),
    ("p169-U2", "footer may meet captions", ["sheets:258-260", "sheets:271-273", "sheets:303-311", "sheets:337-339"]),
    ("p169-U4", "stretched state between tiles", ["sheet:229"]),
    ("p169-answer", "first answer after a hold", ["frame:95439", "sheet:330"]),
    # join review: hearing items, placed here so that the ledger gives a listener the new times
    ("join-J1", "for a listener: turned or turn", ["seconds:307.72", "seconds:308.10"]),
    ("join-J2", "for a listener: a small step?", ["seconds:537.48", "seconds:537.50"]),
    ("join-J3", "for a listener: end of 04_4", ["seconds:538.63", "seconds:538.86"]),
    ("join-J4", "for a listener: start of 04_3", ["seconds:487.15"]),
    # root's own views
    ("root-seen", "root viewed this sheet", ["sheet:112", "sheet:113", "sheet:135", "sheet:140", "sheet:226", "sheet:309", "sheet:339"]),
]

# No old frame: these are the repairs. Sentence numbers are 1-based, as on the sheets; the kit's until_sentence(j) is j + 1.
REPAIRS = [
    ("rep-04-reveal", "sentence:04_7:5", [30, 60], "04_7 fraction only, no answer yet"),
    ("rep-04-reveal", "sentence:04_7:6", [-12, 0, 12, 24, 30, 60], "04_7 answer reveal"),
    ("rep-04-reveal", "voice_end:04_7", [0, 30], "04_7 after the voice, new tail"),
    ("rep-11-gloss", "sentence:11_1:5", [0, 30, 60], "11_1 det gloss fading in"),
    ("rep-11-gloss", "phrase:11_1:a halfway point", [0, 30], "11_1 gloss beside the halfway point"),
    ("rep-11-gloss", "phrase:11_1:squashes the whole plane", [0, 30], "11_1 gloss with avg_b moved"),
    ("rep-11-gloss", "cue:11_2", [-12, 12, 30], "11_1 gloss clears at 11_2"),
    ("rep-11-gloss", "sentence:11_2:6", [30, 60], "11_2 footer: det, trace gloss"),
    ("rep-11-gloss", "phrase:11_2:run backwards", [0, 60], "11_2 before the footer"),
    ("rep-11-sample", "phrase:11_5:send it along both routes", [0, 12, 24, 60, 120], "11_5 here X is a new sample"),
    ("rep-11-sample", "sentence:11_5:5", [-12, 0, 78, 156, 180], "11_5 sample X card, arrows run"),
    ("rep-11-sample", "phrase:11_5:the two routes end in different places", [0, 60], "11_5 routes end apart"),
    ("rep-11-formula", "sentence:11_8:2", [0, 24, 60, 120], "11_8 a complex logarithm, e^X = A"),
    ("rep-11-formula", "sentence:11_8:4", [0, 30, 60], "11_8 singular, e^trace X"),
    ("rep-11-formula", "chapter_end:11_unreachable", [-60, -1], "11_8 last frames of the chapter"),
    ("rep-12-closing", "voice_end:12_21", [0, 12, 24, 36], "12_21 last word, family fade"),
    ("rep-12-closing", "closing:hold_start", [0, 30, 165], "closing hold"),
    ("rep-12-closing", "closing:hold_end", [-1, 0, 22, 45, 67], "closing fade"),
    ("rep-12-closing", "closing:end", [-1], "last frame of the film"),
]


def frames_of(where, pictures):
    kind, _, value = where.partition(":")
    if kind == "frame":
        return [int(value)]
    if kind == "seconds":
        # the reviews round to 0.01 s: take the sampled picture they name when there is one
        near = [p["frame"] for p in pictures if abs(p["seconds"] - float(value)) <= 0.0051]
        return [near[0] if near else round(float(value) * V.FPS)]
    if kind == "mid":
        a, b = value.split("-")
        return [round((float(a) + float(b)) / 2 * V.FPS)]
    a, _, b = value.partition("-")
    low, high = int(a), int(b or a)
    if kind in ("sheet", "sheets"):
        return [p["frame"] for p in pictures[(low - 1) * 4:high * 4]]
    if kind in ("picture", "pictures"):
        return [p["frame"] for p in pictures[low - 1:high]]
    if kind == "frames":
        return [p["frame"] for p in pictures if low <= p["frame"] <= high]
    raise SystemExit(f"unknown place {where}")


def main():
    if len(sys.argv) != 2 or Path(sys.argv[1]).exists():
        raise SystemExit("usage: build_anchors.py NEW_OUTPUT.json")
    data = D.load(V)
    pictures, placements, chapters = data["v"]["pictures"], data["placements"], data["chapters"]
    # the reviews' cross-references, as a check that sheets and pictures are read the way they were numbered
    assert pictures[558]["frame"] == 47005 and pictures[597]["frame"] == 50777 and round(pictures[721]["seconds"], 2) == 1022.53
    sites, stray = V.hold_windows(chapters, placements, data["timelines"], data["holds"])
    assert not stray and len(sites) == 15

    def rebase(f):
        chapter = next(x for x in chapters if x["start_frame"] <= f < x["start_frame"] + x["frames"])
        for s in sites.values():
            if s["start_frame"] <= f < s["next_frame"]:
                return f"hold:{s['cue']}:start", f - s["start_frame"]
        earlier = [p for p in placements if p["chapter"] == chapter["chapter"] and p["start_frame"] <= f]
        if not earlier:
            return f"chapter_start:{chapter['chapter']}", f - chapter["start_frame"]
        p = earlier[-1]
        end = p["start_frame"] + round(p["duration_s"] * V.FPS)
        if f >= end:
            return f"voice_end:{p['cue']}", f - end
        at = [p["start_frame"] + round(t * V.FPS) for t in data["onsets"][p["cue"]]]
        k = max(i for i, t in enumerate(at, 1) if t <= f)
        return f"sentence:{p['cue']}:{k}", f - at[k - 1]

    rows = []

    def row(finding, base, offsets, note, was=None):
        n = 1 + sum(1 for r in rows if r["finding"] == finding)
        rows.append({"id": f"{finding}.{n}", "finding": finding, "base": base, "offsets": offsets, "note": note, "was": was})

    for finding, note, places in FLAGGED:
        grouped = {}
        for where in places:
            for f in frames_of(where, pictures):
                base, offset = rebase(f)
                grouped.setdefault(base, {})[offset] = f
        for base, items in grouped.items():
            offsets = sorted(items)
            row(finding, base, offsets, note, [items[o] for o in offsets])
    for x, nxt in zip(chapters, chapters[1:]):
        name = f"join-{x['chapter'][:2]}-{nxt['chapter'][:2]}"
        row(name, f"chapter_end:{x['chapter']}", [-1], "last frame before the join", [nxt["start_frame"] - 1])
        row(name, f"chapter_start:{nxt['chapter']}", [0], "first frame after the join", [nxt["start_frame"]])
    for s in sites.values():
        cue = s["cue"]
        if s["kind"] == "prediction":
            name = f"pause-{cue}"
            row(name, f"hold:{cue}:start", [0, 30, 120], "pause line on, no answer")
            row(name, f"hold:{cue}:think_end", [-1, 7], "pause line, last full, then fading")
            row(name, f"hold:{cue}:next", [0], "next cue, line gone, no answer")
        else:
            name = f"reading-{cue}"
            row(name, f"hold:{cue}:start", [0, 120], "reading hold")
            row(name, f"hold:{cue}:think_end", [-1], "reading hold, last frame")
    for finding, base, offsets, note in REPAIRS:
        row(finding, base, offsets, note)
    doc = {"schema": "logarithm-final-picture-anchors-v1",
           "about": "Each row is a base resolved on the new schedule, plus frame offsets. `was` is the stage09 film frame for each offset, "
                    "kept for people and the positive control. The helper never reads it. Sentence numbers are 1-based.",
           "stage09_verification_sha256": D.VERIFICATION_SHA256, "review_seals": D.REVIEW_SEALS, "rows": rows}
    with open(sys.argv[1], "x") as handle:
        handle.write(json.dumps(doc, indent=1) + "\n")
    print(f"{len(rows)} rows, {sum(len(r['offsets']) for r in rows)} anchors")


if __name__ == "__main__":
    main()
