#!/usr/bin/env python3
"""Bind the apportionment Oliver plan to the staged combined-repair-stage03 source.

    python3 -I -B bind_plan.py STAGED_TOPIC CANONICAL_TOPIC PREVIEW_RENDER OUT [--check]

Run from the repo root with repo-relative paths. Writes under OUT, mirroring the
canonical layout so the contract's map path holds after integration:

  explainer_corpus/apportionment/docs/narration-direction-map-oliver04.json
  explainer_corpus/apportionment/narration_units.json
  source-audit.json

A text audit of Rust source: nothing is compiled, rendered, synthesized or heard.
Only chapters registered in mod.rs are read. A changed wait site that is not in
SITES below stops the run, so a classification can never go stale silently.
An existing output that differs is refused, never overwritten.
"""
import hashlib
import json
import re
import sys
from pathlib import Path

TOPIC = "apportionment"
REL = f"explainer_corpus/{TOPIC}"
MAP_NAME = "narration-direction-map-oliver04.json"
PREV_MAP = "narration-direction-map-oliver03.json"
UNCLASSIFIED = re.compile(r"wait_frames|stage\.wait|kit\.listen|music_breath|kit\.mark|until_sentence")
SAY = re.compile(r'\bsay\(&kit, stage, "(\d\d_\d+)"\)')
ANY_SAY = re.compile(r"\bsay\(&kit")
THINK = re.compile(r"\bthink(?:_here)?\(&kit, stage")
WIPE = re.compile(r"\bwipe\(&kit, stage\)")
HOLD = re.compile(r"\bkit\.hold\(stage\)")
PAUSE = re.compile(r"\bpause\(stage, ([0-9.]+)\)")
WAIT_UNTIL = re.compile(r"\bwait_until\(stage,")
FINISH = re.compile(r"\bkit\.finish\(stage\)")
WAIT_FOR_LITERAL = re.compile(r'\.wait_for\(stage, "')
WAIT_FOR_OTHER = re.compile(r'\.wait_for\(stage, [^"]')

