#!/usr/bin/env python3
"""Source checks for the logarithm chapter 11 and 12 picture repair.

Read-only, standard library, builds and renders nothing. Run as:

    python3 -I -B checks/check_repair.py [--ws WORKSPACE] [--verification FILE]

--ws adds the comparison with the rendered workspace, its hold map and its facts.
--verification binds the chapter frames to the final verification record.
Without them the recorded hashes and frames are used. Exit 0 only when every check
and every negative control passes. This is not a rebinder: it writes nothing and
only shows that the held lines did not move.
"""

import argparse
import difflib
import hashlib
import json
import math
import re
import sys
from pathlib import Path

BUNDLE = Path(__file__).resolve().parents[1]
CHAPTERS = "explainer_corpus/logarithm_mysteries/src/chapters"
FILES = {
    "unreachable.rs": "c822e8b46796e79e2f8461872fe538de711a4fde755913ec57696710d0cc28a7",
    "what_it_forgets.rs": "6951d3b2f157387683efaa3ec7c9d9d12a23c8448df247531cd5f0081bab1e28",
}
HOLDS = [
    ("unreachable.rs", "11_3", 1070, "predict(&kit, stage, v(right + 1.6, -1.4), 3.0)?;",
     "e34e65ad69a71865c956f2cf6e3d5270433370e8b6e79e5f8defc221555771db"),
    ("unreachable.rs", "11_20", 1590, "kit.music_breath(stage, rule_read + row_reads.iter().sum::<f64>())?;",
     "4a4e6c72d1f92e5e88e7430c03e0f0f062e3e7972184736bfc911a4313fae5e0"),
    ("what_it_forgets.rs", "12_5", 1123, "predict(&kit, stage, v(2.6, -2.1), 3.4)?;",
     "b238a92968284627ce972479bb1c5a93ac5f5df33efee0b17ddea5d362c8387f"),
]
LIOUVILLE_OLD = "for any real linear flow, steady or not, det = exp(∫ trace dt) > 0 (Liouville)"
LIOUVILLE_NEW = (
    "for any real linear flow, steady or not, det (signed area) = exp(∫ trace dt) > 0, "
    "trace = diagonal sum (Liouville)"
)
# baseline line: the new text of a change that touches a clock call, pinned whole
CLOCK_CHANGES = {
    807: ("fade_in_all(stage, &[avg_a], 0.5)?;",
          "let det_gloss = fade_in_with_gloss(&kit, stage, avg_a, v(right, -0.35))?;"),
    1197: ("swap(stage, &[probe_alone], &[first_routes, route_a, route_b], 0.4)?;",
           "let sample = swap_in_sample(&kit, stage, base, probe_alone, [first_routes, route_a, route_b])?;"),
    1244: ("play!(stage; tween(off, 0.0, 2.6)?);", "tween_leaving(stage, off, 0.0, &sample, 2.6)?;"),
}
OTHER_CHANGES = [(57, 57), (800, 800), (832, 832), (839, 839), (958, 958), (1620, 1627), (1662, 1662)]
CLOCK = re.compile(
    r"\b(fade_in_all|fade_out_all|fade_in_lagged|fade_all|swap|reword|play!|tween_all|tween_leaving|play_prepared"
    r"|wait_frames|until_sentence|until_phrase|music_breath|listen|predict|say)\b"
)
HELPER_CLOCK = [
    "fade_in_all(stage, &[line, gloss], 0.5)?;",
    "swap(stage, &[out], &inn, 0.4)?;",
    "stage.play_prepared(anims)?;",
]


def sha(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def em(c):
    """The em table of the crate's own width test."""
    table = (
        ("il,.:;![]", 0.278), ("jf", 0.306), (" -", 0.333), ("I", 0.361), ("rst()", 0.39), ("acez", 0.444),
        ("go^/0123456789", 0.5), ("kqvxy", 0.528), ("bdhnpu", 0.556), ("w", 0.722), ("=+−×±<>", 0.778), ("m", 0.833),
    )
    for chars, value in table:
        if c in chars:
            return value
    return 0.75


def width(s, size):
    return sum(em(c) for c in s) * size * 0.01


def apply_patch(patch, sources):
    """Apply a unified diff to {name: text}, every context and removed line checked."""
    out, lines, i = {}, patch.split("\n"), 0
    while i < len(lines):
        if not lines[i].startswith("--- "):
            i += 1
            continue
        name = lines[i + 1][4:].split("\t")[0].rsplit("/", 1)[1]
        src, res, pos = sources[name].split("\n"), [], 0
        i += 2
        while i < len(lines) and lines[i].startswith("@@"):
            m = re.match(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@", lines[i])
            old_n, new_n = int(m[2] or 1), int(m[4] or 1)
            res.extend(src[pos:int(m[1]) - 1])
            pos = int(m[1]) - 1
            i += 1
            while old_n or new_n:
                tag, body = lines[i][:1], lines[i][1:]
                if tag in " -":
                    if src[pos] != body:
                        raise ValueError(f"{name}: line {pos + 1} does not match the patch")
                    pos += 1
                    old_n -= 1
                if tag in " +":
                    res.append(body)
                    new_n -= 1
                i += 1
        res.extend(src[pos:])
        out[name] = "\n".join(res)
    return out


def say_line(lines, cue):
    needle = f'kit.say(stage, "{cue}")?;'
    hits = [n for n, line in enumerate(lines, 1) if needle in line]
    return hits[0] if len(hits) == 1 else None


def source_checks(before, cand):
    """Every check on the two candidates, as (name, ok, detail). Takes text, so controls can mutate it."""
    rows = []
    b11, c11 = before["unreachable.rs"].split("\n"), cand["unreachable.rs"].split("\n")
    b12, c12 = before["what_it_forgets.rs"].split("\n"), cand["what_it_forgets.rs"].split("\n")
    film11, tests11 = cand["unreachable.rs"].split("#[cfg(test)]", 1)
    film12, _ = cand["what_it_forgets.rs"].split("#[cfg(test)]", 1)
    film = {"unreachable.rs": film11, "what_it_forgets.rs": film12}

    # the held intervals, by the reading the stored hashes were made with
    for name, cue, line, marker, stored in HOLDS:
        lines = cand[name].split("\n")
        base = before[name].split("\n")
        start = say_line(base, cue)
        control = sha("\n".join(base[start - 1:line])) == stored
        same = say_line(lines, cue) == start and sha("\n".join(lines[start - 1:line])) == stored
        rows.append((f"hold {cue}", control and same and marker in lines[line - 1],
                     f"{name} say line {start}, hold line {line}; baseline reproduces the stored hash {control}; "
                     f"candidate unchanged {same}"))

    # cue order
    for name in FILES:
        ids = re.compile(r'"(1[12]_\d+)"')
        old = ids.findall(before[name].split("#[cfg(test)]")[0])
        rows.append((f"cue order {name}", old == ids.findall(film[name]), f"{len(old)} cue references"))

    # chapter 11: only the listed film lines changed, and none of them changes a clock
    start = next(i for i, s in enumerate(c11) if s.startswith("/// 11_5's sample of X"))
    end = next(i for i, s in enumerate(c11) if s == "#[cfg(test)]")
    a, b = b11[:b11.index("#[cfg(test)]")], c11[:start]
    changed = {}
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(a=a, b=b, autojunk=False).get_opcodes():
        if tag != "equal":
            changed[(i1 + 1, i2)] = (a[i1:i2], b[j1:j2])
    want = sorted(OTHER_CHANGES + [(n, n) for n in CLOCK_CHANGES])
    rows.append(("chapter 11 changed lines", sorted(changed) == want, f"baseline ranges {sorted(changed)}"))
    pinned = all(
        changed.get((n, n)) and old in changed[(n, n)][0][0] and new in changed[(n, n)][1][0]
        for n, (old, new) in CLOCK_CHANGES.items()
    )
    rows.append(("chapter 11 clock lines pinned", pinned, "fade 0.5, swap 0.4, tween 2.6 keep their run times"))
    quiet = all(
        not CLOCK.search(" ".join(old + new)) for r, (old, new) in changed.items() if r[0] not in CLOCK_CHANGES
    )
    rows.append(("other changed lines hold no clock call", quiet, "text, place, list and import only"))
    helpers = "\n".join(c11[start:end])
    calls = [s.strip() for s in helpers.split("\n") if CLOCK.search(s) and not s.lstrip().startswith(("///", "fn "))]
    calls = [s for s in calls if not s.startswith("let sample = ")]
    rows.append(("helpers play only the listed calls", calls == HELPER_CLOCK, f"{calls}"))
    leaving = helpers.split("fn tween_leaving", 1)[1]
    rows.append(("tween and fades share one run time",
                 "tween(tracker, to, rt)" in leaving and "a.rt(rt)" in leaving and leaving.count("play_prepared") == 1,
                 "tween_leaving plays both in one play_prepared"))

    # chapter 12: the import, then the closing lines, nothing before them
    fade_at = c12.index("        fade_in_all(stage, &family, 0.4)?;")
    tail = c12[fade_at + 1:fade_at + 5]
    body_same = [n + 1 for n in range(1308) if b12[n] != c12[n]] == [44]
    closing = body_same and fade_at == 1307 and tail[2:] == [
        "        kit.music_breath(stage, CLOSING_HOLD)?;", "        fade_all(stage, CLOSING_FADE)?;",
    ] and c12[fade_at + 5] == "        kit.finish(stage)"
    rows.append(("chapter 12 changed lines", closing, "line 44 import; lines 1309 to 1312 added; lines 1 to 1308 otherwise same"))
    order = ['kit.say(stage, "12_21")?;', "fade_in_all(stage, &family, 0.4)?;", "kit.music_breath(stage, CLOSING_HOLD)?;",
             "fade_all(stage, CLOSING_FADE)?;", "kit.finish(stage)"]
    at = [film12.find(s) for s in order]
    rows.append(("closing order", all(film12.count(s) == 1 for s in order) and at == sorted(at)
                 and "kit.say(" not in film12[at[0] + len(order[0]):], "12_21, card, hold, fade, finish"))

    # the same strings the Rust tests assert
    rows.append(("F2 no log A", r"\log A" not in film11 and r"\text{{a complex logarithm: }} X = " in film11
                 and "e^{{X}} = A" in film11, "film names one complex logarithm, e^X = A"))
    rows.append(("F2 formula has one source", film11.count(r"\text{{a complex logarithm") == 1
                 and "kit.tex(stage, &complex_log_tex(&big), 32.0, blue, &[], v(0.0, 1.3))?;" in film11,
                 "the native test builds the film's helper at the film's size and place"))
    rows.append(("F1 glosses", film11.count(LIOUVILLE_NEW) == 1 and tests11.count(LIOUVILLE_NEW) == 2
                 and LIOUVILLE_OLD not in cand["unreachable.rs"] and r"\operatorname{tr}" not in cand["unreachable.rs"]
                 and r"\operatorname{trace}" in film11 and "has det {}:" not in film11 and "has det 0:" not in tests11,
                 "Liouville footer in film and twice in tests; tr spelled out; colon gone"))
    rows.append(("M1 sample bound to its fact",
                 'let base = fact_matrix("matrix.probe_candidate")?;' in film11
                 and "swap_in_sample(&kit, stage, base, probe_alone," in film11
                 and film11.count("(here X is the sample flow from before)") == 1
                 and film11.find("tween_leaving(stage, off, 0.0, &sample, 2.6)?;")
                 < film11.find("the ends meet only when b = c = 0"), "no number typed; 11_4's sample words kept"))
    return rows


def width_rows():
    """Estimates only, from the crate's em table. The native tests in the patch are the real measure."""
    gloss = "(det = determinant, the signed area factor)"
    word = "here X is a new sample:"
    widest = width("−0.5", 24)
    half = max(24 * 0.011, (widest + 24 * 0.25 / 30) / 2) + max(24 * 0.0095, widest / 2 + 24 * 0.1 / 30)
    card_x = -1.15 + 1.35
    return [
        ("gloss right edge estimate", 2.2 + width(gloss, 20) * 1.03 < 6.86, f"{2.2 + width(gloss, 20) * 1.03:.2f} < 6.86"),
        ("footer width estimate", width("precisely: " + LIOUVILLE_NEW, 22) < 13.6,
         f"{width('precisely: ' + LIOUVILLE_NEW, 22):.2f} < 13.6"),
        ("sample word estimate", -1.15 + width(word, 18) * 1.03 < 2.05, f"right {-1.15 + width(word, 18) * 1.03:.2f} < 2.05"),
        ("sample card estimate", card_x - half > -1.15 + width("X =", 24) * 1.03 and card_x + half < 2.05,
         f"card {card_x - half:.2f} to {card_x + half:.2f}; name ends {-1.15 + width('X =', 24) * 1.03:.2f}"),
    ]


def timing_rows(cand, verification):
    text = cand["what_it_forgets.rs"]
    hold = float(re.search(r"const CLOSING_HOLD: f64 = ([\d.]+);", text)[1])
    fade = float(re.search(r"const CLOSING_FADE: f64 = ([\d.]+);", text)[1])
    ch11, ch12, film = 16550, 9837, 116245
    bound = "recorded"
    if verification:
        data = json.loads(Path(verification).read_text())
        rows = data["chapters"].items() if isinstance(data["chapters"], dict) else [
            (r.get("chapter") or r.get("name") or r.get("id"), r) for r in data["chapters"]
        ]
        frames = {str(k): v["frames"] for k, v in rows}
        ch11 = next(v for k, v in frames.items() if k.startswith("11"))
        ch12 = next(v for k, v in frames.items() if k.startswith("12"))
        film = sum(frames.values())
        bound = "verification record"
    hold_f, fade_f = math.ceil(hold * 60), round(fade * 60)
    print(f"TIMING computed, not measured; baseline frames from the {bound}")
    print(f"  chapter 11 {ch11} frames, unchanged")
    print(f"  chapter 12 {ch12} -> {ch12 + hold_f + fade_f} frames ({ch12 / 60:.2f} s -> {(ch12 + hold_f + fade_f) / 60:.2f} s)")
    print(f"  hold from local frame {ch12} ({ch12 / 60:.2f} s), {hold_f} frames")
    print(f"  fade from local frame {ch12 + hold_f} ({(ch12 + hold_f) / 60:.2f} s), {fade_f} frames")
    print(f"  film {film} -> {film + hold_f + fade_f} frames ({film / 60:.2f} s -> {(film + hold_f + fade_f) / 60:.2f} s), before root's 04_7 delta")
    return [("closing adds 7.0 s", hold_f + fade_f == 420, f"{hold_f} + {fade_f} frames"),
            ("baseline frames", (ch11, ch12, film) == (16550, 9837, 116245), f"{ch11}, {ch12}, {film}")]


def fact_rows(ws):
    root = Path(ws) / "explainer_corpus/logarithm_mysteries"
    facts = (root / "src/facts.rs").read_text()
    m = re.search(r"PROBE_CANDIDATE: model::M2 = \[\[([-\d.]+), ([-\d.]+)\], \[([-\d.]+), ([-\d.]+)\]\]", facts)
    p, q = re.search(r"FLIP_PAIR: \(f64, f64\) = \(([-\d.]+), ([-\d.]+)\)", facts).groups()
    a, b, c, d = (float(x) for x in m.groups())
    oracle = (root / "oracle.py").read_text()
    gap = math.hypot(b * (float(q) - float(p)), c * (float(p) - float(q)))
    return [("sample entries are the fact", (a, b, c, d) == (0.4, 0.6, -0.5, -0.3)
             and 'add("matrix.probe_candidate", "0.4,0.6;-0.5,-0.3")' in oracle,
             f"facts.rs and oracle.py agree: {a} {b} / {c} {d}; card prints them with Rust to_string"),
            ("gap of the sample", f"{gap:.2f}" == "1.17", f"hypot of the two parts {gap:.4f}, shown 1.17")]


def controls(before, cand):
    """Each mutation must fail the named check."""
    def failed(mutant, name):
        return [row for row in source_checks(before, mutant) if row[0] == name and not row[1]]

    def edit(name, old, new):
        out = dict(cand)
        if old not in out[name]:
            raise SystemExit(f"control text missing: {old}")
        out[name] = out[name].replace(old, new, 1)
        return out

    u, w = "unreachable.rs", "what_it_forgets.rs"
    plan = [
        ("a line added above hold 11_3", edit(u, '        kit.say(stage, "11_3")?;', '        // x\n        kit.say(stage, "11_3")?;'), "hold 11_3"),
        ("a word changed inside hold 12_5", edit(w, 'kit.say(stage, "12_5")?;', 'kit.say(stage, "12_5")?; '), "hold 12_5"),
        ("tween 2.6 -> 2.7", edit(u, "tween_leaving(stage, off, 0.0, &sample, 2.6)?;", "tween_leaving(stage, off, 0.0, &sample, 2.7)?;"), "chapter 11 clock lines pinned"),
        ("log A restored", edit(u, r"\text{{a complex logarithm: }} X = ", r"\log A = "), "F2 no log A"),
        ("hold before the card", edit(w, "        fade_in_all(stage, &family, 0.4)?;\n        // the film's end", "        kit.music_breath(stage, 1.0)?;\n        fade_in_all(stage, &family, 0.4)?;\n        // the film's end"), "chapter 12 changed lines"),
        ("old footer restored", edit(u, LIOUVILLE_NEW, LIOUVILLE_OLD), "F1 glosses"),
        ("typed sample", edit(u, "swap_in_sample(&kit, stage, base, probe_alone,", "swap_in_sample(&kit, stage, BASE, probe_alone,"), "M1 sample bound to its fact"),
    ]
    rows = []
    for label, mutant, name in plan:
        rows.append((f"control: {label}", bool(failed(mutant, name)), f"fails {name}"))
    return rows


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--ws")
    parser.add_argument("--verification")
    args = parser.parse_args()
    before = {n: (BUNDLE / f"before-{n}").read_text() for n in FILES}
    cand = {n: (BUNDLE / n).read_text() for n in FILES}
    rows = [(f"baseline {n}", sha(before[n]) == h, h[:16]) for n, h in FILES.items()]
    patch = (BUNDLE / "repair.patch").read_text()
    labels = re.findall(r"^\+\+\+ b/(\S+)", patch, re.M)
    rows.append(("patch touches two files", labels == [f"{CHAPTERS}/{n}" for n in FILES], f"{len(labels)} files"))
    rows.append(("patch rebuilds both candidates", apply_patch(patch, before) == cand, "every context line matched"))
    try:
        apply_patch(patch, {**before, "unreachable.rs": before["unreachable.rs"].replace("BACKGROUND", "BACKDROP", 1)})
        rows.append(("control: patch refuses a changed baseline", False, "applied"))
    except ValueError as e:
        rows.append(("control: patch refuses a changed baseline", True, str(e)))
    if args.ws:
        chapters = Path(args.ws) / CHAPTERS
        rows.extend((f"workspace {n}", (chapters / n).read_text() == before[n], "byte equal to the before copy") for n in FILES)
        held = json.loads((Path(args.ws) / "explainer_corpus/logarithm_mysteries/docs/narration-direction-map-oliver16.json").read_text())
        stored = {r["cue"]: (r["line"], r["marker"], r["interval_sha256"], r["sha256"]) for r in held["visual_holds"]}
        rows.extend((f"map row {cue}", stored.get(cue) == (line, marker, value, FILES[name]), "line, marker, interval, file hash")
                    for name, cue, line, marker, value in HOLDS)
        rows.extend(fact_rows(args.ws))
    rows.extend(source_checks(before, cand))
    rows.extend(width_rows())
    rows.extend(timing_rows(cand, args.verification))
    rows.extend(controls(before, cand))
    secrets = ["/Us" + "ers/", "/pri" + "vate/", "jem" + "anuel"]
    dirty = [p.name for p in BUNDLE.rglob("*") if p.is_file() and p.name != "SHA256SUMS"
             and any(s in p.read_text(errors="replace") for s in secrets)]
    rows.append(("no absolute path or user name in the bundle", not dirty, f"{dirty}"))
    bad = 0
    for name, ok, detail in rows:
        bad += not ok
        print(f"{'PASS' if ok else 'FAIL'} {name}: {detail}")
    print("CANDIDATE HASHES (new file sha256 for the map rows; lines, markers and intervals above are unchanged)")
    for n in FILES:
        print(f"  {n} {FILES[n]} -> {sha(cand[n])}")
    print(f"{len(rows)} checks, {bad} failed. Not compiled, not tested in Cargo, not rendered, not heard.")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