MAP_SCOPE = (
    "Candidate rebuild (bead .30), combined-repair-stage03: 15 authored prediction windows and no "
    "authored reading window, bound to staged source. A row exists only where the source adds a "
    "prediction or reading wait; see source-audit.json for every other wait and why it has no row. "
    "The both_promises.rs and sixteenth.rs hashes are the proposed staged bytes, not the installed "
    "canonical files. No take, listening, timing or independent acceptance."
)
UNITS_SCOPE = (
    "Candidate full-cue source plan for ElevenLabs Oliver, combined-repair-stage03: one unit per cue, "
    "at most 100 words. It says nothing about whether a recording of any cue exists; unchanged "
    "wording is not evidence of a usable take. No synthesis, take selection, listening, timing or "
    "independent acceptance."
)
MEANINGS = {
    "prediction": "draw::think after a question cue: kit.hold (voice end plus the 0.6 s breath), "
    "mark predict, fade 0.2, THINK_S 4.0, fade 0.2, so 5.0 s in source after the voice. Not silence in any take.",
    "wipe": "draw::wipe: kit.hold, then a 0.6 s fade of the whole stage, so 1.2 s in source after the "
    "voice. A scene change, not a reading or prediction hold.",
    "hold_plus_pause": "kit.hold then pause 0.6: 1.2 s in source after the voice, no fade.",
    "hold_then_say": "kit.hold directly before say: the breath say would keep anyway; no added time.",
    "say_breath": "say waits for the previous line, then the 0.6 s breath (LineGaps::Constant).",
    "chapter_end": "The chapter's last cue: closing pause, then kit.finish. No next cue in this chapter.",
}
# Every wait that is not a think, a wipe or a chapter's closing pause: (file, line) -> class, note.
SITES = {
    ("one_more_seat.rs", 117): (
        "picture_beat",
        "Beat after Ash's 2 → 1 note, timed from picture time, before the wipe. Whether it ends before or after the voice is not fixed by source.",
    ),
    ("shares.rs", 153): (
        "transition_after_voice",
        "With line 154: the source's own comment says the hold and pause keep the wipe's timing. A real wait after the voice, but authored as a scene beat, not for reading. Same 1.2 s as the 20 wipes, which have no row either.",
    ),
    ("shares.rs", 154): (
        "transition_after_voice",
        "The added 0.6 s of the pair at lines 153 and 154.",
    ),
    ("shares.rs", 165): (
        "word_sync_inside_voice",
        "Clears the number line 1.1 s before \"Ash's share\" is spoken: inside 02_2.",
    ),
    ("not_every_time.rs", 82): (
        "sequencing_no_added_time",
        "Directly before say 06_3. The comment asks that the result stays up during the introduction; no time is added.",
    ),
    ("not_every_time.rs", 131): (
        "sequencing_no_added_time",
        "Directly before say 06_4. The comment asks that the loss strip stays up; no time is added.",
    ),
    ("not_every_time.rs", 195): (
        "reading_floor_inside_cue",
        "The film's only authored reading time: the quote and its source stay at full strength for READ_CPS after they appear. It delays a picture change inside 06_6, the chapter's last cue, not a cue boundary. It adds time after the voice only if 06_6 ends before the floor, so a boundary row would misdescribe it.",
    ),
    ("both_promises.rs", 544): (
        "picture_beat",
        "Beat on \"16 seats: 1, 4, 11\" before the wipe into 12_4, timed from picture time. Whether it ends before or after the voice is not fixed by source.",
    ),
    ("both_promises.rs", 799): (
        "conditional_walk_beat",
        "Inside the 12_7 walk's wait branch, reached at 6 and at 15 (storyboard: each holds 1.2 s). Part of the picture's walk; it can only push 12_8 back by overrunning the voice.",
    ),
}
# Pairs whose gap on the scratch voice differs from what their boundary kind alone gives.
OVERRUNS = {
    ("01_2", "01_3"): "After \"Cedar gets ten\" Cedar's ten tokens take 2.8 s (0.25 × 10 + 0.3), which outlasts the voice.",
    ("12_2", "12_3"): "The quota walk keeps running after the cue's last phrase.",
    ("12_7", "12_8"): "The walk to 16, with its two 1.2 s waits, keeps running after the cue's last phrase.",
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_sha(path):
    return sha(path.read_bytes())


def read_script(path):
    rows = []
    for line in path.read_text().splitlines():
        if line.startswith("#") or not line.strip():
            continue
        cue, text = line.split("\t", 1)
        rows.append((cue, text))
    return rows


def registered(chapters):
    """Chapter file names in film order, from the registry in mod.rs."""
    src = (chapters / "mod.rs").read_text()
    pairs = re.findall(r'key: "(\d\d_\w+)",\s*make: \|kit\| Box::new\((\w+)::', src)
    declared = set(re.findall(r"^(?:pub )?mod (\w+);", src, re.M))
    for _, module in pairs:
        if module not in declared:
            raise SystemExit(f"{module} is in the registry but not declared")
    return [(key, f"{module}.rs") for key, module in pairs], sorted(declared)


def census(chapters, names, cues):
    """Every say and wait in the registered chapters, by file and line."""
    says, thinks, wipes, holds, pauses, untils, finishes = [], [], [], [], [], [], []
    wait_for, wait_for_other = {}, []
    for name in names:
        current = None
        lines = (chapters / name).read_text().splitlines()
        for n, line in enumerate(lines, 1):
            where = {"file": name, "line": n, "marker": line.strip()}
            if UNCLASSIFIED.search(line):
                raise SystemExit(f"{name}:{n}: a timing call this audit does not classify: {line.strip()}")
            m = SAY.search(line)
            if m:
                current = m.group(1)
                says.append({**where, "cue": current})
            elif ANY_SAY.search(line):
                raise SystemExit(f"{name}:{n}: a say this audit cannot read: {line.strip()}")
            if THINK.search(line):
                thinks.append({**where, "cue": current})
            if WIPE.search(line):
                wipes.append({**where, "cue": current})
            if HOLD.search(line):
                holds.append({**where, "cue": current})
            p = PAUSE.search(line)
            if p:
                closing = n < len(lines) and FINISH.search(lines[n])
                pauses.append({**where, "cue": current, "seconds": float(p.group(1)), "closing": bool(closing)})
            elif re.search(r"\bpause\(stage", line):
                raise SystemExit(f"{name}:{n}: a pause with no literal length: {line.strip()}")
            if WAIT_UNTIL.search(line):
                untils.append({**where, "cue": current})
            if FINISH.search(line):
                finishes.append({**where, "cue": current})
            if WAIT_FOR_LITERAL.search(line):
                wait_for[current] = wait_for.get(current, 0) + 1
            if WAIT_FOR_OTHER.search(line):
                wait_for_other.append({**where, "cue": current})
    for group in (thinks, wipes, holds, pauses, untils):
        for row in group:
            if row["cue"] is None:
                raise SystemExit(f"{row['file']}:{row['line']}: a wait before any cue")
    if [s["cue"] for s in says] != cues:
        raise SystemExit("registered chapters do not say every script cue once, in order")
    return says, thinks, wipes, holds, pauses, untils, finishes, wait_for, wait_for_other


def boundary_kinds(cues, thinks, wipes, holds, pauses):
    """What the source does between each cue's voice end and the next cue."""
    def at(group):
        return {row["cue"] for row in group}

    think, wipe, hold = at(thinks), at(wipes), at(holds)
    held_pause = {p["cue"] for p in pauses if (p["file"], p["line"]) == ("shares.rs", 154)}
    kinds = {}
    for i, cue in enumerate(cues):
        nxt = cues[i + 1] if i + 1 < len(cues) else None
        if nxt is None or nxt[:2] != cue[:2]:
            kinds[cue] = "chapter_end"
        elif cue in think:
            kinds[cue] = "prediction"
        elif cue in wipe:
            kinds[cue] = "wipe"
        elif cue in held_pause:
            kinds[cue] = "hold_plus_pause"
        elif cue in hold:
            kinds[cue] = "hold_then_say"
        else:
            kinds[cue] = "say_breath"
    return kinds


def scratch_gaps(render, chapter_keys, cues, kinds):
    """Compare every in-chapter gap on the preview02 scratch voice with its boundary kind."""
    rows, by_kind, outliers = [], {}, []
    limits = {
        "prediction": (5.0, 5.2),
        "wipe": (1.15, 1.3),
        "hold_plus_pause": (1.15, 1.3),
        "hold_then_say": (0.55, 0.7),
        "say_breath": (0.55, 0.7),
    }
    timing = {}
    for key in chapter_keys:
        for line in (render / f"{key}.cues.tsv").read_text().splitlines():
            parts = line.split("\t")
            try:
                timing[parts[0]] = (float(parts[1]), float(parts[2]))
            except (ValueError, IndexError):
                continue
    for i in range(len(cues) - 1):
        a, b = cues[i], cues[i + 1]
        if a[:2] != b[:2]:
            continue
        gap = round(timing[b][0] - timing[a][0] - timing[a][1], 3)
        kind = kinds[a]
        by_kind.setdefault(kind, []).append(gap)
        lo, hi = limits[kind]
        if not lo <= gap <= hi:
            outliers.append({"cue": a, "next_cue": b, "kind": kind, "scratch_gap_s": gap,
                             "from_source": OVERRUNS.get((a, b))})
    for row in outliers:
        if row["from_source"] is None:
            raise SystemExit(f"unexplained scratch gap: {row}")
    if {(r["cue"], r["next_cue"]) for r in outliers} != set(OVERRUNS):
        raise SystemExit("the scratch overruns are not the ones this audit explains")
    summary = {k: {"pairs": len(v), "min_s": min(v), "max_s": max(v)} for k, v in sorted(by_kind.items())}
    return {
        "claim": "Scratch voice on the older preview02 render, which predates the staged chapters 04 and 12. "
        "Used only to test this classification, not as timing for Oliver.",
        "by_kind": summary,
        "picture_overruns": outliers,
        "note": "Overruns are not authored pauses and have no row. Their size depends on the voice, so "
        "they must be measured on real takes.",
    }


def build(staged, canonical, render):
    chapters = staged / "src" / "chapters"
    script_path = staged / "script.tsv"
    if "LineGaps::Constant" not in (staged / "src" / "main.rs").read_text():
        raise SystemExit("main.rs no longer keeps the constant breath this audit describes")
    if "pub const BREATH: f64 = 0.6;" not in Path("crates/explainer_kit/src/narration.rs").read_text():
        raise SystemExit("the kit's breath is no longer the 0.6 s this audit describes")
    if script_path.read_bytes() != (canonical / "script.tsv").read_bytes():
        raise SystemExit("staged script.tsv differs from canonical")
    script = read_script(script_path)
    cues = [c for c, _ in script]
    texts = dict(script)
    script_sha = file_sha(script_path)
    reg, declared = registered(chapters)
    names = [n for _, n in reg]
    says, thinks, wipes, holds, pauses, untils, finishes, wait_for, wait_for_other = census(chapters, names, cues)
    if len({t["cue"] for t in thinks}) != len(thinks):
        raise SystemExit("two think pauses after one cue")

    previous = json.loads((staged / "docs" / PREV_MAP).read_text())
    if previous != json.loads((canonical / "docs" / PREV_MAP).read_text()):
        raise SystemExit("staged oliver03 differs from canonical")
    prev_over = {r["cue"]: r for r in previous["overrides"]}
    prev_hold = {r["cue"]: r for r in previous["visual_holds"]}
    if {t["cue"] for t in thinks} != set(prev_over) or set(prev_over) != set(prev_hold):
        raise SystemExit("the staged think pauses are not oliver03's authored directions")

    overrides, visual = [], []
    for t in sorted(thinks, key=lambda t: cues.index(t["cue"])):
        cue = t["cue"]
        nxt = cues[cues.index(cue) + 1]
        if not texts[cue].endswith("?") or prev_over[cue]["after"] != texts[cue]:
            raise SystemExit(f"{cue}: oliver03's direction no longer fits the script")
        overrides.append(dict(prev_over[cue]))
        row = dict(prev_hold[cue])
        if row["next_cue"] != nxt or row["marker"] != t["marker"]:
            raise SystemExit(f"{cue}: oliver03's hold row does not fit the staged source")
        row["sha256"] = file_sha(chapters / t["file"])
        row["line"] = t["line"]
        visual.append(row)

    draw = chapters / "draw.rs"
    draw_lines = draw.read_text().splitlines()
    helpers = []
    for needle in ("pub const THINK_S: f64 = 4.0;", "pub fn think(kit: &Kit,", "pub fn think_here(kit: &Kit,"):
        hit = [i for i, line in enumerate(draw_lines, 1) if line.startswith(needle)]
        if len(hit) != 1:
            raise SystemExit(f"draw.rs: {needle!r} is not on exactly one line")
        helpers.append({"path": f"{REL}/src/chapters/draw.rs", "sha256": file_sha(draw), "line": hit[0],
                        "marker": draw_lines[hit[0] - 1].strip()})
    direction_map = {
        "schema_version": 1,
        "scripts": {TOPIC: script_sha},
        "overrides": overrides,
        "visual_holds": visual,
        "hold_kinds": {"prediction": len(visual), "reading": 0},
        "helpers": helpers,
        "scope": MAP_SCOPE,
    }
    map_text = json.dumps(direction_map, indent=2, ensure_ascii=False) + "\n"
    units = {
        "schema_version": 1,
        "script_sha256": script_sha,
        "direction_contract": {
            "schema_version": 1,
            "topic": TOPIC,
            "script_sha256": script_sha,
            "map": {"path": f"{REL}/docs/{MAP_NAME}", "sha256": sha(map_text.encode())},
        },
        "scope": UNITS_SCOPE,
        "units": [{"id": f"oliver_{c}", "line_ids": [c], "text": t} for c, t in script],
    }
    for u in units["units"]:
        if not 1 <= len(u["text"].split()) <= 100:
            raise SystemExit(f"{u['id']}: outside the coherent-unit word bound")
    units_text = json.dumps(units, indent=2, ensure_ascii=False) + "\n"

    # every wait that is not a think, a wipe or a closing pause must be classified above
    others = []
    for row in holds + untils + [p for p in pauses if not p["closing"]]:
        key = (row["file"], row["line"])
        if key not in SITES:
            raise SystemExit(f"{key}: a wait this audit has not classified: {row['marker']}")
    seen = {(r["file"], r["line"]) for r in holds + untils + pauses}
    for key, (kind, note) in SITES.items():
        if key not in seen:
            raise SystemExit(f"{key}: a classified wait that is not in the source")
        row = next(r for r in holds + untils + pauses if (r["file"], r["line"]) == key)
        others.append({"file": key[0], "line": key[1], "marker": row["marker"], "cue": row["cue"],
                       "seconds_in_source": row.get("seconds"), "class": kind, "row_in_map": False, "note": note})
    others.sort(key=lambda r: (cues.index(r["cue"]), r["line"]))

    not_every = (chapters / "not_every_time.rs").read_text()
    parts = re.findall(r'^    "((?:[^"\\]|\\.)*)",$', not_every.split("const QUOTE", 1)[1].split("];", 1)[0], re.M)
    source_line = re.search(r'const SOURCE: &str = "([^"]*)";', not_every).group(1)
    floor_chars = sum(len(p.replace('\\"', '"')) for p in parts) + len(source_line)
    cps = float(re.search(r"const READ_CPS: f64 = ([0-9.]+);", not_every).group(1))
    floor = {"chars": floor_chars, "chars_per_second": cps, "seconds_from_quote_shown": round(floor_chars / cps, 2)}
    for row in others:
        if row["class"] == "reading_floor_inside_cue":
            row["floor"] = floor

    kinds = boundary_kinds(cues, thinks, wipes, holds, pauses)
    order = {c: i for i, c in enumerate(cues)}
    say_at = {s["cue"]: s for s in says}
    cue_rows = []
    for i, (cue, text) in enumerate(script):
        nxt = cues[i + 1] if i + 1 < len(cues) else None
        cue_rows.append({
            "cue": cue,
            "unit": f"oliver_{cue}",
            "next_cue": nxt,
            "next_cue_crosses_chapter": nxt is not None and nxt[:2] != cue[:2],
            "words": len(text.split()),
            "file": say_at[cue]["file"],
            "line": say_at[cue]["line"],
            "boundary": kinds[cue],
            "literal_wait_for_calls": wait_for.get(cue, 0),
        })
    count = lambda k: sum(1 for v in kinds.values() if v == k)

    staged_names = sorted(p.name for p in chapters.iterdir())
    used = set(names) | {"draw.rs", "thumbnail.rs", "mod.rs"}
    ignored = [{"path": f"{REL}/src/chapters/{n}", "sha256": file_sha(chapters / n),
                "in_canonical": (canonical / "src" / "chapters" / n).exists()}
               for n in staged_names if n not in used]

    table = []
    for rel in [f"src/chapters/{n}" for n in names] + ["src/chapters/draw.rs", "script.tsv"]:
        proposed, installed = file_sha(staged / rel), file_sha(canonical / rel)
        table.append({"path": f"{REL}/{rel}", "proposed_sha256": proposed, "installed_canonical_sha256": installed,
                      "status": "same" if proposed == installed else "PROPOSED: not installed"})

    audit = {
        "schema_version": 1,
        "topic": TOPIC,
        "claim": "Text audit of Rust source. Uncompiled, unrendered. No take, duration, listening or acceptance.",
        "script_sha256": script_sha,
        "map_version": "oliver04",
        "kit": {"line_gaps": "Constant", "breath_s": 0.6, "think_s": 4.0},
        "meanings": MEANINGS,
        "counts": {
            "cues": len(cues),
            "units": len(cues),
            "chapters": len(names),
            "words_total": sum(len(t.split()) for _, t in script),
            "words_max": max(len(t.split()) for _, t in script),
            "cues_over_90_words": [c for c, t in script if len(t.split()) > 90],
            "prediction_holds": len(visual),
            "reading_holds": 0,
            "boundaries": {k: count(k) for k in MEANINGS},
            "think_calls": len(thinks),
            "wipe_calls": len(wipes),
            "kit_hold_calls": len(holds),
            "pause_calls": len(pauses),
            "closing_pauses": sum(1 for p in pauses if p["closing"]),
            "wait_until_calls": len(untils),
            "literal_wait_for_calls": sum(wait_for.values()),
            "non_literal_wait_for_calls": len(wait_for_other),
        },
        "prediction_holds": [
            {"cue": r["cue"], "next_cue": r["next_cue"], "file": r["path"].rsplit("/", 1)[1], "line": r["line"],
             "marker": r["marker"], "seconds_in_source_after_voice": 5.0,
             "changed_from_oliver03": sorted(k for k in r if r[k] != prev_hold[r["cue"]].get(k))}
            for r in visual
        ],
        "reading_holds": [],
        "reading_holds_note": "Judgement: no row. No site adds a wait after the voice in order to read a card. "
        "The nearest candidates are the classes transition_after_voice, reading_floor_inside_cue and picture_beat "
        "under other_waits, left for root to overrule.",
        "other_waits": others,
        "closing_pauses": [{"file": p["file"], "line": p["line"], "cue": p["cue"], "seconds_in_source": p["seconds"]}
                           for p in pauses if p["closing"]],
        "wipes": [{"file": w["file"], "line": w["line"], "cue": w["cue"],
                   "next_cue": cues[order[w["cue"]] + 1]} for w in wipes],
        "non_literal_wait_for": wait_for_other,
        "scratch_gap_crosscheck": scratch_gaps(render, [k for k, _ in reg], cues, kinds),
        "registered_chapters": [{"key": k, "file": n} for k, n in reg],
        "unregistered_files_ignored": ignored,
        "not_in_the_film": [{"file": "thumbnail.rs", "line": 75, "note": "stage.wait(1.0) renders the thumbnail, not a chapter."}],
        "proposed_versus_installed": table,
        "cues": cue_rows,
    }
    if sorted(n[:-3] for n in names) != sorted(d for d in declared if d not in ("draw", "thumbnail")):
        raise SystemExit("mod.rs declares a chapter module the registry does not use")
    audit_text = json.dumps(audit, indent=2, ensure_ascii=False) + "\n"
    return map_text, units_text, audit_text, audit


def main(argv):
    check = "--check" in argv
    args = [a for a in argv if a != "--check"]
    staged, canonical, render, out = (Path(a) for a in args)
    map_text, units_text, audit_text, audit = build(staged, canonical, render)
    outputs = {
        out / REL / "docs" / MAP_NAME: map_text,
        out / REL / "narration_units.json": units_text,
        out / "source-audit.json": audit_text,
    }
    bad = 0
    for path, text in outputs.items():
        same = path.is_file() and path.read_text() == text
        if not same and path.exists():
            print(f"DIFFERS, not overwritten: {path}")
            bad += 1
        elif not same and check:
            print(f"MISSING: {path}")
            bad += 1
        elif not same:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        print(f"{sha(text.encode())}  {path.name}")
    c = audit["counts"]
    print(f"{c['cues']} cues, {c['units']} units, {c['words_total']} words, max {c['words_max']}; "
          f"{c['prediction_holds']} prediction holds, {c['reading_holds']} reading holds")
    print("boundaries:", c["boundaries"])
    print("scratch gaps:", json.dumps(audit["scratch_gap_crosscheck"]["by_kind"]))
    for r in audit["scratch_gap_crosscheck"]["picture_overruns"]:
        print(f"overrun on scratch: {r['cue']} -> {r['next_cue']} {r['scratch_gap_s']} s ({r['kind']})")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
